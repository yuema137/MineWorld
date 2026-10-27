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

use mineworld_contracts::{Component, Observation, PlaceId, SimDuration, WorldTime};
use mineworld_movement::{Destination, WALK_STRIDE, Way, Ways};
use mineworld_schedule::{Agenda, AgendaLabel, DisclosedAgenda, NextPart};
use schemars::JsonSchema;
use serde::de::{Error as DeError, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::read::{disclosed, heading};

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
        // Set out already: a walk of mine that goes to the part's place keeps the commute due, so a shorter
        // estimate from where the walk has got to cannot hand the consult to an errand that turns me
        // round (S35 N-h, D-Nh-8: a hungry walker went to and fro between the shop and the station).
        let set_out = heading(observation)
            == Some(Destination::Place(mineworld_contracts::Location::in_place(
                next.place,
            )));
        ((set_out || now + pace.seconds() > leave_by) && now < next.from.seconds())
            .then_some(next.place)
    }
}

/// The next part of my day, when it is punctual, still ahead, and somewhere I am not: the only part a
/// commute is ever planned for.
pub fn pending(observation: &Observation<Value>) -> Option<NextPart> {
    let next = next_part(observation)?;
    let here = observation.self_location().map(|here| here.place());
    (next.punctual && here != Some(next.place) && observation.at() < next.from).then_some(next)
}

/// Whether I am already at a punctual next part's place shortly before it begins: arrived early, I wait
/// there, so the part in force no longer calls me away (`ARC-282`). Never in `at-boundary` mode.
///
/// "Shortly" is the time a round trip to the part in force would take before the commute calls me back:
///
/// ```text
/// holds at t  ⇔  mode = plan ∧ next.punctual ∧ I am in next.place ∧ t < next.from
///                ∧ t + pace > next.from − window
/// window       = min(cap, estimate(way back) + lead(way back))   way back: the disclosed way to the
///                                                                 place of the part in force
/// window       = cap                                             no such way disclosed, or that part
///                                                                 is here
/// ```
///
/// A commute's own arrival is always inside the window (it set out `lead` before `from` and walked about
/// `estimate`), so a person who commuted waits as before. A person who came early for another reason —
/// an errand at a shop that is also their workplace, whose food sold out before they reached the counter
/// (F-Nh-5) — is no longer held for the whole cap: while walking back and setting out again still fits,
/// the part in force has its day. Nothing names a place, a person or a world.
pub fn holds(
    observation: &Observation<Value>,
    commute: &Commute,
    pace: SimDuration,
    stride: SimDuration,
) -> bool {
    let now = observation.at().seconds();
    commute.mode == Mode::Plan
        && next_part(observation).is_some_and(|next| {
            let from = next.from.seconds();
            next.punctual
                && now < from
                && now + pace.seconds() > from - hold_window(observation, commute, &next, stride)
                && observation.self_location().map(|here| here.place()) == Some(next.place)
        })
}

/// [`holds`]' window, in world seconds: the round trip to the part in force, at most the cap.
fn hold_window(
    observation: &Observation<Value>,
    commute: &Commute,
    next: &NextPart,
    stride: SimDuration,
) -> i64 {
    let cap = commute.cap_minutes * 60;
    disclosed_agenda(observation)
        .map(|disclosed| disclosed.agenda.place())
        .filter(|place| *place != next.place)
        .and_then(|place| way_to(observation, place))
        .map_or(cap, |way| {
            cap.min(estimate(&way, stride).saturating_add(commute.lead(Some(&way), stride)))
        })
}

/// The next appointment of the observer's day: the next part schedule discloses with the agenda, or the
/// next start of the shift employment discloses to the employee, whichever binds first
/// (`pr-s35-nh-hands-and-errands.md` R6, F-Nh-1).
///
/// A shift is a time and a place one must be at, whatever the routine says, so it counts as a punctual
/// part at the workplace:
///
/// ```text
/// no shift disclosed                         the agenda's next part, unchanged
/// next part at the workplace, from ≤ shift   that part, punctual (the earlier from at the place wins)
/// next part punctual, from ≤ shift           that part (an earlier appointment comes first)
/// otherwise                                  the shift: workplace, its next start, punctual
/// ```
///
/// So a world author who forgets `punctual: true` on a job's part of the day still has an employee who
/// sets out in time; nothing names a job, a place or a world.
pub fn next_part(observation: &Observation<Value>) -> Option<NextPart> {
    let agenda = agenda_next(observation);
    let Some(shift) = next_shift(observation) else {
        return agenda;
    };
    match agenda {
        Some(next) if next.place == shift.place && next.from <= shift.from => Some(NextPart {
            punctual: true,
            ..next
        }),
        Some(next) if next.punctual && next.from <= shift.from => Some(next),
        _ => Some(shift),
    }
}

/// The next part of the observer's day, as schedule discloses it with the agenda.
fn agenda_next(observation: &Observation<Value>) -> Option<NextPart> {
    disclosed_agenda(observation)?.next
}

/// The part of the observer's day in force and the next, as schedule discloses them.
fn disclosed_agenda(observation: &Observation<Value>) -> Option<DisclosedAgenda> {
    let me = observation.entity(observation.observer())?;
    let record = me
        .components()
        .iter()
        .find(|record| *record.component_type() == Agenda::COMPONENT_TYPE)?;
    serde_json::from_value(record.payload_for::<Agenda>().ok()?.clone()).ok()
}

/// The component type employment discloses a person's own job on, by its slug: employment's crate
/// links a market pack, which this controller may not depend on (`ARC-35` check 2).
const EMPLOYMENT: &str = "employment";

/// Seconds in a day.
const DAY: i64 = 86_400;

/// The part of employment's disclosed record this controller reads: where and from when the job is.
#[derive(Deserialize)]
struct DisclosedEmployment {
    job: DisclosedJob,
}

#[derive(Deserialize)]
struct DisclosedJob {
    workplace: PlaceId,
    from: String,
}

/// The next start of my disclosed shift, strictly after now, as a punctual part at the workplace;
/// nothing without a disclosed job (a world without employment decides exactly as before).
pub fn next_shift(observation: &Observation<Value>) -> Option<NextPart> {
    let me = observation.entity(observation.observer())?;
    let record = me
        .components()
        .iter()
        .find(|record| record.component_type().as_str() == EMPLOYMENT)?;
    let employment: DisclosedEmployment = serde_json::from_value(record.payload().clone()).ok()?;
    let (hours, minutes) = employment.job.from.split_once(':')?;
    let of_day = hours.parse::<i64>().ok()? * 3_600 + minutes.parse::<i64>().ok()? * 60;
    let now = observation.at().seconds();
    let mut from = now - now.rem_euclid(DAY) + of_day;
    if from <= now {
        from += DAY;
    }
    Some(NextPart {
        place: employment.job.workplace,
        label: AgendaLabel::new(SHIFT_LABEL).ok()?,
        from: WorldTime::from_seconds(from),
        punctual: true,
    })
}

/// The label a shift is given as a part of the day.
const SHIFT_LABEL: &str = "work";

/// The way to `place` that movement disclosed to the observer, if it did.
pub fn way_to(observation: &Observation<Value>, place: PlaceId) -> Option<Way> {
    let me = observation.entity(observation.observer())?;
    let Ways(ways) = disclosed::<Ways>(me)?;
    ways.into_iter().find(|way| way.place == place)
}
