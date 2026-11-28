//! A locale: the name of a language a reader may choose for world text that a pack offers in more than
//! one language (`docs/DECISIONS.md` `ARC-129`; `pr-s30k-localisation.md` §5.1).
//!
//! ```text
//! grammar     language [ "_" Script ] [ "_" REGION ]
//!             language  2–3 lower-case ASCII letters
//!             Script    4 ASCII letters, the first upper-case, the rest lower-case
//!             REGION    2 upper-case ASCII letters, or 3 ASCII digits
//! canonical   "_" between the parts, the spelling of the client catalogs (`zh_Hans`, `pt_BR`)
//! input       a person may type "-" instead ("zh-Hans"), normalised by `parse`
//! English     `en` names the declarations themselves; it is never a translation of them
//! ```
//!
//! This crate names no language but English: every other locale is data.

use core::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A validated locale in canonical form.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Locale(String);

/// Why a locale was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleError(String);

impl fmt::Display for LocaleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "'{}' is not a locale: write a language of 2 or 3 lower-case letters, then optionally \
             '_' and a 4-letter script in title case, then optionally '_' and a 2-letter region in \
             upper case or 3 digits (such as en, pt_BR)",
            self.0
        )
    }
}

impl std::error::Error for LocaleError {}

/// The language every declaration is written in.
const ENGLISH: &str = "en";

fn is_language(part: &str) -> bool {
    (2..=3).contains(&part.len()) && part.bytes().all(|byte| byte.is_ascii_lowercase())
}

fn is_script(part: &str) -> bool {
    let bytes = part.as_bytes();
    bytes.len() == 4
        && bytes[0].is_ascii_uppercase()
        && bytes[1..].iter().all(u8::is_ascii_lowercase)
}

fn is_region(part: &str) -> bool {
    (part.len() == 2 && part.bytes().all(|byte| byte.is_ascii_uppercase()))
        || (part.len() == 3 && part.bytes().all(|byte| byte.is_ascii_digit()))
}

impl Locale {
    /// English, the language of the declarations.
    pub fn english() -> Self {
        Self(ENGLISH.to_owned())
    }

    /// A locale in canonical form only (a file name's stem, a wire key).
    ///
    /// # Errors
    ///
    /// [`LocaleError`] when `text` breaks the grammar or is not canonical.
    pub fn canonical(text: &str) -> Result<Self, LocaleError> {
        let mut parts = text.split('_');
        let language = parts.next().unwrap_or_default();
        let rest: Vec<&str> = parts.collect();
        let valid = is_language(language)
            && match rest.as_slice() {
                [] => true,
                [one] => is_script(one) || is_region(one),
                [script, region] => is_script(script) && is_region(region),
                _ => false,
            };
        if valid {
            Ok(Self(text.to_owned()))
        } else {
            Err(LocaleError(text.to_owned()))
        }
    }

    /// A locale as a person types it: `-` is accepted for `_` and normalised.
    ///
    /// # Errors
    ///
    /// [`LocaleError`] when the normalised text breaks the grammar.
    pub fn parse(text: &str) -> Result<Self, LocaleError> {
        Self::canonical(&text.replace('-', "_")).map_err(|_| LocaleError(text.to_owned()))
    }

    /// The canonical text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether this is English, the language of the declarations.
    pub fn is_english(&self) -> bool {
        self.0 == ENGLISH
    }

    /// The language part (`zh` of `zh_Hans`).
    pub fn language(&self) -> &str {
        self.0.split('_').next().unwrap_or_default()
    }

    /// The locale a reader wanting `self` reads among `offered` (`ARC-129`; PDN's rule): `self` when it
    /// is offered; else the one offered locale with the same language, when exactly one has it; else
    /// `None`, which means English.
    pub fn select<'a>(&self, offered: impl IntoIterator<Item = &'a Self>) -> Option<&'a Self> {
        if self.is_english() {
            return None;
        }
        let mut same_language = Vec::new();
        for locale in offered {
            if locale == self {
                return Some(locale);
            }
            if locale.language() == self.language() {
                same_language.push(locale);
            }
        }
        match same_language.as_slice() {
            [one] => Some(one),
            _ => None,
        }
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl core::str::FromStr for Locale {
    type Err = LocaleError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text)
    }
}

impl Serialize for Locale {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

/// Decoding takes the canonical form only: a file or a wire carries what was written.
impl<'de> Deserialize<'de> for Locale {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::canonical(&text).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests;
