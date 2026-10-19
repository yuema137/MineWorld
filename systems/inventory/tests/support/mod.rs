//! A café, a street and a kiosk's operator; Alice and Bob in the café, Carol on the street; coffee, tea
//! and apples declared as kinds, a lantern that is not. Every kind and every holding seeded through
//! the section contract exactly as the World Pack loader seeds it — items' sections first, then
//! organizations', then people's (`ARC-36` item 7).
//!
//! `hands` is a stater that exists only in these tests: inventory provides no action, so something
//! must decide a transfer to exercise it. Its `pass` states inventory's `items-transferred` either
//! through the checked constructor, as every real pack must, or **forged** — bytes built by hand,
//! past the constructor — which is the defect the owner's reduction exists to refuse (`ARC-26`).
//! `workshop` is its twin for `items-produced` and `items-consumed` (`ARC-38`), installed only by the
//! tests of those two facts, so every older test's world is unchanged.

#![allow(dead_code)]

use std::collections::BTreeMap;

use mineworld_authoring::{AuthoredSection, Seeding};
use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionTypeId, EntityId, EntityKey, EntityType,
    EventEnvelope, ItemId, LocalPosition, Location, Millimetres, Observation, PersonId, PlaceId,
    Rejection, SystemId, Visibility, WorldTime,
};
use mineworld_inventory::{
    AuthoredHoldings, Holdings, InventorySystem, ItemsConsumed, ItemsProduced, ItemsTransferred,
};
use mineworld_item::{AuthoredItem, ItemSystem};
use mineworld_kernel::{
    Dispatched, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    World, WorldRead, WorldView,
};
use mineworld_presence::{PerceptionProvider, PresenceSystem, arrival};
use serde::{Deserialize, Serialize};

pub const GENESIS: WorldTime = WorldTime::from_seconds(0);

pub fn key(name: &str) -> EntityKey {
    EntityKey::new(name).expect("a key")
}

pub fn t(seconds: i64) -> WorldTime {
    WorldTime::from_seconds(seconds)
}

// ---------------------------------------------------------------------------------------------
// hands: the test-only stater
// ---------------------------------------------------------------------------------------------

/// The stater.
#[derive(Default)]
pub struct Hands;

impl SystemIdentity for Hands {
    const ID: SystemId = SystemId::from_static("hands");
}

/// Pass `count` of `item` to `to`; `forged` skips the checked constructor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pass {
    pub to: EntityId,
    pub item: ItemId,
    pub count: u32,
    pub forged: bool,
}

impl Action for Pass {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("pass");
    const OWNER: SystemId = Hands::ID;
}

fn read_pass(intent: &ActionIntent) -> Pass {
    serde_json::from_slice(intent.payload().payload()).expect("a pass")
}

impl System for Hands {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([InventorySystem::ID])
            .providing::<Pass>()
            .emitting::<ItemsTransferred>()
    }

    /// The honest path asks the owner's rule; the forged one asks nothing, so that what reaches the
    /// reduction is exactly what a careless stater would record.
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        let pass = read_pass(intent);
        if pass.forged {
            return Ok(());
        }
        mineworld_inventory::admit_transfer(world, intent.actor(), pass.to, pass.item, pass.count)
    }

    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let pass = read_pass(intent);
        let from = intent.actor();
        if pass.forged {
            let bytes = serde_json::to_vec(&serde_json::json!({
                "from": from,
                "to": pass.to,
                "item": pass.item,
                "count": pass.count,
            }))
            .expect("encodes");
            return Ok(vec![
                Emission::new::<ItemsTransferred>(bytes, Visibility::Participants)
                    .about(vec![from, pass.to])
                    .with_participants(vec![from, pass.to]),
            ]);
        }
        let fact =
            mineworld_inventory::transfer(&world.read(), from, pass.to, pass.item, pass.count)
                .expect("validated against the same rule");
        Ok(vec![fact])
    }
}

impl PerceptionProvider for Hands {}

// ---------------------------------------------------------------------------------------------
// make and use: the same stater for the facts that create and remove items (ARC-38)
// ---------------------------------------------------------------------------------------------

/// What a [`Change`] does to `holder`'s holdings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Way {
    /// `items-produced`.
    Make,
    /// `items-consumed`.
    UseUp,
}

/// Produce or use up `count` of `item` in `holder`'s holdings; `forged` skips the checked
/// constructor. The holder is named, not taken from the actor, so a test can name a non-holder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Change {
    pub holder: EntityId,
    pub item: ItemId,
    pub count: u32,
    pub way: Way,
    pub forged: bool,
}

impl Action for Change {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("change");
    const OWNER: SystemId = Workshop::ID;
}

/// A second test-only stater beside [`Hands`], for production and consumption. A system of its own,
/// so that `Hands`' declaration — and every existing test's world — is exactly as it was.
#[derive(Default)]
pub struct Workshop;

impl SystemIdentity for Workshop {
    const ID: SystemId = SystemId::from_static("workshop");
}

impl System for Workshop {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([InventorySystem::ID])
            .providing::<Change>()
            .emitting::<ItemsProduced>()
            .emitting::<ItemsConsumed>()
    }

    /// The honest path asks the owner's rule; the forged one asks nothing.
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        let change = read_change(intent);
        if change.forged {
            return Ok(());
        }
        match change.way {
            Way::Make => mineworld_inventory::admit_production(
                world,
                change.holder,
                change.item,
                change.count,
            ),
            Way::UseUp => mineworld_inventory::admit_consumption(
                world,
                change.holder,
                change.item,
                change.count,
            ),
        }
    }

    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let change = read_change(intent);
        if change.forged {
            let bytes = serde_json::to_vec(&serde_json::json!({
                "holder": change.holder,
                "item": change.item,
                "count": change.count,
            }))
            .expect("encodes");
            let fact = match change.way {
                Way::Make => Emission::new::<ItemsProduced>(bytes, Visibility::Participants),
                Way::UseUp => Emission::new::<ItemsConsumed>(bytes, Visibility::Participants),
            };
            return Ok(vec![
                fact.about(vec![change.holder])
                    .with_participants(vec![change.holder]),
            ]);
        }
        let read = world.read();
        let fact = match change.way {
            Way::Make => {
                mineworld_inventory::produce(&read, change.holder, change.item, change.count)
            }
            Way::UseUp => {
                mineworld_inventory::consume(&read, change.holder, change.item, change.count)
            }
        }
        .expect("validated against the same rule");
        Ok(vec![fact])
    }
}

impl PerceptionProvider for Workshop {}

fn read_change(intent: &ActionIntent) -> Change {
    serde_json::from_slice(intent.payload().payload()).expect("a change")
}

/// The town of [`Town::begun`] with the workshop installed after inventory.
pub fn begun_with_workshop() -> Town {
    let (mut town, facts) = Town::assemble();
    town.world
        .install(Workshop)
        .expect("the workshop installs after inventory");
    town.world.genesis(GENESIS, facts).expect("begins");
    town
}

/// Dispatches one `change` at the workshop, asked by `actor`.
pub fn change(
    world: &mut World,
    actor: EntityId,
    change: &Change,
    id: u64,
) -> Result<Dispatched, KernelError> {
    let at = t(i64::try_from(id).expect("small"));
    let intent = ActionIntent::new(
        ActionId::from_raw(id),
        actor,
        ActionRecord::new::<Change>(serde_json::to_vec(change).expect("encodes")),
        at,
    );
    world.dispatch(&intent, at)
}

// ---------------------------------------------------------------------------------------------
// The town
// ---------------------------------------------------------------------------------------------

/// Kinds declared by an `item:` section; `lantern` has none.
pub const KINDS: [(&str, &str); 3] = [("apple", "food"), ("coffee", "drink"), ("tea", "drink")];

/// Holdings as authored, organizations before people (genesis order).
pub const HOLDINGS: [(&str, &str); 3] = [
    ("kiosk", "{ coffee: 20, apple: 9 }"),
    ("alice", "{ coffee: 2, tea: 1 }"),
    ("bob", "{ apple: 1 }"),
];

/// An empty world with presence, item, inventory and hands installed.
pub fn compose() -> World {
    let mut world = World::new();
    world.install(PresenceSystem).expect("presence");
    world.install(ItemSystem).expect("item");
    world
        .install(InventorySystem)
        .expect("inventory installs after item");
    world
        .install(Hands)
        .expect("hands installs after inventory");
    world
}

pub struct Town {
    pub world: World,
    pub keys: BTreeMap<EntityKey, EntityId>,
}

impl Town {
    /// The town's entities, created in a fresh composed world; no fact stated yet.
    pub fn empty() -> Self {
        let mut world = compose();
        let mut keys = BTreeMap::new();
        for (name, entity_type) in [
            ("cafe", EntityType::Place),
            ("street", EntityType::Place),
            ("alice", EntityType::Person),
            ("bob", EntityType::Person),
            ("carol", EntityType::Person),
            ("apple", EntityType::Item),
            ("coffee", EntityType::Item),
            ("lantern", EntityType::Item),
            ("tea", EntityType::Item),
            ("kiosk", EntityType::Organization),
        ] {
            let id = world
                .create_entity(key(name), entity_type)
                .expect("created");
            keys.insert(key(name), id);
        }
        Self { world, keys }
    }

    /// The town and its genesis facts — placements, kinds, then holdings — not yet stated.
    pub fn assemble() -> (Self, Vec<Emission>) {
        let town = Self::empty();
        let mut facts = Vec::new();
        for (person, place, x) in [("alice", "cafe", 1_000), ("bob", "cafe", 2_000)] {
            facts.push(town.placed(person, place, x));
        }
        facts.push(town.placed("carol", "street", 1_000));
        for (item, category) in KINDS {
            facts.extend(town.kind(item, category));
        }
        for (holder, yaml) in HOLDINGS {
            facts.extend(town.seed_holdings(holder, yaml).expect("seeded"));
        }
        (town, facts)
    }

    /// The town, begun at [`GENESIS`]; and the genesis facts.
    pub fn begun() -> (Self, Vec<EventEnvelope>) {
        let (mut town, facts) = Self::assemble();
        let genesis = town.world.genesis(GENESIS, facts).expect("begins");
        (town, genesis)
    }

    pub fn id(&self, name: &str) -> EntityId {
        self.keys[&key(name)]
    }

    pub fn item(&self, name: &str) -> ItemId {
        ItemId::new(self.id(name), EntityType::Item).expect("an item")
    }

    fn placed(&self, person: &str, place: &str, x: i32) -> Emission {
        let person = PersonId::new(self.id(person), EntityType::Person).expect("a person");
        let place = PlaceId::new(self.id(place), EntityType::Place).expect("a place");
        let at = Location::in_place(place).with_local(LocalPosition::on_ground(
            Millimetres::new(x),
            Millimetres::new(1_000),
        ));
        arrival(&self.world.read(), person, at).expect("admitted")
    }

    pub fn kind(&self, item: &str, category: &str) -> Vec<Emission> {
        let authored: AuthoredItem =
            serde_saphyr::from_str(&format!("{{ category: {category}, name: {item} }}"))
                .expect("a section");
        let read = self.world.read();
        let seeding = Seeding::new(&read, &self.keys);
        ItemSystem::seed(&seeding, self.id(item), &authored).expect("seeded")
    }

    pub fn seed_holdings(&self, holder: &str, yaml: &str) -> Result<Vec<Emission>, Rejection> {
        let authored: AuthoredHoldings = serde_saphyr::from_str(yaml).expect("a section");
        let read = self.world.read();
        let seeding = Seeding::new(&read, &self.keys);
        InventorySystem::seed(&seeding, self.id(holder), &authored)
    }

    pub fn holdings(&self, holder: &str) -> Vec<(String, u32)> {
        holdings_of(&self.world, &self.keys, holder)
    }

    pub fn observation(&self, observer: &str) -> Observation<serde_json::Value> {
        let providers: [&dyn PerceptionProvider; 2] = [&PresenceSystem, &InventorySystem];
        mineworld_presence::observe(&self.world, self.id(observer), GENESIS, &providers)
    }
}

/// A holder's holdings as (item key, count), in item order.
pub fn holdings_of(
    world: &World,
    keys: &BTreeMap<EntityKey, EntityId>,
    holder: &str,
) -> Vec<(String, u32)> {
    let names: BTreeMap<EntityId, &EntityKey> = keys.iter().map(|(k, id)| (*id, k)).collect();
    world
        .read()
        .component::<Holdings>(keys[&key(holder)])
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

/// A `pass` request from `from`, as an intent with this id at this instant.
pub fn pass_intent(from: EntityId, pass: &Pass, id: u64, at: WorldTime) -> ActionIntent {
    ActionIntent::new(
        ActionId::from_raw(id),
        from,
        ActionRecord::new::<Pass>(serde_json::to_vec(pass).expect("encodes")),
        at,
    )
}

/// Dispatches one `pass` in `world`.
pub fn pass(
    world: &mut World,
    from: EntityId,
    pass: &Pass,
    id: u64,
) -> Result<Dispatched, KernelError> {
    let at = t(i64::try_from(id).expect("small"));
    world.dispatch(&pass_intent(from, pass, id, at), at)
}

/// Every component and relation of `world`, as bytes: "writes nothing" is compared on these.
pub fn state(world: &World) -> Vec<u8> {
    let snapshot = world.snapshot().expect("a snapshot");
    serde_json::to_vec(&(&snapshot.components, &snapshot.relations)).expect("encodes")
}
