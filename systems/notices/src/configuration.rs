//! `configure/notices.yaml`, decoded by this pack's own types (`ARC-61`): the world's topics, and which
//! topics people with a tag subscribe to by default.
//!
//! ```yaml
//! topics:                              # 1 … 32 topics, keyed by dotted id
//!   prices.ingredients:
//!     body: prices                     # prices | text
//!     publishers: [carol]              # 1 … 16 person keys
//!     valid_for: { min: "1h", max: "48h" }
//!     per_day: 4                       # in any 24 h of world time
//! defaults:                            # optional
//!   by_tag: { barista: [prices.ingredients] }
//! ```

use std::collections::{BTreeMap, BTreeSet};

use mineworld_authoring::{
    ConfigurationContext, DataDirectory, PackConfiguration, Reference, Seeding, YieldedFile,
};
use mineworld_contracts::{EntityType, EventTypeId, LifecycleState, PersonId, Rejection, Tag};
use mineworld_kernel::Emission;
use serde::Deserialize;

use crate::event::{ConfiguredTopic, NoticesConfigured, Origin, SubscriptionsSeeded};
use crate::system::NoticesSystem;
use crate::topic::{Topic, TopicId};

/// The most topics one world may declare. Because a person subscribes only to declared topics, it is
/// also the most subscriptions one person can hold (F-S31b-7).
pub const TOPICS_MAX: usize = 32;

/// The `attributes` pack's vocabulary directory, which this configuration yields one file into.
const VOCABULARIES: DataDirectory = DataDirectory::from_static("vocabularies");

/// A world's public-notice topics, and the default subscriptions by tag.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "RawNoticesConfiguration")]
#[schemars(with = "RawNoticesConfiguration")]
pub struct NoticesConfiguration {
    topics: BTreeMap<TopicId, Topic>,
    defaults: BTreeMap<Tag, Vec<TopicId>>,
}

/// `configure/notices.yaml` as written, before the cross-field rules are checked.
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(example = serde_json::json!({
    "topics": {
        "news.daily": {
            "body": "text", "publishers": ["hana"],
            "valid_for": { "min": "6h", "max": "7d" }, "per_day": 1
        }
    },
    "defaults": { "by_tag": { "staff": ["news.daily"] } }
}))]
pub struct RawNoticesConfiguration {
    /// The world's topics, 1 to 32, each keyed by its dotted id such as prices.ingredients.
    topics: BTreeMap<TopicId, Topic>,
    /// Default subscriptions. A person whose own file has no `notices:` section subscribes to the
    /// union of the topics listed for its tags.
    #[serde(default)]
    defaults: RawDefaults,
}

/// `defaults:` as written.
#[derive(Debug, Clone, Default, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawDefaults {
    /// A person tag, then the declared topics a person with that tag subscribes to, none twice.
    #[serde(default)]
    #[schemars(with = "BTreeMap<mineworld_authoring::schema::TagName, Vec<TopicId>>")]
    by_tag: BTreeMap<Tag, Vec<TopicId>>,
}

impl TryFrom<RawNoticesConfiguration> for NoticesConfiguration {
    type Error = String;

    fn try_from(raw: RawNoticesConfiguration) -> Result<Self, Self::Error> {
        if raw.topics.is_empty() || raw.topics.len() > TOPICS_MAX {
            return Err(format!(
                "topics declares 1 to {TOPICS_MAX} topics, not {}",
                raw.topics.len()
            ));
        }
        for (tag, topics) in &raw.defaults.by_tag {
            let mut seen = BTreeSet::new();
            for topic in topics {
                if !raw.topics.contains_key(topic) {
                    return Err(format!("defaults.by_tag.{tag}: unknown topic `{topic}`"));
                }
                if !seen.insert(topic) {
                    return Err(format!("defaults.by_tag.{tag}: `{topic}` is listed twice"));
                }
            }
        }
        Ok(Self {
            topics: raw.topics,
            defaults: raw.defaults.by_tag,
        })
    }
}

impl NoticesConfiguration {
    /// The topics, in id order.
    pub const fn topics(&self) -> &BTreeMap<TopicId, Topic> {
        &self.topics
    }

    /// The default subscriptions, by tag.
    pub const fn defaults(&self) -> &BTreeMap<Tag, Vec<TopicId>> {
        &self.defaults
    }
}

impl PackConfiguration for NoticesSystem {
    type Configuration = NoticesConfiguration;
    const FACTS: &'static [EventTypeId] = &[
        <NoticesConfigured as mineworld_contracts::Event>::EVENT_TYPE,
        <SubscriptionsSeeded as mineworld_contracts::Event>::EVENT_TYPE,
    ];

    /// The vocabulary `notices.topics` (S31-b2 §5.2, SD-b2-1): `vocabularies/topics.yaml` listing this
    /// world's topic keys, sorted. Collected only where the world enables `attributes`, the owner of
    /// `vocabularies/`; elsewhere it is skipped and nothing changes.
    fn yields(configuration: &NoticesConfiguration) -> Vec<YieldedFile> {
        let topics: Vec<&str> = configuration.topics.keys().map(TopicId::as_str).collect();
        vec![YieldedFile {
            directory: VOCABULARIES,
            name: "topics.yaml".to_owned(),
            text: format!(
                "format: 1\nvocabularies:\n  topics: [{}]\n",
                topics.join(", ")
            ),
        }]
    }

    /// Every publisher is a Person the world declares; the loader refuses any other by name.
    fn references(configuration: &NoticesConfiguration) -> Vec<Reference<'_>> {
        configuration
            .topics
            .values()
            .flat_map(Topic::publishers)
            .map(|key| Reference {
                key,
                entity_type: EntityType::Person,
            })
            .collect()
    }

    /// In order: one `notices-configured`, SystemInternal and about nobody (`ARC-61` item 8); then,
    /// for every living Person in `EntityId` order whose tags meet `defaults.by_tag`, one
    /// `subscriptions-seeded { origin: default }` with the sorted union, SystemInternal (Q-S31b-2).
    fn seed(
        seeding: &Seeding<'_, '_>,
        configuration: &NoticesConfiguration,
        _: &ConfigurationContext<'_>,
    ) -> Result<Vec<Emission>, Rejection> {
        let mut topics = Vec::with_capacity(configuration.topics.len());
        for (id, topic) in &configuration.topics {
            let mut publishers = Vec::with_capacity(topic.publishers().len());
            for key in topic.publishers() {
                let entity = seeding
                    .resolve(key, EntityType::Person)
                    .ok_or(Rejection::PreconditionFailed)?;
                publishers.push(
                    PersonId::new(entity, EntityType::Person)
                        .map_err(|_| Rejection::PreconditionFailed)?,
                );
            }
            publishers.sort();
            topics.push(ConfiguredTopic::new(id.clone(), topic, publishers));
        }
        let mut seeded = vec![NoticesConfigured::new(topics).emission()];
        if configuration.defaults.is_empty() {
            return Ok(seeded);
        }
        for entity in seeding.world().entities() {
            if entity.entity_type() != EntityType::Person
                || entity.lifecycle() == LifecycleState::Destroyed
            {
                continue;
            }
            let union: BTreeSet<&TopicId> = configuration
                .defaults
                .iter()
                .filter(|(tag, _)| entity.tags().contains(tag))
                .flat_map(|(_, topics)| topics)
                .collect();
            if union.is_empty() {
                continue;
            }
            let person = PersonId::new(entity.id(), EntityType::Person)
                .map_err(|_| Rejection::PreconditionFailed)?;
            seeded.push(
                SubscriptionsSeeded::new(
                    person,
                    union.into_iter().cloned().collect(),
                    Origin::Default,
                )
                .emission(),
            );
        }
        Ok(seeded)
    }
}
