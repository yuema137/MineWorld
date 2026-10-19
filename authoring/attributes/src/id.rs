//! Identifiers: an attribute's namespace and slug, its qualified id, and the slug of one value
//! (`DECISIONS.md` `ARC-122` item 1; `MODULE_SPEC.md` §4.1 "Namespaces and short slugs").
//!
//! ```text
//! Namespace     a pack id: 1-64 bytes of a-z, 0-9 and -, starting with a letter; no '.'
//! Slug          ^[a-z][a-z0-9_.-]{0,63}$ — may contain '.' (skill.cooking)
//! QualifiedId   <namespace>.<slug>, split at its first '.'
//! ValueSlug     ^[a-z0-9][a-z0-9_.-]{0,63}$ — a value of a domain (early-bird, 20s, prices.fuel)
//! ```

use core::fmt;

use serde::de::{Deserializer, Error as _};
use serde::{Deserialize, Serialize, Serializer};

/// Whether `text` is 1-64 bytes, starts with a byte `first` admits, and continues with bytes `rest`
/// admits.
const fn shaped(text: &str, first: fn(u8) -> bool, rest: fn(u8) -> bool) -> bool {
    let bytes = text.as_bytes();
    if bytes.is_empty() || bytes.len() > 64 || !first(bytes[0]) {
        return false;
    }
    let mut at = 1;
    while at < bytes.len() {
        if !rest(bytes[at]) {
            return false;
        }
        at += 1;
    }
    true
}

const fn lower(byte: u8) -> bool {
    byte.is_ascii_lowercase()
}

const fn lower_or_digit(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit()
}

const fn namespace_byte(byte: u8) -> bool {
    lower_or_digit(byte) || byte == b'-'
}

const fn slug_byte(byte: u8) -> bool {
    lower_or_digit(byte) || matches!(byte, b'_' | b'.' | b'-')
}

/// Whether `text` is a namespace: a pack id.
pub const fn is_namespace(text: &str) -> bool {
    shaped(text, lower, namespace_byte)
}

/// Whether `text` is an attribute slug.
pub const fn is_slug(text: &str) -> bool {
    shaped(text, lower, slug_byte)
}

/// Whether `text` is a value slug.
pub const fn is_value_slug(text: &str) -> bool {
    shaped(text, lower_or_digit, slug_byte)
}

/// Whether `text` is a qualified id: a namespace, a `.`, and a slug.
pub const fn is_qualified(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut dot = 0;
    while dot < bytes.len() && bytes[dot] != b'.' {
        dot += 1;
    }
    if dot == 0 || dot >= bytes.len() {
        return false;
    }
    let (namespace, slug) = text.split_at(dot);
    let slug = slug.as_bytes();
    let (_, slug) = slug.split_at(1);
    match core::str::from_utf8(slug) {
        Ok(slug) => is_namespace(namespace) && is_slug(slug),
        Err(_) => false,
    }
}

macro_rules! text_id {
    ($(#[$doc:meta])* $name:ident, $check:path, $what:literal) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// `text`, checked.
            ///
            /// # Errors
            ///
            /// What the rule is, when `text` breaks it.
            pub fn new(text: &str) -> Result<Self, String> {
                if $check(text) {
                    Ok(Self(text.to_owned()))
                } else {
                    Err(format!(concat!("'{}' is not ", $what), text))
                }
            }

            /// As written.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let text = String::deserialize(deserializer)?;
                Self::new(&text).map_err(D::Error::custom)
            }
        }
    };
}

text_id!(
    /// An attribute's namespace: the declaring pack's id (`core` is `person-attributes`').
    Namespace,
    is_namespace,
    "a namespace (a pack id: 1-64 bytes of a-z, 0-9 and -, starting with a letter)"
);
text_id!(
    /// An attribute's slug within its namespace.
    Slug,
    is_slug,
    "an attribute slug (^[a-z][a-z0-9_.-]{0,63}$)"
);
text_id!(
    /// One value of a domain: a `category`'s, a `level`'s, a `set`'s, or a vocabulary's.
    ValueSlug,
    is_value_slug,
    "a value slug (^[a-z0-9][a-z0-9_.-]{0,63}$)"
);

impl ValueSlug {
    /// The default words of this value: the slug with each `-` written as a space.
    pub fn default_words(&self) -> String {
        self.0.replace('-', " ")
    }
}

/// An attribute's qualified id, `<namespace>.<slug>`: what every catalogue, fact and handle names it
/// by. Ordered as its text.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QualifiedId {
    text: String,
    dot: usize,
}

impl QualifiedId {
    /// `text`, checked and split at its first `.`.
    ///
    /// # Errors
    ///
    /// What the rule is, when `text` breaks it.
    pub fn new(text: &str) -> Result<Self, String> {
        if !is_qualified(text) {
            return Err(format!(
                "'{text}' is not a qualified attribute id (<namespace>.<slug>)"
            ));
        }
        let dot = text.find('.').unwrap_or_default();
        Ok(Self {
            text: text.to_owned(),
            dot,
        })
    }

    /// `slug` in `namespace`.
    pub fn of(namespace: &Namespace, slug: &Slug) -> Self {
        Self {
            text: format!("{namespace}.{slug}"),
            dot: namespace.as_str().len(),
        }
    }

    /// The whole id.
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// The namespace: the declaring pack's id.
    pub fn namespace(&self) -> &str {
        &self.text[..self.dot]
    }

    /// The slug: what an author may write alone where it is unique.
    pub fn slug(&self) -> &str {
        &self.text[self.dot + 1..]
    }
}

impl fmt::Display for QualifiedId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

impl Serialize for QualifiedId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.text)
    }
}

impl<'de> Deserialize<'de> for QualifiedId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::new(&text).map_err(D::Error::custom)
    }
}

/// The nearest of `known` to `wanted`: the three with the smallest edit distance, ties by text.
pub fn nearest<'k>(wanted: &str, known: impl IntoIterator<Item = &'k str>) -> Vec<String> {
    let mut scored: Vec<(usize, &str)> = known
        .into_iter()
        .map(|candidate| (distance(wanted, candidate), candidate))
        .collect();
    scored.sort_unstable();
    scored.dedup();
    scored
        .into_iter()
        .take(3)
        .map(|(_, candidate)| candidate.to_owned())
        .collect()
}

/// Levenshtein distance over bytes.
fn distance(a: &str, b: &str) -> usize {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, x) in a.iter().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, y) in b.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = if x == y {
                diagonal
            } else {
                1 + diagonal.min(above).min(row[j])
            };
            diagonal = above;
        }
    }
    row[b.len()]
}

#[cfg(test)]
mod tests;
