//! The typed reader of disclosed attributes, for decision models (S30-f §5.2, D-f-3), and the one
//! composition every consumer applies to them (§5.4).
//!
//! ```text
//! block        { schema: 3, words?, entries } — the payload of `traits`, or the member `attributes`
//!              of an owner's component payload (D-f-1); any other shape is skipped whole
//! Disclosed    the entries of every block of one perceived entity, merged in rank order, and the
//!              first block's reader words per offered locale
//! Reading      the reader's locale among the offered ones (its own; else the one offered of its
//!              language; else English) and the words it composes with (S30-k §5.6)
//! compose      select by facet, join with the reading's `join`, cut at an entry boundary with `more`
//! inspect      every entry with text, one line each
//! sheet        every entry, "label<pair>text" or "label<pair><typed value>"
//! ```
//!
//! A `text` attribute has no typed accessor: no decision model reads text (I-S30-5). Python
//! (`mineworld_sdk.attributes`) and GDScript (`mineworld/attributes.gd`) implement the same
//! composition; one hand-written golden (`tests/fixtures/composition.json`) holds all three equal.

use std::collections::{BTreeMap, BTreeSet};

use mineworld_authoring::Locale;
use mineworld_contracts::{Component, PerceivedEntity};
use serde_json::Value;

use crate::id::{QualifiedId, ValueSlug};
use crate::labels::ReaderWords;
use crate::render::{Block, Entry, SCHEMA, rank_order};
use crate::traits::Traits;
use crate::value::{AttributeValue, ScaledInt};

/// The member of an owner's component payload that carries its block.
pub const OWNER_MEMBER: &str = "attributes";

/// Separator between the texts of an English composition.
pub const SEPARATOR: &str = ", ";

/// What a cut English composition ends with.
pub const CUT_MARK: &str = " …";

/// The block `payload` is, if it is one this reader can read.
fn block(payload: &Value) -> Option<Block> {
    if payload.get("schema").and_then(Value::as_u64) != Some(u64::from(SCHEMA)) {
        return None;
    }
    serde_json::from_value::<Block>(payload.clone()).ok()
}

/// The attributes a perceived entity's components disclose, merged from every attribute block.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Disclosed {
    entries: Vec<Entry>,
    words: BTreeMap<Locale, ReaderWords>,
}

/// How one reader reads a [`Disclosed`]: the locale it reads in, `None` for English, and the words it
/// composes with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reading {
    locale: Option<Locale>,
    words: ReaderWords,
}

impl Reading {
    /// English: S30-f's output, byte for byte.
    pub fn english() -> Self {
        Self {
            locale: None,
            words: ReaderWords::english(),
        }
    }

    /// A reader wanting `wanted`, among the `offered` locales and their words.
    pub fn new(wanted: &Locale, offered: &BTreeMap<Locale, ReaderWords>) -> Self {
        match wanted.select(offered.keys()) {
            Some(locale) => Self {
                locale: Some(locale.clone()),
                words: offered[locale].clone(),
            },
            None => Self::english(),
        }
    }

    /// The locale read in; `None` for English.
    pub fn locale(&self) -> Option<&Locale> {
        self.locale.as_ref()
    }

    /// `entry`'s text in this reading: its localised text, else its English one.
    pub fn text<'e>(&self, entry: &'e Entry) -> Option<&'e str> {
        self.locale
            .as_ref()
            .and_then(|locale| entry.texts.get(locale))
            .or(entry.text.as_ref())
            .map(String::as_str)
    }

    /// `entry`'s localised label, when the holder's view carries one in this reading.
    fn label<'e>(&self, entry: &'e Entry) -> Option<&'e str> {
        self.locale
            .as_ref()
            .and_then(|locale| entry.labels.get(locale))
            .map(String::as_str)
    }
}

impl Disclosed {
    /// The merged blocks of `entity`; never fails: an unknown shape is skipped.
    pub fn of(entity: &PerceivedEntity<Value>) -> Self {
        Self::from_payloads(entity.components().iter().map(|record| {
            (
                *record.component_type() == Traits::COMPONENT_TYPE,
                record.payload(),
            )
        }))
    }

    /// The merged blocks of component payloads, each with whether it is the `traits` payload.
    pub fn from_payloads<'a>(payloads: impl IntoIterator<Item = (bool, &'a Value)>) -> Self {
        let blocks: Vec<Block> = payloads
            .into_iter()
            .filter_map(|(is_traits, payload)| {
                if is_traits {
                    block(payload)
                } else {
                    payload.get(OWNER_MEMBER).and_then(block)
                }
            })
            .collect();
        // Every block one server rendered carries the same words; a reader takes the first block's.
        let words = blocks
            .first()
            .map(|block| block.words.clone())
            .unwrap_or_default();
        let mut entries: Vec<Entry> = blocks.into_iter().flat_map(|block| block.entries).collect();
        entries.sort_by(rank_order);
        Self { entries, words }
    }

    /// The entries, in rank order.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// The reader words per offered locale.
    pub fn words(&self) -> &BTreeMap<Locale, ReaderWords> {
        &self.words
    }

    /// How a reader wanting `wanted` reads these entries.
    pub fn reading(&self, wanted: &Locale) -> Reading {
        Reading::new(wanted, &self.words)
    }

    fn value(&self, id: &QualifiedId) -> Option<&AttributeValue> {
        self.entries
            .iter()
            .find(|entry| entry.attribute == *id)
            .and_then(|entry| entry.value.as_ref())
    }

    /// A level's value; `None` when absent, undisclosed or of another kind.
    pub fn level(&self, id: &QualifiedId) -> Option<&ValueSlug> {
        match self.value(id)? {
            AttributeValue::Level(level) => Some(&level.slug),
            _ => None,
        }
    }

    /// A category's value.
    pub fn category(&self, id: &QualifiedId) -> Option<&ValueSlug> {
        match self.value(id)? {
            AttributeValue::Category(slug) => Some(slug),
            _ => None,
        }
    }

    /// A quantity's value.
    pub fn quantity(&self, id: &QualifiedId) -> Option<ScaledInt> {
        match self.value(id)? {
            AttributeValue::Quantity(quantity) => Some(*quantity),
            _ => None,
        }
    }

    /// A flag's value.
    pub fn flag(&self, id: &QualifiedId) -> Option<bool> {
        match self.value(id)? {
            AttributeValue::Flag(flag) => Some(*flag),
            _ => None,
        }
    }

    /// A set's value.
    pub fn set(&self, id: &QualifiedId) -> Option<&BTreeSet<ValueSlug>> {
        match self.value(id)? {
            AttributeValue::Set(members) => Some(members),
            _ => None,
        }
    }
}

/// The texts of `entries` carrying `facet` (every entry when `None`) in `reading`, in order, empty
/// texts skipped.
fn texts<'a>(reading: &Reading, entries: &'a [Entry], facet: Option<&str>) -> Vec<&'a str> {
    entries
        .iter()
        .filter(|entry| facet.is_none_or(|facet| entry.facets.contains(facet)))
        .filter_map(|entry| reading.text(entry))
        .filter(|text| !text.is_empty())
        .collect()
}

/// `text` cut to at most `budget` bytes at a character boundary.
fn cut(text: &str, budget: usize) -> &str {
    let mut end = budget.min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

/// The English composition: [`compose_in`] with [`Reading::english`].
pub fn compose(entries: &[Entry], facet: Option<&str>, budget: Option<usize>) -> String {
    compose_in(&Reading::english(), entries, facet, budget)
}

/// The composition of §5.4 in `reading`: select by `facet`, join with its `join`, and cut at `budget`
/// bytes at an entry boundary with its `more`; a first entry longer than the budget alone is cut at a
/// character boundary and ends with `…`.
pub fn compose_in(
    reading: &Reading,
    entries: &[Entry],
    facet: Option<&str>,
    budget: Option<usize>,
) -> String {
    let separator = reading.words.join.as_str();
    let mark = reading.words.more.as_str();
    let texts = texts(reading, entries, facet);
    let joined = texts.join(separator);
    let Some(budget) = budget else {
        return joined;
    };
    if joined.len() <= budget {
        return joined;
    }
    let mut length = 0;
    let mut kept = 0;
    for (index, text) in texts.iter().enumerate() {
        let next = length + text.len() + if index == 0 { 0 } else { separator.len() };
        if next + mark.len() > budget {
            break;
        }
        length = next;
        kept = index + 1;
    }
    if kept > 0 {
        return format!("{}{mark}", texts[..kept].join(separator));
    }
    let ellipsis = "…";
    let first = texts.first().copied().unwrap_or_default();
    format!(
        "{}{ellipsis}",
        cut(first, budget.saturating_sub(ellipsis.len()))
    )
}

/// The English inspect list: [`inspect_in`] with [`Reading::english`].
pub fn inspect(entries: &[Entry]) -> Vec<String> {
    inspect_in(&Reading::english(), entries)
}

/// The inspect list in `reading`: every entry's text, one line each.
pub fn inspect_in(reading: &Reading, entries: &[Entry]) -> Vec<String> {
    texts(reading, entries, None)
        .into_iter()
        .map(str::to_owned)
        .collect()
}

/// A typed value as the own sheet shows it when it renders no text, with `words`' flag words and
/// `join`.
fn typed(value: &AttributeValue, words: &ReaderWords) -> String {
    match value {
        AttributeValue::Category(slug) => slug.as_str().to_owned(),
        AttributeValue::Level(level) => level.slug.as_str().to_owned(),
        AttributeValue::Quantity(quantity) => quantity.to_string(),
        AttributeValue::Flag(flag) => if *flag { &words.yes } else { &words.no }.clone(),
        AttributeValue::Set(members) => members
            .iter()
            .map(ValueSlug::as_str)
            .collect::<Vec<_>>()
            .join(&words.join),
        AttributeValue::Text(text) => text.as_str().to_owned(),
        AttributeValue::Instant(at) => at.seconds().to_string(),
    }
}

/// The English own sheet: [`sheet_in`] with [`Reading::english`].
pub fn sheet(entries: &[Entry]) -> Vec<String> {
    sheet_in(&Reading::english(), entries)
}

/// The own sheet in `reading`: every entry, `label<pair>text`, or `label<pair><typed value>` when it
/// renders no text; an entry that shows nothing (no text and no value, or an empty set) is left out.
/// An entry the holder's view carries in the reading's locale is shown in it, with its words; any
/// other is shown in English, whole (D-k-2).
pub fn sheet_in(reading: &Reading, entries: &[Entry]) -> Vec<String> {
    let english = ReaderWords::english();
    entries
        .iter()
        .filter_map(|entry| {
            let (label, text, words) = match reading.label(entry) {
                Some(label) => (label, reading.text(entry), &reading.words),
                None => (entry.label.as_str(), entry.text.as_deref(), &english),
            };
            let shown = match (text, entry.value.as_ref()) {
                (Some(text), _) if !text.is_empty() => text.to_owned(),
                (_, Some(value)) => typed(value, words),
                _ => return None,
            };
            (!shown.is_empty()).then(|| format!("{label}{}{shown}", words.pair))
        })
        .collect()
}

#[cfg(test)]
mod tests;
