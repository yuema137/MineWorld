//! Every way creating, resuming, driving or verifying a persisted world can refuse.

use mineworld_contracts::SystemId;
use mineworld_kernel::{KernelError, SystemVersion};
use thiserror::Error;

use crate::input::WorldRevision;

/// Why a save could not be created, opened, written or trusted.
///
/// Refusals are by name and carry what they found. None of them is repaired: a save that does not
/// fit the running code, or whose history does not reproduce, is refused (`ARC-25`, step-06 I-4, I-5).
#[derive(Debug, Error)]
pub enum PersistError {
    /// A kernel operation refused or a system broke its contract. When this comes out of
    /// [`PersistentWorld::dispatch`](crate::PersistentWorld::dispatch) or
    /// [`advance_to`](crate::PersistentWorld::advance_to), the fault has been journaled as that
    /// input's outcome and the world goes on; anywhere else it is a refusal.
    #[error(transparent)]
    Kernel(#[from] KernelError),

    /// The save was written by a newer format than this code reads.
    #[error("the save is format {saved}; this code reads format {supported}, an older one")]
    SaveFormatTooNew {
        /// The save's format.
        saved: u32,
        /// The format this code reads and writes.
        supported: u32,
    },

    /// The save was written by an older format than this code reads. No migration exists yet.
    #[error("the save is format {saved}; this code reads format {supported} and does not migrate")]
    SaveFormatOutdated {
        /// The save's format.
        saved: u32,
        /// The format this code reads and writes.
        supported: u32,
    },

    /// The world being resumed is not composed as the saved one was: another system, another
    /// declaration, another enabled state, or another registration order.
    #[error(
        "the save's composition differs at position {position}: saved {saved:?}, running {running:?}"
    )]
    CompositionDiffers {
        /// The registration position of the first difference.
        position: usize,
        /// The system the save has there.
        saved: Option<SystemId>,
        /// The system the running world has there.
        running: Option<SystemId>,
    },

    /// The save was written by a newer version of a system than the one running.
    #[error("system '{system}' is {running} here, but the save was written by {saved}")]
    PersistedSystemTooNew {
        /// The system.
        system: SystemId,
        /// The version that wrote the save.
        saved: SystemVersion,
        /// The version running now.
        running: SystemVersion,
    },

    /// The save was written by an older version of a system than the one running. No migration
    /// exists yet.
    #[error("system '{system}' is {running} here, but the save was written by the older {saved}")]
    PersistedSystemOutdated {
        /// The system.
        system: SystemId,
        /// The version that wrote the save.
        saved: SystemVersion,
        /// The version running now.
        running: SystemVersion,
    },

    /// Re-executing a journaled input did not reproduce what the save recorded.
    #[error("replay diverged at revision {revision}: {detail}")]
    ReplayDiverged {
        /// The first revision that did not reproduce.
        revision: WorldRevision,
        /// What differed.
        detail: String,
    },

    /// A stored snapshot is not the state the history produces at its revision.
    #[error("the snapshot at revision {revision} is not the state the history produces there")]
    SnapshotDisagreesWithHistory {
        /// The snapshot's revision.
        revision: WorldRevision,
    },

    /// A commit failed after the world had applied the input, so the world in memory is ahead of
    /// its save. Every later input is refused: continuing would tell clients about revisions that
    /// do not exist on disk.
    #[error("the world is ahead of its save: revision {revision} could not be committed ({cause})")]
    WorldAheadOfSave {
        /// The revision that could not be committed.
        revision: WorldRevision,
        /// Why the commit failed.
        cause: String,
    },

    /// A save already exists where a new one was to be created.
    #[error("a save already exists at {path}")]
    SaveExists {
        /// Where.
        path: String,
    },

    /// No save exists where one was to be opened.
    #[error("no save exists at {path}")]
    NoSave {
        /// Where.
        path: String,
    },

    /// A save's own records are inconsistent in a way no write of this crate produces: a missing
    /// manifest, a journal with a gap, a row that does not decode.
    #[error("the save is damaged: {detail}")]
    Damaged {
        /// What was found.
        detail: String,
    },

    /// The storage engine refused.
    #[error("storage: {detail}")]
    Storage {
        /// What the engine said.
        detail: String,
    },
}
