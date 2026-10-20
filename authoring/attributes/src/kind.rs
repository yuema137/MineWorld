//! Value kinds (`DECISIONS.md` `ARC-122` item 2; `MODULE_SPEC.md` §4.1 "Attribute declarations"): the
//! closed list, units, and the marker types a typed handle is written with.
//!
//! ```text
//! category   a closed list of value slugs, or a vocabulary       stored as the slug
//! level      an ordered list of value slugs                        stored as the slug and its rank
//! quantity   unit, scale (0..=6 decimals), min, max                stored as an integer at the scale
//! flag       —                                                     true or false
//! set        of: a closed list, or a vocabulary; max members       a sorted set of slugs
//! text       max_bytes, at most 280                                bounded UTF-8; never read by a System
//! instant    —                                                     world seconds, possibly negative
//! ```

use core::fmt;
use std::collections::BTreeSet;

use mineworld_contracts::WorldTime;
use serde::{Deserialize, Serialize};

use crate::id::{QualifiedId, ValueSlug};
use crate::value::{AttributeValue, LevelValue, ScaledInt};

/// The largest `max_bytes` a `text` attribute may declare.
pub const TEXT_MAX_BYTES: u16 = 280;

/// The largest scale a `quantity` may declare.
pub const MAX_SCALE: u8 = 6;

/// A quantity's unit: a closed list. Currency is not one (money is `economy`'s).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    /// Metres.
    M,
    /// Kilograms.
    Kg,
    /// Years.
    Year,
    /// Days.
    Day,
    /// Hours.
    Hour,
    /// A count.
    Count,
    /// A ratio.
    Ratio,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::M => "m",
            Self::Kg => "kg",
            Self::Year => "year",
            Self::Day => "day",
            Self::Hour => "hour",
            Self::Count => "count",
            Self::Ratio => "ratio",
        })
    }
}

/// A quantity's declaration: its unit, its scale, and its bounds at that scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuantityKind {
    /// The unit.
    pub unit: Unit,
    /// Decimal places: `m` at scale 3 is millimetres.
    pub scale: u8,
    /// The smallest value, at the scale.
    pub min: i64,
    /// The largest value, at the scale.
    pub max: i64,
}

/// A kind, as the frozen catalogue holds it: every vocabulary already resolved to its values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
pub enum Kind {
    /// One of a closed list.
    Category {
        /// The values, in declared order.
        values: Vec<ValueSlug>,
        /// The vocabulary they came from, if any.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        vocabulary: Option<QualifiedId>,
    },
    /// One of an ordered list.
    Level {
        /// The values, lowest rank first.
        values: Vec<ValueSlug>,
    },
    /// An exact decimal number.
    Quantity(QuantityKind),
    /// True or false.
    Flag,
    /// Some of a closed list.
    Set {
        /// The values, in declared order.
        values: Vec<ValueSlug>,
        /// The vocabulary they came from, if any.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        vocabulary: Option<QualifiedId>,
        /// The most members a value may have.
        max: u16,
    },
    /// Bounded prose: rendered, disclosed, never read by a System.
    Text {
        /// The most bytes a value may have.
        max_bytes: u16,
    },
    /// A world instant.
    Instant,
}

impl Kind {
    /// The kind's name.
    pub const fn name(&self) -> KindName {
        match self {
            Self::Category { .. } => KindName::Category,
            Self::Level { .. } => KindName::Level,
            Self::Quantity(_) => KindName::Quantity,
            Self::Flag => KindName::Flag,
            Self::Set { .. } => KindName::Set,
            Self::Text { .. } => KindName::Text,
            Self::Instant => KindName::Instant,
        }
    }

    /// The values of a closed list, for a `category`, `level` or `set`.
    pub fn values(&self) -> Option<&[ValueSlug]> {
        match self {
            Self::Category { values, .. } | Self::Level { values } | Self::Set { values, .. } => {
                Some(values)
            }
            Self::Quantity(_) | Self::Flag | Self::Text { .. } | Self::Instant => None,
        }
    }

    /// The domain, as a refusal prints it.
    pub fn domain(&self) -> String {
        match self {
            Self::Category { values, .. } | Self::Level { values } | Self::Set { values, .. } => {
                let listed: Vec<&str> = values.iter().map(ValueSlug::as_str).collect();
                format!("{} of [{}]", self.name(), listed.join(", "))
            }
            Self::Quantity(quantity) => format!(
                "quantity in {} at scale {}, {} … {}",
                quantity.unit,
                quantity.scale,
                ScaledInt::new(quantity.min, quantity.scale),
                ScaledInt::new(quantity.max, quantity.scale)
            ),
            Self::Flag => "flag (true or false)".to_owned(),
            Self::Text { max_bytes } => format!("text of at most {max_bytes} bytes"),
            Self::Instant => "instant (world seconds)".to_owned(),
        }
    }
}

/// A kind's name: what a handle requests, what `uses.reads` declares, and the `kind` of a disclosed
/// entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KindName {
    /// `category`.
    Category,
    /// `level`.
    Level,
    /// `quantity`.
    Quantity,
    /// `flag`.
    Flag,
    /// `set`.
    Set,
    /// `text`.
    Text,
    /// `instant`.
    Instant,
}

impl fmt::Display for KindName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Category => "category",
            Self::Level => "level",
            Self::Quantity => "quantity",
            Self::Flag => "flag",
            Self::Set => "set",
            Self::Text => "text",
            Self::Instant => "instant",
        })
    }
}

/// The marker of a `category` handle.
#[derive(Debug)]
pub enum Category {}
/// The marker of a `level` handle.
#[derive(Debug)]
pub enum Level {}
/// The marker of a `quantity` handle.
#[derive(Debug)]
pub enum Quantity {}
/// The marker of a `flag` handle.
#[derive(Debug)]
pub enum Flag {}
/// The marker of a `set` handle.
#[derive(Debug)]
pub enum Set {}
/// The marker of an `instant` handle.
#[derive(Debug)]
pub enum Instant {}
/// The `text` kind, which no handle may name: it implements no [`SystemReadable`], so
/// `Handle<Text>` does not compile (X-S30-11). No System reads prose.
#[derive(Debug)]
pub enum Text {}

/// A kind a System may read through a typed handle, and the value it reads as.
pub trait SystemReadable: 'static {
    /// The kind's name.
    const KIND: KindName;
    /// What a read returns.
    type Value: fmt::Debug + Clone + PartialEq;
    /// The value, if `value` is of this kind.
    fn read(value: &AttributeValue) -> Option<Self::Value>;
}

impl SystemReadable for Category {
    const KIND: KindName = KindName::Category;
    type Value = ValueSlug;
    fn read(value: &AttributeValue) -> Option<ValueSlug> {
        match value {
            AttributeValue::Category(slug) => Some(slug.clone()),
            _ => None,
        }
    }
}

impl SystemReadable for Level {
    const KIND: KindName = KindName::Level;
    type Value = LevelValue;
    fn read(value: &AttributeValue) -> Option<LevelValue> {
        match value {
            AttributeValue::Level(level) => Some(level.clone()),
            _ => None,
        }
    }
}

impl SystemReadable for Quantity {
    const KIND: KindName = KindName::Quantity;
    type Value = ScaledInt;
    fn read(value: &AttributeValue) -> Option<ScaledInt> {
        match value {
            AttributeValue::Quantity(quantity) => Some(*quantity),
            _ => None,
        }
    }
}

impl SystemReadable for Flag {
    const KIND: KindName = KindName::Flag;
    type Value = bool;
    fn read(value: &AttributeValue) -> Option<bool> {
        match value {
            AttributeValue::Flag(flag) => Some(*flag),
            _ => None,
        }
    }
}

impl SystemReadable for Set {
    const KIND: KindName = KindName::Set;
    type Value = BTreeSet<ValueSlug>;
    fn read(value: &AttributeValue) -> Option<BTreeSet<ValueSlug>> {
        match value {
            AttributeValue::Set(members) => Some(members.clone()),
            _ => None,
        }
    }
}

impl SystemReadable for Instant {
    const KIND: KindName = KindName::Instant;
    type Value = WorldTime;
    fn read(value: &AttributeValue) -> Option<WorldTime> {
        match value {
            AttributeValue::Instant(at) => Some(*at),
            _ => None,
        }
    }
}
