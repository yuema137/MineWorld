//! A small town for movement's tests: a café, the street outside it joined by one doorway, and an
//! attic that opens onto nothing — composed from the real packs, begun by genesis, and driven the way
//! a server drives a world.
//!
//! ```text
//! cafe     the doorway at (4600, 2000)  ─┐
//! street   the doorway at (0, 2000)     ─┘  one passage, both sides known
//! attic    no passage: reaching it from anywhere is travel, not a move
//! ```
//!
//! Every number is a literal from this layout, never derived from `MAX_STRIDE` (`ARC-23` rule 2).

#![allow(dead_code)]

use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, ActionResult, EntityId, EntityKey, EntityType,
    EventEnvelope, LocalPosition, Location, Millimetres, Observation, PersonId, PlaceId, WorldTime,
};
use mineworld_conversation::ConversationSystem;
use mineworld_kernel::{Emission, SystemIdentity, World};
use mineworld_movement::{Move, MovementSystem, passage};
use mineworld_presence::{PerceptionProvider, PresenceSystem, arrival, present_in};
use serde::Serialize;

/// The instant every request in these tests is made at (`AC-12`: supplied, never read from a clock).
pub const NOW: WorldTime = WorldTime::from_seconds(3_600);

/// The café side of the doorway.
pub const CAFE_DOOR: (i32, i32) = (4_600, 2_000);
/// The street side of the same doorway.
pub const STREET_DOOR: (i32, i32) = (0, 2_000);

/// Whether this world has walking in it, and how it does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Movement {
    /// Installed and enabled.
    Enabled,
    /// Installed, then disabled by configuration.
    Disabled,
    /// Never installed.
    Absent,
}

/// Whether the places are joined, and whether the world models positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// One café, nothing joined: what a world without movement can also begin with.
    CafeOnly,
    /// Café and street joined at known doorways, plus the attic.
    Joined,
    /// The same three places and one passage, with no position anywhere.
    Semantic,
}

pub struct Town {
    pub world: World,
    pub providers: Vec<Box<dyn PerceptionProvider>>,
    pub cafe: PlaceId,
    pub street: PlaceId,
    pub attic: PlaceId,
    pub visitor: EntityId,
    pub alice: EntityId,
    pub stranger: EntityId,
    pub genesis: Vec<EventEnvelope>,
    next_action: u64,
}

pub fn at(place: PlaceId, x: i32, y: i32) -> Location {
    Location::in_place(place).with_local(LocalPosition::on_ground(
        Millimetres::new(x),
        Millimetres::new(y),
    ))
}

fn local((x, y): (i32, i32)) -> LocalPosition {
    LocalPosition::on_ground(Millimetres::new(x), Millimetres::new(y))
}

pub fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("a payload encodes")
}

/// The packs, composed as `movement` says, in the order a World Pack lists them.
pub fn compose(movement: Movement) -> (World, Vec<Box<dyn PerceptionProvider>>) {
    let mut world = World::new();
    let mut providers: Vec<Box<dyn PerceptionProvider>> = vec![Box::new(PresenceSystem)];
    world.install(PresenceSystem).expect("presence installs");
    if movement != Movement::Absent {
        world
            .install(MovementSystem)
            .expect("movement installs after presence");
        providers.push(Box::new(MovementSystem));
    }
    world
        .install(ConversationSystem)
        .expect("conversation installs");
    providers.push(Box::new(ConversationSystem));
    if movement == Movement::Disabled {
        world
            .disable(&MovementSystem::ID)
            .expect("nothing depends on movement");
    }
    (world, providers)
}

impl Town {
    /// The town assembled — composed, its entities created — and its genesis facts, not yet stated:
    /// the visitor at `visitor_at` and Alice at (1200, 1000) in the café.
    ///
    /// `visitor_at` is a function of the places, because their identities are allocated here.
    pub fn assemble(
        movement: Movement,
        layout: Layout,
        visitor_at: impl FnOnce(&Town) -> Location,
    ) -> (Self, Vec<Emission>) {
        let (mut world, providers) = compose(movement);
        let mut create = |key: &str, entity_type| {
            world
                .create_entity(EntityKey::new(key).expect("a key"), entity_type)
                .expect("created")
        };
        let place = |entity| PlaceId::new(entity, EntityType::Place).expect("a place");
        let cafe = place(create("cafe", EntityType::Place));
        let street = place(create("street", EntityType::Place));
        let attic = place(create("attic", EntityType::Place));
        let visitor = create("visitor", EntityType::Person);
        let alice = create("alice", EntityType::Person);
        let stranger = create("stranger", EntityType::Person);
        let town = Self {
            world,
            providers,
            cafe,
            street,
            attic,
            visitor,
            alice,
            stranger,
            genesis: Vec::new(),
            next_action: 1,
        };

        let mut facts: Vec<Emission> = Vec::new();
        let semantic = layout == Layout::Semantic;
        match layout {
            Layout::CafeOnly => {}
            Layout::Joined => facts.push(passage(
                cafe,
                Some(local(CAFE_DOOR)),
                street,
                Some(local(STREET_DOOR)),
            )),
            Layout::Semantic => facts.push(passage(cafe, None, street, None)),
        }
        let visitor_location = visitor_at(&town);
        let alice_location = if semantic {
            Location::in_place(cafe)
        } else {
            at(cafe, 1_200, 1_000)
        };
        for (person, location) in [(visitor, visitor_location), (alice, alice_location)] {
            let person = PersonId::new(person, EntityType::Person).expect("a person");
            facts.push(arrival(&town.world.read(), person, location).expect("presence admits it"));
        }
        (town, facts)
    }

    /// The town, begun with the visitor at `visitor_at` and Alice at (1200, 1000) in the café.
    pub fn new(
        movement: Movement,
        layout: Layout,
        visitor_at: impl FnOnce(&Town) -> Location,
    ) -> Self {
        let (mut town, facts) = Self::assemble(movement, layout, visitor_at);
        town.genesis = town.world.genesis(NOW, facts).expect("the town begins");
        town
    }

    /// The joined, positioned town with movement enabled.
    pub fn joined(visitor_at: impl FnOnce(&Town) -> Location) -> Self {
        Self::new(Movement::Enabled, Layout::Joined, visitor_at)
    }

    /// The next action identity, as a server allocates it.
    pub fn next_id(&mut self) -> ActionId {
        let id = ActionId::from_raw(self.next_action);
        self.next_action += 1;
        id
    }

    /// Submits any record as `actor`, with no target.
    pub fn submit(
        &mut self,
        actor: EntityId,
        record: ActionRecord,
    ) -> (ActionResult, Vec<EventEnvelope>) {
        let id = self.next_id();
        let intent = ActionIntent::new(id, actor, record, NOW);
        let dispatched = self
            .world
            .dispatch(&intent, NOW)
            .expect("dispatch answers rather than failing");
        (dispatched.result().clone(), dispatched.events().to_vec())
    }

    /// Asks for `person` to move to `to`.
    pub fn r#move(&mut self, person: EntityId, to: Location) -> (ActionResult, Vec<EventEnvelope>) {
        self.submit(person, ActionRecord::new::<Move>(encode(&Move::new(to))))
    }

    /// Where presence says `person` is.
    pub fn presence(&self, person: EntityId) -> Option<Location> {
        self.world
            .components()
            .get::<mineworld_presence::Presence>(person)
            .map(mineworld_presence::Presence::location)
    }

    /// Every place `person` has a `present-in` edge to.
    pub fn present_in(&self, person: EntityId) -> Vec<EntityId> {
        self.world
            .relations()
            .touching(person)
            .filter(|edge| *edge.relation_type() == present_in() && edge.from() == person)
            .map(|edge| edge.to())
            .collect()
    }

    /// What this world tells `observer`.
    pub fn observation(&self, observer: EntityId) -> Observation<serde_json::Value> {
        let providers: Vec<&dyn PerceptionProvider> =
            self.providers.iter().map(AsRef::as_ref).collect();
        mineworld_presence::observe(&self.world, observer, NOW, &providers)
    }

    /// The whole world's state, as bytes — every field of the snapshot except `ran`.
    ///
    /// `ran` records that the world has dispatched *anything*, whatever the answer, so it flips on
    /// the first request even when that request is refused; it says the world can no longer take
    /// genesis, not that a refusal changed state. Everything else — composition, entities,
    /// components, edges, clock and counters — is compared.
    pub fn snapshot_bytes(&self) -> Vec<u8> {
        let snapshot = self.world.snapshot().expect("a snapshot");
        encode(&(
            &snapshot.composition,
            &snapshot.entities,
            &snapshot.components,
            &snapshot.relations,
            &snapshot.time,
            snapshot.clock_started,
        ))
    }
}
