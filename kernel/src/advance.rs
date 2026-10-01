//! Advancing a world: moving its clock to the next instant something is due, and firing what is due.
//!
//! ```text
//! advance_to(until)   fire every instant at or before `until`, in order; the clock ends at `until`
//! step()              fire exactly the next instant, if there is one
//! ```
//!
//! # A logical instant
//!
//! Everything scheduled at one [`WorldTime`] is one logical instant (SD-3). Its entries fire in
//! sequence order — the order they were queued in — and each is reduced completely, through the
//! same synchronous, registration-ordered reduction a request goes through, before the next one
//! fires. A fact a reaction emits is reduced in the same instant; a fact it *defers* lands strictly
//! later and waits its turn. That is what makes a wage paid in the instant it falls due rather than
//! a tick later, while every deferral still goes through one ordered queue.
//!
//! Each fired entry has the same cascade budget a request has ([`CASCADE_DEPTH_LIMIT`]), and the
//! instant itself cannot run forever: nothing may be scheduled *at* the instant being fired, only
//! strictly after it, so an instant is a finite number of bounded chains (step-04 §8 F-10).
//!
//! # Idle time is skipped
//!
//! The clock jumps from one due instant to the next. A world with nothing due for four simulated
//! hours spends no work on them; [`Advanced::instants`] counts the instants that actually ran, which
//! is how a test sees it (SD-2, `AC-11`).
//!
//! # An error is a system's bug
//!
//! As with dispatch: an `Err` out of advancing means a system broke its own contract while handling
//! an entry, after some of that entry's writes may have landed. The world is to be stopped and the
//! error reported, not retried ([`crate::dispatch`] documents why reduction is not transactional).
//!
//! [`CASCADE_DEPTH_LIMIT`]: crate::CASCADE_DEPTH_LIMIT

use mineworld_contracts::{EventEnvelope, WorldTime};

use crate::error::KernelError;
use crate::schedule::Scheduled;
use crate::world::World;

/// What advancing a world produced.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[must_use]
pub struct Advanced {
    events: Vec<EventEnvelope>,
    instants: u64,
    skipped: u64,
}

impl Advanced {
    /// Every fact recorded while advancing, in recording order — instant by instant, entry by entry,
    /// and within an entry the fact itself followed by each generation of reaction.
    pub fn events(&self) -> &[EventEnvelope] {
        &self.events
    }

    /// The facts, for a caller that has to move them onward.
    pub fn into_events(self) -> Vec<EventEnvelope> {
        self.events
    }

    /// How many distinct instants had something due and were fired. Instants with nothing due are
    /// never visited, so this is the measure of idle time skipped.
    pub const fn instants(&self) -> u64 {
        self.instants
    }

    /// How many due entries were not fired because the system that would have acted is disabled.
    pub const fn skipped(&self) -> u64 {
        self.skipped
    }

    fn absorb(&mut self, other: Self) {
        self.events.extend(other.events);
        self.instants += other.instants;
        self.skipped += other.skipped;
    }
}

impl World {
    /// Fires every instant at or before `until`, in order, and leaves the clock at `until`.
    ///
    /// Refused, changing nothing, if `until` is earlier than the world's clock. Advancing to the
    /// instant the world is already at fires anything still due at it — which is how a caller
    /// clears the way before dispatching a request at that instant (see [`World::dispatch`]).
    pub fn advance_to(&mut self, until: WorldTime) -> Result<Advanced, KernelError> {
        self.run_at_least(until)?;
        let mut advanced = Advanced::default();
        while let Some(next) = self.next_instant().filter(|next| *next <= until) {
            advanced.absorb(self.fire_instant(next)?);
        }
        self.run_at(until)?;
        Ok(advanced)
    }

    /// Fires exactly the next instant anything is due, moving the clock there. `None` when nothing
    /// is scheduled.
    pub fn step(&mut self) -> Result<Option<Advanced>, KernelError> {
        let Some(next) = self.next_instant() else {
            return Ok(None);
        };
        self.fire_instant(next).map(Some)
    }

    /// Checks that `until` is a legal destination before anything fires, so a refusal changes
    /// nothing.
    fn run_at_least(&self, until: WorldTime) -> Result<(), KernelError> {
        if self.clock_has_started() && until < self.now() {
            return Err(KernelError::ClockWouldMoveBackwards {
                now: self.now(),
                at: until,
            });
        }
        Ok(())
    }

    /// Moves the clock to `at` and fires every entry due there, in sequence order, each fully
    /// reduced before the next.
    fn fire_instant(&mut self, at: WorldTime) -> Result<Advanced, KernelError> {
        self.run_at(at)?;
        let mut advanced = Advanced {
            instants: 1,
            ..Advanced::default()
        };
        while let Some((_, item)) = self.pop_due(at) {
            let mut dispatcher = self.dispatcher(at);
            let fired = match item {
                Scheduled::Fact(deferral) => dispatcher.fire_fact(deferral)?,
            };
            if !fired {
                advanced.skipped += 1;
            }
            advanced.events.extend(dispatcher.into_recorded());
        }
        Ok(advanced)
    }
}
