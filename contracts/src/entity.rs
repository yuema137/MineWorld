//! The entity record: identity, taxonomy, lifecycle and authoring provenance — and nothing
//! else.
//!
//! An [`Entity`] deliberately has no name, no position, no owner, no inventory and no health.
//! Every one of those is a component owned by the system that provides it
//! (`docs/CORE_CONCEPTS.md` §4), which is what makes a world composable: removing a system
//! removes the meaning it contributed without invalidating the entity. A field added here
//! would be a field every world pays for, in the kernel's vocabulary, whether or not its
//! systems are installed.

use core::fmt;

use serde::{Deserialize, Serialize};

use crate::error::{ContractError, IdentifierKind};
use crate::ids::{EntityId, EntityKey, EntityType, validate_identifier};

/// One semantic label on an entity, drawn from an open vocabulary: `cafe`, `furniture`,
/// `night-shift`.
///
/// Tags are a taxonomy, not state: a system reads them to decide whether an entity is of
/// interest to it. They obey the same character rule as every other authored name in this
/// crate, so a tag can be written in a World Pack, a query and a file name unchanged.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Tag(String);

impl Tag {
    /// Validates a label against the identifier rule documented in [`crate::ids`].
    pub fn new(value: impl Into<String>) -> Result<Self, ContractError> {
        let value = value.into();
        validate_identifier(IdentifierKind::Tag, &value)?;
        Ok(Self(value))
    }

    /// The label as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Tag {
    type Error = ContractError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Tag {
    type Error = ContractError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Tag> for String {
    fn from(value: Tag) -> Self {
        value.0
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The set of tags on an entity.
///
/// A set, not a list: a tag is either present or it is not, so supplying one twice is not an
/// error and does not change the entity. Backed by a [`BTreeSet`](std::collections::BTreeSet),
/// so iteration and serialization are in one fixed order regardless of the order the tags were
/// authored or inserted in — the property the event log depends on.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Tags(std::collections::BTreeSet<Tag>);

impl Tags {
    /// Collects tags into a set, discarding duplicates.
    pub fn new(tags: impl IntoIterator<Item = Tag>) -> Self {
        Self(tags.into_iter().collect())
    }

    /// Whether the entity carries this tag.
    pub fn contains(&self, tag: &Tag) -> bool {
        self.0.contains(tag)
    }

    /// The tags, in their fixed order.
    pub fn iter(&self) -> impl Iterator<Item = &Tag> {
        self.0.iter()
    }

    /// How many distinct tags the entity carries.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the entity carries no tags.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a> IntoIterator for &'a Tags {
    type Item = &'a Tag;
    type IntoIter = std::collections::btree_set::Iter<'a, Tag>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// Whether an entity is simulated, sleeping, or gone.
///
/// `Dormant` exists so that a world can stop spending simulation on an entity without
/// destroying it — an inhabitant of a town nobody is visiting still exists. `Destroyed` is
/// terminal: history in the event log is immutable, so an entity that has been destroyed is
/// never resurrected under the same identity, and an [`EntityId`] is never reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleState {
    /// Simulated: systems run for this entity.
    Active,
    /// Present but not simulated.
    Dormant,
    /// Gone. A terminal state with no outgoing transition.
    Destroyed,
}

impl LifecycleState {
    /// Whether this state may become `next`.
    ///
    /// A state may not "transition" to itself: a no-op is not a lifecycle change, and treating
    /// it as legal would let a caller believe it had moved an entity that never moved.
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Active, Self::Dormant)
                | (Self::Active, Self::Destroyed)
                | (Self::Dormant, Self::Active)
                | (Self::Dormant, Self::Destroyed)
        )
    }
}

impl fmt::Display for LifecycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Active => "active",
            Self::Dormant => "dormant",
            Self::Destroyed => "destroyed",
        };
        f.write_str(name)
    }
}

/// Where an entity came from, for a human debugging a world.
///
/// Authoring provenance only, and never read as gameplay state (`docs/CORE_CONCEPTS.md` §3).
/// Nothing in the simulation may branch on these values: a system that needs to know something
/// about an entity reads a component or a tag. They are free text on purpose — `source_pack`
/// becomes a declared pack identity when pack loading becomes a contract, and until then
/// inventing that type here would be a guess.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Metadata {
    /// The pack the entity was authored in, as the pack names itself.
    pub source_pack: String,
    /// Where in that pack it was authored, such as `people/alice.yaml`.
    pub source_path: String,
    /// A note from whoever authored it.
    pub authoring_note: Option<String>,
}

/// A thing that exists: identity, what kind of thing it is, its tags, its lifecycle state, and
/// where it was authored.
///
/// Everything else an entity appears to have — a name, a location, money, a job — is a
/// component owned by a system. This record is what remains true of an entity when every
/// system is uninstalled.
///
/// Fields are private because two of them are not free to change: an entity's identity and its
/// type are fixed for its whole life, and its lifecycle may only move the way
/// [`LifecycleState::can_transition_to`] permits. Reaching in to set `lifecycle = Destroyed`
/// would bypass the only check that makes destruction final.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entity {
    id: EntityId,
    key: EntityKey,
    entity_type: EntityType,
    tags: Tags,
    lifecycle: LifecycleState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    metadata: Option<Metadata>,
}

impl Entity {
    /// The three facts an entity cannot exist without. It starts
    /// [`Active`](LifecycleState::Active), untagged and without provenance; anything optional
    /// is added with [`Entity::with_tags`] and [`Entity::with_metadata`], so there is no way to
    /// build a half-initialized entity or to forget a required field.
    pub fn new(id: EntityId, key: EntityKey, entity_type: EntityType) -> Self {
        Self {
            id,
            key,
            entity_type,
            tags: Tags::default(),
            lifecycle: LifecycleState::Active,
            metadata: None,
        }
    }

    /// Attaches the entity's tags.
    #[must_use]
    pub fn with_tags(mut self, tags: Tags) -> Self {
        self.tags = tags;
        self
    }

    /// Attaches authoring provenance. An entity created by a running world rather than by an
    /// author has none, which is why it is optional.
    #[must_use]
    pub fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Runtime identity.
    pub const fn id(&self) -> EntityId {
        self.id
    }

    /// Authoring identity.
    pub const fn key(&self) -> &EntityKey {
        &self.key
    }

    /// Which of the four kinds of thing this is.
    pub const fn entity_type(&self) -> EntityType {
        self.entity_type
    }

    /// The entity's tags.
    pub const fn tags(&self) -> &Tags {
        &self.tags
    }

    /// Whether the entity is simulated, dormant, or gone.
    pub const fn lifecycle(&self) -> LifecycleState {
        self.lifecycle
    }

    /// Where the entity was authored, if it was authored.
    pub const fn metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }

    /// Moves the entity to `next` if [`LifecycleState::can_transition_to`] permits it, and
    /// reports the refused transition otherwise. This is the only way the lifecycle changes.
    pub fn transition_to(&mut self, next: LifecycleState) -> Result<(), ContractError> {
        if !self.lifecycle.can_transition_to(next) {
            return Err(ContractError::IllegalLifecycleTransition {
                entity: self.id,
                from: self.lifecycle,
                to: next,
            });
        }
        self.lifecycle = next;
        Ok(())
    }
}
