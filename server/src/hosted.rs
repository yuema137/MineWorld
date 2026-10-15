//! In-server controllers: the seam a controller the server runs on the world thread is written to.
//!
//! ```text
//! --agent SEAT, --town    the composition root adapts a controller crate onto HostedController and
//!                         registers a factory per seat (HostedWorld::hosting)
//! the world thread        on every tick, consults each hosted seat whose instant is due: computes
//!                         that seat's observation through the same perception call a session gets,
//!                         asks `decide`, and submits the request through the session's own path
//! ```
//!
//! A hosted controller is not a privileged path. It skips the socket and nothing else: its request
//! meets the same actor check, receives a server-allocated `ActionId` and instant, and is journaled
//! before it is answered (`docs/DECISIONS.md` `ARC-42`). It is also not a client, and is not counted as
//! one.
//!
//! # Synchronous and bounded
//!
//! `decide` runs on the thread that owns the world, so it must return in microseconds and may not
//! block, wait or call out. A controller that needs to wait — a language model — connects as a
//! session over the protocol instead, so that the world never waits for it (`ARCHITECTURE.md` §9).
//!
//! The server names no controller crate: this trait is all it knows of one.

use std::num::NonZeroU32;

use mineworld_contracts::{ActionRequest, WorldTime};

use crate::host::Submitted;
use crate::protocol::{RefusalCode, WireObservation};

/// A controller the server runs on the world thread, driving one seat's Person.
pub trait HostedController: 'static {
    /// The first world instant strictly after `after` at which this controller wants to be consulted.
    ///
    /// A paced controller answers from its lattice; a reactive one answers `after` plus one second,
    /// that is, every second the clock moves.
    fn next_consult(&self, after: WorldTime) -> WorldTime;

    /// What the Person attempts, from its own observation only (`INV-13`) — or nothing, which is the
    /// usual answer. Never given a world handle.
    fn decide(&mut self, observation: &WireObservation) -> Option<ActionRequest>;

    /// What became of the request [`decide`](Self::decide) just returned. Default: nothing to do.
    fn answered(&mut self, _answer: &HostedAnswer) {}

    /// The host changed the time scale while the world runs (`ARC-69` item 2). `pending` is the instant
    /// the server has moved this controller's pending consult to; from it on, [`next_consult`](Self::next_consult)
    /// answers at `scale`. Required: a controller that ignored a rescale would keep its old cadence silently.
    fn rescaled(&mut self, pending: WorldTime, scale: NonZeroU32);
}

/// What a controller is bound with: the instant it is bound at, and the time scale in force then. A
/// seat released after a live rescale is rebound at the scale in force, not the one at the start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostedBinding {
    /// The world instant the controller is bound at.
    pub at: WorldTime,
    /// World seconds per wall second in force at the binding.
    pub scale: NonZeroU32,
}

/// What became of a hosted controller's request: exactly what a session would have been told.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostedAnswer {
    /// The world considered it: the identity the server allocated, and the world's answer.
    Answered(Submitted),
    /// It was refused before the world considered it — an actor that is not this seat's Person, or a
    /// world that cannot go on.
    Refused(RefusalCode),
}

/// Builds a seat's controller, bound as its [`HostedBinding`] says: at the server's start, and every
/// time the seat returns to its controller after a person drove it. A controller is always built
/// afresh, never resumed from memory (`PROTOCOL.md` §4.2).
pub type HostedFactory = Box<dyn Fn(HostedBinding) -> Box<dyn HostedController>>;

/// A seat's bound controller and the instant it is next due.
pub(crate) struct HostedSlot {
    controller: Box<dyn HostedController>,
    next: WorldTime,
}

impl HostedSlot {
    /// A controller built from `factory`, bound at `at` with the scale `scale`.
    pub(crate) fn bind(factory: &HostedFactory, at: WorldTime, scale: NonZeroU32) -> Self {
        let controller = factory(HostedBinding { at, scale });
        let next = controller.next_consult(at);
        Self { controller, next }
    }

    /// When this controller is next due.
    pub(crate) const fn next(&self) -> WorldTime {
        self.next
    }

    /// Asks the controller, and moves its next consult past `now`. The chain starts from the instant
    /// this consult was **due**, never from the tick's reading (`ARC-69` item 3): in-server instants
    /// do not follow tick jitter. A consult that was missed — the clock moved faster than the ticks —
    /// is skipped, never queued.
    pub(crate) fn decide(
        &mut self,
        observation: &WireObservation,
        now: WorldTime,
    ) -> Option<ActionRequest> {
        let request = self.controller.decide(observation);
        let mut next = self.controller.next_consult(self.next);
        while next <= now {
            next = self.controller.next_consult(next);
        }
        self.next = next;
        request
    }

    /// Tells the controller what became of its request.
    pub(crate) fn answered(&mut self, answer: &HostedAnswer) {
        self.controller.answered(answer);
    }

    /// The host changed the scale at world instant `at` (`ARC-69` item 2), from `from` to `to`. A pending
    /// consult still in the future moves by the same wall wait, in the new world seconds:
    /// `at + ⌈(next − at)·to/from⌉`. A consult already due stays where it is. The controller is always told.
    pub(crate) fn rescale(&mut self, at: WorldTime, from: NonZeroU32, to: NonZeroU32) {
        if self.next > at {
            let wait = i128::from(self.next.seconds()) - i128::from(at.seconds());
            let scaled =
                (wait * i128::from(to.get()) + i128::from(from.get()) - 1) / i128::from(from.get());
            let moved = i128::from(at.seconds()).saturating_add(scaled);
            self.next = WorldTime::from_seconds(i64::try_from(moved).unwrap_or(i64::MAX));
        }
        self.controller.rescaled(self.next, to);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn non_zero(value: u32) -> NonZeroU32 {
        NonZeroU32::new(value).expect("non-zero")
    }

    /// A controller whose cadence is a fixed period, which records the scale it was told of.
    struct Every {
        period: i64,
    }

    impl HostedController for Every {
        fn next_consult(&self, after: WorldTime) -> WorldTime {
            WorldTime::from_seconds(after.seconds() + self.period)
        }
        fn decide(&mut self, _observation: &WireObservation) -> Option<ActionRequest> {
            None
        }
        fn rescaled(&mut self, _pending: WorldTime, scale: NonZeroU32) {
            self.period = i64::from(scale.get());
        }
    }

    fn slot(period: i64, next: i64) -> HostedSlot {
        HostedSlot {
            controller: Box::new(Every { period }),
            next: WorldTime::from_seconds(next),
        }
    }

    /// SD-TWc-3: a pending consult in the future keeps its wall wait; `⌈·⌉` rounds it up in world seconds.
    #[test]
    fn a_pending_consult_keeps_its_wall_wait_across_a_rescale() {
        // 100 world s at 12× is 8⅓ wall s; at 24× that is 200 world s.
        let mut up = slot(12, 1_100);
        up.rescale(WorldTime::from_seconds(1_000), non_zero(12), non_zero(24));
        assert_eq!(up.next(), WorldTime::from_seconds(1_200));
        // 101 world s at 12× is 8.41 wall s; at 6× that is 50.5 world s, rounded up to 51.
        let mut down = slot(12, 1_101);
        down.rescale(WorldTime::from_seconds(1_000), non_zero(12), non_zero(6));
        assert_eq!(down.next(), WorldTime::from_seconds(1_051));
    }

    /// SD-TWc-3: a consult already due stays due; the controller is told the new scale all the same.
    #[test]
    fn a_consult_already_due_stays_due_and_the_controller_is_told() {
        let mut due = slot(12, 990);
        due.rescale(WorldTime::from_seconds(1_000), non_zero(12), non_zero(24));
        assert_eq!(due.next(), WorldTime::from_seconds(990));
        due.decide(&alone(), WorldTime::from_seconds(1_000));
        assert_eq!(
            due.next(),
            WorldTime::from_seconds(1_014),
            "from the instant it was due (990), one period at the new scale 24 is past now"
        );
    }

    /// SD-TWc-4: the next instant is the due instant plus whole periods, not the tick's reading plus one.
    #[test]
    fn a_missed_consult_skips_forward_from_the_instant_it_was_due() {
        let mut late = slot(5, 100);
        late.decide(&alone(), WorldTime::from_seconds(117));
        assert_eq!(
            late.next(),
            WorldTime::from_seconds(120),
            "due at 100, consulted at 117: the next is 120, not 122"
        );
    }

    fn alone() -> WireObservation {
        use mineworld_contracts::{EntityId, Observation};
        Observation::new(EntityId::from_raw(3), WorldTime::EPOCH)
    }
}
