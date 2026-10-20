//! When to set out for a punctual part of the day: the shared arithmetic, and the commute policy as
//! data (`DECISIONS.md` `ARC-282`; `pr-le-leave-early.md` SD-LE-5, §0.4).
//!
//! The world discloses facts only: schedule the next part of my day (`agenda.next`, with `punctual`),
//! movement the way to each place (`ways`: doorway count and length, never a duration). The host
//! states its stride cadence (`Consult::stride`). This module turns them into a lead time, and a
//! [`Commute`] policy — parameters a world writes in `decisions.yaml`, never constants — says how much
//! slack and margin to keep and whether to plan at all:
//!
//! ```text
//! strides(way)   = ceil(length ÷ WALK_STRIDE) + doors + 1       a stride per doorway crossing, and one
//!                                                               for the first step after walk-to
//! estimate(way)  = strides(way) × stride                        stride = the host's cadence, world s
//! lead(way)      = min(cap, estimate × (1 000 + slack) ÷ 1 000 + margin)
//! lead(absent)   = cap                                          the place is not in `ways`
//! due at t       ⇔ mode = plan ∧ next.punctual ∧ I am not in next.place
//!                  ∧ t + pace > next.from − lead                waiting one more pace could make me late
//!                  ∧ t < next.from                              once it begins, it is the agenda
//! ```
//!
//! The guarantee: a person who sets out at the first due consult arrives by `from` whenever the walk
//! takes no longer than `estimate × (1 000 + slack) ÷ 1 000 + margin`. Integer arithmetic only; a pure
//! function of `(observation, pace, stride, policy)` — no clock, no state, no draw (`ARC-27`). Each
//! model decides with it in its own way: `rule` as a precedence rung, `motivated` as a scored option.

use core::fmt;

use mineworld_contracts::{Component, Observation, PlaceId, SimDuration};
use mineworld_movement::{WALK_STRIDE, Way, Ways};
use mineworld_schedule::{Agenda, DisclosedAgenda, NextPart};
use schemars::JsonSchema;
use serde::de::{Error as DeError, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::read::disclosed;

/// Whether the person plans the way at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    /// Set out early enough to arrive by the part's `from`, from the estimate.
    #[default]
    Plan,
    /// Set out when the part begins, as before punctual parts existed: arrive one walk late.
    AtBoundary,
}

/// Where a due departure stands among a model's hard rungs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Precedence {
    /// Before answering an invitation or a line: a due departure is asked first.
    #[default]
    First,
    /// After answering an invitation or a line.
    AfterReplies,
}

/// A commute policy: how early a person sets out for a punctual part of the day (§0.4). Every field
/// may be omitted and keeps its default; the defaults were fixed before any run (`ARC-23`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[schemars(extend("required" = Vec::<&str>::new()))]
pub struct Commute {
    /// `plan` (set out from the estimate) or `at-boundary` (set out when the part begins).
    pub mode: Mode,
    /// Per mille added to the estimate, 0 … 2 000: detours and replans.
    #[schemars(range(min = 0, max = 2_000))]
    pub slack: u32,
    /// Minutes kept in hand, 0 … 60.
    #[schemars(range(min = 0, max = 60))]
    pub margin_minutes: i64,
    /// Minutes, 15 … 480: the most a lead may be, and the lead when the place is not in `ways`.
    #[schemars(range(min = 15, max = 480))]
    pub cap_minutes: i64,
    /// `first` (a due departure before answering) or `after-replies`.
    pub precedence: Precedence,
}

impl Default for Commute {
    fn default() -> Self {
        Self {
            mode: Mode::Plan,
            slack: 500,
            margin_minutes: 5,
            cap_minutes: 120,
            precedence: Precedence::First,
        }
    }
}

/// The keys of a commute block.
const KEYS: &[&str] = &[
    "mode",
    "slack",
    "margin_minutes",
    "cap_minutes",
    "precedence",
];

/// A value inside `low ..= high`, or a refusal naming the bound, raised while the decoder stands on
/// the value (`DEP-10`).
fn within<E: DeError>(key: &str, value: i64, low: i64, high: i64) -> Result<i64, E> {
    if value < low || value > high {
        Err(E::custom(format!("'{key}' is {low} … {high}, not {value}")))
    } else {
        Ok(value)
    }
}

impl<'de> Deserialize<'de> for Commute {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(CommuteFields)
    }
}

struct CommuteFields;

impl<'de> Visitor<'de> for CommuteFields {
    type Value = Commute;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "a commute block ({})", KEYS.join(", "))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Commute, A::Error> {
        let mut commute = Commute::default();
        let mut seen: Vec<String> = Vec::new();
        while let Some(key) = map.next_key::<String>()? {
            if seen.contains(&key) {
                return Err(A::Error::custom(format!("'{key}' is written twice")));
            }
            match key.as_str() {
                "mode" => commute.mode = map.next_value()?,
                "precedence" => commute.precedence = map.next_value()?,
                "slack" => {
                    let slack = within::<A::Error>(&key, map.next_value()?, 0, 2_000)?;
                    commute.slack = u32::try_from(slack).map_err(A::Error::custom)?;
                }
                "margin_minutes" => {
                    commute.margin_minutes = within::<A::Error>(&key, map.next_value()?, 0, 60)?;
                }
                "cap_minutes" => {
                    commute.cap_minutes = within::<A::Error>(&key, map.next_value()?, 15, 480)?;
                }
                _ => return Err(A::Error::unknown_field(&key, KEYS)),
            }
            seen.push(key);
        }
        Ok(commute)
    }
}

/// The strides a way takes: one per [`WALK_STRIDE`] of its length, rounded up, one per doorway, and one
/// for the first step after `walk-to` is resolved.
pub fn strides(way: &Way) -> i64 {
    let length = i64::from(way.length.value()).max(0);
    let stride = i64::from(WALK_STRIDE.value());
    (length + stride - 1) / stride + i64::from(way.doors) + 1
}

/// How long a way probably takes, in world seconds, at the host's stride cadence.
pub fn estimate(way: &Way, stride: SimDuration) -> i64 {
    strides(way).saturating_mul(stride.seconds())
}

impl Commute {
    /// The lead, in world seconds: how long before a part begins the person sets out for it. The cap
    /// when the way is not known.
    pub fn lead(&self, way: Option<&Way>, stride: SimDuration) -> i64 {
        let cap = self.cap_minutes * 60;
        let Some(way) = way else {
            return cap;
        };
        let padded = estimate(way, stride).saturating_mul(1_000 + i64::from(self.slack)) / 1_000;
        cap.min(padded.saturating_add(self.margin_minutes * 60))
    }

    /// The instant by which to set out for `next`, given what the observation discloses.
    pub fn leave_by(
        &self,
        observation: &Observation<Value>,
        next: &NextPart,
        stride: SimDuration,
    ) -> i64 {
        let way = way_to(observation, next.place);
        next.from.seconds() - self.lead(way.as_ref(), stride)
    }

    /// The place to set out for now, if a commute is due at this consult (the rule above); nothing in
    /// `at-boundary` mode, with no punctual next part, or already there.
    pub fn due(
        &self,
        observation: &Observation<Value>,
        pace: SimDuration,
        stride: SimDuration,
    ) -> Option<PlaceId> {
        let next = pending(observation)?;
        if self.mode == Mode::AtBoundary {
            return None;
        }
        let now = observation.at().seconds();
        let leave_by = self.leave_by(observation, &next, stride);
        (now + pace.seconds() > leave_by && now < next.from.seconds()).then_some(next.place)
    }
}

/// The next part of my day, when it is punctual, still ahead, and somewhere I am not: the only part a
/// commute is ever planned for.
pub fn pending(observation: &Observation<Value>) -> Option<NextPart> {
    let next = next_part(observation)?;
    let here = observation.self_location().map(|here| here.place());
    (next.punctual && here != Some(next.place) && observation.at() < next.from).then_some(next)
}

/// Whether I am already at a punctual next part's place shortly before it begins — within the cap and
/// one pace of `from`, the longest a commute could have been planned for: arrived early, I wait there,
/// so the part in force no longer calls me away (`ARC-282`). Earlier than that, the part in force has
/// its day as usual. Never in `at-boundary` mode.
pub fn holds(observation: &Observation<Value>, commute: &Commute, pace: SimDuration) -> bool {
    let now = observation.at().seconds();
    commute.mode == Mode::Plan
        && next_part(observation).is_some_and(|next| {
            let from = next.from.seconds();
            next.punctual
                && now < from
                && now + pace.seconds() > from - commute.cap_minutes * 60
                && observation.self_location().map(|here| here.place()) == Some(next.place)
        })
}

/// The next part of the observer's day, as schedule discloses it with the agenda.
pub fn next_part(observation: &Observation<Value>) -> Option<NextPart> {
    let me = observation.entity(observation.observer())?;
    let record = me
        .components()
        .iter()
        .find(|record| *record.component_type() == Agenda::COMPONENT_TYPE)?;
    let disclosed: DisclosedAgenda =
        serde_json::from_value(record.payload_for::<Agenda>().ok()?.clone()).ok()?;
    disclosed.next
}

/// The way to `place` that movement disclosed to the observer, if it did.
pub fn way_to(observation: &Observation<Value>, place: PlaceId) -> Option<Way> {
    let me = observation.entity(observation.observer())?;
    let Ways(ways) = disclosed::<Ways>(me)?;
    ways.into_iter().find(|way| way.place == place)
}
