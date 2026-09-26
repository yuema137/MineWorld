//! Typed contracts for MineWorld: the identity and state vocabulary that the kernel, the
//! systems, the controllers and the clients all speak.
//!
//! This crate is **pure data plus validation and serialization**. It contains no storage, no
//! dispatch, no scheduling, no persistence and no behavior beyond checking that a value is
//! well formed. Those live in the kernel and in systems, which depend on this crate; nothing
//! here depends back.
//!
//! # Five rules for anything added to this crate
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
//! **4. Exactly one type erases a payload per contract family.** [`ComponentRecord`],
//! [`ActionRecord`] and [`EventRecord`] hold their contents in a form this crate cannot
//! interpret, because a database row and a network frame carry bytes rather than Rust types. Each
//! is documented as that boundary, and none can be mislabelled: a record is built from a
//! component, action or event type and is handed back only to code that names the same type —
//! and, for a component, the same schema version. Anywhere else, an untyped payload is a defect.
//!
//! **5. Validation happens in constructors, and rejects rather than repairs.** A constructor
//! that can fail returns [`Result<_, ContractError>`](ContractError) and is named `new`.
//! Deserialization runs the same check, so a malformed authored file or a corrupted record
//! fails where it is read instead of somewhere later. Nothing is silently normalized.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod action;
pub mod component;
pub mod entity;
pub mod error;
pub mod event;
pub mod ids;
pub mod relation;
pub mod time;

pub use action::{
    Action, ActionIntent, ActionRecord, ActionResult, ActionTypeId, Rejection, RejectionCode,
};
pub use component::{Component, ComponentDeclaration, ComponentRecord, ComponentSchemaVersion};
pub use entity::{Entity, LifecycleState, Metadata, Tag, Tags};
pub use error::{ContractError, IdentifierKind};
pub use event::{
    Causation, Event, EventEnvelope, EventRecord, EventTypeId, Provenance, Visibility,
};
pub use ids::{
    ActionId, ComponentTypeId, EntityId, EntityKey, EntityType, EventId, ItemId,
    MAX_IDENTIFIER_LENGTH, OrganizationId, PersonId, PlaceId, ProcessId, RelationTypeId, SystemId,
};
pub use relation::{
    EntityTypeSet, Relation, RelationDirection, RelationEnd, RelationTypeDeclaration, SelfEdges,
};
pub use time::{SimDuration, WorldTime};
