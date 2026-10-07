//! What sort of thing an activity is — content, which this pack carries and never interprets.

use serde::{Deserialize, Serialize};

use crate::error::GroupActivityError;

/// The longest activity kind, in bytes.
pub const KIND_MAX_BYTES: usize = 32;

/// What an inviter proposes doing: `coffee`, `walk`, `chat`.
///
/// A newtype over a slug rather than an enum, because the set of things people do together belongs to
/// a world's content, not to this pack's code (`INV-12`): a market town's `lunch` must not need a
/// release of this crate. A slug rather than free text, because it reaches the event log.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ActivityKind(String);

impl ActivityKind {
    /// A kind, refused unless it is 1 to [`KIND_MAX_BYTES`] characters of `a-z`, `0-9` and `-`.
    pub fn new(kind: impl Into<String>) -> Result<Self, GroupActivityError> {
        let kind = kind.into();
        let valid = !kind.is_empty()
            && kind.len() <= KIND_MAX_BYTES
            && kind
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
        if valid {
            Ok(Self(kind))
        } else {
            Err(GroupActivityError::InvalidKind {
                kind,
                max: KIND_MAX_BYTES,
            })
        }
    }

    /// The slug.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ActivityKind {
    type Error = GroupActivityError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<ActivityKind> for String {
    fn from(value: ActivityKind) -> Self {
        value.0
    }
}

impl core::fmt::Display for ActivityKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}
