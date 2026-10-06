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
//! points at this crate; walking, with its spatial rules, lives in `systems/movement/tests/`.
//!
//! This pack provides no action (`DECISIONS.md` `ARC-26`), so people are moved here by `Mover`, a
//! test-only system that depends on presence and states presence's `arrived` through the checked
//! constructor [`arrival`] — the pattern from presence's side, with no movement rule in the way.

use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionResult, ActionTypeId, Causation, Component,
    EntityId, EntityKey, EntityType, Event, EventEnvelope, LocalPosition, Location, Millimetres,
    Observation, PersonId, PlaceId, Rejection, Relation, SystemId, Visibility, WorldTime,
};
use mineworld_kernel::{
    Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion, World,
    WorldView,
};
use mineworld_presence::{
    Arrived, PerceptionProvider, PersonEnteredPlace, Presence, PresenceSystem, admit, arrival,
    present_in_declaration,
};
use serde::{Deserialize, Serialize};

/// The instant every test in this file works in. Supplied to dispatch, never read from a clock, so
/// that a replayed run produces the same facts (`AC-12`).
const NOW: WorldTime = WorldTime::from_seconds(3_600);

/// A world with this pack and `Mover` installed, and the four entities the tests use.
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
        world
            .install(Mover)
            .expect("Mover depends on presence, so it may state presence's fact");
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

    /// Has `Mover` put `person` at `at`, through presence's checked constructor, the way a server
    /// dispatches: the world allocates the identity and supplies the instant.
    fn relocate(&mut self, person: EntityId, at: Location) -> Vec<EventEnvelope> {
        let id = ActionId::from_raw(self.next_action);
        self.next_action += 1;
        let put = Put {
            person: PersonId::new(person, EntityType::Person).expect("a person"),
            to: at,
            raw: false,
        };
        let intent = ActionIntent::new(id, person, ActionRecord::new::<Put>(encode(&put)), NOW);
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

/// The whole of what an arrival does, end to end: it records one fact, and reducing that fact writes
/// the position and the edge that says which place it is in.
///
/// The fact is checked as well as the state, because the state *is* a reduction of the fact: if the
/// event's audience or place were wrong, every client and every later perception step would be
/// wrong with it, and the component would still look right.
#[test]
fn arriving_records_a_fact_and_reducing_it_writes_the_position_and_the_edge() {
    let mut fixture = Fixture::new();
    let counter = at(fixture.cafe, 1_000, 0);
    let events = fixture.relocate(fixture.alice, counter);

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
        &Mover::ID,
        "the kernel records who stated it — the deciding system — while the fact stays presence's"
    );
    assert_eq!(Arrived::OWNER, PresenceSystem::ID);
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
    fixture.relocate(fixture.alice, at(fixture.cafe, 0, 0));
    fixture.relocate(fixture.alice, at(fixture.promenade, 4_000, 0));

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
    fixture.relocate(fixture.alice, at(fixture.cafe, 0, 0));
    let by_the_window = at(fixture.cafe, 2_500, 1_200);
    fixture.relocate(fixture.alice, by_the_window);

    assert_eq!(
        fixture.world.components().get::<Presence>(fixture.alice),
        Some(&Presence::at(by_the_window))
    );
    assert_eq!(fixture.world.relations().len(), 1);
}

/// The action this pack used to provide, as a client would still send it.
#[derive(Serialize, Deserialize)]
struct RetiredArrive {
    location: Location,
}

impl Action for RetiredArrive {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("arrive");
    const OWNER: SystemId = PresenceSystem::ID;
}

/// This pack provides no action: `arrive` is retired (`DECISIONS.md` `ARC-26`), and a request for it
/// is answered `Unavailable` by the world, with nothing recorded and nobody moved.
///
/// The point is the bypass it closes. An action that relocated anybody anywhere, checking only who
/// and where, would make any distance rule another system declares decorative — the same client
/// would get the refused position one request later.
#[test]
fn this_pack_provides_no_action_and_an_arrive_request_is_unavailable() {
    let mut fixture = Fixture::new();
    let declared = fixture
        .world
        .composition()
        .into_iter()
        .find(|record| record.declaration.system() == &PresenceSystem::ID)
        .expect("presence is installed");
    assert!(
        declared.declaration.provides().is_empty(),
        "presence declares no action: {:?}",
        declared.declaration.provides()
    );

    let intent = ActionIntent::new(
        ActionId::from_raw(99),
        fixture.alice,
        ActionRecord::new::<RetiredArrive>(encode(&RetiredArrive {
            location: at(fixture.promenade, 0, 0),
        })),
        NOW,
    );
    let dispatched = fixture
        .world
        .dispatch(&intent, NOW)
        .expect("dispatch answers");

    assert_eq!(*dispatched.result(), ActionResult::Unavailable);
    assert!(dispatched.events().is_empty(), "a refusal records nothing");
    assert_eq!(
        fixture.world.components().get::<Presence>(fixture.alice),
        None
    );
}

/// Occupancy changes are presence's facts: a change of place is stated as `person-entered-place`,
/// caused by the `arrived`, stated by presence whoever decided the move, naming where the person came
/// from. A first placement at genesis and a move within a place state none.
#[test]
fn entering_another_place_is_stated_by_presence_and_nothing_else_is() {
    let mut fixture = Fixture::new();
    let alice = PersonId::new(fixture.alice, EntityType::Person).expect("a person");
    let placed = arrival(&fixture.world.read(), alice, at(fixture.cafe, 0, 0)).expect("admitted");
    let genesis = fixture.world.genesis(NOW, vec![placed]).expect("begins");
    let kinds = |events: &[EventEnvelope]| -> Vec<String> {
        events
            .iter()
            .map(|event| event.event_type().as_str().to_owned())
            .collect()
    };
    println!("genesis: {:?}", kinds(&genesis));
    assert_eq!(
        kinds(&genesis),
        ["arrived"],
        "a first placement enters nothing"
    );

    let within = fixture.relocate(fixture.alice, at(fixture.cafe, 1_500, 0));
    println!("within the café: {:?}", kinds(&within));
    assert_eq!(
        kinds(&within),
        ["arrived"],
        "a move within a place enters nothing"
    );

    let out = fixture.relocate(fixture.alice, at(fixture.promenade, 0, 0));
    println!("onto the promenade: {:?}", kinds(&out));
    assert_eq!(kinds(&out), ["arrived", "person-entered-place"]);
    let (arrived, entered) = (&out[0], &out[1]);
    assert_eq!(entered.provenance().emitted_by(), &PresenceSystem::ID);
    assert_eq!(*entered.caused_by(), Causation::Event(arrived.id()));
    assert_eq!(*entered.visibility(), Visibility::Place(fixture.promenade));
    assert_eq!(entered.place(), Some(fixture.promenade));
    let fact: PersonEnteredPlace =
        serde_json::from_slice(entered.payload().payload()).expect("it decodes");
    assert_eq!(
        (fact.person(), fact.place(), fact.from()),
        (alice, fixture.promenade, fixture.cafe)
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
    fixture.relocate(fixture.alice, counter);
    fixture.relocate(fixture.bob, table);
    fixture.relocate(carol, at(fixture.promenade, 0, 0));

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

/// Somebody this pack knows no location for perceives nothing, and is offered nothing by this pack.
///
/// Absence of state is treated as ignorance rather than as an error, which is what lets a pack be
/// installed into a world that already has people in it. And this pack offers no action of its own:
/// what a person may attempt — walking, talking — is offered by the packs that provide it.
#[test]
fn an_observer_with_no_recorded_location_perceives_nothing_and_is_offered_nothing_here() {
    let fixture = Fixture::new();
    let observation = fixture.observation(fixture.alice);

    assert!(observation.entities().is_empty());
    assert!(observation.relations().is_empty());
    assert_eq!(observation.self_location(), None);
    assert!(observation.affordances().is_empty());
}

/// Disabling this pack — once the one system depending on it is disabled first, as the registry
/// requires — leaves the state it owns alone, and stops it reducing anything.
///
/// A disabled owner reduces nothing: a request that would state an arrival is `Unavailable` because
/// the deciding system is disabled too, and the position stays what it was. The observation still
/// describes the world, because perception is a read and reading is not an action.
#[test]
fn disabling_this_pack_keeps_its_state() {
    let mut fixture = Fixture::new();
    let counter = at(fixture.cafe, 0, 0);
    fixture.relocate(fixture.alice, counter);

    assert!(
        fixture.world.disable(&PresenceSystem::ID).is_err(),
        "a system another enabled system depends on cannot be disabled"
    );
    fixture
        .world
        .disable(&Mover::ID)
        .expect("nothing depends on Mover");
    fixture
        .world
        .disable(&PresenceSystem::ID)
        .expect("and now nothing enabled depends on presence");

    let observation = fixture.observation(fixture.alice);
    let put = Put {
        person: PersonId::new(fixture.alice, EntityType::Person).expect("a person"),
        to: at(fixture.promenade, 0, 0),
        raw: false,
    };
    let intent = ActionIntent::new(
        ActionId::from_raw(50),
        fixture.alice,
        ActionRecord::new::<Put>(encode(&put)),
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

    // The observation still describes the world, because perception is a read.
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
    fixture.relocate(fixture.alice, at(fixture.cafe, 0, 0));
    fixture.relocate(fixture.bob, at(fixture.cafe, 900, 0));
    let carol = create(&mut fixture.world, "carol", EntityType::Person);
    fixture.relocate(carol, at(fixture.promenade, 0, 0));
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

/// This pack offers nothing, against nobody and against anybody: it provides no action.
#[test]
fn this_pack_offers_nothing_with_or_without_a_target() {
    let mut fixture = Fixture::new();
    fixture.relocate(fixture.alice, at(fixture.cafe, 0, 0));
    fixture.relocate(fixture.bob, at(fixture.cafe, 100, 0));
    let read = fixture.world.read();

    assert!(PresenceSystem.offers(&read, fixture.alice, None).is_empty());
    assert!(
        PresenceSystem
            .offers(&read, fixture.alice, Some(fixture.bob))
            .is_empty()
    );
    assert!(fixture.observation(fixture.alice).affordances().is_empty());
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
    // Walking is another pack's too: presence owns where people are and never names who moves them.
    let forbidden = [
        "talk",
        "conversation",
        "spoke",
        "utterance",
        "movement",
        "passage",
        "stride",
        "f32",
        "f64",
    ];
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

// ---------------------------------------------------------------------------------------------
// The owner still decides (`DECISIONS.md` `ARC-26`, clarification 2).
// ---------------------------------------------------------------------------------------------

/// A system that decides where somebody goes and states it in this pack's vocabulary, as `ARC-26`
/// allows: it depends on presence and declares `arrived`. It can state the fact the contract's way,
/// through [`arrival`], or bypass the constructor and build the payload itself — the second is the
/// defect the reduction check exists for.
struct Mover;

impl SystemIdentity for Mover {
    const ID: SystemId = SystemId::from_static("test-mover");
}

/// `Mover`'s one request: put `person` at `to`, built `raw` or through the constructor.
#[derive(Serialize, Deserialize)]
struct Put {
    person: PersonId,
    to: Location,
    raw: bool,
}

impl Action for Put {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("test-put");
    const OWNER: SystemId = Mover::ID;
}

impl System for Mover {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([PresenceSystem::ID])
            .providing::<Put>()
            .emitting::<Arrived>()
    }

    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let put: Put = serde_json::from_slice(intent.payload().payload()).expect("a Put");
        if put.raw {
            // The bypass: presence's payload, built without asking presence.
            return Ok(vec![
                Emission::new::<Arrived>(
                    encode(&Arrived::new(put.person, put.to)),
                    Visibility::Place(put.to.place()),
                )
                .about(vec![put.person.entity_id()]),
            ]);
        }
        let fact = arrival(&world.read(), put.person, put.to)
            .expect("the contract's way refuses before anything is built");
        Ok(vec![fact])
    }
}

fn put(
    fixture: &mut Fixture,
    person: EntityId,
    to: Location,
    raw: bool,
) -> Result<ActionResult, KernelError> {
    let id = ActionId::from_raw(fixture.next_action);
    fixture.next_action += 1;
    let person = PersonId::new(person, EntityType::Person).expect("a person");
    let intent = ActionIntent::new(
        id,
        person.entity_id(),
        ActionRecord::new::<Put>(encode(&Put { person, to, raw })),
        NOW,
    );
    fixture
        .world
        .dispatch(&intent, NOW)
        .map(|dispatched| dispatched.result().clone())
}

/// The state presence owns, as bytes: every component row and every edge.
fn owned_state(world: &World) -> Vec<u8> {
    let snapshot = world.snapshot().expect("a snapshot");
    encode(&(snapshot.components, snapshot.relations))
}

/// The constructor is the first place presence decides: a destroyed person, a place that is not a
/// place, and a place this world never allocated are refused before any fact exists, and the living
/// case is built.
#[test]
fn the_constructor_refuses_what_presence_may_not_hold_and_builds_the_rest() {
    let mut fixture = Fixture::new();
    let bob = PersonId::new(fixture.bob, EntityType::Person).expect("a person");
    let alice = PersonId::new(fixture.alice, EntityType::Person).expect("a person");
    let counter = at(fixture.cafe, 0, 0);

    let built =
        arrival(&fixture.world.read(), alice, counter).expect("a living person, a real place");
    assert_eq!(built.owner(), &PresenceSystem::ID);

    // A person entity named as if it were a place: the typed id carries the claim, the world refutes it.
    let not_a_place =
        PlaceId::new(fixture.bob, EntityType::Place).expect("the id trusts its claim");
    let never_allocated =
        PlaceId::new(EntityId::from_raw(999), EntityType::Place).expect("the id trusts its claim");
    for (case, location) in [
        ("a person as the place", Location::in_place(not_a_place)),
        (
            "a place this world never allocated",
            Location::in_place(never_allocated),
        ),
    ] {
        assert_eq!(
            arrival(&fixture.world.read(), alice, location).err(),
            Some(Rejection::PreconditionFailed),
            "{case}"
        );
    }

    fixture
        .world
        .destroy_entity(fixture.bob)
        .expect("bob is destroyed");
    assert_eq!(
        arrival(&fixture.world.read(), bob, counter).err(),
        Some(Rejection::PreconditionFailed),
        "a destroyed person cannot be anywhere"
    );
    assert_eq!(admit(&fixture.world.read(), alice, counter), Ok(()));
}

/// The reduction is the second: a system that bypasses the constructor and states an `arrived` for a
/// destroyed person is refused by presence as the owner, and presence writes nothing. The same
/// system stating a valid arrival the same raw way is reduced — the positive control that makes the
/// refusal the check, not a path that never worked.
#[test]
fn presence_refuses_to_reduce_an_arrival_it_may_not_hold_and_writes_nothing() {
    let mut fixture = Fixture::new();
    let counter = at(fixture.cafe, 0, 0);
    let (alice, bob) = (fixture.alice, fixture.bob);

    let accepted = put(&mut fixture, alice, counter, true).expect("a valid raw arrival reduces");
    assert!(
        matches!(accepted, ActionResult::Accepted { .. }),
        "{accepted:?}"
    );
    assert_eq!(
        fixture.world.components().get::<Presence>(alice),
        Some(&Presence::at(counter)),
        "positive control: presence reduced a fact another system stated"
    );
    let outside = at(fixture.promenade, 0, 0);
    let accepted = put(&mut fixture, alice, outside, false).expect("the constructor's way reduces");
    assert!(
        matches!(accepted, ActionResult::Accepted { .. }),
        "{accepted:?}"
    );
    assert_eq!(
        fixture.world.components().get::<Presence>(alice),
        Some(&Presence::at(outside))
    );

    fixture.world.destroy_entity(bob).expect("bob is destroyed");
    let before = owned_state(&fixture.world);
    let beside = at(fixture.promenade, 500, 0);
    let refused = put(&mut fixture, bob, beside, true);
    match refused {
        Err(KernelError::FactRefusedByOwner {
            system,
            event_type,
            reason,
        }) => {
            assert_eq!(system, PresenceSystem::ID);
            assert_eq!(event_type, Arrived::EVENT_TYPE);
            assert_eq!(reason, Rejection::PreconditionFailed);
        }
        other => panic!("presence must refuse as the owner, but dispatch returned {other:?}"),
    }
    assert_eq!(
        fixture.world.components().get::<Presence>(bob),
        None,
        "a refused arrival writes no position"
    );
    assert_eq!(
        owned_state(&fixture.world),
        before,
        "no component row and no edge changed"
    );
}
