//! A person's character, as authored: how they are, and how they speak.
//!
//! Every bound is checked where its value is decoded, so the loader reports a refusal at the line and
//! column of the value that broke it (the decoder reports a position only for a refusal raised inside
//! one value; `worldpack`'s refusals tests rely on it). Nothing is truncated to fit.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The longest summary, in bytes.
pub const SUMMARY_MAX_BYTES: usize = 280;
/// The most traits a person may have.
pub const TRAITS_MAX: usize = 8;
/// The longest trait, in bytes.
pub const TRAIT_MAX_BYTES: usize = 32;
/// The longest speech note, in bytes.
pub const SPEECH_MAX_BYTES: usize = 160;

/// What a persona may not be. Each refusal names the bound it broke and what was offered.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PersonaError {
    /// The summary is empty.
    #[error("a persona's summary must not be empty")]
    EmptySummary,
    /// The summary is longer than [`SUMMARY_MAX_BYTES`].
    #[error("a persona's summary must be at most {max} bytes, but is {len}")]
    SummaryTooLong {
        /// The offered length, in bytes.
        len: usize,
        /// The longest summary this pack records.
        max: usize,
    },
    /// There are more traits than [`TRAITS_MAX`].
    #[error("a persona may have at most {max} traits, but has {count}")]
    TooManyTraits {
        /// The offered count.
        count: usize,
        /// The most traits this pack records.
        max: usize,
    },
    /// A trait is longer than [`TRAIT_MAX_BYTES`].
    #[error("a persona's trait must be at most {max} bytes, but {trait_text:?} is {len}")]
    TraitTooLong {
        /// The offered trait.
        trait_text: String,
        /// Its length, in bytes.
        len: usize,
        /// The longest trait this pack records.
        max: usize,
    },
    /// The speech note is longer than [`SPEECH_MAX_BYTES`].
    #[error("a persona's speech must be at most {max} bytes, but is {len}")]
    SpeechTooLong {
        /// The offered length, in bytes.
        len: usize,
        /// The longest speech note this pack records.
        max: usize,
    },
}

/// A person's summary of who they are: 1 to [`SUMMARY_MAX_BYTES`] bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "String", into = "String")]
pub struct Summary(String);

impl Summary {
    /// A summary, refused unless it is non-empty and within the bound.
    ///
    /// # Errors
    ///
    /// [`PersonaError::EmptySummary`] or [`PersonaError::SummaryTooLong`].
    pub fn new(text: impl Into<String>) -> Result<Self, PersonaError> {
        let text = text.into();
        if text.is_empty() {
            return Err(PersonaError::EmptySummary);
        }
        if text.len() > SUMMARY_MAX_BYTES {
            return Err(PersonaError::SummaryTooLong {
                len: text.len(),
                max: SUMMARY_MAX_BYTES,
            });
        }
        Ok(Self(text))
    }

    /// The summary.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One trait: 0 to [`TRAIT_MAX_BYTES`] bytes of content, which this pack carries and never interprets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "String", into = "String")]
pub struct Trait(String);

impl Trait {
    /// A trait, refused beyond [`TRAIT_MAX_BYTES`].
    ///
    /// # Errors
    ///
    /// [`PersonaError::TraitTooLong`].
    pub fn new(text: impl Into<String>) -> Result<Self, PersonaError> {
        let text = text.into();
        if text.len() > TRAIT_MAX_BYTES {
            return Err(PersonaError::TraitTooLong {
                len: text.len(),
                max: TRAIT_MAX_BYTES,
                trait_text: text,
            });
        }
        Ok(Self(text))
    }

    /// The trait.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A person's traits, in the order authored: at most [`TRAITS_MAX`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "Vec<Trait>", into = "Vec<Trait>")]
pub struct Traits(Vec<Trait>);

impl Traits {
    /// The traits, refused beyond [`TRAITS_MAX`].
    ///
    /// # Errors
    ///
    /// [`PersonaError::TooManyTraits`].
    pub fn new(traits: Vec<Trait>) -> Result<Self, PersonaError> {
        if traits.len() > TRAITS_MAX {
            return Err(PersonaError::TooManyTraits {
                count: traits.len(),
                max: TRAITS_MAX,
            });
        }
        Ok(Self(traits))
    }

    /// The traits, in the order authored.
    pub fn as_slice(&self) -> &[Trait] {
        &self.0
    }
}

/// A note on how the person speaks: 0 to [`SPEECH_MAX_BYTES`] bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "String", into = "String")]
pub struct Speech(String);

impl Speech {
    /// A note on speech, refused beyond [`SPEECH_MAX_BYTES`].
    ///
    /// # Errors
    ///
    /// [`PersonaError::SpeechTooLong`].
    pub fn new(text: impl Into<String>) -> Result<Self, PersonaError> {
        let text = text.into();
        if text.len() > SPEECH_MAX_BYTES {
            return Err(PersonaError::SpeechTooLong {
                len: text.len(),
                max: SPEECH_MAX_BYTES,
            });
        }
        Ok(Self(text))
    }

    /// The note.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Summary {
    type Error = PersonaError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Summary> for String {
    fn from(value: Summary) -> Self {
        value.0
    }
}

impl TryFrom<String> for Trait {
    type Error = PersonaError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Trait> for String {
    fn from(value: Trait) -> Self {
        value.0
    }
}

impl TryFrom<Vec<Trait>> for Traits {
    type Error = PersonaError;

    fn try_from(value: Vec<Trait>) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Traits> for Vec<Trait> {
    fn from(value: Traits) -> Self {
        value.0
    }
}

impl TryFrom<String> for Speech {
    type Error = PersonaError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Speech> for String {
    fn from(value: Speech) -> Self {
        value.0
    }
}

/// A person's character: a summary, up to [`TRAITS_MAX`] traits, and an optional note on speech.
///
/// Authored as `persona: { summary: …, traits: […], speech: … }`. Unknown keys are refused at their
/// line; `summary` is required; `traits` and `speech` are optional. The text is content a World Pack
/// author writes and a controller may read when it speaks for the person; this pack never interprets
/// it. A persona shapes speech, and grants nothing (`ARC-74`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Persona {
    /// Who the person is, in a sentence or two: 1 to 280 bytes. Required.
    summary: Summary,
    /// Short words for the person's character, in the order written: at most 8, each at most 32
    /// bytes. Absent, none.
    #[serde(default = "no_traits", skip_serializing_if = "Traits::is_empty")]
    traits: Traits,
    /// A note on how the person speaks: at most 160 bytes. Absent, none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    speech: Option<Speech>,
}

fn no_traits() -> Traits {
    Traits(Vec::new())
}

impl Traits {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Persona {
    /// A persona built in code, refused unless every bound holds. Authored personas are decoded from a
    /// section, which applies the same bounds value by value.
    ///
    /// # Errors
    ///
    /// [`PersonaError`] naming the first bound broken.
    pub fn new(
        summary: impl Into<String>,
        traits: Vec<String>,
        speech: Option<String>,
    ) -> Result<Self, PersonaError> {
        let traits = traits
            .into_iter()
            .map(Trait::new)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            summary: Summary::new(summary)?,
            traits: Traits::new(traits)?,
            speech: speech.map(Speech::new).transpose()?,
        })
    }

    /// A summary of who the person is.
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }

    /// The person's traits, in the order authored.
    pub fn traits(&self) -> Vec<&str> {
        self.traits.as_slice().iter().map(Trait::as_str).collect()
    }

    /// How the person speaks, if the author said.
    pub fn speech(&self) -> Option<&str> {
        self.speech.as_ref().map(Speech::as_str)
    }
}
