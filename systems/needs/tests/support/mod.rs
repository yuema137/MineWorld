//! Two small test worlds for the needs pack, composed from the real packs — presence, movement, item,
//! inventory, consumption, sleep and needs — begun by genesis and driven by real requests. Kinds and
//! holdings are seeded through the section contract as the World Pack loader seeds them: items'
//! sections first, then people's (`ARC-36` item 7).
//!
//! ```text
//! flat-and-cafe   flat (tag residential) ─ cafe (tag cafe)
//!                 ana   needs: {}               flat   soup 2, coffee 1
//!                 ben   needs: { hunger: 900 }  flat   apple 1
//!                 cy    (no needs)              cafe   soup 1
//!                 soup  food   needs: { nourishment: { hunger: 600 } }
//!                 apple food   needs: { nourishment: { hunger: 200 } }
//!                 coffee drink (no needs)
//!                 no configure/needs.yaml
//! night-shift     ward (tag residential) ─ desk (tag desk)
//!                 dee  (tag night) needs: {}    ward   stew 2, tea 1
//!                 eli              needs: {}    ward   bread 1
//!                 fay  (no needs)               desk   stew 1
//!                 stew  food   needs: { nourishment: { hunger: 500 } }
//!                 bread food   needs: { nourishment: { hunger: 300 } }
//!                 tea   drink  (no needs)
//!                 configure/needs.yaml: thresholds [300, 500, 800];
//!                   { actor: night, hunger_per_hour: 90, rest_per_hour: 200 }
//! ```
//!
//! Every world begins at 00:00 on day 0. Nothing here is read from the pack under test: every number is
//! a literal of this layout.

#![allow(dead_code)]

use std::collections::BTreeMap;

use mineworld_authoring::{AuthoredSection, Seeding};
use mineworld_consumption::{ConsumptionSystem, Eat};
use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionResult, EntityId, EntityKey, EntityType,
    Event, EventEnvelope, ItemId, LocalPosition, Location, Millimetres, Observation, PersonId,
    PlaceId, Tag, Tags, Visibility, WorldTime,
};
use mineworld_inventory::{AuthoredHoldings, InventorySystem};
use mineworld_item::{AuthoredItem, ItemSystem};
use mineworld_kernel::{Emission, World};
use mineworld_movement::{MovementSystem, passage};
use mineworld_needs::{AuthoredNeeds, Needs, NeedsSystem};
use mineworld_presence::{PerceptionProvider, PresenceSystem, arrival};
use mineworld_sdk::interactions::{self, InteractionSection, Section, SectionConfigured};
use mineworld_sleep::{Sleep, SleepSystem, Wake};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// The genesis fact a configured `configure/needs.yaml` becomes, built as the loader builds it.
pub fn configured(section: &str, classes: &str) -> Emission {
    let section: Section<NeedsSystem> =
        Section::deserialize(&mut serde_json::Deserializer::from_str(section))
            .expect("the test's section decodes");
    let classes = serde_json::from_str(classes).expect("the test's classes decode");
    let resolved = interactions::resolve(&section, &classes).expect("resolves");
    Emission::new::<SectionConfigured<NeedsSystem>>(
        <NeedsSystem as InteractionSection>::encode(&resolved),
        Visibility::SystemInternal,
    )
}

/// When every test world begins: 00:00 on day 0.
pub const GENESIS: WorldTime = WorldTime::EPOCH;

/// An hour, in seconds.
pub const HOUR: i64 = 3_600;

/// Which test world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// flat and cafe; no section.
    FlatAndCafe,
    /// ward and desk; a configured section.
    NightShift,
}

/// night-shift's classes, as `configure/classes.yaml` would say them.
pub const NIGHT_CLASSES: &str = r#"[ { "class": "night", "of": "person", "tag": "night" } ]"#;

/// night-shift's `configure/needs.yaml`, rendered as JSON.
pub const NIGHT_SECTION: &str = r#"{
  "parameters": [
    { "thresholds": [300, 500, 800] },
    { "actor": "night", "hunger_per_hour": 90, "rest_per_hour": 200 }
  ]
}"#;

/// A layout: two places (key, tags), item kinds (key, category, needs section), people (key, tags,
/// place index, x, holdings, needs section).
struct Layout {
    places: [(&'static str, &'static str); 2],
    kinds: [(&'static str, &'static str, Option<&'static str>); 3],
    people: [Resident; 3],
}

type Resident = (
    &'static str,
    &'static [&'static str],
    usize,
    i32,
    &'static str,
    Option<&'static str>,
);

fn layout(kind: Kind) -> Layout {
    match kind {
        Kind::FlatAndCafe => Layout {
            places: [("flat", "residential"), ("cafe", "cafe")],
            kinds: [
                ("soup", "food", Some("{ nourishment: { hunger: 600 } }")),
                ("apple", "food", Some("{ nourishment: { hunger: 200 } }")),
                ("coffee", "drink", None),
            ],
            people: [
                ("ana", &[], 0, 1_000, "{ soup: 2, coffee: 1 }", Some("{}")),
                (
                    "ben",
                    &[],
                    0,
                    2_000,
                    "{ apple: 1 }",
                    Some("{ hunger: 900 }"),
                ),
                ("cy", &[], 1, 1_000, "{ soup: 1 }", None),
            ],
        },
        Kind::NightShift => Layout {
            places: [("ward", "residential"), ("desk", "desk")],
            kinds: [
                ("stew", "food", Some("{ nourishment: { hunger: 500 } }")),
                ("bread", "food", Some("{ nourishment: { hunger: 300 } }")),
                ("tea", "drink", None),
            ],
            people: [
                (
                    "dee",
                    &["night"],
                    0,
                    1_000,
                    "{ stew: 2, tea: 1 }",
                    Some("{}"),
                ),
                ("eli", &[], 0, 2_000, "{ bread: 1 }", Some("{}")),
                ("fay", &[], 1, 1_000, "{ stew: 1 }", None),
            ],
        },
    }
}

/// One test world.
pub struct Town {
    pub kind: Kind,
    pub world: World,
    pub providers: Vec<Box<dyn PerceptionProvider>>,
    pub keys: BTreeMap<String, EntityId>,
    /// Every fact recorded, genesis first, in log order.
    pub log: Vec<EventEnvelope>,
    clock: i64,
    next_action: u64,
}

/// The packs of a test world, installed in the order a World Pack lists them.
pub fn compose(needs: bool) -> (World, Vec<Box<dyn PerceptionProvider>>) {
    let mut world = World::new();
    let mut providers: Vec<Box<dyn PerceptionProvider>> = Vec::new();
    world.install(PresenceSystem).expect("presence");
    providers.push(Box::new(PresenceSystem));
    world.install(MovementSystem).expect("movement");
    providers.push(Box::new(MovementSystem));
    world.install(ItemSystem).expect("item");
    providers.push(Box::new(ItemSystem));
    world.install(InventorySystem).expect("inventory");
    providers.push(Box::new(InventorySystem));
    world.install(ConsumptionSystem).expect("consumption");
    providers.push(Box::new(ConsumptionSystem));
    world.install(SleepSystem).expect("sleep");
    providers.push(Box::new(SleepSystem));
    if needs {
        world
            .install(NeedsSystem)
            .expect("needs depends on nothing");
        providers.push(Box::new(NeedsSystem));
    }
    (world, providers)
}

/// A test world composed, its entities created, and its genesis facts — not yet stated.
pub struct Assembled {
    pub world: World,
    pub providers: Vec<Box<dyn PerceptionProvider>>,
    pub keys: BTreeMap<String, EntityId>,
    pub facts: Vec<Emission>,
}

/// `kind`, with or without the needs pack and its sections and section.
pub fn assemble(kind: Kind, needs: bool) -> Assembled {
    let (mut world, providers) = compose(needs);
    let shape = layout(kind);
    let mut keys = BTreeMap::new();
    let mut create = |world: &mut World, key: &str, entity_type, tags: &[&str]| {
        let id = world
            .create_authored_entity(
                EntityKey::new(key).expect("a key"),
                entity_type,
                Tags::new(tags.iter().map(|tag| Tag::new(*tag).expect("a tag"))),
                None,
            )
            .expect("created");
        keys.insert(key.to_owned(), id);
        id
    };
    let mut places = Vec::new();
    for (key, tag) in shape.places {
        let place = create(&mut world, key, EntityType::Place, &[tag]);
        places.push(PlaceId::new(place, EntityType::Place).expect("a place"));
    }
    for (key, _, _) in shape.kinds {
        create(&mut world, key, EntityType::Item, &[]);
    }
    for (key, tags, ..) in shape.people {
        create(&mut world, key, EntityType::Person, tags);
    }

    let local = |x: i32| LocalPosition::on_ground(Millimetres::new(x), Millimetres::new(1_000));
    let mut facts = vec![passage(
        places[0],
        Some(local(5_000)),
        places[1],
        Some(local(0)),
    )];
    for (key, _, place, x, ..) in shape.people {
        let person = PersonId::new(keys[key], EntityType::Person).expect("a person");
        let at = Location::in_place(places[place]).with_local(local(x));
        facts.push(arrival(&world.read(), person, at).expect("admitted"));
    }
    if needs && kind == Kind::NightShift {
        facts.push(configured(NIGHT_SECTION, NIGHT_CLASSES));
    }
    let entity_keys: BTreeMap<EntityKey, EntityId> = keys
        .iter()
        .map(|(key, id)| (EntityKey::new(key.as_str()).expect("a key"), *id))
        .collect();
    {
        let read = world.read();
        let seeding = Seeding::new(&read, &entity_keys);
        for (key, category, section) in shape.kinds {
            let item: AuthoredItem =
                serde_saphyr::from_str(&format!("{{ category: {category}, name: {key} }}"))
                    .expect("an item section");
            facts.extend(ItemSystem::seed(&seeding, keys[key], &item).expect("seeded"));
            if let (true, Some(section)) = (needs, section) {
                let authored: AuthoredNeeds = serde_saphyr::from_str(section).expect("decodes");
                facts.extend(NeedsSystem::seed(&seeding, keys[key], &authored).expect("seeded"));
            }
        }
        for (key, _, _, _, holdings, section) in shape.people {
            let held: AuthoredHoldings = serde_saphyr::from_str(holdings).expect("holdings");
            facts.extend(InventorySystem::seed(&seeding, keys[key], &held).expect("seeded"));
            if let (true, Some(section)) = (needs, section) {
                let authored: AuthoredNeeds = serde_saphyr::from_str(section).expect("decodes");
                facts.extend(NeedsSystem::seed(&seeding, keys[key], &authored).expect("seeded"));
            }
        }
    }
    Assembled {
        world,
        providers,
        keys,
        facts,
    }
}

pub fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("a payload encodes")
}

pub fn record<A: Action + Serialize>(action: &A) -> ActionRecord {
    ActionRecord::new::<A>(encode(action))
}

impl Town {
    /// `kind`, with the needs pack.
    pub fn new(kind: Kind) -> Self {
        Self::build(kind, true)
    }

    /// `kind` without the needs pack, its sections or its configuration.
    pub fn without_needs(kind: Kind) -> Self {
        Self::build(kind, false)
    }

    fn build(kind: Kind, needs: bool) -> Self {
        let Assembled {
            mut world,
            providers,
            keys,
            facts,
        } = assemble(kind, needs);
        let log = world.genesis(GENESIS, facts).expect("begins");
        Self {
            kind,
            world,
            providers,
            keys,
            log,
            clock: GENESIS.seconds(),
            next_action: 1,
        }
    }

    /// A world restored from saved state, continuing at `clock`.
    pub fn resumed(kind: Kind, world: World, providers: Vec<Box<dyn PerceptionProvider>>) -> Self {
        let keys = assemble(kind, true).keys;
        let clock = world.read().at().seconds();
        Self {
            kind,
            world,
            providers,
            keys,
            log: Vec::new(),
            clock,
            next_action: 1_000_000,
        }
    }

    pub fn id(&self, name: &str) -> EntityId {
        self.keys[name]
    }

    pub fn item(&self, name: &str) -> ItemId {
        ItemId::new(self.id(name), EntityType::Item).expect("an item")
    }

    /// The instant of the last request, or of the last [`pass_to`](Self::pass_to).
    pub fn now(&self) -> WorldTime {
        WorldTime::from_seconds(self.clock)
    }

    /// Advances to exactly `at`, firing whatever falls due; the facts it caused.
    pub fn pass_to(&mut self, at: WorldTime) -> Vec<EventEnvelope> {
        self.clock = at.seconds();
        let facts = self.world.advance_to(at).expect("advances").into_events();
        self.log.extend(facts.iter().cloned());
        facts
    }

    /// Submits a request at the current instant.
    pub fn submit(
        &mut self,
        actor: EntityId,
        record: ActionRecord,
    ) -> (ActionResult, Vec<EventEnvelope>) {
        let when = self.now();
        let id = ActionId::from_raw(self.next_action);
        self.next_action += 1;
        let intent = ActionIntent::new(id, actor, record, when);
        let dispatched = self.world.dispatch(&intent, when).expect("answers");
        self.log.extend(dispatched.events().iter().cloned());
        (dispatched.result().clone(), dispatched.events().to_vec())
    }

    /// `who` eats one `item` now.
    pub fn eat(&mut self, who: &str, item: &str) -> ActionResult {
        let item = self.item(item);
        self.submit(self.id(who), record(&Eat::new(item))).0
    }

    /// `who` sleeps for `minutes` from now.
    pub fn sleep(&mut self, who: &str, minutes: u16) -> ActionResult {
        self.submit(self.id(who), record(&Sleep::new(minutes))).0
    }

    /// `who` gets up now.
    pub fn wake(&mut self, who: &str) -> ActionResult {
        self.submit(self.id(who), record(&Wake {})).0
    }

    /// `who`'s stored needs.
    pub fn needs(&self, who: &str) -> Option<Needs> {
        self.world.read().component::<Needs>(self.id(who)).copied()
    }

    pub fn observation(&self, who: &str) -> Observation<serde_json::Value> {
        let providers: Vec<&dyn PerceptionProvider> =
            self.providers.iter().map(AsRef::as_ref).collect();
        mineworld_presence::observe(&self.world, self.id(who), self.now(), &providers)
    }

    /// The `needs` record `observer` is disclosed about `subject`, if any.
    pub fn disclosed(&self, observer: &str, subject: &str) -> Option<serde_json::Value> {
        let subject = self.id(subject);
        self.observation(observer)
            .entities()
            .iter()
            .flat_map(|entity| entity.components())
            .find(|record| {
                record.entity() == subject && record.component_type().as_str() == "needs"
            })
            .map(|record| record.payload().clone())
    }

    /// `who`'s disclosed `(hunger, tiredness)`.
    pub fn levels(&self, who: &str) -> (u64, u64) {
        let record = self.disclosed(who, who).expect("disclosed to oneself");
        let number = |key: &str| record[key].as_u64().expect("a number");
        (number("hunger"), number("tiredness"))
    }
}

/// The facts of type `E` among `facts`, decoded.
pub fn facts_of<E: Event + DeserializeOwned>(facts: &[EventEnvelope]) -> Vec<(WorldTime, E)> {
    facts
        .iter()
        .filter(|fact| *fact.event_type() == E::EVENT_TYPE)
        .map(|fact| {
            let payload = fact.payload().payload_for::<E>().expect("its type");
            (
                fact.at(),
                serde_json::from_slice(payload).expect("it decodes"),
            )
        })
        .collect()
}

pub fn accepted(result: &ActionResult) -> bool {
    matches!(result, ActionResult::Accepted { .. })
}

pub fn hour(h: i64) -> WorldTime {
    WorldTime::from_seconds(h * HOUR)
}
