//! `ObservationDelta`: one observation as the change from the previous one (`PROTOCOL.md` §5.3).
//!
//! Pure functions over the contract's [`Observation`], keyed by contract identity: an entity is
//! replaced or removed by its [`EntityId`], relations and affordances are replaced whole when either
//! changed (an affordance's position in the list is meaningful, `ARC-34`), and `events` are always
//! carried — they are a since-the-last-frame stream already, never accumulated. [`apply`] of
//! [`diff`] reproduces the next observation exactly, with its entities in ascending id order, which
//! is the canonical order revision 2 states for every observation.
//!
//! Whether these deltas go on the wire is decided by measurement (`DECISIONS.md` `DEP-15`).

use std::collections::BTreeMap;

use mineworld_contracts::{
    Affordance, EntityId, Location, Observation, PerceivedEntity, PerceivedEvent, Relation,
    WorldTime,
};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use super::WireObservation;

/// What changed between two observations of one observer (`PROTOCOL.md` §5.3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationDelta {
    /// The instant of the new observation; always present.
    pub at: WorldTime,
    /// Present only when the observer's location changed; `null` when it now has none.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub self_location: Option<Option<Location>>,
    /// Entities added or changed, and entities no longer perceived.
    #[serde(default, skip_serializing_if = "EntityChanges::is_empty")]
    pub entities: EntityChanges,
    /// The whole list, when it changed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relations: Option<Vec<Relation>>,
    /// The whole list, in order, when it changed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub affordances: Option<Vec<Affordance<Value>>>,
    /// The facts learned since the previous frame; always present, replacing the previous ones.
    pub events: Vec<PerceivedEvent<Value>>,
}

/// The entity half of a delta.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityChanges {
    /// Each replaces the entity with its id, or is added.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub upsert: Vec<PerceivedEntity<Value>>,
    /// Ids no longer perceived.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remove: Vec<EntityId>,
}

impl EntityChanges {
    /// Whether nothing about the entities changed.
    pub fn is_empty(&self) -> bool {
        self.upsert.is_empty() && self.remove.is_empty()
    }
}

/// A present field is `Some`, even when it is `null`; an absent one is `None` (by `default`).
fn present<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

/// Why a delta cannot be applied to the observation a client holds.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DeltaError {
    /// The delta removes an entity the held observation does not list.
    #[error("the delta removes entity {0}, which the held observation does not list")]
    UnknownRemove(EntityId),
}

/// `observation` with its entities in ascending [`EntityId`] order: the canonical form revision 2
/// states, so that "reconstructed equals whole" does not depend on a perception's iteration order.
pub fn canonical(observation: WireObservation) -> WireObservation {
    let mut entities = observation.entities().to_vec();
    entities.sort_by_key(PerceivedEntity::id);
    rebuild(&observation, entities)
}

/// The change from `previous` to `next`, which must be observations of one observer.
pub fn diff(previous: &WireObservation, next: &WireObservation) -> ObservationDelta {
    let before: BTreeMap<EntityId, &PerceivedEntity<Value>> = previous
        .entities()
        .iter()
        .map(|entity| (entity.id(), entity))
        .collect();
    let after: BTreeMap<EntityId, &PerceivedEntity<Value>> = next
        .entities()
        .iter()
        .map(|entity| (entity.id(), entity))
        .collect();
    let upsert = after
        .iter()
        .filter(|(id, entity)| before.get(id) != Some(entity))
        .map(|(_, entity)| (*entity).clone())
        .collect();
    let remove = before
        .keys()
        .filter(|id| !after.contains_key(id))
        .copied()
        .collect();
    ObservationDelta {
        at: next.at(),
        self_location: (previous.self_location() != next.self_location())
            .then(|| next.self_location().copied()),
        entities: EntityChanges { upsert, remove },
        relations: (previous.relations() != next.relations()).then(|| next.relations().to_vec()),
        affordances: (previous.affordances() != next.affordances())
            .then(|| next.affordances().to_vec()),
        events: next.events().to_vec(),
    }
}

/// The observation `delta` describes, from the one a client holds.
///
/// # Errors
///
/// [`DeltaError::UnknownRemove`] when the delta removes an entity `held` does not list — which
/// cannot happen on one connection, and which a client answers by resuming (`PROTOCOL.md` §5.3).
pub fn apply(
    held: &WireObservation,
    delta: &ObservationDelta,
) -> Result<WireObservation, DeltaError> {
    let mut entities: BTreeMap<EntityId, PerceivedEntity<Value>> = held
        .entities()
        .iter()
        .map(|entity| (entity.id(), entity.clone()))
        .collect();
    for id in &delta.entities.remove {
        if entities.remove(id).is_none() {
            return Err(DeltaError::UnknownRemove(*id));
        }
    }
    for entity in &delta.entities.upsert {
        entities.insert(entity.id(), entity.clone());
    }
    let location = match delta.self_location {
        Some(changed) => changed,
        None => held.self_location().copied(),
    };
    let mut next = Observation::new(held.observer(), delta.at);
    if let Some(location) = location {
        next = next.at_location(location);
    }
    Ok(next
        .perceiving(entities.into_values().collect())
        .relating(
            delta
                .relations
                .clone()
                .unwrap_or_else(|| held.relations().to_vec()),
        )
        .offering(
            delta
                .affordances
                .clone()
                .unwrap_or_else(|| held.affordances().to_vec()),
        )
        .with_events(delta.events.clone()))
}

/// `observation` with these entities, everything else unchanged.
fn rebuild(
    observation: &WireObservation,
    entities: Vec<PerceivedEntity<Value>>,
) -> WireObservation {
    let mut rebuilt = Observation::new(observation.observer(), observation.at());
    if let Some(location) = observation.self_location() {
        rebuilt = rebuilt.at_location(*location);
    }
    rebuilt
        .perceiving(entities)
        .relating(observation.relations().to_vec())
        .offering(observation.affordances().to_vec())
        .with_events(observation.events().to_vec())
}
