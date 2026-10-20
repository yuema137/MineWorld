//! What this pack refuses to build.

use thiserror::Error;

/// Every way an authored job can fail to be built.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EmploymentError {
    /// A shift that does not end after it starts. A shift lies within one day (step-10 QS-50).
    #[error(
        "a shift must end after it starts, within one day, but this one runs from {from} until {until}"
    )]
    ShiftOutOfOrder {
        /// When it starts.
        from: String,
        /// When it ends.
        until: String,
    },
    /// A job's `title` is empty, longer than 40 bytes or holds a control character (S30-f).
    #[error("a job title is 1 to 40 bytes with no control character: {why}")]
    TitleRefused {
        /// Which rule it breaks.
        why: String,
    },
}
