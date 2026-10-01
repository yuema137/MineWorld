//! The world clock: which instant a world is at, and the one rule about moving it.
//!
//! A world's clock counts simulated seconds ([`WorldTime`]) and nothing else. It does not know what
//! a day, an hour, a weekday or opening hours are: those are interpretations of seconds, and the
//! system that needs one owns it (`INV-12`, SD-7). The moment the kernel knew what 09:00 meant it
//! would have a calendar, and the next request would be holidays.
//!
//! It is also not wall time. Nothing here reads an OS clock: a world's instant moves only because
//! the world was *told* to move it — by dispatch, by advancing to the next scheduled instant, or by
//! assembly stating when the world begins. That is what lets a replayed world produce the same
//! facts at the same instants as the run it replays (`AC-12`). How fast a hosted world's seconds
//! pass in real time is the host's decision, not this type's.
//!
//! # The one rule
//!
//! **A clock that has started never moves backwards.** It starts at the first instant a world is
//! given — genesis, a restored snapshot, a dispatch or an advance — and from then on every instant
//! it is moved to is at or after the one it holds. A fact recorded at `t100` after a fact at `t200`
//! would be a history that runs backwards, and no order of reduction could make it consistent.

use mineworld_contracts::WorldTime;
use serde::{Deserialize, Serialize};

use crate::error::KernelError;

/// The instant a world is at.
///
/// Read-only outside the kernel: a world's time moves through [`World`](crate::World)'s own
/// operations, never by a caller setting it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct WorldClock {
    now: WorldTime,
    /// Whether the world has been given an instant yet. Before that, any first instant is
    /// acceptable — a world may begin at whatever moment its assembler states — and [`now`] reads
    /// as the epoch.
    ///
    /// [`now`]: WorldClock::now
    started: bool,
}

impl WorldClock {
    /// A clock that has not been given an instant yet.
    pub const fn new() -> Self {
        Self {
            now: WorldTime::EPOCH,
            started: false,
        }
    }

    /// The instant the world is at. The epoch, until the world is first given one.
    pub const fn now(&self) -> WorldTime {
        self.now
    }

    /// Whether the world has been given an instant yet.
    pub const fn has_started(&self) -> bool {
        self.started
    }

    /// Refuses an instant this clock may not move to, changing nothing.
    pub(crate) const fn check(&self, at: WorldTime) -> Result<(), KernelError> {
        if self.started && at.seconds() < self.now.seconds() {
            return Err(KernelError::ClockWouldMoveBackwards { now: self.now, at });
        }
        Ok(())
    }

    /// Moves to `at`, which is the clock's first instant or not earlier than the current one.
    pub(crate) fn advance_to(&mut self, at: WorldTime) -> Result<(), KernelError> {
        self.check(at)?;
        self.now = at;
        self.started = true;
        Ok(())
    }

    /// Replaces the clock with a persisted one — world assembly from a snapshot, which states the
    /// instant rather than moving to it.
    pub(crate) const fn restore(&mut self, now: WorldTime) {
        self.now = now;
        self.started = true;
    }
}
