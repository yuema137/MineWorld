//! What this pack refuses to build.

use thiserror::Error;

/// Every way a value this pack owns can fail to be built.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum GroupActivityError {
    /// An activity kind that is not a short lowercase slug.
    ///
    /// Refused where it is built, so a hand-written client frame and a persisted record are held to
    /// the same rule as Rust code: the kind reaches the event log, and a log is not where free text
    /// with no bound belongs.
    #[error("an activity kind must be 1 to {max} characters of a-z, 0-9 and '-', but got {kind:?}")]
    InvalidKind {
        /// What was offered.
        kind: String,
        /// The longest kind this pack records.
        max: usize,
    },
}
