//! MineWorld's world state: who exists, what state they carry, and how they are connected.
//!
//! This crate holds the authoritative state of a running world and the rules that protect it.
//! It is deliberately small. It knows identity, component storage and the relation graph;
//! it does not know what a job is, what money is, or that people sleep. Those are System Packs,
//! and a world with every system removed is still a valid world made of these types
//! (`INV-12`).
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
pub mod components;
pub mod entities;
pub mod error;
pub mod relations;

#[doc(hidden)]
pub mod macro_support;

pub use access::{OwnedBy, SystemIdentity, WriteAccess, WriteToken};
pub use components::ComponentStore;
pub use entities::{EntityRegistry, EntityRegistrySnapshot};
pub use error::KernelError;
pub use relations::{RelationStore, RelationStoreSnapshot};
