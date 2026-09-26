//! Identity: the vocabulary every other contract type is written in.
//!
//! Three kinds of identity live here, and they are deliberately not interchangeable.
//!
//! **Opaque runtime identity**, allocated by the kernel while a world runs:
//! [`EntityId`], [`EventId`], [`ActionId`], [`ProcessId`]. Compact, ordered, and meaningless
//! outside the world that allocated it.
//!
//! **Authoring identity**, written by a human and stable across runs: [`EntityKey`] for an
//! entity in a World Pack, and the declaration names [`SystemId`], [`ComponentTypeId`] and
//! [`RelationTypeId`]. A World Pack refers to `alice`; the runtime resolves that to an
//! [`EntityId`] when it loads the pack.
//!
//! **Typed entity references**: [`PersonId`], [`PlaceId`], [`ItemId`], [`OrganizationId`].
//! These carry, in the type system, the [`EntityType`] they were checked against, so that a
//! place cannot be passed where a person is required.
//!
//! Every validated textual identifier obeys one rule: 1 to [`MAX_IDENTIFIER_LENGTH`] bytes of
//! lowercase ASCII letters, digits, `-` and `_`, neither beginning nor ending with `-` or `_`.
//! A rejected value is never normalized — silently lower-casing or trimming an authored name
//! would let two World Packs that read differently resolve to the same entity.
//!
//! Nothing here is random or time-derived, and no identifier carries arithmetic. Allocation is
//! the kernel's responsibility and arrives with the kernel, so this crate cannot silently wrap
//! at the top of the identifier space, and two runs of the same seeded world cannot differ
//! because of identity.

use core::fmt;

use serde::{Deserialize, Serialize};

use crate::error::{ContractError, IdentifierKind};

/// Maximum length, in bytes, of every validated textual identifier in this crate.
pub const MAX_IDENTIFIER_LENGTH: usize = 64;

/// The one character rule shared by every validated textual identifier in this crate: 1 to
/// [`MAX_IDENTIFIER_LENGTH`] bytes of lowercase ASCII letters, digits, `-` and `_`, neither
/// beginning nor ending with `-` or `_`.
///
/// The set is the one the specification's own examples already use — `lakewood`,
/// `modern-life`, `conversation`, `pacific_northwest` in `docs/MODULE_SPEC.md` §4 — and is
/// restricted enough that an identifier can appear in a file name, a URL and a database key
/// without escaping. A rejected value is never normalized: silently lower-casing or trimming
/// an authored name would make two World Packs that read differently resolve to one entity.
fn validate_identifier(kind: IdentifierKind, value: &str) -> Result<(), ContractError> {
    if value.is_empty() {
        return Err(ContractError::IdentifierEmpty { kind });
    }
    if value.len() > MAX_IDENTIFIER_LENGTH {
        return Err(ContractError::IdentifierTooLong {
            kind,
            length: value.len(),
            max: MAX_IDENTIFIER_LENGTH,
        });
    }
    if let Some((position, character)) = value
        .char_indices()
        .find(|(_, character)| !is_legal_identifier_character(*character))
    {
        return Err(ContractError::IdentifierIllegalCharacter {
            kind,
            character,
            position,
        });
    }

    // Safe to index: the value is non-empty and, after the character check above, ASCII.
    let bytes = value.as_bytes();
    for edge in [bytes[0], bytes[bytes.len() - 1]] {
        if edge == b'-' || edge == b'_' {
            return Err(ContractError::IdentifierSeparatorAtEdge {
                kind,
                character: char::from(edge),
            });
        }
    }

    Ok(())
}

fn is_legal_identifier_character(character: char) -> bool {
    character.is_ascii_lowercase()
        || character.is_ascii_digit()
        || character == '-'
        || character == '_'
}

// ---------------------------------------------------------------------------------------------
// Opaque runtime identity
// ---------------------------------------------------------------------------------------------

/// Opaque runtime identity of an entity: stable, unique, and never reused.
///
/// Allocation belongs to the kernel, not to this crate. [`EntityId::from_raw`] exists so that
/// the allocator and the persistence layer can rebuild an identity they already own; it is not
/// a way to invent one. There is deliberately no `Default`, no arithmetic and no increment
/// here: an allocator that could reuse or overflow an identity is a kernel concern that must
/// stay visible there rather than hide behind a convenience in a contract type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EntityId(u64);

impl EntityId {
    /// Rebuilds the identity the kernel allocated or persistence recorded.
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// The underlying value, for persistence, wire encoding and diagnostics.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identity of a recorded event in the event log.
///
/// The event payload contracts arrive with the action and event layer; the identity type lives
/// here so that every layer wraps the same one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventId(u64);

impl EventId {
    /// Rebuilds the identity the kernel allocated or persistence recorded.
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// The underlying value, for persistence, wire encoding and diagnostics.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identity of a submitted action, used to correlate an intent with its outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActionId(u64);

impl ActionId {
    /// Rebuilds the identity the kernel allocated or persistence recorded.
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// The underlying value, for persistence, wire encoding and diagnostics.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ActionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identity of a running process — something that takes simulated time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProcessId(u64);

impl ProcessId {
    /// Rebuilds the identity the kernel allocated or persistence recorded.
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// The underlying value, for persistence, wire encoding and diagnostics.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ProcessId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---------------------------------------------------------------------------------------------
// Authoring identity
// ---------------------------------------------------------------------------------------------

/// The authoring name of an entity: the `alice` in `people/alice.yaml`.
///
/// A key is stable across runs and readable by a human, which is what a World Pack needs; an
/// [`EntityId`] is compact and allocated, which is what the runtime needs. They are separate
/// types because conflating them would force either string comparison in the runtime's hot
/// paths or unstable references in authored content.
///
/// Deserialization validates, so a malformed key in an authored file is an error at load time
/// rather than a value that fails somewhere later.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct EntityKey(String);

impl EntityKey {
    /// Validates an authored key against the identifier rule stated in this module's
    /// documentation.
    pub fn new(value: impl Into<String>) -> Result<Self, ContractError> {
        let value = value.into();
        validate_identifier(IdentifierKind::EntityKey, &value)?;
        Ok(Self(value))
    }

    /// The key as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for EntityKey {
    type Error = ContractError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for EntityKey {
    type Error = ContractError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<EntityKey> for String {
    fn from(value: EntityKey) -> Self {
        value.0
    }
}

impl fmt::Display for EntityKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The declared name of a system, and therefore the name of a component's single writer.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct SystemId(String);

impl SystemId {
    /// Validates a declared system name against the identifier rule stated in this module's
    /// documentation.
    pub fn new(value: impl Into<String>) -> Result<Self, ContractError> {
        let value = value.into();
        validate_identifier(IdentifierKind::SystemId, &value)?;
        Ok(Self(value))
    }

    /// The name as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for SystemId {
    type Error = ContractError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for SystemId {
    type Error = ContractError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<SystemId> for String {
    fn from(value: SystemId) -> Self {
        value.0
    }
}

impl fmt::Display for SystemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The declared name of a component type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ComponentTypeId(String);

impl ComponentTypeId {
    /// Validates a declared component type name against the identifier rule stated in this
    /// module's documentation.
    pub fn new(value: impl Into<String>) -> Result<Self, ContractError> {
        let value = value.into();
        validate_identifier(IdentifierKind::ComponentTypeId, &value)?;
        Ok(Self(value))
    }

    /// The name as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ComponentTypeId {
    type Error = ContractError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for ComponentTypeId {
    type Error = ContractError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<ComponentTypeId> for String {
    fn from(value: ComponentTypeId) -> Self {
        value.0
    }
}

impl fmt::Display for ComponentTypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The declared name of a relation type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RelationTypeId(String);

impl RelationTypeId {
    /// Validates a declared relation type name against the identifier rule stated in this
    /// module's documentation.
    pub fn new(value: impl Into<String>) -> Result<Self, ContractError> {
        let value = value.into();
        validate_identifier(IdentifierKind::RelationTypeId, &value)?;
        Ok(Self(value))
    }

    /// The name as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for RelationTypeId {
    type Error = ContractError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for RelationTypeId {
    type Error = ContractError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<RelationTypeId> for String {
    fn from(value: RelationTypeId) -> Self {
        value.0
    }
}

impl fmt::Display for RelationTypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ---------------------------------------------------------------------------------------------
// Entity types and typed references
// ---------------------------------------------------------------------------------------------

/// The four kinds of thing that can exist (`docs/CORE_CONCEPTS.md` §3).
///
/// This list is closed on purpose. A pack that introduces vehicles or oxygen tanks adds tags
/// and components to one of these four; it does not add a variant here, because the kernel's
/// taxonomy is what every system and every client is written against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    /// An individual. Not an agent: what decides its actions is a controller, bound
    /// separately and replaceable at runtime.
    Person,
    /// A location.
    Place,
    /// A thing that can be held, moved, owned or consumed.
    Item,
    /// A group that can act as one: a company, a household, a guild.
    Organization,
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Person => "person",
            Self::Place => "place",
            Self::Item => "item",
            Self::Organization => "organization",
        };
        f.write_str(name)
    }
}

/// The serialized form of a typed entity reference.
///
/// A typed reference is written out with the [`EntityType`] it was checked against, so that
/// reading one back is the same checked construction as building one in code. Without the tag
/// any integer in a saved world could be deserialized as a [`PersonId`], and the type-level
/// guarantee would hold only until the first round trip.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct TypedEntityRef {
    entity: EntityId,
    entity_type: EntityType,
}

/// A reference to an entity that has been checked to be an [`EntityType::Person`].
///
/// Converts to [`EntityId`] for free, and can be produced from one only through
/// [`PersonId::new`], which must be told the entity's actual type. The wrapped field is
/// private, so no code outside this module can relabel an arbitrary [`EntityId`] as a person.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(into = "TypedEntityRef", try_from = "TypedEntityRef")]
pub struct PersonId(EntityId);

impl PersonId {
    /// The entity type this reference requires.
    pub const ENTITY_TYPE: EntityType = EntityType::Person;

    /// Checks `entity_type` against [`PersonId::ENTITY_TYPE`] and produces the reference, or
    /// reports the mismatch.
    pub fn new(entity: EntityId, entity_type: EntityType) -> Result<Self, ContractError> {
        if entity_type == Self::ENTITY_TYPE {
            Ok(Self(entity))
        } else {
            Err(ContractError::EntityTypeMismatch {
                entity,
                expected: Self::ENTITY_TYPE,
                actual: entity_type,
            })
        }
    }

    /// The underlying entity identity.
    pub const fn entity_id(self) -> EntityId {
        self.0
    }
}

impl From<PersonId> for EntityId {
    fn from(value: PersonId) -> Self {
        value.0
    }
}

impl From<PersonId> for TypedEntityRef {
    fn from(value: PersonId) -> Self {
        Self {
            entity: value.0,
            entity_type: PersonId::ENTITY_TYPE,
        }
    }
}

impl TryFrom<TypedEntityRef> for PersonId {
    type Error = ContractError;

    fn try_from(value: TypedEntityRef) -> Result<Self, Self::Error> {
        Self::new(value.entity, value.entity_type)
    }
}

impl fmt::Display for PersonId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", Self::ENTITY_TYPE, self.0)
    }
}

/// A reference to an entity that has been checked to be an [`EntityType::Place`].
///
/// Converts to [`EntityId`] for free, and can be produced from one only through
/// [`PlaceId::new`], which must be told the entity's actual type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(into = "TypedEntityRef", try_from = "TypedEntityRef")]
pub struct PlaceId(EntityId);

impl PlaceId {
    /// The entity type this reference requires.
    pub const ENTITY_TYPE: EntityType = EntityType::Place;

    /// Checks `entity_type` against [`PlaceId::ENTITY_TYPE`] and produces the reference, or
    /// reports the mismatch.
    pub fn new(entity: EntityId, entity_type: EntityType) -> Result<Self, ContractError> {
        if entity_type == Self::ENTITY_TYPE {
            Ok(Self(entity))
        } else {
            Err(ContractError::EntityTypeMismatch {
                entity,
                expected: Self::ENTITY_TYPE,
                actual: entity_type,
            })
        }
    }

    /// The underlying entity identity.
    pub const fn entity_id(self) -> EntityId {
        self.0
    }
}

impl From<PlaceId> for EntityId {
    fn from(value: PlaceId) -> Self {
        value.0
    }
}

impl From<PlaceId> for TypedEntityRef {
    fn from(value: PlaceId) -> Self {
        Self {
            entity: value.0,
            entity_type: PlaceId::ENTITY_TYPE,
        }
    }
}

impl TryFrom<TypedEntityRef> for PlaceId {
    type Error = ContractError;

    fn try_from(value: TypedEntityRef) -> Result<Self, Self::Error> {
        Self::new(value.entity, value.entity_type)
    }
}

impl fmt::Display for PlaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", Self::ENTITY_TYPE, self.0)
    }
}

/// A reference to an entity that has been checked to be an [`EntityType::Item`].
///
/// Converts to [`EntityId`] for free, and can be produced from one only through
/// [`ItemId::new`], which must be told the entity's actual type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(into = "TypedEntityRef", try_from = "TypedEntityRef")]
pub struct ItemId(EntityId);

impl ItemId {
    /// The entity type this reference requires.
    pub const ENTITY_TYPE: EntityType = EntityType::Item;

    /// Checks `entity_type` against [`ItemId::ENTITY_TYPE`] and produces the reference, or
    /// reports the mismatch.
    pub fn new(entity: EntityId, entity_type: EntityType) -> Result<Self, ContractError> {
        if entity_type == Self::ENTITY_TYPE {
            Ok(Self(entity))
        } else {
            Err(ContractError::EntityTypeMismatch {
                entity,
                expected: Self::ENTITY_TYPE,
                actual: entity_type,
            })
        }
    }

    /// The underlying entity identity.
    pub const fn entity_id(self) -> EntityId {
        self.0
    }
}

impl From<ItemId> for EntityId {
    fn from(value: ItemId) -> Self {
        value.0
    }
}

impl From<ItemId> for TypedEntityRef {
    fn from(value: ItemId) -> Self {
        Self {
            entity: value.0,
            entity_type: ItemId::ENTITY_TYPE,
        }
    }
}

impl TryFrom<TypedEntityRef> for ItemId {
    type Error = ContractError;

    fn try_from(value: TypedEntityRef) -> Result<Self, Self::Error> {
        Self::new(value.entity, value.entity_type)
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", Self::ENTITY_TYPE, self.0)
    }
}

/// A reference to an entity that has been checked to be an [`EntityType::Organization`].
///
/// Converts to [`EntityId`] for free, and can be produced from one only through
/// [`OrganizationId::new`], which must be told the entity's actual type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(into = "TypedEntityRef", try_from = "TypedEntityRef")]
pub struct OrganizationId(EntityId);

impl OrganizationId {
    /// The entity type this reference requires.
    pub const ENTITY_TYPE: EntityType = EntityType::Organization;

    /// Checks `entity_type` against [`OrganizationId::ENTITY_TYPE`] and produces the
    /// reference, or reports the mismatch.
    pub fn new(entity: EntityId, entity_type: EntityType) -> Result<Self, ContractError> {
        if entity_type == Self::ENTITY_TYPE {
            Ok(Self(entity))
        } else {
            Err(ContractError::EntityTypeMismatch {
                entity,
                expected: Self::ENTITY_TYPE,
                actual: entity_type,
            })
        }
    }

    /// The underlying entity identity.
    pub const fn entity_id(self) -> EntityId {
        self.0
    }
}

impl From<OrganizationId> for EntityId {
    fn from(value: OrganizationId) -> Self {
        value.0
    }
}

impl From<OrganizationId> for TypedEntityRef {
    fn from(value: OrganizationId) -> Self {
        Self {
            entity: value.0,
            entity_type: OrganizationId::ENTITY_TYPE,
        }
    }
}

impl TryFrom<TypedEntityRef> for OrganizationId {
    type Error = ContractError;

    fn try_from(value: TypedEntityRef) -> Result<Self, Self::Error> {
        Self::new(value.entity, value.entity_type)
    }
}

impl fmt::Display for OrganizationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", Self::ENTITY_TYPE, self.0)
    }
}
