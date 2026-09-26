//! Entity identity: allocation, authoring-key resolution, and lifecycle.
//!
//! The registry is the only thing in MineWorld that allocates an [`EntityId`], and it does so
//! from a plain monotonic counter. That is a requirement rather than a simplification: a world
//! replayed from its event log has to produce the same identities in the same order it did the
//! first time (`AC-12`), so allocation may not consult a random source, the clock, the hash of
//! anything, or the order a map happened to iterate in. Given the same sequence of creations,
//! two registries agree exactly.
//!
//! An identity is never reused. Destroying an entity moves its lifecycle to
//! [`LifecycleState::Destroyed`] and leaves the record in place, because the event log already
//! refers to it and history is immutable (`INV-11`); the counter never moves backwards, so a
//! later entity cannot inherit a dead one's identity and, with it, its history.
//!
//! # Two identities, one entity
//!
//! Authored content refers to `alice`; the runtime refers to entity `1`. The registry is where
//! those meet: an [`EntityKey`] is resolved to an [`EntityId`] once, when the content that
//! names it is loaded (`KD-3`), and everything afterwards carries the identity. Keys are
//! therefore unique within a world, and a second entity claiming one is refused rather than
//! silently renamed.

use std::collections::BTreeMap;

use mineworld_contracts::{
    Entity, EntityId, EntityKey, EntityType, LifecycleState, Metadata, Tags,
};
use serde::{Deserialize, Serialize};

use crate::error::KernelError;

/// The first identity a world allocates.
///
/// One, not zero. Zero is what an absent, defaulted or truncated number looks like in a
/// serialized record, so leaving it unallocated turns that class of mistake into an
/// [`UnknownEntity`](KernelError::UnknownEntity) refusal instead of a silent hit on whichever
/// entity happened to be created first.
const FIRST_ENTITY_ID: u64 = 1;

/// Every entity in one world, and the counter that named them.
///
/// Iteration is in [`EntityId`] order — allocation order, therefore — and never in insertion
/// order, because insertion order is not a property of the world, it is a property of the run
/// (`KD-5`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "EntityRegistrySnapshot")]
pub struct EntityRegistry {
    next_id: u64,
    entities: BTreeMap<EntityId, Entity>,
    /// Derived from `entities`, so it is rebuilt on load rather than persisted: two copies of
    /// the same fact in one file are two chances for them to disagree.
    #[serde(skip)]
    by_key: BTreeMap<EntityKey, EntityId>,
}

impl Default for EntityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityRegistry {
    /// An empty world, about to allocate [`FIRST_ENTITY_ID`].
    pub fn new() -> Self {
        Self {
            next_id: FIRST_ENTITY_ID,
            entities: BTreeMap::new(),
            by_key: BTreeMap::new(),
        }
    }

    /// Creates an entity with no tags and no authoring provenance — an entity a running world
    /// brought into being rather than one an author wrote.
    pub fn create(
        &mut self,
        key: EntityKey,
        entity_type: EntityType,
    ) -> Result<EntityId, KernelError> {
        self.create_authored(key, entity_type, Tags::default(), None)
    }

    /// Creates an entity as authored content describes it.
    ///
    /// The identity is allocated here rather than supplied, which is why this returns one: a
    /// caller that could choose an identity could reuse one.
    pub fn create_authored(
        &mut self,
        key: EntityKey,
        entity_type: EntityType,
        tags: Tags,
        metadata: Option<Metadata>,
    ) -> Result<EntityId, KernelError> {
        if let Some(existing) = self.by_key.get(&key) {
            return Err(KernelError::EntityKeyAlreadyUsed {
                key,
                existing: *existing,
            });
        }

        let id = EntityId::from_raw(self.next_id);
        let next_id = self
            .next_id
            .checked_add(1)
            .ok_or(KernelError::EntityIdSpaceExhausted)?;

        let mut entity = Entity::new(id, key.clone(), entity_type).with_tags(tags);
        if let Some(metadata) = metadata {
            entity = entity.with_metadata(metadata);
        }

        self.next_id = next_id;
        self.by_key.insert(key, id);
        self.entities.insert(id, entity);
        Ok(id)
    }

    /// Resolves an authoring key to the identity the runtime uses.
    pub fn resolve(&self, key: &EntityKey) -> Result<EntityId, KernelError> {
        self.by_key
            .get(key)
            .copied()
            .ok_or_else(|| KernelError::UnknownEntityKey { key: key.clone() })
    }

    /// The entity record, if this world allocated that identity.
    pub fn get(&self, entity: EntityId) -> Option<&Entity> {
        self.entities.get(&entity)
    }

    /// The entity record, or the refusal naming the identity that does not exist.
    ///
    /// Reading an entity is open to every system (`KD-4`); this exists for the callers whose
    /// next step needs the record and should stop with a named error rather than silently do
    /// nothing.
    pub fn require(&self, entity: EntityId) -> Result<&Entity, KernelError> {
        self.get(entity)
            .ok_or(KernelError::UnknownEntity { entity })
    }

    /// Moves an entity's lifecycle, if [`LifecycleState::can_transition_to`] permits it.
    ///
    /// This is the only way a lifecycle changes, and it is how an entity is destroyed: the
    /// record stays, the identity stays spent, and the transition out of
    /// [`Destroyed`](LifecycleState::Destroyed) does not exist.
    pub fn transition(
        &mut self,
        entity: EntityId,
        next: LifecycleState,
    ) -> Result<(), KernelError> {
        let record = self
            .entities
            .get_mut(&entity)
            .ok_or(KernelError::UnknownEntity { entity })?;
        record.transition_to(next)?;
        Ok(())
    }

    /// Every entity, in [`EntityId`] order.
    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values()
    }

    /// How many entities this world has ever created, destroyed ones included.
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Whether the world has no entities at all.
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }
}

/// A registry as it is written down.
///
/// The counter and the entity records, with the key index left out because it is derived: two
/// copies of one fact in a file are two chances for them to disagree. This shape is public
/// because it is what persistence reads and writes, and because the refusals below are part of
/// the contract — a caller rebuilding a world wants the named error, not a message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityRegistrySnapshot {
    /// The next identity the registry will allocate.
    pub next_id: u64,
    /// Every entity the world has created, filed under its own identity.
    pub entities: BTreeMap<EntityId, Entity>,
}

impl From<&EntityRegistry> for EntityRegistrySnapshot {
    fn from(registry: &EntityRegistry) -> Self {
        Self {
            next_id: registry.next_id,
            entities: registry.entities.clone(),
        }
    }
}

impl TryFrom<EntityRegistrySnapshot> for EntityRegistry {
    type Error = KernelError;

    /// Rebuilds a registry from its persisted form, refusing one that has lost the properties a
    /// registry is supposed to guarantee.
    ///
    /// Three things are checked, because a file can say all three wrongly and each would break
    /// something a later reader depends on: a record filed under an identity other than its own
    /// makes identity ambiguous, a counter that has fallen behind would hand out an identity
    /// twice, and one key on two entities makes authored references ambiguous. A world is
    /// refused at load rather than misbehaving later.
    fn try_from(snapshot: EntityRegistrySnapshot) -> Result<Self, Self::Error> {
        if snapshot.next_id < FIRST_ENTITY_ID {
            return Err(KernelError::PersistedIdCounterTooLow {
                next: snapshot.next_id,
                first: FIRST_ENTITY_ID,
            });
        }

        let mut by_key = BTreeMap::new();
        for (id, entity) in &snapshot.entities {
            if entity.id() != *id {
                return Err(KernelError::PersistedEntityIdMismatch {
                    at: *id,
                    found: entity.id(),
                });
            }
            if id.raw() >= snapshot.next_id {
                return Err(KernelError::PersistedIdWouldBeReused {
                    next: snapshot.next_id,
                    allocated: *id,
                });
            }
            if let Some(first) = by_key.insert(entity.key().clone(), *id) {
                return Err(KernelError::PersistedEntityKeyRepeated {
                    key: entity.key().clone(),
                    first,
                    second: *id,
                });
            }
        }

        Ok(Self {
            next_id: snapshot.next_id,
            entities: snapshot.entities,
            by_key,
        })
    }
}
