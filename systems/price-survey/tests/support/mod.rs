//! W1 (`pr-s31c-price-survey.md` §8): a corner store, a café and the street between them, composed
//! from the real packs — presence, movement, item, inventory, economy, price-survey. Kinds, holdings,
//! wallets, the shop and the surveyor are seeded through the section contract exactly as the World
//! Pack loader seeds them: the configuration, then items' sections, organizations', people's
//! (`ARC-36` item 7).
//!
//! ```text
//! kinds         apple (food), bread (food), juice (drink), pen (goods)
//! corner-store  runs the store: asks apple 100, bread 300, juice 200, pen 150; holds apple 3,
//!               bread 2, pen 2 (juice 0)
//! grace         in the store, 2 000 mm from alice; holds bread 2; asks bread 260, pen 120
//! pia           the surveyor: { kinds: { categories: [food, drink] }, depth: 4, window: "1d",
//!               resurvey_after: "1h" }, wallet 2 000; in the store unless a test moves her
//! alice, bob    buyers, wallet 2 000 each, in the store
//! nat           a person with no section, in the store
//! ```

#![allow(dead_code)]

use std::collections::BTreeMap;

use mineworld_authoring::{
    Attached, AuthoredSection, ConfigurationContext, EntityClasses, PackConfiguration, Seeding,
};
use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, ActionResult, Affordance, EntityId, EntityKey,
    EntityType, Event, EventEnvelope, ItemId, LocalPosition, Location, Millimetres, Observation,
    PersonId, PlaceId, Rejection, RejectionCode, WorldTime,
};
use mineworld_economy::{AuthoredEconomy, Buy, EconomyConfiguration, EconomySystem};
use mineworld_inventory::{AuthoredHoldings, InventorySystem};
use mineworld_item::{AuthoredItem, ItemSystem};
use mineworld_kernel::{Dispatched, Emission, KernelError, World};
use mineworld_movement::{MovementSystem, passage};
use mineworld_presence::{PerceptionProvider, PresenceSystem, arrival};
use mineworld_price_survey::{AuthoredSurvey, PriceBook, PriceSurveySystem, Survey};
use serde::de::DeserializeOwned;
use serde_json::Value;

pub const HOUR: i64 = 3_600;
pub const DAY: i64 = 86_400;

/// Pia's section in W1.
pub const PIA: &str =
    "{ kinds: { categories: [food, drink] }, depth: 4, window: \"1d\", resurvey_after: \"1h\" }";

pub const KINDS: [(&str, &str); 4] = [
    ("apple", "food"),
    ("bread", "food"),
    ("juice", "drink"),
    ("pen", "goods"),
];

const CURRENCY: &str =
    "{ currency: { name: test coin, symbol: tc, exponent: 2, symbol_at: after } }";

/// W1, or a variant of it.
#[derive(Debug, Clone)]
pub struct Setup {
    /// Pia's `price-survey:` section; `None` for none.
    pub pia: Option<String>,
    /// Where Pia stands at genesis.
    pub pia_at: &'static str,
    /// What the store holds.
    pub stock: String,
}

impl Default for Setup {
    fn default() -> Self {
        Self {
            pia: Some(PIA.to_owned()),
            pia_at: "store",
            stock: "{ apple: 3, bread: 2, pen: 2 }".to_owned(),
        }
    }
}

impl Setup {
    #[must_use]
    pub fn pia_at(mut self, place: &'static str) -> Self {
        self.pia_at = place;
        self
    }

    #[must_use]
    pub fn pia(mut self, section: &str) -> Self {
        self.pia = Some(section.to_owned());
        self
    }

    #[must_use]
    pub fn stock(mut self, stock: &str) -> Self {
        self.stock = stock.to_owned();
        self
    }
}

pub fn key(name: &str) -> EntityKey {
    EntityKey::new(name).expect("a key")
}

pub fn t(seconds: i64) -> WorldTime {
    WorldTime::from_seconds(seconds)
}

/// A `price-survey:` section decoded from YAML exactly as the loader decodes it.
pub fn section(yaml: &str) -> Result<AuthoredSurvey, serde_saphyr::Error> {
    serde_saphyr::from_str(yaml)
}

pub fn providers() -> [&'static dyn PerceptionProvider; 6] {
    [
        &PresenceSystem,
        &MovementSystem,
        &ItemSystem,
        &InventorySystem,
        &EconomySystem,
        &PriceSurveySystem,
    ]
}

pub struct Town {
    pub world: World,
    pub keys: BTreeMap<EntityKey, EntityId>,
    /// Every fact so far, genesis first.
    pub log: Vec<EventEnvelope>,
    next: u64,
}

const ECONOMY: [(&str, &str); 6] = [
    (
        "corner-store",
        "{ wallet: 1000, shop: { at: store, prices: { apple: 100, bread: 300, juice: 200, pen: 150 } } }",
    ),
    ("alice", "{ wallet: 2000 }"),
    ("bob", "{ wallet: 2000 }"),
    ("grace", "{ wallet: 1000, asks: { bread: 260, pen: 120 } }"),
    ("nat", "{ wallet: 1000 }"),
    ("pia", "{ wallet: 2000 }"),
];

impl Town {
    pub fn begun() -> Self {
        Self::build(&Setup::default()).expect("W1 begins")
    }

    pub fn with(setup: &Setup) -> Self {
        Self::build(setup).expect("the variant begins")
    }

    /// W1 from `setup`, or genesis's own refusal.
    pub fn build(setup: &Setup) -> Result<Self, KernelError> {
        let mut world = World::new();
        world.install(PresenceSystem).expect("presence");
        world.install(MovementSystem).expect("movement");
        world.install(ItemSystem).expect("item");
        world.install(InventorySystem).expect("inventory");
        world.install(EconomySystem).expect("economy");
        world
            .install(PriceSurveySystem)
            .expect("price-survey installs after economy, item and presence");
        let mut keys = BTreeMap::new();
        for (name, entity_type) in [
            ("store", EntityType::Place),
            ("cafe", EntityType::Place),
            ("street", EntityType::Place),
            ("alice", EntityType::Person),
            ("bob", EntityType::Person),
            ("grace", EntityType::Person),
            ("nat", EntityType::Person),
            ("pia", EntityType::Person),
            ("apple", EntityType::Item),
            ("bread", EntityType::Item),
            ("juice", EntityType::Item),
            ("pen", EntityType::Item),
            ("corner-store", EntityType::Organization),
        ] {
            let id = world
                .create_entity(key(name), entity_type)
                .expect("created");
            keys.insert(key(name), id);
        }
        let place =
            |name: &str| PlaceId::new(keys[&key(name)], EntityType::Place).expect("a place");
        let mut facts: Vec<Emission> = vec![
            passage(place("store"), None, place("street"), None),
            passage(place("cafe"), None, place("street"), None),
        ];
        let placed: [(&str, &str, i32, i32); 5] = [
            ("alice", "store", 1_000, 1_000),
            ("bob", "store", 2_000, 1_000),
            ("grace", "store", 1_000, 3_000),
            ("nat", "store", 3_000, 1_000),
            ("pia", setup.pia_at, 4_000, 1_000),
        ];
        for (person, at, x, y) in placed {
            let person = PersonId::new(keys[&key(person)], EntityType::Person).expect("a person");
            let at = Location::in_place(place(at)).with_local(LocalPosition::on_ground(
                Millimetres::new(x),
                Millimetres::new(y),
            ));
            facts.push(arrival(&world.read(), person, at).expect("admitted"));
        }
        {
            let read = world.read();
            let seeding = Seeding::new(&read, &keys);
            let (classes, attached) = (EntityClasses::default(), Attached::none());
            let configuration: EconomyConfiguration =
                serde_saphyr::from_str(CURRENCY).expect("a configuration");
            facts.extend(
                <EconomySystem as PackConfiguration>::seed(
                    &seeding,
                    &configuration,
                    &ConfigurationContext::new(&classes, &attached),
                )
                .expect("seeded"),
            );
            for (item, category) in KINDS {
                let authored: AuthoredItem =
                    serde_saphyr::from_str(&format!("{{ category: {category}, name: {item} }}"))
                        .expect("a section");
                facts.extend(
                    ItemSystem::seed(&seeding, keys[&key(item)], &authored).expect("seeded"),
                );
            }
            for (holder, yaml) in ECONOMY {
                let held = match holder {
                    "corner-store" => Some(setup.stock.as_str()),
                    "grace" => Some("{ bread: 2 }"),
                    _ => None,
                };
                if let Some(held) = held {
                    let authored: AuthoredHoldings =
                        serde_saphyr::from_str(held).expect("a section");
                    facts.extend(
                        InventorySystem::seed(&seeding, keys[&key(holder)], &authored)
                            .expect("seeded"),
                    );
                }
                let authored: AuthoredEconomy = serde_saphyr::from_str(yaml).expect("a section");
                facts.extend(
                    <EconomySystem as AuthoredSection>::seed(
                        &seeding,
                        keys[&key(holder)],
                        &authored,
                    )
                    .expect("seeded"),
                );
                if holder == "pia"
                    && let Some(yaml) = &setup.pia
                {
                    facts.extend(
                        <PriceSurveySystem as AuthoredSection>::seed(
                            &seeding,
                            keys[&key("pia")],
                            &section(yaml).expect("pia's section decodes"),
                        )
                        .expect("seeded"),
                    );
                }
            }
        }
        let log = world.genesis(t(0), facts)?;
        Ok(Self {
            world,
            keys,
            log,
            next: 1,
        })
    }

    pub fn id(&self, name: &str) -> EntityId {
        self.keys[&key(name)]
    }

    pub fn item(&self, name: &str) -> ItemId {
        ItemId::new(self.id(name), EntityType::Item).expect("an item")
    }

    pub fn place(&self, name: &str) -> PlaceId {
        PlaceId::new(self.id(name), EntityType::Place).expect("a place")
    }

    /// Dispatches `actor`'s `record` at second `at`, aimed at `target` when named.
    pub fn act(
        &mut self,
        actor: &str,
        target: Option<&str>,
        record: ActionRecord,
        at: i64,
    ) -> Dispatched {
        let mut intent =
            ActionIntent::new(ActionId::from_raw(self.next), self.id(actor), record, t(at));
        self.next += 1;
        if let Some(target) = target {
            intent = intent.with_target(self.id(target));
        }
        let dispatched = self.world.dispatch(&intent, t(at)).expect("dispatched");
        self.log.extend(dispatched.events().iter().cloned());
        dispatched
    }

    /// `buyer` buys one `item` at `price`, from `seller` when named, else from the shop it stands in.
    pub fn buy(
        &mut self,
        buyer: &str,
        seller: Option<&str>,
        item: &str,
        price: u64,
        at: i64,
    ) -> Vec<EventEnvelope> {
        let record = ActionRecord::new::<Buy>(
            serde_json::to_vec(&Buy::new(self.item(item), price)).expect("encodes"),
        );
        let dispatched = self.act(buyer, seller, record, at);
        assert!(
            matches!(dispatched.result(), ActionResult::Accepted { .. }),
            "{buyer} buys {item}: {:?}",
            dispatched.result()
        );
        dispatched.events().to_vec()
    }

    /// `actor` takes `survey` at second `at`.
    pub fn survey(&mut self, actor: &str, at: i64) -> Dispatched {
        let record = ActionRecord::new::<Survey>(serde_json::to_vec(&Survey {}).expect("encodes"));
        self.act(actor, None, record, at)
    }

    /// The world, advanced to second `at` — any fact stated on the way is kept in [`Town::log`] —
    /// observed by `observer`.
    pub fn observation(&mut self, observer: &str, at: i64) -> Observation<Value> {
        let advanced = self.world.advance_to(t(at)).expect("advances");
        self.log.extend(advanced.events().iter().cloned());
        mineworld_presence::observe(&self.world, self.id(observer), t(at), &providers())
    }

    /// The book `observer` is shown of itself at second `at`.
    pub fn book(&mut self, observer: &str, at: i64) -> Option<PriceBook> {
        record(
            &self.observation(observer, at),
            self.id(observer),
            "survey-book",
        )
    }

    /// Every component, relation and process, as bytes: "state unchanged" is compared on these.
    pub fn state(&self) -> Vec<u8> {
        let snapshot = self.world.snapshot().expect("a snapshot");
        serde_json::to_vec(&(
            &snapshot.components,
            &snapshot.relations,
            &snapshot.time.processes,
        ))
        .expect("encodes")
    }

    /// The facts of the log `name` perceived, by presence's audience rule (`ARC-43`).
    pub fn perceived(&self, name: &str) -> Vec<EventEnvelope> {
        mineworld_presence::audience::perceived_by(self.log.iter().cloned(), self.id(name), None)
            .collect()
    }
}

/// The record of component type `kind` about `subject` in `observation`, decoded.
pub fn record<T: DeserializeOwned>(
    observation: &Observation<Value>,
    subject: EntityId,
    kind: &str,
) -> Option<T> {
    observation
        .entities()
        .iter()
        .filter(|entity| entity.id() == subject)
        .flat_map(|entity| entity.components())
        .find(|component| component.component_type().as_str() == kind)
        .map(|component| serde_json::from_value(component.payload().clone()).expect("decodes"))
}

/// Every record of component type `kind` in `observation`, about anybody.
pub fn records_of(observation: &Observation<Value>, kind: &str) -> usize {
    observation
        .entities()
        .iter()
        .flat_map(|entity| entity.components())
        .filter(|component| component.component_type().as_str() == kind)
        .count()
}

/// The facts of type `E` among `facts`, decoded.
pub fn of<E: Event + DeserializeOwned>(facts: &[EventEnvelope]) -> Vec<(E, EventEnvelope)> {
    facts
        .iter()
        .filter(|fact| *fact.event_type() == E::EVENT_TYPE)
        .map(|fact| {
            let bytes = fact.payload().payload_for::<E>().expect("its type");
            (
                serde_json::from_slice(bytes).expect("decodes"),
                fact.clone(),
            )
        })
        .collect()
}

/// The `survey` affordances of an observation.
pub fn surveys(observation: &Observation<Value>) -> Vec<&Affordance<Value>> {
    observation
        .affordances()
        .iter()
        .filter(|affordance| affordance.action_type().as_str() == "survey")
        .collect()
}

/// A dispatch's rejection, if it was rejected.
pub fn rejection(dispatched: &Dispatched) -> Option<&Rejection> {
    match dispatched.result() {
        ActionResult::Rejected(reason) => Some(reason),
        _ => None,
    }
}

/// The `Rejection::System` carrying `code`.
pub fn code(code: &'static str) -> Rejection {
    Rejection::System {
        code: RejectionCode::from_static(code),
        detail: None,
    }
}
