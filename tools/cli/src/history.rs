//! Reading a save's history: its whole fact log, and what one Person perceived of it.
//!
//! [`SavedHistory`] is the server's `perceived` backfill for a persisted world (`server/PROTOCOL.md`
//! §5.8, `docs/DECISIONS.md` `ARC-43`): the same function `mineworld perceived` runs, over the same
//! save, so that a resumed stream and an offline export agree fact for fact. It is called off the
//! world's thread, and it opens its own connection for each read and drops it before returning — a
//! second reader beside the world thread's writer, which SQLite's WAL mode allows, and no handle that
//! outlives the read, so that on Windows the save can still be moved once the server stops
//! (step-12 §17.14).

use std::path::{Path, PathBuf};

use mineworld_contracts::{EntityId, EventEnvelope, EventId};
use mineworld_persistence::{Durability, PersistError, PersistenceBackend, SqliteBackend, format};
use mineworld_presence::audience::perceived_by;
use mineworld_server::{HistoryUnavailable, PerceivedHistory};

/// Every fact a save holds, oldest first, from a connection opened for this read and closed before
/// it returns.
pub(crate) fn saved_facts(save: &Path) -> Result<Vec<EventEnvelope>, PersistError> {
    let backend = SqliteBackend::open(save, Durability::ProcessCrash)?;
    let all = usize::try_from(i64::MAX).unwrap_or(usize::MAX);
    let facts = backend
        .last_facts(all)?
        .iter()
        .map(|fact| format::decode(&fact.bytes, "fact"))
        .collect();
    drop(backend);
    facts
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
        let facts = saved_facts(&self.save)
            .map_err(|error| HistoryUnavailable(format!("the save could not be read: {error}")))?;
        Ok(perceived_by(facts, observer, since)
            .take_while(|fact| fact.id() <= through)
            .collect())
    }
}
