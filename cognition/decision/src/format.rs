//! The decision list's vocabulary: the roster a host hands the reader, and the validated names a
//! `decisions.yaml` writes (`docs/MODULE_SPEC.md` §4.1).
//!
//! The roster is the world's, read by the host from the World Pack and handed in: this crate never reads
//! a World Pack and depends on no loader (`pr-dl-a-decision-list.md` SD-DLa-3, SD-DLa-4).

use std::collections::BTreeSet;
use std::fmt;

use mineworld_contracts::{EntityKey, SystemId};
use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Unexpected, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};

/// An entity class a `class:` assignment names (`DECISIONS.md` `ARC-64`): the spelling rule of a class
/// name, which is a system id's. Its own type rather than the authoring crate's `ClassName`, which would
/// bring the kernel into this crate's direct dependencies for one spelling rule.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct ClassRef(SystemId);

impl ClassRef {
    /// Validates a class name.
    ///
    /// # Errors
    ///
    /// The identifier rule's refusal.
    pub fn new(value: impl Into<String>) -> Result<Self, mineworld_contracts::ContractError> {
        SystemId::new(value).map(Self)
    }

    /// The name as text.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for ClassRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A memory policy's id, as FG-b spells it: `^[a-z][a-z0-9-]{0,47}$` (`pr-fg-b-retention-host.md` §6).
/// Spelled here, never interpreted: whether a policy exists is the cognition process's to judge
/// (QFGb-1).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PolicyId(String);

impl PolicyId {
    /// The longest policy id, in bytes.
    pub const MAX_BYTES: usize = 48;

    /// The id as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for PolicyId {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        let mut bytes = text.bytes();
        let first = bytes.next();
        let spelled = first.is_some_and(|byte| byte.is_ascii_lowercase())
            && text.len() <= Self::MAX_BYTES
            && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
        if spelled {
            Ok(Self(text))
        } else {
            Err(format!(
                "'{text}' is not a policy id: a lowercase letter, then up to 47 lowercase letters, \
                 digits and '-'"
            ))
        }
    }
}

impl From<PolicyId> for String {
    fn from(id: PolicyId) -> Self {
        id.0
    }
}

impl fmt::Display for PolicyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// One seat of the world, with the class its person is in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatEntry {
    /// The seat's person.
    pub seat: EntityKey,
    /// The person's class: the first defined class it matches, else the implicit `person`
    /// (`ARC-64`). A person is in exactly one.
    pub class: ClassRef,
}

/// What a decision list is checked and resolved against: the world's seats in roster order, its
/// classes and its people. Built by the host from the World Pack.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Roster {
    /// Every seat, in roster order — the order resolution and the digest follow.
    pub seats: Vec<SeatEntry>,
    /// The classes of persons the world defines (`configure/classes.yaml`).
    pub classes: BTreeSet<ClassRef>,
    /// The implicit classes (`person`, `place`, `item`, `organization`), which an assignment may not
    /// name: every person is in `person`, so that is what `default:` is for.
    pub implicit: BTreeSet<ClassRef>,
    /// Every person of the world, seat or not.
    pub people: BTreeSet<EntityKey>,
}

impl Roster {
    /// The seat's entry, if it is a seat.
    pub fn seat(&self, key: &EntityKey) -> Option<&SeatEntry> {
        self.seats.iter().find(|entry| entry.seat == *key)
    }
}

/// Where a node was written: 1-based line and column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    /// The line.
    pub line: u64,
    /// The column.
    pub column: u64,
}

impl<T> From<&serde_saphyr::Spanned<T>> for Position {
    fn from(spanned: &serde_saphyr::Spanned<T>) -> Self {
        Self {
            line: spanned.referenced.line(),
            column: spanned.referenced.column(),
        }
    }
}

/// A `memory:` entry's `parameters`, passed through: a mapping with string keys, as a JSON object.
/// YAML scalars become JSON scalars; a non-string key, a non-finite number and anything that is not a
/// mapping at the top are refused (`pr-dl-a-decision-list.md` SD-DLa-9).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct JsonObject(pub Map<String, Value>);

impl<'de> Deserialize<'de> for JsonObject {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(ObjectVisitor).map(Self)
    }
}

struct ObjectVisitor;

impl<'de> Visitor<'de> for ObjectVisitor {
    type Value = Map<String, Value>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a mapping with string keys")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut object = Map::new();
        while let Some(key) = map.next_key_seed(KeySeed)? {
            let value = map.next_value_seed(Untagged)?;
            object.insert(key, value);
        }
        Ok(object)
    }
}

/// A passed-through value that carries no YAML tag. A tag means something to a YAML reader and nothing
/// to the JSON the cognition process receives, so it is refused rather than dropped. serde-saphyr hands
/// a node's tag to a newtype named `__yaml_tagged` as a second sequence element (its `Tagged` wrapper's
/// protocol); any other decoder calls `visit_newtype_struct`, and there is no tag.
struct Untagged;

/// The newtype name serde-saphyr reports a node's tag through.
const TAGGED: &str = "__yaml_tagged";

impl<'de> DeserializeSeed<'de> for Untagged {
    type Value = Value;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        deserializer.deserialize_newtype_struct(TAGGED, self)
    }
}

impl<'de> Visitor<'de> for Untagged {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an untagged value")
    }

    fn visit_newtype_struct<D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Value, D::Error> {
        ValueSeed.deserialize(deserializer)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let value = seq
            .next_element_seed(ValueSeed)?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        match seq.next_element::<String>()? {
            None => Ok(value),
            Some(tag) => Err(de::Error::custom(format!(
                "the tag '{tag}' is refused: parameters pass through as plain JSON"
            ))),
        }
    }
}

/// A key of a passed-through mapping: a string, never a number or a boolean read as one.
struct KeySeed;

impl<'de> DeserializeSeed<'de> for KeySeed {
    type Value = String;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<String, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl Visitor<'_> for KeySeed {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a string key")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<String, E> {
        Ok(value.to_owned())
    }

    fn visit_string<E: de::Error>(self, value: String) -> Result<String, E> {
        Ok(value)
    }
}

/// A value of a passed-through mapping, as JSON.
struct ValueSeed;

impl<'de> DeserializeSeed<'de> for ValueSeed {
    type Value = Value;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for ValueSeed {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON-representable value")
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Value, E> {
        Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::invalid_value(Unexpected::Float(value), &"a finite number"))
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Value, E> {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E: de::Error>(self, value: String) -> Result<Value, E> {
        Ok(Value::String(value))
    }

    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_none<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        deserializer.deserialize_any(self)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut items = Vec::new();
        while let Some(item) = seq.next_element_seed(Untagged)? {
            items.push(item);
        }
        Ok(Value::Array(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Value, A::Error> {
        ObjectVisitor.visit_map(map).map(Value::Object)
    }
}
