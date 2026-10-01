//! MineWorld's world state: who exists, what state they carry, and how they are connected.
//!
//! This crate holds the authoritative state of a running world and the rules that protect it.
//! It is deliberately small. It knows identity, component storage and the relation graph;
//! it does not know what a job is, what money is, or that people sleep. Those are System Packs,
//! and a world with every system removed is still a valid world made of these types
//! (`INV-12`).
//!
//! # Three stores, and one rule about writing to them
//!
//! ```text
//! entities     EntityRegistry   identity: allocated, monotonic, never reused
//! components   ComponentStore   one table per component type
//! relations    RelationStore    typed edges between any two entities
//! access       who may write what — the part to read first
//! ```
//!
//! # And the systems that write to them
//!
//! ```text
//! system       System: what a system declares, and the six things it is asked to do
//! view         what a running system is handed — reads open, writes gated on ownership
//! registry     which systems a world is composed of, in registration order
//! dispatch     ActionIntent → route → validate → resolve → Event(s) → reduce
//! world        World: the composed whole, and the only issuer of write capability in it
//! ```
//!
//! # And time
//!
//! ```text
//! clock        WorldClock: simulated seconds, only forward, never a wall clock (INV-12)
//! schedule     the (WorldTime, Sequence) queue of deferred facts and process ends (DEP-6)
//! advance      World::advance_to / step: jump to the next due instant, fire it
//! process      Process: state over time, changed only by its owner (INV-3, INV-7)
//! ```
//!
//! # And a world written down
//!
//! ```text
//! snapshot     WorldSnapshot: the whole of a world's state as data; World::snapshot / restore (S5)
//! ```
//!
//! The journal and the fact log that make a snapshot part of a save live one crate up, in
//! `mineworld-persistence`, so that no storage engine enters this crate (`DEP-2`, `ARC-25`).
//!
//! Reads are open and writes are owned. Any system may read any component; a component is
//! written only by the system that owns it, and [`access`] is where that is made the compiler's
//! rule rather than a review convention (`INV-7`). It is also where the edges of the guarantee
//! are documented, which matters more than the guarantee's headline: a reader who knows only
//! that "ownership is enforced" will eventually rely on something that is not true.
//!
//! # Four rules for anything added to this crate
//!
//! **1. No domain concept appears here.** The rule the contract layer states applies with more
//! force one layer up, because the kernel is where the temptation is: a component called
//! `Hunger`, a special case for a `Cafe`, a field only a market town needs. Every one of those
//! belongs to a system.
//!
//! **2. Nothing is ordered by when it happened.** Collections are `BTreeMap`, `BTreeSet` and
//! `Vec`, and iteration follows the key, never insertion (`KD-5`). A world replayed from its
//! event log must produce the same state in the same order, and insertion order is a property
//! of one run rather than of the world (`AC-12`).
//!
//! **3. Identity is allocated, monotonic, and never reused.** [`EntityRegistry`] is the only
//! allocator. It consults no clock and no random source, and a destroyed entity's identity is
//! spent forever, because the event log still refers to it.
//!
//! **4. A refusal changes nothing.** Every operation that returns
//! [`Result<_, KernelError>`](KernelError) validates before it mutates, so a caller that
//! handles the error is looking at exactly the state it had before the call. No partial write,
//! no half-applied change.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod access;
pub mod advance;
pub mod clock;
pub mod components;
pub mod dispatch;
pub mod entities;
pub mod error;
pub mod process;
pub mod registry;
pub mod relations;
pub mod schedule;
pub mod snapshot;
pub mod system;
pub mod view;
pub mod world;

#[doc(hidden)]
pub mod macro_support;

pub use access::{OwnedBy, SystemIdentity, WriteAccess, WriteToken};
pub use advance::Advanced;
pub use clock::WorldClock;
pub use components::ComponentStore;
pub use dispatch::{CASCADE_DEPTH_LIMIT, Dispatched};
pub use entities::{EntityRegistry, EntityRegistrySnapshot};
pub use error::KernelError;
pub use process::{
    InterruptOutcome, InterruptRequest, Interruptibility, Process, ProcessKind, ProcessPhase,
    ProcessStart, ProcessStore,
};
pub use registry::SystemRegistry;
pub use relations::{RelationStore, RelationStoreSnapshot};
pub use schedule::{ScheduleSnapshot, Scheduled, ScheduledEntry, Sequence};
pub use snapshot::{InstalledSystemRecord, WorldSnapshot};
pub use system::{Deferral, Emission, System, SystemDeclaration, SystemVersion};
pub use view::{Declarations, WorldRead, WorldView};
pub use world::World;
