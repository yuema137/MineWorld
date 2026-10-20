//! Reading a save's history: its fact log as a stream, and what one Person perceived of it.
//!
//! [`SavedHistory`] is the server's `perceived` backfill for a persisted world (`server/PROTOCOL.md`
//! §5.8, `docs/DECISIONS.md` `ARC-43`): the same function `mineworld perceived` runs, over the same
//! save, so that a resumed stream and an offline export agree fact for fact. It is called off the
//! world's thread, and it opens its own connection for each read and drops it before returning — a
//! second reader beside the world thread's writer, which SQLite's WAL mode allows, and no handle that
//! outlives the read, so that on Windows the save can still be moved once the server stops
//! (step-12 §17.14).
//!
//! The log is read as a stream, one sealed block at a time (`ARC-89`, `F-S24-1`): no reader here holds
//! the whole log in memory, only what it keeps of it.

use std::path::{Path, PathBuf};

use mineworld_contracts::{EntityId, EventEnvelope, EventId};
use mineworld_persistence::backend::Facts;
use mineworld_persistence::{Durability, PersistError, PersistenceBackend, SqliteBackend, format};
use mineworld_presence::audience::perceived_by;
use mineworld_server::{HistoryUnavailable, PerceivedHistory};

/// The facts of a save, decoded one at a time, as an infallible iterator for consumers such as
/// [`perceived_by`]: the first error ends the stream and is kept, so that the caller reports it rather
/// than a truncated success.
struct Decoded<'a> {
    rows: Facts<'a>,
    error: Option<PersistError>,
}

impl Iterator for Decoded<'_> {
    type Item = EventEnvelope;

    fn next(&mut self) -> Option<EventEnvelope> {
        if self.error.is_some() {
            return None;
        }
        let decoded = self
            .rows
            .next()?
            .and_then(|row| format::decode(&row.bytes, "fact"));
        match decoded {
            Ok(fact) => Some(fact),
            Err(error) => {
                self.error = Some(error);
                None
            }
        }
    }
}

/// Runs `read` over every fact the save holds, oldest first, streamed from a connection opened for
/// this read and closed before it returns. `read` may stop early. A fact that cannot be read is the
/// error, whatever `read` made of the facts before it.
pub(crate) fn with_saved_facts<T>(
    save: &Path,
    read: impl FnOnce(&mut dyn Iterator<Item = EventEnvelope>) -> T,
) -> Result<T, PersistError> {
    let backend = SqliteBackend::open(save, Durability::ProcessCrash)?;
    let mut facts = Decoded {
        rows: backend.facts_after(None)?,
        error: None,
    };
    let result = read(&mut facts);
    match facts.error {
        Some(error) => Err(error),
        None => Ok(result),
    }
}

/// A persisted world's `perceived` backfill: the save's log, judged by presence's audience rule.
pub struct SavedHistory {
    save: PathBuf,
}

impl SavedHistory {
    /// The history of the save in `save`.
    pub const fn new(save: PathBuf) -> Self {
        Self { save }
    }
}

impl PerceivedHistory for SavedHistory {
    fn perceived(
        &self,
        observer: EntityId,
        since: Option<EventId>,
        through: EventId,
    ) -> Result<Vec<EventEnvelope>, HistoryUnavailable> {
        // The audience rule applies whereabouts from the first fact, so the stream starts at genesis;
        // it stops at `through`, since no later fact is asked for (`ARC-89` D-SRB-8).
        with_saved_facts(&self.save, |facts| {
            perceived_by(
                facts.take_while(|fact| fact.id() <= through),
                observer,
                since,
            )
            .collect()
        })
        .map_err(|error| HistoryUnavailable(format!("the save could not be read: {error}")))
    }
}
