//! Rendering and the visibility filter (`DECISIONS.md` `ARC-122` items 4, 6; `MODULE_SPEC.md` §4.1
//! "Disclosure", "Rendering"): one function serves the pack and every owner, so no owner writes its
//! own visibility rules (`ARC-124` item 5).
//!
//! ```text
//! holder          every value except a hidden one, exact
//! other observer  perceivable values only; a value with perceived_as is disclosed as its band's
//!                 words alone — no value — so "tall" never carries "1.87"
//! phrase          phrase.replacen("{value}", words, 1); a quantity's words are its number at its
//!                 scale (F-S30c-5); a set's are "a, b and c" in domain order; a flag's are its label in
//!                 lower case when true and nothing when false (F-S30c-6); an empty word renders no text
//! bound           no text exceeds 160 bytes; a longer one is cut at a character boundary with "…"
//! locales         every locale the world's label packs offer, by the same rules with that locale's
//!                 pieces (S30-k, `ARC-129`); an entry missing any piece it needs carries nothing for
//!                 that locale, and its reader shows it in English, whole
//! ```

use std::collections::{BTreeMap, BTreeSet};

use mineworld_authoring::Locale;
use mineworld_contracts::SystemId;
use serde::{Deserialize, Serialize};

use crate::catalogue::Catalogue;
use crate::declaration::{Declaration, Visibility};
use crate::derive::band_index;
use crate::id::{QualifiedId, ValueSlug};
use crate::kind::{Kind, KindName};
use crate::labels::{AttributeLabels, Labels, LocaleLabels, ReaderWords};
use crate::value::{AttributeValue, ScaledInt};

/// The schema of a disclosed payload: 2 added each entry's `label`, `facets` and `rank`, and ordered
/// entries by rank (S30-f, Q-S30f-1); 3 adds each entry's `texts` and, in the holder's view, `labels`
/// per offered locale, and the block's reader `words` per offered locale (S30-k, Q-S30k-1).
pub const SCHEMA: u32 = 3;

/// The most bytes one entry's text may have.
pub const TEXT_BOUND: usize = 160;

/// Whose observation a disclosure is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// The person's own.
    Holder,
    /// Anyone else's.
    Other,
}

impl View {
    /// The view of `observer` on `subject`.
    pub fn of<T: PartialEq>(observer: T, subject: T) -> Self {
        if observer == subject {
            Self::Holder
        } else {
            Self::Other
        }
    }
}

/// One disclosed attribute. A later schema adds fields under a new `schema` number; a reader of this one
/// refuses a field it does not know, so a block is read whole or not at all (as the Python and GDScript
/// readers do).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    /// The attribute.
    pub attribute: QualifiedId,
    /// Its kind's name.
    pub kind: KindName,
    /// Its label in prose.
    pub label: String,
    /// The facets a consumer selects it by, sorted.
    pub facets: BTreeSet<String>,
    /// Its `prompt_rank`: the order a byte-budgeted rendering keeps entries in; absent sorts last.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<u32>,
    /// The exact value; absent when only words are disclosed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<AttributeValue>,
    /// The rendered phrase; absent when it renders nothing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// The label per offered locale this entry is rendered in, in the holder's view only.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub labels: BTreeMap<Locale, String>,
    /// The text per offered locale this entry is rendered in, when it renders one.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub texts: BTreeMap<Locale, String>,
}

/// The order of entries in a block: by `(rank, attribute)`, an absent rank after every present one.
pub fn rank_order(left: &Entry, right: &Entry) -> std::cmp::Ordering {
    (left.rank.is_none(), left.rank, &left.attribute).cmp(&(
        right.rank.is_none(),
        right.rank,
        &right.attribute,
    ))
}

/// A disclosed payload, an attribute block: `{ schema, entries }`, entries in rank order. The
/// `traits` component's payload is one; an owner embeds one under its payload's `attributes` member.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Block {
    /// [`SCHEMA`].
    pub schema: u32,
    /// The words a reader composes with, per offered locale; empty when only English is offered.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub words: BTreeMap<Locale, ReaderWords>,
    /// The entries.
    pub entries: Vec<Entry>,
}

/// The words of `slug` in `declaration`.
fn word(declaration: &Declaration, slug: &ValueSlug) -> String {
    declaration.words_of(slug)
}

/// "a, b and c".
fn joined(words: &[String]) -> String {
    match words {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// The words `value` renders as under `declaration`; empty for a false flag or an empty set.
pub fn words(declaration: &Declaration, value: &AttributeValue) -> String {
    match value {
        AttributeValue::Category(slug) => word(declaration, slug),
        AttributeValue::Level(level) => word(declaration, &level.slug),
        AttributeValue::Quantity(quantity) => quantity.to_string(),
        AttributeValue::Flag(true) => declaration.label.to_lowercase(),
        AttributeValue::Flag(false) => String::new(),
        AttributeValue::Set(members) => {
            let ordered: Vec<String> = declaration
                .kind
                .values()
                .unwrap_or_default()
                .iter()
                .filter(|value| members.contains(*value))
                .map(|slug| word(declaration, slug))
                .collect();
            joined(&ordered)
        }
        AttributeValue::Text(text) => text.as_str().to_owned(),
        // An instant renders no text (Q-S30f-4): a date needs the world's calendar, which this
        // crate cannot read; its typed value still reaches the holder.
        AttributeValue::Instant(_) => String::new(),
    }
}

/// The phrase with `words` in it, bounded; `None` when the words are empty. A quantity of exactly
/// one unit takes `phrase_one` when the declaration has one.
pub fn phrase(declaration: &Declaration, value: &AttributeValue, words: &str) -> Option<String> {
    if words.is_empty() {
        return None;
    }
    let template = match (value, &declaration.phrase_one) {
        (AttributeValue::Quantity(quantity), Some(one)) if is_one_unit(*quantity) => one,
        _ => &declaration.phrase,
    };
    Some(bounded(template.replacen("{value}", words, 1)))
}

/// Whether `quantity` is exactly one unit: 1 at scale 0, 1.000 at scale 3, and so on.
fn is_one_unit(quantity: ScaledInt) -> bool {
    10_i64.checked_pow(u32::from(quantity.scale())) == Some(quantity.value())
}

/// `text` cut to [`TEXT_BOUND`] bytes at a character boundary, with "…".
fn bounded(text: String) -> String {
    if text.len() <= TEXT_BOUND {
        return text;
    }
    let ellipsis = "…";
    let mut cut = TEXT_BOUND - ellipsis.len();
    while !text.is_char_boundary(cut) {
        cut -= 1;
    }
    format!("{}{ellipsis}", &text[..cut])
}

/// The band of `value` another observer is told, when the declaration has bands.
fn band_of(declaration: &Declaration, value: &AttributeValue) -> Option<usize> {
    let bands = declaration.perceived_as.as_ref()?;
    let below: Vec<_> = bands.iter().filter_map(|band| band.below.clone()).collect();
    band_index(value, &below).filter(|index| *index < bands.len())
}

/// One entry for `view`, or `None` when the view may not see it; with `labels`, rendered in every
/// offered locale too (§5.4).
pub fn entry(
    declaration: &Declaration,
    value: &AttributeValue,
    view: View,
    labels: Option<&Labels>,
) -> Option<Entry> {
    let visible = match view {
        View::Holder => declaration.visibility != Visibility::Hidden,
        View::Other => declaration.visibility == Visibility::Perceivable,
    };
    if !visible {
        return None;
    }
    let shell = |value: Option<AttributeValue>, text: Option<String>| Entry {
        attribute: declaration.id.clone(),
        kind: declaration.kind.name(),
        label: declaration.label.clone(),
        facets: declaration.facets.clone(),
        rank: declaration.prompt_rank,
        value,
        text,
        labels: BTreeMap::new(),
        texts: BTreeMap::new(),
    };
    let mut entry = if view == View::Other
        && let Some(bands) = &declaration.perceived_as
    {
        let band = bands.get(band_of(declaration, value)?)?;
        shell(None, phrase_of_band(&band.words))
    } else {
        shell(
            Some(value.clone()),
            phrase(declaration, value, &words(declaration, value)),
        )
    };
    for (locale, locale_labels) in labels.into_iter().flat_map(Labels::iter) {
        let Some(localised) = localised(declaration, value, view, locale_labels) else {
            continue;
        };
        if view == View::Holder {
            entry.labels.insert(locale.clone(), localised.label);
        }
        if let Some(text) = localised.text {
            entry.texts.insert(locale.clone(), text);
        }
    }
    Some(entry)
}

/// One entry rendered in one locale: its label, and its text when it renders one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Localised {
    /// The label.
    pub label: String,
    /// The text; `None` when the entry renders none (a false flag, an empty set, an instant).
    pub text: Option<String>,
}

/// `declaration`'s `value` for `view` in one locale, by §5.4's rules; `None` when the locale lacks any
/// piece the entry needs, so the entry is shown in English, whole (D-k-2). Visibility is the caller's:
/// this renders, it does not filter.
pub fn localised(
    declaration: &Declaration,
    value: &AttributeValue,
    view: View,
    labels: &LocaleLabels,
) -> Option<Localised> {
    let pieces = labels.attributes.get(&declaration.id)?;
    let label = pieces.label.clone()?;
    if view == View::Other && declaration.perceived_as.is_some() {
        let words = pieces
            .perceived_as
            .as_ref()?
            .get(band_of(declaration, value)?)?;
        return Some(Localised {
            label,
            text: phrase_of_band(words),
        });
    }
    let text = match localised_words(declaration, value, pieces, labels)? {
        None => None,
        Some(words) => {
            let template = match (&declaration.kind, &pieces.phrase) {
                (_, Some(phrase)) => phrase.as_str(),
                // A flag's words may stand alone (the file's `flag` is the whole text); every other
                // kind needs its phrase.
                (Kind::Flag, None) => "{value}",
                (_, None) => return None,
            };
            Some(bounded(template.replacen("{value}", &words, 1)))
        }
    };
    Some(Localised { label, text })
}

/// The words of `value` in one locale: `Some(None)` when it renders no text, `None` when a word is
/// missing. Never a slug default: a missing word is missing.
fn localised_words(
    declaration: &Declaration,
    value: &AttributeValue,
    pieces: &AttributeLabels,
    labels: &LocaleLabels,
) -> Option<Option<String>> {
    Some(Some(match value {
        AttributeValue::Category(slug) => pieces.words.get(slug)?.clone(),
        AttributeValue::Level(level) => pieces.words.get(&level.slug)?.clone(),
        AttributeValue::Quantity(quantity) => quantity.to_string(),
        AttributeValue::Flag(true) => pieces.flag.clone()?,
        AttributeValue::Flag(false) | AttributeValue::Instant(_) => return Some(None),
        AttributeValue::Set(members) => {
            let ordered = declaration
                .kind
                .values()
                .unwrap_or_default()
                .iter()
                .filter(|value| members.contains(*value))
                .map(|slug| pieces.words.get(slug).cloned())
                .collect::<Option<Vec<String>>>()?;
            match ordered.as_slice() {
                [] => return Some(None),
                [one] => one.clone(),
                [rest @ .., last] => format!(
                    "{}{}{last}",
                    rest.join(labels.words.list.as_str()),
                    labels.words.and.as_str()
                ),
            }
        }
        AttributeValue::Text(text) if text.as_str().is_empty() => return Some(None),
        // Authored text is world content, shown as written (F-S30k-7).
        AttributeValue::Text(text) => text.as_str().to_owned(),
    }))
}

/// A band's words, as its whole text.
fn phrase_of_band(words: &str) -> Option<String> {
    (!words.is_empty()).then(|| bounded(words.to_owned()))
}

/// The disclosure of `owner`'s attributes for `view`: each declaration `owner` owns, in rank order,
/// with the value `value_of` gives it (stored, derived or projected), through the one filter and
/// renderer, in English and every locale `labels` offers. `None` when nothing is disclosable.
pub fn disclose(
    catalogue: &Catalogue,
    owner: &SystemId,
    value_of: impl Fn(&Declaration) -> Option<AttributeValue>,
    view: View,
    labels: Option<&Labels>,
) -> Option<Block> {
    let mut entries: Vec<Entry> = catalogue
        .attributes()
        .filter(|declaration| declaration.owner == *owner)
        .filter_map(|declaration| entry(declaration, &value_of(declaration)?, view, labels))
        .collect();
    entries.sort_by(rank_order);
    let words = labels
        .into_iter()
        .flat_map(Labels::iter)
        .map(|(locale, labels)| (locale.clone(), labels.words.for_readers()))
        .collect();
    (!entries.is_empty()).then_some(Block {
        schema: SCHEMA,
        words,
        entries,
    })
}

/// Whether `kind` can carry `perceived_as` bands.
pub const fn banded(kind: &Kind) -> bool {
    matches!(kind, Kind::Quantity(_) | Kind::Level { .. })
}

#[cfg(test)]
mod tests;
