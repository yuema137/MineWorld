//! Values (`DECISIONS.md` `ARC-122` item 2): the closed enum every fact, component and disclosure
//! carries, exact decimals, and the decoding of an authored value against its kind.
//!
//! No float is stored, compared or used in arithmetic. A YAML number reaches this crate as `f64`; it
//! is written once through Rust's shortest round-trip `Display`, which returns the literal for every
//! number of at most 15 significant digits, and that text is parsed as a decimal by integer arithmetic.
//!
//! The serialized form is externally tagged by kind and self-describing:
//!
//! ```text
//! {"category":"auburn"}   {"level":"early-bird","rank":0}   {"quantity":"1.660"}   {"flag":true}
//! {"set":["a","b"]}       {"text":"…"}                       {"instant":-1072915200}
//! ```
//!
//! A level carries its rank, and a quantity its scale (the decimals of its string), so a value is read
//! without the catalogue.

use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use mineworld_contracts::WorldTime;
use serde::de::{self, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize, Serializer};

use crate::id::{QualifiedId, ValueSlug, nearest};
use crate::kind::{Kind, KindName, MAX_SCALE, TEXT_MAX_BYTES};

/// An exact decimal: an integer and the number of decimals it is counted in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScaledInt {
    value: i64,
    scale: u8,
}

impl ScaledInt {
    /// `value` counted in `scale` decimals: `ScaledInt::new(1660, 3)` is 1.660.
    pub const fn new(value: i64, scale: u8) -> Self {
        Self { value, scale }
    }

    /// The integer at the scale.
    pub const fn value(self) -> i64 {
        self.value
    }

    /// The number of decimals.
    pub const fn scale(self) -> u8 {
        self.scale
    }

    /// The decimal `text` at `scale`.
    ///
    /// # Errors
    ///
    /// When `text` is not a decimal, has more decimals than `scale`, or overflows.
    pub fn parse(text: &str, scale: u8) -> Result<Self, DecimalError> {
        parse_decimal(text, scale).map(|value| Self::new(value, scale))
    }

    /// The decimal text as written at its own scale: as many decimals as it has.
    ///
    /// # Errors
    ///
    /// When `text` is not a decimal, or overflows.
    pub fn parse_exact(text: &str) -> Result<Self, DecimalError> {
        let decimals = text
            .split_once('.')
            .map_or(0, |(_, fraction)| fraction.len());
        let scale = u8::try_from(decimals)
            .ok()
            .filter(|scale| *scale <= MAX_SCALE)
            .ok_or(DecimalError::TooManyDecimals {
                text: text.to_owned(),
                scale: MAX_SCALE,
            })?;
        Self::parse(text, scale)
    }
}

impl fmt::Display for ScaledInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.scale == 0 {
            return write!(f, "{}", self.value);
        }
        let unit = 10_i64.pow(u32::from(self.scale));
        let sign = if self.value < 0 { "-" } else { "" };
        let magnitude = self.value.unsigned_abs();
        let unit = unit.unsigned_abs();
        write!(
            f,
            "{sign}{}.{:0width$}",
            magnitude / unit,
            magnitude % unit,
            width = usize::from(self.scale)
        )
    }
}

/// Why a decimal was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DecimalError {
    /// Not a decimal number.
    #[error("'{text}' is not a decimal number")]
    NotANumber {
        /// As written.
        text: String,
    },
    /// More decimals than the scale admits.
    #[error("{text} has more than {scale} decimals")]
    TooManyDecimals {
        /// As written.
        text: String,
        /// The scale.
        scale: u8,
    },
    /// Too large to hold.
    #[error("{text} is too large")]
    Overflow {
        /// As written.
        text: String,
    },
}

/// `text` as an integer at `scale`, by integer arithmetic alone.
fn parse_decimal(text: &str, scale: u8) -> Result<i64, DecimalError> {
    let not = || DecimalError::NotANumber {
        text: text.to_owned(),
    };
    let overflow = || DecimalError::Overflow {
        text: text.to_owned(),
    };
    let (negative, unsigned) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text),
    };
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    let digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
    if whole.is_empty()
        || !digits(whole)
        || !digits(fraction)
        || (unsigned.contains('.') && fraction.is_empty())
    {
        return Err(not());
    }
    if fraction.len() > usize::from(scale) {
        return Err(DecimalError::TooManyDecimals {
            text: text.to_owned(),
            scale,
        });
    }
    let mut value: i64 = 0;
    let padded = fraction.bytes().chain(core::iter::repeat_n(
        b'0',
        usize::from(scale) - fraction.len(),
    ));
    for byte in whole.bytes().chain(padded) {
        value = value
            .checked_mul(10)
            .and_then(|value| value.checked_add(i64::from(byte - b'0')))
            .ok_or_else(overflow)?;
    }
    Ok(if negative { -value } else { value })
}

/// A level's value: its slug and its rank in the declared order, lowest first.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LevelValue {
    /// The value.
    pub slug: ValueSlug,
    /// Its position in the declared order.
    pub rank: u16,
}

/// Bounded prose: at most 280 bytes of UTF-8 with no control characters.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct BoundedText(String);

impl BoundedText {
    /// `text`, if it is at most `max_bytes` bytes and holds no control character.
    ///
    /// # Errors
    ///
    /// Which rule `text` breaks.
    pub fn new(text: &str, max_bytes: u16) -> Result<Self, String> {
        if text.len() > usize::from(max_bytes.min(TEXT_MAX_BYTES)) {
            return Err(format!(
                "{} bytes is more than the {max_bytes} this text admits",
                text.len()
            ));
        }
        if text.chars().any(char::is_control) {
            return Err("text holds a control character".to_owned());
        }
        Ok(Self(text.to_owned()))
    }

    /// The text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for BoundedText {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::new(&text, TEXT_MAX_BYTES).map_err(de::Error::custom)
    }
}

/// One attribute's value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AttributeValue {
    /// A `category` value.
    Category(ValueSlug),
    /// A `level` value, with its rank.
    Level(LevelValue),
    /// A `quantity` value, at its declaration's scale.
    Quantity(ScaledInt),
    /// A `flag` value.
    Flag(bool),
    /// A `set` value.
    Set(BTreeSet<ValueSlug>),
    /// A `text` value.
    Text(BoundedText),
    /// An `instant` value.
    Instant(WorldTime),
}

impl AttributeValue {
    /// Its kind's name.
    pub const fn kind(&self) -> KindName {
        match self {
            Self::Category(_) => KindName::Category,
            Self::Level(_) => KindName::Level,
            Self::Quantity(_) => KindName::Quantity,
            Self::Flag(_) => KindName::Flag,
            Self::Set(_) => KindName::Set,
            Self::Text(_) => KindName::Text,
            Self::Instant(_) => KindName::Instant,
        }
    }

    /// The value as an author writes it, for tools.
    pub fn display(&self) -> String {
        match self {
            Self::Category(slug) | Self::Level(LevelValue { slug, .. }) => slug.to_string(),
            Self::Quantity(quantity) => quantity.to_string(),
            Self::Flag(flag) => flag.to_string(),
            Self::Set(members) => {
                let members: Vec<&str> = members.iter().map(ValueSlug::as_str).collect();
                format!("[{}]", members.join(", "))
            }
            Self::Text(text) => format!("{:?}", text.as_str()),
            Self::Instant(at) => at.seconds().to_string(),
        }
    }
}

impl Serialize for AttributeValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let rank = matches!(self, Self::Level(_));
        let mut map = serializer.serialize_map(Some(if rank { 2 } else { 1 }))?;
        match self {
            Self::Category(slug) => map.serialize_entry("category", slug)?,
            Self::Level(level) => {
                map.serialize_entry("level", &level.slug)?;
                map.serialize_entry("rank", &level.rank)?;
            }
            Self::Quantity(quantity) => map.serialize_entry("quantity", &quantity.to_string())?,
            Self::Flag(flag) => map.serialize_entry("flag", flag)?,
            Self::Set(members) => map.serialize_entry("set", members)?,
            Self::Text(text) => map.serialize_entry("text", text)?,
            Self::Instant(at) => map.serialize_entry("instant", &at.seconds())?,
        }
        map.end()
    }
}

/// The serialized form, read back: exactly one kind's key, and `rank` with `level` only.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    category: Option<ValueSlug>,
    level: Option<ValueSlug>,
    rank: Option<u16>,
    quantity: Option<String>,
    flag: Option<bool>,
    set: Option<BTreeSet<ValueSlug>>,
    text: Option<BoundedText>,
    instant: Option<i64>,
}

impl<'de> Deserialize<'de> for AttributeValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = Wire::deserialize(deserializer)?;
        let mut found = Vec::new();
        if let Some(slug) = wire.category {
            found.push(Self::Category(slug));
        }
        match (wire.level, wire.rank) {
            (Some(slug), Some(rank)) => found.push(Self::Level(LevelValue { slug, rank })),
            (None, None) => {}
            _ => return Err(de::Error::custom("a level value is a level and its rank")),
        }
        if let Some(text) = wire.quantity {
            found.push(Self::Quantity(
                ScaledInt::parse_exact(&text).map_err(de::Error::custom)?,
            ));
        }
        if let Some(flag) = wire.flag {
            found.push(Self::Flag(flag));
        }
        if let Some(members) = wire.set {
            found.push(Self::Set(members));
        }
        if let Some(text) = wire.text {
            found.push(Self::Text(text));
        }
        if let Some(seconds) = wire.instant {
            found.push(Self::Instant(WorldTime::from_seconds(seconds)));
        }
        let mut found = found.into_iter();
        match (found.next(), found.next()) {
            (Some(value), None) => Ok(value),
            _ => Err(de::Error::custom(
                "an attribute value names exactly one kind",
            )),
        }
    }
}

/// An authored value of any shape, before its declaration says what it must be: what a person file,
/// a `default:` or a threshold is decoded into first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Loose {
    /// `true` or `false`.
    Bool(bool),
    /// A number, as its exact decimal text.
    Number(String),
    /// A string.
    Text(String),
    /// A list.
    List(Vec<Loose>),
    /// A map with string keys.
    Map(BTreeMap<String, Loose>),
}

impl Loose {
    /// What it is, for a refusal.
    pub fn describe(&self) -> String {
        match self {
            Self::Bool(flag) => flag.to_string(),
            Self::Number(text) => text.clone(),
            Self::Text(text) => format!("{text:?}"),
            Self::List(items) => {
                let items: Vec<String> = items.iter().map(Self::describe).collect();
                format!("[{}]", items.join(", "))
            }
            Self::Map(_) => "a map".to_owned(),
        }
    }
}

impl<'de> Deserialize<'de> for Loose {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(LooseVisitor)
    }
}

struct LooseVisitor;

impl<'de> Visitor<'de> for LooseVisitor {
    type Value = Loose;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a value: a slug, a number, true or false, a list, a string or an instant")
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Loose, E> {
        Ok(Loose::Bool(value))
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Loose, E> {
        Ok(Loose::Number(value.to_string()))
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Loose, E> {
        Ok(Loose::Number(value.to_string()))
    }

    /// The shortest round-trip text of `value`: the literal for every number of at most 15
    /// significant digits. Written, never computed with.
    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Loose, E> {
        if !value.is_finite() {
            return Err(E::custom("a number must be finite"));
        }
        Ok(Loose::Number(format!("{value}")))
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Loose, E> {
        Ok(Loose::Text(value.to_owned()))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Loose, A::Error> {
        let mut items = Vec::new();
        while let Some(item) = seq.next_element()? {
            items.push(item);
        }
        Ok(Loose::List(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Loose, A::Error> {
        let mut entries = BTreeMap::new();
        while let Some((key, value)) = map.next_entry::<String, Loose>()? {
            entries.insert(key, value);
        }
        Ok(Loose::Map(entries))
    }
}

/// Why an authored value does not fit its declaration. Each names what `MODULE_SPEC.md` §4.1's
/// *Names* column lists, with the attribute added by the caller.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ValueError {
    /// A value of another kind.
    #[error("WrongKind: {value} is not a value of {domain}")]
    WrongKind {
        /// The value as written.
        value: String,
        /// The declared domain.
        domain: String,
    },
    /// Outside a closed list.
    #[error("OutOfDomain: {value} is not in {domain}")]
    OutOfDomain {
        /// The value as written.
        value: String,
        /// The declared domain.
        domain: String,
    },
    /// Outside a vocabulary.
    #[error("UnknownTopic: {value} is not in the vocabulary {vocabulary} (nearest: {})", .nearest.join(", "))]
    UnknownTopic {
        /// The vocabulary.
        vocabulary: QualifiedId,
        /// The value as written.
        value: String,
        /// The nearest ids it holds.
        nearest: Vec<String>,
    },
    /// A quantity outside its bounds, or with more decimals than its scale.
    #[error("OutOfRange: {value} is outside {bounds}")]
    OutOfRange {
        /// The value as written.
        value: String,
        /// The bounds, or the scale's rule.
        bounds: String,
    },
}

/// One slug of `values`, or the refusal naming the domain (and, for a vocabulary, the nearest).
fn member(
    kind: &Kind,
    values: &[ValueSlug],
    vocabulary: Option<&QualifiedId>,
    loose: &Loose,
) -> Result<ValueSlug, ValueError> {
    let Loose::Text(text) = loose else {
        return Err(wrong(kind, loose));
    };
    if let Some(found) = values.iter().find(|value| value.as_str() == text) {
        return Ok(found.clone());
    }
    Err(match vocabulary {
        Some(vocabulary) => ValueError::UnknownTopic {
            vocabulary: vocabulary.clone(),
            value: text.clone(),
            nearest: nearest(text, values.iter().map(ValueSlug::as_str)),
        },
        None => ValueError::OutOfDomain {
            value: text.clone(),
            domain: kind.domain(),
        },
    })
}

fn wrong(kind: &Kind, loose: &Loose) -> ValueError {
    ValueError::WrongKind {
        value: loose.describe(),
        domain: kind.domain(),
    }
}

/// `loose` as a value of `kind`.
///
/// # Errors
///
/// [`ValueError`]: of the wrong kind, outside the domain or vocabulary, or out of range.
pub fn decode(kind: &Kind, loose: &Loose) -> Result<AttributeValue, ValueError> {
    match kind {
        Kind::Category { values, vocabulary } => {
            member(kind, values, vocabulary.as_ref(), loose).map(AttributeValue::Category)
        }
        Kind::Level { values } => {
            let slug = member(kind, values, None, loose)?;
            let rank = values
                .iter()
                .position(|value| *value == slug)
                .unwrap_or_default();
            let rank = u16::try_from(rank).unwrap_or(u16::MAX);
            Ok(AttributeValue::Level(LevelValue { slug, rank }))
        }
        Kind::Quantity(quantity) => {
            let text = match loose {
                Loose::Number(text) | Loose::Text(text) => text,
                _ => return Err(wrong(kind, loose)),
            };
            let value = ScaledInt::parse(text, quantity.scale).map_err(|error| match error {
                DecimalError::NotANumber { .. } => wrong(kind, loose),
                DecimalError::TooManyDecimals { .. } | DecimalError::Overflow { .. } => {
                    ValueError::OutOfRange {
                        value: text.clone(),
                        bounds: error.to_string(),
                    }
                }
            })?;
            if value.value() < quantity.min || value.value() > quantity.max {
                return Err(ValueError::OutOfRange {
                    value: text.clone(),
                    bounds: format!(
                        "{} … {}",
                        ScaledInt::new(quantity.min, quantity.scale),
                        ScaledInt::new(quantity.max, quantity.scale)
                    ),
                });
            }
            Ok(AttributeValue::Quantity(value))
        }
        Kind::Flag => match loose {
            Loose::Bool(flag) => Ok(AttributeValue::Flag(*flag)),
            _ => Err(wrong(kind, loose)),
        },
        Kind::Set {
            values,
            vocabulary,
            max,
        } => {
            let Loose::List(items) = loose else {
                return Err(wrong(kind, loose));
            };
            let mut members = BTreeSet::new();
            for item in items {
                members.insert(member(kind, values, vocabulary.as_ref(), item)?);
            }
            if members.len() > usize::from(*max) {
                return Err(ValueError::OutOfRange {
                    value: loose.describe(),
                    bounds: format!("at most {max} members"),
                });
            }
            Ok(AttributeValue::Set(members))
        }
        Kind::Text { max_bytes } => match loose {
            Loose::Text(text) => BoundedText::new(text, *max_bytes)
                .map(AttributeValue::Text)
                .map_err(|bounds| ValueError::OutOfRange {
                    value: format!("{} bytes of text", text.len()),
                    bounds,
                }),
            _ => Err(wrong(kind, loose)),
        },
        Kind::Instant => match loose {
            Loose::Number(text) => text
                .parse::<i64>()
                .map(|seconds| AttributeValue::Instant(WorldTime::from_seconds(seconds)))
                .map_err(|_| wrong(kind, loose)),
            _ => Err(wrong(kind, loose)),
        },
    }
}

/// Whether `value` is a value of `kind`: of its kind, in its domain, within its bounds and scale — what
/// a proposal's `to` is checked by (`ARC-120` item 3).
pub fn admits(kind: &Kind, value: &AttributeValue) -> bool {
    match (kind, value) {
        (Kind::Category { values, .. }, AttributeValue::Category(slug)) => values.contains(slug),
        (Kind::Level { values }, AttributeValue::Level(level)) => {
            values.get(usize::from(level.rank)) == Some(&level.slug)
        }
        (Kind::Quantity(quantity), AttributeValue::Quantity(value)) => {
            value.scale() == quantity.scale
                && (quantity.min..=quantity.max).contains(&value.value())
        }
        (Kind::Flag, AttributeValue::Flag(_)) | (Kind::Instant, AttributeValue::Instant(_)) => true,
        (Kind::Set { values, max, .. }, AttributeValue::Set(members)) => {
            members.len() <= usize::from(*max)
                && members.iter().all(|member| values.contains(member))
        }
        (Kind::Text { max_bytes }, AttributeValue::Text(text)) => {
            text.as_str().len() <= usize::from(*max_bytes)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests;
