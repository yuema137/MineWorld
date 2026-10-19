//! The slugs this pack names sit points and their furniture by.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The longest key, in bytes, of a sit point or of a sit group.
pub const KEY_MAX_BYTES: usize = 48;

/// What a sit point's key or its group may not be.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("a {role} must be 1 to {max} bytes of a-z, 0-9, '-' and '.', but got {key:?}")]
pub struct InvalidSitKey {
    /// Which slug was refused: `sit point key` or `sit group`.
    pub role: &'static str,
    /// What was offered.
    pub key: String,
    /// The longest key this pack records.
    pub max: usize,
}

/// Whether `key` is a legal slug: 1 to [`KEY_MAX_BYTES`] bytes of `a`–`z`, `0`–`9`, `-` and `.`.
///
/// The dot is allowed so that a bench's points can be numbered within it (`window-bench.3`).
fn valid(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= KEY_MAX_BYTES
        && key.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'.'
        })
}

fn checked(role: &'static str, key: String) -> Result<String, InvalidSitKey> {
    if valid(&key) {
        Ok(key)
    } else {
        Err(InvalidSitKey {
            role,
            key,
            max: KEY_MAX_BYTES,
        })
    }
}

/// The name of a sit point, unique within its place: `window-bench.1`, `pier-bench.2`.
///
/// Refused rather than normalised: a key is never lowercased or truncated to make it valid. It names
/// the point and says nothing of what the point is made of; a Presentation Pack draws the furniture.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(try_from = "String", into = "String")]
#[schemars(extend("pattern" = "^[a-z0-9.-]+$", "minLength" = 1, "maxLength" = KEY_MAX_BYTES))]
pub struct SitPointKey(String);

impl SitPointKey {
    /// A key, refused unless it is a legal slug.
    ///
    /// # Errors
    ///
    /// [`InvalidSitKey`] naming what was offered.
    pub fn new(key: impl Into<String>) -> Result<Self, InvalidSitKey> {
        checked("sit point key", key.into()).map(Self)
    }

    /// The key.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for SitPointKey {
    type Error = InvalidSitKey;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<SitPointKey> for String {
    fn from(value: SitPointKey) -> Self {
        value.0
    }
}

impl core::fmt::Display for SitPointKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

/// The furniture a sit point belongs to, so that a depiction and a word can name it once: the
/// `window-bench` of `window-bench.1` and `window-bench.2`.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(try_from = "String", into = "String")]
#[schemars(extend("pattern" = "^[a-z0-9.-]+$", "minLength" = 1, "maxLength" = KEY_MAX_BYTES))]
pub struct SitGroupKey(String);

impl SitGroupKey {
    /// A group, refused unless it is a legal slug.
    ///
    /// # Errors
    ///
    /// [`InvalidSitKey`] naming what was offered.
    pub fn new(group: impl Into<String>) -> Result<Self, InvalidSitKey> {
        checked("sit group", group.into()).map(Self)
    }

    /// The group.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for SitGroupKey {
    type Error = InvalidSitKey;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<SitGroupKey> for String {
    fn from(value: SitGroupKey) -> Self {
        value.0
    }
}
