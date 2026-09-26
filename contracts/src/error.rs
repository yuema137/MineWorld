//! The single error type every validating constructor in this crate returns.
//!
//! One enum rather than one error type per module: a caller loading a World Pack or decoding a
//! persisted record handles all of these at the same boundary, and each variant carries the
//! offending values as fields so that the caller can react to them, rather than a
//! pre-formatted message it would have to parse.

use core::fmt;

use thiserror::Error;

use crate::ids::{EntityId, EntityType};

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
}

impl fmt::Display for IdentifierKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::EntityKey => "entity key",
            Self::SystemId => "system id",
            Self::ComponentTypeId => "component type id",
            Self::RelationTypeId => "relation type id",
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
}
