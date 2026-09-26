//! The single error type every fallible kernel operation returns.
//!
//! One enum rather than one per module, for the same reason the contract layer has one: a
//! caller loading a world, replaying an event log or running a system handles these at the same
//! boundary, and each variant carries the offending values as fields so that the caller can
//! react to them rather than parse a message.
//!
//! Every variant here describes a *refusal*, and a refusal never leaves state half-changed.
//! That is a promise of this crate, not an accident of the current implementation: an operation
//! that can fail validates before it mutates, so a caller that handles the error is looking at
//! the state it had before the call.

use mineworld_contracts::{ContractError, EntityId, EntityKey};

use thiserror::Error;

/// Every way a kernel operation can refuse.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum KernelError {
    /// A contract value refused to be built or decoded. The kernel never repairs one; it
    /// reports what the contract layer said.
    #[error(transparent)]
    Contract(#[from] ContractError),

    /// Two entities were created with the same authoring key. Keys are how authored content
    /// refers to entities, so allowing a second one would make the reference ambiguous.
    #[error("entity key '{key}' is already used by entity {existing}")]
    EntityKeyAlreadyUsed {
        /// The key that was offered a second time.
        key: EntityKey,
        /// The entity that already holds it.
        existing: EntityId,
    },

    /// An authoring key was resolved that no entity in this world holds.
    #[error("no entity in this world has the key '{key}'")]
    UnknownEntityKey {
        /// The key that could not be resolved.
        key: EntityKey,
    },

    /// An operation named an entity this world has never allocated.
    #[error("entity {entity} does not exist in this world")]
    UnknownEntity {
        /// The identity that does not exist.
        entity: EntityId,
    },

    /// Identity allocation reached the top of the identifier space. Reported rather than
    /// wrapped: reusing an identity would break every event that already refers to it.
    #[error("this world has allocated every available entity identity")]
    EntityIdSpaceExhausted,

    /// A persisted registry held an entity under an identity other than its own — the record
    /// and its key disagree, so one of them is wrong and the kernel cannot tell which.
    #[error("persisted entity record at {at} carries identity {found}")]
    PersistedEntityIdMismatch {
        /// The identity the record was stored under.
        at: EntityId,
        /// The identity the record itself claims.
        found: EntityId,
    },

    /// A persisted registry would allocate an identity it has already used. Loading it would
    /// break the promise that an identity is never reused.
    #[error(
        "persisted registry would next allocate {next}, which entity {allocated} already holds"
    )]
    PersistedIdWouldBeReused {
        /// The next identity the persisted registry would hand out.
        next: u64,
        /// The entity that already holds it.
        allocated: EntityId,
    },

    /// A persisted registry's counter is below the first identity a world allocates, so loading
    /// it would hand out identity 0 — the value this crate documents as never allocated.
    #[error("persisted registry would next allocate {next}, below the first identity {first}")]
    PersistedIdCounterTooLow {
        /// The next identity the persisted registry would hand out.
        next: u64,
        /// The first identity a world allocates.
        first: u64,
    },

    /// A persisted registry held one authoring key on two entities.
    #[error("persisted registry holds key '{key}' on both entity {first} and entity {second}")]
    PersistedEntityKeyRepeated {
        /// The repeated key.
        key: EntityKey,
        /// The first entity holding it.
        first: EntityId,
        /// The second entity holding it.
        second: EntityId,
    },
}
