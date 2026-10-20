//! Two small test worlds for the sleep pack, composed from the real packs, begun by genesis and driven
//! by real requests, the way a server drives a world.
//!
//! ```text
//! flat-and-park   flat (tag residential) ─ street ─ park (tag park)
//!                 ana and ben in the flat, a metre apart; cy in the street
//!                 no configure/sleep.yaml
//! hostel          dorm (tag dormitory) ─ hall ─ garden (tag garden)
//!                 ana and ben (tag traveller) in the dorm; cy in the hall
//!                 classes: dorm-place = place tag dormitory, traveller = person tag traveller
//!                 configure/sleep.yaml: permit dorm-place; in the garden region, permit everyone
//!                 but travellers; woken_by_talk false; longest_minutes 600
//! ```
//!
//! Nothing here is read from the pack under test: every number is a literal of this layout.

#![allow(dead_code)]

use std::collections::BTreeMap;

use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionResult, EntityId, EntityKey, EntityType,
    Event, EventEnvelope, LocalPosition, Location, Millimetres, Observation, PersonId, PlaceId,
    Tag, Tags, Visibility, WorldTime,
};
use mineworld_conversation::{ConversationSystem, Talk, Utterance};
use mineworld_kernel::{Emission, World};
use mineworld_movement::{Destination, MovementSystem, WalkEnded, WalkStep, WalkTo, passage};
use mineworld_presence::{PerceptionProvider, Presence, PresenceSystem, arrival};
use mineworld_sdk::interactions::{self, InteractionSection, Section, SectionConfigured};
use mineworld_sleep::{Asleep, Sleep, SleepProcess, SleepSystem, Wake};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// When every test world begins: 01:00 on day 0.
pub const GENESIS: WorldTime = WorldTime::from_seconds(3_600);

/// Which test world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// flat, street and park; no section.
    FlatAndPark,
    /// dorm, hall and garden; a configured section.
    Hostel,
}

/// The hostel's classes, as `configure/classes.yaml` would say them.
pub const HOSTEL_CLASSES: &str = r#"[
  { "class": "dorm-place", "of": "place", "tag": "dormitory" },
  { "class": "traveller", "of": "person", "tag": "traveller" }
]"#;

/// The hostel's `configure/sleep.yaml`, rendered as JSON.
pub const HOSTEL_SECTION: &str = r#"{
  "rules": [ { "action": "sleep", "place": "dorm-place", "effect": "permit" } ],
  "parameters": [ { "woken_by_talk": false, "longest_minutes": 600 } ],
  "regions": {
    "garden": {
      "rules": [
        { "action": "sleep", "effect": "permit" },
        { "action": "sleep", "actor": "traveller", "effect": "forbid" }
      ]
    }
  }
}"#;

/// The genesis fact a configured `configure/sleep.yaml` becomes, built as the loader builds it: the
/// section decoded by its owner's type, resolved against `classes`, encoded by the pack.
pub fn configured(section: &str, classes: &str) -> Emission {
    let section: Section<SleepSystem> =
        Section::deserialize(&mut serde_json::Deserializer::from_str(section))
            .expect("the test's section decodes");
    let classes = serde_json::from_str(classes).expect("the test's classes decode");
    let resolved = interactions::resolve(&section, &classes).expect("resolves");
    Emission::new::<SectionConfigured<SleepSystem>>(
        <SleepSystem as InteractionSection>::encode(&resolved),
        Visibility::SystemInternal,
    )
}

/// One test world.
pub struct Town {
    pub kind: Kind,
    pub world: World,
    pub providers: Vec<Box<dyn PerceptionProvider>>,
    pub places: BTreeMap<&'static str, PlaceId>,
    pub people: BTreeMap<&'static str, EntityId>,
    /// Every fact recorded, genesis first, in log order.
    pub log: Vec<EventEnvelope>,
    clock: i64,
    next_action: u64,
}

pub fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("a payload encodes")
}

pub fn record<A: Action + Serialize>(action: &A) -> ActionRecord {
    ActionRecord::new::<A>(encode(action))
}

fn at(place: PlaceId, x: i32, y: i32) -> Location {
    Location::in_place(place).with_local(local(x, y))
}

fn local(x: i32, y: i32) -> LocalPosition {
    LocalPosition::on_ground(Millimetres::new(x), Millimetres::new(y))
}

/// The layout of a world: three places in a row, each joined to the next by a doorway, and three
/// people with their tags, place and position.
struct Layout {
    places: [(&'static str, &'static [&'static str]); 3],
    people: [Resident; 3],
}

/// A person of a layout: key, tags, index of their place, position there.
type Resident = (&'static str, &'static [&'static str], usize, (i32, i32));

fn layout(kind: Kind) -> Layout {
    match kind {
        Kind::FlatAndPark => Layout {
            places: [
                ("flat", &["residential"]),
                ("street", &["street"]),
                ("park", &["park"]),
            ],
            people: [
                ("ana", &[], 0, (1_000, 1_000)),
                ("ben", &[], 0, (2_000, 1_000)),
                ("cy", &[], 1, (3_000, 2_000)),
            ],
        },
        Kind::Hostel => Layout {
            places: [
                ("dorm", &["dormitory"]),
                ("hall", &["hall"]),
                ("garden", &["garden"]),
            ],
            people: [
                ("ana", &[], 0, (1_000, 1_000)),
                ("ben", &["traveller"], 0, (2_000, 1_000)),
                ("cy", &[], 1, (3_000, 2_000)),
            ],
        },
    }
}

impl Town {
    /// The world, with the sleep pack installed.
    pub fn new(kind: Kind) -> Self {
        Self::build(kind, true, None)
    }

    /// flat-and-park configured by `section` with `classes` (JSON renderings of the two files).
    pub fn configured(section: &str, classes: &str) -> Self {
        Self::build(Kind::FlatAndPark, true, Some(configured(section, classes)))
    }

    /// The world without the sleep pack (and so without its section).
    pub fn without_sleep(kind: Kind) -> Self {
        Self::build(kind, false, None)
    }

    fn build(kind: Kind, sleep: bool, section: Option<Emission>) -> Self {
        let Assembled {
            mut world,
            providers,
            places,
            people,
            facts,
        } = assemble(kind, sleep, section);
        let log = world.genesis(GENESIS, facts).expect("begins");
        Self {
            kind,
            world,
            providers,
            places,
            people,
            log,
            clock: GENESIS.seconds(),
            next_action: 1,
        }
    }
}

/// The packs of a test world, installed in the order a World Pack lists them.
pub fn compose(sleep: bool) -> (World, Vec<Box<dyn PerceptionProvider>>) {
    let mut world = World::new();
    let mut providers: Vec<Box<dyn PerceptionProvider>> = Vec::new();
    world.install(PresenceSystem).expect("presence");
    providers.push(Box::new(PresenceSystem));
    world.install(MovementSystem).expect("movement");
    providers.push(Box::new(MovementSystem));
    world.install(ConversationSystem).expect("conversation");
    providers.push(Box::new(ConversationSystem));
    if sleep {
        world
            .install(SleepSystem)
            .expect("sleep installs after presence");
        providers.push(Box::new(SleepSystem));
    }
    (world, providers)
}

/// A test world composed, its entities created, and its genesis facts — not yet stated.
pub struct Assembled {
    pub world: World,
    pub providers: Vec<Box<dyn PerceptionProvider>>,
    pub places: BTreeMap<&'static str, PlaceId>,
    pub people: BTreeMap<&'static str, EntityId>,
    pub facts: Vec<Emission>,
}

pub fn assemble(kind: Kind, sleep: bool, section: Option<Emission>) -> Assembled {
    let (mut world, providers) = compose(sleep);
    let shape = layout(kind);
    let mut create = |key: &str, entity_type, tags: &[&str]| {
        world
            .create_authored_entity(
                EntityKey::new(key).expect("a key"),
                entity_type,
                Tags::new(tags.iter().map(|tag| Tag::new(*tag).expect("a tag"))),
                None,
            )
            .expect("created")
    };
    let mut places = BTreeMap::new();
    let mut order = Vec::new();
    for (key, tags) in shape.places {
        let place =
            PlaceId::new(create(key, EntityType::Place, tags), EntityType::Place).expect("a place");
        places.insert(key, place);
        order.push(place);
    }
    let mut people = BTreeMap::new();
    let mut where_ = Vec::new();
    for (key, tags, place, spot) in shape.people {
        let person = create(key, EntityType::Person, tags);
        people.insert(key, person);
        where_.push((person, order[place], spot));
    }

    let mut facts = vec![
        passage(
            order[0],
            Some(local(5_000, 2_000)),
            order[1],
            Some(local(0, 2_000)),
        ),
        passage(
            order[1],
            Some(local(10_000, 2_000)),
            order[2],
            Some(local(0, 2_000)),
        ),
    ];
    for (person, place, (x, y)) in where_ {
        let person = PersonId::new(person, EntityType::Person).expect("a person");
        facts.push(arrival(&world.read(), person, at(place, x, y)).expect("admitted"));
    }
    if sleep && kind == Kind::Hostel {
        facts.push(configured(HOSTEL_SECTION, HOSTEL_CLASSES));
    }
    facts.extend(section);
    Assembled {
        world,
        providers,
        places,
        people,
        facts,
    }
}

impl Town {
    pub fn person(&self, name: &str) -> EntityId {
        self.people[name]
    }

    pub fn place(&self, name: &str) -> PlaceId {
        self.places[name]
    }

    /// The instant of the last request, or of the last [`pass`](Self::pass).
    pub fn now(&self) -> WorldTime {
        WorldTime::from_seconds(self.clock)
    }

    /// Lets `seconds` pass with no request, firing whatever falls due; the facts it caused.
    pub fn pass(&mut self, seconds: i64) -> Vec<EventEnvelope> {
        self.clock += seconds;
        let facts = self
            .world
            .advance_to(WorldTime::from_seconds(self.clock))
            .expect("advances")
            .into_events();
        self.log.extend(facts.iter().cloned());
        facts
    }

    /// Advances to exactly `at`, firing whatever falls due; the facts it caused.
    pub fn pass_to(&mut self, at: WorldTime) -> Vec<EventEnvelope> {
        self.pass(at.seconds() - self.clock)
    }

    /// Submits a request one second after the last; the facts due before it are fired first and
    /// returned with the request's own.
    pub fn submit(
        &mut self,
        actor: EntityId,
        target: Option<EntityId>,
        record: ActionRecord,
    ) -> (ActionResult, Vec<EventEnvelope>) {
        let mut facts = self.pass(1);
        let (result, own) = self.submit_now(actor, target, record);
        facts.extend(own);
        (result, facts)
    }

    /// Submits a request at the current instant, firing nothing first.
    pub fn submit_now(
        &mut self,
        actor: EntityId,
        target: Option<EntityId>,
        record: ActionRecord,
    ) -> (ActionResult, Vec<EventEnvelope>) {
        let when = self.now();
        let id = ActionId::from_raw(self.next_action);
        self.next_action += 1;
        let mut intent = ActionIntent::new(id, actor, record, when);
        if let Some(target) = target {
            intent = intent.with_target(target);
        }
        let dispatched = self.world.dispatch(&intent, when).expect("answers");
        self.log.extend(dispatched.events().iter().cloned());
        (dispatched.result().clone(), dispatched.events().to_vec())
    }

    pub fn sleep(&mut self, who: &str, minutes: u16) -> (ActionResult, Vec<EventEnvelope>) {
        let actor = self.person(who);
        self.submit(actor, None, record(&Sleep::new(minutes)))
    }

    pub fn wake(&mut self, who: &str) -> (ActionResult, Vec<EventEnvelope>) {
        let actor = self.person(who);
        self.submit(actor, None, record(&Wake {}))
    }

    pub fn talk(&mut self, from: &str, to: &str) -> (ActionResult, Vec<EventEnvelope>) {
        let (from, to) = (self.person(from), self.person(to));
        let said = Utterance::new("Hello.").expect("an utterance");
        self.submit(from, Some(to), record(&Talk::new(said)))
    }

    /// Asks `who` to walk into `place` and steps until the walk ends; every fact on the way.
    pub fn walk(&mut self, who: &str, place: &str) -> (ActionResult, Vec<EventEnvelope>) {
        let actor = self.person(who);
        let to = Destination::Place(Location::in_place(self.place(place)));
        let (result, mut facts) = self.submit(actor, None, record(&WalkTo::new(to)));
        if !matches!(result, ActionResult::Accepted { .. }) {
            return (result, facts);
        }
        for _ in 0..40 {
            let (_, events) = self.submit(actor, None, record(&WalkStep::default()));
            let ended = events
                .iter()
                .any(|fact| *fact.event_type() == WalkEnded::EVENT_TYPE);
            facts.extend(events);
            if ended {
                return (result, facts);
            }
        }
        panic!("the walk did not end in 40 steps");
    }

    pub fn asleep(&self, who: &str) -> Option<Asleep> {
        self.world
            .read()
            .component::<Asleep>(self.person(who))
            .copied()
    }

    /// How many `sleep` processes are running.
    pub fn sleeping_processes(&self) -> usize {
        self.world
            .read()
            .processes()
            .filter(|process| {
                process.process_type().as_str()
                    == <SleepProcess as mineworld_kernel::ProcessKind>::PROCESS_TYPE.as_str()
            })
            .count()
    }

    pub fn place_of(&self, who: &str) -> Option<PlaceId> {
        self.world
            .read()
            .component::<Presence>(self.person(who))
            .map(|presence| presence.location().place())
    }

    pub fn observation(&self, who: &str) -> Observation<serde_json::Value> {
        let providers: Vec<&dyn PerceptionProvider> =
            self.providers.iter().map(AsRef::as_ref).collect();
        mineworld_presence::observe(&self.world, self.person(who), self.now(), &providers)
    }
}

impl Town {
    /// The types of the facts `who` perceived, over the whole log, as presence decides it.
    pub fn perceived(&self, who: &str) -> Vec<String> {
        mineworld_presence::audience::perceived_by(self.log.iter(), self.person(who), None)
            .map(|fact| fact.event_type().as_str().to_owned())
            .collect()
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

pub fn kinds(facts: &[EventEnvelope]) -> Vec<&str> {
    facts
        .iter()
        .map(|fact| fact.event_type().as_str())
        .collect()
}

pub fn accepted(result: &ActionResult) -> bool {
    matches!(result, ActionResult::Accepted { .. })
}
