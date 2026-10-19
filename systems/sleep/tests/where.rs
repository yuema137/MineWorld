//! AC-Na-2: where one may sleep (SD-Na-3), every cell of the design's table, plus the hostel.
//!
//! The table is asked of a three-room world — `flat` (residential), `park` (park), `guest-room`
//! (residential, guest) — with a local and a visitor in each room, each asking to sleep once.

mod support;

use mineworld_contracts::{
    ActionResult, EntityKey, EntityType, LocalPosition, Location, Millimetres, PersonId, PlaceId,
    Rejection, Tag, Tags,
};
use mineworld_conversation::ConversationSystem;
use mineworld_kernel::{Emission, World};
use mineworld_movement::MovementSystem;
use mineworld_presence::{PresenceSystem, arrival};
use mineworld_sleep::{Sleep, SleepSystem};
use support::{Kind, Town, configured, encode};

/// A world of three places — `flat` (residential), `park` (park), `guest-room` (residential, guest) —
/// with one person in each and one visitor (tag `visitor`) in each, configured by `section` and
/// `classes`.
struct Rooms {
    world: World,
    sleepers: Vec<(&'static str, bool, mineworld_contracts::EntityId)>,
}

fn rooms(section: Option<(&str, &str)>) -> Rooms {
    let mut world = World::new();
    world.install(PresenceSystem).expect("presence");
    world.install(MovementSystem).expect("movement");
    world.install(ConversationSystem).expect("conversation");
    world.install(SleepSystem).expect("sleep");
    let mut create = |key: &str, entity_type, tags: &[&str]| {
        world
            .create_authored_entity(
                EntityKey::new(key).expect("a key"),
                entity_type,
                Tags::new(tags.iter().map(|tag| Tag::new(*tag).expect("a tag"))),
                None,
            )
            .expect("created")
    };
    let places: Vec<(&'static str, PlaceId)> = [
        ("flat", &["residential"][..]),
        ("park", &["park"][..]),
        ("guest-room", &["residential", "guest"][..]),
    ]
    .into_iter()
    .map(|(key, tags)| {
        (
            key,
            PlaceId::new(create(key, EntityType::Place, tags), EntityType::Place).expect("a place"),
        )
    })
    .collect();
    let mut people = Vec::new();
    for (key, place) in &places {
        let local = create(&format!("local-{key}"), EntityType::Person, &[]);
        let visitor = create(&format!("visitor-{key}"), EntityType::Person, &["visitor"]);
        people.push((*key, false, local, *place));
        people.push((*key, true, visitor, *place));
    }
    let mut facts: Vec<Emission> = people
        .iter()
        .map(|(_, _, person, place)| {
            let person = PersonId::new(*person, EntityType::Person).expect("a person");
            let spot = Location::in_place(*place).with_local(LocalPosition::on_ground(
                Millimetres::new(1_000),
                Millimetres::new(1_000),
            ));
            arrival(&world.read(), person, spot).expect("admitted")
        })
        .collect();
    if let Some((section, classes)) = section {
        facts.push(configured(section, classes));
    }
    world
        .genesis(support::GENESIS, facts)
        .expect("begins");
    Rooms {
        world,
        sleepers: people
            .into_iter()
            .map(|(place, visitor, person, _)| (place, visitor, person))
            .collect(),
    }
}

/// Whether each (place, visitor?) may sleep, as the dispatch answers.
fn answers(section: Option<(&str, &str)>) -> Vec<(&'static str, bool, bool)> {
    let mut rooms = rooms(section);
    let now = support::GENESIS;
    let mut out = Vec::new();
    for (index, (place, visitor, person)) in rooms.sleepers.clone().into_iter().enumerate() {
        let intent = mineworld_contracts::ActionIntent::new(
            mineworld_contracts::ActionId::from_raw(index as u64 + 1),
            person,
            mineworld_contracts::ActionRecord::new::<Sleep>(encode(&Sleep::new(60))),
            now,
        );
        let result = rooms.world.dispatch(&intent, now).expect("answers");
        let permitted = match result.result() {
            ActionResult::Accepted { .. } => true,
            ActionResult::Rejected(Rejection::PermissionDenied) => false,
            other => panic!("{place}: {other:?}"),
        };
        out.push((place, visitor, permitted));
    }
    out
}

const BENCH: &str = r#"[ { "class": "bench-park", "of": "place", "tag": "park" } ]"#;
const GUEST: &str = r#"[ { "class": "guest-place", "of": "place", "tag": "guest" } ]"#;
const VISITOR: &str = r#"[ { "class": "visitor", "of": "person", "tag": "visitor" } ]"#;

/// (flat, park, guest-room) for a local and for a visitor.
fn table(section: Option<(&str, &str)>) -> [(bool, bool); 3] {
    let got = answers(section);
    let cell = |place: &str, visitor: bool| {
        got.iter()
            .find(|(p, v, _)| *p == place && *v == visitor)
            .expect("asked")
            .2
    };
    ["flat", "park", "guest-room"].map(|place| (cell(place, false), cell(place, true)))
}

#[test]
fn with_no_section_people_sleep_where_the_place_is_residential() {
    assert_eq!(
        table(None),
        [(true, true), (false, false), (true, true)]
    );
}

#[test]
fn a_rule_permitting_a_park_bench_widens_the_tag_rule() {
    let section = r#"{ "rules": [ { "action": "sleep", "place": "bench-park", "effect": "permit" } ] }"#;
    assert_eq!(
        table(Some((section, BENCH))),
        [(true, true), (true, true), (true, true)]
    );
}

#[test]
fn a_rule_forbidding_a_guest_room_narrows_the_tag_rule() {
    let section =
        r#"{ "rules": [ { "action": "sleep", "place": "guest-place", "effect": "forbid" } ] }"#;
    assert_eq!(
        table(Some((section, GUEST))),
        [(true, true), (false, false), (false, false)]
    );
}

#[test]
fn default_forbid_leaves_only_what_a_rule_permits() {
    let section = r#"{ "default": "forbid",
        "rules": [ { "action": "sleep", "place": "bench-park", "effect": "permit" } ] }"#;
    assert_eq!(
        table(Some((section, BENCH))),
        [(false, false), (true, true), (false, false)]
    );
}

#[test]
fn a_rule_for_every_place_permits_everywhere() {
    let section = r#"{ "rules": [ { "action": "sleep", "effect": "permit" } ] }"#;
    assert_eq!(
        table(Some((section, "[]"))),
        [(true, true), (true, true), (true, true)]
    );
}

#[test]
fn a_rule_forbidding_visitors_leaves_everyone_else_to_the_tag_rule() {
    let section = r#"{ "rules": [ { "action": "sleep", "actor": "visitor", "effect": "forbid" } ] }"#;
    assert_eq!(
        table(Some((section, VISITOR))),
        [(true, false), (false, false), (true, false)]
    );
}

/// The hostel: its dorm carries no `residential` tag and a rule permits it; the hall is not a place to
/// sleep; the garden region permits everyone but travellers.
#[test]
fn the_hostel_decides_by_its_own_rules() {
    let mut town = Town::new(Kind::Hostel);
    let (result, _) = town.sleep("ana", 60);
    assert!(support::accepted(&result), "the dorm, by its rule: {result:?}");
    let (result, _) = town.sleep("ben", 60);
    assert!(support::accepted(&result), "travellers too: {result:?}");
    let (result, _) = town.sleep("cy", 60);
    assert_eq!(result, ActionResult::Rejected(Rejection::PermissionDenied));

    let (result, _) = town.walk("cy", "garden");
    assert!(support::accepted(&result));
    assert_eq!(town.place_of("cy"), Some(town.place("garden")));
    let (result, _) = town.sleep("cy", 60);
    assert!(support::accepted(&result), "the garden region: {result:?}");

    let (_, _) = town.wake("ben");
    let (result, _) = town.walk("ben", "garden");
    assert!(support::accepted(&result));
    assert_eq!(town.place_of("ben"), Some(town.place("garden")));
    let (result, _) = town.sleep("ben", 60);
    assert_eq!(
        result,
        ActionResult::Rejected(Rejection::PermissionDenied),
        "no traveller sleeps in the garden"
    );
}
