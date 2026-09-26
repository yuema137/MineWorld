//! The single error type every validating constructor in this crate returns.
//!
//! One enum rather than one error type per module: a caller loading a World Pack or decoding a
//! persisted record handles all of these at the same boundary, and each variant carries the
//! offending values as fields so that the caller can react to them, rather than a
//! pre-formatted message it would have to parse.

use core::fmt;

use thiserror::Error;

use crate::component::ComponentSchemaVersion;
use crate::entity::LifecycleState;
use crate::ids::{ComponentTypeId, EntityId, EntityType};

/// Which validated textual identifier a [`ContractError`] is describing.
///
/// Every identifier in this crate obeys the same character rule, so the error variants are
/// shared and this enum says which kind of name was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IdentifierKind {
    /// An [`crate::ids::EntityKey`]: the authoring name of an entity in a World Pack.
    EntityKey,
    /// A [`crate::ids::SystemId`].
    SystemId,
    /// A [`crate::ids::ComponentTypeId`].
    ComponentTypeId,
    /// A [`crate::ids::RelationTypeId`].
    RelationTypeId,
    /// A [`crate::entity::Tag`]: one semantic label on an entity.
    Tag,
}

impl fmt::Display for IdentifierKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::EntityKey => "entity key",
            Self::SystemId => "system id",
            Self::ComponentTypeId => "component type id",
            Self::RelationTypeId => "relation type id",
            Self::Tag => "tag",
        };
        f.write_str(name)
    }
}

/// Every way a contract value can fail to be constructed or decoded.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContractError {
    /// A textual identifier was empty.
    #[error("an {kind} must not be empty")]
    IdentifierEmpty {
        /// Which identifier was rejected.
        kind: IdentifierKind,
    },

    /// A textual identifier exceeded [`crate::ids::MAX_IDENTIFIER_LENGTH`].
    #[error("an {kind} must be at most {max} bytes long, but this one is {length}")]
    IdentifierTooLong {
        /// Which identifier was rejected.
        kind: IdentifierKind,
        /// The rejected length, in bytes.
        length: usize,
        /// The permitted maximum, in bytes.
        max: usize,
    },

    /// A textual identifier contained a character outside the permitted set.
    #[error(
        "an {kind} may contain only lowercase ASCII letters, digits, '-' and '_', \
         but this one contains {character:?} at byte {position}"
    )]
    IdentifierIllegalCharacter {
        /// Which identifier was rejected.
        kind: IdentifierKind,
        /// The first offending character.
        character: char,
        /// Its byte offset in the rejected value.
        position: usize,
    },

    /// A textual identifier began or ended with a separator.
    #[error("an {kind} must not begin or end with {character:?}")]
    IdentifierSeparatorAtEdge {
        /// Which identifier was rejected.
        kind: IdentifierKind,
        /// The separator found at the beginning or the end.
        character: char,
    },

    /// A typed entity reference was constructed for an entity of a different type.
    #[error("entity {entity} is of type {actual}, so it cannot be referenced as {expected}")]
    EntityTypeMismatch {
        /// The entity that was referenced.
        entity: EntityId,
        /// The type the reference requires.
        expected: EntityType,
        /// The type the entity actually has.
        actual: EntityType,
    },

    /// A lifecycle change the state machine does not permit, such as anything at all out of
    /// `Destroyed`.
    #[error("entity {entity} cannot move from {from} to {to}")]
    IllegalLifecycleTransition {
        /// The entity whose lifecycle was to change.
        entity: EntityId,
        /// The state it is in.
        from: LifecycleState,
        /// The state it was asked to move to.
        to: LifecycleState,
    },

    /// A component record was read as a component type it was not written for.
    #[error("a {actual} record cannot be read as {expected}")]
    ComponentTypeMismatch {
        /// The component type the caller asked for.
        expected: ComponentTypeId,
        /// The component type the record was written from.
        actual: ComponentTypeId,
    },

    /// A component record was written by a newer schema than the reading code knows. Refused
    /// rather than guessed at: a newer schema may carry fields this code would drop.
    #[error("{component_type} record is {record}, newer than the {supported} this code reads")]
    ComponentSchemaTooNew {
        /// The component type being read.
        component_type: ComponentTypeId,
        /// The version the record was written against.
        record: ComponentSchemaVersion,
        /// The version the reading code supports.
        supported: ComponentSchemaVersion,
    },

    /// A component record predates the reading code's schema. Not a corruption: it is ordinary
    /// history, and a migration has to bring it forward rather than the reader assuming.
    #[error("{component_type} record is {record}, older than the {supported} this code reads")]
    ComponentSchemaOutdated {
        /// The component type being read.
        component_type: ComponentTypeId,
        /// The version the record was written against.
        record: ComponentSchemaVersion,
        /// The version the reading code supports.
        supported: ComponentSchemaVersion,
    },
}
