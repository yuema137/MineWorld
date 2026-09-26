//! Typed contracts for MineWorld: the identity and state vocabulary that the kernel, the
//! systems, the controllers and the clients all speak.
//!
//! This crate is **pure data plus validation and serialization**. It contains no storage, no
//! dispatch, no scheduling, no persistence and no behavior beyond checking that a value is
//! well formed. Those live in the kernel and in systems, which depend on this crate; nothing
//! here depends back.
//!
//! # Four rules for anything added to this crate
//!
//! **1. No domain concept appears here.** A type called `Employment`, `Money`, `Hunger` or
//! `Conversation` belongs to a System Pack, never to the contract layer. The kernel does not
//! know what a job is, and neither does this vocabulary. What the contract layer provides is
//! the machinery — entities, components, relations — that a system uses to say such things.
//!
//! **2. Collections are order-deterministic.** `BTreeMap`, `BTreeSet` and `Vec` only.
//! `HashMap` and `HashSet` are rejected by `clippy.toml` at the workspace root, because their
//! iteration order would reach serialized state and the event log, and MineWorld requires a
//! fixed seed with fixed inputs to reproduce a run exactly.
//!
//! **3. Every contract type serializes.** History is the event log, and state is rebuilt from
//! it, so a contract type that cannot round-trip through `serde` cannot be persisted or
//! replayed.
//!
//! **4. Validation happens in constructors, and rejects rather than repairs.** A constructor
//! that can fail returns [`Result<_, ContractError>`](ContractError) and is named `new`.
//! Deserialization runs the same check, so a malformed authored file or a corrupted record
//! fails where it is read instead of somewhere later. Nothing is silently normalized.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod ids;

pub use error::{ContractError, IdentifierKind};
pub use ids::{
    ActionId, ComponentTypeId, EntityId, EntityKey, EntityType, EventId, ItemId,
    MAX_IDENTIFIER_LENGTH, OrganizationId, PersonId, PlaceId, ProcessId, RelationTypeId, SystemId,
};
