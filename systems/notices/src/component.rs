//! The state this pack keeps on a person, and the two records it discloses.
//!
//! `subscriptions` is a person's component and is disclosed as itself, to that person only.
//! `notice-board` is a declared component type that no entity carries, like calendar's records: the
//! notices live in the board Process, and this is how the current ones are shown to one person.

use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::board::Notice;
use crate::system::NoticesSystem;
use crate::topic::{BodyKind, TopicId};

/// The topics a person subscribes to: sorted, none twice, at most 32 (the topics bound, F-S31b-7).
///
/// The operator's "attribute for subscribing to public information" (step-31 §0.1). Written only by
/// this pack's reductions of `subscriptions-seeded`, `subscribed` and `unsubscribed` (`I-S31-2`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subscriptions {
    topics: Vec<TopicId>,
}

impl Subscriptions {
    pub(crate) fn of(topics: impl IntoIterator<Item = TopicId>) -> Self {
        let set: std::collections::BTreeSet<TopicId> = topics.into_iter().collect();
        Self {
            topics: set.into_iter().collect(),
        }
    }

    /// The topics, sorted.
    pub fn topics(&self) -> &[TopicId] {
        &self.topics
    }

    /// Whether the person subscribes to `topic`.
    pub fn contains(&self, topic: &TopicId) -> bool {
        self.topics.binary_search(topic).is_ok()
    }

    /// Whether the person subscribes to nothing.
    pub fn is_empty(&self) -> bool {
        self.topics.is_empty()
    }

    pub(crate) fn with(&self, topic: TopicId) -> Self {
        Self::of(self.topics.iter().cloned().chain([topic]))
    }

    pub(crate) fn without(&self, topic: &TopicId) -> Self {
        Self::of(self.topics.iter().filter(|held| *held != topic).cloned())
    }
}

owned_component! {
    component = Subscriptions,
    owner = NoticesSystem,
    component_type = "subscriptions",
    schema_version = 1,
}

/// The `subscriptions` record as disclosed (S31-b2 §5.5): the topics, and — where the world enables
/// `attributes` — the `notices.subscriptions` attribute block, rendered by the model crate
/// (`disclose_owned`). Without `attributes` the field is omitted and the record is byte-identical to the
/// component itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisclosedSubscriptions {
    /// The topics, sorted.
    pub topics: Vec<TopicId>,
    /// The owned attribute's disclosure, when the world enables `attributes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<serde_json::Value>,
}

/// `notice-board`: the notices current now on the topics the observer subscribes to or publishes, in
/// (topic, publisher) order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoticeBoardRecord {
    /// The current notices.
    pub notices: Vec<Notice>,
}

/// `topics` (S31-b2 §5.6; Q-S31g-3): every topic the world declares, in id order, as one person sees
/// it — so a person can know what there is to follow before subscribing. A declared component type no
/// entity carries, computed from the board and the person's [`Subscriptions`] at each read, and
/// disclosed only about the observer (Q-S31b2-5): `may_publish` and `subscribed` are about them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopicsRecord {
    /// Every declared topic, in id order (at most 32).
    pub topics: Vec<TopicLine>,
}

/// One declared topic, as the observer sees it. The publisher list is not disclosed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopicLine {
    /// The topic.
    pub topic: TopicId,
    /// What its notices carry.
    pub body: BodyKind,
    /// The shortest validity of a notice on it, in seconds (as in `notices-configured`).
    pub valid_min: u32,
    /// The longest validity, in seconds.
    pub valid_max: u32,
    /// The most notices one publisher may post on it in any 24 hours.
    pub per_day: u8,
    /// Whether the observer is one of its publishers.
    pub may_publish: bool,
    /// Whether the observer subscribes to it now.
    pub subscribed: bool,
}

owned_component! {
    component = TopicsRecord,
    owner = NoticesSystem,
    component_type = "topics",
    schema_version = 1,
}

owned_component! {
    component = NoticeBoardRecord,
    owner = NoticesSystem,
    component_type = "notice-board",
    schema_version = 1,
}
