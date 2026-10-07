//! A day, in parts: what a routine is, and the rules every routine keeps.

use mineworld_contracts::{EntityKey, WorldTime};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::error::ScheduleError;
use crate::label::AgendaLabel;
use crate::time::{DAY, TimeOfDay};

/// The fewest segments a routine may have: one segment never changes, so it would be no agenda.
pub const MIN_SEGMENTS: usize = 2;
/// The most segments a routine may have.
pub const MAX_SEGMENTS: usize = 24;

/// One part of a day: from a time, at a place, for something.
///
/// Generic over how the place is named: by authoring key in a pack file, by `PlaceId` once the world
/// exists. The rules are the same either way, so they are stated once, in [`Segments::new`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Segment<P> {
    from: TimeOfDay,
    place: P,
    label: AgendaLabel,
}

impl<P> Segment<P> {
    /// A segment.
    pub const fn new(from: TimeOfDay, place: P, label: AgendaLabel) -> Self {
        Self { from, place, label }
    }

    /// When it begins, each day.
    pub const fn from(&self) -> TimeOfDay {
        self.from
    }

    /// Where it is.
    pub const fn place(&self) -> &P {
        &self.place
    }

    /// What it is for.
    pub const fn label(&self) -> &AgendaLabel {
        &self.label
    }
}

/// The segments of a day, in order, valid by construction (`ARC-32`):
///
/// ```text
/// count       2..=24
/// from        strictly increasing, so no two overlap and none is empty
/// neighbours  differ in place or label — the last against the first included, because the day
///             wraps: a boundary that changes nothing is not a boundary
/// ```
///
/// The first segment need not begin at midnight: the time before it belongs to the last.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Segments<P>(Vec<Segment<P>>);

/// A routine as authored: places named by key.
pub type AuthoredRoutine = Segments<EntityKey>;

/// Which segment is in force at an instant, since when, and until when.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InForce<'a, P> {
    /// The segment.
    pub segment: &'a Segment<P>,
    /// When it began (perhaps the day before, for the segment that wraps past midnight).
    pub since: WorldTime,
    /// The next boundary.
    pub until: WorldTime,
}

impl<P: PartialEq> Segments<P> {
    /// Segments, refused unless they keep the rules above.
    ///
    /// # Errors
    ///
    /// The [`ScheduleError`] naming the rule that was broken.
    pub fn new(segments: Vec<Segment<P>>) -> Result<Self, ScheduleError> {
        if !(MIN_SEGMENTS..=MAX_SEGMENTS).contains(&segments.len()) {
            return Err(ScheduleError::SegmentCount {
                got: segments.len(),
                min: MIN_SEGMENTS,
                max: MAX_SEGMENTS,
            });
        }
        for pair in segments.windows(2) {
            if pair[1].from <= pair[0].from {
                return Err(ScheduleError::NotIncreasing {
                    earlier: pair[0].from.to_string(),
                    later: pair[1].from.to_string(),
                });
            }
        }
        for (at, segment) in segments.iter().enumerate() {
            let before = &segments[(at + segments.len() - 1) % segments.len()];
            if before.place == segment.place && before.label == segment.label {
                return Err(ScheduleError::RepeatsItsNeighbour {
                    at: segment.from.to_string(),
                });
            }
        }
        Ok(Self(segments))
    }
}

impl<P> Segments<P> {
    /// The segments, in order.
    pub fn segments(&self) -> &[Segment<P>] {
        &self.0
    }

    /// The segment in force at `at`, and its bounds.
    pub fn at(&self, at: WorldTime) -> InForce<'_, P> {
        let of_day = TimeOfDay::of(at);
        let midnight = at.seconds() - of_day;
        let last = self.0.len() - 1;
        match self
            .0
            .iter()
            .rposition(|segment| segment.from.seconds() <= of_day)
        {
            Some(index) => {
                let until = match self.0.get(index + 1) {
                    Some(next) => midnight + next.from.seconds(),
                    None => midnight + DAY + self.0[0].from.seconds(),
                };
                InForce {
                    segment: &self.0[index],
                    since: WorldTime::from_seconds(midnight + self.0[index].from.seconds()),
                    until: WorldTime::from_seconds(until),
                }
            }
            // Before the first segment of the day: the last one, begun yesterday, is still in force.
            None => InForce {
                segment: &self.0[last],
                since: WorldTime::from_seconds(midnight - DAY + self.0[last].from.seconds()),
                until: WorldTime::from_seconds(midnight + self.0[0].from.seconds()),
            },
        }
    }

    /// The same routine with every place renamed by `name`, or the first refusal.
    ///
    /// # Errors
    ///
    /// Whatever `name` refuses.
    pub fn map_places<Q, E>(
        &self,
        mut name: impl FnMut(&P) -> Result<Q, E>,
    ) -> Result<Segments<Q>, E> {
        let segments = self
            .0
            .iter()
            .map(|segment| {
                Ok(Segment {
                    from: segment.from,
                    place: name(&segment.place)?,
                    label: segment.label.clone(),
                })
            })
            .collect::<Result<Vec<_>, E>>()?;
        // Renaming keeps every rule except "neighbours differ", which two keys naming one place could
        // break; the routine was valid under its old names and the names are injective here (keys to
        // the ids they resolve to), so it stays valid.
        Ok(Segments(segments))
    }
}

impl<'de, P: Deserialize<'de> + PartialEq> Deserialize<'de> for Segments<P> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let segments = Vec::<Segment<P>>::deserialize(deserializer)?;
        Self::new(segments).map_err(D::Error::custom)
    }
}
