//! A shop's daily sales hours (`pr-nightlife.md` N1-I1, N1-I2): optional data on the [`Shop`], read
//! only by this pack to judge a sale.
//!
//! ```text
//! absent        always open — every shop before N1, and any world that writes no hours
//! []            closed all day
//! 1 … 8         intervals { from: "HH:MM", until: "HH:MM" }, half-open [from, until)
//! until < from  the interval wraps midnight: 22:00–06:00 is open at 23:59:59 and 00:00:00
//! from = until  refused: an empty interval says nothing
//! overlap       refused after wrapping: two intervals may touch, never share a second
//! ```
//!
//! The time of day is `at mod 86 400` (instant 0 is local midnight, `INV-TW-4`), schedule's
//! [`TimeOfDay`] convention, reused as a value type only (`ARC-28` extended to a value type): economy
//! has no system dependency on schedule. No open/closed flag is stored and nothing is scheduled: whether
//! a shop is open is a pure function of its hours and the instant a read is taken at.
//!
//! [`Shop`]: crate::component::Shop

use core::fmt;

use mineworld_contracts::WorldTime;
use mineworld_schedule::{DAY, TimeOfDay};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

/// The most intervals one shop's hours may have.
pub const MAX_OPENINGS: usize = 8;

/// One interval of a day in which a shop sells: from `from`, up to but not including `until`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct Opening {
    /// When the shop begins selling, "HH:MM".
    from: TimeOfDay,
    /// When it stops, "HH:MM", exclusive. Earlier than `from`, the interval runs past midnight.
    until: TimeOfDay,
}

impl Opening {
    /// The interval `[from, until)`; whether it is valid is [`Hours::new`]'s to judge.
    pub const fn new(from: TimeOfDay, until: TimeOfDay) -> Self {
        Self { from, until }
    }

    /// When it opens.
    pub const fn from(&self) -> TimeOfDay {
        self.from
    }

    /// When it closes.
    pub const fn until(&self) -> TimeOfDay {
        self.until
    }

    /// The interval as one or two spans of seconds since midnight, `[start, end)` within one day.
    fn spans(self) -> impl Iterator<Item = (i64, i64)> {
        let (from, until) = (self.from.seconds(), self.until.seconds());
        let (first, second) = if from < until {
            ((from, until), None)
        } else {
            ((from, DAY), Some((0, until)))
        };
        core::iter::once(first).chain(second.filter(|(start, end)| start < end))
    }
}

/// A shop's sales hours: zero to [`MAX_OPENINGS`] intervals, none empty and no two overlapping once
/// wrapped across midnight, in authored order. Valid by construction and on every decode, so a save
/// or a fact that carries invalid hours is refused, never read as "always open".
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Hours(Vec<Opening>);

/// Why hours were refused, naming the interval by its index in the authored list (0-based).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HoursError {
    /// More than [`MAX_OPENINGS`] intervals.
    TooMany {
        /// How many were written.
        got: usize,
    },
    /// An interval whose `from` equals its `until`.
    Empty {
        /// Its index.
        index: usize,
        /// The time both ends name.
        at: TimeOfDay,
    },
    /// Two intervals that share a second of the day, after wrapping.
    Overlap {
        /// The lower index.
        first: usize,
        /// The higher index.
        second: usize,
    },
}

impl fmt::Display for HoursError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooMany { got } => write!(
                f,
                "a shop's hours have at most {MAX_OPENINGS} intervals, but these have {got}"
            ),
            Self::Empty { index, at } => write!(
                f,
                "hours interval {index} opens and closes at {at}: an empty interval is refused (omit \
                 `hours` for a shop that is always open)"
            ),
            Self::Overlap { first, second } => write!(
                f,
                "hours intervals {first} and {second} overlap (after wrapping past midnight)"
            ),
        }
    }
}

impl std::error::Error for HoursError {}

impl Hours {
    /// Hours, refused unless they keep the rules in the module's table.
    ///
    /// # Errors
    ///
    /// The [`HoursError`] naming the first rule broken and the interval(s) breaking it.
    pub fn new(openings: Vec<Opening>) -> Result<Self, HoursError> {
        if openings.len() > MAX_OPENINGS {
            return Err(HoursError::TooMany {
                got: openings.len(),
            });
        }
        if let Some((index, opening)) = openings
            .iter()
            .enumerate()
            .find(|(_, opening)| opening.from == opening.until)
        {
            return Err(HoursError::Empty {
                index,
                at: opening.from,
            });
        }
        let mut spans: Vec<(i64, i64, usize)> = openings
            .iter()
            .enumerate()
            .flat_map(|(index, opening)| opening.spans().map(move |(s, e)| (s, e, index)))
            .collect();
        spans.sort_unstable();
        for pair in spans.windows(2) {
            let ((_, end, a), (start, _, b)) = (pair[0], pair[1]);
            if start < end {
                return Err(HoursError::Overlap {
                    first: a.min(b),
                    second: a.max(b),
                });
            }
        }
        Ok(Self(openings))
    }

    /// The intervals, as authored.
    pub fn openings(&self) -> &[Opening] {
        &self.0
    }

    /// Whether these hours sell at `at`: its time of day lies in one of the intervals.
    pub fn open_at(&self, at: WorldTime) -> bool {
        let now = TimeOfDay::of(at);
        self.0
            .iter()
            .flat_map(|opening| opening.spans())
            .any(|(start, end)| (start..end).contains(&now))
    }
}

impl<'de> Deserialize<'de> for Hours {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let openings = Vec::<Opening>::deserialize(deserializer)?;
        Self::new(openings).map_err(D::Error::custom)
    }
}

/// Hours as authored: 0 to 8 intervals (`pr-nightlife.md` N1-I2); the emptiness and overlap rules
/// [`Hours::new`] enforces are stated in the description.
impl schemars::JsonSchema for Hours {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("Hours")
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let opening = generator.subschema_for::<Opening>();
        schemars::json_schema!({
            "type": "array",
            "items": opening,
            "maxItems": MAX_OPENINGS,
            "description": "The shop's daily sales hours: intervals [from, until), \"HH:MM\". An \
                            interval whose until is earlier than its from runs past midnight. \
                            from = until is refused, and no two intervals may overlap once wrapped. \
                            [] is closed all day; leaving `hours` out is always open.",
            "examples": [[
                { "from": "05:30", "until": "22:00" },
            ], [
                { "from": "08:00", "until": "12:00" },
                { "from": "14:00", "until": "18:00" },
            ]],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Hours, HoursError, Opening};
    use mineworld_contracts::WorldTime;
    use mineworld_schedule::TimeOfDay;

    fn time(text: &str) -> TimeOfDay {
        TimeOfDay::try_from(text.to_owned()).expect("a time")
    }

    fn hours(intervals: &[(&str, &str)]) -> Result<Hours, HoursError> {
        Hours::new(
            intervals
                .iter()
                .map(|(from, until)| Opening::new(time(from), time(until)))
                .collect(),
        )
    }

    /// Hand-computed: 22:00–06:00 is open at 23:59:59 (86 399) and 00:00:00 of the next day (86 400),
    /// closed at 06:00:00 (21 600) and at 21:59:59 (79 199).
    #[test]
    fn an_interval_past_midnight_is_open_until_its_end_and_not_at_it() {
        let night = hours(&[("22:00", "06:00")]).expect("valid");
        let open = |seconds| night.open_at(WorldTime::from_seconds(seconds));
        assert!(open(86_399) && open(86_400) && open(79_200) && open(21_599));
        assert!(!open(21_600) && !open(79_199) && !open(43_200));
    }

    /// Two intervals that share 05:00–06:00 once the night one wraps; touching intervals are allowed.
    #[test]
    fn overlap_is_judged_after_wrapping_and_touching_is_not_overlap() {
        assert_eq!(
            hours(&[("08:00", "12:00"), ("22:00", "06:00"), ("05:00", "07:00")]),
            Err(HoursError::Overlap {
                first: 1,
                second: 2
            })
        );
        let split = hours(&[("08:00", "12:00"), ("12:00", "14:00"), ("22:00", "08:00")])
            .expect("touching intervals cover the day but for 14:00–22:00");
        assert!(split.open_at(WorldTime::from_seconds(12 * 3_600)));
        assert!(!split.open_at(WorldTime::from_seconds(14 * 3_600)));
    }
}
