//! The world thread's half of the admin surface: the seats report, kick, release, and the host
//! clock (`PROTOCOL.md` §11, `docs/DECISIONS.md` `ARC-44`).
//!
//! Host state only. Nothing here dispatches, advances, states a fact or moves the world's revision
//! (`ARC-40`, step-12 I-4). A pacing change — a pause, a resume, a rescale — is host state too; its
//! journaling is the host journal's, not a fact's (`ARC-69`). The permission check and its delay are the
//! transport's (`crate::admin`); by the time a command arrives here it is the host's.

use std::num::NonZeroU32;
use std::time::Instant;

use mineworld_contracts::{EntityKey, WorldTime};
use mineworld_persistence::{PacingChange, PersistError};

use super::WorldRuntime;
use crate::host::Hosted;
use crate::protocol::{ClockState, ClosingReason, SessionId};
use crate::seats::SeatReport;

/// A change to the host clock, as `POST /admin/clock` asks for it: either field, or both. Applied in
/// one world command, the scale first (`PROTOCOL.md` §11.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ClockChange {
    /// Pause (`true`) or resume (`false`); `None` leaves the pause state as it is.
    pub(crate) paused: Option<bool>,
    /// The new time scale, already checked against its range and the `--solo` rule; `None` leaves it.
    pub(crate) time_scale: Option<NonZeroU32>,
}

/// What the admin surface asks of the world thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ControlCommand {
    /// Every seated connection's drop count.
    Sessions,
    /// Every seat's binding.
    Seats,
    /// Close a connection and return its seat to its default, with no hold.
    Kick(SessionId),
    /// Return a seat to its default.
    Release(EntityKey),
    /// The host clock.
    Clock,
    /// Change the host clock: its scale, its pause state, or both.
    Change(ClockChange),
}

/// One seated connection as only the world thread knows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SessionCount {
    pub(crate) session: SessionId,
    /// Observations the world dropped because this connection was not reading.
    pub(crate) observations_dropped: u64,
}

/// The world thread's answer to a [`ControlCommand`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ControlAnswer {
    Sessions(Vec<SessionCount>),
    Seats(Vec<SeatReport>),
    /// The kicked connection's seat, now at its default.
    Kicked(SeatReport),
    /// Whether the seat changed, and what it is now.
    Released {
        released: bool,
        now: SeatReport,
    },
    Clock(ClockState),
    UnknownSession,
    UnknownSeat,
}

impl WorldRuntime {
    /// Answers one admin command. Bounded, synchronous, and nothing in it waits (step-12 I-11).
    pub(super) fn control(&mut self, command: ControlCommand) -> ControlAnswer {
        self.control_at(command, Instant::now())
    }

    /// [`control`](Self::control) at the wall instant `wall`: the entry the tests drive with an injected clock.
    pub(crate) fn control_at(&mut self, command: ControlCommand, wall: Instant) -> ControlAnswer {
        let at = self.clock.now_at(wall);
        match command {
            ControlCommand::Sessions => ControlAnswer::Sessions(
                self.subscribers
                    .iter()
                    .map(|subscriber| SessionCount {
                        session: subscriber.session,
                        observations_dropped: subscriber.dropped,
                    })
                    .collect(),
            ),
            ControlCommand::Seats => ControlAnswer::Seats(self.table.report(wall)),
            ControlCommand::Kick(session) => {
                let Some((subscription, seat)) = self.table.kick(session, at) else {
                    return ControlAnswer::UnknownSession;
                };
                // Both on this thread, in one command: no consult and no join can come between the
                // seat's rebinding and the connection's release, so the seat never has two drivers.
                self.release(subscription, ClosingReason::Kicked);
                self.seat_now(&seat, wall)
                    .map_or(ControlAnswer::UnknownSeat, ControlAnswer::Kicked)
            }
            ControlCommand::Release(seat) => {
                let Some(released) = self.table.release(&seat, at) else {
                    return ControlAnswer::UnknownSeat;
                };
                if let Some(subscription) = released.displaced {
                    self.release(subscription, ClosingReason::Kicked);
                }
                self.seat_now(&seat, wall)
                    .map_or(ControlAnswer::UnknownSeat, |now| ControlAnswer::Released {
                        released: released.released,
                        now,
                    })
            }
            ControlCommand::Clock => ControlAnswer::Clock(self.clock_state(wall)),
            ControlCommand::Change(change) => self.change_clock(change, wall),
        }
    }

    /// Applies a clock change: the scale first, then the pause state (`PROTOCOL.md` §11.2). A change
    /// that alters neither the scale nor the pause state records and announces nothing.
    fn change_clock(&mut self, change: ClockChange, wall: Instant) -> ControlAnswer {
        let before = self.clock_state(wall);
        if let Some(to) = change.time_scale
            && let Some(at) = self.rescale(to, wall)
        {
            self.journal_or_stop(PacingChange::Scaled { to }, at);
        }
        if let Some(paused) = change.paused
            && paused != self.clock.paused()
        {
            // Recorded at the reading the change was made at: the pause's own instant, which is also the
            // resume's, since a paused clock stands still (`ARC-69` item 6).
            let at = self.clock.now_at(wall);
            if paused {
                self.clock.pause_at(wall);
                self.journal_or_stop(PacingChange::Paused, at);
            } else {
                self.clock.resume_at(wall);
                self.journal_or_stop(PacingChange::Resumed, at);
            }
        }
        let after = self.clock_state(wall);
        if after.time_scale != before.time_scale || after.paused != before.paused {
            // `send_replace` neither fails without receivers nor waits for any: a session reads the
            // newest value when it next looks (step-12 SD-D8).
            self.announced.send_replace(after);
        }
        ControlAnswer::Clock(after)
    }

    /// Changes the scale at the instant the clock reads now, rescheduling every hosted consult in the
    /// same command (`ARC-69` item 2). A paused clock's reading is frozen, so the rule applies at the
    /// frozen instant and the scale takes effect from the resume.
    /// Returns the instant the change was made at, or `None` when the scale was already `to`.
    fn rescale(&mut self, to: NonZeroU32, wall: Instant) -> Option<WorldTime> {
        let from = self.clock.scale();
        if from == to {
            return None;
        }
        let at = self.clock.now_at(wall);
        self.clock.rescale_at(wall, to);
        self.table.rescale(at, from, to);
        Some(at)
    }

    /// Writes one pacing change to the host journal of a persisted world (`ARC-69` item 6). Not a fact and not
    /// a revision: the input journal, the facts and the snapshots are untouched.
    pub(super) fn journal(
        &mut self,
        change: PacingChange,
        at: WorldTime,
    ) -> Result<(), PersistError> {
        match &mut self.world {
            Hosted::Ephemeral(_) => Ok(()),
            Hosted::Persisted(world) => world.record_host(change, at),
        }
    }

    /// [`journal`](Self::journal) for a change the world has already made. A save that cannot take the record
    /// stops the world, like a failed commit (`Failure::Stopped`): the clock has moved, and the save must not
    /// go on without the record of it.
    pub(super) fn journal_or_stop(&mut self, change: PacingChange, at: WorldTime) {
        if let Err(error) = self.journal(change, at) {
            self.stopped = Some(error.to_string());
        }
    }

    fn seat_now(&self, seat: &EntityKey, wall: Instant) -> Option<SeatReport> {
        self.table.state_of(seat, wall)
    }

    /// The host clock as a `clock` frame states it.
    pub(super) fn clock_state(&self, wall: Instant) -> ClockState {
        ClockState {
            at: self.clock.now_at(wall),
            time_scale: self.clock.scale().get(),
            paused: self.clock.paused(),
        }
    }
}
