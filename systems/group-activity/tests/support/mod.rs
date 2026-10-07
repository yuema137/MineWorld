//! A small café for group-activity's tests, composed from the real packs and begun by genesis.
//!
//! ```text
//! cafe     alice (1 000, 1 000)  bob (2 000, 1 000)  carol (3 000, 1 000)
//!          far   (4 001, 1 000) — 3 001 mm from alice, one millimetre out of invite reach
//!          the doorway at (4 600, 2 000) onto the street's (0, 2 000)
//! street   nobody
//! ```
//!
//! Every number is a literal from this layout, never derived from a constant under test (`ARC-23`
//! rule 2).

#![allow(dead_code)]

use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionResult, EntityId, EntityKey, EntityType,
    EventEnvelope, LocalPosition, Location, Millimetres, Observation, PersonId, PlaceId, WorldTime,
};
use mineworld_group_activity::{
    AcceptInvitation, ActivityKind, DeclineInvitation, GroupActivitySystem, Invitations, Invite,
    JoinGroupActivity, LeaveGroupActivity, Participation,
};
use mineworld_kernel::{Emission, World};
use mineworld_movement::{Move, MovementSystem, passage};
use mineworld_presence::{PerceptionProvider, PresenceSystem, arrival};
use serde::Serialize;

/// The instant genesis happens at; every request is an offset from it.
pub const GENESIS: WorldTime = WorldTime::from_seconds(3_600);

pub const CAFE_DOOR: (i32, i32) = (4_600, 2_000);
pub const STREET_DOOR: (i32, i32) = (0, 2_000);

pub fn t(offset: i64) -> WorldTime {
    WorldTime::from_seconds(GENESIS.seconds() + offset)
}

pub fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("a payload encodes")
}

pub fn record<A: Action + Serialize>(action: &A) -> ActionRecord {
    ActionRecord::new::<A>(encode(action))
}

pub fn kind(slug: &str) -> ActivityKind {
    ActivityKind::new(slug).expect("a valid kind")
}

pub fn at(place: PlaceId, (x, y): (i32, i32)) -> Location {
    Location::in_place(place).with_local(LocalPosition::on_ground(
        Millimetres::new(x),
        Millimetres::new(y),
    ))
}

pub struct Cafe {
    pub world: World,
    pub providers: Vec<Box<dyn PerceptionProvider>>,
    pub cafe: PlaceId,
    pub street: PlaceId,
    pub alice: EntityId,
    pub bob: EntityId,
    pub carol: EntityId,
    pub far: EntityId,
    next_action: u64,
}

/// The packs a group-activity world needs, in the order a World Pack would list them.
pub fn compose() -> (World, Vec<Box<dyn PerceptionProvider>>) {
    let mut world = World::new();
    world.install(PresenceSystem).expect("presence installs");
    world.install(MovementSystem).expect("movement installs");
    world
        .install(GroupActivitySystem)
        .expect("group-activity installs, presence being present");
    let providers: Vec<Box<dyn PerceptionProvider>> = vec![
        Box::new(PresenceSystem),
        Box::new(MovementSystem),
        Box::new(GroupActivitySystem),
    ];
    (world, providers)
}

impl Cafe {
    /// The café assembled, and its genesis facts — not yet stated.
    pub fn assemble(
        world: World,
        providers: Vec<Box<dyn PerceptionProvider>>,
    ) -> (Self, Vec<Emission>) {
        let mut world = world;
        let mut create = |key: &str, entity_type| {
            world
                .create_entity(EntityKey::new(key).expect("a key"), entity_type)
                .expect("created")
        };
        let place = |entity| PlaceId::new(entity, EntityType::Place).expect("a place");
        let cafe = place(create("cafe", EntityType::Place));
        let street = place(create("street", EntityType::Place));
        let alice = create("alice", EntityType::Person);
        let bob = create("bob", EntityType::Person);
        let carol = create("carol", EntityType::Person);
        let far = create("far", EntityType::Person);
        let local =
            |(x, y): (i32, i32)| LocalPosition::on_ground(Millimetres::new(x), Millimetres::new(y));
        let mut facts = vec![passage(
            cafe,
            Some(local(CAFE_DOOR)),
            street,
            Some(local(STREET_DOOR)),
        )];
        for (person, spot) in [
            (alice, (1_000, 1_000)),
            (bob, (2_000, 1_000)),
            (carol, (3_000, 1_000)),
            (far, (4_001, 1_000)),
        ] {
            let person = PersonId::new(person, EntityType::Person).expect("a person");
            facts.push(arrival(&world.read(), person, at(cafe, spot)).expect("presence admits it"));
        }
        (
            Self {
                world,
                providers,
                cafe,
                street,
                alice,
                bob,
                carol,
                far,
                next_action: 1,
            },
            facts,
        )
    }

    /// The café, begun.
    pub fn new() -> Self {
        let (world, providers) = compose();
        let (mut cafe, facts) = Self::assemble(world, providers);
        cafe.world.genesis(GENESIS, facts).expect("the café begins");
        cafe
    }

    pub fn next_id(&mut self) -> ActionId {
        let id = ActionId::from_raw(self.next_action);
        self.next_action += 1;
        id
    }

    /// Advances to `when`, then submits `record` as `actor` against `target`, as a server would.
    pub fn submit(
        &mut self,
        when: WorldTime,
        actor: EntityId,
        target: Option<EntityId>,
        record: ActionRecord,
    ) -> (ActionResult, Vec<EventEnvelope>) {
        let _due = self.world.advance_to(when).expect("the clock advances");
        let id = self.next_id();
        let mut intent = ActionIntent::new(id, actor, record, when);
        if let Some(target) = target {
            intent = intent.with_target(target);
        }
        let dispatched = self
            .world
            .dispatch(&intent, when)
            .expect("dispatch answers rather than failing");
        (dispatched.result().clone(), dispatched.events().to_vec())
    }

    pub fn invite(
        &mut self,
        when: i64,
        from: EntityId,
        to: EntityId,
    ) -> (ActionResult, Vec<EventEnvelope>) {
        self.submit(
            t(when),
            from,
            Some(to),
            record(&Invite::new(kind("coffee"))),
        )
    }

    pub fn accept(
        &mut self,
        when: i64,
        by: EntityId,
        inviter: EntityId,
    ) -> (ActionResult, Vec<EventEnvelope>) {
        self.submit(t(when), by, Some(inviter), record(&AcceptInvitation {}))
    }

    pub fn decline(
        &mut self,
        when: i64,
        by: EntityId,
        inviter: EntityId,
    ) -> (ActionResult, Vec<EventEnvelope>) {
        self.submit(t(when), by, Some(inviter), record(&DeclineInvitation {}))
    }

    pub fn join(
        &mut self,
        when: i64,
        by: EntityId,
        member: EntityId,
    ) -> (ActionResult, Vec<EventEnvelope>) {
        self.submit(t(when), by, Some(member), record(&JoinGroupActivity {}))
    }

    pub fn leave(&mut self, when: i64, by: EntityId) -> (ActionResult, Vec<EventEnvelope>) {
        self.submit(t(when), by, None, record(&LeaveGroupActivity {}))
    }

    pub fn walk(
        &mut self,
        when: i64,
        who: EntityId,
        to: Location,
    ) -> (ActionResult, Vec<EventEnvelope>) {
        self.submit(t(when), who, None, record(&Move::new(to)))
    }

    pub fn participation(&self, person: EntityId) -> Option<Participation> {
        self.world
            .read()
            .component::<Participation>(person)
            .cloned()
    }

    pub fn invitations(&self, person: EntityId) -> Option<Invitations> {
        self.world.read().component::<Invitations>(person).cloned()
    }

    pub fn observation(&self, observer: EntityId, when: i64) -> Observation<serde_json::Value> {
        let providers: Vec<&dyn PerceptionProvider> =
            self.providers.iter().map(AsRef::as_ref).collect();
        mineworld_presence::observe(&self.world, observer, t(when), &providers)
    }
}

/// The event types of a list of facts, in order.
pub fn types(facts: &[EventEnvelope]) -> Vec<String> {
    facts
        .iter()
        .map(|fact| fact.event_type().as_str().to_owned())
        .collect()
}
