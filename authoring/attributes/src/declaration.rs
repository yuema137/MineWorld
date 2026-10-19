//! Declarations (`DECISIONS.md` `ARC-122`; `MODULE_SPEC.md` §4.1 "Attribute declarations"): the file
//! formats of `attributes/` and `vocabularies/`, decoded with their line and column, and one declaration
//! as the frozen catalogue holds it.
//!
//! ```yaml
//! format: 1
//! attributes:   { <slug>: { group, facets?, kind, visibility, mutability, derive?, owner?, label,
//!                           phrase?, words?, perceived_as?, default?, since?, prompt_rank? } }
//! overrides:    { <qualified id>: { label?, phrase?, words?, perceived_as?, facets?, default?,
//!                                   visibility? } }
//! uses:         { reads?: { <id>: <kind> }, proposes?: [<id>] }     only in a file a System Pack ships
//! ```
//!
//! A field's own rule is checked inside its deserializer, so the refusal keeps the field's line and
//! column; a rule across fields (a `default` of the declared kind, a band's threshold) is checked when
//! the catalogue is built, naming the declaration's line.

use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use mineworld_contracts::SystemId;
use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};
use serde_saphyr::Spanned;

use crate::id::{QualifiedId, Slug, ValueSlug};
use crate::kind::{Kind, KindName, MAX_SCALE, QuantityKind, TEXT_MAX_BYTES, Unit};
use crate::value::{AttributeValue, LevelValue, Loose, ScaledInt, ValueError, decode};

/// The six built-in groups.
pub const GROUPS: [&str; 6] = [
    "appearance",
    "background",
    "psychology",
    "capability",
    "behaviour",
    "lifestyle",
];

/// Who may learn a value (`CORE_CONCEPTS.md` §4.5), ordered from widest to narrowest: an override may
/// only move a visibility later in this order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// Anyone who perceives the person.
    Perceivable,
    /// The holder; others only by learning it, which nothing in MVP-0 does yet.
    Learnable,
    /// The holder alone.
    Private,
    /// Nobody, not even the holder's observation.
    Hidden,
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Perceivable => "perceivable",
            Self::Learnable => "learnable",
            Self::Private => "private",
            Self::Hidden => "hidden",
        })
    }
}

/// Whether a value changes (`CORE_CONCEPTS.md` §4.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mutability {
    /// Never after genesis.
    Fixed,
    /// By an accepted proposal.
    Evolving,
    /// Computed from other state; never stored.
    Derived,
}

impl fmt::Display for Mutability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Fixed => "fixed",
            Self::Evolving => "evolving",
            Self::Derived => "derived",
        })
    }
}

/// A threshold a band compares against: a quantity at its scale, or a level by rank.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Threshold {
    /// Below this quantity.
    Quantity(ScaledInt),
    /// Below this level's rank.
    Level(LevelValue),
}

impl Serialize for ScaledInt {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ScaledInt {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse_exact(&text).map_err(de::Error::custom)
    }
}

impl Serialize for LevelValue {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (&self.slug, self.rank).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for LevelValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (slug, rank) = <(ValueSlug, u16)>::deserialize(deserializer)?;
        Ok(Self { slug, rank })
    }
}

/// How a derived attribute is computed (`MODULE_SPEC.md` §4.1 "Derived attributes").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
pub enum Derive {
    /// Whole years since an `instant` attribute, at the read's instant.
    Age {
        /// The instant attribute.
        from: QualifiedId,
    },
    /// The level value of the first threshold the source is below, else the last.
    Band {
        /// A quantity or level attribute.
        from: QualifiedId,
        /// Strictly increasing thresholds, one fewer than the level's values.
        below: Vec<Threshold>,
    },
    /// The owner's code, through `AttributeOwner::project`.
    Projection,
}

/// One band of `perceived_as`: what another observer is told while the value is below `below`; the
/// last band has none.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Band {
    /// The upper bound, exclusive; absent on the last band.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub below: Option<Threshold>,
    /// The words.
    pub words: String,
}

/// One declaration, as the frozen catalogue holds it and `attributes-composed` states it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Declaration {
    /// Its qualified id.
    pub id: QualifiedId,
    /// Its group.
    pub group: String,
    /// Open tags consumers select by.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub facets: BTreeSet<String>,
    /// Its kind, vocabularies resolved.
    pub kind: Kind,
    /// Who may learn it.
    pub visibility: Visibility,
    /// Whether it changes.
    pub mutability: Mutability,
    /// How it is derived, for a `derived` attribute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derive: Option<Derive>,
    /// The System Pack that owns its value.
    pub owner: SystemId,
    /// Its name in prose.
    pub label: String,
    /// Its phrase: exactly one `{value}`.
    pub phrase: String,
    /// Words per value, where they differ from the default.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub words: BTreeMap<ValueSlug, String>,
    /// What another observer is told, when coarser than the value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perceived_as: Option<Vec<Band>>,
    /// The value when nothing else states one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<AttributeValue>,
    /// The order in which a byte-budgeted rendering keeps attributes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_rank: Option<u32>,
}

impl Declaration {
    /// The words of `value`: the declared ones, else the slug with `-` as spaces.
    pub fn words_of(&self, value: &ValueSlug) -> String {
        self.words
            .get(value)
            .cloned()
            .unwrap_or_else(|| value.default_words())
    }
}

// ---- The file formats ------------------------------------------------------------------------

/// The one format version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatOne;

impl<'de> Deserialize<'de> for FormatOne {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match u32::deserialize(deserializer)? {
            1 => Ok(Self),
            other => Err(de::Error::custom(format!(
                "format {other} is not one this build reads; it reads format 1"
            ))),
        }
    }
}

/// A group: one of [`GROUPS`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group(pub String);

impl<'de> Deserialize<'de> for Group {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        if GROUPS.contains(&text.as_str()) {
            Ok(Self(text))
        } else {
            Err(de::Error::custom(format!(
                "BadDeclaration: '{text}' is not a group; the groups are {}",
                GROUPS.join(", ")
            )))
        }
    }
}

/// A phrase: exactly one `{value}` and no other `{`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phrase(pub String);

/// Whether `phrase` names exactly one `{value}` and no other placeholder.
pub fn is_phrase(phrase: &str) -> bool {
    phrase.matches("{value}").count() == 1 && phrase.matches('{').count() == 1
}

impl<'de> Deserialize<'de> for Phrase {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        if is_phrase(&text) {
            Ok(Self(text))
        } else {
            Err(de::Error::custom(format!(
                "BadDeclaration: the phrase {text:?} must name exactly one {{value}} and no other \
                 placeholder"
            )))
        }
    }
}

/// A value list or a vocabulary: `[a, b]` or `{ vocabulary: <ns>.<name> }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawDomain {
    /// A closed list.
    Values(Vec<ValueSlug>),
    /// A vocabulary, resolved at build.
    Vocabulary(QualifiedId),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VocabularyRef {
    vocabulary: QualifiedId,
}

impl<'de> Deserialize<'de> for RawDomain {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct DomainVisitor;
        impl<'de> de::Visitor<'de> for DomainVisitor {
            type Value = RawDomain;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a list of value slugs, or { vocabulary: <namespace>.<name> }")
            }
            fn visit_seq<A: de::SeqAccess<'de>>(self, seq: A) -> Result<RawDomain, A::Error> {
                let values =
                    Vec::<ValueSlug>::deserialize(de::value::SeqAccessDeserializer::new(seq))?;
                distinct(&values).map_err(de::Error::custom)?;
                Ok(RawDomain::Values(values))
            }
            fn visit_map<A: de::MapAccess<'de>>(self, map: A) -> Result<RawDomain, A::Error> {
                let reference =
                    VocabularyRef::deserialize(de::value::MapAccessDeserializer::new(map))?;
                Ok(RawDomain::Vocabulary(reference.vocabulary))
            }
        }
        deserializer.deserialize_any(DomainVisitor)
    }
}

/// Refuses a list naming a value twice, or no value.
fn distinct(values: &[ValueSlug]) -> Result<(), String> {
    if values.is_empty() {
        return Err("BadDeclaration: a domain lists at least one value".to_owned());
    }
    let mut seen = BTreeSet::new();
    match values.iter().find(|value| !seen.insert(*value)) {
        Some(twice) => Err(format!(
            "BadDeclaration: the value '{twice}' is listed twice"
        )),
        None => Ok(()),
    }
}

/// `quantity: { unit, scale, min, max }`, the bounds still decimal text.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawQuantity {
    unit: Unit,
    scale: Scale,
    min: Loose,
    max: Loose,
}

/// A quantity's scale, `0..=6`.
#[derive(Debug, Clone, Copy)]
struct Scale(u8);

impl<'de> Deserialize<'de> for Scale {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let scale = u8::deserialize(deserializer)?;
        if scale > MAX_SCALE {
            return Err(de::Error::custom(format!(
                "BadDeclaration: a scale is at most {MAX_SCALE} decimals, not {scale}"
            )));
        }
        Ok(Self(scale))
    }
}

/// `set: { of | vocabulary, max }`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSet {
    #[serde(default)]
    of: Option<RawDomain>,
    #[serde(default)]
    vocabulary: Option<QualifiedId>,
    max: u16,
}

/// `text: { max_bytes }`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawText {
    max_bytes: MaxBytes,
}

#[derive(Debug, Clone, Copy)]
struct MaxBytes(u16);

impl<'de> Deserialize<'de> for MaxBytes {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let max = u16::deserialize(deserializer)?;
        if max == 0 || max > TEXT_MAX_BYTES {
            return Err(de::Error::custom(format!(
                "BadDeclaration: a text's max_bytes is 1 … {TEXT_MAX_BYTES}, not {max}"
            )));
        }
        Ok(Self(max))
    }
}

/// `kind:` as authored.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
pub enum RawKind {
    /// `{ category: [...] }` or `{ category: { vocabulary } }`.
    Category(RawDomain),
    /// `{ level: [...] }`.
    Level(Vec<ValueSlug>),
    /// `{ quantity: { unit, scale, min, max } }`.
    Quantity(RawQuantity),
    /// `flag`.
    Flag,
    /// `{ set: { of | vocabulary, max } }`.
    Set(RawSet),
    /// `{ text: { max_bytes } }`.
    Text(RawText),
    /// `instant`.
    Instant,
}

/// `derive:` as authored.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
pub enum RawDerive {
    /// `{ age: { from } }`.
    Age {
        /// The instant attribute.
        from: String,
    },
    /// `{ band: { from, below: [...] } }`.
    Band {
        /// The source attribute.
        from: String,
        /// The thresholds, as written.
        below: Vec<Loose>,
    },
    /// `projection`.
    Projection,
}

/// One `perceived_as` band as authored.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBand {
    #[serde(default)]
    below: Option<Loose>,
    words: String,
}

/// `perceived_as: { bands: [...] }`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPerceivedAs {
    bands: Vec<RawBand>,
}

/// One declaration as authored.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawDeclaration {
    group: Group,
    #[serde(default)]
    facets: BTreeSet<ValueSlug>,
    kind: RawKind,
    visibility: Visibility,
    mutability: Mutability,
    #[serde(default)]
    derive: Option<RawDerive>,
    #[serde(default)]
    owner: Option<String>,
    label: String,
    #[serde(default)]
    phrase: Option<Phrase>,
    #[serde(default)]
    words: BTreeMap<ValueSlug, String>,
    #[serde(default)]
    perceived_as: Option<RawPerceivedAs>,
    #[serde(default)]
    default: Option<Loose>,
    #[serde(default)]
    since: Option<String>,
    #[serde(default)]
    prompt_rank: Option<u32>,
}

/// How a pack reads one attribute: its kind, and whether it may be absent under a backend (S30-e).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Read {
    /// The kind the pack's handle names.
    pub kind: KindName,
    /// Whether the read may be absent (S30-e).
    #[serde(default, skip_serializing_if = "core::ops::Not::not")]
    pub optional: bool,
}

/// A read as authored: `<kind>` or `{ kind, optional }`.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RawRead {
    /// The kind alone.
    Kind(KindName),
    /// The kind and the optional flag.
    Full(Read),
}

/// `uses:` as authored.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawUses {
    /// Each id read, with its kind.
    #[serde(default)]
    pub reads: BTreeMap<String, RawRead>,
    /// Each id proposed.
    #[serde(default)]
    pub proposes: Vec<String>,
}

/// One `attributes/*.yaml` file, decoded.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttributesFile {
    #[allow(dead_code)]
    format: FormatOne,
    #[serde(default)]
    pub(crate) attributes: BTreeMap<Slug, Spanned<RawDeclaration>>,
    #[serde(default)]
    pub(crate) overrides: BTreeMap<String, Spanned<BTreeMap<String, Loose>>>,
    #[serde(default)]
    pub(crate) uses: Option<Spanned<RawUses>>,
}

/// One `vocabularies/*.yaml` file, decoded.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VocabulariesFile {
    #[allow(dead_code)]
    format: FormatOne,
    pub(crate) vocabularies: BTreeMap<Slug, VocabularyValues>,
}

/// A vocabulary's values: distinct, at least one.
#[derive(Debug, Clone)]
pub struct VocabularyValues(pub Vec<ValueSlug>);

impl<'de> Deserialize<'de> for VocabularyValues {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let values = Vec::<ValueSlug>::deserialize(deserializer)?;
        distinct(&values).map_err(de::Error::custom)?;
        Ok(Self(values))
    }
}

/// A file of either directory.
#[derive(Debug, Clone)]
pub enum DataFile {
    /// `attributes/*.yaml`.
    Attributes(AttributesFile),
    /// `vocabularies/*.yaml`.
    Vocabularies(VocabulariesFile),
}

// ---- One declaration, resolved ---------------------------------------------------------------

/// Why one declaration was refused: a field and the rule, or a value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DeclarationFault {
    /// A field breaks its rule.
    Field { field: &'static str, reason: String },
    /// A vocabulary the catalogue does not hold.
    UnknownVocabulary {
        vocabulary: QualifiedId,
        nearest: Vec<String>,
    },
    /// A `default` that does not fit the kind.
    Default(ValueError),
}

fn fault(field: &'static str, reason: impl Into<String>) -> DeclarationFault {
    DeclarationFault::Field {
        field,
        reason: reason.into(),
    }
}

impl RawDeclaration {
    /// The owner it names, if any.
    pub(crate) fn owner(&self) -> Option<&str> {
        self.owner.as_deref()
    }

    /// The declared `since`, if any.
    pub(crate) fn since(&self) -> Option<&str> {
        self.since.as_deref()
    }

    /// The kind, its vocabularies resolved through `vocabulary`.
    fn kind(
        &self,
        vocabulary: &dyn Fn(&QualifiedId) -> Result<Vec<ValueSlug>, DeclarationFault>,
    ) -> Result<Kind, DeclarationFault> {
        let domain = |raw: &RawDomain| match raw {
            RawDomain::Values(values) => Ok((values.clone(), None)),
            RawDomain::Vocabulary(id) => Ok((vocabulary(id)?, Some(id.clone()))),
        };
        Ok(match &self.kind {
            RawKind::Category(raw) => {
                let (values, vocabulary) = domain(raw)?;
                Kind::Category { values, vocabulary }
            }
            RawKind::Level(values) => {
                distinct(values).map_err(|reason| fault("kind", reason))?;
                Kind::Level {
                    values: values.clone(),
                }
            }
            RawKind::Quantity(raw) => {
                let scale = raw.scale.0;
                let bound = |field: &'static str, loose: &Loose| match loose {
                    Loose::Number(text) | Loose::Text(text) => ScaledInt::parse(text, scale)
                        .map(ScaledInt::value)
                        .map_err(|error| fault(field, error.to_string())),
                    other => Err(fault(
                        field,
                        format!("{} is not a number", other.describe()),
                    )),
                };
                let (min, max) = (bound("kind.min", &raw.min)?, bound("kind.max", &raw.max)?);
                if min > max {
                    return Err(fault("kind", "min is greater than max"));
                }
                Kind::Quantity(QuantityKind {
                    unit: raw.unit,
                    scale,
                    min,
                    max,
                })
            }
            RawKind::Flag => Kind::Flag,
            RawKind::Set(raw) => {
                let (values, vocabulary) = match (&raw.of, &raw.vocabulary) {
                    (Some(of), None) => domain(of)?,
                    (None, Some(id)) => (vocabulary(id)?, Some(id.clone())),
                    _ => {
                        return Err(fault(
                            "kind",
                            "a set takes exactly one of `of` and `vocabulary`",
                        ));
                    }
                };
                if raw.max == 0 {
                    return Err(fault("kind", "a set's max is at least 1"));
                }
                Kind::Set {
                    values,
                    vocabulary,
                    max: raw.max,
                }
            }
            RawKind::Text(raw) => Kind::Text {
                max_bytes: raw.max_bytes.0,
            },
            RawKind::Instant => Kind::Instant,
        })
    }

    /// The declaration with id `id`, owned by `owner`, its vocabularies resolved through
    /// `vocabulary`. Derived sources are checked by the catalogue, which sees every declaration.
    pub(crate) fn resolve(
        &self,
        id: QualifiedId,
        owner: SystemId,
        vocabulary: &dyn Fn(&QualifiedId) -> Result<Vec<ValueSlug>, DeclarationFault>,
    ) -> Result<Declaration, DeclarationFault> {
        let kind = self.kind(vocabulary)?;
        let derive = match (&self.derive, self.mutability) {
            (None, Mutability::Derived) => {
                return Err(fault("derive", "a derived attribute declares `derive:`"));
            }
            (Some(_), Mutability::Fixed | Mutability::Evolving) => {
                return Err(fault(
                    "derive",
                    "only a derived attribute declares `derive:`",
                ));
            }
            (None, _) => None,
            (Some(raw), Mutability::Derived) => Some(raw_derive(raw)?),
        };
        if let Some(values) = kind.values() {
            if let Some(stray) = self.words.keys().find(|value| !values.contains(value)) {
                return Err(fault(
                    "words",
                    format!("'{stray}' is not a value of {}", kind.domain()),
                ));
            }
        } else if !self.words.is_empty() {
            return Err(fault(
                "words",
                format!("a {} has no per-value words", kind.name()),
            ));
        }
        let perceived_as = self
            .perceived_as
            .as_ref()
            .map(|raw| bands(&kind, &raw.bands))
            .transpose()?;
        let default = match (&self.default, self.mutability) {
            (Some(_), Mutability::Derived) => {
                return Err(fault("default", "nothing states a derived value"));
            }
            (Some(loose), _) => Some(decode(&kind, loose).map_err(DeclarationFault::Default)?),
            (None, _) => None,
        };
        if self.label.is_empty() || self.label.len() > 80 {
            return Err(fault("label", "a label is 1 … 80 bytes"));
        }
        Ok(Declaration {
            id,
            group: self.group.0.clone(),
            facets: self.facets.iter().map(ToString::to_string).collect(),
            kind,
            visibility: self.visibility,
            mutability: self.mutability,
            derive,
            owner,
            label: self.label.clone(),
            phrase: self
                .phrase
                .as_ref()
                .map_or_else(|| "{value}".to_owned(), |phrase| phrase.0.clone()),
            words: self.words.clone(),
            perceived_as,
            default,
            prompt_rank: self.prompt_rank,
        })
    }
}

/// `derive:` with its source ids parsed; their kinds are the catalogue's to check.
fn raw_derive(raw: &RawDerive) -> Result<Derive, DeclarationFault> {
    let id = |text: &str| QualifiedId::new(text).map_err(|reason| fault("derive.from", reason));
    Ok(match raw {
        RawDerive::Age { from } => Derive::Age { from: id(from)? },
        // The thresholds are resolved by the catalogue, against the source's kind (`band_source`).
        RawDerive::Band { from, .. } => Derive::Band {
            from: id(from)?,
            below: Vec::new(),
        },
        RawDerive::Projection => Derive::Projection,
    })
}

/// The thresholds of `below`, against `source`'s kind: a quantity at its scale, or a level's values;
/// strictly increasing.
pub(crate) fn thresholds(
    source: &Kind,
    below: &[Loose],
    field: &'static str,
) -> Result<Vec<Threshold>, DeclarationFault> {
    let mut thresholds = Vec::with_capacity(below.len());
    for loose in below {
        let threshold = match source {
            Kind::Quantity(quantity) => match loose {
                Loose::Number(text) | Loose::Text(text) => Threshold::Quantity(
                    ScaledInt::parse(text, quantity.scale)
                        .map_err(|error| fault(field, error.to_string()))?,
                ),
                other => {
                    return Err(fault(
                        field,
                        format!("{} is not a number", other.describe()),
                    ));
                }
            },
            Kind::Level { .. } => match decode(source, loose) {
                Ok(AttributeValue::Level(level)) => Threshold::Level(level),
                Ok(_) | Err(_) => {
                    return Err(fault(
                        field,
                        format!("{} is not a value of {}", loose.describe(), source.domain()),
                    ));
                }
            },
            other => {
                return Err(fault(field, format!("a {} has no bands", other.name())));
            }
        };
        thresholds.push(threshold);
    }
    let increasing = thresholds
        .windows(2)
        .all(|pair| match (&pair[0], &pair[1]) {
            (Threshold::Quantity(a), Threshold::Quantity(b)) => a < b,
            (Threshold::Level(a), Threshold::Level(b)) => a.rank < b.rank,
            _ => false,
        });
    if !increasing {
        return Err(fault(field, "thresholds are strictly increasing"));
    }
    Ok(thresholds)
}

/// `perceived_as` bands against the attribute's own kind: every band but the last has a `below`.
fn bands(kind: &Kind, raw: &[RawBand]) -> Result<Vec<Band>, DeclarationFault> {
    let Some((last, rest)) = raw.split_last() else {
        return Err(fault("perceived_as", "at least one band"));
    };
    if last.below.is_some() || rest.iter().any(|band| band.below.is_none()) {
        return Err(fault(
            "perceived_as",
            "every band but the last states `below`, and the last states none",
        ));
    }
    let below: Vec<Loose> = rest.iter().filter_map(|band| band.below.clone()).collect();
    let thresholds = thresholds(kind, &below, "perceived_as")?;
    let mut bands: Vec<Band> = thresholds
        .into_iter()
        .zip(rest)
        .map(|(threshold, band)| Band {
            below: Some(threshold),
            words: band.words.clone(),
        })
        .collect();
    bands.push(Band {
        below: None,
        words: last.words.clone(),
    });
    Ok(bands)
}

/// The raw thresholds of a `band` derive, for the catalogue to resolve against the source.
pub(crate) fn band_source(raw: &RawDeclaration) -> Option<&[Loose]> {
    match &raw.derive {
        Some(RawDerive::Band { below, .. }) => Some(below),
        _ => None,
    }
}
