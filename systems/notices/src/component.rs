//! The state this pack keeps on a person, and the two records it discloses.
//!
//! `subscriptions` is a person's component and is disclosed as itself, to that person only.
//! `notice-board` is a declared component type that no entity carries, like calendar's records: the
//! notices live in the board Process, and this is how the current ones are shown to one person.

use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::board::Notice;
use crate::system::NoticesSystem;
use crate::topic::TopicId;

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

/// `notice-board`: the notices current now on the topics the observer subscribes to or publishes, in
/// (topic, publisher) order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoticeBoardRecord {
    /// The current notices.
    pub notices: Vec<Notice>,
}

owned_component! {
    component = NoticeBoardRecord,
    owner = NoticesSystem,
    component_type = "notice-board",
    schema_version = 1,
}
