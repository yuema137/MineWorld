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

/// Builds a seat's controller, bound at the given world instant: at the server's start, and every
/// time the seat returns to its controller after a person drove it. A controller is always built
/// afresh, never resumed from memory (`PROTOCOL.md` §4.2).
pub type HostedFactory = Box<dyn Fn(WorldTime) -> Box<dyn HostedController>>;

/// A seat's bound controller and the instant it is next due.
pub(crate) struct HostedSlot {
    controller: Box<dyn HostedController>,
    next: WorldTime,
}

impl HostedSlot {
    /// A controller built from `factory`, bound at `at`.
    pub(crate) fn bind(factory: &HostedFactory, at: WorldTime) -> Self {
        let controller = factory(at);
        let next = controller.next_consult(at);
        Self { controller, next }
    }

    /// When this controller is next due.
    pub(crate) const fn next(&self) -> WorldTime {
        self.next
    }

    /// Asks the controller, and moves its next consult past `now`: a consult that was missed — the
    /// clock moved faster than the ticks — is skipped, never queued.
    pub(crate) fn decide(
        &mut self,
        observation: &WireObservation,
        now: WorldTime,
    ) -> Option<ActionRequest> {
        let request = self.controller.decide(observation);
        self.next = self.controller.next_consult(now.max(self.next));
        request
    }

    /// Tells the controller what became of its request.
    pub(crate) fn answered(&mut self, answer: &HostedAnswer) {
        self.controller.answered(answer);
    }
}
