//! Host operations: the lifecycle of a single-player advance, kept beside a save (pr-nightlife N2-I8).
//!
//! An advance runs a sleeping player's night faster than wall time (`ARC-374`). Its lifecycle is **host
//! state**, like the host journal's pacing changes (`ARC-69`): not a fact, not an input, not a snapshot
//! and not a revision. Re-execution never reads it and `verify` never compares it with regenerated
//! bytes; it only checks that the table is coherent, so a save whose operation history cannot have
//! happened is refused by name.
//!
//! ```text
//! advance-started  { operation, seat, adapter, prior_scale }   before the first instant is visited
//! advance-ended    { operation, outcome, reason }              after the last
//! ```
//!
//! The records live in their own table, `host_operations`, created by the first append. A save that
//! never advanced has no such table, and a binary from before this table reads past it as it reads past
//! any table it does not know — no `SAVE_FORMAT` change. The pacing journal's `PacingChange` is left as
//! it is.
//!
//! Each row is encoded with [`crate::format::encode`], so its bytes are the `serde_json` encoding of
//! [`OperationRecord`].

use std::fmt;
use std::num::NonZeroU32;
use std::str::FromStr;

use mineworld_contracts::{EntityKey, WorldTime};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::backend::{HostRow, PersistenceBackend};
use crate::error::PersistError;
use crate::format::encode;
use crate::input::WorldRevision;

/// Which advance: a 128-bit identity the client generates, so a repeated request is recognised rather
/// than run twice (N2-I9). Written on the wire and in the save as 32 lowercase hexadecimal digits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OperationId(u128);

impl OperationId {
    /// The identity with this value.
    pub const fn from_raw(value: u128) -> Self {
        Self(value)
    }

    /// Its value.
    pub const fn raw(self) -> u128 {
        self.0
    }
}

/// Why a written operation identity was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("an operation is 32 hexadecimal digits")]
pub struct MalformedOperationId;

impl FromStr for OperationId {
    type Err = MalformedOperationId;

    /// Exactly 32 hexadecimal digits, either case; nothing else — no sign, no prefix, no whitespace.
    fn from_str(written: &str) -> Result<Self, Self::Err> {
        if written.len() != 32 || !written.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(MalformedOperationId);
        }
        u128::from_str_radix(written, 16)
            .map(Self)
            .map_err(|_| MalformedOperationId)
    }
}

impl fmt::Display for OperationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

impl Serialize for OperationId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for OperationId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let written = String::deserialize(deserializer)?;
        written.parse().map_err(serde::de::Error::custom)
    }
}

/// How an advance ended (N2-I10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdvanceOutcome {
    /// The sleep it ran through ended: the alarm, or any other wake.
    Woke,
    /// The host was asked to stop it. The sleeper is still asleep; the server submitted nothing.
    Cancelled,
    /// The sleeper's connection went: it left, dropped, was kicked or released, or lagged.
    Disconnected,
    /// The host stopped it: a graceful stop, or a request it could not yet execute.
    Stopped,
    /// The process hosting it ended before it did; recorded at the next start (N2-I8).
    Interrupted,
    /// A save that could not be written stopped the world during it.
    Failed,
}

/// The named cause beside an outcome, where one outcome has several.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndReason {
    /// This build admits an advance but does not yet run one (`stopped`; pr-nightlife N2-C2 only).
    NotYetExecutable,
    /// The sleeper's perceived stream fell behind its bound (`disconnected`; N2-I13).
    Lagged,
}

/// One change in an operation's lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationEvent {
    /// An admitted advance, before its first instant.
    AdvanceStarted {
        /// Which.
        operation: OperationId,
        /// The seat whose Person sleeps.
        seat: EntityKey,
        /// The stop adapter it runs until, by name.
        adapter: String,
        /// The scale in force when it started, and again when it ends.
        prior_scale: NonZeroU32,
    },
    /// An advance's end, after its last instant.
    AdvanceEnded {
        /// Which.
        operation: OperationId,
        /// How it ended.
        outcome: AdvanceOutcome,
        /// Why, where the outcome has more than one cause.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<EndReason>,
    },
}

impl OperationEvent {
    /// The operation the event belongs to.
    pub const fn operation(&self) -> OperationId {
        match self {
            Self::AdvanceStarted { operation, .. } | Self::AdvanceEnded { operation, .. } => {
                *operation
            }
        }
    }
}

/// One host-operation record: the event, the world's instant when it happened, and the save's head.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationRecord {
    /// The world's instant when it was written.
    pub at: WorldTime,
    /// The save's head when it was written: orders it against the inputs.
    pub revision: WorldRevision,
    /// What happened.
    pub event: OperationEvent,
}

impl OperationRecord {
    /// The row this record is stored as.
    pub(crate) fn row(&self) -> Result<HostRow, PersistError> {
        Ok(HostRow {
            revision: self.revision,
            at: self.at.seconds(),
            record: encode(self)?,
        })
    }
}

/// The operation that started and has not ended, if any: what a host that starts finds interrupted.
pub fn unfinished(records: &[OperationRecord]) -> Option<OperationId> {
    let mut open = None;
    for record in records {
        match &record.event {
            OperationEvent::AdvanceStarted { operation, .. } => open = Some(*operation),
            OperationEvent::AdvanceEnded { .. } => open = None,
        }
    }
    open
}

/// Reads a save's host operations in record order, refusing a history that cannot have happened: every
/// record decodes and agrees with its row; the revision never decreases and never passes the head; the
/// instant never decreases, except at an `interrupted` end, which states the durable instant the world
/// resumed at; an operation starts once, ends once, after its start, and only one is open at a time. The error
/// names the record by its position from 1. An absent table is an empty history.
pub(crate) fn read_operations(
    backend: &dyn PersistenceBackend,
    head: WorldRevision,
) -> Result<Vec<OperationRecord>, PersistError> {
    let mut records: Vec<OperationRecord> = Vec::new();
    for (index, row) in backend.operation_records()?.iter().enumerate() {
        let seq = index + 1;
        let record: OperationRecord = serde_json::from_slice(&row.record)
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
            // An interrupted end states the durable instant the world resumed at, which may be earlier than
            // an instant the lost process had reached without committing it (N2-I8).
            let interrupted = matches!(
                record.event,
                OperationEvent::AdvanceEnded {
                    outcome: AdvanceOutcome::Interrupted,
                    ..
                }
            );
            if record.at < previous.at && !interrupted {
                return Err(damaged(
                    seq,
                    "its instant is earlier than the record before it",
                ));
            }
        }
        lifecycle(&records, &record.event).map_err(|why| damaged(seq, why))?;
        records.push(record);
    }
    Ok(records)
}

/// Whether `event` may follow `earlier`.
fn lifecycle(earlier: &[OperationRecord], event: &OperationEvent) -> Result<(), &'static str> {
    let operation = event.operation();
    let started = earlier.iter().any(|record| {
        matches!(record.event, OperationEvent::AdvanceStarted { .. })
            && record.event.operation() == operation
    });
    match event {
        OperationEvent::AdvanceStarted { .. } if started => {
            Err("it starts an operation that already started")
        }
        OperationEvent::AdvanceStarted { .. } if unfinished(earlier).is_some() => {
            Err("it starts an operation while another is open")
        }
        OperationEvent::AdvanceEnded { .. } if unfinished(earlier) != Some(operation) => {
            Err("it ends an operation that is not the open one")
        }
        _ => Ok(()),
    }
}

fn damaged(seq: usize, why: &str) -> PersistError {
    PersistError::Damaged {
        detail: format!("host operation record {seq} is not coherent: {why}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The wire form: 32 hex digits round-trip; anything else is refused before it is an identity.
    #[test]
    fn an_operation_id_is_exactly_32_hex_digits() {
        let id: OperationId = "0123456789ABCDEF0123456789abcdef".parse().expect("parses");
        assert_eq!(id.to_string(), "0123456789abcdef0123456789abcdef");
        for refused in [
            "",
            "0123456789abcdef0123456789abcde",
            "0123456789abcdef0123456789abcdef0",
            "+123456789abcdef0123456789abcdef",
            "0x23456789abcdef0123456789abcdef",
            " 123456789abcdef0123456789abcdef",
            "g123456789abcdef0123456789abcdef",
        ] {
            assert_eq!(
                refused.parse::<OperationId>(),
                Err(MalformedOperationId),
                "{refused:?}"
            );
        }
    }
}
