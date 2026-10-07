//! Shared by the social acceptance tests (biography, Milestone B, composition): reading the social
//! packs' facts with their owners' published types, and counting them where they happened.
//!
//! The six biographical types are literals here, from `step-09-social.md` SD-12, rather than read from
//! the packs' `BIOGRAPHICAL` constants: this is the oracle a biography is checked against, and an
//! oracle built from the code under test would agree with any mistake in it (rules §25).

#![allow(dead_code)]

use mineworld_contracts::{EntityId, Event, EventEnvelope, PersonId};
use mineworld_group_activity::{
    GroupActivityEnded, GroupActivityStarted, JoinedGroupActivity, LeftGroupActivity,
};
use mineworld_relationships::{BecameAcquainted, RelationshipChanged};

pub const BIOGRAPHICAL_TYPES: [&str; 6] = [
    "became-acquainted",
    "relationship-changed",
    "group-activity-started",
    "joined-group-activity",
    "left-group-activity",
    "group-activity-ended",
];

/// A fact's payload as its owner's type `E`, if it is an `E`.
pub fn decoded<E: Event>(fact: &EventEnvelope) -> Option<E> {
    serde_json::from_slice(fact.payload().payload_for::<E>().ok()?).ok()
}

/// Whether one of the six biographical facts names `person` **in its own payload**; [`None`] for any
/// other kind of fact.
pub fn payload_names(fact: &EventEnvelope, person: EntityId) -> Option<bool> {
    let is = |who: PersonId| who.entity_id() == person;
    let among = |people: &[PersonId]| people.iter().any(|who| is(*who));
    match fact.event_type().as_str() {
        "became-acquainted" => decoded::<BecameAcquainted>(fact)
            .map(|fact| is(fact.person()) || is(fact.counterpart())),
        "relationship-changed" => decoded::<RelationshipChanged>(fact)
            .map(|fact| is(fact.person()) || is(fact.counterpart())),
        "group-activity-started" => {
            decoded::<GroupActivityStarted>(fact).map(|fact| among(fact.members()))
        }
        "group-activity-ended" => {
            decoded::<GroupActivityEnded>(fact).map(|fact| among(fact.members()))
        }
        "joined-group-activity" => {
            decoded::<JoinedGroupActivity>(fact).map(|fact| is(fact.person()))
        }
        "left-group-activity" => decoded::<LeftGroupActivity>(fact).map(|fact| is(fact.person())),
        _ => None,
    }
}
