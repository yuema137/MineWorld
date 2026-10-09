//! The world thread's half of the admin surface: the seats report, kick, release, and the host
//! clock (`PROTOCOL.md` §11, `docs/DECISIONS.md` `ARC-44`).
//!
//! Host state only. Nothing here dispatches, advances, journals or states a fact, so no admin command
//! moves the world's revision (`ARC-40`, step-12 I-4). The permission check and its delay are the
//! transport's (`crate::admin`); by the time a command arrives here it is the host's.

use std::time::Instant;

use mineworld_contracts::EntityKey;

use super::WorldRuntime;
use crate::protocol::{ClockState, ClosingReason, SessionId};
use crate::seats::SeatReport;

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
    /// Pause (`true`) or resume (`false`) the host clock.
    Pause(bool),
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
        let wall = Instant::now();
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
            ControlCommand::Pause(paused) => {
                if paused != self.clock.paused() {
                    if paused {
                        self.clock.pause_at(wall);
                    } else {
                        self.clock.resume_at(wall);
                    }
                    // `send_replace` neither fails without receivers nor waits for any: a session
                    // reads the newest value when it next looks (step-12 SD-D8).
                    self.announced.send_replace(self.clock_state(wall));
                }
                ControlAnswer::Clock(self.clock_state(wall))
            }
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
