//! The time of day: schedule's own convention, which the kernel does not know (`INV-12`, `ARC-32`).

use mineworld_contracts::WorldTime;
use serde::{Deserialize, Serialize};

use crate::error::ScheduleError;

/// Seconds in a day.
pub const DAY: i64 = 86_400;

/// A time of day, to the minute: `"06:30"`.
///
/// World seconds since the epoch, modulo [`DAY`]. Written and recorded as `"HH:MM"` — a person reads it
/// in a pack file and in a log — and held as seconds since midnight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct TimeOfDay(i64);

impl TimeOfDay {
    /// `hours:minutes`, refused outside one day.
    ///
    /// # Errors
    ///
    /// [`ScheduleError::InvalidTime`].
    pub fn new(hours: i64, minutes: i64) -> Result<Self, ScheduleError> {
        if (0..24).contains(&hours) && (0..60).contains(&minutes) {
            Ok(Self(hours * 3_600 + minutes * 60))
        } else {
            Err(ScheduleError::InvalidTime(format!("{hours}:{minutes}")))
        }
    }

    /// Seconds since midnight.
    pub const fn seconds(self) -> i64 {
        self.0
    }

    /// The time of day an instant falls at.
    pub const fn of(at: WorldTime) -> i64 {
        at.seconds().rem_euclid(DAY)
    }
}

impl TryFrom<String> for TimeOfDay {
    type Error = ScheduleError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let invalid = || ScheduleError::InvalidTime(value.clone());
        let (hours, minutes) = value.split_once(':').ok_or_else(invalid)?;
        let two_digits = |part: &str| part.len() == 2 && part.bytes().all(|b| b.is_ascii_digit());
        if !two_digits(hours) || !two_digits(minutes) {
            return Err(invalid());
        }
        let hours: i64 = hours.parse().map_err(|_| invalid())?;
        let minutes: i64 = minutes.parse().map_err(|_| invalid())?;
        Self::new(hours, minutes).map_err(|_| invalid())
    }
}

impl From<TimeOfDay> for String {
    fn from(value: TimeOfDay) -> Self {
        value.to_string()
    }
}

impl core::fmt::Display for TimeOfDay {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:02}:{:02}", self.0 / 3_600, self.0 % 3_600 / 60)
    }
}
