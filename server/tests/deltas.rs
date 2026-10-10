//! CA-9: `ObservationDelta` against hand-reviewed golden cases (`PROTOCOL.md` §5.3, step-12 §17).
//!
//! Each file under `tests/frames/deltas/` holds `{ "base", "delta", "next" }`: two observations of
//! one observer and the delta between them, as reviewed against `PROTOCOL.md` §5.3's table. The
//! server's `diff(base, next)` must write the file's delta, and `apply(base, delta)` must give the
//! file's next — so the files, not the code, are the oracle, and a client in another language (the
//! GDScript `delta.gd`) checks its applier against the same files.

use std::path::PathBuf;

use mineworld_contracts::{
    ActionTypeId, Affordance, Causation, EntityId, EntityType, Event, EventEnvelope, EventId,
    EventRecord, EventSchemaVersion, EventTypeId, Location, Observation, PerceivedEntity, PlaceId,
    Provenance, SpatialRequirement, SystemId, Visibility, WorldTime,
};
use mineworld_server::WireObservation;
use mineworld_server::protocol::delta::{ObservationDelta, apply, canonical, diff};
use mineworld_server::wire_fact;
use serde_json::Value;

const OBSERVER: u64 = 101;

fn cafe() -> PlaceId {
    PlaceId::new(EntityId::from_raw(1), EntityType::Place).expect("a place")
}

fn person(id: u64) -> PerceivedEntity<Value> {
    PerceivedEntity::new(EntityId::from_raw(id), EntityType::Person).at(Location::in_place(cafe()))
}

fn observation(at: i64, entities: Vec<PerceivedEntity<Value>>) -> WireObservation {
    Observation::new(EntityId::from_raw(OBSERVER), WorldTime::from_seconds(at))
        .at_location(Location::in_place(cafe()))
        .perceiving(entities)
}

fn offer(action: &'static str, target: u64) -> Affordance<Value> {
    Affordance::available(
        ActionTypeId::from_static(action),
        Some(EntityId::from_raw(target)),
        SpatialRequirement::NONE,
    )
}

fn rang() -> mineworld_contracts::PerceivedEvent<Value> {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Rang;
    impl Event for Rang {
        const EVENT_TYPE: EventTypeId = EventTypeId::from_static("rang");
        const OWNER: SystemId = SystemId::from_static("bells");
        const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
    }
    let fact = EventEnvelope::new(
        EventId::from_raw(7),
        WorldTime::from_seconds(4_113),
        EventRecord::new::<Rang>(br#"{"bell":"low"}"#.to_vec()),
        Causation::WorldGenesis,
        Visibility::Public,
        Provenance::new(SystemId::from_static("bells")),
    );
    wire_fact(&fact).expect("renders").0
}

/// The six reviewed cases, as built here.
fn cases() -> Vec<(&'static str, WireObservation, WireObservation)> {
    let moved =
        PerceivedEntity::new(EntityId::from_raw(102), EntityType::Person).at(Location::in_place(
            PlaceId::new(EntityId::from_raw(2), EntityType::Place).expect("a place"),
        ));
    vec![
        (
            "entity-added",
            observation(4_112, vec![person(101)]),
            observation(4_113, vec![person(101), person(102)]),
        ),
        (
            "entity-added-before",
            observation(4_112, vec![person(102)]),
            observation(4_113, vec![person(101), person(102)]),
        ),
        (
            "entity-removed",
            observation(4_112, vec![person(101), person(102)]),
            observation(4_113, vec![person(101)]),
        ),
        (
            "entity-changed",
            observation(4_112, vec![person(101), person(102)]),
            observation(4_113, vec![person(101), moved]),
        ),
        (
            "self-location-to-null",
            observation(4_112, vec![person(101)]),
            Observation::new(EntityId::from_raw(OBSERVER), WorldTime::from_seconds(4_113)),
        ),
        (
            "affordances-reordered",
            observation(4_112, vec![person(101), person(102)])
                .offering(vec![offer("talk", 102), offer("wave", 102)]),
            observation(4_112, vec![person(101), person(102)])
                .offering(vec![offer("wave", 102), offer("talk", 102)]),
        ),
        (
            "events-only",
            observation(4_113, vec![person(101)]),
            observation(4_113, vec![person(101)]).with_events(vec![rang()]),
        ),
    ]
}

fn golden(name: &str) -> (PathBuf, Value) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/frames/deltas")
        .join(format!("{name}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} is missing: {error}", path.display()));
    (path, serde_json::from_str(&text).expect("JSON"))
}

#[test]
fn every_golden_case_is_written_by_diff_and_read_back_by_apply() {
    for (name, base, next) in cases() {
        let (path, file) = golden(name);
        let written = serde_json::json!({
            "base": base,
            "delta": diff(&base, &next),
            "next": next,
        });
        assert_eq!(
            written,
            file,
            "{} no longer matches; the server now writes:\n{}",
            path.display(),
            serde_json::to_string_pretty(&written).expect("printable")
        );
        let held: WireObservation = serde_json::from_value(file["base"].clone()).expect("base");
        let delta: ObservationDelta = serde_json::from_value(file["delta"].clone()).expect("delta");
        let expected: WireObservation = serde_json::from_value(file["next"].clone()).expect("next");
        assert_eq!(
            apply(&held, &delta).expect("applies"),
            canonical(expected),
            "{name}: apply(base, delta) = next"
        );
    }
}

#[test]
fn a_delta_removing_an_entity_not_held_is_an_error() {
    let (held, next) = (observation(1, vec![person(101)]), observation(2, vec![]));
    let delta = diff(&observation(1, vec![person(101), person(102)]), &next);
    assert!(apply(&held, &delta).is_err());
}
