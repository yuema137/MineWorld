//! A café with Alice and Bob a metre apart, composed from the real packs and driven by real actions.
//!
//! Relationships is never driven directly — it provides no action. Everything it does here it does
//! because conversation or group-activity stated a fact.

#![allow(dead_code)]

use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionResult, EntityId, EntityKey, EntityType,
    EventEnvelope, LocalPosition, Location, Millimetres, Observation, PersonId, PlaceId, SystemId,
    WorldTime,
};
use mineworld_conversation::{ConversationSystem, Talk, Utterance};
use mineworld_group_activity::{
    AcceptInvitation, ActivityKind, DeclineInvitation, GroupActivitySystem, Invite,
    LeaveGroupActivity,
};
use mineworld_kernel::{SystemIdentity, World};
use mineworld_movement::MovementSystem;
use mineworld_presence::{PerceptionProvider, PresenceSystem, arrival};
use mineworld_relationships::{Acquaintances, RelationshipValues, RelationshipsSystem, knows};
use serde::Serialize;

pub const GENESIS: WorldTime = WorldTime::from_seconds(3_600);

/// Which of the social packs a test world installs besides presence and relationships.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Packs {
    pub conversation: bool,
    pub group_activity: bool,
}

pub const ALL: Packs = Packs {
    conversation: true,
    group_activity: true,
};

pub struct Cafe {
    pub world: World,
    pub providers: Vec<Box<dyn PerceptionProvider>>,
    pub cafe: PlaceId,
    pub alice: EntityId,
    pub bob: EntityId,
    clock: i64,
    next_action: u64,
}

fn record<A: Action + Serialize>(action: &A) -> ActionRecord {
    ActionRecord::new::<A>(serde_json::to_vec(action).expect("encodes"))
}

impl Cafe {
    pub fn new(packs: Packs) -> Self {
        let mut world = World::new();
        let mut providers: Vec<Box<dyn PerceptionProvider>> = Vec::new();
        world.install(PresenceSystem).expect("presence");
        providers.push(Box::new(PresenceSystem));
        world.install(MovementSystem).expect("movement");
        providers.push(Box::new(MovementSystem));
        if packs.conversation {
            world.install(ConversationSystem).expect("conversation");
            providers.push(Box::new(ConversationSystem));
        }
        if packs.group_activity {
            world.install(GroupActivitySystem).expect("group-activity");
            providers.push(Box::new(GroupActivitySystem));
        }
        world
            .install(RelationshipsSystem)
            .expect("relationships installs with no system dependency");
        providers.push(Box::new(RelationshipsSystem));

        let cafe = world
            .create_entity(EntityKey::new("cafe").expect("key"), EntityType::Place)
            .expect("created");
        let cafe = PlaceId::new(cafe, EntityType::Place).expect("a place");
        let alice = world
            .create_entity(EntityKey::new("alice").expect("key"), EntityType::Person)
            .expect("created");
        let bob = world
            .create_entity(EntityKey::new("bob").expect("key"), EntityType::Person)
            .expect("created");
        let mut facts = Vec::new();
        for (person, x) in [(alice, 1_000), (bob, 2_000)] {
            let person = PersonId::new(person, EntityType::Person).expect("a person");
            let spot = Location::in_place(cafe).with_local(LocalPosition::on_ground(
                Millimetres::new(x),
                Millimetres::new(1_000),
            ));
            facts.push(arrival(&world.read(), person, spot).expect("admitted"));
        }
        world.genesis(GENESIS, facts).expect("begins");
        Self {
            world,
            providers,
            cafe,
            alice,
            bob,
            clock: GENESIS.seconds(),
            next_action: 1,
        }
    }

    /// The next instant: every request is a second after the last, so no request outruns a wake.
    fn tick(&mut self) -> WorldTime {
        self.clock += 1;
        WorldTime::from_seconds(self.clock)
    }

    pub fn submit(
        &mut self,
        actor: EntityId,
        target: Option<EntityId>,
        record: ActionRecord,
    ) -> (ActionResult, Vec<EventEnvelope>) {
        let when = self.tick();
        let _due = self.world.advance_to(when).expect("advances");
        let id = ActionId::from_raw(self.next_action);
        self.next_action += 1;
        let mut intent = ActionIntent::new(id, actor, record, when);
        if let Some(target) = target {
            intent = intent.with_target(target);
        }
        let dispatched = self.world.dispatch(&intent, when).expect("answers");
        (dispatched.result().clone(), dispatched.events().to_vec())
    }

    pub fn talk(&mut self, from: EntityId, to: EntityId) -> (ActionResult, Vec<EventEnvelope>) {
        let said = Utterance::new("Hello.").expect("an utterance");
        self.submit(from, Some(to), record(&Talk::new(said)))
    }

    pub fn invite(&mut self, from: EntityId, to: EntityId) -> (ActionResult, Vec<EventEnvelope>) {
        let kind = ActivityKind::new("coffee").expect("a kind");
        self.submit(from, Some(to), record(&Invite::new(kind)))
    }

    pub fn accept(
        &mut self,
        by: EntityId,
        inviter: EntityId,
    ) -> (ActionResult, Vec<EventEnvelope>) {
        self.submit(by, Some(inviter), record(&AcceptInvitation {}))
    }

    pub fn decline(
        &mut self,
        by: EntityId,
        inviter: EntityId,
    ) -> (ActionResult, Vec<EventEnvelope>) {
        self.submit(by, Some(inviter), record(&DeclineInvitation {}))
    }

    pub fn leave(&mut self, by: EntityId) -> (ActionResult, Vec<EventEnvelope>) {
        self.submit(by, None, record(&LeaveGroupActivity {}))
    }

    pub fn person(&self, entity: EntityId) -> PersonId {
        PersonId::new(entity, EntityType::Person).expect("a person")
    }

    /// What `holder` holds about `counterpart`.
    pub fn values(&self, holder: EntityId, counterpart: EntityId) -> Option<RelationshipValues> {
        self.world
            .read()
            .component::<Acquaintances>(holder)
            .and_then(|known| known.of(self.person(counterpart)).cloned())
    }

    /// Every `knows` edge, as (from, to).
    pub fn edges(&self) -> Vec<(EntityId, EntityId)> {
        self.world
            .read()
            .relations_of_type(&knows())
            .map(|edge| (edge.from(), edge.to()))
            .collect()
    }

    /// Every Acquaintances entry, as (holder, counterpart).
    pub fn entries(&self) -> Vec<(EntityId, EntityId)> {
        let mut entries: Vec<(EntityId, EntityId)> = self
            .world
            .read()
            .components::<Acquaintances>()
            .flat_map(|(holder, known)| {
                known
                    .known()
                    .iter()
                    .map(move |entry| (holder, entry.counterpart().entity_id()))
                    .collect::<Vec<_>>()
            })
            .collect();
        entries.sort();
        entries
    }

    pub fn observation(&self, observer: EntityId) -> Observation<serde_json::Value> {
        let providers: Vec<&dyn PerceptionProvider> =
            self.providers.iter().map(AsRef::as_ref).collect();
        mineworld_presence::observe(
            &self.world,
            observer,
            WorldTime::from_seconds(self.clock),
            &providers,
        )
    }

    pub fn disable(&mut self, system: &SystemId) {
        self.world.disable(system).expect("nothing depends on it");
    }

    pub fn enable(&mut self, system: &SystemId) {
        self.world.enable(system).expect("re-enabled");
    }
}

pub fn conversation_id() -> SystemId {
    ConversationSystem::ID
}

/// The facts of one type among `facts`.
pub fn of_type<'a>(facts: &'a [EventEnvelope], name: &str) -> Vec<&'a EventEnvelope> {
    facts
        .iter()
        .filter(|fact| fact.event_type().as_str() == name)
        .collect()
}
