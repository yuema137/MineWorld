//! The rule controllers, adapted onto the server's in-server seam (`DECISIONS.md` `ARC-42`).
//!
//! ```text
//! --agent SEAT   ReactiveSeat   RuleController::since(bound): answers what it hears while bound,
//!                               consulted every wall second (F-13)
//! --town         PacedSeat      PacedRuleController::new(seed, pace × scale): seat k consulted at
//!                               genesis + k + m·pace·scale — `run`'s lattice, with --pace in wall
//!                               seconds — and, while its person walks, asked for the next stride
//!                               at genesis + k + n·scale: once a wall second (ARC-75)
//! ```
//!
//! Cadence is wall time (operator ruling QTW-13, 2026-10-08, reversing QS11B-4): a hosted Person
//! walks one stride per wall second and walking is embodied, so `--time-scale` must never make it walk
//! or talk faster. The seam stays in world time; an adapter states its wall cadence as `cadence × scale`
//! world seconds, the form S19 reschedules on a live scale change (`step-19-time-weather.md` §4.4).
//!
//! `mineworld-server` names no controller crate and `mineworld-rule-controller` names no transport,
//! so the adapter lives in the only crate that depends on both: this binary. It also counts what
//! became of each seat's requests, for the operator's statistics line on shutdown — counted here,
//! because only a controller's adapter knows which action type it asked for; the server prints no
//! action type.

use std::collections::BTreeMap;
use std::fmt;
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex, PoisonError};

use mineworld_contracts::{
    Action, ActionRequest, ActionResult, ActionTypeId, EntityKey, SimDuration, WorldTime,
};
use mineworld_movement::{WalkStep, WalkTo};
use mineworld_rule_controller::{PacedRuleController, RuleController, walks};
use mineworld_server::{HostedAnswer, HostedController, WireObservation};

/// What one hosted seat's controllers did, across every rebinding of the seat.
#[derive(Debug, Default)]
pub struct SeatStatistics {
    consults: u64,
    accepted: BTreeMap<ActionTypeId, u64>,
    rejected: u64,
    refused: u64,
}

/// Shared between a seat's factory, every controller it builds, and the composition root that prints
/// it. The world thread is the only writer; the lock is held for an increment.
pub type Tally = Arc<Mutex<SeatStatistics>>;

impl fmt::Display for SeatStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let accepted: u64 = self.accepted.values().sum();
        write!(f, "{} consults, {accepted} accepted", self.consults)?;
        if !self.accepted.is_empty() {
            let by_type: Vec<String> = self
                .accepted
                .iter()
                .map(|(action_type, count)| format!("{action_type} {count}"))
                .collect();
            write!(f, " ({})", by_type.join(", "))?;
        }
        write!(f, ", {} rejected, {} refused", self.rejected, self.refused)
    }
}

/// Counts consults and outcomes for one controller, remembering what it asked for last.
struct Counting {
    tally: Tally,
    asked: Option<ActionTypeId>,
}

impl Counting {
    fn new(tally: &Tally) -> Self {
        Self {
            tally: Arc::clone(tally),
            asked: None,
        }
    }

    fn consulted(&mut self, decided: Option<ActionRequest>) -> Option<ActionRequest> {
        let mut statistics = self.tally.lock().unwrap_or_else(PoisonError::into_inner);
        statistics.consults += 1;
        self.asked = decided
            .as_ref()
            .map(|request| request.action_type().clone());
        decided
    }

    fn answered(&mut self, answer: &HostedAnswer) {
        let mut statistics = self.tally.lock().unwrap_or_else(PoisonError::into_inner);
        match answer {
            HostedAnswer::Answered(submitted) => match submitted.result() {
                ActionResult::Accepted { .. } => {
                    if let Some(action_type) = self.asked.take() {
                        *statistics.accepted.entry(action_type).or_insert(0) += 1;
                    }
                }
                _ => statistics.rejected += 1,
            },
            HostedAnswer::Refused(_) => statistics.refused += 1,
        }
    }
}

/// `--agent SEAT`: the reactive rule controller, answering only lines heard after it was bound.
pub struct ReactiveSeat {
    controller: RuleController,
    /// One wall second, in world seconds: the time scale.
    step: i64,
    counting: Counting,
}

impl ReactiveSeat {
    /// Bound at `at`: a line heard at or before it was said to whoever drove the Person before.
    /// Consulted once a wall second, `scale` world seconds.
    pub fn bound(at: WorldTime, scale: NonZeroU32, tally: &Tally) -> Self {
        Self {
            controller: RuleController::since(at),
            step: i64::from(scale.get()),
            counting: Counting::new(tally),
        }
    }
}

impl HostedController for ReactiveSeat {
    fn next_consult(&self, after: WorldTime) -> WorldTime {
        WorldTime::from_seconds(after.seconds().saturating_add(self.step))
    }

    fn decide(&mut self, observation: &WireObservation) -> Option<ActionRequest> {
        let decided = self.controller.decide(observation);
        self.counting.consulted(decided)
    }

    fn answered(&mut self, answer: &HostedAnswer) {
        self.counting.answered(answer);
    }
}

/// How often a hosted walker is asked for the next stride of its walk: once a **wall** second.
///
/// One `walk-step` carries at most `WALK_STRIDE` (1 340 mm), so one a wall second is 1.34 m/s on
/// screen whatever the time scale. That is the mean free walking speed of pedestrians (Weidmann 1993);
/// Bohannon (1997) measured a comfortable gait of 1.27–1.46 m/s for adults aged 20–59. This constant is
/// the adapter's, never a pack's: it is stated to the world-time seam as `EMBODIED_STEP × scale` world
/// seconds, QTW-13's form (`step-11-bodies.md` SD-N14; `DECISIONS.md` `ARC-42` note, `ARC-75`).
const EMBODIED_STEP: i64 = 1;

/// `--town`: the paced rule controller, on `run`'s consult lattice with the server's pace, and asked
/// for its walk's strides once a wall second while it walks.
pub struct PacedSeat {
    controller: PacedRuleController,
    /// `genesis + k`: this seat's first instant on the lattice.
    first: i64,
    /// The pace in world seconds: `--pace` wall seconds times the time scale.
    pace: i64,
    /// [`EMBODIED_STEP`] in world seconds: one wall second times the time scale. `pace` is a whole
    /// number of these, so every lattice instant is also a step instant.
    step: i64,
    /// Whether the person was walking at the last consult: the last observation disclosed their walk,
    /// or the last request asked for one or for its stride. Only then are step consults scheduled, so
    /// an idle seat costs what it did before walks existed.
    walking: bool,
    /// The instant of the last consult, to tell a lattice consult from a step consult when the world
    /// had already moved past the due instant (the runtime observes at `due.max(now)`).
    last: Option<i64>,
    counting: Counting,
}

/// What a paced seat is told: the seed, `--pace` in wall seconds, and the time scale.
#[derive(Clone, Copy)]
pub struct Pacing {
    pub seed: u64,
    pub pace: NonZeroU32,
    pub scale: NonZeroU32,
}

impl PacedSeat {
    /// Seat number `k` (roster order) of a world whose lattice begins at `genesis`.
    ///
    /// The controller is told its pace in world seconds, because its answering window is one pace of
    /// the world's time long (`PacedRuleController::new`).
    pub fn new(pacing: Pacing, genesis: WorldTime, k: usize, tally: &Tally) -> Self {
        let scale = i64::from(pacing.scale.get());
        let pace = i64::from(pacing.pace.get()) * scale;
        Self {
            controller: PacedRuleController::new(pacing.seed, SimDuration::from_seconds(pace)),
            // Seat k's offset is k wall seconds too, so seats stay spread across ticks at any scale.
            first: genesis
                .seconds()
                .saturating_add(i64::try_from(k).unwrap_or(i64::MAX).saturating_mul(scale)),
            pace,
            step: EMBODIED_STEP * scale,
            walking: false,
            last: None,
            counting: Counting::new(tally),
        }
    }

    /// The first `first + m·period` (m ≥ 0) strictly after `after`.
    fn next_on(&self, period: i64, after: i64) -> i64 {
        if after < self.first {
            return self.first;
        }
        let rounds = (after - self.first) / period + 1;
        self.first.saturating_add(rounds.saturating_mul(period))
    }

    /// Whether a lattice instant lies in `(last, at]`: whether this consult is the one `decide` is owed.
    fn on_lattice(&self, at: i64) -> bool {
        match self.last {
            None => at >= self.first,
            Some(last) => self.next_on(self.pace, last) <= at,
        }
    }
}

impl HostedController for PacedSeat {
    /// The first lattice instant `genesis + k + m·pace` strictly after `after`; while the person walks,
    /// the first step instant `genesis + k + n·step` instead, of which the lattice is a subset.
    fn next_consult(&self, after: WorldTime) -> WorldTime {
        let period = if self.walking { self.step } else { self.pace };
        WorldTime::from_seconds(self.next_on(period, after.seconds()))
    }

    /// On the lattice, `decide` — and, when it answers nothing for a walker, the next stride; between
    /// lattice instants, only the next stride (`step-11-bodies.md` SD-N14, N-D13).
    fn decide(&mut self, observation: &WireObservation) -> Option<ActionRequest> {
        let at = observation.at().seconds();
        let lattice = self.on_lattice(at);
        self.last = Some(at);
        let decided = if lattice {
            self.controller
                .decide(observation)
                .or_else(|| self.controller.step(observation))
        } else {
            self.controller.step(observation)
        };
        let asked_to_walk = decided.as_ref().is_some_and(|request| {
            *request.action_type() == WalkTo::ACTION_TYPE
                || *request.action_type() == WalkStep::ACTION_TYPE
        });
        // A step consult that answers nothing ends the stepping: the walk is over, or was refused.
        self.walking = if lattice || decided.is_some() {
            walks(observation) || asked_to_walk
        } else {
            false
        };
        self.counting.consulted(decided)
    }

    fn answered(&mut self, answer: &HostedAnswer) {
        self.counting.answered(answer);
    }
}

/// The operator's shutdown lines, one per hosted seat, in roster order.
pub fn report(tallies: &[(EntityKey, Tally)]) {
    for (seat, tally) in tallies {
        let statistics = tally.lock().unwrap_or_else(PoisonError::into_inner);
        println!("[world] hosted {seat}: {statistics}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn non_zero(value: u32) -> NonZeroU32 {
        NonZeroU32::new(value).expect("non-zero")
    }

    fn paced(k: usize, pace: u32, scale: u32) -> PacedSeat {
        let pacing = Pacing {
            seed: 0,
            pace: non_zero(pace),
            scale: non_zero(scale),
        };
        PacedSeat::new(pacing, WorldTime::from_seconds(1_000), k, &Tally::default())
    }

    fn consults(seat: &dyn HostedController, from: i64, count: usize) -> Vec<i64> {
        let mut at = WorldTime::from_seconds(from);
        (0..count)
            .map(|_| {
                at = seat.next_consult(at);
                at.seconds()
            })
            .collect()
    }

    /// At scale 1 the lattice is `run`'s: seat k at genesis + k + m·pace, each instant strictly after
    /// the last.
    #[test]
    fn a_paced_seat_is_consulted_on_run_s_lattice() {
        let seat = paced(2, 5, 1);
        assert_eq!(consults(&seat, 990, 4), [1_002, 1_007, 1_012, 1_017]);
        assert_eq!(
            seat.next_consult(WorldTime::from_seconds(1_008)).seconds(),
            1_012,
            "from between two instants, the next one"
        );
    }

    /// QTW-13: `--pace` and the reactive second are wall time. At scale 60 the consults are sixty
    /// times further apart in world time — the same number per wall second — so the time scale never
    /// makes a hosted Person walk or talk faster.
    #[test]
    fn cadence_is_wall_time_whatever_the_scale() {
        assert_eq!(
            consults(&paced(2, 5, 60), 990, 3),
            [1_120, 1_420, 1_720],
            "seat 2 is two wall seconds in, then every five wall seconds"
        );
        let reactive =
            ReactiveSeat::bound(WorldTime::from_seconds(0), non_zero(60), &Tally::default());
        assert_eq!(consults(&reactive, 1_000, 2), [1_060, 1_120]);
    }

    /// SD-N14: a walking seat is asked for a stride once a **wall** second — every `scale` world
    /// seconds, at offset `k·scale` — at every scale, and its lattice instants are among them.
    #[test]
    fn a_walking_seat_steps_once_a_wall_second_whatever_the_scale() {
        for scale in [1_i64, 12, 60] {
            let mut seat = paced(2, 5, u32::try_from(scale).expect("small"));
            let lattice = consults(&seat, 990, 3);
            seat.walking = true;
            let steps = consults(&seat, 990, 16);
            let first = 1_000 + 2 * scale;
            let expected: Vec<i64> = (0..16).map(|n| first + n * scale).collect();
            assert_eq!(
                steps, expected,
                "scale {scale}: one consult per wall second"
            );
            for instant in lattice {
                assert!(
                    steps.contains(&instant),
                    "scale {scale}: lattice instant {instant} is a step instant"
                );
            }
        }
    }

    /// Idle seats stay cheap: a seat that is not walking is consulted on its lattice only.
    #[test]
    fn a_seat_that_is_not_walking_is_consulted_on_the_lattice_only() {
        let seat = paced(2, 5, 12);
        assert!(!seat.walking, "a seat starts idle");
        assert_eq!(consults(&seat, 990, 3), [1_024, 1_084, 1_144]);
    }

    /// A consult is the lattice's when a lattice instant lies between the previous consult and this
    /// one, even if the world had moved past the due instant (the runtime observes at
    /// `due.max(now)`); otherwise it is a step consult.
    #[test]
    fn a_lattice_consult_is_recognized_even_when_observed_late() {
        let mut seat = paced(2, 5, 12);
        assert!(seat.on_lattice(1_024), "the first consult is the lattice's");
        seat.last = Some(1_024);
        assert!(
            !seat.on_lattice(1_036),
            "a wall second later: a step consult"
        );
        assert!(
            !seat.on_lattice(1_072),
            "still before the next lattice instant 1 084"
        );
        assert!(seat.on_lattice(1_084), "the lattice instant itself");
        assert!(
            seat.on_lattice(1_090),
            "observed after it, with no consult between"
        );
        seat.last = Some(1_084);
        assert!(
            !seat.on_lattice(1_096),
            "the next step after the lattice consult"
        );
    }

    /// The observer at `at`, alone, with their walk disclosed or not and `walk-step` offered or not.
    fn alone(at: i64, walking: bool) -> WireObservation {
        use mineworld_contracts::{
            Affordance, ComponentRecord, EntityId, EntityType, Observation, PerceivedEntity,
            SpatialRequirement,
        };
        let me = EntityId::from_raw(3);
        let mut person = PerceivedEntity::new(me, EntityType::Person);
        let mut offers = Vec::new();
        if walking {
            person =
                person.with_components(vec![ComponentRecord::new::<mineworld_movement::Walking>(
                    me,
                    serde_json::json!({ "destination": { "person": 4 }, "next": [] }),
                )]);
            offers.push(Affordance::available(
                WalkStep::ACTION_TYPE,
                None,
                SpatialRequirement::NONE,
            ));
        }
        Observation::new(me, WorldTime::from_seconds(at))
            .perceiving(vec![person])
            .offering(offers)
    }

    /// A step consult asks for the stride while the walk is disclosed, and the first one that answers
    /// nothing — the walk is over — returns the seat to its lattice.
    #[test]
    fn stepping_continues_while_the_walk_is_disclosed_and_stops_when_it_ends() {
        let mut seat = paced(2, 5, 12);
        let asked = seat.decide(&alone(1_024, true));
        assert!(
            asked.is_some_and(|request| *request.action_type() == WalkStep::ACTION_TYPE),
            "a walker with nothing else to do on the lattice steps"
        );
        assert_eq!(
            seat.next_consult(WorldTime::from_seconds(1_024)).seconds(),
            1_036
        );
        let asked = seat.decide(&alone(1_036, true));
        assert!(asked.is_some_and(|request| *request.action_type() == WalkStep::ACTION_TYPE));
        assert_eq!(
            seat.decide(&alone(1_048, false)),
            None,
            "the walk has ended"
        );
        assert_eq!(
            seat.next_consult(WorldTime::from_seconds(1_048)).seconds(),
            1_084,
            "back on the lattice"
        );
    }
}
