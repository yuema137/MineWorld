//! The three actions this pack provides. None needs anything of space: a bulletin is sent, like a
//! message, and subscribing is a person's own choice (`CLAUDE.md` §4 rule 15).

use mineworld_contracts::{Action, ActionTypeId, SpatialRequirement, SystemId};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::body::NoticeBody;
use crate::system::NoticesSystem;
use crate::topic::TopicId;

/// What every action of this pack requires of space: nothing.
pub const REQUIREMENT: SpatialRequirement = SpatialRequirement::NONE;

/// Post a notice on `topic`, valid for `valid_for` seconds from now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublishNotice {
    topic: TopicId,
    body: NoticeBody,
    valid_for: u32,
}

impl Action for PublishNotice {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("publish-notice");
    const OWNER: SystemId = NoticesSystem::ID;
}

impl PublishNotice {
    /// A request.
    pub const fn new(topic: TopicId, body: NoticeBody, valid_for: u32) -> Self {
        Self {
            topic,
            body,
            valid_for,
        }
    }

    /// The topic.
    pub const fn topic(&self) -> &TopicId {
        &self.topic
    }

    /// The body.
    pub const fn body(&self) -> &NoticeBody {
        &self.body
    }

    /// The validity asked for, in seconds.
    pub const fn valid_for(&self) -> u32 {
        self.valid_for
    }
}

/// Subscribe to `topic`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Subscribe {
    topic: TopicId,
}

impl Action for Subscribe {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("subscribe");
    const OWNER: SystemId = NoticesSystem::ID;
}

impl Subscribe {
    /// A request.
    pub const fn new(topic: TopicId) -> Self {
        Self { topic }
    }

    /// The topic.
    pub const fn topic(&self) -> &TopicId {
        &self.topic
    }
}

/// Unsubscribe from `topic`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Unsubscribe {
    topic: TopicId,
}

impl Action for Unsubscribe {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("unsubscribe");
    const OWNER: SystemId = NoticesSystem::ID;
}

impl Unsubscribe {
    /// A request.
    pub const fn new(topic: TopicId) -> Self {
        Self { topic }
    }

    /// The topic.
    pub const fn topic(&self) -> &TopicId {
        &self.topic
    }
}
