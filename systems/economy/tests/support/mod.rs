//! A corner store that opens onto a street, composed from the real packs — presence, movement, item,
//! inventory, economy unless a test leaves it out, and employment when a test asks for wages. Kinds,
//! holdings, wallets, the shop and jobs are seeded through the section contract exactly as the World
//! Pack loader seeds them: items' sections first, then organizations', then people's (`ARC-36` item
//! 7), each file's sections in the order the world installs their owners.
//!
//! ```text
//! corner-store  runs the store: apple 100, bread 300, juice 200 (none in stock); holds apple 3,
//!               bread 2; wallet 1 000
//! poor-co       an employer with 100 in its wallet
//! alice         in the store, wallet 1 000                    — can buy apple and bread
//! bob           in the store, wallet 150                      — can buy only the apple
//! erin          in the store, wallet 1 000, carrying six      — can carry nothing more
//! carol         on the street, wallet 1 000                   — in no shop
//! pen           a kind the store does not price
//! ```

#![allow(dead_code)]

use std::collections::BTreeMap;

use mineworld_authoring::{AuthoredSection, Seeding};
use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, Affordance, EntityId, EntityKey, EntityType,
    EventEnvelope, ItemId, LocalPosition, Location, Millimetres, Observation, PersonId, PlaceId,
    WorldTime,
};
use mineworld_economy::{AuthoredEconomy, Buy, EconomySystem, Wallet};
use mineworld_employment::{AuthoredJob, EmploymentSystem};
use mineworld_inventory::{AuthoredHoldings, Holdings, InventorySystem};
use mineworld_item::{AuthoredItem, ItemSystem};
use mineworld_kernel::{Dispatched, Emission, KernelError, World};
use mineworld_movement::{MovementSystem, passage};
use mineworld_presence::{PerceptionProvider, PresenceSystem, arrival};
use serde_json::Value;

pub const GENESIS: WorldTime = WorldTime::from_seconds(0);
pub const HOUR: i64 = 3_600;

pub const KINDS: [(&str, &str); 4] = [
    ("apple", "food"),
    ("bread", "food"),
    ("juice", "drink"),
    ("pen", "goods"),
];

/// What the store holds at genesis, unless a test says otherwise.
pub const STOCK: &str = "{ apple: 3, bread: 2 }";

/// What erin carries: six, a person's capacity.
pub const ERIN_CARRIES: &str = "{ pen: 6 }";

pub const ECONOMY: [(&str, &str); 6] = [
    (
        "corner-store",
        "{ wallet: 1000, shop: { at: store, prices: { apple: 100, bread: 300, juice: 200 } } }",
    ),
    ("poor-co", "{ wallet: 100 }"),
    ("alice", "{ wallet: 1000 }"),
    ("bob", "{ wallet: 150 }"),
    ("carol", "{ wallet: 1000 }"),
    ("erin", "{ wallet: 1000 }"),
];

/// Jobs, when employment is installed: alice for the store, bob for poor-co, both 08:00–12:00 at 120
/// an hour — 480 for a full shift, which the store can pay and poor-co cannot.
pub const JOBS: [(&str, &str); 2] = [
    (
        "alice",
        "{ employer: corner-store, workplace: store, from: \"08:00\", until: \"12:00\", wage: 120 }",
    ),
    (
        "bob",
        "{ employer: poor-co, workplace: store, from: \"08:00\", until: \"12:00\", wage: 120 }",
    ),
];

pub const PLACED: [(&str, &str, i32); 4] = [
    ("alice", "store", 1_000),
    ("bob", "store", 2_000),
    ("erin", "store", 3_000),
    ("carol", "street", 1_000),
];

/// Which packs a world here has, beyond presence, movement, item and inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Packs {
    pub economy: bool,
    pub employment: bool,
}

pub const SHOP: Packs = Packs {
    economy: true,
    employment: false,
};

pub fn key(name: &str) -> EntityKey {
    EntityKey::new(name).expect("a key")
}

pub fn t(seconds: i64) -> WorldTime {
    WorldTime::from_seconds(seconds)
}

pub fn section(yaml: &str) -> Result<AuthoredEconomy, serde_saphyr::Error> {
    serde_saphyr::from_str(yaml)
}

/// Every provider a run consults, economy included whether or not the world installed it: a provider
/// of a pack the world does not have is dropped by perception's route map (`INV-10`).
pub fn providers() -> [&'static dyn PerceptionProvider; 6] {
    [
        &PresenceSystem,
        &MovementSystem,
        &ItemSystem,
        &InventorySystem,
        &EconomySystem,
        &EmploymentSystem,
    ]
}

pub struct Town {
    pub world: World,
    pub keys: BTreeMap<EntityKey, EntityId>,
    pub genesis: Vec<EventEnvelope>,
}

impl Town {
    /// The store, begun at [`GENESIS`] with these packs.
    pub fn begun(packs: Packs) -> Self {
        Self::begun_plus(packs, STOCK, |_| {})
    }

    /// The store with these packs and the store holding `stock`, `more` installed after the packs (a
    /// test-only stater), then begun.
    pub fn begun_plus(packs: Packs, stock: &str, more: impl FnOnce(&mut World)) -> Self {
        let mut world = World::new();
        world.install(PresenceSystem).expect("presence");
        world.install(MovementSystem).expect("movement");
        world.install(ItemSystem).expect("item");
        world.install(InventorySystem).expect("inventory");
        if packs.economy {
            world
                .install(EconomySystem)
                .expect("economy installs after inventory and presence, without employment");
        }
        if packs.employment {
            world
                .install(EmploymentSystem)
                .expect("employment installs after inventory and presence");
        }
        more(&mut world);
        let mut keys = BTreeMap::new();
        for (name, entity_type) in [
            ("store", EntityType::Place),
            ("street", EntityType::Place),
            ("alice", EntityType::Person),
            ("bob", EntityType::Person),
            ("carol", EntityType::Person),
            ("erin", EntityType::Person),
            ("apple", EntityType::Item),
            ("bread", EntityType::Item),
            ("juice", EntityType::Item),
            ("pen", EntityType::Item),
            ("corner-store", EntityType::Organization),
            ("poor-co", EntityType::Organization),
        ] {
            let id = world
                .create_entity(key(name), entity_type)
                .expect("created");
            keys.insert(key(name), id);
        }
        let place =
            |name: &str| PlaceId::new(keys[&key(name)], EntityType::Place).expect("a place");
        let mut facts: Vec<Emission> = vec![passage(place("store"), None, place("street"), None)];
        for (person, at, x) in PLACED {
            let person = PersonId::new(keys[&key(person)], EntityType::Person).expect("a person");
            let at = Location::in_place(place(at)).with_local(LocalPosition::on_ground(
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
                    serde_saphyr::from_str(&format!("{{ category: {category}, name: {item} }}"))
                        .expect("a section");
                facts.extend(
                    ItemSystem::seed(&seeding, keys[&key(item)], &authored).expect("seeded"),
                );
            }
            // Organizations' sections, then people's; within a file, inventory before economy before
            // employment — the order the world installs them.
            for holder in ["corner-store", "poor-co", "alice", "bob", "carol", "erin"] {
                let held = match holder {
                    "corner-store" => Some(stock),
                    "erin" => Some(ERIN_CARRIES),
                    _ => None,
                };
                if let Some(yaml) = held {
                    let authored: AuthoredHoldings =
                        serde_saphyr::from_str(yaml).expect("a section");
                    facts.extend(
                        InventorySystem::seed(&seeding, keys[&key(holder)], &authored)
                            .expect("seeded"),
                    );
                }
                if packs.economy
                    && let Some((_, yaml)) = ECONOMY.iter().find(|(name, _)| *name == holder)
                {
                    facts.extend(
                        EconomySystem::seed(
                            &seeding,
                            keys[&key(holder)],
                            &section(yaml).expect("a section"),
                        )
                        .expect("seeded"),
                    );
                }
                if packs.employment
                    && let Some((_, yaml)) = JOBS.iter().find(|(name, _)| *name == holder)
                {
                    let authored: AuthoredJob = serde_saphyr::from_str(yaml).expect("a job");
                    facts.extend(
                        EmploymentSystem::seed(&seeding, keys[&key(holder)], &authored)
                            .expect("seeded"),
                    );
                }
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

    pub fn wallet(&self, holder: &str) -> Option<u64> {
        self.world
            .read()
            .component::<Wallet>(self.id(holder))
            .map(Wallet::balance)
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

    /// Dispatches `buyer` buying one `item`, as request `id` at second `id`.
    pub fn buy(&mut self, buyer: &str, item: &str, id: u64) -> Result<Dispatched, KernelError> {
        let at = t(i64::try_from(id).expect("small"));
        let record = ActionRecord::new::<Buy>(
            serde_json::to_vec(&Buy::new(self.item(item))).expect("encodes"),
        );
        let intent = ActionIntent::new(ActionId::from_raw(id), self.id(buyer), record, at);
        self.world.dispatch(&intent, at)
    }

    /// Every component and relation, as bytes: "writes nothing" is compared on these.
    pub fn state(&self) -> Vec<u8> {
        let snapshot = self.world.snapshot().expect("a snapshot");
        serde_json::to_vec(&(&snapshot.components, &snapshot.relations)).expect("encodes")
    }
}

// ---------------------------------------------------------------------------------------------
// ledger: a test-only stater of economy's money-transferred, past every rule
// ---------------------------------------------------------------------------------------------

/// A stater that exists only in these tests. Its `post` states economy's `money-transferred` with
/// bytes built by hand — past `buy`, past the wage answer, past any check — which is the defect the
/// owner's reduction exists to refuse (`ARC-26`).
#[derive(Default)]
pub struct Ledger;

impl mineworld_kernel::SystemIdentity for Ledger {
    const ID: mineworld_contracts::SystemId = mineworld_contracts::SystemId::from_static("ledger");
}

/// Post `amount` from `from` to `to`, unchecked.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Post {
    pub from: EntityId,
    pub to: EntityId,
    pub amount: u64,
}

impl mineworld_contracts::Action for Post {
    const ACTION_TYPE: mineworld_contracts::ActionTypeId =
        mineworld_contracts::ActionTypeId::from_static("post");
    const OWNER: mineworld_contracts::SystemId = <Ledger as mineworld_kernel::SystemIdentity>::ID;
}

impl mineworld_kernel::System for Ledger {
    const VERSION: mineworld_kernel::SystemVersion = mineworld_kernel::SystemVersion::new(1);

    fn declaration(&self) -> mineworld_kernel::SystemDeclaration {
        mineworld_kernel::SystemDeclaration::of::<Self>()
            .depending_on([<EconomySystem as mineworld_kernel::SystemIdentity>::ID])
            .providing::<Post>()
            .emitting::<mineworld_economy::MoneyTransferred>()
    }

    fn validate(
        &self,
        _world: &mineworld_kernel::WorldRead<'_>,
        _intent: &ActionIntent,
    ) -> Result<(), mineworld_contracts::Rejection> {
        Ok(())
    }

    fn resolve(
        &self,
        _world: &mut mineworld_kernel::WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let post: Post = serde_json::from_slice(intent.payload().payload()).expect("a post");
        let bytes = serde_json::to_vec(&serde_json::json!({
            "from": post.from,
            "to": post.to,
            "amount": post.amount,
        }))
        .expect("encodes");
        Ok(vec![
            Emission::new::<mineworld_economy::MoneyTransferred>(
                bytes,
                mineworld_contracts::Visibility::Participants,
            )
            .about(vec![post.from, post.to])
            .with_participants(vec![post.from, post.to]),
        ])
    }
}

impl PerceptionProvider for Ledger {}

/// Dispatches one forged `post`, asked by `actor`.
pub fn post(town: &mut Town, actor: &str, post: &Post, id: u64) -> Result<Dispatched, KernelError> {
    let at = t(i64::try_from(id).expect("small"));
    let intent = ActionIntent::new(
        ActionId::from_raw(id),
        town.id(actor),
        ActionRecord::new::<Post>(serde_json::to_vec(post).expect("encodes")),
        at,
    );
    town.world.dispatch(&intent, at)
}

/// The `buy` affordances of an observation, in observation order.
pub fn buys(observation: &Observation<Value>) -> Vec<&Affordance<Value>> {
    observation
        .affordances()
        .iter()
        .filter(|affordance| affordance.action_type().as_str() == "buy")
        .collect()
}

/// The facts of one type, decoded through the owner's published type.
pub fn of<E: mineworld_contracts::Event + serde::de::DeserializeOwned>(
    facts: &[EventEnvelope],
) -> Vec<(E, EventEnvelope)> {
    facts
        .iter()
        .filter(|fact| *fact.event_type() == E::EVENT_TYPE)
        .map(|fact| {
            let decoded =
                serde_json::from_slice(fact.payload().payload_for::<E>().expect("labelled"))
                    .expect("decodes");
            (decoded, fact.clone())
        })
        .collect()
}
