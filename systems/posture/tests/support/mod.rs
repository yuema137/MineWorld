//! Scratch World Packs for posture's tests, assembled through the real loader with this pack in the
//! installed set, so sit points are read from a place file and the list from `configure/posture.yaml`.
//!
//! Two worlds, so that nothing here passes because of one layout (the charter's "second world" rule):
//!
//! ```text
//! cafe-world   cafe     bench.1 (2 000, 1 000) facing 180 000, bench.2 (2 600, 1 000) facing 180 000,
//!                       stool.1 (6 000, 1 000) facing 0
//!                       alice (2 000, 1 500)  500 mm from bench.1
//!                       dave  (2 000,   400)  600 mm from bench.1
//!                       carol (2 600, 1 600)  600 mm from bench.2
//!                       bob   (4 000, 1 000)  1 400 mm from bench.2, out of reach of every point
//!                       eve   (6 000, 1 300)  standing 300 mm from stool.1
//!                       frank (6 000, 1 700)  700 mm from stool.1, within reach
//!                       the doorway at (2 000, 2 500), onto the street's (500, 500)
//!              street   no sit points
//! pier-world   pier     pier-bench.1 (3 000, 3 000) facing 90 000; a 10 m floor (bodies)
//!                       gus (3 600, 3 000) 600 mm from it; hal (3 000, 3 900)
//! ```
//!
//! Every number is a literal from this layout, never derived from a constant under test (`ARC-23`).

#![allow(dead_code)]

use std::collections::BTreeMap;

use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionResult, Affordance, EntityId, EntityKey,
    EventEnvelope, LocalPosition, Location, Millimetres, Observation, PlaceId, WorldTime,
};
use mineworld_kernel::World;
use mineworld_posture::{Posture, Sit, SitPointKey, Stand};
use mineworld_presence::{PerceptionProvider, Presence};
use mineworld_test_support::Scratch;
use mineworld_worldpack::{AssembledWorld, WorldPack};
use serde::Serialize;
use serde_json::Value;

/// The instant genesis happens at; every request is an offset from it.
pub const GENESIS: WorldTime = WorldTime::from_seconds(3_600);

pub fn t(offset: i64) -> WorldTime {
    WorldTime::from_seconds(GENESIS.seconds() + offset)
}

pub fn record<A: Action + Serialize>(action: &A) -> ActionRecord {
    ActionRecord::new::<A>(serde_json::to_vec(action).expect("a payload encodes"))
}

pub fn key(text: &str) -> SitPointKey {
    SitPointKey::new(text).expect("a valid key")
}

pub fn sit(point: &str) -> ActionRecord {
    record(&Sit::new(key(point)))
}

pub fn stand() -> ActionRecord {
    record(&Stand {})
}

pub fn at(place: PlaceId, (x, y): (i32, i32)) -> Location {
    Location::in_place(place).with_local(LocalPosition::on_ground(
        Millimetres::new(x),
        Millimetres::new(y),
    ))
}

const CAFE_PLACE: &str = "\
tags: [cafe]
passages:
  - to: street
    here: { x: 2000, y: 2500 }
    there: { x: 500, y: 500 }
sit-points:
  - { key: bench.1, group: bench, at: { x: 2000, y: 1000 }, facing: 180000 }
  - { key: bench.2, group: bench, at: { x: 2600, y: 1000 }, facing: 180000 }
  - { key: stool.1, at: { x: 6000, y: 1000 }, facing: 0 }
";

const CAFE_PEOPLE: &[(&str, &str, i32, i32)] = &[
    ("alice", "cafe", 2_000, 1_500),
    ("bob", "cafe", 4_000, 1_000),
    ("carol", "cafe", 2_600, 1_600),
    ("dave", "cafe", 2_000, 400),
    ("eve", "cafe", 6_000, 1_300),
    ("frank", "cafe", 6_000, 1_700),
];

const PIER_PLACE: &str = "\
tags: [pier]
body:
  floor:
    min: { x: 0, y: 0 }
    max: { x: 10000, y: 10000 }
sit-points:
  - { key: pier-bench.1, at: { x: 3000, y: 3000 }, facing: 90000 }
";

const PIER_PEOPLE: &[(&str, &str, i32, i32)] =
    &[("gus", "pier", 3_600, 3_000), ("hal", "pier", 3_000, 3_900)];

/// A scratch World Pack's text: its places, people, systems and `configure/posture.yaml`.
pub struct Layout {
    pub systems: Vec<&'static str>,
    pub places: Vec<(&'static str, &'static str)>,
    pub people: &'static [(&'static str, &'static str, i32, i32)],
    pub posture: Option<String>,
}

impl Layout {
    /// The café and the street, with presence, movement and posture.
    pub fn cafe() -> Self {
        Self {
            systems: vec!["presence", "movement", "posture"],
            places: vec![("cafe", CAFE_PLACE), ("street", "tags: [street]\n")],
            people: CAFE_PEOPLE,
            posture: None,
        }
    }

    /// The pier, with bodies too.
    pub fn pier() -> Self {
        Self {
            systems: vec!["presence", "movement", "bodies", "posture"],
            places: vec![("pier", PIER_PLACE)],
            people: PIER_PEOPLE,
            posture: None,
        }
    }

    /// This layout with `configure/posture.yaml` as given.
    pub fn configured(mut self, text: &str) -> Self {
        self.posture = Some(text.to_owned());
        self
    }

    /// This layout without the posture pack, and its sit points left out of the place files (a world
    /// that does not enable posture may not carry its section).
    pub fn without_posture(mut self) -> Self {
        self.systems.retain(|system| *system != "posture");
        self.places = self
            .places
            .iter()
            .map(|(name, text)| {
                let kept: &'static str = Box::leak(
                    text.split("sit-points:")
                        .next()
                        .expect("a prefix")
                        .to_owned()
                        .into_boxed_str(),
                );
                (*name, kept)
            })
            .collect();
        self
    }

    /// Writes the pack into a fresh scratch directory named `name`.
    pub fn write(&self, name: &str) -> Scratch {
        let scratch = mineworld_test_support::scratch!(empty name);
        let root = scratch.path();
        for directory in ["people", "places", "configure"] {
            std::fs::create_dir_all(root.join(directory)).expect("writable");
        }
        let write =
            |file: &str, text: &str| std::fs::write(root.join(file), text).expect("writable");
        let list = |items: &mut dyn Iterator<Item = &str>| -> String {
            items.map(|item| format!("  - {item}\n")).collect()
        };
        let configure = if self.posture.is_some() {
            "configure:\n  - posture\n".to_owned()
        } else {
            String::new()
        };
        write(
            "world.yaml",
            &format!(
                "world:\n  id: {}\n  name: A Place To Sit\nsystems:\n{}{configure}places:\n{}population:\n{}",
                // A pack's id is its directory's name, so each scratch world is named for its test.
                name,
                list(&mut self.systems.iter().copied()),
                list(&mut self.places.iter().map(|(name, _)| *name)),
                list(&mut self.people.iter().map(|(name, ..)| *name)),
            ),
        );
        for (name, text) in &self.places {
            write(&format!("places/{name}.yaml"), text);
        }
        for (name, place, x, y) in self.people {
            write(
                &format!("people/{name}.yaml"),
                &format!("location:\n  place: {place}\n  position: {{ x: {x}, y: {y} }}\n"),
            );
        }
        if let Some(text) = &self.posture {
            write("configure/posture.yaml", text);
        }
        scratch
    }

    /// The pack, read and assembled, not yet begun; the scratch directory stays alive with it.
    pub fn assemble(&self, name: &str) -> (Scratch, AssembledWorld) {
        let scratch = self.write(name);
        let assembled = WorldPack::read(scratch.path())
            .and_then(|pack| pack.assemble())
            .unwrap_or_else(|error| panic!("the world assembles: {error}"));
        (scratch, assembled)
    }

    /// Why the loader refuses this pack.
    pub fn refusal(&self, name: &str) -> String {
        let scratch = self.write(name);
        WorldPack::read(scratch.path())
            .and_then(|pack| pack.assemble().map(|_| ()))
            .expect_err("refused")
            .to_string()
    }
}

/// A begun world, its keys, and the packs that answer perception.
pub struct Town {
    _scratch: Option<Scratch>,
    pub world: World,
    pub ids: BTreeMap<EntityKey, EntityId>,
    pub providers: Vec<Box<dyn PerceptionProvider>>,
    next_action: u64,
}

impl Town {
    pub fn begin(layout: &Layout, name: &str) -> Self {
        let (scratch, assembled) = layout.assemble(name);
        let AssembledWorld {
            mut world,
            ids,
            facts,
            providers,
        } = assembled;
        world.genesis(GENESIS, facts).expect("the world begins");
        Self {
            _scratch: Some(scratch),
            world,
            ids,
            providers,
            next_action: 1,
        }
    }

    /// A shipped World Pack, read from `path` and begun.
    pub fn open(path: &std::path::Path) -> Self {
        let AssembledWorld {
            mut world,
            ids,
            facts,
            providers,
        } = WorldPack::read(path)
            .and_then(|pack| pack.assemble())
            .unwrap_or_else(|error| panic!("{} assembles: {error}", path.display()));
        world.genesis(GENESIS, facts).expect("the world begins");
        Self {
            _scratch: None,
            world,
            ids,
            providers,
            next_action: 1,
        }
    }

    pub fn id(&self, key: &str) -> EntityId {
        *self
            .ids
            .get(&EntityKey::new(key).expect("a key"))
            .unwrap_or_else(|| panic!("{key} is in the world"))
    }

    pub fn place(&self, key: &str) -> PlaceId {
        PlaceId::new(self.id(key), mineworld_contracts::EntityType::Place).expect("a place")
    }

    /// Advances to `t(when)`, then submits `record` as `actor`, directed at nobody, as a server would.
    pub fn submit(
        &mut self,
        when: i64,
        actor: &str,
        record: ActionRecord,
    ) -> (ActionResult, Vec<EventEnvelope>) {
        self.submit_to(when, actor, None, record)
    }

    pub fn submit_to(
        &mut self,
        when: i64,
        actor: &str,
        target: Option<&str>,
        record: ActionRecord,
    ) -> (ActionResult, Vec<EventEnvelope>) {
        let when = t(when);
        let _due = self.world.advance_to(when).expect("the clock advances");
        let id = ActionId::from_raw(self.next_action);
        self.next_action += 1;
        let mut intent = ActionIntent::new(id, self.id(actor), record, when);
        if let Some(target) = target {
            intent = intent.with_target(self.id(target));
        }
        let dispatched = self
            .world
            .dispatch(&intent, when)
            .expect("dispatch answers rather than failing");
        (dispatched.result().clone(), dispatched.events().to_vec())
    }

    pub fn posture(&self, person: &str) -> Option<Posture> {
        self.world
            .read()
            .component::<Posture>(self.id(person))
            .cloned()
    }

    pub fn location(&self, person: &str) -> Location {
        self.world
            .read()
            .component::<Presence>(self.id(person))
            .map(Presence::location)
            .expect("placed")
    }

    pub fn observation(&self, observer: &str, when: i64) -> Observation<Value> {
        let providers: Vec<&dyn PerceptionProvider> =
            self.providers.iter().map(AsRef::as_ref).collect();
        mineworld_presence::observe(&self.world, self.id(observer), t(when), &providers)
    }

    /// The observer's `sit` affordance for `point`, found by its complete payload.
    pub fn sit_offer(&self, observer: &str, point: &str, when: i64) -> Option<Affordance<Value>> {
        let wanted = serde_json::json!({ "point": point });
        self.observation(observer, when)
            .affordances()
            .iter()
            .find(|offer| offer.action_type().as_str() == "sit" && offer.payload() == Some(&wanted))
            .cloned()
    }
}

pub fn accepted(result: &ActionResult) -> bool {
    matches!(result, ActionResult::Accepted { .. })
}

/// The event types of a list of facts, in order.
pub fn types(facts: &[EventEnvelope]) -> Vec<String> {
    facts
        .iter()
        .map(|fact| fact.event_type().as_str().to_owned())
        .collect()
}

/// The `stood-up` fact's cause, if the list holds one.
pub fn stood_up_cause(facts: &[EventEnvelope]) -> Option<String> {
    facts
        .iter()
        .find(|fact| fact.event_type().as_str() == "stood-up")
        .map(|fact| {
            let value: Value =
                serde_json::from_slice(fact.payload().payload()).expect("a JSON payload");
            value["cause"].as_str().expect("a cause").to_owned()
        })
}
