//! Loading the real World Pack, and asserting the world it produced.
//!
//! Not a parser test. `worlds/social-cafe` is the pack the vertical slice ships, this file loads *that
//! directory* off the disk, and every assertion is about the world that came out: who exists, what
//! identity each authored key got, where the people are, what the log says about how they got there,
//! and what the world answers when somebody speaks. A test that read a YAML string and compared it to
//! a struct would pass while the world was wrong.
//!
//! ```text
//! A1  the pack loads, and the world is the one the YAML describes
//! A2  entity keys resolve to ids deterministically, twice identically
//! ```
//!
//! A3 — refusals — is `refusals.rs`. A4 — the command — is `tools/cli/tests/`.

use std::collections::BTreeMap;

use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionResult, Causation, EntityId, EntityKey,
    EntityType, EventTypeId, LifecycleState, Location, PersonId, Rejection, Tag, WorldTime,
};
use mineworld_conversation::{ConversationHistory, ConversationSystem, Talk, Utterance};
use mineworld_kernel::SystemIdentity;
use mineworld_presence::{PerceptionProvider, Presence, PresenceSystem, observe, present_in};
use mineworld_worldpack::{Capability, LoadedWorld, WorldPack};

/// The pack every test in this file loads: the real one, from the repository.
const PACK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../worlds/social-cafe");

fn key(value: &str) -> EntityKey {
    EntityKey::new(value).expect("a legal authoring key")
}

fn tag(value: &str) -> Tag {
    Tag::new(value).expect("a legal tag")
}

fn pack() -> WorldPack {
    WorldPack::read(PACK).expect("the repository's own World Pack reads")
}

fn loaded() -> LoadedWorld {
    pack()
        .load(WorldTime::EPOCH)
        .expect("the repository's own World Pack loads")
}

/// Where the loaded world says somebody is.
fn where_is(world: &LoadedWorld, who: &str) -> Option<Location> {
    let id = world.id(&key(who))?;
    world
        .world()
        .read()
        .component::<Presence>(id)
        .map(Presence::location)
}

#[test]
fn the_pack_says_what_world_it_is() {
    let pack = pack();

    assert_eq!(pack.id(), "social-cafe", "a pack's id is its directory");
    assert_eq!(pack.name(), "Social Café");
    assert_eq!(
        pack.systems(),
        [
            Capability::Presence,
            Capability::Movement,
            Capability::Conversation
        ],
        "in the order the pack states, which is installation order",
    );
    assert_eq!(
        pack.places().keys().cloned().collect::<Vec<_>>(),
        ["apartments", "cafe", "park", "store", "street", "workplace"].map(key),
    );
    assert_eq!(
        pack.people().keys().cloned().collect::<Vec<_>>(),
        [
            "alice", "bob", "carol", "dev", "erin", "felix", "grace", "hana", "ivan", "otto",
            "visitor", "wanderer",
        ]
        .map(key),
    );
    assert_eq!(
        pack.seats().iter().cloned().collect::<Vec<_>>(),
        [
            "alice", "bob", "carol", "dev", "erin", "felix", "grace", "hana", "ivan", "visitor",
            "wanderer",
        ]
        .map(key),
        "every Person but one is a seat: the Persons a client occupies, the one an agent drives and \
         the town a headless run drives — nothing about the world distinguishes them (INV-1)",
    );
}

#[test]
fn the_world_is_the_one_the_yaml_describes() {
    let world = loaded();
    let read = world.world().read();

    assert_eq!(
        world.world().entities().len(),
        18,
        "six places, twelve people"
    );
    let systems = world.world().systems();
    for system in [PresenceSystem::ID, ConversationSystem::ID] {
        assert!(
            systems.is_installed(&system) && systems.is_enabled(&system),
            "{system} is installed and enabled",
        );
    }

    let cafe = read
        .entity(world.id(&key("cafe")).expect("the café resolves"))
        .expect("the café exists");
    assert_eq!(cafe.entity_type(), EntityType::Place);
    assert!(
        cafe.tags().contains(&tag("cafe")) && cafe.tags().contains(&tag("public")),
        "the place carries the tags the pack authored: {:?}",
        cafe.tags(),
    );

    let alice = read
        .entity(world.id(&key("alice")).expect("alice resolves"))
        .expect("alice exists");
    assert_eq!(alice.entity_type(), EntityType::Person);
    assert_eq!(alice.lifecycle(), LifecycleState::Active);
    assert!(alice.tags().contains(&tag("barista")));

    // Provenance, so that a world that came out wrong can be traced back to the files that wrote it.
    let provenance = alice
        .metadata()
        .expect("an authored entity records where it came from");
    assert_eq!(provenance.source_pack, "social-cafe");
    assert_eq!(provenance.source_path, "people/alice.yaml");
    assert_eq!(
        provenance.authoring_note.as_deref(),
        Some("Runs the counter. Speaks to whoever comes in."),
    );
}

#[test]
fn entity_keys_resolve_to_ids_deterministically() {
    let first = loaded();
    let second = loaded();

    // Pinned as literals, not merely compared between the two loads: a test that only compared two
    // runs would pass if both were wrong in the same way, and the ids are what every event in this
    // world's log refers to.
    let expected: BTreeMap<EntityKey, EntityId> = [
        (key("apartments"), EntityId::from_raw(1)),
        (key("cafe"), EntityId::from_raw(2)),
        (key("park"), EntityId::from_raw(3)),
        (key("store"), EntityId::from_raw(4)),
        (key("street"), EntityId::from_raw(5)),
        (key("workplace"), EntityId::from_raw(6)),
        (key("alice"), EntityId::from_raw(7)),
        (key("bob"), EntityId::from_raw(8)),
        (key("carol"), EntityId::from_raw(9)),
        (key("dev"), EntityId::from_raw(10)),
        (key("erin"), EntityId::from_raw(11)),
        (key("felix"), EntityId::from_raw(12)),
        (key("grace"), EntityId::from_raw(13)),
        (key("hana"), EntityId::from_raw(14)),
        (key("ivan"), EntityId::from_raw(15)),
        (key("otto"), EntityId::from_raw(16)),
        (key("visitor"), EntityId::from_raw(17)),
        (key("wanderer"), EntityId::from_raw(18)),
    ]
    .into_iter()
    .collect();

    assert_eq!(
        *first.ids(),
        expected,
        "places in key order, then people in key order — the order load.rs states",
    );
    assert_eq!(
        first.ids(),
        second.ids(),
        "the same pack loaded twice resolves the same keys to the same ids (AC-12)",
    );
}

#[test]
fn the_same_pack_loaded_twice_produces_the_same_history() {
    let history = |world: &LoadedWorld| -> Vec<(u64, EventTypeId, Causation, Vec<u8>)> {
        world
            .genesis()
            .iter()
            .map(|event| {
                (
                    event.id().raw(),
                    event.event_type().clone(),
                    event.caused_by().clone(),
                    event.payload().payload().to_vec(),
                )
            })
            .collect()
    };

    let first = loaded();
    let second = loaded();

    assert_eq!(
        history(&first),
        history(&second),
        "identical event identities, kinds, causes and payload bytes — the AC-12 claim at this layer",
    );
    assert_eq!(
        first.genesis().len(),
        17,
        "the five places' doors onto the street, then one arrival per person the pack placed",
    );
}

#[test]
fn an_authored_position_is_a_recorded_fact_rather_than_a_write() {
    let world = loaded();

    // The passages come first — facts about places, which exist before anybody is in them — and they
    // are movement's, stated by movement: one per place that opens onto the street.
    const DOORS: usize = 5;
    for door in &world.genesis()[..DOORS] {
        assert_eq!(
            *door.event_type(),
            EventTypeId::new("passage-opened").expect("a legal event type")
        );
        assert_eq!(door.provenance().emitted_by().as_str(), "movement");
        assert_eq!(*door.caused_by(), Causation::WorldGenesis);
    }

    for (nth, person) in ["alice", "bob", "carol"].into_iter().enumerate() {
        let event = &world.genesis()[nth + DOORS];
        assert_eq!(
            *event.event_type(),
            EventTypeId::new("arrived").expect("a legal event type"),
            "the pack's placements are arrivals, stated by the pack that owns location",
        );
        assert_eq!(
            *event.caused_by(),
            Causation::WorldGenesis,
            "initial state is caused by the world coming into existence (AC-9)",
        );
        assert_eq!(
            event.provenance().controller_decision(),
            None,
            "and by no request: nothing invents an ActionId to seed a world",
        );
        assert_eq!(
            event.provenance().emitted_by(),
            &PresenceSystem::ID,
            "the fact belongs to the system that owns the state it becomes",
        );
        assert_eq!(
            event.subjects(),
            [world.id(&key(person)).expect("the person resolves")],
            "in the pack's own order: {person} is the {nth}th person in key order",
        );
    }

    // And the state that followed from those facts.
    let cafe = world.id(&key("cafe")).expect("the café resolves");
    let alice = where_is(&world, "alice").expect("alice was placed");
    assert_eq!(alice.place().entity_id(), cafe);
    let position = alice.local().expect("the pack authored a position");
    assert_eq!(
        (
            position.x().value(),
            position.y().value(),
            position.z().value()
        ),
        (6000, 8000, 0),
        "millimetres exactly as authored — integers, so a replay elsewhere agrees",
    );
    assert!(
        alice.facing().is_some(),
        "and the orientation the pack authored",
    );

    assert!(
        world
            .world()
            .read()
            .relations_of_type(&present_in())
            .count()
            == 12,
        "the presence pack's own edge for each person it placed",
    );
}

#[test]
fn a_pack_seeds_only_what_it_authored() {
    let world = loaded();

    // `conversation` is installed and owns a history component; nobody has one, because the pack
    // authored no conversations. A loader that "initialised" every component of every enabled system
    // would be inventing content the author did not write.
    for person in ["alice", "bob", "visitor"] {
        let id = world.id(&key(person)).expect("the person resolves");
        assert!(
            world
                .world()
                .read()
                .component::<ConversationHistory>(id)
                .is_none(),
            "{person} starts with nothing remembered",
        );
    }
}

#[test]
fn the_authored_geometry_is_what_the_world_answers_with() {
    let mut world = loaded();
    let alice = world.id(&key("alice")).expect("alice resolves");
    let bob = world.id(&key("bob")).expect("bob resolves");
    let visitor = world.id(&key("visitor")).expect("the visitor resolves");

    // The strongest form of "the world is the one the YAML describes": the positions in the files
    // decide what the world permits. Bob is 2.4 m from Alice and the visitor is 8.6 m away, against
    // conversation's three-metre range — and no test here computes that distance, the world does.
    let mut speak = |actor: EntityId, target: EntityId, nth: u64| -> ActionResult {
        let request = ActionIntent::new(
            ActionId::from_raw(nth),
            actor,
            ActionRecord::new::<Talk>(
                serde_json::to_vec(&Talk::new(
                    Utterance::new("good morning").expect("a legal utterance"),
                ))
                .expect("a payload encodes"),
            ),
            WorldTime::EPOCH,
        )
        .with_target(target);
        world
            .world_mut()
            .dispatch(&request, WorldTime::EPOCH)
            .expect("the conversation system resolves its own action")
            .result()
            .clone()
    };

    assert!(
        matches!(speak(bob, alice, 1), ActionResult::Accepted { .. }),
        "bob is within reach of alice, as the pack placed them",
    );
    assert_eq!(
        speak(visitor, alice, 2),
        ActionResult::Rejected(Rejection::TooFarAway),
        "the visitor starts at the door, out of reach — decided by the system, not by this test",
    );
}

#[test]
fn an_observer_perceives_the_world_through_the_systems_the_pack_enabled() {
    let world = loaded();
    let visitor = world.id(&key("visitor")).expect("the visitor resolves");
    let providers: Vec<&dyn PerceptionProvider> = vec![&PresenceSystem, &ConversationSystem];

    let observation = observe(world.world(), visitor, WorldTime::EPOCH, &providers);

    let perceived: Vec<EntityId> = observation.entities().iter().map(|e| e.id()).collect();
    assert_eq!(
        perceived.len(),
        5,
        "the place, and the four people in it: {perceived:?}",
    );
    assert!(
        observation.self_location().is_some(),
        "and where the observer is, because the pack placed them",
    );

    let talk = Talk::ACTION_TYPE;
    let offered: Vec<(bool, Option<EntityId>)> = observation
        .affordances()
        .iter()
        .filter(|affordance| *affordance.action_type() == talk)
        .map(|affordance| (affordance.is_available(), affordance.target()))
        .collect();
    assert_eq!(
        offered.len(),
        3,
        "talk is offered against the three other people: {offered:?}",
    );
    assert!(
        offered.iter().all(|(available, _)| !*available),
        "and refused for both, because the pack put the visitor at the door: {offered:?}",
    );
}

#[test]
fn a_seat_is_a_person_the_world_can_resolve() {
    let world = loaded();

    for seat in pack().seats() {
        let resolved = world
            .world()
            .read()
            .resolve_key(seat)
            .expect("a seat the pack offers must name an entity the world has");
        let entity = world.world().read().entity(resolved).expect("it exists");
        assert!(
            PersonId::new(entity.id(), entity.entity_type()).is_ok(),
            "and it must be a Person, or a client would connect as something that cannot act",
        );
    }
}

/// `step-09-social.md` SD-3: the town is a star — every place opens onto the street and nothing else —
/// so any place is at most two doors from any other. Read from the doors the movement system recorded
/// at genesis, which are what decides whether a walk between two places is possible, not from the
/// YAML.
#[test]
fn every_place_is_at_most_two_doors_from_any_other() {
    use mineworld_movement::Passages;

    let world = loaded();
    let read = world.world().read();
    let places: Vec<(EntityKey, EntityId)> = pack()
        .places()
        .keys()
        .map(|place| (place.clone(), world.id(place).expect("declared")))
        .collect();
    let name = |id: EntityId| {
        places
            .iter()
            .find(|(_, place)| *place == id)
            .map(|(key, _)| key.as_str().to_owned())
            .expect("a declared place")
    };
    let doors = |id: EntityId| -> Vec<EntityId> {
        read.component::<Passages>(id)
            .map(|passages| passages.iter().map(|p| p.to().entity_id()).collect())
            .unwrap_or_default()
    };

    for (key, id) in &places {
        println!(
            "{key:>10} opens onto {:?}",
            doors(*id).into_iter().map(name).collect::<Vec<_>>()
        );
    }
    for (from, a) in &places {
        for (to, b) in &places {
            let one = doors(*a).contains(b);
            let two = doors(*a).iter().any(|middle| doors(*middle).contains(b));
            assert!(
                a == b || one || two,
                "{from} reaches {to} through at most two doors",
            );
        }
    }
    let street = world.id(&key("street")).expect("declared");
    assert_eq!(
        doors(street).len(),
        places.len() - 1,
        "the street opens onto every other place",
    );
}

/// Where a customer stands at the café counter, in the café's frame: 6.0 m along the frontage from the
/// room's inner west face, 6.2 m in from the inner face of the front wall — just short of the
/// counter's customer face at y = 6 570 mm. Derived from the 3D slice's own constants
/// (`clients/3d-spike/scripts/slice/cafe_interior.gd`: the counter runs from 3.86 m to the east wall,
/// `COUNTER_Z` −7.30 m, `COUNTER_D` 0.78 m), not from where the pack happens to put Alice — this is the
/// spot a player in Demo B walks up to.
const AT_THE_COUNTER: (i32, i32) = (6_000, 6_200);

/// The café's doorway on the café side, where the slice binds its door: 1.61 m from the inner west
/// face (`cafe.gd` `DOOR_X` −2.55 m on a 9 m frontage with 0.34 m walls), 0.2 m into the room.
const IN_THE_DOORWAY: (i32, i32) = (1_610, 200);

/// `step-09-social.md` §2.5, CP-5 — the check the 3D client failed before S8: a person who walks to the
/// counter can talk to Alice, and the same person standing in the doorway cannot.
///
/// The person walks there in `move` strides the server accepts, from wherever the pack seated them,
/// so the positions are reached the way a client reaches them. Whether `talk` is in reach is decided by
/// the conversation system against `INTERACTION_RANGE`; this test computes no distance to Alice. With
/// the pre-S8 café — Alice at (1 200, 2 400), the door on the east wall — the same walk ends 6.1 m from
/// her and the talk is refused, which is the defect this test exists to keep fixed.
#[test]
fn a_person_at_the_counter_can_talk_to_alice() {
    use mineworld_contracts::{LocalPosition, Millimetres, PlaceId};
    use mineworld_movement::{MAX_STRIDE, Move};

    let mut world = loaded();
    let alice = world.id(&key("alice")).expect("alice resolves");
    let visitor = world.id(&key("visitor")).expect("the visitor resolves");
    let cafe = PlaceId::new(world.id(&key("cafe")).expect("declared"), EntityType::Place)
        .expect("a place");
    let seated = where_is(&world, "visitor")
        .and_then(|location| location.local())
        .expect("the pack seats the visitor at a position");
    let mut here = (seated.x().value(), seated.y().value());
    let mut next_action = 0_u64;

    // Straight-line strides of at most MAX_STRIDE, the same division `tools/cli/tests/support` uses:
    // the fewest equal strides such that every consecutive pair is within the published limit.
    let strides = |from: (i32, i32), to: (i32, i32)| -> Vec<(i32, i32)> {
        let (dx, dy) = (i64::from(to.0 - from.0), i64::from(to.1 - from.1));
        let limit = i64::from(MAX_STRIDE.value());
        let point = |k: i64, n: i64| {
            (
                i32::try_from(i64::from(from.0) + dx * k / n).expect("on the map"),
                i32::try_from(i64::from(from.1) + dy * k / n).expect("on the map"),
            )
        };
        let fits = |n: i64| {
            (1..=n).all(|k| {
                let (a, b) = (point(k - 1, n), point(k, n));
                let (sx, sy) = (i64::from(b.0 - a.0), i64::from(b.1 - a.1));
                sx * sx + sy * sy <= limit * limit
            })
        };
        let n = (1_i64..)
            .find(|n| fits(*n))
            .expect("some number of strides fits");
        (1..=n).map(|k| point(k, n)).collect()
    };
    let walk_to = |world: &mut LoadedWorld, here: &mut (i32, i32), to: (i32, i32), id: &mut u64| {
        for point in strides(*here, to) {
            *id += 1;
            let to = Location::in_place(cafe).with_local(LocalPosition::on_ground(
                Millimetres::new(point.0),
                Millimetres::new(point.1),
            ));
            let request = ActionIntent::new(
                ActionId::from_raw(*id),
                visitor,
                ActionRecord::new::<Move>(serde_json::to_vec(&Move::new(to)).expect("encodes")),
                WorldTime::EPOCH,
            );
            let answer = world
                .world_mut()
                .dispatch(&request, WorldTime::EPOCH)
                .expect("dispatch answers");
            assert!(
                matches!(answer.result(), ActionResult::Accepted { .. }),
                "the stride to {point:?} is legal: {:?}",
                answer.result(),
            );
            *here = point;
        }
    };
    let talk = |world: &mut LoadedWorld, id: &mut u64| -> ActionResult {
        *id += 1;
        let id = ActionId::from_raw(*id);
        let request = ActionIntent::new(
            id,
            visitor,
            ActionRecord::new::<Talk>(
                serde_json::to_vec(&Talk::new(
                    Utterance::new("a flat white, please").expect("a legal utterance"),
                ))
                .expect("a payload encodes"),
            ),
            WorldTime::EPOCH,
        )
        .with_target(alice);
        world
            .world_mut()
            .dispatch(&request, WorldTime::EPOCH)
            .expect("the conversation system resolves its own action")
            .result()
            .clone()
    };

    walk_to(&mut world, &mut here, AT_THE_COUNTER, &mut next_action);
    let at_counter = talk(&mut world, &mut next_action);
    walk_to(&mut world, &mut here, IN_THE_DOORWAY, &mut next_action);
    let at_door = talk(&mut world, &mut next_action);
    println!(
        "seated at {:?}; at the counter {AT_THE_COUNTER:?}: {at_counter:?}; in the doorway \
         {IN_THE_DOORWAY:?}: {at_door:?}; alice at {:?}",
        (seated.x().value(), seated.y().value()),
        where_is(&world, "alice").and_then(|l| l.local()),
    );

    assert!(
        matches!(at_counter, ActionResult::Accepted { .. }),
        "a person at the counter can talk to Alice: {at_counter:?}",
    );
    assert_eq!(
        at_door,
        ActionResult::Rejected(Rejection::TooFarAway),
        "and from the doorway Alice is out of reach — decided by the conversation system",
    );
}

/// S6's headline fact in the world the repository ships, not only in a fixture: the visitor walks to
/// the café's front door and out into the street with `move` strides, presence states
/// `person-entered-place` from the café to the street, and the `present-in` edge moves with them.
///
/// Positions are the pack's own literals: the visitor is seated at (1610, 600), the door is at
/// (1610, 200) in the café and (0, 3000) on the street.
#[test]
fn the_visitor_walks_out_of_the_cafe_into_the_street() {
    use mineworld_contracts::{LocalPosition, Millimetres, PlaceId};
    use mineworld_movement::Move;
    use mineworld_presence::PersonEnteredPlace;

    let mut world = loaded();
    let visitor = world.id(&key("visitor")).expect("the visitor resolves");
    let place = |name: &str, world: &LoadedWorld| {
        PlaceId::new(world.id(&key(name)).expect("declared"), EntityType::Place).expect("a place")
    };
    let (cafe, street) = (place("cafe", &world), place("street", &world));
    let at = |place: PlaceId, x: i32, y: i32| {
        Location::in_place(place).with_local(LocalPosition::on_ground(
            Millimetres::new(x),
            Millimetres::new(y),
        ))
    };
    let edges = |world: &LoadedWorld| -> Vec<EntityId> {
        world
            .world()
            .relations()
            .touching(visitor)
            .filter(|edge| *edge.relation_type() == present_in())
            .map(|edge| edge.to())
            .collect()
    };

    let before = edges(&world);
    let mut kinds = Vec::new();
    let mut entered = None;
    for (index, to) in [at(cafe, 1_610, 200), at(street, 500, 3_000)]
        .into_iter()
        .enumerate()
    {
        let request = ActionIntent::new(
            ActionId::from_raw(u64::try_from(index).expect("small") + 1),
            visitor,
            ActionRecord::new::<Move>(serde_json::to_vec(&Move::new(to)).expect("encodes")),
            WorldTime::EPOCH,
        );
        let dispatched = world
            .world_mut()
            .dispatch(&request, WorldTime::EPOCH)
            .expect("dispatch answers");
        assert!(
            matches!(dispatched.result(), ActionResult::Accepted { .. }),
            "{to:?}: {:?}",
            dispatched.result()
        );
        for event in dispatched.events() {
            kinds.push(event.event_type().as_str().to_owned());
            if event.event_type().as_str() == "person-entered-place" {
                entered = Some(event.clone());
            }
        }
    }
    let after = edges(&world);
    println!("facts: {kinds:?}; present-in before {before:?}, after {after:?}");

    assert_eq!(kinds, ["arrived", "arrived", "person-entered-place"]);
    let entered = entered.expect("the occupancy change was recorded");
    assert_eq!(entered.provenance().emitted_by(), &PresenceSystem::ID);
    let fact: PersonEnteredPlace =
        serde_json::from_slice(entered.payload().payload()).expect("it decodes");
    assert_eq!(
        (fact.person().entity_id(), fact.from(), fact.place()),
        (visitor, cafe, street)
    );
    assert_eq!(before, vec![cafe.entity_id()]);
    assert_eq!(after, vec![street.entity_id()]);
    assert_eq!(where_is(&world, "visitor"), Some(at(street, 500, 3_000)));
}
