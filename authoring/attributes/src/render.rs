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
//! ```

use mineworld_contracts::SystemId;
use serde::{Deserialize, Serialize};

use crate::catalogue::Catalogue;
use crate::declaration::{Declaration, Visibility};
use crate::derive::band_index;
use crate::id::{QualifiedId, ValueSlug};
use crate::kind::{Kind, KindName};
use crate::value::AttributeValue;

/// The schema of a disclosed payload.
pub const SCHEMA: u32 = 1;

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

/// One disclosed attribute. A struct with optional fields, so a later schema adds fields without
/// changing the ones here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// The attribute.
    pub attribute: QualifiedId,
    /// Its kind's name.
    pub kind: KindName,
    /// The exact value; absent when only words are disclosed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<AttributeValue>,
    /// The rendered phrase; absent when it renders nothing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// A disclosed payload: `{ schema, entries }`, entries in catalogue order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Disclosed {
    /// [`SCHEMA`].
    pub schema: u32,
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
        AttributeValue::Instant(at) => at.seconds().to_string(),
    }
}

/// The phrase with `words` in it, bounded; `None` when the words are empty.
pub fn phrase(declaration: &Declaration, words: &str) -> Option<String> {
    if words.is_empty() {
        return None;
    }
    Some(bounded(declaration.phrase.replacen("{value}", words, 1)))
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

/// One entry for `view`, or `None` when the view may not see it.
pub fn entry(declaration: &Declaration, value: &AttributeValue, view: View) -> Option<Entry> {
    let visible = match view {
        View::Holder => declaration.visibility != Visibility::Hidden,
        View::Other => declaration.visibility == Visibility::Perceivable,
    };
    if !visible {
        return None;
    }
    if view == View::Other
        && let Some(bands) = &declaration.perceived_as
    {
        let below: Vec<_> = bands.iter().filter_map(|band| band.below.clone()).collect();
        let band = band_index(value, &below).and_then(|index| bands.get(index))?;
        return Some(Entry {
            attribute: declaration.id.clone(),
            kind: declaration.kind.name(),
            value: None,
            text: phrase_of_band(&band.words),
        });
    }
    Some(Entry {
        attribute: declaration.id.clone(),
        kind: declaration.kind.name(),
        value: Some(value.clone()),
        text: phrase(declaration, &words(declaration, value)),
    })
}

/// A band's words, as its whole text.
fn phrase_of_band(words: &str) -> Option<String> {
    (!words.is_empty()).then(|| bounded(words.to_owned()))
}

/// The disclosure of `owner`'s attributes for `view`: each declaration `owner` owns, in catalogue order,
/// with the value `value_of` gives it (stored, derived or projected), through the one filter and
/// renderer. `None` when nothing is disclosable.
pub fn disclose(
    catalogue: &Catalogue,
    owner: &SystemId,
    value_of: impl Fn(&Declaration) -> Option<AttributeValue>,
    view: View,
) -> Option<Disclosed> {
    let entries: Vec<Entry> = catalogue
        .attributes()
        .filter(|declaration| declaration.owner == *owner)
        .filter_map(|declaration| entry(declaration, &value_of(declaration)?, view))
        .collect();
    (!entries.is_empty()).then_some(Disclosed {
        schema: SCHEMA,
        entries,
    })
}

/// Whether `kind` can carry `perceived_as` bands.
pub const fn banded(kind: &Kind) -> bool {
    matches!(kind, Kind::Quantity(_) | Kind::Level { .. })
}

#[cfg(test)]
mod tests;
