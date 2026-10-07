//! What a part of the day is for — content, which this pack carries and never interprets.

use serde::{Deserialize, Serialize};

use crate::error::ScheduleError;

/// The longest label, in bytes.
pub const LABEL_MAX_BYTES: usize = 32;

/// What a segment of a routine is for: `work`, `coffee`, `home`.
///
/// A slug rather than an enum, because what people do with their days is a world's content, not this
/// pack's code (`INV-12`); a slug rather than free text, because it reaches the event log.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AgendaLabel(String);

impl AgendaLabel {
    /// A label, refused unless it is 1 to [`LABEL_MAX_BYTES`] characters of `a-z`, `0-9` and `-`.
    ///
    /// # Errors
    ///
    /// [`ScheduleError::InvalidLabel`].
    pub fn new(label: impl Into<String>) -> Result<Self, ScheduleError> {
        let label = label.into();
        let valid = !label.is_empty()
            && label.len() <= LABEL_MAX_BYTES
            && label
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
        if valid {
            Ok(Self(label))
        } else {
            Err(ScheduleError::InvalidLabel {
                label,
                max: LABEL_MAX_BYTES,
            })
        }
    }

    /// The slug.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for AgendaLabel {
    type Error = ScheduleError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<AgendaLabel> for String {
    fn from(value: AgendaLabel) -> Self {
        value.0
    }
}

impl core::fmt::Display for AgendaLabel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}
