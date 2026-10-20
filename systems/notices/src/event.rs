//! The facts this pack records. Only this pack states them, and only its reductions turn them into
//! state (`I-S31-2`).
//!
//! ```text
//! notices-configured     genesis: the topic table          SystemInternal, about nobody
//! subscriptions-seeded   genesis: a person's first set     SystemInternal, about the person (Q-S31b-2)
//! subscribed             a person adds a topic             to that person only
//! unsubscribed           a person drops a topic            to that person only
//! notice-published       a publisher posts a notice        to the topic's subscribers then, and the
//!                                                          publisher
//! ```

use std::collections::BTreeSet;

use mineworld_contracts::{
    EntityId, Event, EventSchemaVersion, EventTypeId, PersonId, SystemId, Visibility,
};
use mineworld_kernel::{Emission, SystemIdentity};
use serde::{Deserialize, Serialize};

use crate::board::Notice;
use crate::codec;
use crate::system::NoticesSystem;
use crate::topic::{BodyKind, Topic, TopicId};

/// One topic as the world configured it, with durations in seconds and publishers resolved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfiguredTopic {
    topic: TopicId,
    body: BodyKind,
    publishers: Vec<PersonId>,
    valid_min: u32,
    valid_max: u32,
    per_day: u8,
}

impl ConfiguredTopic {
    pub(crate) fn new(topic: TopicId, declared: &Topic, publishers: Vec<PersonId>) -> Self {
        Self {
            topic,
            body: declared.body(),
            publishers,
            valid_min: declared.valid_for().min(),
            valid_max: declared.valid_for().max(),
            per_day: declared.per_day(),
        }
    }

    /// The topic.
    pub const fn topic(&self) -> &TopicId {
        &self.topic
    }

    /// What its notices carry.
    pub const fn body(&self) -> BodyKind {
        self.body
    }

    /// Who may post on it, in id order.
    pub fn publishers(&self) -> &[PersonId] {
        &self.publishers
    }

    /// The shortest validity, in seconds.
    pub const fn valid_min(&self) -> u32 {
        self.valid_min
    }

    /// The longest validity, in seconds.
    pub const fn valid_max(&self) -> u32 {
        self.valid_max
    }

    /// The rolling limit per 24 hours.
    pub const fn per_day(&self) -> u8 {
        self.per_day
    }
}

/// The world's topics, as configured.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoticesConfigured {
    topics: Vec<ConfiguredTopic>,
}

impl Event for NoticesConfigured {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("notices-configured");
    const OWNER: SystemId = NoticesSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl NoticesConfigured {
    pub(crate) const fn new(topics: Vec<ConfiguredTopic>) -> Self {
        Self { topics }
    }

    /// The topics, in id order.
    pub fn topics(&self) -> &[ConfiguredTopic] {
        &self.topics
    }

    /// SystemInternal, about nobody (`ARC-61` item 8).
    pub(crate) fn emission(&self) -> Emission {
        Emission::new::<Self>(codec::encode(self), Visibility::SystemInternal)
    }
}

/// Where a person's first subscriptions came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    /// The configuration's `defaults.by_tag`: inserted only when the person has no set yet.
    Default,
    /// The person's own `notices:` section: replaces whatever is there.
    Authored,
}

/// A person's subscriptions as the world begins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionsSeeded {
    person: PersonId,
    topics: Vec<TopicId>,
    origin: Origin,
}

impl Event for SubscriptionsSeeded {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("subscriptions-seeded");
    const OWNER: SystemId = NoticesSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl SubscriptionsSeeded {
    /// `topics` are sorted and deduplicated here, so the fact has one form.
    pub(crate) fn new(person: PersonId, topics: Vec<TopicId>, origin: Origin) -> Self {
        let topics: BTreeSet<TopicId> = topics.into_iter().collect();
        Self {
            person,
            topics: topics.into_iter().collect(),
            origin,
        }
    }

    /// Whose.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// The topics, sorted.
    pub fn topics(&self) -> &[TopicId] {
        &self.topics
    }

    /// Where they came from.
    pub const fn origin(&self) -> Origin {
        self.origin
    }

    /// SystemInternal (Q-S31b-2): nobody's inbox or memory starts with a fact nobody witnessed. The
    /// holder sees its set through the `subscriptions` record.
    pub(crate) fn emission(&self) -> Emission {
        Emission::new::<Self>(codec::encode(self), Visibility::SystemInternal)
            .about(vec![self.person.entity_id()])
    }
}

/// A person subscribed to a topic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subscribed {
    person: PersonId,
    topic: TopicId,
}

impl Event for Subscribed {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("subscribed");
    const OWNER: SystemId = NoticesSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl Subscribed {
    pub(crate) const fn new(person: PersonId, topic: TopicId) -> Self {
        Self { person, topic }
    }

    /// Who.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// To what.
    pub const fn topic(&self) -> &TopicId {
        &self.topic
    }

    /// To that person only.
    pub(crate) fn emission(&self) -> Emission {
        private::<Self>(codec::encode(self), self.person.entity_id())
    }
}

/// A person unsubscribed from a topic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unsubscribed {
    person: PersonId,
    topic: TopicId,
}

impl Event for Unsubscribed {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("unsubscribed");
    const OWNER: SystemId = NoticesSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl Unsubscribed {
    pub(crate) const fn new(person: PersonId, topic: TopicId) -> Self {
        Self { person, topic }
    }

    /// Who.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// From what.
    pub const fn topic(&self) -> &TopicId {
        &self.topic
    }

    /// To that person only.
    pub(crate) fn emission(&self) -> Emission {
        private::<Self>(codec::encode(self), self.person.entity_id())
    }
}

/// A fact visible to `person` alone, about them.
fn private<E: Event>(payload: Vec<u8>, person: EntityId) -> Emission {
    Emission::new::<E>(payload, Visibility::Entities(BTreeSet::from([person])))
        .about(vec![person])
        .with_participants(vec![person])
}

/// A publisher posted a notice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoticePublished {
    notice: Notice,
}

impl Event for NoticePublished {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("notice-published");
    const OWNER: SystemId = NoticesSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl NoticePublished {
    pub(crate) const fn new(notice: Notice) -> Self {
        Self { notice }
    }

    /// The notice.
    pub const fn notice(&self) -> &Notice {
        &self.notice
    }

    /// To exactly `audience` — the topic's subscribers at this instant — and the publisher, about the
    /// publisher (`ARC-115` item 4).
    pub(crate) fn emission(&self, audience: BTreeSet<EntityId>) -> Emission {
        let publisher = self.notice.publisher().entity_id();
        let mut audience = audience;
        audience.insert(publisher);
        Emission::new::<Self>(codec::encode(self), Visibility::Entities(audience))
            .about(vec![publisher])
            .with_participants(vec![publisher])
    }
}
