//! A Person can see where the doorways of the room they stand in are — and only that (step-08 C2).
//!
//! A controller is handed an observation and never the world (`INV-13`), so a door it is not told
//! about is a door it cannot use. Movement therefore discloses a place's `Passages` on the place's
//! own perceived entity. These tests read the observation exactly as a controller does.

mod support;

use mineworld_contracts::{
    Component, EntityId, EntityKey, EntityType, LocalPosition, Location, Millimetres, Observation,
    PerceivedEntity, PersonId, PlaceId,
};
use mineworld_kernel::SystemIdentity;
use mineworld_movement::{MovementSystem, Passages, passage};
use mineworld_presence::{PerceptionProvider, arrival, observe};
use serde_json::{Value, json};
use support::{CAFE_DOOR, Layout, Movement, NOW, STREET_DOOR, Town, at, compose};

/// The raw passages record on `subject` in `observation`, as the wire carries it, if there is one.
fn raw_passages_on(observation: &Observation<Value>, subject: EntityId) -> Option<Value> {
    let entity: &PerceivedEntity<Value> = observation.entity(subject)?;
    let record = entity
        .components()
        .iter()
        .find(|record| *record.component_type() == Passages::COMPONENT_TYPE)?;
    Some(
        record
            .payload_for::<Passages>()
            .expect("labelled as passages")
            .clone(),
    )
}

/// The passages record on `subject` in `observation`, decoded, if there is one.
fn passages_on(observation: &Observation<Value>, subject: EntityId) -> Option<Passages> {
    let value = raw_passages_on(observation, subject)?;
    Some(serde_json::from_value(value).expect("a passages record decodes"))
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

#[test]
fn a_person_in_the_street_is_shown_the_door_sign_of_the_cafe_and_the_street_is_shown_its_own() {
    let outside = Town::joined(|town| at(town.street, 500, 2_000));
    let seen = outside.observation(outside.visitor);
    let raw = raw_passages_on(&seen, outside.street.entity_id())
        .expect("the street's doorway is disclosed on the street");
    let ways = raw["leads_to"].as_array().expect("a list of doorways");
    assert_eq!(ways.len(), 1, "{ways:?}");
    assert_eq!(
        ways[0]["to_tags"],
        json!(["cafe", "public"]),
        "the café's tags, not the observer's own street's: {ways:?}",
    );

    let inside = Town::joined(|town| at(town.cafe, 4_000, 2_000));
    let raw = raw_passages_on(&inside.observation(inside.visitor), inside.cafe.entity_id())
        .expect("the café's doorway is disclosed on the café");
    assert_eq!(raw["leads_to"][0]["to_tags"], json!(["public", "street"]));
}

#[test]
fn a_person_in_the_street_perceives_no_place_but_the_street() {
    let outside = Town::joined(|town| at(town.street, 500, 2_000));
    let seen = outside.observation(outside.visitor);
    for place in [outside.cafe, outside.attic] {
        assert!(
            seen.entity(place.entity_id()).is_none(),
            "{place:?} is not perceived from the street: its door sign is all the street learns",
        );
    }
    assert!(seen.entity(outside.street.entity_id()).is_some());
}

#[test]
fn the_stored_passages_carry_no_destination_tags_and_genesis_is_what_persistence_records() {
    let town = Town::joined(|town| at(town.street, 500, 2_000));
    let stored = town
        .world
        .components()
        .get::<Passages>(town.street.entity_id())
        .expect("the street has passages");
    let value = serde_json::to_value(stored).expect("the stored passages encode");
    for way in value["leads_to"].as_array().expect("a list of doorways") {
        assert!(
            way.get("to_tags").is_none(),
            "to_tags is a disclosure, never stored: {way:?}",
        );
    }
    assert_eq!(
        town.genesis.len(),
        3,
        "one passage and two placements, as persisted.rs states for the same layout",
    );
}

#[test]
fn the_disclosed_passages_decode_to_exactly_the_stored_passages() {
    let town = Town::joined(|town| at(town.street, 500, 2_000));
    let seen = town.observation(town.visitor);
    let stored = town
        .world
        .components()
        .get::<Passages>(town.street.entity_id())
        .cloned()
        .expect("the street has passages");
    assert_eq!(
        passages_on(&seen, town.street.entity_id()),
        Some(stored),
        "the decode paced.rs and agenda.rs use reads the same ways out, with no tags",
    );
}

#[test]
fn a_doorway_to_an_untagged_place_says_so_with_an_empty_list() {
    let (mut world, providers) = compose(Movement::Enabled);
    let mut create = |key: &str| {
        let entity = world
            .create_entity(EntityKey::new(key).expect("a key"), EntityType::Place)
            .expect("created");
        PlaceId::new(entity, EntityType::Place).expect("a place")
    };
    let hall = create("hall");
    let porch = create("porch");
    let visitor = PersonId::new(
        world
            .create_entity(
                EntityKey::new("visitor").expect("a key"),
                EntityType::Person,
            )
            .expect("created"),
        EntityType::Person,
    )
    .expect("a person");
    let placed = arrival(&world.read(), visitor, Location::in_place(hall))
        .expect("presence admits the visitor");
    world
        .genesis(NOW, vec![passage(hall, None, porch, None), placed])
        .expect("the town begins");

    let providers: Vec<&dyn PerceptionProvider> = providers.iter().map(AsRef::as_ref).collect();
    let seen = observe(&world, visitor.entity_id(), NOW, &providers);
    let raw = raw_passages_on(&seen, hall.entity_id()).expect("the hall's doorway is disclosed");
    assert_eq!(raw["leads_to"][0]["to_tags"], json!([]), "{raw:?}");
}
