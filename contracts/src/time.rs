//! Simulated time as a value: a point on a world's own clock, and a length of it.
//!
//! # No calendar here
//!
//! A world's clock counts simulated seconds from that world's epoch, and nothing in this crate
//! knows what a date, a weekday, a month or a season is. Those belong to whichever system a
//! world installs to interpret time: a modern-life world and a space-colony world can disagree
//! about what "a day" means without the kernel having an opinion. This is also why there is no
//! `chrono::DateTime` in the contract layer — importing a calendar would import exactly the
//! domain knowledge the kernel must not have.
//!
//! # No scheduling here either
//!
//! These are value types with an order and a difference. How time advances, how a tick is
//! chosen, how processes are woken and what happens when two of them are due at the same moment
//! are the scheduler's contracts, and they arrive with it. Nothing here advances a clock.

use serde::{Deserialize, Serialize};

/// A point on a world's clock, as a signed count of simulated seconds from that world's epoch.
///
/// Signed, because a world can refer to moments before its own epoch: an authored biography, a
/// place built two hundred years before the simulation starts. Ordered, because comparing two
/// moments is the operation every consumer needs.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(transparent)]
pub struct WorldTime(i64);

impl WorldTime {
    /// The world's own zero: the moment its clock counts from.
    pub const EPOCH: Self = Self(0);

    /// The moment `seconds` after the epoch — before it, if negative.
    pub const fn from_seconds(seconds: i64) -> Self {
        Self(seconds)
    }

    /// Simulated seconds from the epoch.
    pub const fn seconds(self) -> i64 {
        self.0
    }

    /// How long after `earlier` this moment is, negative if it is in fact before it.
    ///
    /// `None` only when the two moments are further apart than [`i64`] can express, which no
    /// world reaches; it is an `Option` rather than a wrap or a saturation because a contract
    /// type must not quietly return a wrong duration.
    pub const fn duration_since(self, earlier: Self) -> Option<SimDuration> {
        match self.0.checked_sub(earlier.0) {
            Some(seconds) => Some(SimDuration::from_seconds(seconds)),
            None => None,
        }
    }
}

impl core::fmt::Display for WorldTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "t{}", self.0)
    }
}

/// A length of simulated time, as a signed count of seconds.
///
/// Signed for the same reason a difference is: `duration_since` of an earlier moment is
/// negative, and a type that could not say so would have to lie or panic.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(transparent)]
pub struct SimDuration(i64);

impl SimDuration {
    /// No time at all.
    pub const ZERO: Self = Self(0);

    /// A length of `seconds` simulated seconds.
    pub const fn from_seconds(seconds: i64) -> Self {
        Self(seconds)
    }

    /// The length in simulated seconds.
    pub const fn seconds(self) -> i64 {
        self.0
    }
}

impl core::fmt::Display for SimDuration {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}s", self.0)
    }
}
