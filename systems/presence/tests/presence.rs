//! This pack on its own, in a headless world: what an arrival changes, what an observer is told, and
//! what disabling the pack takes away.
//!
//! An external crate, so everything reachable here is everything a World Pack, a server or another
//! System Pack can reach. Every test drives the real pipeline —
//! `ActionIntent → route → validate → resolve → record → reduce` — because the claims are about a
//! composed world and not about the functions in isolation (`ENGINEERING_STANDARDS.md` §§17, 20).
//!
//! The two-pack claims — talking, its spatial refusal, and `AC-2` with a real system — live in
//! `systems/conversation/tests/conversation_and_presence.rs`, on the side of the dependency that
//! points at this crate.

use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionResult, Component, EntityId, EntityKey,
    EntityType, Event, EventEnvelope, LocalPosition, Location, Millimetres, Observation, PlaceId,
    Rejection, Relation, Visibility, WorldTime,
};
use mineworld_kernel::{SystemIdentity, World};
use mineworld_presence::{
    Arrive, Arrived, PerceptionProvider, Presence, PresenceSystem, present_in_declaration,
};
use serde::Serialize;

/// The instant every test in this file works in. Supplied to dispatch, never read from a clock, so
/// that a replayed run produces the same facts (`AC-12`).
const NOW: WorldTime = WorldTime::from_seconds(3_600);

/// A world with this pack installed and nothing else, and the three entities the tests use.
struct Fixture {
    world: World,
    alice: EntityId,
    bob: EntityId,
    cafe: PlaceId,
    promenade: PlaceId,
    next_action: u64,
}

impl Fixture {
    fn new() -> Self {
        let mut world = World::new();
        world.install(PresenceSystem).expect("the pack installs");
        let alice = create(&mut world, "alice", EntityType::Person);
        let bob = create(&mut world, "bob", EntityType::Person);
        let cafe = place(create(&mut world, "cafe", EntityType::Place));
        let promenade = place(create(&mut world, "promenade", EntityType::Place));
        Self {
            world,
            alice,
            bob,
            cafe,
            promenade,
            next_action: 1,
        }
    }

    /// Dispatches one `arrive`, the way a server would: the world allocates the identity, the world
    /// supplies the instant.
    fn arrive(&mut self, person: EntityId, at: Location) -> Vec<EventEnvelope> {
        let id = ActionId::from_raw(self.next_action);
        self.next_action += 1;
        let intent = ActionIntent::new(
            id,
            person,
            ActionRecord::new::<Arrive>(encode(&Arrive::new(at))),
            NOW,
        );
        let dispatched = self
            .world
            .dispatch(&intent, NOW)
            .expect("dispatch answers rather than failing");
        assert!(
            matches!(dispatched.result(), ActionResult::Accepted { .. }),
            "arriving somewhere that exists is accepted, but was {:?}",
            dispatched.result()
        );
        dispatched.events().to_vec()
    }

    /// What this pack tells `observer`, offering its own actions through the same seam every pack
    /// uses.
    fn observation(&self, observer: EntityId) -> Observation<serde_json::Value> {
        mineworld_presence::observe(&self.world, observer, NOW, &[&PresenceSystem])
    }
}

fn create(world: &mut World, key: &str, entity_type: EntityType) -> EntityId {
    world
        .create_entity(
            EntityKey::new(key).expect("a legal authoring name"),
            entity_type,
        )
        .expect("an entity is created")
}

fn place(entity: EntityId) -> PlaceId {
    PlaceId::new(entity, EntityType::Place).expect("a place")
}

fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("a payload encodes")
}

/// A position inside a place, in millimetres: this pack's world has no floats in it.
fn at(place: PlaceId, x: i32, y: i32) -> Location {
    Location::in_place(place).with_local(LocalPosition::on_ground(
        Millimetres::new(x),
        Millimetres::new(y),
    ))
}

/// The edge this pack writes, built the only way an edge can be built — through its declaration.
fn present_in_edge(world: &World, person: EntityId, place: PlaceId) -> Relation {
    Relation::between(
        &present_in_declaration(),
        world.entities().require(person).expect("the person exists"),
        world
            .entities()
            .require(place.entity_id())
            .expect("the place exists"),
    )
    .expect("a legal edge")
}

// ---------------------------------------------------------------------------------------------
// What an arrival changes.
// ---------------------------------------------------------------------------------------------

/// The whole of what `arrive` does, end to end: it records one fact, and reducing that fact writes
/// the position and the edge that says which place it is in.
///
/// The fact is checked as well as the state, because the state *is* a reduction of the fact: if the
/// event's audience or place were wrong, every client and every later perception step would be
/// wrong with it, and the component would still look right.
#[test]
fn arriving_records_a_fact_and_reducing_it_writes_the_position_and_the_edge() {
    let mut fixture = Fixture::new();
    let counter = at(fixture.cafe, 1_000, 0);
    let events = fixture.arrive(fixture.alice, counter);

    assert_eq!(events.len(), 1, "one arrival is one fact");
    let arrival = &events[0];
    assert_eq!(*arrival.event_type(), Arrived::EVENT_TYPE);
    assert_eq!(
        arrival.at(),
        NOW,
        "the instant is the world's, not a clock's"
    );
    assert_eq!(arrival.place(), Some(fixture.cafe));
    assert_eq!(
        *arrival.visibility(),
        Visibility::Place(fixture.cafe),
        "anyone in the place could have seen somebody arrive"
    );
    assert_eq!(
        arrival.provenance().emitted_by(),
        &PresenceSystem::ID,
        "the kernel records who emitted it, not the system"
    );
    let payload: Arrived = serde_json::from_slice(arrival.payload().payload()).expect("it decodes");
    assert_eq!(payload.person().entity_id(), fixture.alice);
    assert_eq!(payload.location(), counter);

    assert_eq!(
        fixture.world.components().get::<Presence>(fixture.alice),
        Some(&Presence::at(counter)),
        "the position is the one the fact stated"
    );
    assert!(
        fixture.world.read().is_related(&present_in_edge(
            &fixture.world,
            fixture.alice,
            fixture.cafe
        )),
        "and the edge that says which place she is in"
    );
    assert_eq!(fixture.world.relations().len(), 1);
}

/// Walking out of the café and along the promenade leaves one edge, not two.
///
/// The regression this pins is a false fact rather than a leak: a stale `present-in` would make Alice
/// present in two places at once, and every reader of the relation graph — perception, a client, a
/// later pack — would believe it.
#[test]
fn arriving_somewhere_else_replaces_the_edge_rather_than_adding_one() {
    let mut fixture = Fixture::new();
    fixture.arrive(fixture.alice, at(fixture.cafe, 0, 0));
    fixture.arrive(fixture.alice, at(fixture.promenade, 4_000, 0));

    assert_eq!(fixture.world.relations().len(), 1, "she is in one place");
    assert!(fixture.world.read().is_related(&present_in_edge(
        &fixture.world,
        fixture.alice,
        fixture.promenade
    )));
    assert!(
        !fixture.world.read().is_related(&present_in_edge(
            &fixture.world,
            fixture.alice,
            fixture.cafe
        )),
        "and no longer in the one she left"
    );
    assert_eq!(
        fixture
            .world
            .components()
            .get::<Presence>(fixture.alice)
            .map(|presence| presence.location().place()),
        Some(fixture.promenade)
    );
}

/// Crossing the room is the same action as crossing the town, at a finer resolution: the position
/// changes and the containment does not.
#[test]
fn moving_within_a_place_updates_the_position_and_keeps_the_one_edge() {
    let mut fixture = Fixture::new();
    fixture.arrive(fixture.alice, at(fixture.cafe, 0, 0));
    let by_the_window = at(fixture.cafe, 2_500, 1_200);
    fixture.arrive(fixture.alice, by_the_window);

    assert_eq!(
        fixture.world.components().get::<Presence>(fixture.alice),
        Some(&Presence::at(by_the_window))
    );
    assert_eq!(fixture.world.relations().len(), 1);
}

/// A place is not a person, and this pack's state is about people.
///
/// The refusal is `NoSupportedInteraction` rather than a system code: the café exists and is
/// reachable, and there is simply no sense in which it arrives somewhere.
#[test]
fn a_place_cannot_arrive_anywhere() {
    let mut fixture = Fixture::new();
    let intent = ActionIntent::new(
        ActionId::from_raw(99),
        fixture.cafe.entity_id(),
        ActionRecord::new::<Arrive>(encode(&Arrive::new(Location::in_place(fixture.promenade)))),
        NOW,
    );
    let dispatched = fixture
        .world
        .dispatch(&intent, NOW)
        .expect("dispatch answers");

    assert_eq!(
        *dispatched.result(),
        ActionResult::Rejected(Rejection::NoSupportedInteraction)
    );
    assert!(dispatched.events().is_empty(), "a refusal records nothing");
    assert_eq!(fixture.world.relations().len(), 0);
}

/// A request this pack cannot read is refused with its own reason code and a detail a client
/// developer can act on — and, being a refusal, changes nothing.
///
/// This is the one place a `Rejection::System` is exercised end to end: the five kernel reasons are
/// facts about the world, and a frame that does not decode is a fact about the request.
#[test]
fn a_payload_this_pack_cannot_read_is_refused_with_its_own_code() {
    let mut fixture = Fixture::new();
    let intent = ActionIntent::new(
        ActionId::from_raw(1),
        fixture.alice,
        ActionRecord::new::<Arrive>(b"{\"location\":\"the cafe\"}".to_vec()),
        NOW,
    );
    let dispatched = fixture
        .world
        .dispatch(&intent, NOW)
        .expect("dispatch answers");

    match dispatched.result() {
        ActionResult::Rejected(Rejection::System { code, detail }) => {
            assert_eq!(code.as_str(), "malformed-payload");
            assert!(
                detail.is_some(),
                "a client developer is told what was wrong"
            );
        }
        other => panic!("expected this pack's own refusal, got {other:?}"),
    }
    assert!(
        fixture
            .world
            .components()
            .get::<Presence>(fixture.alice)
            .is_none()
    );
}

// ---------------------------------------------------------------------------------------------
// What an observer is told.
// ---------------------------------------------------------------------------------------------

/// An observation is what one person was shown, and `INV-13` is that there is nothing else to ask.
///
/// Alice and Bob are in the café and Carol is on the promenade, so the claim has a negative half: a
/// person the observer cannot perceive is absent from every part of the observation, not merely from
/// its entity list.
#[test]
fn an_observation_lists_who_is_here_where_they_are_and_the_edges_this_pack_wrote() {
    let mut fixture = Fixture::new();
    let carol = create(&mut fixture.world, "carol", EntityType::Person);
    let counter = at(fixture.cafe, 0, 0);
    let table = at(fixture.cafe, 1_500, 0);
    fixture.arrive(fixture.alice, counter);
    fixture.arrive(fixture.bob, table);
    fixture.arrive(carol, at(fixture.promenade, 0, 0));

    let observation = fixture.observation(fixture.alice);

    assert_eq!(observation.observer(), fixture.alice);
    assert_eq!(observation.at(), NOW);
    assert_eq!(observation.self_location(), Some(&counter));

    let perceived: Vec<_> = observation
        .entities()
        .iter()
        .map(|entity| entity.id())
        .collect();
    assert_eq!(
        perceived,
        vec![fixture.cafe.entity_id(), fixture.alice, fixture.bob],
        "the place itself, then everybody in it, in identity order"
    );
    assert_eq!(
        observation
            .entity(fixture.bob)
            .and_then(|bob| bob.location()),
        Some(&table),
        "where Bob is, because a client has to draw him somewhere"
    );
    assert_eq!(
        observation
            .entity(fixture.cafe.entity_id())
            .and_then(|cafe| cafe.location()),
        None,
        "a place has no location inside itself"
    );
    assert!(
        observation.entity(carol).is_none(),
        "Carol is elsewhere, and there is nowhere in an observation to ask about her"
    );

    assert_eq!(
        observation.relations(),
        [
            present_in_edge(&fixture.world, fixture.alice, fixture.cafe),
            present_in_edge(&fixture.world, fixture.bob, fixture.cafe),
        ],
        "containment is stated as a relation rather than left to a client to infer"
    );
    assert!(
        !observation.relations().contains(&present_in_edge(
            &fixture.world,
            carol,
            fixture.promenade
        )),
        "including an edge about somebody the observer cannot perceive would widen the view"
    );
}

/// Somebody this pack knows no location for perceives nothing — and is still told they may arrive,
/// because arriving requires nothing of space.
///
/// Both halves matter. The first is that absence of state is treated as ignorance rather than as an
/// error, which is what lets a pack be installed into a world that already has people in it. The
/// second is that an affordance whose requirement is `NONE` does not need a position to be answered,
/// which is the same reason a message or a telephone call will not.
#[test]
fn an_observer_with_no_recorded_location_perceives_nothing_and_may_still_arrive() {
    let fixture = Fixture::new();
    let observation = fixture.observation(fixture.alice);

    assert!(observation.entities().is_empty());
    assert!(observation.relations().is_empty());
    assert_eq!(observation.self_location(), None);

    assert_eq!(observation.affordances().len(), 1);
    let affordance = &observation.affordances()[0];
    assert_eq!(*affordance.action_type(), Arrive::ACTION_TYPE);
    assert_eq!(affordance.target(), None, "arriving is directed at nobody");
    assert!(affordance.is_available());
    assert_eq!(affordance.unavailable_reason(), None);
}

/// Disabling this pack removes its action and its affordance, and leaves the state it owns alone.
///
/// The affordance half is the load-bearing one. Perception asks the *kernel's route map* whether an
/// action is still provided, rather than keeping a list of its own — so a world that disables a pack
/// stops advertising it everywhere, with no edit to the pack that produces observations (`AC-2`).
/// A client would otherwise keep drawing an interaction the server now answers `Unavailable`.
#[test]
fn disabling_this_pack_removes_its_action_and_its_affordance_and_keeps_its_state() {
    let mut fixture = Fixture::new();
    let counter = at(fixture.cafe, 0, 0);
    fixture.arrive(fixture.alice, counter);
    assert_eq!(fixture.observation(fixture.alice).affordances().len(), 1);

    fixture
        .world
        .disable(&PresenceSystem::ID)
        .expect("nothing depends on it in this world");

    let observation = fixture.observation(fixture.alice);
    assert!(
        observation.affordances().is_empty(),
        "an action no enabled system provides is not offered to anybody"
    );

    let intent = ActionIntent::new(
        ActionId::from_raw(50),
        fixture.alice,
        ActionRecord::new::<Arrive>(encode(&Arrive::new(at(fixture.promenade, 0, 0)))),
        NOW,
    );
    let dispatched = fixture
        .world
        .dispatch(&intent, NOW)
        .expect("dispatch answers");
    assert_eq!(*dispatched.result(), ActionResult::Unavailable);
    assert!(dispatched.events().is_empty());

    assert_eq!(
        fixture.world.components().get::<Presence>(fixture.alice),
        Some(&Presence::at(counter)),
        "disabling a pack removes its effect, not its state"
    );
    assert!(
        fixture
            .world
            .components()
            .is_declared(&Presence::COMPONENT_TYPE)
    );
    assert!(fixture.world.systems().is_installed(&PresenceSystem::ID));
    assert!(!fixture.world.systems().is_enabled(&PresenceSystem::ID));

    // The observation still describes the world, because perception is a read and reading is not an
    // action this pack provides.
    assert_eq!(observation.self_location(), Some(&counter));
    assert_eq!(observation.entities().len(), 2);
}

// ---------------------------------------------------------------------------------------------
// The disclosure seam, and the half of it a pack is NOT trusted with.
// ---------------------------------------------------------------------------------------------

/// A pack that answers about somebody other than the subject it was asked about.
///
/// Not a straw man: this is the shape of every honest mistake and of the one dishonest case that
/// matters, because `ARC-8` makes a Tier 1 System Pack a WASM component and a WASM component is not
/// code this repository wrote. It discloses a real component type owned by a real enabled system, so
/// the only thing standing between it and a client is the subject check itself.
struct NamesSomebodyElse {
    /// The entity every record it returns is about, whoever it is asked about.
    about: EntityId,
}

impl PerceptionProvider for NamesSomebodyElse {
    fn discloses(
        &self,
        world: &mineworld_kernel::WorldRead<'_>,
        _observer: EntityId,
        _subject: EntityId,
    ) -> Vec<mineworld_contracts::ComponentRecord<serde_json::Value>> {
        let Some(presence) = world.component::<Presence>(self.about) else {
            return Vec::new();
        };
        vec![mineworld_contracts::ComponentRecord::new::<Presence>(
            self.about,
            serde_json::to_value(presence).expect("a component serializes"),
        )]
    }
}

/// A record about anybody but the subject is dropped, so a pack cannot leak a third party's state
/// through a disclosure about somebody else.
///
/// Carol is in the promenade, and Alice is in the café: Alice does not perceive her, and an
/// observation is exactly the list of what was exposed. A pack that answers about Carol however it
/// is asked is therefore trying to tell Alice something she was not shown — the leak the check
/// exists for, and the one a WASM pack could attempt without this crate being able to read its code
/// (`ARC-8`).
///
/// Two halves, and the second is what makes the first mean something. A provider answering about the
/// subject it was asked about is believed, so the disclosure path is working and this test is not
/// passing because nothing reaches a client at all. The same provider answering about Carol
/// discloses nothing: not attached to her, not attached to anybody.
///
/// What it returns is a real `Presence`, owned by this pack, which is installed and enabled — so
/// neither of perception's other two filters can be what dropped it. Only the subject check can,
/// which is the point.
#[test]
fn a_pack_that_names_a_third_party_discloses_nothing() {
    let mut fixture = Fixture::new();
    fixture.arrive(fixture.alice, at(fixture.cafe, 0, 0));
    fixture.arrive(fixture.bob, at(fixture.cafe, 900, 0));
    let carol = create(&mut fixture.world, "carol", EntityType::Person);
    fixture.arrive(carol, at(fixture.promenade, 0, 0));
    let (alice, bob) = (fixture.alice, fixture.bob);

    // The positive control: asked about Bob, a provider that answers about Bob is believed — and the
    // same provider's answer is dropped for every other subject, which is the filter working in the
    // same frame.
    let honest = NamesSomebodyElse { about: bob };
    let believed = mineworld_presence::observe(
        &fixture.world,
        alice,
        NOW,
        &[&PresenceSystem, &honest as &dyn PerceptionProvider],
    );
    assert_eq!(
        believed
            .entity(bob)
            .expect("bob is perceived")
            .components()
            .len(),
        1,
        "a disclosure about the subject reaches the observation",
    );
    assert!(
        believed
            .entity(alice)
            .expect("alice perceives herself")
            .components()
            .is_empty(),
        "and the same provider's answer about Bob is not attached to Alice",
    );

    // And the case that must not reach anybody: a subject this observer was never shown.
    let lying = NamesSomebodyElse { about: carol };
    let refused = mineworld_presence::observe(
        &fixture.world,
        alice,
        NOW,
        &[&PresenceSystem, &lying as &dyn PerceptionProvider],
    );
    assert!(
        refused.entity(carol).is_none(),
        "carol is in another place and is not perceived, which is what makes her a third party",
    );
    for perceived in refused.entities() {
        assert!(
            perceived.components().is_empty(),
            "a record naming a third party is dropped rather than attached to anybody: {} carries \
             {:?}",
            perceived.id(),
            perceived.components(),
        );
    }
}

/// The offer is made against nobody and nobody else: this pack answers for its own action through
/// the same seam every other pack uses, so `offers` is asked with each candidate target too.
#[test]
fn this_pack_offers_its_action_once_and_not_against_a_target() {
    let mut fixture = Fixture::new();
    fixture.arrive(fixture.alice, at(fixture.cafe, 0, 0));
    fixture.arrive(fixture.bob, at(fixture.cafe, 100, 0));
    let read = fixture.world.read();

    assert_eq!(
        PresenceSystem.offers(&read, fixture.alice, None).len(),
        1,
        "offered once, with no target"
    );
    assert!(
        PresenceSystem
            .offers(&read, fixture.alice, Some(fixture.bob))
            .is_empty(),
        "arriving is not something one does to somebody"
    );
    assert_eq!(
        fixture.observation(fixture.alice).affordances().len(),
        1,
        "and two people in the room do not multiply it"
    );
}

// ---------------------------------------------------------------------------------------------
// The structural claim.
// ---------------------------------------------------------------------------------------------

/// Neither another pack's vocabulary nor a floating-point number appears in this crate's sources.
///
/// Two claims that are both **absences**, which is why one structural test owns them and no
/// behavioural test can.
///
/// Perception produces affordances without knowing what any action is, so the day somebody adds
/// `if action == "talk"` here, this pack has stopped being composable — and nothing else in the suite
/// would notice, because the behaviour would be identical.
///
/// And this is the pack that compares positions, by handing them to the contract layer's evaluator.
/// One `f32` in a position would make two runs of one world diverge on a different platform, which
/// `AC-12` forbids and which no test that runs on one machine would catch
/// (`contracts/tests/spatial.rs` makes the same claim about the vocabulary itself).
///
/// It scans `src/` and not `tests/`, because this file has to name the forbidden words in order to
/// look for them.
#[test]
fn no_other_packs_vocabulary_and_no_floating_point_appear_in_this_crate() {
    let sources = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let forbidden = ["talk", "conversation", "spoke", "utterance", "f32", "f64"];
    let mut scanned = 0;
    let mut offences = Vec::new();

    for entry in std::fs::read_dir(&sources).expect("the crate's src/ must be readable") {
        let path = entry.expect("a directory entry must be readable").path();
        if path.extension().is_none_or(|extension| extension != "rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("a source file must be readable");
        for (number, line) in text.lines().enumerate() {
            let lowered = line.to_lowercase();
            for word in forbidden {
                if lowered.contains(word) {
                    offences.push(format!(
                        "{}:{}: {}",
                        path.display(),
                        number + 1,
                        line.trim()
                    ));
                }
            }
        }
        scanned += 1;
    }

    assert!(scanned >= 7, "every module is scanned, not a stale subset");
    assert!(
        offences.is_empty(),
        "this pack must know neither another pack's vocabulary nor a float:\n{}",
        offences.join("\n")
    );
}
