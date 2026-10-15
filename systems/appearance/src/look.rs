//! A look: the key naming a person's appearance.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The longest look, in bytes.
pub const LOOK_MAX_BYTES: usize = 48;

/// What a look may not be.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("a look must be 1 to {max} bytes of a-z, 0-9 and '-', but got {look:?}")]
pub struct InvalidLook {
    /// What was offered.
    pub look: String,
    /// The longest look this pack records.
    pub max: usize,
}

/// An opaque key naming a person's appearance. Not a mesh, a file, a texture, a colour or a height:
/// the world states *that* Alice has the appearance called `alice-moreau`, never what it is made of.
/// A Presentation Pack resolves it (`ARC-92`).
///
/// Refused rather than normalised: a key is never lowercased or truncated to make it valid.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct LookKey(String);

impl LookKey {
    /// A look, refused unless it is 1 to 48 bytes of `a`–`z`, `0`–`9` and `-`.
    ///
    /// # Errors
    ///
    /// [`InvalidLook`] naming what was offered.
    pub fn new(look: impl Into<String>) -> Result<Self, InvalidLook> {
        let look = look.into();
        let valid = !look.is_empty()
            && look.len() <= LOOK_MAX_BYTES
            && look
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
        if valid {
            Ok(Self(look))
        } else {
            Err(InvalidLook {
                look,
                max: LOOK_MAX_BYTES,
            })
        }
    }

    /// The key.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for LookKey {
    type Error = InvalidLook;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<LookKey> for String {
    fn from(value: LookKey) -> Self {
        value.0
    }
}

impl core::fmt::Display for LookKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}
