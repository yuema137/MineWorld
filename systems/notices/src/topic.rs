//! What a topic is: its name, what its notices carry, who may post on it, how long a notice on it may
//! stay valid, and how often one publisher may post.
//!
//! Every type here refuses a bad value while it decodes, with a message naming the key, so a world
//! whose `configure/notices.yaml` breaks a bound is refused at its line and column (`ARC-61` item 1).

use std::collections::BTreeSet;

use mineworld_contracts::EntityKey;
use serde::{Deserialize, Serialize};

/// The longest topic id, in bytes.
pub const TOPIC_MAX_BYTES: usize = 64;

/// The shortest validity a topic may allow: one minute.
pub const VALID_MIN_SECONDS: u32 = 60;

/// The longest validity a topic may allow: thirty days.
pub const VALID_MAX_SECONDS: u32 = 30 * 86_400;

/// The most publishers one topic may name.
pub const PUBLISHERS_MAX: usize = 16;

/// The most notices one publisher may post on one topic in any 24 hours.
pub const PER_DAY_MAX: u8 = 96;

/// A topic's name: dotted lowercase segments, such as `prices.ingredients` or `news.daily`.
///
/// 1 to 64 bytes. Each segment is `a-z` and `0-9`, with `-` allowed inside it but not at either end;
/// segments are joined by `.`. The same form as an attribute vocabulary's ids (step-30 N-1), so the
/// topic keys can become `notices.topics` unchanged when `notices` adopts S30's owner interface.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(try_from = "String", into = "String")]
#[schemars(
    with = "String",
    extend("pattern" = "^[a-z0-9]([a-z0-9-]*[a-z0-9])?(\\.[a-z0-9]([a-z0-9-]*[a-z0-9])?)*$"),
    extend("maxLength" = 64)
)]
pub struct TopicId(String);

impl TopicId {
    /// A topic id, refused unless it keeps the rule above.
    ///
    /// # Errors
    ///
    /// A message naming what was offered.
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        let segment_ok = |segment: &str| {
            let bytes = segment.as_bytes();
            !bytes.is_empty()
                && bytes
                    .iter()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
                && bytes.first() != Some(&b'-')
                && bytes.last() != Some(&b'-')
        };
        if !value.is_empty() && value.len() <= TOPIC_MAX_BYTES && value.split('.').all(segment_ok) {
            Ok(Self(value))
        } else {
            Err(format!(
                "a topic is 1 to {TOPIC_MAX_BYTES} bytes of dotted segments of a-z, 0-9 and '-' \
                 (no '-' at either end of a segment), such as prices.ingredients, not '{value}'"
            ))
        }
    }

    /// The id as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for TopicId {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<TopicId> for String {
    fn from(value: TopicId) -> Self {
        value.0
    }
}

impl core::fmt::Display for TopicId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

/// What a notice on a topic carries: the body-kind catalog (`ARC-115`). Adding a kind is one variant
/// here, one in [`NoticeBody`](crate::NoticeBody) and its renderer, reviewed as a vocabulary change.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum BodyKind {
    /// Price lines: an item and an amount in minor units, with how many observations back it.
    Prices,
    /// One line of text, 1 to 280 bytes.
    Text,
}

/// A duration written `"<n>m"`, `"<n>h"` or `"<n>d"` (minutes, hours, days), such as `"6h"` or
/// `"7d"`, from one minute to thirty days. Held in seconds.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(try_from = "String")]
#[schemars(with = "String", extend("pattern" = "^[1-9][0-9]*[mhd]$"))]
pub struct NoticeDuration(u32);

impl NoticeDuration {
    /// The duration in seconds.
    pub const fn seconds(self) -> u32 {
        self.0
    }
}

impl TryFrom<String> for NoticeDuration {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        let refused = || {
            format!(
                "a duration is \"<n>m\", \"<n>h\" or \"<n>d\" from 1m to 30d, such as \"6h\", not \
                 '{text}'"
            )
        };
        let Some(unit) = text.chars().last() else {
            return Err(refused());
        };
        let per = match unit {
            'm' => 60_u64,
            'h' => 3_600,
            'd' => 86_400,
            _ => return Err(refused()),
        };
        let digits = &text[..text.len() - 1];
        if digits.is_empty()
            || !digits.bytes().all(|b| b.is_ascii_digit())
            || digits.starts_with('0')
        {
            return Err(refused());
        }
        let count: u64 = digits.parse().map_err(|_| refused())?;
        let seconds = count.checked_mul(per).ok_or_else(refused)?;
        if seconds < u64::from(VALID_MIN_SECONDS) || seconds > u64::from(VALID_MAX_SECONDS) {
            return Err(refused());
        }
        u32::try_from(seconds).map(Self).map_err(|_| refused())
    }
}

/// How long a notice on a topic may stay valid: from `min` to `max`, with `min` at most `max`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "RawValidFor")]
#[schemars(with = "RawValidFor")]
pub struct ValidFor {
    min: NoticeDuration,
    max: NoticeDuration,
}

/// `valid_for` as written, before `min ≤ max` is checked.
#[derive(Debug, Clone, Copy, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawValidFor {
    /// The shortest validity a notice on this topic may ask for, such as "1h".
    min: NoticeDuration,
    /// The longest validity a notice on this topic may ask for, such as "48h"; at least `min`.
    max: NoticeDuration,
}

impl TryFrom<RawValidFor> for ValidFor {
    type Error = String;

    fn try_from(raw: RawValidFor) -> Result<Self, Self::Error> {
        if raw.min <= raw.max {
            Ok(Self {
                min: raw.min,
                max: raw.max,
            })
        } else {
            Err(format!(
                "valid_for: min ({} s) must not be longer than max ({} s)",
                raw.min.seconds(),
                raw.max.seconds()
            ))
        }
    }
}

impl ValidFor {
    /// The shortest validity, in seconds.
    pub const fn min(self) -> u32 {
        self.min.seconds()
    }

    /// The longest validity, in seconds.
    pub const fn max(self) -> u32 {
        self.max.seconds()
    }
}

/// The most notices one publisher may post on a topic in any 24 hours of world time, 1 to 96
/// (Q-S31b-4: a rolling window of 86 400 s, not a calendar day).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "u8")]
#[schemars(with = "u8", extend("minimum" = 1, "maximum" = 96))]
pub struct PerDay(u8);

impl PerDay {
    /// The count.
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for PerDay {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if (1..=PER_DAY_MAX).contains(&value) {
            Ok(Self(value))
        } else {
            Err(format!(
                "per_day is 1 to {PER_DAY_MAX} notices in any 24 hours, not {value}"
            ))
        }
    }
}

/// The people who may post on a topic: 1 to 16 person keys, none twice.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "Vec<EntityKey>")]
#[schemars(
    with = "Vec<mineworld_authoring::schema::Key>",
    extend("minItems" = 1, "maxItems" = 16, "uniqueItems" = true)
)]
pub struct Publishers(Vec<EntityKey>);

impl Publishers {
    /// The keys, as written.
    pub fn keys(&self) -> &[EntityKey] {
        &self.0
    }
}

impl TryFrom<Vec<EntityKey>> for Publishers {
    type Error = String;

    fn try_from(keys: Vec<EntityKey>) -> Result<Self, Self::Error> {
        if keys.is_empty() || keys.len() > PUBLISHERS_MAX {
            return Err(format!(
                "publishers lists 1 to {PUBLISHERS_MAX} person keys, not {}",
                keys.len()
            ));
        }
        let mut seen = BTreeSet::new();
        for key in &keys {
            if !seen.insert(key) {
                return Err(format!("publishers: `{}` is listed twice", key.as_str()));
            }
        }
        Ok(Self(keys))
    }
}

/// One topic, as `configure/notices.yaml` declares it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Topic {
    /// What its notices carry: `prices` (item and amount lines) or `text` (one line).
    body: BodyKind,
    /// Who may post on it: 1 to 16 person keys.
    publishers: Publishers,
    /// How long a notice on it may stay valid: `{ min, max }`, each "<n>m", "<n>h" or "<n>d".
    valid_for: ValidFor,
    /// The most notices one publisher may post on it in any 24 hours of world time, 1 to 96.
    per_day: PerDay,
}

impl Topic {
    /// What its notices carry.
    pub const fn body(&self) -> BodyKind {
        self.body
    }

    /// Who may post on it.
    pub fn publishers(&self) -> &[EntityKey] {
        self.publishers.keys()
    }

    /// Its validity bounds.
    pub const fn valid_for(&self) -> ValidFor {
        self.valid_for
    }

    /// Its rolling limit.
    pub const fn per_day(&self) -> u8 {
        self.per_day.get()
    }
}
