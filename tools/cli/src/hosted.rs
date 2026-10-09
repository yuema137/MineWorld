//! The rule controllers, adapted onto the server's in-server seam (`DECISIONS.md` `ARC-42`).
//!
//! ```text
//! --agent SEAT   ReactiveSeat   RuleController::since(bound): answers what it hears while bound,
//!                               consulted every wall second (F-13)
//! --town         PacedSeat      PacedRuleController::new(seed, pace × scale): seat k consulted at
//!                               genesis + k + m·pace·scale — `run`'s lattice, with --pace in wall
//!                               seconds
//! ```
//!
//! Cadence is wall time (operator ruling QTW-13, 2026-10-08, reversing QS11B-4): a hosted Person
//! walks one stride per consult and walking is embodied, so `--time-scale` must never make it walk or
//! talk faster. The seam stays in world time; an adapter states its wall cadence as `cadence × scale`
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
    ActionRequest, ActionResult, ActionTypeId, EntityKey, SimDuration, WorldTime,
};
use mineworld_rule_controller::{PacedRuleController, RuleController};
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

/// `--town`: the paced rule controller, on `run`'s consult lattice with the server's pace.
pub struct PacedSeat {
    controller: PacedRuleController,
    /// `genesis + k`: this seat's first instant on the lattice.
    first: i64,
    /// The pace in world seconds: `--pace` wall seconds times the time scale.
    pace: i64,
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
        let pace = i64::from(pacing.pace.get()) * i64::from(pacing.scale.get());
        Self {
            controller: PacedRuleController::new(pacing.seed, SimDuration::from_seconds(pace)),
            // Seat k's offset is k wall seconds too, so seats stay spread across ticks at any scale.
            first: genesis.seconds().saturating_add(
                i64::try_from(k)
                    .unwrap_or(i64::MAX)
                    .saturating_mul(i64::from(pacing.scale.get())),
            ),
            pace,
            counting: Counting::new(tally),
        }
    }
}

impl HostedController for PacedSeat {
    /// The first `genesis + k + m·pace` (m ≥ 0) strictly after `after`.
    fn next_consult(&self, after: WorldTime) -> WorldTime {
        let after = after.seconds();
        if after < self.first {
            return WorldTime::from_seconds(self.first);
        }
        let rounds = (after - self.first) / self.pace + 1;
        WorldTime::from_seconds(self.first.saturating_add(rounds.saturating_mul(self.pace)))
    }

    fn decide(&mut self, observation: &WireObservation) -> Option<ActionRequest> {
        let decided = self.controller.decide(observation);
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
}
