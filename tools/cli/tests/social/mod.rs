//! Shared by the social acceptance tests (biography, Milestone B, composition): reading the social
//! packs' facts with their owners' published types, and counting them where they happened.
//!
//! The six biographical types are literals here, from `step-09-social.md` SD-12, rather than read from
//! the packs' `BIOGRAPHICAL` constants: this is the oracle a biography is checked against, and an
//! oracle built from the code under test would agree with any mistake in it (rules §25).

#![allow(dead_code)]

use std::collections::BTreeMap;

use mineworld_contracts::{Causation, EntityId, Event, EventEnvelope, PersonId};
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

const BUCKET: i64 = 30 * 86_400;

/// The facts that must occur in **every** 30-day bucket (I-4 as amended by QB-2).
pub const EVERY_BUCKET: [&str; 4] = [
    "group-activity-started",
    "group-activity-ended",
    "invitation-accepted",
    "joined-group-activity",
];

/// The facts a relationship change may be caused by: the four relationships subscribes to.
const SUBSCRIBED: [&str; 4] = [
    "spoke",
    "invitation-accepted",
    "invitation-declined",
    "group-activity-ended",
];

/// Per 30-day bucket, per counted type, how many — with `relationship-changed` split into `…-up` and
/// `…-down` by its own payload, so saturation and decline are both visible.
///
/// A world run to an age of `days` stops at the instant `days × 86 400`, and what falls due at that
/// very instant (an activity's wake) is the last day's: it is counted in the last bucket, not in a
/// bucket of its own past the run.
pub fn per_bucket(facts: &[EventEnvelope], days: i64) -> BTreeMap<i64, BTreeMap<String, u64>> {
    let last = (days * 86_400 - 1) / BUCKET;
    let mut counted: BTreeMap<i64, BTreeMap<String, u64>> = BTreeMap::new();
    for fact in facts {
        let kind = fact.event_type().as_str();
        let name = match kind {
            "relationship-changed" => match decoded::<RelationshipChanged>(fact) {
                Some(change) if change.to() > change.from() => "relationship-changed-up",
                Some(_) => "relationship-changed-down",
                None => continue,
            },
            "became-acquainted" | "invitation-declined" | "left-group-activity" => kind,
            _ if EVERY_BUCKET.contains(&kind) => kind,
            _ => continue,
        };
        *counted
            .entry((fact.at().seconds() / BUCKET).min(last))
            .or_default()
            .entry(name.to_owned())
            .or_default() += 1;
    }
    counted
}

/// The social activity precondition (I-4, amended by QB-2), checked **before** any comparison of
/// runs, with the per-bucket counts printed so relationship saturation is visible rather than hidden:
///
/// ```text
/// every bucket   group-activity-started, -ended, invitation-accepted, joined-group-activity
/// first bucket   became-acquainted and an upward relationship-changed
/// the run        a downward relationship-changed (values move both ways), when `downward` is asked
/// every fact     every relationship fact caused by a fact of a type relationships subscribes to
/// ```
///
/// Returns the number of buckets read, so a caller can see it read what it meant to (`ARC-23`).
pub fn precondition(facts: &[EventEnvelope], days: i64, downward: bool) -> usize {
    let counted = per_bucket(facts, days);
    let buckets = usize::try_from((days * 86_400 + BUCKET - 1) / BUCKET).expect("positive");
    eprintln!("social facts per 30-day bucket (relationship facts saturate: values never decay):");
    for (bucket, counts) in &counted {
        eprintln!(
            "  days {}-{}: {counts:?}",
            bucket * 30 + 1,
            (bucket + 1) * 30
        );
    }
    assert_eq!(
        counted.len(),
        buckets,
        "a bucket with no social fact at all: {counted:?}"
    );
    for (bucket, counts) in &counted {
        for kind in EVERY_BUCKET {
            assert!(
                counts.get(kind).copied().unwrap_or(0) > 0,
                "bucket {bucket}: no {kind} — the precondition fails: {counts:?}"
            );
        }
    }
    let first = &counted[&0];
    for kind in ["became-acquainted", "relationship-changed-up"] {
        assert!(
            first.get(kind).copied().unwrap_or(0) > 0,
            "no {kind} in the first bucket"
        );
    }
    if downward {
        assert!(
            counted
                .values()
                .any(|counts| counts.contains_key("relationship-changed-down")),
            "no downward relationship-changed anywhere: values only ever rose"
        );
    }
    let types: BTreeMap<u64, &str> = facts
        .iter()
        .map(|fact| (fact.id().raw(), fact.event_type().as_str()))
        .collect();
    let mut checked = 0;
    for fact in facts {
        let kind = fact.event_type().as_str();
        if kind != "became-acquainted" && kind != "relationship-changed" {
            continue;
        }
        let Causation::Event(cause) = fact.caused_by() else {
            panic!(
                "#{} {kind} is not caused by a fact: {:?}",
                fact.id().raw(),
                fact.caused_by()
            );
        };
        let cause_type = types.get(&cause.raw()).copied().unwrap_or("missing");
        assert!(
            SUBSCRIBED.contains(&cause_type),
            "#{} {kind} is caused by #{} {cause_type}, which relationships does not subscribe to",
            fact.id().raw(),
            cause.raw()
        );
        checked += 1;
    }
    assert!(checked > 0, "relationship facts were read");
    eprintln!("  every one of {checked} relationship facts is caused by a subscribed fact");
    buckets
}

/// Every fact of a save, decoded.
pub fn facts_of(tables: &crate::headless::Tables) -> Vec<EventEnvelope> {
    tables
        .facts
        .iter()
        .map(|(_, bytes)| {
            mineworld_persistence::format::decode(bytes, "fact").expect("a stored fact decodes")
        })
        .collect()
}
