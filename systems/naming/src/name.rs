//! A display name: what people call somebody.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The longest name, in bytes.
pub const NAME_MAX_BYTES: usize = 64;

/// What a name may not be.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error(
    "a name must be 1 to {max} bytes, with no control characters and no space at either end, but \
     got {name:?}"
)]
pub struct InvalidName {
    /// What was offered.
    pub name: String,
    /// The longest name this pack records.
    pub max: usize,
}

/// A person's name, as people say it: `Alice Moreau`.
///
/// Free text, unlike a key: names are content a person reads, in any script. Bounded, because it
/// reaches the event log and the words other people say (`UTTERANCE_MAX_BYTES` has to hold a sentence
/// that quotes it). No control character, so a name cannot break a line of a transcript or a
/// biography, and no surrounding space, so two spellings of one name cannot differ invisibly.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Name(String);

impl Name {
    /// A name, refused unless it keeps the rule above.
    ///
    /// # Errors
    ///
    /// [`InvalidName`] naming what was offered.
    pub fn new(name: impl Into<String>) -> Result<Self, InvalidName> {
        let name = name.into();
        let valid = !name.is_empty()
            && name.len() <= NAME_MAX_BYTES
            && !name.chars().any(char::is_control)
            && name.trim() == name;
        if valid {
            Ok(Self(name))
        } else {
            Err(InvalidName {
                name,
                max: NAME_MAX_BYTES,
            })
        }
    }

    /// The name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Name {
    type Error = InvalidName;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Name> for String {
    fn from(value: Name) -> Self {
        value.0
    }
}

impl core::fmt::Display for Name {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}
