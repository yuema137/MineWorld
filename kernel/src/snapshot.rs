//! A world's whole state, written down: what a save stores and a restart restores.
//!
//! [`WorldSnapshot`] composes the persistence shapes the stores already had — the
//! [`EntityRegistrySnapshot`], the [`RelationStoreSnapshot`] and S4's [`ScheduleSnapshot`] — with the
//! two things only this layer could add: every component row as a [`ComponentRecord`], and the
//! composition the state belongs to. [`World::snapshot`](crate::World::snapshot) takes one;
//! [`World::restore`](crate::World::restore) puts one back into a freshly composed world.
//!
//! # What a snapshot is, and is not
//!
//! It is a checkpoint of a world's state at one moment, and nothing more authoritative than that.
//! History is the fact log, and a world's state is the kernel's pipeline applied to its recorded
//! inputs (`docs/DECISIONS.md` `ARC-25`); a snapshot is how a restart avoids re-running all of them,
//! and it is checked against what re-running them produces.
//!
//! # Why restoring writes no component through a system
//!
//! A component is written only by its owning system (`INV-7`). Restoring does not write one: it
//! replaces whole stores with stores decoded from records, each row decoded by the very component
//! type its owner declared, into a world whose composition is checked to be the one the snapshot was
//! taken of. No [`WriteToken`](crate::WriteToken) is minted or used, and nothing a system could not
//! already have written appears — the snapshot is that system's own state, written down and read back.
//!
//! # Refused, never guessed
//!
//! Every restore validates everything before replacing anything (kernel rule 4): the composition,
//! every component record (its table, its schema version, its entity, its decoding, its uniqueness),
//! the relation declarations and every edge's endpoints, the schedule and the processes (S4's
//! checks). A snapshot that could not have come from this world is refused by name.

use mineworld_contracts::ComponentRecord;
use serde::{Deserialize, Serialize};

use crate::entities::EntityRegistrySnapshot;
use crate::relations::RelationStoreSnapshot;
use crate::schedule::ScheduleSnapshot;
use crate::system::SystemDeclaration;

/// One installed system as a snapshot records it: what it declared — name, version, dependencies,
/// owned components, actions, events — and whether it was enabled.
///
/// Listed in registration order, because registration order is reduction order (`BD-4`): the same
/// systems installed in another order are another world.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledSystemRecord {
    /// What the system declared at installation.
    pub declaration: SystemDeclaration,
    /// Whether it was in the pipeline.
    pub enabled: bool,
}

/// The whole of a world's state, as data.
///
/// Plain data with public fields, like the snapshot shapes it composes, because it is a persistence
/// format rather than a live object: [`World::restore`](crate::World::restore) refuses one that could
/// not have come from the world it is restored into, whoever built it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldSnapshot {
    /// The systems the state belongs to, in registration order.
    pub composition: Vec<InstalledSystemRecord>,
    /// Every entity and the identity counter.
    pub entities: EntityRegistrySnapshot,
    /// Every component row: tables in component type name order, rows in entity order.
    pub components: Vec<ComponentRecord>,
    /// The declared edge types and every edge.
    pub relations: RelationStoreSnapshot,
    /// The clock, the schedule, the processes and the event and process counters.
    pub time: ScheduleSnapshot,
    /// Whether the clock had been given an instant. A world that has not begun yet accepts any
    /// first instant, and one that has does not; the snapshot keeps the difference.
    pub clock_started: bool,
    /// Whether the world had run — dispatched or advanced. A world that has run refuses genesis.
    pub ran: bool,
}
