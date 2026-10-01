//! What a world is asked, and what it answered: the entries of the journal (`ARC-25`).

use mineworld_contracts::{ActionIntent, ActionResult, WorldTime};
use mineworld_kernel::{Emission, WorldSnapshot};
use serde::{Deserialize, Serialize};

/// The position of an input in a world's journal: which persisted revision of the world this is.
///
/// Revision 1 is genesis, and each later journaled input is the next one. Monotonic, never reused,
/// and committed before anybody is told it — which is what makes it the *persisted state revision*
/// `docs/MVP.md` §9.1 asks `AC-15` evidence to name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorldRevision(u64);

impl WorldRevision {
    /// The revision a world is created at: genesis.
    pub const GENESIS: Self = Self(1);

    /// A revision number read back from a save or a frame.
    pub const fn from_raw(value: u64) -> Self {
        Self(value)
    }

    /// The number.
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// The revision after this one.
    pub(crate) const fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

impl core::fmt::Display for WorldRevision {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "r{}", self.0)
    }
}

/// One input that moved a world. Three kinds and no fourth, because only three things move a
/// persisted world: coming into existence, being asked to act, and time passing with something due.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorldInput {
    /// The world beginning: the assembled world before genesis — systems, entities, nothing else —
    /// and the facts it begins with, at the instant it begins (`ARC-15`). Recorded so that genesis is
    /// re-run on verification rather than trusted.
    Genesis {
        /// The assembled world, before genesis.
        before: Box<WorldSnapshot>,
        /// The instant the world begins at.
        at: WorldTime,
        /// What is true of it as it begins.
        facts: Vec<Emission>,
    },
    /// A request, whatever the world answered.
    Dispatch {
        /// The request as the world received it, identity and all.
        intent: ActionIntent,
        /// The instant it was dispatched at.
        at: WorldTime,
    },
    /// The clock advanced to `until` and something was due on the way.
    Advance {
        /// Where the clock was moved to.
        until: WorldTime,
    },
}

impl WorldInput {
    /// The instant this input happened at.
    pub const fn at(&self) -> WorldTime {
        match self {
            Self::Genesis { at, .. } | Self::Dispatch { at, .. } => *at,
            Self::Advance { until } => *until,
        }
    }
}

/// What the world answered an input — the part of a journal entry replay must reproduce exactly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// Genesis ran.
    Began,
    /// The answer to a request.
    Dispatched(ActionResult),
    /// How many instants fired and how many entries were skipped.
    Advanced {
        /// Instants fired.
        instants: u64,
        /// Entries skipped because their system was disabled.
        skipped: u64,
    },
    /// A system broke its own contract while the input ran. The kernel is deterministic, so replay
    /// must reproduce the same error — and with it the partial writes the kernel documents.
    Fault(String),
}

/// One journal row: the input and what the world answered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntry {
    /// What the world was asked.
    pub input: WorldInput,
    /// What it answered.
    pub outcome: Outcome,
}
