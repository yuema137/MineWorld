//! A world to host: two stub System Packs, four people, and a perception that scopes by room.
//!
//! The server's acceptance is about the *server*. What it needs from a world is only that the world
//! be real — a real `World`, real systems written the way a System Pack writes one, real dispatch,
//! real events with real `Visibility` — and that two observers legitimately perceive different
//! things. `systems/conversation` and `systems/presence` are another session's; nothing here is a
//! preview of them, and none of it belongs to the production crate.
//!
//! ```text
//! placement   owns Room, emits and reduces `placed`
//! chatter     provides `speak` (an event everyone may learn of) and `whisper` (only the speaker)
//!
//! alice   cafe     bob     street
//! carol   cafe     dave    street
//! seats: alice, bob        so two clients can be two different observers
//! ```
//!
//! The four people are put in their rooms by `World::genesis`: four `placed` facts, recorded as
//! caused by the world coming into existence and reduced into `Room` by the system that owns it.
//! Until PR 05c that was four dispatched intents carrying `ActionId`s allocated from `9_000_000`,
//! because nothing else could write a component — the gap that PR recorded and this one closed.
//!
//! The two events are the point of the pack: `speak` emits with [`Visibility::Public`] and `whisper`
//! with [`Visibility::Participants`], so one dispatch produces a fact both clients are entitled to
//! and another produces a fact only one of them is. A perception that broadcast everything would
//! pass the first assertion and fail the second.

// This module serves two test binaries, and each uses a subset of it: the socket tests do not build
// worlds by hand and the headless tests do not need a client. `dead_code` would otherwise fire on
// whichever half the current binary does not touch.
#![allow(dead_code)]

use mineworld_contracts::{
    Action, ActionIntent, ActionTypeId, Affordance, ComponentRecord, EntityId, EntityKey,
    EntityType, Event, EventEnvelope, EventRecord, EventSchemaVersion, EventTypeId, Observation,
    PerceivedEntity, PerceivedEvent, Rejection, SpatialRequirement, SystemId, Visibility,
    WorldTime,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    World, WorldRead, WorldView, owned_component,
};
use mineworld_server::{
    HostConfig, HostError, HostedWorld, Perception, PerceptionContext, SeatRoster, WireObservation,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// The two seats this world offers, and the two people who are not seats.
pub const ALICE: &str = "alice";
pub const BOB: &str = "bob";
pub const CAROL: &str = "carol";
pub const DAVE: &str = "dave";

/// The two rooms. Alice and Carol are in one; Bob and Dave are in the other.
pub const CAFE: &str = "cafe";
pub const STREET: &str = "street";

pub fn key(value: &str) -> EntityKey {
    EntityKey::new(value).expect("a legal authoring key")
}

/// The seats a client may occupy: two, so that two clients are two different observers.
pub fn seats() -> SeatRoster {
    SeatRoster::new([key(ALICE), key(BOB)])
}

/// A host configuration that streams faster than the default, so a test waits milliseconds.
pub fn brisk() -> HostConfig {
    HostConfig {
        observation_interval: std::time::Duration::from_millis(20),
        ..HostConfig::default()
    }
}

/// Assembles the world. Runs on the world's own thread, which is why it takes nothing and returns
/// everything.
pub fn build() -> Result<HostedWorld, HostError> {
    let mut world = World::new();
    world.install(Placement)?;
    world.install(Chatter)?;

    // Every person, then every fact about them: world assembly states what is true of the world it
    // has just created, and the systems that own that state reduce it.
    let mut facts = Vec::new();
    for (person, room) in [(ALICE, CAFE), (BOB, STREET), (CAROL, CAFE), (DAVE, STREET)] {
        let entity = world.create_entity(key(person), EntityType::Person)?;
        facts.push(placed(entity, room));
    }
    world.genesis(WorldTime::EPOCH, facts)?;

    Ok(HostedWorld::new(world)
        .seating(seats())
        .perceiving(RoomPerception))
}

pub fn payload<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("a payload encodes")
}

// ---------------------------------------------------------------------------------------------
// placement: owns where a person is, and reduces the fact the world begins with.
// ---------------------------------------------------------------------------------------------

pub struct Placement;

impl SystemIdentity for Placement {
    const ID: SystemId = SystemId::from_static("placement");
}

/// Which room a person is in. Owned by `placement`, written by nothing else.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Room {
    pub name: String,
}

owned_component! {
    component = Room,
    owner = Placement,
    component_type = "room",
    schema_version = 1,
}

/// The fact that puts somebody in a room, and the only thing that writes [`Room`].
///
/// This pack provides no action: nothing in this world asks to be placed, because placement is what
/// the world *starts* with. So the fact is stated at genesis and reduced here, which is how a real
/// System Pack makes its component a projection of the log rather than a parallel account of it.
#[derive(Debug, Serialize, Deserialize)]
pub struct Placed {
    pub room: String,
}

impl Event for Placed {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("placed");
    const OWNER: SystemId = Placement::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// One genesis fact: `entity` is in `room`.
pub fn placed(entity: EntityId, room: &str) -> Emission {
    Emission::new::<Placed>(
        payload(&Placed {
            room: room.to_owned(),
        }),
        Visibility::SystemInternal,
    )
    .about(vec![entity])
}

impl System for Placement {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<Room>()
            .emitting::<Placed>()
            .subscribing_to::<Placed>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Room>()
    }

    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if *event.event_type() != Placed::EVENT_TYPE {
            return Ok(Vec::new());
        }
        let fact: Placed = serde_json::from_slice(event.payload().payload())
            .expect("this pack's own payload, written by this pack");
        world.insert(event.subjects()[0], Room { name: fact.room })?;
        Ok(Vec::new())
    }
}

// ---------------------------------------------------------------------------------------------
// chatter: two actions, and two facts with deliberately different audiences.
// ---------------------------------------------------------------------------------------------

pub struct Chatter;

impl SystemIdentity for Chatter {
    const ID: SystemId = SystemId::from_static("chatter");
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Speak {
    pub words: String,
}

impl Action for Speak {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("speak");
    const OWNER: SystemId = Chatter::ID;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Whisper {
    pub words: String,
}

impl Action for Whisper {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("whisper");
    const OWNER: SystemId = Chatter::ID;
}

/// A fact anyone in the world may learn of — and one that carries an `EntityId` **inside its
/// payload**, which is the position `FINDINGS.md` F2 showed a protocol-level encoder cannot reach.
#[derive(Debug, Serialize, Deserialize)]
pub struct Spoke {
    pub words: String,
    pub to: EntityId,
}

impl Event for Spoke {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("spoke");
    const OWNER: SystemId = Chatter::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// A fact only the person who said it may learn of.
#[derive(Debug, Serialize, Deserialize)]
pub struct Whispered {
    pub words: String,
}

impl Event for Whispered {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("whispered");
    const OWNER: SystemId = Chatter::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl System for Chatter {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([Placement::ID])
            .providing::<Speak>()
            .providing::<Whisper>()
            .emitting::<Spoke>()
            .emitting::<Whispered>()
    }

    /// One rule, so that a rejection is reachable from a client: speaking needs somebody to speak to.
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        if *intent.action_type() != Speak::ACTION_TYPE {
            return Ok(());
        }
        match intent.target() {
            None => Err(Rejection::PreconditionFailed),
            Some(target) if world.entity(target).is_none() => Err(Rejection::TargetUnavailable),
            Some(_) => Ok(()),
        }
    }

    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let _ = world;
        let unresolved = || KernelError::ActionNotResolvedBySystem {
            system: Self::ID,
            action_type: intent.action_type().clone(),
        };

        if *intent.action_type() == Speak::ACTION_TYPE {
            let bytes = intent
                .payload()
                .payload_for::<Speak>()
                .map_err(|_| unresolved())?;
            let request: Speak = serde_json::from_slice(bytes).map_err(|_| unresolved())?;
            let target = intent.target().ok_or_else(unresolved)?;
            return Ok(vec![
                Emission::new::<Spoke>(
                    payload(&Spoke {
                        words: request.words,
                        to: target,
                    }),
                    Visibility::Public,
                )
                .about(vec![intent.actor(), target])
                .with_participants(vec![intent.actor(), target]),
            ]);
        }

        let bytes = intent
            .payload()
            .payload_for::<Whisper>()
            .map_err(|_| unresolved())?;
        let request: Whisper = serde_json::from_slice(bytes).map_err(|_| unresolved())?;
        Ok(vec![
            Emission::new::<Whispered>(
                payload(&Whispered {
                    words: request.words,
                }),
                Visibility::Participants,
            )
            .about(vec![intent.actor()])
            .with_participants(vec![intent.actor()]),
        ])
    }
}

// ---------------------------------------------------------------------------------------------
// The perception: what one observer is shown, and nothing else.
// ---------------------------------------------------------------------------------------------

/// Perceives the room the observer is in: who is there, and the facts the observer is entitled to.
///
/// The judgement is per observer, which is what makes two clients' observations legitimately
/// different. Two people in the same world and different rooms perceive different sets of people —
/// so an observation that named all four would be a defect the tests can see.
pub struct RoomPerception;

impl Perception for RoomPerception {
    fn observe(&self, context: &PerceptionContext<'_>) -> WireObservation {
        let world = context.world();
        let Some(here) = world.component::<Room>(context.observer()) else {
            // An observer the placement system knows nothing about perceives nothing. Absence of
            // knowledge, not an error.
            return Observation::new(context.observer(), context.at());
        };

        let mut entities = Vec::new();
        let mut affordances = Vec::new();
        for (entity, room) in world.components::<Room>() {
            if room != here {
                continue;
            }
            let entity_type = world
                .entity(entity)
                .map_or(EntityType::Person, mineworld_contracts::Entity::entity_type);
            entities.push(
                PerceivedEntity::new(entity, entity_type).with_components(vec![
                    ComponentRecord::new::<Room>(entity, json!({ "name": room.name.clone() })),
                ]),
            );
            if entity != context.observer() {
                // The server states what may be attempted; a client renders it and evaluates
                // nothing (`DD-11`).
                affordances.push(Affordance::available(
                    Speak::ACTION_TYPE,
                    Some(entity),
                    SpatialRequirement::NONE,
                ));
            }
        }

        let events = context
            .recent_events()
            .iter()
            .filter(|envelope| entitled(context.observer(), envelope))
            .filter_map(perceived)
            .collect();

        Observation::new(context.observer(), context.at())
            .perceiving(entities)
            .with_events(events)
            .offering(affordances)
    }
}

/// Whether this observer could have learned of this fact, from the audience the emitting system
/// declared.
fn entitled(observer: EntityId, envelope: &EventEnvelope) -> bool {
    match envelope.visibility() {
        Visibility::Public => true,
        Visibility::Participants => envelope.participants().contains(&observer),
        Visibility::Entities(audience) => audience.contains(&observer),
        // This pack models no places, and a fact nobody may learn of is shown to nobody.
        Visibility::Place(_) | Visibility::SystemInternal => false,
    }
}

/// Re-encodes one of this pack's own facts for a JSON transport.
///
/// The transcoding is *here*, in the pack that declared the event types, because `EventRecord` can
/// only be built by naming an event's Rust type — which is the contract making sure a payload cannot
/// be relabelled. A fact this pack does not recognize is not shown, rather than guessed at.
fn perceived(envelope: &EventEnvelope) -> Option<PerceivedEvent<Value>> {
    let payload: Value = serde_json::from_slice(envelope.payload().payload()).ok()?;
    let record = if envelope.event_type() == &Spoke::EVENT_TYPE {
        EventRecord::new::<Spoke>(payload)
    } else if envelope.event_type() == &Whispered::EVENT_TYPE {
        EventRecord::new::<Whispered>(payload)
    } else {
        return None;
    };

    let rebuilt = EventEnvelope::new(
        envelope.id(),
        envelope.at(),
        record,
        envelope.caused_by().clone(),
        envelope.visibility().clone(),
        envelope.provenance().clone(),
    )
    .about(envelope.subjects().to_vec())
    .with_participants(envelope.participants().to_vec());
    Some(PerceivedEvent::new(match envelope.place() {
        Some(place) => rebuilt.at_place(place),
        None => rebuilt,
    }))
}

// ---------------------------------------------------------------------------------------------
// Reading an observation, the way a client would.
// ---------------------------------------------------------------------------------------------

/// The entities an observation lists.
pub fn perceived_ids(observation: &WireObservation) -> Vec<EntityId> {
    observation
        .entities()
        .iter()
        .map(PerceivedEntity::id)
        .collect()
}

/// Whether an observation carries a fact with this identity.
pub fn carries_event(observation: &WireObservation, event: mineworld_contracts::EventId) -> bool {
    observation
        .events()
        .iter()
        .any(|perceived| perceived.envelope().id() == event)
}

/// A `speak` request, as a client builds one: no identity, no instant.
pub fn speak_request(actor: EntityId, target: EntityId, words: &str) -> Value {
    json!({
        "actor": actor,
        "action_type": "speak",
        "target": target,
        "payload": { "action_type": "speak", "payload": { "words": words } },
        "actor_location": null,
    })
}

/// A `whisper` request: no target, and a fact only the speaker will be entitled to.
pub fn whisper_request(actor: EntityId, words: &str) -> Value {
    json!({
        "actor": actor,
        "action_type": "whisper",
        "payload": { "action_type": "whisper", "payload": { "words": words } },
        "target": null,
        "actor_location": null,
    })
}
