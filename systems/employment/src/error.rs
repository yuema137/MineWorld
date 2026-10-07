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
}
