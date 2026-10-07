//! What a System Pack declares about the part of an authored file it owns.

use std::collections::BTreeMap;

use mineworld_contracts::{EntityId, EntityKey, EntityType, Rejection};
use mineworld_kernel::{Emission, SystemIdentity, WorldRead};
use serde::de::DeserializeOwned;

/// The key a section has in a content file: one word, `1..=32` bytes of `a-z`, `0-9` and `-`.
///
/// One namespace across every pack a build provides, and distinct from the format's own fields — the
/// precedent action types set. A pack states its name as a constant, so the name is checked when the
/// pack is compiled rather than when a world is loaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SectionName(&'static str);

impl SectionName {
    /// The longest section name, in bytes.
    pub const MAX_BYTES: usize = 32;

    /// A section name, checked: a constant that breaks the rule does not compile.
    ///
    /// # Panics
    ///
    /// When `name` is empty, longer than [`SectionName::MAX_BYTES`], or holds a byte other than
    /// `a-z`, `0-9` and `-` — at compile time, for a constant.
    pub const fn from_static(name: &'static str) -> Self {
        assert!(
            is_valid(name),
            "a section name is 1-32 bytes of a-z, 0-9 and -"
        );
        Self(name)
    }

    /// The key as it is written in a file.
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

const fn is_valid(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.is_empty() || bytes.len() > SectionName::MAX_BYTES {
        return false;
    }
    let mut at = 0;
    while at < bytes.len() {
        let byte = bytes[at];
        if !(byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-') {
            return false;
        }
        at += 1;
    }
    true
}

impl core::fmt::Display for SectionName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.0)
    }
}

/// What kind of content file a key names: a person, place, item or organization file (`ARC-36`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContentKind {
    /// `people/<key>.yaml`.
    Person,
    /// `places/<key>.yaml`.
    Place,
    /// `items/<key>.yaml`: an item kind, not one object (`ARC-36`).
    Item,
    /// `organizations/<key>.yaml`.
    Organization,
}

impl ContentKind {
    /// Every kind, for a guard that must cover each one.
    pub const ALL: [Self; 4] = [Self::Person, Self::Place, Self::Item, Self::Organization];

    /// The directory this kind of content lives in.
    pub const fn directory(self) -> &'static str {
        match self {
            Self::Person => "people",
            Self::Place => "places",
            Self::Item => "items",
            Self::Organization => "organizations",
        }
    }

    /// What one file of this kind describes, for a message to name it.
    pub const fn describes(self) -> &'static str {
        match self {
            Self::Person => "person",
            Self::Place => "place",
            Self::Item => "item",
            Self::Organization => "organization",
        }
    }

    /// The entity type a file of this kind creates.
    pub const fn entity_type(self) -> EntityType {
        match self {
            Self::Person => EntityType::Person,
            Self::Place => EntityType::Place,
            Self::Item => EntityType::Item,
            Self::Organization => EntityType::Organization,
        }
    }
}

impl core::fmt::Display for ContentKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.describes())
    }
}

/// Another entity a section names by its authoring key, and the type that entity must be.
///
/// Returned by [`AuthoredSection::references`], so the loader can refuse a routine that names a place
/// the pack never declared — or a person where a place belongs — by name, before any world exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reference<'a> {
    /// The key, as authored.
    pub key: &'a EntityKey,
    /// What it must name.
    pub entity_type: EntityType,
}

/// What an owner is handed when it seeds: the assembled world, read-only, and what its keys became.
///
/// The world has every entity and every installed system and no state yet beyond the genesis facts
/// stated before this one, so an owner may check the entities its section names against it — which is
/// what makes an authored value pass the same check as any other.
pub struct Seeding<'s, 'w> {
    world: &'s WorldRead<'w>,
    keys: &'s BTreeMap<EntityKey, EntityId>,
}

impl<'s, 'w> Seeding<'s, 'w> {
    /// Seeding over this world and these resolved keys.
    pub const fn new(world: &'s WorldRead<'w>, keys: &'s BTreeMap<EntityKey, EntityId>) -> Self {
        Self { world, keys }
    }

    /// The assembled world.
    pub const fn world(&self) -> &'s WorldRead<'w> {
        self.world
    }

    /// The entity a key resolved to, if it is of `entity_type`.
    ///
    /// [`None`] for an undeclared key or one of another type. The loader has already refused both for
    /// every key [`AuthoredSection::references`] lists, so an owner reaching [`None`] here names a key
    /// it did not list.
    pub fn resolve(&self, key: &EntityKey, entity_type: EntityType) -> Option<EntityId> {
        let entity = *self.keys.get(key)?;
        self.world
            .entity(entity)
            .is_some_and(|record| record.entity_type() == entity_type)
            .then_some(entity)
    }
}

/// A part of an authored person, place, item or organization file that a System Pack owns
/// (`ARC-31`, `ARC-36`).
///
/// Implemented by the System Pack that owns the state the section becomes. The pack decides what the
/// section *is* — its type, its rules, its facts — and the World Pack loader decides only what every
/// section shares.
pub trait AuthoredSection: SystemIdentity {
    /// The key this pack owns in a content file.
    const SECTION: SectionName;

    /// The content files that may carry it.
    const CARRIED_BY: &'static [ContentKind];

    /// The section, as authored.
    ///
    /// Deserializing it *is* the owner's validation, so an invalid section cannot be constructed: the
    /// loader decodes it straight from the file, and a refusal carries the file's line and column and
    /// this type's own message.
    type Authored: DeserializeOwned + core::fmt::Debug + Send + Sync + 'static;

    /// The other entities the section names by key, each with the entity type it must be.
    fn references(authored: &Self::Authored) -> Vec<Reference<'_>> {
        let _ = authored;
        Vec::new()
    }

    /// The genesis facts the section becomes, about `subject` — in this pack's own vocabulary and
    /// built with its own codec. The loader refuses a fact of any other pack's vocabulary.
    ///
    /// # Errors
    ///
    /// A [`Rejection`] when the world as assembled cannot take the value.
    fn seed(
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
        authored: &Self::Authored,
    ) -> Result<Vec<Emission>, Rejection>;
}
