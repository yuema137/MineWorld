//! The typed reader of disclosed attributes, for decision models (S30-f §5.2, D-f-3), and the one
//! composition every consumer applies to them (§5.4).
//!
//! ```text
//! block        { schema: 2, entries } — the payload of `traits`, or the member `attributes` of an
//!              owner's component payload (D-f-1); any other shape is skipped whole
//! Disclosed    the entries of every block of one perceived entity, merged in rank order
//! compose      select by facet, join with ", ", cut at an entry boundary with " …"
//! inspect      every entry with text, one line each
//! sheet        every entry, "label: text" or "label: <typed value>"
//! ```
//!
//! A `text` attribute has no typed accessor: no decision model reads text (I-S30-5). Python
//! (`mineworld_sdk.attributes`) and GDScript (`mineworld/attributes.gd`) implement the same
//! composition; one hand-written golden (`tests/fixtures/composition.json`) holds all three equal.

use std::collections::BTreeSet;

use mineworld_contracts::{Component, PerceivedEntity};
use serde_json::Value;

use crate::id::{QualifiedId, ValueSlug};
use crate::render::{Block, Entry, SCHEMA, rank_order};
use crate::traits::Traits;
use crate::value::{AttributeValue, ScaledInt};

/// The member of an owner's component payload that carries its block.
pub const OWNER_MEMBER: &str = "attributes";

/// Separator between the texts of a composition.
pub const SEPARATOR: &str = ", ";

/// What a cut composition ends with.
pub const CUT_MARK: &str = " …";

/// The block `payload` is, if it is one this reader can read.
fn block(payload: &Value) -> Option<Vec<Entry>> {
    if payload.get("schema").and_then(Value::as_u64) != Some(u64::from(SCHEMA)) {
        return None;
    }
    serde_json::from_value::<Block>(payload.clone())
        .ok()
        .map(|block| block.entries)
}

/// The attributes a perceived entity's components disclose, merged from every attribute block.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Disclosed {
    entries: Vec<Entry>,
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
        let mut entries: Vec<Entry> = payloads
            .into_iter()
            .filter_map(|(is_traits, payload)| {
                if is_traits {
                    block(payload)
                } else {
                    payload.get(OWNER_MEMBER).and_then(block)
                }
            })
            .flatten()
            .collect();
        entries.sort_by(rank_order);
        Self { entries }
    }

    /// The entries, in rank order.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
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

/// The texts of `entries` carrying `facet` (every entry when `None`), in order, empty texts skipped.
fn texts<'a>(entries: &'a [Entry], facet: Option<&str>) -> Vec<&'a str> {
    entries
        .iter()
        .filter(|entry| facet.is_none_or(|facet| entry.facets.contains(facet)))
        .filter_map(|entry| entry.text.as_deref())
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

/// The composition of §5.4: select by `facet`, join with [`SEPARATOR`], and cut at `budget` bytes at
/// an entry boundary with [`CUT_MARK`]; a first entry longer than the budget alone is cut at a
/// character boundary and ends with `…`.
pub fn compose(entries: &[Entry], facet: Option<&str>, budget: Option<usize>) -> String {
    let texts = texts(entries, facet);
    let joined = texts.join(SEPARATOR);
    let Some(budget) = budget else {
        return joined;
    };
    if joined.len() <= budget {
        return joined;
    }
    let mut length = 0;
    let mut kept = 0;
    for (index, text) in texts.iter().enumerate() {
        let next = length + text.len() + if index == 0 { 0 } else { SEPARATOR.len() };
        if next + CUT_MARK.len() > budget {
            break;
        }
        length = next;
        kept = index + 1;
    }
    if kept > 0 {
        return format!("{}{CUT_MARK}", texts[..kept].join(SEPARATOR));
    }
    let ellipsis = "…";
    let first = texts.first().copied().unwrap_or_default();
    format!(
        "{}{ellipsis}",
        cut(first, budget.saturating_sub(ellipsis.len()))
    )
}

/// The inspect list: every entry's text, one line each.
pub fn inspect(entries: &[Entry]) -> Vec<String> {
    texts(entries, None)
        .into_iter()
        .map(str::to_owned)
        .collect()
}

/// A typed value as the own sheet shows it when it renders no text.
fn typed(value: &AttributeValue) -> String {
    match value {
        AttributeValue::Category(slug) => slug.as_str().to_owned(),
        AttributeValue::Level(level) => level.slug.as_str().to_owned(),
        AttributeValue::Quantity(quantity) => quantity.to_string(),
        AttributeValue::Flag(flag) => if *flag { "yes" } else { "no" }.to_owned(),
        AttributeValue::Set(members) => members
            .iter()
            .map(ValueSlug::as_str)
            .collect::<Vec<_>>()
            .join(SEPARATOR),
        AttributeValue::Text(text) => text.as_str().to_owned(),
        AttributeValue::Instant(at) => at.seconds().to_string(),
    }
}

/// The own sheet: every entry, `label: text`, or `label: <typed value>` when it renders no text; an
/// entry that shows nothing (no text and no value, or an empty set) is left out.
pub fn sheet(entries: &[Entry]) -> Vec<String> {
    entries
        .iter()
        .filter_map(|entry| {
            let shown = match (entry.text.as_deref(), entry.value.as_ref()) {
                (Some(text), _) if !text.is_empty() => text.to_owned(),
                (_, Some(value)) => typed(value),
                _ => return None,
            };
            (!shown.is_empty()).then(|| format!("{}: {shown}", entry.label))
        })
        .collect()
}

#[cfg(test)]
mod tests;
