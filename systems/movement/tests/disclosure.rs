//! A Person can see where the doorways of the room they stand in are — and only that (step-08 C2).
//!
//! A controller is handed an observation and never the world (`INV-13`), so a door it is not told
//! about is a door it cannot use. Movement therefore discloses a place's `Passages` on the place's
//! own perceived entity. These tests read the observation exactly as a controller does.

mod support;

use mineworld_contracts::{
    Component, EntityId, LocalPosition, Millimetres, Observation, PerceivedEntity,
};
use mineworld_kernel::SystemIdentity;
use mineworld_movement::{MovementSystem, Passages};
use serde_json::Value;
use support::{CAFE_DOOR, Layout, Movement, STREET_DOOR, Town, at};

/// The passages record on `subject` in `observation`, decoded, if there is one.
fn passages_on(observation: &Observation<Value>, subject: EntityId) -> Option<Passages> {
    let entity: &PerceivedEntity<Value> = observation.entity(subject)?;
    let record = entity
        .components()
        .iter()
        .find(|record| *record.component_type() == Passages::COMPONENT_TYPE)?;
    let value = record
        .payload_for::<Passages>()
        .expect("labelled as passages");
    Some(serde_json::from_value(value.clone()).expect("a passages record decodes"))
}

fn local((x, y): (i32, i32)) -> LocalPosition {
    LocalPosition::on_ground(Millimetres::new(x), Millimetres::new(y))
}

#[test]
fn a_person_in_the_cafe_is_told_where_its_door_is_and_where_it_leads() {
    let town = Town::joined(|town| at(town.cafe, 4_000, 2_000));
    let seen = town.observation(town.visitor);

    let passages = passages_on(&seen, town.cafe.entity_id())
        .expect("the café's doorways are disclosed on the café");
    let ways: Vec<_> = passages.iter().collect();
    assert_eq!(ways.len(), 1, "one doorway: {ways:?}");
    assert_eq!(ways[0].to(), town.street);
    assert_eq!(ways[0].here(), Some(local(CAFE_DOOR)));
    assert_eq!(ways[0].there(), Some(local(STREET_DOOR)));

    // Disclosed on the place, never on a person in it.
    for person in [town.visitor, town.alice] {
        assert!(
            passages_on(&seen, person).is_none(),
            "no doorway record is attached to person {person}",
        );
    }
}

#[test]
fn the_street_is_not_described_to_somebody_standing_in_the_cafe() {
    let town = Town::joined(|town| at(town.cafe, 4_000, 2_000));
    let seen = town.observation(town.visitor);
    assert!(
        seen.entity(town.street.entity_id()).is_none(),
        "the street is not perceived from inside the café",
    );

    let outside = Town::joined(|town| at(town.street, 500, 2_000));
    let seen = outside.observation(outside.visitor);
    let passages = passages_on(&seen, outside.street.entity_id())
        .expect("the street's side of the same doorway is disclosed on the street");
    let ways: Vec<_> = passages.iter().collect();
    assert_eq!(ways.len(), 1);
    assert_eq!(ways[0].to(), outside.cafe);
    assert_eq!(ways[0].here(), Some(local(STREET_DOOR)));
}

#[test]
fn a_place_with_no_doorway_discloses_nothing_and_a_disabled_movement_discloses_no_doorways() {
    let unjoined = Town::new(Movement::Enabled, Layout::CafeOnly, |town| {
        at(town.cafe, 4_000, 2_000)
    });
    let seen = unjoined.observation(unjoined.visitor);
    assert!(seen.entity(unjoined.cafe.entity_id()).is_some());
    assert!(
        passages_on(&seen, unjoined.cafe.entity_id()).is_none(),
        "absence, not an empty record",
    );

    // Located first: the joined café does disclose its door...
    let mut joined = Town::joined(|town| at(town.cafe, 4_000, 2_000));
    assert!(passages_on(&joined.observation(joined.visitor), joined.cafe.entity_id()).is_some());
    // ...and with movement disabled by configuration the passages are still in state, but a
    // disabled system's state is not disclosed: perception's own AC-2 route, with no edit here.
    joined
        .world
        .disable(&MovementSystem::ID)
        .expect("nothing depends on movement");
    assert!(
        joined
            .world
            .components()
            .get::<Passages>(joined.cafe.entity_id())
            .is_some(),
        "the state is still there",
    );
    assert!(passages_on(&joined.observation(joined.visitor), joined.cafe.entity_id()).is_none());
}
