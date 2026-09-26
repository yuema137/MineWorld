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
//! [`EntityId`] when it loads the pack. [`ActionTypeId`](crate::action::ActionTypeId) is a
//! declaration name too, and obeys the same rule, but lives beside the contract that gives it
//! meaning: an action type says nothing except next to the [`Action`](crate::action::Action)
//! trait that declares it.
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
//!
//! # How an opaque identity is encoded, and why the rule lives on the type
//!
//! The four opaque runtime identities are 64-bit, and a great many consumers of this contract
//! read JSON with a parser that has one number type: a double. Above 2^53 a double cannot hold
//! an integer exactly, so an id serialized as a JSON *number* is corrupted on the way in — and
//! silently, because the Rust side is correct and the JSON text is correct. A renderer
//! integration spike measured it from inside a Godot client: `9007199254740995` and
//! `9007199254740997` both arrived as `9007199254740996`, so two distinct entities became one
//! (`spike/FINDINGS.md` F1).
//!
//! So [`EntityId`], [`EventId`], [`ActionId`] and [`ProcessId`] hand-write their `serde`
//! implementations and key them on [`Serializer::is_human_readable`]:
//!
//! ```text
//! human-readable     (serde_json, YAML, TOML)   a decimal string, "9007199254740995"
//! not human-readable (bincode, postcard, …)     the u64 it has always been
//! ```
//!
//! Reading back in the human-readable case accepts **both** forms, so an existing JSON fixture
//! or a hand-written test datum that holds a number still loads. A float is refused outright
//! rather than truncated: an id that has already lost precision must fail where it is read, not
//! resolve to whichever entity it rounded onto.
//!
//! **Why the rule is on the type rather than at the protocol boundary.** `DD-15` originally
//! assigned this to the wire encoding, and rejected encoding ids as strings in the contract on
//! the grounds that it "would distort persistence and any binary encoding to suit one client's
//! parser". `is_human_readable()` is precisely the distinction that prevents that: a binary
//! encoding still receives a `u64` and is untouched, so persistence and replay determinism are
//! unaffected. What the protocol boundary could not do is reach inside a payload.
//! [`ComponentRecord`](crate::component::ComponentRecord) and
//! [`EventRecord`](crate::event::EventRecord) erase their contents by design, so a protocol
//! layer cannot find the ids in them — and real payloads carry ids: an employment component
//! naming an employer, a conversation component naming who is being talked to. An encoder that
//! guessed from field names would also corrupt the integers that are *not* ids. Keying on the
//! type is the only rule that travels with the value into a payload this crate cannot read
//! (`spike/FINDINGS.md` F2, which supersedes `DD-15`'s assignment of the problem).

use core::fmt;
use std::borrow::Cow;

use serde::de::{self, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::{ContractError, IdentifierKind};

/// Maximum length, in bytes, of every validated textual identifier in this crate.
pub const MAX_IDENTIFIER_LENGTH: usize = 64;

/// What is wrong with an identifier, as much of it as a `const fn` can determine.
///
/// The rule is checked in one place, [`check_identifier`], so that a name written as a literal
/// in code and a name read from an authored file cannot be judged by two drifting
/// implementations. This enum is the const-compatible half of the answer: byte positions and
/// lengths, no `char` decoding and no [`ContractError`], which needs both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IdentifierFault {
    Empty,
    TooLong { length: usize },
    IllegalByte { position: usize },
    SeparatorAtEdge { position: usize },
}

/// The one character rule shared by every validated textual identifier in this crate: 1 to
/// [`MAX_IDENTIFIER_LENGTH`] bytes of lowercase ASCII letters, digits, `-` and `_`, neither
/// beginning nor ending with `-` or `_`.
///
/// The set is the one the specification's own examples already use — `lakewood`,
/// `modern-life`, `conversation`, `pacific_northwest` in `docs/MODULE_SPEC.md` §4 — and is
/// restricted enough that an identifier can appear in a file name, a URL and a database key
/// without escaping. A rejected value is never normalized: silently lower-casing or trimming
/// an authored name would make two World Packs that read differently resolve to one entity.
///
/// This is a `const fn` so that an identifier declared as a literal in code — a system's name,
/// a component type's name — is checked while the crate that declares it compiles, rather than
/// when it is first loaded.
///
/// Visible to the whole crate rather than to this module, because the declaration names that
/// live beside their own contracts — an action type, an event type — must be judged by this
/// implementation and not by a second one that could drift from it.
pub(crate) const fn check_identifier(value: &str) -> Result<(), IdentifierFault> {
    let bytes = value.as_bytes();
    if bytes.is_empty() {
        return Err(IdentifierFault::Empty);
    }
    if bytes.len() > MAX_IDENTIFIER_LENGTH {
        return Err(IdentifierFault::TooLong {
            length: bytes.len(),
        });
    }

    let mut position = 0;
    while position < bytes.len() {
        if !is_legal_identifier_byte(bytes[position]) {
            return Err(IdentifierFault::IllegalByte { position });
        }
        position += 1;
    }

    // Every legal byte is ASCII, so the first and last bytes are whole characters here.
    if is_separator(bytes[0]) {
        return Err(IdentifierFault::SeparatorAtEdge { position: 0 });
    }
    let last = bytes.len() - 1;
    if is_separator(bytes[last]) {
        return Err(IdentifierFault::SeparatorAtEdge { position: last });
    }

    Ok(())
}

/// Applies [`check_identifier`] and turns its verdict into the error a caller can act on,
/// decoding the offending character only when there is one to report.
pub(crate) fn validate_identifier(kind: IdentifierKind, value: &str) -> Result<(), ContractError> {
    match check_identifier(value) {
        Ok(()) => Ok(()),
        Err(IdentifierFault::Empty) => Err(ContractError::IdentifierEmpty { kind }),
        Err(IdentifierFault::TooLong { length }) => Err(ContractError::IdentifierTooLong {
            kind,
            length,
            max: MAX_IDENTIFIER_LENGTH,
        }),
        Err(IdentifierFault::IllegalByte { position }) => {
            // The first illegal byte is never a UTF-8 continuation byte: every legal byte is
            // ASCII, so a multi-byte character's leading byte is itself illegal and is found
            // first. The slice therefore starts on a character boundary.
            let character = value
                .get(position..)
                .and_then(|rest| rest.chars().next())
                .unwrap_or(char::REPLACEMENT_CHARACTER);
            Err(ContractError::IdentifierIllegalCharacter {
                kind,
                character,
                position,
            })
        }
        Err(IdentifierFault::SeparatorAtEdge { position }) => {
            Err(ContractError::IdentifierSeparatorAtEdge {
                kind,
                character: char::from(value.as_bytes()[position]),
            })
        }
    }
}

const fn is_legal_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit() || is_separator(byte)
}

const fn is_separator(byte: u8) -> bool {
    byte == b'-' || byte == b'_'
}

// ---------------------------------------------------------------------------------------------
// Opaque runtime identity
// ---------------------------------------------------------------------------------------------

/// Writes one opaque identity, as a decimal string for a human-readable format and as the `u64`
/// it is for every other. See this module's documentation for why the rule lives here.
///
/// `collect_str` rather than a `String`: `serde_json` writes the digits straight into its output,
/// so the human-readable path allocates nothing.
fn serialize_opaque_id<S: Serializer>(raw: u64, serializer: S) -> Result<S::Ok, S::Error> {
    if serializer.is_human_readable() {
        serializer.collect_str(&raw)
    } else {
        serializer.serialize_u64(raw)
    }
}

/// Reads one opaque identity back.
///
/// Deliberately implements no floating-point method. Serde's default refuses a float with
/// `invalid type: floating point ...`, which is the required behaviour — an id that arrived as a
/// double has already lost precision, and truncating it would resolve it to whichever identity it
/// rounded onto. Writing the rejection out by hand would also mean naming a float type in this
/// crate's source, which the contract layer forbids (`tests/spatial.rs`).
struct OpaqueIdVisitor {
    /// The type being read, for the message a failure produces.
    type_name: &'static str,
}

impl<'de> Visitor<'de> for OpaqueIdVisitor {
    type Value = u64;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} as a decimal string or an unsigned 64-bit integer",
            self.type_name
        )
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
        Ok(value)
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
        u64::try_from(value).map_err(|_| E::invalid_value(Unexpected::Signed(value), &self))
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        value
            .parse::<u64>()
            .map_err(|_| E::invalid_value(Unexpected::Str(value), &self))
    }
}

/// Reads one opaque identity, accepting both encoded forms where the format is self-describing.
///
/// The branch is not an optimization: a binary format is typically not self-describing, so
/// `deserialize_any` is not answerable there and the concrete `deserialize_u64` is what it
/// requires.
fn deserialize_opaque_id<'de, D: Deserializer<'de>>(
    type_name: &'static str,
    deserializer: D,
) -> Result<u64, D::Error> {
    let visitor = OpaqueIdVisitor { type_name };
    if deserializer.is_human_readable() {
        deserializer.deserialize_any(visitor)
    } else {
        deserializer.deserialize_u64(visitor)
    }
}

/// Gives one opaque identity newtype the encoding this module documents.
///
/// A macro because the four identities must agree exactly: four hand-written copies of the same
/// twenty lines is four chances for one of them to drift, and a drifting id encoding is the class
/// of defect this whole rule exists to remove.
macro_rules! opaque_id_serde {
    ($name:ident) => {
        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serialize_opaque_id(self.0, serializer)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                deserialize_opaque_id(stringify!($name), deserializer).map(Self)
            }
        }
    };
}

/// Opaque runtime identity of an entity: stable, unique, and never reused.
///
/// Allocation belongs to the kernel, not to this crate. [`EntityId::from_raw`] exists so that
/// the allocator and the persistence layer can rebuild an identity they already own; it is not
/// a way to invent one. There is deliberately no `Default`, no arithmetic and no increment
/// here: an allocator that could reuse or overflow an identity is a kernel concern that must
/// stay visible there rather than hide behind a convenience in a contract type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(u64);

opaque_id_serde!(EntityId);

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventId(u64);

opaque_id_serde!(EventId);

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ActionId(u64);

opaque_id_serde!(ActionId);

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessId(u64);

opaque_id_serde!(ProcessId);

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
pub struct SystemId(Cow<'static, str>);

impl SystemId {
    /// Validates a declared system name against the identifier rule stated in this module's
    /// documentation.
    pub fn new(value: impl Into<String>) -> Result<Self, ContractError> {
        let value = value.into();
        validate_identifier(IdentifierKind::SystemId, &value)?;
        Ok(Self(Cow::Owned(value)))
    }

    /// Declares the name as a literal in code, checked while the declaring crate compiles.
    ///
    /// This is what lets a component state its type and its owning system as part of its type
    /// rather than as data: an associated constant cannot hold a validated `String`, but it can
    /// hold this. An illegal literal is a compile error, so a declaration that would be
    /// rejected at load time never reaches a running world.
    pub const fn from_static(value: &'static str) -> Self {
        match check_identifier(value) {
            Ok(()) => Self(Cow::Borrowed(value)),
            Err(_) => panic!(
                "a system id literal must be 1 to 64 bytes of lowercase ASCII letters, digits, \
                 '-' and '_', and must not begin or end with a separator"
            ),
        }
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
        value.0.into_owned()
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
pub struct ComponentTypeId(Cow<'static, str>);

impl ComponentTypeId {
    /// Validates a declared component type name against the identifier rule stated in this
    /// module's documentation.
    pub fn new(value: impl Into<String>) -> Result<Self, ContractError> {
        let value = value.into();
        validate_identifier(IdentifierKind::ComponentTypeId, &value)?;
        Ok(Self(Cow::Owned(value)))
    }

    /// Declares the name as a literal in code, checked while the declaring crate compiles.
    ///
    /// This is what lets a component state its type and its owning system as part of its type
    /// rather than as data: an associated constant cannot hold a validated `String`, but it can
    /// hold this. An illegal literal is a compile error, so a declaration that would be
    /// rejected at load time never reaches a running world.
    pub const fn from_static(value: &'static str) -> Self {
        match check_identifier(value) {
            Ok(()) => Self(Cow::Borrowed(value)),
            Err(_) => panic!(
                "a component type id literal must be 1 to 64 bytes of lowercase ASCII letters, digits, \
                 '-' and '_', and must not begin or end with a separator"
            ),
        }
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
        value.0.into_owned()
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
