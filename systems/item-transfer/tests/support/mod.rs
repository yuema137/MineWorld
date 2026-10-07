//! A café with Alice and Bob within reach of each other and Carol across the room, composed from the
//! real packs — presence, movement, item, inventory and, unless a test leaves it out, item-transfer.
//! Kinds and holdings are seeded through the section contract exactly as the World Pack loader seeds
//! them: items' sections first, then people's (`ARC-36` item 7).

#![allow(dead_code)]

use std::collections::BTreeMap;

use mineworld_authoring::{AuthoredSection, Seeding};
use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, Affordance, EntityId, EntityKey, EntityType,
    EventEnvelope, ItemId, LocalPosition, Location, Millimetres, Observation, PersonId, PlaceId,
    WorldTime,
};
use mineworld_inventory::{AuthoredHoldings, Holdings, InventorySystem};
use mineworld_item::{AuthoredItem, ItemSystem};
use mineworld_item_transfer::{Give, ItemTransferSystem};
use mineworld_kernel::{Dispatched, Emission, KernelError, World};
use mineworld_movement::MovementSystem;
use mineworld_presence::{PerceptionProvider, PresenceSystem, arrival};
use serde_json::Value;

pub const GENESIS: WorldTime = WorldTime::from_seconds(0);

/// Kinds, each declared by an `item:` section.
pub const KINDS: [(&str, &str); 3] = [("apple", "food"), ("coffee", "drink"), ("tea", "drink")];

/// Alice holds two kinds, Bob one, Carol nothing.
pub const HOLDINGS: [(&str, &str); 2] =
    [("alice", "{ coffee: 1, tea: 1 }"), ("bob", "{ apple: 1 }")];

/// Where each person stands in the café, in millimetres: Alice and Bob 1 m apart, Carol 8 m from
/// Alice — beyond a give's 3 m.
pub const PLACED: [(&str, i32); 3] = [("alice", 1_000), ("bob", 2_000), ("carol", 9_000)];

pub fn key(name: &str) -> EntityKey {
    EntityKey::new(name).expect("a key")
}

pub fn t(seconds: i64) -> WorldTime {
    WorldTime::from_seconds(seconds)
}

/// Every provider a run consults, item-transfer included whether or not the world installed it: a
/// provider of a pack the world does not have is dropped by perception's route map (`INV-10`).
pub fn providers() -> [&'static dyn PerceptionProvider; 5] {
    [
        &PresenceSystem,
        &MovementSystem,
        &ItemSystem,
        &InventorySystem,
        &ItemTransferSystem,
    ]
}

pub struct Town {
    pub world: World,
    pub keys: BTreeMap<EntityKey, EntityId>,
    pub genesis: Vec<EventEnvelope>,
}

impl Town {
    /// The café, begun at [`GENESIS`] with these holdings, item-transfer installed or not.
    pub fn begun(with_item_transfer: bool, holdings: &[(&str, &str)]) -> Self {
        let mut world = World::new();
        world.install(PresenceSystem).expect("presence");
        world.install(MovementSystem).expect("movement");
        world.install(ItemSystem).expect("item");
        world.install(InventorySystem).expect("inventory");
        if with_item_transfer {
            world
                .install(ItemTransferSystem)
                .expect("item-transfer installs after inventory and presence");
        }
        let mut keys = BTreeMap::new();
        for (name, entity_type) in [
            ("cafe", EntityType::Place),
            ("alice", EntityType::Person),
            ("bob", EntityType::Person),
            ("carol", EntityType::Person),
            ("apple", EntityType::Item),
            ("coffee", EntityType::Item),
            ("tea", EntityType::Item),
        ] {
            let id = world
                .create_entity(key(name), entity_type)
                .expect("created");
            keys.insert(key(name), id);
        }
        let cafe = PlaceId::new(keys[&key("cafe")], EntityType::Place).expect("a place");
        let mut facts: Vec<Emission> = Vec::new();
        for (person, x) in PLACED {
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

    /// Dispatches `giver` giving `count` of `item` to `taker` (any entity), as request `id` at second
    /// `id`.
    pub fn give(
        &mut self,
        giver: &str,
        taker: Option<&str>,
        item: &str,
        count: u32,
        id: u64,
    ) -> Result<Dispatched, KernelError> {
        let at = t(i64::try_from(id).expect("small"));
        let record = ActionRecord::new::<Give>(
            serde_json::to_vec(&Give::new(self.item(item), count)).expect("encodes"),
        );
        let mut intent = ActionIntent::new(ActionId::from_raw(id), self.id(giver), record, at);
        if let Some(taker) = taker {
            intent = intent.with_target(self.id(taker));
        }
        self.world.dispatch(&intent, at)
    }
}

/// The `give` affordances of an observation aimed at `target`, in observation order.
pub fn gives_to(observation: &Observation<Value>, target: EntityId) -> Vec<&Affordance<Value>> {
    observation
        .affordances()
        .iter()
        .filter(|affordance| affordance.action_type().as_str() == "give")
        .filter(|affordance| affordance.target() == Some(target))
        .collect()
}
