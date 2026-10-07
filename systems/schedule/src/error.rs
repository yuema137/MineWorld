//! What this pack refuses to build.

use thiserror::Error;

/// Every way a value this pack owns can fail to be built.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ScheduleError {
    /// A time of day that is not `HH:MM` within one day.
    #[error("a time of day is \"HH:MM\", from \"00:00\" to \"23:59\", but got {0:?}")]
    InvalidTime(String),

    /// An agenda label that is not a short lowercase slug.
    #[error("an agenda label must be 1 to {max} characters of a-z, 0-9 and '-', but got {label:?}")]
    InvalidLabel {
        /// What was offered.
        label: String,
        /// The longest label this pack records.
        max: usize,
    },

    /// A routine with too few or too many parts of the day.
    #[error("a routine has {min} to {max} segments, but this one has {got}")]
    SegmentCount {
        /// How many it has.
        got: usize,
        /// The fewest a routine may have: a single segment never changes, so it is no agenda.
        min: usize,
        /// The most.
        max: usize,
    },

    /// Segments whose `from` does not strictly increase: two would overlap or be empty.
    #[error(
        "routine segments must start at strictly increasing times, but {later} follows {earlier}"
    )]
    NotIncreasing {
        /// The earlier segment's start.
        earlier: String,
        /// The one that does not come after it.
        later: String,
    },

    /// Two neighbouring segments that are the same — a boundary at which nothing would change. The
    /// last segment's neighbour is the first, because the day wraps.
    #[error(
        "the segment from {at} repeats the one before it (same place and label): a boundary that \
         changes nothing is not a boundary"
    )]
    RepeatsItsNeighbour {
        /// The start of the repeating segment.
        at: String,
    },
}
