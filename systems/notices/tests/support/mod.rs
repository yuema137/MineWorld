//! A café composed from the real packs — presence and notices — with the fixture of
//! `pr-s31b-notices.md` §8. The configuration and the sections are decoded and seeded exactly as the
//! World Pack loader does it: the configuration first, then each person's section (`ARC-61`).
//!
//! ```text
//! news.daily           text    publisher pia   valid 1 h … 7 d    per_day 2
//! prices.ingredients   prices  publisher pia   valid 1 h … 48 h   per_day 4
//! pia     the publisher, subscribed to nothing
//! sam     subscribed to both
//! sue     subscribed to both
//! nat     subscribed to nothing
//! bread, milk   two item kinds; cafe   the place everybody stands in
//! ```

#![allow(dead_code)]

use std::collections::BTreeMap;

use mineworld_authoring::{
    Attached, AuthoredSection, ConfigurationContext, EntityClasses, Seeding,
};
use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, EntityId, EntityKey, EntityType, Event,
    EventEnvelope, LocalPosition, Location, Millimetres, Observation, PersonId, PlaceId, Tag, Tags,
    WorldTime,
};
use mineworld_kernel::{Dispatched, Emission, KernelError, World};
use mineworld_notices::{
    AuthoredNotices, Notice, NoticeBoardRecord, NoticeBody, NoticeText, NoticesSystem, PriceLine,
    PublishNotice, Subscribe, Subscriptions, TopicId, Unsubscribe,
};
use mineworld_presence::{PerceptionProvider, PresenceSystem, arrival};
use mineworld_sdk::SystemPack;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub const HOUR: u32 = 3_600;
pub const DAY: i64 = 86_400;

/// The fixture's configuration (§8).
pub const CONFIGURATION: &str = "\
topics:
  news.daily:
    body: text
    publishers: [pia]
    valid_for: { min: \"1h\", max: \"7d\" }
    per_day: 2
  prices.ingredients:
    body: prices
    publishers: [pia]
    valid_for: { min: \"1h\", max: \"48h\" }
    per_day: 4
";

/// The fixture's sections: Sam and Sue subscribe to both topics.
pub const SECTIONS: [(&str, &str); 2] = [
    ("sam", "subscribe: [news.daily, prices.ingredients]"),
    ("sue", "subscribe: [news.daily, prices.ingredients]"),
];

pub const PEOPLE: [&str; 4] = ["nat", "pia", "sam", "sue"];

pub fn t(seconds: i64) -> WorldTime {
    WorldTime::from_seconds(seconds)
}

pub fn key(name: &str) -> EntityKey {
    EntityKey::new(name).expect("a key")
}

pub fn topic(name: &str) -> TopicId {
    TopicId::new(name).expect("a topic")
}

pub fn text(words: &str) -> NoticeBody {
    NoticeBody::Text(NoticeText::new(words))
}

pub fn providers() -> [&'static dyn PerceptionProvider; 2] {
    [&PresenceSystem, &NoticesSystem]
}

/// How a café is assembled: its configuration text (or none), its sections, and its people's tags.
pub struct Recipe<'a> {
    pub configuration: Option<&'a str>,
    pub sections: Vec<(&'a str, &'a str)>,
    pub tags: Vec<(&'a str, &'a str)>,
}

impl Default for Recipe<'_> {
    fn default() -> Self {
        Self {
            configuration: Some(CONFIGURATION),
            sections: SECTIONS.to_vec(),
            tags: Vec::new(),
        }
    }
}

pub struct Cafe {
    pub world: World,
    pub keys: BTreeMap<EntityKey, EntityId>,
    pub genesis: Vec<EventEnvelope>,
    /// Every fact so far, genesis first: genesis, then each dispatch's facts in order.
    pub log: Vec<EventEnvelope>,
    next: u64,
}

/// Decodes the configuration as the loader does and seeds it.
fn seed_configuration(
    world: &World,
    keys: &BTreeMap<EntityKey, EntityId>,
    text: &str,
) -> Vec<Emission> {
    let decoded = serde_saphyr::with_deserializer_from_str(text, |file| {
        NoticesSystem::decode_configuration(file)
    })
    .expect("the configuration decodes");
    let read = world.read();
    let (classes, attached) = (EntityClasses::default(), Attached::none());
    decoded
        .seed(
            &Seeding::new(&read, keys),
            &ConfigurationContext::new(&classes, &attached),
        )
        .expect("seeds")
}

impl Cafe {
    /// The fixture café, begun at instant 0.
    pub fn begun() -> Self {
        Self::try_begin(&Recipe::default()).expect("genesis")
    }

    /// A café from `recipe`, begun at instant 0, or the refusal genesis reports.
    pub fn try_begin(recipe: &Recipe<'_>) -> Result<Self, KernelError> {
        let mut world = World::new();
        world.install(PresenceSystem).expect("presence");
        world
            .install(NoticesSystem)
            .expect("notices, with no dependency");
        let mut keys = BTreeMap::new();
        let place = world
            .create_entity(key("cafe"), EntityType::Place)
            .expect("created");
        keys.insert(key("cafe"), place);
        for name in PEOPLE {
            let tags: Tags = Tags::new(
                recipe
                    .tags
                    .iter()
                    .filter(|(person, _)| *person == name)
                    .map(|(_, tag)| Tag::new(*tag).expect("a tag")),
            );
            let id = world
                .create_authored_entity(key(name), EntityType::Person, tags, None)
                .expect("created");
            keys.insert(key(name), id);
        }
        for name in ["bread", "milk"] {
            let id = world
                .create_entity(key(name), EntityType::Item)
                .expect("created");
            keys.insert(key(name), id);
        }
        let cafe = PlaceId::new(place, EntityType::Place).expect("a place");
        let mut facts = Vec::new();
        for (index, name) in PEOPLE.iter().enumerate() {
            let person = PersonId::new(keys[&key(name)], EntityType::Person).expect("a person");
            let x = 1_000 * (i32::try_from(index).expect("small") + 1);
            let at = Location::in_place(cafe).with_local(LocalPosition::on_ground(
                Millimetres::new(x),
                Millimetres::new(1_000),
            ));
            facts.push(arrival(&world.read(), person, at).expect("admitted"));
        }
        if let Some(text) = recipe.configuration {
            facts.extend(seed_configuration(&world, &keys, text));
        }
        {
            let read = world.read();
            let seeding = Seeding::new(&read, &keys);
            for (person, yaml) in &recipe.sections {
                let authored: AuthoredNotices = serde_saphyr::from_str(yaml).expect("a section");
                facts.extend(
                    NoticesSystem::seed(&seeding, keys[&key(person)], &authored).expect("seeded"),
                );
            }
        }
        let genesis = world.genesis(t(0), facts)?;
        Ok(Self {
            world,
            keys,
            log: genesis.clone(),
            genesis,
            next: 1,
        })
    }

    pub fn id(&self, name: &str) -> EntityId {
        self.keys[&key(name)]
    }

    pub fn person(&self, name: &str) -> PersonId {
        PersonId::new(self.id(name), EntityType::Person).expect("a person")
    }

    /// A price line for item `name`.
    pub fn line(&self, name: &str, price: u64, basis: u16) -> PriceLine {
        PriceLine::new(self.id(name), price, basis)
    }

    /// Dispatches `action` by `actor` at second `at`, as the next request.
    pub fn act<A: Action + Serialize>(
        &mut self,
        actor: &str,
        action: &A,
        at: i64,
    ) -> Result<Dispatched, KernelError> {
        let record = ActionRecord::new::<A>(serde_json::to_vec(action).expect("encodes"));
        self.act_raw(actor, record, at)
    }

    /// Dispatches a request whose payload bytes are given as they are.
    pub fn act_raw(
        &mut self,
        actor: &str,
        record: ActionRecord,
        at: i64,
    ) -> Result<Dispatched, KernelError> {
        let intent =
            ActionIntent::new(ActionId::from_raw(self.next), self.id(actor), record, t(at));
        self.next += 1;
        let dispatched = self.world.dispatch(&intent, t(at))?;
        self.log.extend(dispatched.events().iter().cloned());
        Ok(dispatched)
    }

    pub fn publish(
        &mut self,
        actor: &str,
        on: &str,
        body: NoticeBody,
        valid_for: u32,
        at: i64,
    ) -> Result<Dispatched, KernelError> {
        self.act(actor, &PublishNotice::new(topic(on), body, valid_for), at)
    }

    pub fn subscribe(&mut self, actor: &str, on: &str, at: i64) -> Result<Dispatched, KernelError> {
        self.act(actor, &Subscribe::new(topic(on)), at)
    }

    pub fn unsubscribe(
        &mut self,
        actor: &str,
        on: &str,
        at: i64,
    ) -> Result<Dispatched, KernelError> {
        self.act(actor, &Unsubscribe::new(topic(on)), at)
    }

    /// The world, advanced to second `at` — any fact stated on the way is kept in [`Cafe::log`] —
    /// observed by `observer`.
    pub fn observation(&mut self, observer: &str, at: i64) -> Observation<Value> {
        let advanced = self.world.advance_to(t(at)).expect("advances");
        self.log.extend(advanced.events().iter().cloned());
        mineworld_presence::observe(&self.world, self.id(observer), t(at), &providers())
    }

    /// The `notice-board` record `observer` is shown at second `at`, if any.
    pub fn board(&mut self, observer: &str, at: i64) -> Vec<Notice> {
        let me = self.id(observer);
        let observation = self.observation(observer, at);
        record::<NoticeBoardRecord>(&observation, me, "notice-board")
            .map(|board| board.notices)
            .unwrap_or_default()
    }

    /// `name`'s subscriptions, read off the component.
    pub fn subscriptions(&self, name: &str) -> Vec<String> {
        self.world
            .read()
            .component::<Subscriptions>(self.id(name))
            .map(|held| {
                held.topics()
                    .iter()
                    .map(|t| t.as_str().to_owned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Every component and relation, and the processes, as bytes: "nothing moves" is compared on
    /// these.
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

/// Asserts that a dispatch was accepted, naming what was asked.
pub fn accepted(dispatched: Result<Dispatched, KernelError>, what: &str) {
    let dispatched = dispatched.unwrap_or_else(|error| panic!("{what}: {error:?}"));
    assert!(
        matches!(
            dispatched.result(),
            mineworld_contracts::ActionResult::Accepted { .. }
        ),
        "{what}: {:?}",
        dispatched.result()
    );
}
