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
    /// The person means to *be there* when this part begins (`ARC-280`). Content the pack carries and
    /// never interprets: schedule's own behaviour does not change. Not written when false, so a
    /// routine without a punctual part keeps its bytes (I-LE-5).
    #[serde(default, skip_serializing_if = "is_false")]
    punctual: bool,
}

/// Serde's `skip_serializing_if` for a flag that is absent unless set.
#[allow(clippy::trivially_copy_pass_by_ref)]
const fn is_false(flag: &bool) -> bool {
    !*flag
}

impl<P> Segment<P> {
    /// A segment, not punctual.
    pub const fn new(from: TimeOfDay, place: P, label: AgendaLabel) -> Self {
        Self {
            from,
            place,
            label,
            punctual: false,
        }
    }

    /// The same segment, punctual or not.
    #[must_use]
    pub const fn punctually(mut self, punctual: bool) -> Self {
        self.punctual = punctual;
        self
    }

    /// Whether the person means to be at the place when it begins, rather than set out then.
    pub const fn punctual(&self) -> bool {
        self.punctual
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
                    punctual: segment.punctual,
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

/// One part of a person's day: from when, where, and what for.
///
/// [`Segment`]'s fields with the place by key — what the generated reference describes (`ARC-90`).
#[derive(schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct AuthoredSegment {
    /// When this part of the day begins, every day.
    from: TimeOfDay,
    /// Where the person spends it: a place key.
    #[schemars(with = "mineworld_authoring::schema::Key")]
    place: EntityKey,
    /// What it is for, in the world's own words: the label others see.
    label: AgendaLabel,
    /// The person means to be at the place when this part begins, so whoever drives them sets out
    /// early enough to arrive by `from`. Without it, `from` is when they set out. Marking a part
    /// that follows a shift punctual makes the person leave that shift early.
    #[serde(default)]
    punctual: bool,
}

/// A routine as authored: 2 to 24 parts of a day (`ARC-90`); the order and neighbour rules
/// [`Segments::new`] enforces are stated in the description.
impl schemars::JsonSchema for Segments<EntityKey> {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("Routine")
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let segment = generator.subschema_for::<AuthoredSegment>();
        schemars::json_schema!({
            "type": "array",
            "items": segment,
            "minItems": MIN_SEGMENTS,
            "maxItems": MAX_SEGMENTS,
            "description": "A person's day, in parts: each from a time, at a place, for something. \
                            Times strictly increase; two neighbouring parts (the last and the first \
                            included, since the day wraps) differ in place or label; the time before \
                            the first part belongs to the last.",
            "examples": [[
                { "from": "08:00", "place": "shop", "label": "work", "punctual": true },
                { "from": "18:00", "place": "home", "label": "home" },
            ]],
        })
    }
}
