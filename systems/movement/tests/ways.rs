//! `ways`: the way from where a person stands to each place they can reach, told to that person only
//! (`ARC-281`, SD-LE-3, AC-LE-9's `ways` half).
//!
//! ```text
//! lengths     straight segments, observer → first doorway → doorway to doorway; 0 in the destination
//! unmodelled  a side with no doorway position counts 0: a lower bound
//! order       ascending (doors, place); the observer's own place not listed
//! absent      an unreachable place; the 33rd place beyond the cap of 32
//! audience    the observer about themself; nobody else is told another's ways
//! ```

use mineworld_contracts::{
    Component, EntityId, EntityKey, EntityType, LocalPosition, Location, Millimetres, Observation,
    PersonId, PlaceId, WorldTime,
};
use mineworld_kernel::{Emission, World};
use mineworld_movement::{DISCLOSED_WAYS, MovementSystem, Way, Ways, passage};
use mineworld_presence::{PerceptionProvider, PresenceSystem, arrival, observe};

const NOW: WorldTime = WorldTime::from_seconds(0);

fn local(x: i32, y: i32) -> Option<LocalPosition> {
    Some(LocalPosition::on_ground(
        Millimetres::new(x),
        Millimetres::new(y),
    ))
}

struct Built {
    world: World,
    places: Vec<PlaceId>,
    people: Vec<EntityId>,
}

/// A world of `places` places and `people` people, every person at (0, 0) in the first place, with
/// the passages `joins` makes from the place list.
fn build(places: usize, people: usize, joins: impl FnOnce(&[PlaceId]) -> Vec<Emission>) -> Built {
    let mut world = World::new();
    world.install(PresenceSystem).expect("presence");
    world.install(MovementSystem).expect("movement");
    let mut create = |key: String, kind| {
        world
            .create_entity(EntityKey::new(&key).expect("a key"), kind)
            .expect("created")
    };
    let places: Vec<PlaceId> = (0..places)
        .map(|n| {
            PlaceId::new(
                create(format!("place-{n}"), EntityType::Place),
                EntityType::Place,
            )
            .expect("a place")
        })
        .collect();
    let people: Vec<EntityId> = (0..people)
        .map(|n| create(format!("person-{n}"), EntityType::Person))
        .collect();
    let mut facts = joins(&places);
    for person in &people {
        let person = PersonId::new(*person, EntityType::Person).expect("a person");
        let at = Location::in_place(places[0]).with_local(local(0, 0).expect("a point"));
        facts.push(arrival(&world.read(), person, at).expect("admitted"));
    }
    world.genesis(NOW, facts).expect("begins");
    Built {
        world,
        places,
        people,
    }
}

fn observation(built: &Built, observer: EntityId) -> Observation<serde_json::Value> {
    let providers: [&dyn PerceptionProvider; 2] = [&PresenceSystem, &MovementSystem];
    observe(&built.world, observer, NOW, &providers)
}

/// The `ways` record on `subject` as `observer` is told it, decoded.
fn ways(built: &Built, observer: EntityId, subject: EntityId) -> Option<Vec<Way>> {
    let seen = observation(built, observer);
    let record = seen
        .entity(subject)?
        .components()
        .iter()
        .find(|record| *record.component_type() == Ways::COMPONENT_TYPE)?
        .payload()
        .clone();
    Some(
        serde_json::from_value::<Ways>(record)
            .expect("a ways record decodes")
            .0,
    )
}

fn way(place: PlaceId, doors: u8, length: i32) -> Way {
    Way {
        place,
        doors,
        length: Millimetres::new(length),
    }
}

#[test]
fn lengths_are_straight_segments_doorway_to_doorway_and_unmodelled_sides_count_nothing() {
    // 0 (here, at 0,0) ─ door (3000,4000)|(0,0) ─ 1 ─ door (6000,8000)|(1,1) ─ 2
    // 0 ─ unmodelled ─ 3 ─ (unmodelled | 500,500) ─ 4 ;   5 is joined to nothing.
    let built = build(6, 1, |p| {
        vec![
            passage(p[0], local(3_000, 4_000), p[1], local(0, 0)),
            passage(p[1], local(6_000, 8_000), p[2], local(1, 1)),
            passage(p[0], None, p[3], None),
            passage(p[3], None, p[4], local(500, 500)),
        ]
    });
    let me = built.people[0];
    let p = &built.places;
    assert_eq!(
        ways(&built, me, me).expect("told my ways"),
        [
            way(p[1], 1, 5_000),
            way(p[3], 1, 0),
            way(p[2], 2, 15_000),
            way(p[4], 2, 0),
        ],
        "by (doors, place); a lower bound where a doorway is not modelled; place 5 unreachable; \
         my own place not listed"
    );
}

#[test]
fn at_most_thirty_two_ways_the_nearest_first() {
    // A hub with 33 neighbours, one of which leads on to a 35th place.
    let built = build(35, 1, |p| {
        let mut joins: Vec<Emission> = (1..=33).map(|n| passage(p[0], None, p[n], None)).collect();
        joins.push(passage(p[1], None, p[34], None));
        joins
    });
    let me = built.people[0];
    let told = ways(&built, me, me).expect("told my ways");
    assert_eq!(told.len(), DISCLOSED_WAYS);
    let expected: Vec<PlaceId> = built.places[1..=32].to_vec();
    assert_eq!(
        told.iter().map(|way| way.place).collect::<Vec<_>>(),
        expected,
        "the 33rd neighbour and the place two doors away are beyond the cap"
    );
}

#[test]
fn a_person_is_told_their_own_ways_and_not_another_persons() {
    let built = build(2, 2, |p| vec![passage(p[0], None, p[1], None)]);
    let (me, other) = (built.people[0], built.people[1]);
    assert!(ways(&built, me, me).is_some());
    assert!(
        observation(&built, other).entity(me).is_some(),
        "the other perceives me"
    );
    assert!(ways(&built, other, me).is_none(), "and is not told my ways");
}

#[test]
fn nowhere_to_go_is_no_record() {
    let built = build(2, 1, |_| Vec::new());
    let me = built.people[0];
    assert!(ways(&built, me, me).is_none(), "absence, not an empty list");
}
