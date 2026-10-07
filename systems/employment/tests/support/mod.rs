//! A café that opens onto a street, an employer, and four employees with the same job — the café,
//! 08:00–12:00, 120 minor units an hour, four coffees and two croissants a full shift — composed from
//! the real packs: presence, movement, item, inventory and, unless a test leaves it out, employment.
//! Kinds, holdings and jobs are seeded through the section contract exactly as the World Pack loader
//! seeds them: items' sections first, then organizations', then people's (`ARC-36` item 7).
//!
//! Alice stays in the café, Bob leaves at 10:00, Carol arrives at 11:00, Dave never comes — the
//! script [`Town::work_one_day`] walks them.

#![allow(dead_code)]

use std::collections::BTreeMap;

use mineworld_authoring::{AuthoredSection, Seeding};
use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, ActionResult, EntityId, EntityKey, EntityType,
    EventEnvelope, ItemId, LocalPosition, Location, Millimetres, OrganizationId, PersonId, PlaceId,
    WorldTime,
};
use mineworld_employment::{AuthoredJob, EmploymentSystem};
use mineworld_inventory::{AuthoredHoldings, Holdings, InventorySystem};
use mineworld_item::{AuthoredItem, ItemSystem};
use mineworld_kernel::{Emission, World};
use mineworld_movement::{Move, MovementSystem, passage};
use mineworld_presence::{PerceptionProvider, PresenceSystem, arrival};

/// Midnight of day 0.
pub const GENESIS: WorldTime = WorldTime::from_seconds(0);
pub const HOUR: i64 = 3_600;

/// The job everybody here holds.
pub const JOB: &str = "{ employer: cafe-company, workplace: cafe, from: \"08:00\", until: \"12:00\", \
                       wage: 120, produces: { coffee: 4, croissant: 2 } }";

/// Where each employee is at genesis.
pub const PLACED: [(&str, &str); 4] = [
    ("alice", "cafe"),
    ("bob", "cafe"),
    ("carol", "street"),
    ("dave", "street"),
];

pub fn key(name: &str) -> EntityKey {
    EntityKey::new(name).expect("a key")
}

pub fn t(seconds: i64) -> WorldTime {
    WorldTime::from_seconds(seconds)
}

pub fn job(yaml: &str) -> Result<AuthoredJob, serde_saphyr::Error> {
    serde_saphyr::from_str(yaml)
}

/// Every provider a run consults.
pub fn providers() -> [&'static dyn PerceptionProvider; 5] {
    [
        &PresenceSystem,
        &MovementSystem,
        &ItemSystem,
        &InventorySystem,
        &EmploymentSystem,
    ]
}

/// An empty world with presence, movement, item, inventory and, if asked, employment.
pub fn compose(with_employment: bool) -> World {
    let mut world = World::new();
    world.install(PresenceSystem).expect("presence");
    world.install(MovementSystem).expect("movement");
    world.install(ItemSystem).expect("item");
    world.install(InventorySystem).expect("inventory");
    if with_employment {
        world
            .install(EmploymentSystem)
            .expect("employment installs after inventory and presence");
    }
    world
}

pub struct Town {
    pub world: World,
    pub keys: BTreeMap<EntityKey, EntityId>,
    pub genesis: Vec<EventEnvelope>,
}

impl Town {
    /// The entities of the town in `world`, and its genesis facts — the passage, placements, kinds,
    /// the employer's opening stock, then every job — not yet stated.
    pub fn assemble(
        mut world: World,
        with_jobs: bool,
    ) -> (World, BTreeMap<EntityKey, EntityId>, Vec<Emission>) {
        let mut keys = BTreeMap::new();
        for (name, entity_type) in [
            ("cafe", EntityType::Place),
            ("street", EntityType::Place),
            ("alice", EntityType::Person),
            ("bob", EntityType::Person),
            ("carol", EntityType::Person),
            ("dave", EntityType::Person),
            ("coffee", EntityType::Item),
            ("croissant", EntityType::Item),
            ("cafe-company", EntityType::Organization),
        ] {
            let id = world
                .create_entity(key(name), entity_type)
                .expect("created");
            keys.insert(key(name), id);
        }
        let place =
            |name: &str| PlaceId::new(keys[&key(name)], EntityType::Place).expect("a place");
        let mut facts = vec![passage(place("cafe"), None, place("street"), None)];
        for (index, (person, at)) in PLACED.into_iter().enumerate() {
            let person = PersonId::new(keys[&key(person)], EntityType::Person).expect("a person");
            let x = 1_000 * (i32::try_from(index).expect("small") + 1);
            let at = Location::in_place(place(at)).with_local(LocalPosition::on_ground(
                Millimetres::new(x),
                Millimetres::new(1_000),
            ));
            facts.push(arrival(&world.read(), person, at).expect("admitted"));
        }
        {
            let read = world.read();
            let seeding = Seeding::new(&read, &keys);
            for (item, category) in [("coffee", "drink"), ("croissant", "food")] {
                let authored: AuthoredItem =
                    serde_saphyr::from_str(&format!("category: {category}")).expect("a section");
                facts.extend(
                    ItemSystem::seed(&seeding, keys[&key(item)], &authored).expect("seeded"),
                );
            }
            let stock: AuthoredHoldings =
                serde_saphyr::from_str("{ coffee: 1 }").expect("a section");
            facts.extend(
                InventorySystem::seed(&seeding, keys[&key("cafe-company")], &stock)
                    .expect("seeded"),
            );
            if with_jobs {
                for (person, _) in PLACED {
                    facts.extend(
                        EmploymentSystem::seed(
                            &seeding,
                            keys[&key(person)],
                            &job(JOB).expect("a job"),
                        )
                        .expect("seeded"),
                    );
                }
            }
        }
        (world, keys, facts)
    }

    /// The town, begun at [`GENESIS`], employment installed (and every job seeded) or not.
    pub fn begun(with_employment: bool) -> Self {
        let (mut world, keys, facts) = Self::assemble(compose(with_employment), with_employment);
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

    pub fn person(&self, name: &str) -> PersonId {
        PersonId::new(self.id(name), EntityType::Person).expect("a person")
    }

    pub fn employer(&self) -> OrganizationId {
        OrganizationId::new(self.id("cafe-company"), EntityType::Organization)
            .expect("an organization")
    }

    pub fn item(&self, name: &str) -> ItemId {
        ItemId::new(self.id(name), EntityType::Item).expect("an item")
    }

    /// What the employer holds, as (item key, count), in item order.
    pub fn stock(&self) -> Vec<(String, u32)> {
        let names: BTreeMap<EntityId, &EntityKey> =
            self.keys.iter().map(|(k, id)| (*id, k)).collect();
        self.world
            .read()
            .component::<Holdings>(self.id("cafe-company"))
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

    /// Advances to `at`, returning every fact recorded on the way.
    pub fn advance(&mut self, at: i64) -> Vec<EventEnvelope> {
        self.world
            .advance_to(t(at))
            .expect("the world advances")
            .into_events()
    }

    /// Advances to `at`, then walks `who` into `place`, returning every fact on the way.
    pub fn walk(&mut self, who: &str, place: &str, at: i64, id: u64) -> Vec<EventEnvelope> {
        let mut facts = self.advance(at);
        let place = PlaceId::new(self.id(place), EntityType::Place).expect("a place");
        let to = Location::in_place(place).with_local(LocalPosition::on_ground(
            Millimetres::new(500),
            Millimetres::new(500),
        ));
        let intent = ActionIntent::new(
            ActionId::from_raw(id),
            self.id(who),
            ActionRecord::new::<Move>(serde_json::to_vec(&Move::new(to)).expect("encodes")),
            t(at),
        );
        let done = self
            .world
            .dispatch(&intent, t(at))
            .expect("dispatch answers");
        assert!(
            matches!(done.result(), ActionResult::Accepted { .. }),
            "{who} walks: {:?}",
            done.result()
        );
        facts.extend(done.events().iter().cloned());
        facts
    }

    /// Day 0, scripted: Bob leaves the café at 10:00, Carol walks in at 11:00, and the world runs to
    /// 13:00. Every fact recorded after genesis, in order.
    pub fn work_one_day(&mut self) -> Vec<EventEnvelope> {
        let mut facts = self.walk("bob", "street", 10 * HOUR, 1);
        facts.extend(self.walk("carol", "cafe", 11 * HOUR, 2));
        facts.extend(self.advance(13 * HOUR));
        facts
    }
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
