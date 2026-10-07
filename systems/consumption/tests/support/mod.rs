//! A café with Alice, Bob and Carol, composed from the real packs — presence, movement, item, inventory
//! and, unless a test leaves it out, consumption. Kinds and holdings are seeded through the section
//! contract exactly as the World Pack loader seeds them: items' sections first, then people's
//! (`ARC-36` item 7).
//!
//! ```text
//! coffee, tea   drink        croissant   food        mug   goods
//! alice         coffee 1, croissant 1, mug 1
//! bob           tea 1
//! carol         nothing
//! ```

#![allow(dead_code)]

use std::collections::BTreeMap;

use mineworld_authoring::{AuthoredSection, Seeding};
use mineworld_consumption::{ConsumptionSystem, Drink, Eat};
use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, Affordance, EntityId, EntityKey, EntityType,
    EventEnvelope, ItemId, LocalPosition, Location, Millimetres, Observation, PersonId, PlaceId,
    WorldTime,
};
use mineworld_inventory::{AuthoredHoldings, Holdings, InventorySystem};
use mineworld_item::{AuthoredItem, ItemSystem};
use mineworld_kernel::{Dispatched, Emission, KernelError, World};
use mineworld_movement::MovementSystem;
use mineworld_presence::{PerceptionProvider, PresenceSystem, arrival};
use serde_json::Value;

pub const GENESIS: WorldTime = WorldTime::from_seconds(0);

pub const KINDS: [(&str, &str); 4] = [
    ("coffee", "drink"),
    ("croissant", "food"),
    ("mug", "goods"),
    ("tea", "drink"),
];

pub const HOLDINGS: [(&str, &str); 2] = [
    ("alice", "{ coffee: 1, croissant: 1, mug: 1 }"),
    ("bob", "{ tea: 1 }"),
];

pub fn key(name: &str) -> EntityKey {
    EntityKey::new(name).expect("a key")
}

pub fn t(seconds: i64) -> WorldTime {
    WorldTime::from_seconds(seconds)
}

/// Every provider a run consults, consumption included whether or not the world installed it.
pub fn providers() -> [&'static dyn PerceptionProvider; 5] {
    [
        &PresenceSystem,
        &MovementSystem,
        &ItemSystem,
        &InventorySystem,
        &ConsumptionSystem,
    ]
}

/// What a meal request is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Meal {
    Eat,
    Drink,
}

pub struct Town {
    pub world: World,
    pub keys: BTreeMap<EntityKey, EntityId>,
    pub genesis: Vec<EventEnvelope>,
}

impl Town {
    /// The café, begun at [`GENESIS`] with these holdings, consumption installed or not.
    pub fn begun(with_consumption: bool, holdings: &[(&str, &str)]) -> Self {
        let mut world = World::new();
        world.install(PresenceSystem).expect("presence");
        world.install(MovementSystem).expect("movement");
        world.install(ItemSystem).expect("item");
        world.install(InventorySystem).expect("inventory");
        if with_consumption {
            world
                .install(ConsumptionSystem)
                .expect("consumption installs after inventory");
        }
        let mut keys = BTreeMap::new();
        for (name, entity_type) in [
            ("cafe", EntityType::Place),
            ("alice", EntityType::Person),
            ("bob", EntityType::Person),
            ("carol", EntityType::Person),
            ("coffee", EntityType::Item),
            ("croissant", EntityType::Item),
            ("mug", EntityType::Item),
            ("tea", EntityType::Item),
        ] {
            let id = world
                .create_entity(key(name), entity_type)
                .expect("created");
            keys.insert(key(name), id);
        }
        let cafe = PlaceId::new(keys[&key("cafe")], EntityType::Place).expect("a place");
        let mut facts: Vec<Emission> = Vec::new();
        for (person, x) in [("alice", 1_000), ("bob", 2_000), ("carol", 3_000)] {
            let person = PersonId::new(keys[&key(person)], EntityType::Person).expect("a person");
            let at = Location::in_place(cafe).with_local(LocalPosition::on_ground(
                Millimetres::new(x),
                Millimetres::new(1_000),
            ));
            facts.push(arrival(&world.read(), person, at).expect("admitted"));
        }
        {
            let read = world.read();
            let seeding = Seeding::new(&read, &keys);
            for (item, category) in KINDS {
                let authored: AuthoredItem =
                    serde_saphyr::from_str(&format!("category: {category}")).expect("a section");
                facts.extend(
                    ItemSystem::seed(&seeding, keys[&key(item)], &authored).expect("seeded"),
                );
            }
            for (holder, yaml) in holdings {
                let authored: AuthoredHoldings = serde_saphyr::from_str(yaml).expect("a section");
                facts.extend(
                    InventorySystem::seed(&seeding, keys[&key(holder)], &authored).expect("seeded"),
                );
            }
        }
        let genesis = world.genesis(GENESIS, facts).expect("begins");
        Self {
            world,
            keys,
            genesis,
        }
    }

    pub fn id(&self, name: &str) -> EntityId {
        self.keys[&key(name)]
    }

    pub fn item(&self, name: &str) -> ItemId {
        ItemId::new(self.id(name), EntityType::Item).expect("an item")
    }

    pub fn observation(&self, observer: &str, at: WorldTime) -> Observation<Value> {
        mineworld_presence::observe(&self.world, self.id(observer), at, &providers())
    }

    /// A holder's holdings as (item key, count), in item order.
    pub fn holdings(&self, holder: &str) -> Vec<(String, u32)> {
        let names: BTreeMap<EntityId, &EntityKey> =
            self.keys.iter().map(|(k, id)| (*id, k)).collect();
        self.world
            .read()
            .component::<Holdings>(self.id(holder))
            .map(|held| {
                held.held()
                    .iter()
                    .map(|line| {
                        (
                            names[&line.item().entity_id()].as_str().to_owned(),
                            line.count(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Dispatches `who` eating or drinking one `item`, perhaps at `target`, as request `id` at second
    /// `id`.
    pub fn meal(
        &mut self,
        who: &str,
        meal: Meal,
        item: &str,
        target: Option<&str>,
        id: u64,
    ) -> Result<Dispatched, KernelError> {
        let at = t(i64::try_from(id).expect("small"));
        let item = self.item(item);
        let record = match meal {
            Meal::Eat => {
                ActionRecord::new::<Eat>(serde_json::to_vec(&Eat::new(item)).expect("encodes"))
            }
            Meal::Drink => {
                ActionRecord::new::<Drink>(serde_json::to_vec(&Drink::new(item)).expect("encodes"))
            }
        };
        let mut intent = ActionIntent::new(ActionId::from_raw(id), self.id(who), record, at);
        if let Some(target) = target {
            intent = intent.with_target(self.id(target));
        }
        self.world.dispatch(&intent, at)
    }

    /// Every component and relation, as bytes: "writes nothing" is compared on these.
    pub fn state(&self) -> Vec<u8> {
        let snapshot = self.world.snapshot().expect("a snapshot");
        serde_json::to_vec(&(&snapshot.components, &snapshot.relations)).expect("encodes")
    }
}

/// The `eat` and `drink` affordances of an observation, in observation order.
pub fn meals(observation: &Observation<Value>) -> Vec<&Affordance<Value>> {
    observation
        .affordances()
        .iter()
        .filter(|affordance| matches!(affordance.action_type().as_str(), "eat" | "drink"))
        .collect()
}
