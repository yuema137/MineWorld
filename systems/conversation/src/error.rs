//! What this pack refuses to build.
//!
//! One enum, for the same reason the contract layer has one: a caller loading authored content or
//! decoding a request handles all of it at the same boundary. It is not a second
//! [`ContractError`](mineworld_contracts::ContractError) — the contract layer's errors are about
//! kernel vocabulary, and these are about this pack's own.

use thiserror::Error;

/// Every way a value this pack owns can fail to be built.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConversationError {
    /// Somebody said nothing at all.
    ///
    /// Refused rather than recorded: a `spoke` fact with no content is a fact about nothing, and a
    /// world that admits them fills its history with entries a controller has to filter out.
    #[error("an utterance must not be empty")]
    EmptyUtterance,

    /// Somebody said more than a world will carry.
    ///
    /// The bound exists because an utterance reaches the event log, and the log is persisted,
    /// replicated and replayed — so an unbounded string in it is an unbounded row and an unbounded
    /// frame. Refused where it is written rather than truncated where it is stored, because a
    /// truncated utterance is a different thing said.
    #[error("an utterance must be at most {max} bytes long, but this one is {length}")]
    UtteranceTooLong {
        /// How long the offered utterance was, in bytes.
        length: usize,
        /// The most this pack records.
        max: usize,
    },
}
