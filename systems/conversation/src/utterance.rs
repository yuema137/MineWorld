//! What somebody said.

use serde::{Deserialize, Serialize};

use crate::error::ConversationError;

/// The most a single utterance may carry, in bytes.
///
/// A conversational turn, not a document: 480 bytes is several sentences of any language this
/// project's worlds are written in. The number is this pack's own policy and becomes a configuration
/// field when World Packs gain a configuration schema (S7); what must not happen in the meantime is
/// for the bound to be *absent*, because the value reaches the event log.
pub const UTTERANCE_MAX_BYTES: usize = 480;

/// What one person said to another, as far as the world records it.
///
/// A newtype rather than a `String`, so that an utterance cannot be confused with a name, a tag or an
/// identifier, and so that the bound is checked once — where the value is built — rather than at
/// every place that stores one. Deserialization goes through the same check, which is what keeps a
/// hand-written client frame and a persisted record on the same terms as Rust code.
///
/// It carries no language, no speaker and no time. The speaker and the time are the fact's, recorded
/// on the [`Spoke`](crate::Spoke) event and the [`Heard`](crate::Heard) entry; a language tag is
/// presentation, and a contract that carried one would pull localization into the simulation
/// (`DD-13`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Utterance(String);

impl Utterance {
    /// Records what was said, refusing an empty or over-long utterance.
    pub fn new(said: impl Into<String>) -> Result<Self, ConversationError> {
        let said = said.into();
        if said.is_empty() {
            return Err(ConversationError::EmptyUtterance);
        }
        if said.len() > UTTERANCE_MAX_BYTES {
            return Err(ConversationError::UtteranceTooLong {
                length: said.len(),
                max: UTTERANCE_MAX_BYTES,
            });
        }
        Ok(Self(said))
    }

    /// What was said.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Utterance {
    type Error = ConversationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Utterance> for String {
    fn from(value: Utterance) -> Self {
        value.0
    }
}

impl core::fmt::Display for Utterance {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}
