//! Observation readers shared by the decision models: what an observation discloses, decoded once
//! (`pr-dl-b-motivated.md` SD-DLb-1, F-DLb-1).
//!
//! Every function here reads a disclosed shape and takes no draw, so a model that reads a record decodes it
//! through exactly one implementation and two models cannot disagree about what a record means. They moved
//! here from the modules that first needed them, with their bodies unchanged; the draw-taking code that
//! decides with them stays where it was.

use std::collections::BTreeMap;

use mineworld_contracts::{
    Action, Component, EntityId, EntityType, LocalPosition, Location, Observation, PerceivedEntity,
    PlaceId, Tag,
};
use mineworld_conversation::{ConversationHistory, Heard, Talk};
use mineworld_group_activity::Participation;
use mineworld_movement::{Destination, Passages, Walking};
use mineworld_naming::DisplayName;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// The part of movement's disclosed `walking` record this controller reads: where the walk goes.
///
/// A private copy of movement's wire shape (`{ destination, next }`), because movement does not
/// export its disclosed type. Finding F-12n2-1 (step-11 §21.15): movement should export a typed
/// disclosed walk, and this copy should go.
#[derive(Deserialize)]
struct DisclosedWalk {
    destination: Destination,
}

/// The observer's own conversation history, as the observation disclosed it.
///
/// Through [`ComponentRecord::payload_for`](mineworld_contracts::ComponentRecord::payload_for), so a
/// record labelled with a different component type is not read as this one — the same check every
/// other reader of a record gets. An observation that disclosed nothing yields [`None`], which is
/// *this person has been told nothing they may know about*, not an error: a controller cannot ask
/// again, because there is nothing to ask (`INV-13`).
pub fn disclosed_history(me: &PerceivedEntity<Value>) -> Option<ConversationHistory> {
    let record = me
        .components()
        .iter()
        .find(|record| *record.component_type() == ConversationHistory::COMPONENT_TYPE)?;
    serde_json::from_value(record.payload_for::<ConversationHistory>().ok()?.clone()).ok()
}

/// The newest thing each person has said to this observer, by speaker.
///
/// Oldest first in the history, so a later entry overwrites an earlier one and the map holds the
/// newest. The observer's own entry is dropped if one ever appears: answering oneself is not a
/// conversation, and the conversation pack refuses it anyway.
pub fn newest_per_speaker(
    history: &ConversationHistory,
    me: EntityId,
) -> BTreeMap<EntityId, &Heard> {
    let mut newest = BTreeMap::new();
    for heard in history.heard() {
        let speaker = heard.speaker().entity_id();
        if speaker == me {
            continue;
        }
        newest.insert(speaker, heard);
    }
    newest
}

/// Whether the **server** says this observer may speak to that person right now.
///
/// Read, never computed. A controller that worked out for itself whether somebody was in reach would
/// be the duplicated rule `ENGINEERING_RULES.md` §9 calls an architectural failure, and it would
/// disagree with dispatch the first time either changed.
pub fn may_talk_to(observation: &Observation<Value>, person: EntityId) -> bool {
    observation.affordances().iter().any(|affordance| {
        *affordance.action_type() == Talk::ACTION_TYPE
            && affordance.target() == Some(person)
            && affordance.is_available()
    })
}

/// The display name an observation discloses about `person`, decoded with `naming`'s own type.
pub fn disclosed_name(observation: &Observation<Value>, person: EntityId) -> Option<String> {
    let record = observation
        .entity(person)?
        .components()
        .iter()
        .find(|record| *record.component_type() == DisplayName::COMPONENT_TYPE)?;
    let name: DisplayName =
        serde_json::from_value(record.payload_for::<DisplayName>().ok()?.clone()).ok()?;
    Some(name.name().as_str().to_owned())
}

/// Whether this observer is part of an activity, by its own disclosed `Participation`.
pub fn in_activity(observation: &Observation<Value>) -> bool {
    observation
        .entity(observation.observer())
        .and_then(disclosed::<Participation>)
        .is_some()
}

/// Everybody the server says `A` is available against right now, in the order the observation lists.
pub fn people_offering<A: Action>(observation: &Observation<Value>) -> Vec<EntityId> {
    observation
        .entities()
        .iter()
        .map(PerceivedEntity::id)
        .filter(|entity| *entity != observation.observer())
        .filter(|entity| available::<A>(observation, Some(*entity)))
        .collect()
}

/// Whether the server offers `A` against `target` and says it is possible now.
pub fn available<A: Action>(observation: &Observation<Value>, target: Option<EntityId>) -> bool {
    observation.affordances().iter().any(|affordance| {
        *affordance.action_type() == A::ACTION_TYPE
            && affordance.target() == target
            && affordance.is_available()
    })
}

/// A component this entity's record discloses, decoded with its owner's type.
pub fn disclosed<C: Component + DeserializeOwned>(entity: &PerceivedEntity<Value>) -> Option<C> {
    let record = entity
        .components()
        .iter()
        .find(|record| *record.component_type() == C::COMPONENT_TYPE)?;
    serde_json::from_value(record.payload_for::<C>().ok()?.clone()).ok()
}

/// Everybody the observation lists as a person, except the observer, in the order it lists them.
pub fn others(observation: &Observation<Value>) -> impl Iterator<Item = &PerceivedEntity<Value>> {
    let me = observation.observer();
    observation
        .entities()
        .iter()
        .filter(move |entity| entity.entity_type() == EntityType::Person && entity.id() != me)
}

/// The doorways the place I stand in discloses, as the observation carries them.
pub fn doorways(observation: &Observation<Value>, here: Location) -> Option<Passages> {
    let place = observation.entity(here.place().entity_id())?;
    let record = place
        .components()
        .iter()
        .find(|record| *record.component_type() == Passages::COMPONENT_TYPE)?;
    serde_json::from_value(record.payload_for::<Passages>().ok()?.clone()).ok()
}

/// The `to_tags` of each doorway of the place `here`, from the disclosed passages payload, by the place
/// each leads to. Read from the payload because `to_tags` is a disclosure only: movement never stores
/// it, and no typed record carries it (moved from `motivated`, Q-S35-OP1, so both models read it once).
pub fn destination_tags(
    observation: &Observation<Value>,
    here: Location,
) -> BTreeMap<PlaceId, Vec<Tag>> {
    let mut tags = BTreeMap::new();
    let Some(place) = observation.entity(here.place().entity_id()) else {
        return tags;
    };
    let Some(payload) = place
        .components()
        .iter()
        .find(|record| *record.component_type() == Passages::COMPONENT_TYPE)
        .and_then(|record| record.payload_for::<Passages>().ok())
    else {
        return tags;
    };
    let Some(leads) = payload.get("leads_to").and_then(Value::as_array) else {
        return tags;
    };
    for entry in leads {
        let Some(to) = entry
            .get("to")
            .and_then(|value| serde_json::from_value::<PlaceId>(value.clone()).ok())
        else {
            continue;
        };
        let names: Vec<Tag> = entry
            .get("to_tags")
            .and_then(Value::as_array)
            .map(|list| {
                list.iter()
                    .filter_map(Value::as_str)
                    .filter_map(|name| Tag::new(name).ok())
                    .collect()
            })
            .unwrap_or_default();
        tags.insert(to, names);
    }
    tags
}

/// Straight-line distance on the floor, in whole millimetres, rounded down: only to choose whom to
/// walk to (`CLOSE_ENOUGH`), never to measure a stride.
pub fn distance(a: LocalPosition, b: LocalPosition) -> i64 {
    let (dx, dy) = delta(a, b);
    isqrt(dx * dx + dy * dy)
}

fn delta(a: LocalPosition, b: LocalPosition) -> (i64, i64) {
    (
        i64::from(b.x().value()) - i64::from(a.x().value()),
        i64::from(b.y().value()) - i64::from(a.y().value()),
    )
}

/// The integer square root of a non-negative number, rounded down.
fn isqrt(value: i64) -> i64 {
    value.max(0).cast_unsigned().isqrt().cast_signed()
}

/// Where my own disclosed walk goes, if I am walking and the record reads.
pub fn heading(observation: &Observation<Value>) -> Option<Destination> {
    let me = observation.entity(observation.observer())?;
    let record = me
        .components()
        .iter()
        .find(|record| *record.component_type() == Walking::COMPONENT_TYPE)?;
    let walk: DisclosedWalk =
        serde_json::from_value(record.payload_for::<Walking>().ok()?.clone()).ok()?;
    Some(walk.destination)
}

/// Whether the observation discloses the observer's own walk: what a host reads to keep stepping a
/// seat, so that no host decodes a pack's component itself.
pub fn walks(observation: &Observation<Value>) -> bool {
    observation
        .entity(observation.observer())
        .is_some_and(|me| {
            me.components()
                .iter()
                .any(|record| *record.component_type() == Walking::COMPONENT_TYPE)
        })
}
