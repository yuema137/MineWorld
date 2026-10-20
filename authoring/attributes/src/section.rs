//! The person file's `attributes:` section (`DECISIONS.md` `ARC-122`, `ARC-123`; `MODULE_SPEC.md` §4.1
//! "The person file's section"), decoded against the catalogue so every refusal keeps the line and
//! column of the entry it is about.
//!
//! ```yaml
//! attributes:
//!   stature: 1.66                         # a value, in the kind's own form
//!   attributes-town.chronotype: early-bird
//!   hair-colour: { allowed: [brown, auburn] }   # a restriction the sampler respects (S30-d)
//!   weight: { between: [55, 70] }
//! ```

use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, Visitor};

use crate::catalogue::Catalogue;
use crate::declaration::{Declaration, Mutability};
use crate::id::{QualifiedId, ValueSlug};
use crate::kind::Kind;
use crate::value::{AttributeValue, Loose, ScaledInt, ValueError, decode};

/// A restriction a person file places on a value the sampler draws (S30-d reads it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Restriction {
    /// One of these values (a `category` or `level`), or only these members (a `set`).
    Allowed(BTreeSet<ValueSlug>),
    /// A quantity within these bounds, inclusive, at the declaration's scale.
    Between(ScaledInt, ScaledInt),
}

/// One person's section, decoded and checked: the values it states and the restrictions it places.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Authored {
    /// The values it states.
    pub values: BTreeMap<QualifiedId, AttributeValue>,
    /// The restrictions it places.
    pub restrictions: BTreeMap<QualifiedId, Restriction>,
}

/// Decodes a section against `catalogue`.
#[derive(Debug, Clone, Copy)]
pub struct SectionIn<'c>(pub &'c Catalogue);

impl<'de> DeserializeSeed<'de> for SectionIn<'_> {
    type Value = Authored;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Authored, D::Error> {
        deserializer.deserialize_map(self)
    }
}

impl<'de> Visitor<'de> for SectionIn<'_> {
    type Value = Authored;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a map of attribute ids to values or restrictions")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Authored, A::Error> {
        let mut authored = Authored::default();
        while let Some(written) = map.next_key::<String>()? {
            let declaration = self.0.resolve(&written).map_err(de::Error::custom)?;
            let id = declaration.id.clone();
            if authored.values.contains_key(&id) || authored.restrictions.contains_key(&id) {
                return Err(de::Error::custom(format!(
                    "'{id}' is stated twice in this section"
                )));
            }
            match map.next_value_seed(EntryIn(declaration))? {
                Entry::Value(value) => {
                    authored.values.insert(id, value);
                }
                Entry::Restriction(restriction) => {
                    authored.restrictions.insert(id, restriction);
                }
            }
        }
        Ok(authored)
    }
}

/// One entry: a value or a restriction.
enum Entry {
    Value(AttributeValue),
    Restriction(Restriction),
}

/// Decodes one entry against its declaration, refusing at the entry's own position.
struct EntryIn<'d>(&'d Declaration);

impl<'de> DeserializeSeed<'de> for EntryIn<'_> {
    type Value = Entry;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Entry, D::Error> {
        let declaration = self.0;
        let loose = <Loose as serde::Deserialize>::deserialize(deserializer)?;
        let refuse =
            |error: ValueError| de::Error::custom(format!("'{}': {error}", declaration.id));
        if declaration.mutability == Mutability::Derived {
            return Err(refuse(ValueError::OutOfDomain {
                value: loose.describe(),
                domain: "nothing: nothing states a derived value".to_owned(),
            }));
        }
        if let Loose::Map(map) = &loose
            && map.len() == 1
        {
            if let Some(allowed) = map.get("allowed") {
                return restriction_allowed(&declaration.kind, allowed)
                    .map(Entry::Restriction)
                    .map_err(refuse);
            }
            if let Some(between) = map.get("between") {
                return restriction_between(&declaration.kind, between)
                    .map(Entry::Restriction)
                    .map_err(refuse);
            }
        }
        decode(&declaration.kind, &loose)
            .map(Entry::Value)
            .map_err(refuse)
    }
}

/// `{ allowed: [...] }`, for a `category`, `level` or `set`: every value in the domain.
fn restriction_allowed(kind: &Kind, allowed: &Loose) -> Result<Restriction, ValueError> {
    let wrong = || ValueError::WrongKind {
        value: format!("{{ allowed: {} }}", allowed.describe()),
        domain: kind.domain(),
    };
    let Loose::List(items) = allowed else {
        return Err(wrong());
    };
    let single = match kind {
        Kind::Category { values, vocabulary } => Kind::Category {
            values: values.clone(),
            vocabulary: vocabulary.clone(),
        },
        Kind::Level { values } => Kind::Level {
            values: values.clone(),
        },
        Kind::Set {
            values, vocabulary, ..
        } => Kind::Category {
            values: values.clone(),
            vocabulary: vocabulary.clone(),
        },
        Kind::Quantity(_) | Kind::Flag | Kind::Text { .. } | Kind::Instant => return Err(wrong()),
    };
    let mut slugs = BTreeSet::new();
    for item in items {
        match decode(&single, item)? {
            AttributeValue::Category(slug) => slugs.insert(slug),
            AttributeValue::Level(level) => slugs.insert(level.slug),
            _ => return Err(wrong()),
        };
    }
    if slugs.is_empty() {
        return Err(wrong());
    }
    Ok(Restriction::Allowed(slugs))
}

/// `{ between: [min, max] }`, for a `quantity`: both within the declared bounds, min ≤ max.
fn restriction_between(kind: &Kind, between: &Loose) -> Result<Restriction, ValueError> {
    let wrong = || ValueError::WrongKind {
        value: format!("{{ between: {} }}", between.describe()),
        domain: kind.domain(),
    };
    let (Kind::Quantity(_), Loose::List(items)) = (kind, between) else {
        return Err(wrong());
    };
    let [low, high] = items.as_slice() else {
        return Err(wrong());
    };
    let (AttributeValue::Quantity(low), AttributeValue::Quantity(high)) =
        (decode(kind, low)?, decode(kind, high)?)
    else {
        return Err(wrong());
    };
    if low > high {
        return Err(ValueError::OutOfRange {
            value: format!("[{low}, {high}]"),
            bounds: "min ≤ max".to_owned(),
        });
    }
    Ok(Restriction::Between(low, high))
}

/// The section's shape, for the generated reference (`ARC-90`): a map of attribute ids to an entry. The
/// rules a value must meet are its declaration's, so the schema states the forms, not the domains.
#[derive(Debug, schemars::JsonSchema)]
#[schemars(example = serde_json::json!({
    "stature": 1.66,
    "core.chronotype": "early-bird",
    "glasses": true,
    "languages-spoken": ["english", "spanish"],
    "hair-colour": { "allowed": ["brown", "auburn"] },
}))]
#[allow(dead_code)]
pub struct SectionSchema(
    /// Each attribute a person file states, by its qualified id or its unique short slug.
    BTreeMap<String, EntrySchema>,
);

/// One entry of the section: a value written in its kind's own form, or a restriction.
#[derive(Debug, schemars::JsonSchema)]
#[serde(untagged)]
#[allow(dead_code)]
pub enum EntrySchema {
    /// A `category` or `level` value's slug, a `quantity` written as a decimal string, or a `text`.
    Text(String),
    /// A `quantity` in its declared unit, or an `instant` in world seconds.
    Number(serde_json::Number),
    /// A `flag`.
    Flag(bool),
    /// A `set`'s members.
    Members(Vec<String>),
    /// A restriction the sampler respects (S30-d).
    Restriction(RestrictionSchema),
}

/// `{ allowed: [...] }` or `{ between: [min, max] }`.
#[derive(Debug, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
pub struct RestrictionSchema {
    /// For a `category`, `level` or `set`: the values the sampler may draw.
    #[serde(default)]
    pub allowed: Option<Vec<String>>,
    /// For a `quantity`: the inclusive bounds the sampler may draw within.
    #[serde(default)]
    pub between: Option<Vec<serde_json::Number>>,
}
