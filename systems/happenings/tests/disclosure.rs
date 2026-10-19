//! T14 (design §5.7): an observer is told the happenings running at their place, and the happenings
//! announced and not yet begun anywhere, on the place they are in and on nothing else; an observer in
//! no place is told nothing.

mod support;

use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, ActionResult, Component, EntityId,
};
use mineworld_happenings::{HappeningsAnnounced, HappeningsHere};
use mineworld_kernel::World;
use mineworld_presence::observe;
use serde_json::Value;
use support::{DAY, HOUR, PlaceMe, key, place, providers, run, t};

const TOWN: &str = "seed: 7
kinds:
  - { id: market-day, title: Saturday market, places: [street], hours: \"08:00-14:00\" }
  - { id: concert, places: [square], hours: \"18:00-20:00\" }
calendar:
  - { kind: market-day, place: street, weekdays: [saturday] }
  - { kind: concert, place: square, weekdays: [saturday] }
";

/// The happening records an observation carries: (entity, type, payload).
fn records(world: &World, observer: EntityId, at: i64) -> Vec<(EntityId, String, Value)> {
    observe(world, observer, t(at), &providers())
        .entities()
        .iter()
        .flat_map(|entity| entity.components())
        .filter(|record| record.component_type().as_str().starts_with("happenings"))
        .map(|record| {
            (
                record.entity(),
                record.component_type().to_string(),
                record.payload().clone(),
            )
        })
        .collect()
}

/// The kinds a record lists.
fn kinds(payload: &Value) -> Vec<String> {
    payload
        .as_array()
        .expect("a list")
        .iter()
        .map(|shown| {
            shown["happening"]["kind"]
                .as_str()
                .expect("a kind")
                .to_owned()
        })
        .collect()
}

#[test]
fn only_an_observer_at_a_place_is_told_its_happenings() {
    // Day 2 (2026-10-10, a Saturday) at 10:00: the market runs on the street; the concert on the
    // square is announced and begins at 18:00.
    let at = 2 * DAY + 10 * HOUR;
    let (mut world, keys, _) = run(Some(TOWN), at);
    let (ada, bo) = (keys[&key("ada")], keys[&key("bo")]);
    let (street, park) = (place(&keys, "street"), place(&keys, "park"));

    assert_eq!(records(&world, bo, at), [], "bo is in no place");

    let told = records(&world, ada, at);
    assert_eq!(told.len(), 2, "{told:?}");
    assert!(
        told.iter()
            .all(|(entity, _, _)| *entity == street.entity_id())
    );
    assert_eq!(told[0].1, HappeningsHere::COMPONENT_TYPE.to_string());
    assert_eq!(kinds(&told[0].2), ["market-day"]);
    assert_eq!(told[0].2[0]["title"], "Saturday market");
    assert_eq!(told[0].2[0]["until"], 2 * DAY + 14 * HOUR);
    assert_eq!(told[1].1, HappeningsAnnounced::COMPONENT_TYPE.to_string());
    assert_eq!(kinds(&told[1].2), ["concert"]);

    let intent = ActionIntent::new(
        ActionId::from_raw(1),
        bo,
        ActionRecord::new::<PlaceMe>(
            serde_json::to_vec(&PlaceMe { place: park }).expect("encodes"),
        ),
        t(at),
    );
    let done = world.dispatch(&intent, t(at)).expect("dispatches");
    assert!(matches!(done.result(), ActionResult::Accepted { .. }));
    let at_park = records(&world, bo, at);
    assert_eq!(at_park.len(), 1, "nothing runs at the park: {at_park:?}");
    assert_eq!(at_park[0].0, park.entity_id());
    assert_eq!(
        at_park[0].1,
        HappeningsAnnounced::COMPONENT_TYPE.to_string()
    );
    assert_eq!(kinds(&at_park[0].2), ["concert"]);

    // After the concert has begun nothing is announced; at 22:00 nothing runs anywhere.
    let late = 2 * DAY + 22 * HOUR;
    let _ = world.advance_to(t(late)).expect("advances");
    assert_eq!(records(&world, ada, late), []);
}
