//! Public, time-limited notices: posted on topics a world declares, delivered to exactly the people
//! who subscribe.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-115`; design
//! `.structured-coding/plans/mvp0/pr-s31b-notices.md`). It answers the operator's requirement that
//! time-sensitive public information reaches other people's inboxes, and that each person has an
//! attribute saying what they subscribe to.
//!
//! ```text
//! configure   configure/notices.yaml    topics: 1 … 32, each { body, publishers, valid_for, per_day };
//!                                       defaults.by_tag: default subscriptions by person tag
//! section     notices                   a person's { subscribe: [topic] }; replaces the defaults
//! provides    publish-notice            { topic, body, valid_for }   a named publisher posts
//!             subscribe / unsubscribe   { topic }                    the person's own choice
//!                                       every offer incomplete and once (Q-S31a-3); no spatial
//!                                       requirement
//! emits       notices-configured        genesis; SystemInternal
//!             subscriptions-seeded      genesis; SystemInternal (Q-S31b-2)
//!             subscribed, unsubscribed  to that person only
//!             notice-published          to the topic's subscribers at that instant, and the publisher
//! owns        Subscriptions             on a Person: the sorted topics, ≤ 32
//!             one `notices-board`       Process, never woken: topics, counter, newest notice per
//!                                       (topic, publisher), recent issue times
//! discloses   subscriptions             to the person only
//!             notice-board              to the person only: the notices current now on its topics
//! depends on  nothing
//! owner of    notices.subscriptions     where the world enables `attributes` (S31-b2, ARC-124): the
//!                                       declaration is shipped, the vocabulary notices.topics is yielded
//!                                       by configure/notices.yaml, values arrive by attributes-handed;
//!                                       defaults.by_tag and the notices: section are then refused
//!                                       (subscriptions-are-attributes)
//! ```
//!
//! **Lazy expiry.** A notice is current while `issued_at ≤ now < valid_until` and no newer notice from
//! the same publisher on the same topic replaces it. The board is computed from world time at each
//! read, so an expired notice leaves it exactly at `valid_until` with no wake and no fact; the inbox
//! and memory keep it.
//!
//! **Integers only.** Times are world seconds, prices minor units; no float, no hash order.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;
mod section;

pub mod action;
pub mod board;
pub mod body;
pub mod component;
pub mod configuration;
pub mod event;
pub mod judge;
pub mod system;
pub mod topic;

pub use action::{PublishNotice, REQUIREMENT, Subscribe, Unsubscribe};
pub use board::{BoardState, Notice, NoticeId, NoticesProcess, WINDOW_SECONDS, board};
pub use body::{
    NOTICE_PRICE_MAX, NoticeBody, NoticeText, PRICE_LINES_MAX, PriceLine, TEXT_MAX_BYTES,
    judge_body,
};
pub use component::{DisclosedSubscriptions, NoticeBoardRecord, Subscriptions};
pub use configuration::{NoticesConfiguration, TOPICS_MAX};
pub use event::{
    ConfiguredTopic, NoticePublished, NoticesConfigured, Origin, Subscribed, SubscriptionsSeeded,
    Unsubscribed,
};
pub use judge::{Change, judge_publish, judge_subscription};
pub use section::AuthoredNotices;
pub use system::{BIOGRAPHICAL, NoticesSystem, SUBSCRIPTIONS_ATTRIBUTE};
pub use topic::{BodyKind, NoticeDuration, TopicId, ValidFor};
