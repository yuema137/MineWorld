//! The integration checkpoint for PR 05a: two real System Packs in one headless world, driven
//! through the whole pipeline, and the three claims the PR is accepted on.
//!
//! ```text
//! a talk between two people in one place is accepted, emits its facts, and is remembered
//! the same request from too far away is refused TooFarAway and writes nothing
//! disabling ConversationSystem makes talk Unavailable with nothing else changing   ← AC-2
//! ```
//!
//! An external crate, so everything reachable here is what a World Pack or a server can reach. The
//! third claim is `AC-2` with real systems rather than the stubs of
//! `kernel/tests/two_systems.rs`: two worlds differing in **one boolean**, the same script, the same
//! instant, and one copy of every system's code.
//!
//! Two further tests carry weight beyond the acceptance list. One checks that the affordance a client
//! is shown and the answer dispatch gives are the same answer — the property that makes
//! `ENGINEERING_RULES.md` §8 true rather than merely intended. The other checks that a reply
//! continues a conversation instead of starting a second one, which is the rule this pack owns and
//! the one most likely to be broken by a plausible-looking change.

use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionResult, Component, ComponentRecord,
    EntityId, EntityKey, EntityType, Event, EventEnvelope, LocalPosition, Location, Millimetres,
    PersonId, PlaceId, Rejection, Relation, Visibility, WorldTime,
};
use mineworld_conversation::{
    CONVERSATION_GAP, ConversationHistory, ConversationStarted, ConversationSystem,
    REMEMBERED_AT_MOST, Spoke, Talk, Utterance, talk_requirement,
};
use mineworld_kernel::{
    EntityRegistrySnapshot, KernelError, RelationStoreSnapshot, SystemIdentity, World,
};
use mineworld_movement::{Move, MovementSystem};
use mineworld_presence::{Arrived, Presence, PresenceSystem, arrival, present_in_declaration};
use serde::{Deserialize, Serialize};

/// The instant the scripted part of every test happens in. Supplied to dispatch, never read from a
/// clock (`AC-12`).
const NOW: WorldTime = WorldTime::from_seconds(3_600);

/// Conversational distance: inside `talk`'s declared interaction range of three metres.
const ACROSS_A_TABLE: i32 = 900;

/// The far side of the room: outside it, and in the same place, so the refusal is about distance and
/// not about the room.
const ACROSS_THE_ROOM: i32 = 9_000;

// ---------------------------------------------------------------------------------------------
// The composition. This — and only this — is what the AC-2 test changes.
// ---------------------------------------------------------------------------------------------

/// What a World Pack chooses when it composes this world.
///
/// One field, because one field is the claim: a capability is added or removed by **configuration**,
/// not by editing anything. Presence is not optional here, and that is not an oversight —
/// `ConversationSystem` declares a dependency on it, so a world without presence is one the registry
/// refuses to compose rather than one that fails at run time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Composition {
    /// Whether people in this world can speak to each other.
    conversation: bool,
}

/// Composes the world. Identical for every configuration: the same packs are installed in the same
/// order, and the only thing the configuration decides is whether one of them acts.
fn compose(configuration: Composition) -> World {
    let mut world = World::new();
    world.install(PresenceSystem).expect("presence installs");
    // People walk here, so a test can move somebody closer the way a client does: with `move`.
    world
        .install(MovementSystem)
        .expect("movement installs after presence");
    world
        .install(ConversationSystem)
        .expect("conversation installs, its dependency being present and enabled");
    if !configuration.conversation {
        world
            .disable(&ConversationSystem::ID)
            .expect("a pack nothing depends on is disabled by configuration");
    }
    world
}

/// Every pack whose actions may appear in an observation, in the order this world composed them.
///
/// The same list whatever the configuration: a disabled pack is still asked, and its offers are
/// dropped because the kernel's route map no longer provides its actions. Filtering the list here
/// instead would move the `AC-2` decision out of the world's composition and into a caller.
const PROVIDERS: [&dyn mineworld_presence::PerceptionProvider; 3] =
    [&PresenceSystem, &MovementSystem, &ConversationSystem];

// ---------------------------------------------------------------------------------------------
// The world under test, and the script both configurations run.
// ---------------------------------------------------------------------------------------------

/// One composed world, its people, and an action counter standing in for the identity a server
/// allocates.
struct Cafe {
    world: World,
    alice: EntityId,
    bob: EntityId,
    cafe: PlaceId,
    next_action: u64,
    /// Whether the world has been asked anything yet. Before that, placing somebody is genesis.
    running: bool,
    /// The facts genesis recorded, in order.
    genesis: Vec<EventEnvelope>,
}

impl Cafe {
    /// A world composed as `configuration` says, with Alice, Bob and one café in it — and nobody
    /// anywhere yet, because where people are is something the world has to be told.
    fn new(configuration: Composition) -> Self {
        let mut world = compose(configuration);
        let alice = create(&mut world, "alice", EntityType::Person);
        let bob = create(&mut world, "bob", EntityType::Person);
        let cafe = place(create(&mut world, "cafe", EntityType::Place));
        Self {
            world,
            alice,
            bob,
            cafe,
            next_action: 1,
            running: false,
            genesis: Vec::new(),
        }
    }

    /// Places somebody at `location` as the world begins: a genesis fact presence reduces, exactly
    /// as a World Pack places its people (`ARC-15`). Only before the world has been asked anything.
    fn place_at(&mut self, person: EntityId, location: Location) {
        assert!(!self.running, "placement is genesis, before the world runs");
        let person = PersonId::new(person, EntityType::Person).expect("a person");
        let fact = arrival(&self.world.read(), person, location).expect("presence admits it");
        let recorded = self
            .world
            .genesis(NOW, vec![fact])
            .expect("the world begins with this fact");
        self.genesis.extend(recorded);
    }

    /// Dispatches one request the way a server does: the world allocates the identity and supplies
    /// the instant, and the answer comes back with the facts it caused.
    fn submit<A: Action>(
        &mut self,
        actor: EntityId,
        target: Option<EntityId>,
        payload: &A,
        at: WorldTime,
    ) -> (ActionResult, Vec<EventEnvelope>) {
        let id = ActionId::from_raw(self.next_action);
        self.next_action += 1;
        self.running = true;
        let mut intent = ActionIntent::new(id, actor, ActionRecord::new::<A>(encode(payload)), at);
        if let Some(target) = target {
            intent = intent.with_target(target);
        }
        let (result, events, deferred) = self
            .world
            .dispatch(&intent, at)
            .expect("dispatch answers rather than failing")
            .into_parts();
        assert!(
            deferred.is_empty(),
            "neither pack defers work: there is no scheduler in this slice"
        );
        (result, events)
    }

    /// Puts somebody at a position inside the café, along one wall.
    ///
    /// Before the world runs, that is a genesis placement. Afterwards it is a walk: `move` strides of
    /// at most 2 000 mm (a literal, `ARC-23`) from where presence says they are, each of which the
    /// movement system must accept — the way a client walks somebody closer.
    fn stand(&mut self, person: EntityId, millimetres_along: i32) {
        let cafe = self.cafe;
        let at_x = move |x: i32| {
            Location::in_place(cafe).with_local(LocalPosition::on_ground(
                Millimetres::new(x),
                Millimetres::ZERO,
            ))
        };
        if !self.running {
            self.place_at(person, at_x(millimetres_along));
            return;
        }
        let mut x = self
            .world
            .components()
            .get::<Presence>(person)
            .and_then(|presence| presence.location().local())
            .map(|local| local.x().value())
            .expect("somebody already placed walks from where they are");
        while x != millimetres_along {
            x = if millimetres_along > x {
                millimetres_along.min(x + 2_000)
            } else {
                millimetres_along.max(x - 2_000)
            };
            let (result, _) = self.submit(person, None, &Move::new(at_x(x)), NOW);
            assert!(
                matches!(result, ActionResult::Accepted { .. }),
                "a stride of at most 2 m is accepted: {result:?}"
            );
        }
    }

    /// Speaks, and hands back the answer and the facts.
    fn talk(
        &mut self,
        speaker: EntityId,
        listener: EntityId,
        said: &str,
        at: WorldTime,
    ) -> (ActionResult, Vec<EventEnvelope>) {
        let talk = Talk::new(utterance(said));
        self.submit(speaker, Some(listener), &talk, at)
    }

    /// What this world tells `observer`, through the pack that owns perception.
    fn observation(
        &self,
        observer: EntityId,
    ) -> mineworld_contracts::Observation<serde_json::Value> {
        mineworld_presence::observe(&self.world, observer, NOW, &PROVIDERS)
    }

    /// What `observer` is told about `action_type` against `target`, if anything.
    fn affordance(
        &self,
        observer: EntityId,
        action_type: &mineworld_contracts::ActionTypeId,
        target: Option<EntityId>,
    ) -> Option<mineworld_contracts::Affordance<serde_json::Value>> {
        self.observation(observer)
            .affordances()
            .iter()
            .find(|affordance| {
                affordance.action_type() == action_type && affordance.target() == target
            })
            .cloned()
    }

    /// What Bob remembers being told.
    fn history(&self, person: EntityId) -> Option<&ConversationHistory> {
        self.world.components().get::<ConversationHistory>(person)
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

fn utterance(said: &str) -> Utterance {
    Utterance::new(said).expect("a legal utterance")
}

fn decode<T: serde::de::DeserializeOwned>(event: &EventEnvelope) -> T {
    serde_json::from_slice(event.payload().payload()).expect("a payload decodes")
}

fn event_types(events: &[EventEnvelope]) -> Vec<String> {
    events
        .iter()
        .map(|event| event.event_type().as_str().to_owned())
        .collect()
}

/// Everything one world holds that does **not** belong to `ConversationSystem`.
///
/// The instrument the `AC-2` test needs: "nothing else changing" has to be checked against something,
/// and checking a handful of assertions would only prove that the handful did not change. This
/// carries identity, presence, the relation graph and the arrival facts, so a world composed without
/// conversation has to match a world composed with it in every one of them.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct EverythingElse {
    entities: EntityRegistrySnapshot,
    presence: Vec<ComponentRecord<serde_json::Value>>,
    relations: RelationStoreSnapshot,
    arrivals: Vec<serde_json::Value>,
}

impl EverythingElse {
    fn of(world: &World, history: &[EventEnvelope]) -> Self {
        Self {
            entities: EntityRegistrySnapshot::from(world.entities()),
            presence: world
                .components()
                .iter::<Presence>()
                .map(|(entity, presence)| {
                    ComponentRecord::new::<Presence>(
                        entity,
                        serde_json::to_value(presence).expect("a component serializes"),
                    )
                })
                .collect(),
            relations: RelationStoreSnapshot::from(world.relations()),
            arrivals: history
                .iter()
                .filter(|event| *event.event_type() == Arrived::EVENT_TYPE)
                .map(|event| serde_json::to_value(event).expect("a fact serializes"))
                .collect(),
        }
    }
}

/// The script both configurations run, and the only difference between the two runs is the world it
/// is run against.
///
/// It returns every fact the world recorded, in order, which is what the `AC-2` test compares: history
/// is state too.
fn run(cafe: &mut Cafe) -> Vec<EventEnvelope> {
    cafe.stand(cafe.alice, 0);
    cafe.stand(cafe.bob, ACROSS_A_TABLE);
    let mut history = cafe.genesis.clone();

    let (_, spoken) = cafe.talk(cafe.alice, cafe.bob, "good morning", NOW);
    history.extend(spoken);
    history
}

// ---------------------------------------------------------------------------------------------
// Acceptance 1 — a talk is accepted, emits its facts, and is remembered.
// ---------------------------------------------------------------------------------------------

/// Alice speaks to Bob across a table: the request is accepted, two facts are recorded, and Bob
/// remembers being spoken to.
///
/// Every layer of the claim is checked, because each one can be wrong on its own:
///
/// ```text
/// the answer        Accepted, naming the facts it caused
/// the facts         conversation-started then spoke, with the audiences this pack decided
/// the projection    Bob's history holds who spoke, when, and what — and Alice's holds nothing
/// ```
///
/// The last line is the one that makes the component a record of what a person *heard* rather than a
/// log of the exchange, which is what a controller needs in order to answer "who spoke to me".
#[test]
fn a_talk_between_two_people_in_one_place_is_accepted_and_remembered() {
    let mut cafe = Cafe::new(Composition { conversation: true });
    cafe.stand(cafe.alice, 0);
    cafe.stand(cafe.bob, ACROSS_A_TABLE);

    let (result, events) = cafe.talk(cafe.alice, cafe.bob, "good morning", NOW);

    match &result {
        ActionResult::Accepted { events: caused } => assert_eq!(
            caused.len(),
            2,
            "the answer names the facts the request caused"
        ),
        other => panic!("a talk across a table is accepted, got {other:?}"),
    }
    assert_eq!(
        event_types(&events),
        vec!["conversation-started", "spoke"],
        "beginning to talk and saying something are two facts"
    );

    let started: ConversationStarted = decode(&events[0]);
    assert_eq!(started.speaker().entity_id(), cafe.alice);
    assert_eq!(started.listener().entity_id(), cafe.bob);
    assert_eq!(
        *events[0].visibility(),
        Visibility::Participants,
        "two people beginning to talk is between them"
    );

    let spoke: Spoke = decode(&events[1]);
    assert_eq!(spoke.speaker().entity_id(), cafe.alice);
    assert_eq!(spoke.listener().entity_id(), cafe.bob);
    assert_eq!(spoke.utterance().as_str(), "good morning");
    assert_eq!(
        *events[1].visibility(),
        Visibility::Place(cafe.cafe),
        "and anyone in the café could have overheard it"
    );
    assert_eq!(events[1].place(), Some(cafe.cafe));
    assert_eq!(events[1].subjects(), [cafe.bob]);
    assert_eq!(events[1].participants(), [cafe.alice, cafe.bob]);
    assert_eq!(
        events[1].provenance().emitted_by(),
        &ConversationSystem::ID,
        "the kernel records the emitter; the system cannot misstate it"
    );

    let heard = cafe
        .history(cafe.bob)
        .expect("Bob remembers being spoken to")
        .heard()
        .to_vec();
    assert_eq!(heard.len(), 1);
    assert_eq!(heard[0].speaker().entity_id(), cafe.alice);
    assert_eq!(heard[0].at(), NOW, "on the world's clock");
    assert_eq!(heard[0].utterance().as_str(), "good morning");
    assert!(
        cafe.history(cafe.alice).is_none(),
        "the history records what a person heard, not what they said"
    );
}

// ---------------------------------------------------------------------------------------------
// Acceptance 2 — too far away is refused, and nothing is written.
// ---------------------------------------------------------------------------------------------

/// The same request, from nine metres away in the same room, is refused `TooFarAway` — and the world
/// is byte-for-byte what it was before the request.
///
/// Both halves are the claim. The refusal proves the declared
/// [`SpatialRequirement`](mineworld_contracts::SpatialRequirement) is actually evaluated; the
/// unchanged world proves it is evaluated in `validate`, which cannot write, rather than in `resolve`
/// after something has already landed. A system that checked distance while resolving would produce
/// the same rejection *and* a half-written world.
///
/// Same place, deliberately: if the two were in different rooms the refusal would be right for the
/// wrong reason, and the interaction range would go untested.
#[test]
fn the_same_request_from_too_far_away_is_refused_and_writes_nothing() {
    let mut cafe = Cafe::new(Composition { conversation: true });
    cafe.stand(cafe.alice, 0);
    cafe.stand(cafe.bob, ACROSS_THE_ROOM);

    let before = EverythingElse::of(&cafe.world, &[]);
    let (result, events) = cafe.talk(cafe.alice, cafe.bob, "good morning", NOW);

    assert_eq!(result, ActionResult::Rejected(Rejection::TooFarAway));
    assert!(events.is_empty(), "a refusal records no fact");
    assert!(
        cafe.history(cafe.bob).is_none(),
        "and nothing is remembered, because nothing happened"
    );
    assert_eq!(
        EverythingElse::of(&cafe.world, &[]),
        before,
        "a refusal changes nothing at all"
    );

    // The distance is the only reason: the two are in one room, and moving Bob within reach is enough
    // for the identical request to be accepted.
    cafe.stand(cafe.bob, ACROSS_A_TABLE);
    let (result, _) = cafe.talk(cafe.alice, cafe.bob, "good morning", NOW);
    assert!(matches!(result, ActionResult::Accepted { .. }));
}

/// Somebody in another place is refused the same way, and somebody this world knows no position for
/// cannot speak at all.
///
/// `TooFarAway` for the wrong room is the contract layer's decision and worth pinning from a system:
/// from the actor's point of view the difference between the wrong room and the wrong town is only how
/// far they must travel. A speaker with no recorded position is a different answer —
/// `PreconditionFailed` — because no distance was ever established.
#[test]
fn the_wrong_room_is_too_far_and_an_unlocated_speaker_cannot_speak() {
    let mut cafe = Cafe::new(Composition { conversation: true });
    let promenade = place(create(&mut cafe.world, "promenade", EntityType::Place));
    cafe.stand(cafe.alice, 0);
    cafe.place_at(cafe.bob, Location::in_place(promenade));

    let (result, events) = cafe.talk(cafe.alice, cafe.bob, "good morning", NOW);
    assert_eq!(result, ActionResult::Rejected(Rejection::TooFarAway));
    assert!(events.is_empty());

    let carol = create(&mut cafe.world, "carol", EntityType::Person);
    let (result, events) = cafe.talk(carol, cafe.alice, "hello", NOW);
    assert_eq!(
        result,
        ActionResult::Rejected(Rejection::PreconditionFailed)
    );
    assert!(events.is_empty());
}

// ---------------------------------------------------------------------------------------------
// Acceptance 3 — AC-2, with a real system.
// ---------------------------------------------------------------------------------------------

/// **`AC-2`.** Two worlds, composed by the same function from the same packs in the same order. One
/// configuration value differs. In one of them `talk` is an action; in the other it does not exist,
/// asking for it is answered [`ActionResult::Unavailable`], and **everything else is identical**.
///
/// This is `kernel/tests/two_systems.rs`'s claim made against real System Packs: no line of either
/// pack's code differs between the two worlds, because there is only one copy of it. What the second
/// world loses is an action, two kinds of fact, one component and one affordance — and what it keeps
/// is a working world with people standing in a café.
///
/// The affordance half is the part a stub could not have shown. A client is told what it may attempt;
/// if disabling a pack left its affordance in place, every client would keep offering an interaction
/// the server now refuses, and no server-side test would notice.
#[test]
fn disabling_conversation_makes_talk_unavailable_with_nothing_else_changing() {
    // ── The only difference between the two worlds. ──────────────────────────────────────────
    let mut talkative = Cafe::new(Composition { conversation: true });
    let mut silent = Cafe::new(Composition {
        conversation: false,
    });

    let talkative_history = run(&mut talkative);
    let silent_history = run(&mut silent);

    // ── What the configuration changed. ─────────────────────────────────────────────────────
    assert_eq!(
        talkative.world.systems().provider(&Talk::ACTION_TYPE),
        Some(&ConversationSystem::ID)
    );
    assert_eq!(silent.world.systems().provider(&Talk::ACTION_TYPE), None);

    let (answer, events) = silent.talk(silent.alice, silent.bob, "good morning", NOW);
    assert_eq!(
        answer,
        ActionResult::Unavailable,
        "there is no talking in a world with no conversation in it (INV-10)"
    );
    assert!(events.is_empty());
    assert!(
        silent.history(silent.bob).is_none(),
        "and nothing was remembered, because nothing was said"
    );
    assert_eq!(
        event_types(&talkative_history),
        vec!["arrived", "arrived", "conversation-started", "spoke"],
    );
    assert_eq!(
        event_types(&silent_history),
        vec!["arrived", "arrived"],
        "the facts this pack states are not facts in that world"
    );

    // The affordance goes with the action, and the other pack's stays.
    let offered = |cafe: &Cafe| -> Vec<String> {
        cafe.observation(cafe.alice)
            .affordances()
            .iter()
            .map(|affordance| affordance.action_type().as_str().to_owned())
            .collect()
    };
    // Movement offers `walk-to` beside `move` since S15's PR 12n-1 (ARC-73): the other pack's stay.
    assert_eq!(offered(&talkative), vec!["move", "walk-to", "talk"]);
    assert_eq!(offered(&silent), vec!["move", "walk-to"]);

    // ── What it did not change. ─────────────────────────────────────────────────────────────
    assert_eq!(
        EverythingElse::of(&silent.world, &silent_history),
        EverythingElse::of(&talkative.world, &talkative_history),
        "identity, positions, the relation graph and the other pack's facts are the same world"
    );

    // And the disabled pack is still installed, still owns its table, and still holds the write token
    // this world granted it: disabling removes a pack's effect, not its state.
    assert!(silent.world.systems().is_installed(&ConversationSystem::ID));
    assert!(!silent.world.systems().is_enabled(&ConversationSystem::ID));
    assert!(
        silent
            .world
            .components()
            .is_declared(&ConversationHistory::COMPONENT_TYPE)
    );
    assert_eq!(
        silent.world.writers().count(),
        3,
        "presence, movement and conversation each still own their component"
    );

    // Re-enabling is the same boolean read the other way: the world gains the action back, with the
    // state it kept.
    silent
        .world
        .enable(&ConversationSystem::ID)
        .expect("its dependency is installed and enabled");
    let (answer, events) = silent.talk(silent.alice, silent.bob, "good morning", NOW);
    assert!(matches!(answer, ActionResult::Accepted { .. }));
    assert_eq!(event_types(&events), vec!["conversation-started", "spoke"]);
}

// ---------------------------------------------------------------------------------------------
// The affordance and the answer are the same answer.
// ---------------------------------------------------------------------------------------------

/// What a client is shown and what the server answers agree, in every case the slice can produce.
///
/// This is `ENGINEERING_RULES.md` §§8–9 as an executable claim, and it is the reason both paths go
/// through [`SpatialRequirement::evaluate`](mineworld_contracts::SpatialRequirement::evaluate)
/// rather than through two checks that look alike. If they ever disagree, a client greys out an
/// interaction the server would have allowed, or offers one it refuses — and the player experiences
/// the architecture failing.
///
/// The affordance also carries the requirement itself, unevaluated, so a client can *show* what the
/// action needs — three metres, the same room — without implementing the check.
#[test]
fn the_affordance_a_client_is_shown_and_the_answer_dispatch_gives_are_the_same_answer() {
    let mut cafe = Cafe::new(Composition { conversation: true });
    cafe.stand(cafe.alice, 0);
    cafe.stand(cafe.bob, ACROSS_THE_ROOM);

    let far = cafe
        .affordance(cafe.alice, &Talk::ACTION_TYPE, Some(cafe.bob))
        .expect("talking to Bob is something Alice may attempt in principle");
    assert!(!far.is_available());
    assert_eq!(far.unavailable_reason(), Some(&Rejection::TooFarAway));
    assert_eq!(
        *far.requirement(),
        talk_requirement(),
        "the requirement travels to the client unevaluated, so it can be shown"
    );
    assert_eq!(
        far.requirement().within_range(),
        Some(Millimetres::new(3_000))
    );

    let (refused, _) = cafe.talk(cafe.alice, cafe.bob, "good morning", NOW);
    assert_eq!(
        refused,
        ActionResult::Rejected(far.unavailable_reason().cloned().expect("a reason")),
        "the client was told exactly what the server would answer"
    );

    cafe.stand(cafe.bob, ACROSS_A_TABLE);
    let near = cafe
        .affordance(cafe.alice, &Talk::ACTION_TYPE, Some(cafe.bob))
        .expect("still offered");
    assert!(near.is_available());
    assert_eq!(near.unavailable_reason(), None);
    let (accepted, _) = cafe.talk(cafe.alice, cafe.bob, "good morning", NOW);
    assert!(matches!(accepted, ActionResult::Accepted { .. }));

    // Nobody is offered a conversation with themselves, in either direction of the pair.
    assert!(
        cafe.affordance(cafe.alice, &Talk::ACTION_TYPE, Some(cafe.alice))
            .is_none()
    );
    let (self_talk, _) = cafe.talk(cafe.alice, cafe.alice, "good morning", NOW);
    assert_eq!(
        self_talk,
        ActionResult::Rejected(Rejection::NoSupportedInteraction)
    );
}

/// Somebody who has left the world for good is offered as unavailable, and the request is refused for
/// that reason rather than for a distance.
///
/// Availability is this pack's judgement — the kernel must never learn what makes a person
/// unavailable — and the order matters: walking closer cannot make a departed person available, so the
/// contract layer reports availability before distance, and a player is not sent on a pointless
/// journey.
#[test]
fn a_person_who_has_left_the_world_is_unavailable_rather_than_far_away() {
    let mut cafe = Cafe::new(Composition { conversation: true });
    cafe.stand(cafe.alice, 0);
    cafe.stand(cafe.bob, ACROSS_THE_ROOM);
    cafe.world
        .destroy_entity(cafe.bob)
        .expect("an entity is destroyed");

    let offered = cafe
        .affordance(cafe.alice, &Talk::ACTION_TYPE, Some(cafe.bob))
        .expect("Bob is still perceived: this pack still knows a position for him");
    assert!(!offered.is_available());
    assert_eq!(
        offered.unavailable_reason(),
        Some(&Rejection::TargetUnavailable),
        "availability is reported before distance, though he is also across the room"
    );

    let (result, events) = cafe.talk(cafe.alice, cafe.bob, "good morning", NOW);
    assert_eq!(result, ActionResult::Rejected(Rejection::TargetUnavailable));
    assert!(events.is_empty());
}

// ---------------------------------------------------------------------------------------------
// The rule this pack owns: when an exchange begins a conversation.
// ---------------------------------------------------------------------------------------------

/// A reply is part of the same conversation; a greeting an hour later begins a new one.
///
/// The rule lives in the pack that owns the concept, and it is decided from the world's clock and from
/// what the two of them remember hearing — not from a flag somebody has to maintain. The reply half is
/// the one a plausible implementation gets wrong: Alice's history records what Bob said to her and
/// Bob's records what she said to him, so a check in one direction only would make every reply start a
/// second conversation.
#[test]
fn a_reply_continues_the_conversation_and_a_later_greeting_starts_another() {
    let mut cafe = Cafe::new(Composition { conversation: true });
    cafe.stand(cafe.alice, 0);
    cafe.stand(cafe.bob, ACROSS_A_TABLE);

    let (_, opening) = cafe.talk(cafe.alice, cafe.bob, "good morning", NOW);
    assert_eq!(event_types(&opening), vec!["conversation-started", "spoke"]);

    let (_, reply) = cafe.talk(cafe.bob, cafe.alice, "good morning to you", NOW);
    assert_eq!(
        event_types(&reply),
        vec!["spoke"],
        "a reply is part of the conversation they are already having"
    );

    let (_, again) = cafe.talk(cafe.alice, cafe.bob, "about the weather", NOW);
    assert_eq!(event_types(&again), vec!["spoke"]);

    let later = WorldTime::from_seconds(NOW.seconds() + CONVERSATION_GAP.seconds() + 1);
    let (_, after_a_silence) = cafe.talk(cafe.alice, cafe.bob, "still here?", later);
    assert_eq!(
        event_types(&after_a_silence),
        vec!["conversation-started", "spoke"],
        "after a silence longer than the gap, they have begun talking again"
    );

    // Four exchanges, and each is remembered by whoever heard it.
    assert_eq!(
        cafe.history(cafe.bob).map(ConversationHistory::len),
        Some(3)
    );
    assert_eq!(
        cafe.history(cafe.alice).map(ConversationHistory::len),
        Some(1)
    );
}

/// A history keeps the most recent exchanges and forgets the oldest, because it is a projection and
/// not a memory.
///
/// Driven through real dispatches rather than by calling the component's own method: the claim is that
/// a person spoken to a great many times holds a bounded component, which is a property of the pack
/// and not of the `Vec` inside it.
#[test]
fn a_history_remembers_the_most_recent_exchanges_and_forgets_the_oldest() {
    let mut cafe = Cafe::new(Composition { conversation: true });
    cafe.stand(cafe.alice, 0);
    cafe.stand(cafe.bob, ACROSS_A_TABLE);

    let said = REMEMBERED_AT_MOST + 8;
    for line in 1..=said {
        let (result, _) = cafe.talk(cafe.alice, cafe.bob, &format!("line {line}"), NOW);
        assert!(matches!(result, ActionResult::Accepted { .. }));
    }

    let history = cafe.history(cafe.bob).expect("Bob has been spoken to");
    assert_eq!(history.len(), REMEMBERED_AT_MOST);
    assert_eq!(
        history.heard()[0].utterance().as_str(),
        format!("line {}", said - REMEMBERED_AT_MOST + 1),
        "the oldest still remembered"
    );
    assert_eq!(
        history
            .heard()
            .last()
            .expect("a non-empty history")
            .utterance()
            .as_str(),
        format!("line {said}"),
        "and the most recent thing said"
    );
    assert_eq!(
        history
            .last_heard_from(PersonId::new(cafe.alice, EntityType::Person).expect("a person"))
            .map(|heard| heard.at()),
        Some(NOW)
    );
}

// ---------------------------------------------------------------------------------------------
// Composition refusals, decided before any world runs.
// ---------------------------------------------------------------------------------------------

/// A world cannot have conversation without presence, and cannot lose presence while conversation is
/// enabled.
///
/// The declared dependency is what makes both refusals happen at composition time, naming the missing
/// pack, instead of at run time as a `talk` that mysteriously cannot find anybody's position. That is
/// the point of declaring it: space is presence's state, and a pack that reads another's state says so.
#[test]
fn conversation_cannot_be_composed_without_presence() {
    let mut world = World::new();
    let refusal = world
        .install(ConversationSystem)
        .expect_err("its dependency is not installed");
    assert!(
        matches!(
            refusal,
            KernelError::SystemDependencyMissing { ref dependency, .. }
                if *dependency == PresenceSystem::ID
        ),
        "the refusal names what is missing: {refusal}"
    );

    let mut composed = compose(Composition { conversation: true });
    // Movement depends on presence too; disabled first, so the refusal below can only be conversation's.
    composed
        .disable(&MovementSystem::ID)
        .expect("nothing depends on movement");
    let refusal = composed
        .disable(&PresenceSystem::ID)
        .expect_err("conversation depends on it");
    assert!(matches!(
        refusal,
        KernelError::SystemRequiredByAnotherSystem { ref required_by, .. }
            if *required_by == ConversationSystem::ID
    ));
}

// ---------------------------------------------------------------------------------------------
// How Alice remembers, from outside: the disclosure a controller reads (MVP.md §9.2, INV-13).
// ---------------------------------------------------------------------------------------------

/// `MVP.md` §9.2's third arrow: *history → her controller's context*.
///
/// A controller is handed an `Observation` and never the world, so a history it cannot read is a
/// history it does not have. This is the whole of the mechanism: the pack that owns the component
/// discloses it, in the observation of the person whose memory it is.
///
/// The second half of the test is the half that matters, and it is an **absence**: the same history is
/// not in the other person's observation of her. A disclosure that exposed a component because it
/// exists would pass the first assertion and fail this one, which is exactly the difference between
/// `INV-13` holding and `INV-13` being intended.
#[test]
fn a_person_is_told_what_they_have_heard_and_nobody_is_told_what_somebody_else_heard() {
    let mut cafe = Cafe::new(Composition { conversation: true });
    cafe.stand(cafe.alice, 0);
    cafe.stand(cafe.bob, ACROSS_A_TABLE);
    cafe.talk(cafe.alice, cafe.bob, "we open at seven", NOW);

    // Bob is the listener, so the entry is Bob's — and it is in Bob's own observation.
    let disclosed =
        heard_by(&cafe.observation(cafe.bob), cafe.bob).expect("bob is told what bob has heard");
    assert_eq!(disclosed.len(), 1);
    assert_eq!(disclosed.heard()[0].speaker().entity_id(), cafe.alice);
    assert_eq!(
        disclosed.heard()[0].utterance().as_str(),
        "we open at seven",
        "the words, not a summary of them: interpretation is a controller's business"
    );
    assert_eq!(
        Some(&disclosed),
        cafe.history(cafe.bob),
        "and what he is told is what the world holds, not a second account of it"
    );

    // Alice perceives Bob — she is in the same room and the observation lists him — and is told
    // nothing about what he has heard.
    let alices_view = cafe.observation(cafe.alice);
    assert!(
        alices_view.entity(cafe.bob).is_some(),
        "alice perceives bob, which is what makes the next assertion about disclosure and not \
         about perception"
    );
    assert_eq!(
        heard_by(&alices_view, cafe.bob),
        None,
        "what somebody else was told is not hers to read"
    );
    assert_eq!(
        heard_by(&alices_view, cafe.alice),
        None,
        "and she has been told nothing herself, so there is nothing to disclose about her either"
    );
}

/// Disabling the pack removes the disclosure from every observation in the world, with no edit
/// anywhere — the same route `AC-2` takes for an affordance.
///
/// The state itself survives, which is the other half of `AC-2`: a disabled pack stops acting and
/// stops answering, and does not lose what it knows.
#[test]
fn a_disabled_pack_discloses_nothing_and_still_holds_its_state() {
    let mut cafe = Cafe::new(Composition { conversation: true });
    cafe.stand(cafe.alice, 0);
    cafe.stand(cafe.bob, ACROSS_A_TABLE);
    cafe.talk(cafe.alice, cafe.bob, "we open at seven", NOW);
    assert!(heard_by(&cafe.observation(cafe.bob), cafe.bob).is_some());

    cafe.world
        .disable(&ConversationSystem::ID)
        .expect("nothing depends on it");

    assert_eq!(
        heard_by(&cafe.observation(cafe.bob), cafe.bob),
        None,
        "a world that does not have conversation in it does not disclose what anybody heard"
    );
    assert_eq!(
        cafe.history(cafe.bob).map(ConversationHistory::len),
        Some(1),
        "and the state is still there, because disabling a pack is not forgetting"
    );
}

/// The disclosed payload is a value a client can read, not the bytes a log carries.
///
/// `spike/FINDINGS.md` F8.2 measured what the opaque default reaches a client as: an array of byte
/// integers, unusable. A client has no Rust type to decode with, so this is not a convenience.
#[test]
fn a_disclosed_component_is_self_describing_and_labelled_by_its_own_type() {
    let mut cafe = Cafe::new(Composition { conversation: true });
    cafe.stand(cafe.alice, 0);
    cafe.stand(cafe.bob, ACROSS_A_TABLE);
    cafe.talk(cafe.alice, cafe.bob, "we open at seven", NOW);

    let observation = cafe.observation(cafe.bob);
    let record = observation
        .entity(cafe.bob)
        .expect("bob perceives himself")
        .components()
        .iter()
        .find(|record| *record.component_type() == ConversationHistory::COMPONENT_TYPE)
        .expect("the disclosure is there");

    assert_eq!(record.entity(), cafe.bob);
    assert_eq!(record.schema_version(), ConversationHistory::SCHEMA_VERSION);
    let payload = record.payload();
    assert!(
        payload.is_object() && payload["heard"].is_array(),
        "a client reads fields, not a byte array: {payload}"
    );
    assert_eq!(
        payload["heard"][0]["speaker"]["entity"],
        serde_json::json!(cafe.alice.raw().to_string()),
        "and an identity inside a disclosed payload is a decimal string, which is the position \
         FINDINGS.md F2 measured as unprotected before PR 04"
    );
}

/// The disclosed history of one observer, decoded with the real component type.
///
/// Through `payload_for`, so the test cannot read a record that was labelled with a different
/// component type — the same check any other reader of a record gets.
fn heard_by(
    observation: &mineworld_contracts::Observation<serde_json::Value>,
    subject: EntityId,
) -> Option<ConversationHistory> {
    let record = observation
        .entity(subject)?
        .components()
        .iter()
        .find(|record| *record.component_type() == ConversationHistory::COMPONENT_TYPE)?;
    let payload = record
        .payload_for::<ConversationHistory>()
        .expect("a record labelled with this component type");
    Some(
        serde_json::from_value(payload.clone())
            .expect("the pack's own payload, written by the pack"),
    )
}

/// The two packs write only their own state, and the edge one of them declared belongs to it alone.
///
/// The write neither of them can make is not testable here at all: it does not compile, which is where
/// a claim about code that must not exist belongs
/// (`kernel/tests/compile_fail/a_system_cannot_write_another_systems_component.rs`).
#[test]
fn each_pack_writes_its_own_state_and_reads_the_others() {
    let mut cafe = Cafe::new(Composition { conversation: true });
    cafe.stand(cafe.alice, 0);
    cafe.stand(cafe.bob, ACROSS_A_TABLE);
    cafe.talk(cafe.alice, cafe.bob, "good morning", NOW);

    assert_eq!(cafe.world.components().count::<Presence>(), 2);
    assert_eq!(cafe.world.components().count::<ConversationHistory>(), 1);
    assert_eq!(
        cafe.world
            .components()
            .declarations()
            .map(|declaration| declaration.owner().as_str().to_owned())
            .collect::<Vec<_>>(),
        // Conversation owns two since S17's PR IL-b: its history, and its section of the World's
        // Interaction List (ARC-63), which an unconfigured world leaves empty. Movement owns two since
        // S15's PR 12n-1: its passages, and a walker's `walking` (ARC-73), listed by component type.
        vec![
            "conversation",
            "conversation",
            "movement",
            "presence",
            "movement",
        ],
        "each component type owned by the pack that declared it"
    );

    let edge = Relation::between(
        &present_in_declaration(),
        cafe.world
            .entities()
            .require(cafe.alice)
            .expect("alice exists"),
        cafe.world
            .entities()
            .require(cafe.cafe.entity_id())
            .expect("the café exists"),
    )
    .expect("a legal edge");
    assert!(cafe.world.read().is_related(&edge));
    assert_eq!(
        cafe.world
            .relations()
            .declaration(&mineworld_presence::present_in())
            .map(|declaration| declaration.owner().clone()),
        Some(PresenceSystem::ID),
        "the edge type belongs to the pack that declared it, and only it may write one"
    );
}

/// No floating-point number appears in this crate's sources.
///
/// An **absence**, so a structural test owns it and no behavioural test can. This pack declares a
/// distance — `INTERACTION_RANGE` — and a distance in `f32` would make the same request accepted on one
/// platform and refused on another, which is exactly the divergence `AC-12` forbids and which a suite
/// running on one machine would never show. The contract layer makes the same claim about itself in
/// `contracts/tests/spatial.rs`.
///
/// It scans `src/` and not `tests/`, because this file has to name the forbidden types to look for
/// them.
#[test]
fn no_floating_point_appears_in_this_crate() {
    let sources = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut scanned = 0;
    let mut offences = Vec::new();

    for entry in std::fs::read_dir(&sources).expect("the crate's src/ must be readable") {
        let path = entry.expect("a directory entry must be readable").path();
        if path.extension().is_none_or(|extension| extension != "rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("a source file must be readable");
        for (number, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or_default();
            if code.contains("f32") || code.contains("f64") {
                offences.push(format!(
                    "{}:{}: {}",
                    path.display(),
                    number + 1,
                    line.trim()
                ));
            }
        }
        scanned += 1;
    }

    assert!(scanned >= 7, "every module is scanned, not a stale subset");
    assert!(
        offences.is_empty(),
        "no distance in this pack may be a float:\n{}",
        offences.join("\n")
    );
}
