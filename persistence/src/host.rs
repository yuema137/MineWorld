//! The host journal: how a persisted world's pacing was changed, and at what instant (`ARC-69`).
//!
//! A host records a pacing change beside the save, never inside it: a pause, a resume, a rescale and the
//! start and stop of a hosted world. A record is **not a fact, not an input, not a snapshot and not a
//! revision** (`INV-TW-9`). Re-execution never reads it, and `verify` never compares it with regenerated
//! bytes (`ARC-25`); it only checks that the journal is coherent, so that a save whose pacing history cannot
//! have happened is refused by name.
//!
//! ```text
//! at        the host clock's reading when the change was made, in whole world seconds
//! revision  the save's head when the record was written: orders it against the inputs
//! change     what changed (PacingChange)
//! ```
//!
//! Encoded with [`crate::format::encode`] like every other row, so the bytes are the `serde_json` encoding
//! of [`HostRecord`]. The table holding the records is created lazily by the first append, so a save that
//! is never hosted has exactly the tables it had before this journal existed.

use std::num::NonZeroU32;

use mineworld_contracts::WorldTime;
use serde::{Deserialize, Serialize};

use crate::backend::{HostRow, PersistenceBackend};
use crate::error::PersistError;
use crate::format::encode;
use crate::input::WorldRevision;

/// One pacing change, as the host journal holds it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostRecord {
    /// The host clock's reading when the change was made.
    pub at: WorldTime,
    /// The save's head when the record was written.
    pub revision: WorldRevision,
    /// What changed.
    pub change: PacingChange,
}

/// What a host did to a world's pacing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PacingChange {
    /// The host started a persisted world: its scale, and whether its clock starts paused.
    Started {
        /// World seconds per wall second in force from the start.
        scale: NonZeroU32,
        /// Whether the clock starts paused. Always `false` in this version: a restarted world runs
        /// (`ARC-69` item 8), so a pause is never carried across a restart.
        paused: bool,
    },
    /// The host paused the clock.
    Paused,
    /// The host resumed the clock.
    Resumed,
    /// The host changed the scale while the world runs (or is paused).
    Scaled {
        /// The scale now in force.
        to: NonZeroU32,
    },
    /// The host shut the world down gracefully, after its checkpoint.
    Stopped,
}

impl HostRecord {
    /// The row this record is stored as.
    pub(crate) fn row(&self) -> Result<HostRow, PersistError> {
        Ok(HostRow {
            revision: self.revision,
            at: self.at.seconds(),
            record: encode(self)?,
        })
    }
}

/// Reads a save's host journal in record order, refusing a journal that cannot have happened
/// (`ARC-69` item 7): every record must decode and agree with its row; the revision and the instant
/// each never decrease down the journal, restarts included; and no record names a revision past the
/// head. The error names the record by its sequence number, which is its position from 1 — the table
/// is only ever appended to.
///
/// An absent journal is an empty one: a save that was never hosted has none.
pub(crate) fn read_journal(
    backend: &dyn PersistenceBackend,
    head: WorldRevision,
) -> Result<Vec<HostRecord>, PersistError> {
    let mut records: Vec<HostRecord> = Vec::new();
    for (index, row) in backend.host_records()?.iter().enumerate() {
        let seq = index + 1;
        let record: HostRecord = serde_json::from_slice(&row.record)
            .map_err(|error| damaged(seq, &format!("it does not decode: {error}")))?;
        if record.at.seconds() != row.at || record.revision != row.revision {
            return Err(damaged(seq, "its row and its record disagree"));
        }
        if record.revision > head {
            return Err(damaged(
                seq,
                &format!(
                    "it names revision {}, past the head {head}",
                    record.revision.raw()
                ),
            ));
        }
        if let Some(previous) = records.last() {
            if record.revision < previous.revision {
                return Err(damaged(
                    seq,
                    "its revision is earlier than the record before it",
                ));
            }
            if record.at < previous.at {
                return Err(damaged(
                    seq,
                    &format!(
                        "its instant {} is earlier than the record before it at {}",
                        record.at.seconds(),
                        previous.at.seconds()
                    ),
                ));
            }
        }
        records.push(record);
    }
    Ok(records)
}

fn damaged(seq: usize, why: &str) -> PersistError {
    PersistError::Damaged {
        detail: format!("host journal record {seq} is not coherent: {why}"),
    }
}
