//! A café and a park, Alice and Bob in the café, Alice's day seeded through the section contract
//! exactly as the World Pack loader seeds it. Composed from the real packs.

#![allow(dead_code)]

use std::collections::BTreeMap;

use mineworld_authoring::{AuthoredSection, Seeding};
use mineworld_contracts::{
    EntityId, EntityKey, EntityType, EventEnvelope, LocalPosition, Location, Millimetres,
    Observation, PersonId, PlaceId, WorldTime,
};
use mineworld_kernel::{Emission, World};
use mineworld_movement::MovementSystem;
use mineworld_presence::{PerceptionProvider, PresenceSystem, arrival};
use mineworld_schedule::{AuthoredRoutine, ScheduleSystem};

/// Midnight: the genesis of every world here, so the wrap past midnight is exercised at once.
pub const GENESIS: WorldTime = WorldTime::from_seconds(0);

/// Alice's day: the café from six, the park from six in the evening (which wraps past midnight).
pub const ALICE_DAY: &str = "
- { from: \"06:00\", place: cafe, label: work }
- { from: \"18:00\", place: park, label: walk }
";

pub fn key(name: &str) -> EntityKey {
    EntityKey::new(name).expect("a key")
}

pub fn t(seconds: i64) -> WorldTime {
    WorldTime::from_seconds(seconds)
}

pub fn routine(yaml: &str) -> AuthoredRoutine {
    serde_saphyr::from_str(yaml).expect("a valid routine")
}

/// An empty world with presence, movement and schedule installed.
pub fn compose() -> (World, Vec<Box<dyn PerceptionProvider>>) {
    let mut world = World::new();
    world.install(PresenceSystem).expect("presence");
    world.install(MovementSystem).expect("movement");
    world
        .install(ScheduleSystem)
        .expect("schedule installs with no dependency");
    let providers: Vec<Box<dyn PerceptionProvider>> = vec![
        Box::new(PresenceSystem),
        Box::new(MovementSystem),
        Box::new(ScheduleSystem),
    ];
    (world, providers)
}

pub struct Town {
    pub world: World,
    pub providers: Vec<Box<dyn PerceptionProvider>>,
    pub keys: BTreeMap<EntityKey, EntityId>,
    pub alice: EntityId,
    pub bob: EntityId,
    pub cafe: PlaceId,
    pub park: PlaceId,
}

impl Town {
    /// The town, entities created, and the genesis facts — arrivals, then Alice's routine — not yet
    /// stated.
    pub fn assemble(
        mut world: World,
        providers: Vec<Box<dyn PerceptionProvider>>,
    ) -> (Self, Vec<Emission>) {
        let mut keys = BTreeMap::new();
        for (name, entity_type) in [
            ("cafe", EntityType::Place),
            ("park", EntityType::Place),
            ("alice", EntityType::Person),
            ("bob", EntityType::Person),
        ] {
            let id = world
                .create_entity(key(name), entity_type)
                .expect("created");
            keys.insert(key(name), id);
        }
        let cafe = PlaceId::new(keys[&key("cafe")], EntityType::Place).expect("a place");
        let park = PlaceId::new(keys[&key("park")], EntityType::Place).expect("a place");
        let (alice, bob) = (keys[&key("alice")], keys[&key("bob")]);
        let mut facts = Vec::new();
        for (person, x) in [(alice, 1_000), (bob, 2_000)] {
            let person = PersonId::new(person, EntityType::Person).expect("a person");
            let at = Location::in_place(cafe).with_local(LocalPosition::on_ground(
                Millimetres::new(x),
                Millimetres::new(1_000),
            ));
            facts.push(arrival(&world.read(), person, at).expect("admitted"));
        }
        {
            let read = world.read();
            let seeding = Seeding::new(&read, &keys);
            facts.extend(
                ScheduleSystem::seed(&seeding, alice, &routine(ALICE_DAY)).expect("seeded"),
            );
        }
        (
            Self {
                world,
                providers,
                keys,
                alice,
                bob,
                cafe,
                park,
            },
            facts,
        )
    }

    /// The town, begun at [`GENESIS`]; and the genesis facts.
    pub fn begun() -> (Self, Vec<EventEnvelope>) {
        let (world, providers) = compose();
        let (mut town, facts) = Self::assemble(world, providers);
        let genesis = town.world.genesis(GENESIS, facts).expect("begins");
        (town, genesis)
    }

    pub fn observation(&self, observer: EntityId, at: WorldTime) -> Observation<serde_json::Value> {
        let providers: Vec<&dyn PerceptionProvider> =
            self.providers.iter().map(AsRef::as_ref).collect();
        mineworld_presence::observe(&self.world, observer, at, &providers)
    }
}

/// The event types of `facts`, in order.
pub fn types(facts: &[EventEnvelope]) -> Vec<&str> {
    facts
        .iter()
        .map(|fact| fact.event_type().as_str())
        .collect()
}
