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
        vec![key("cafe"), key("street")],
    );
    assert_eq!(
        pack.people().keys().cloned().collect::<Vec<_>>(),
        vec![key("alice"), key("bob"), key("visitor"), key("wanderer")],
    );
    assert_eq!(
        pack.seats().iter().cloned().collect::<Vec<_>>(),
        vec![key("alice"), key("visitor"), key("wanderer")],
        "three seats: two Persons a client occupies, and the one an agent drives — nothing about \
         the world distinguishes them (INV-1)",
    );
}

#[test]
fn the_world_is_the_one_the_yaml_describes() {
    let world = loaded();
    let read = world.world().read();

    assert_eq!(world.world().entities().len(), 6, "two places, four people");
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
        (key("cafe"), EntityId::from_raw(1)),
        (key("street"), EntityId::from_raw(2)),
        (key("alice"), EntityId::from_raw(3)),
        (key("bob"), EntityId::from_raw(4)),
        (key("visitor"), EntityId::from_raw(5)),
        (key("wanderer"), EntityId::from_raw(6)),
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
        5,
        "the café's front door, then one arrival per person the pack placed",
    );
}

#[test]
fn an_authored_position_is_a_recorded_fact_rather_than_a_write() {
    let world = loaded();

    // The passage comes first — a fact about places, which exist before anybody is in them — and it
    // is movement's, stated by movement.
    let door = &world.genesis()[0];
    assert_eq!(
        *door.event_type(),
        EventTypeId::new("passage-opened").expect("a legal event type")
    );
    assert_eq!(door.provenance().emitted_by().as_str(), "movement");
    assert_eq!(*door.caused_by(), Causation::WorldGenesis);

    for (nth, person) in ["alice", "bob", "visitor"].into_iter().enumerate() {
        let event = &world.genesis()[nth + 1];
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
        (1200, 2400, 0),
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
            == 4,
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
    // decide what the world permits. Bob is 1.8 m from Alice and the visitor is 4.0 m away, against
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
