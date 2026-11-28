//! The single-player advance: its admission, its lifecycle and its durable identity (`PROTOCOL.md`
//! §11.4, `docs/DECISIONS.md` `ARC-374`; pr-nightlife N2-I3, N2-I8 … N2-I12).
//!
//! A sleeping player may ask the host to run the night faster than wall time. That is **host state**: an
//! operation the host runs, never a fact, an input or a revision. Its lifecycle is written to the save's
//! host operations (`mineworld_persistence::operation`) before the first instant and after the last, so a
//! restart finds an interrupted one and resumes nothing.
//!
//! ```text
//! admit     solo world · known operation? · no other advance · the adapter exists in this world ·
//!           the seat is connected · no other seat is connected or held · the clock runs ·
//!           the seat's Person is in the adapter's state             else a typed refusal, no record
//! start     advance-started { operation, seat, adapter, prior_scale }
//! end       advance-ended { operation, outcome, reason }            woke · cancelled · disconnected ·
//!                                                                   stopped · interrupted · failed
//! ```
//!
//! The host knows the adapter's trait (`mineworld_sdk::advance`), never what sleep is. While an advance
//! runs, every command that could seat, drive or re-pace another seat is refused `advance_in_progress`;
//! the sleeper's departure ends it `disconnected`; a stop ends it `stopped`. The night itself runs in
//! slices, each one command on the world thread (`advance/slice.rs`).

use std::time::Instant;

use mineworld_contracts::{EntityId, EntityKey};
use mineworld_persistence::{
    AdvanceOutcome, EndReason, OperationEvent, OperationId, OperationRecord, PersistError,
    WorldRevision,
};
use mineworld_sdk::advance::{AdvanceStop, StopIdentity, stop_named};

use super::WorldRuntime;
use crate::host::{Hosted, SubscriptionId};
use crate::protocol::{Refusal, RefusalCode};

/// `POST /admin/advance`: which operation, whose seat, and until which adapter says stop. No instant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AdvanceRequest {
    /// The client's identity for this advance.
    pub(crate) operation: OperationId,
    /// The seat whose Person sleeps.
    pub(crate) seat: EntityKey,
    /// The stop adapter, by name.
    pub(crate) until: String,
}

/// Why an advance request was refused. Nothing was recorded and nothing moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AdvanceRefusal {
    /// The world was not started `--solo`; a shared or both-client host never advances.
    NotSolo,
    /// The named seat is not driven by exactly one connection.
    SeatMismatch,
    /// Another seat is connected, or held for a connection that dropped.
    OtherControllerConnected,
    /// The seat's Person is not in the adapter's state.
    NotAsleep,
    /// The host clock is paused.
    Paused,
    /// Another advance runs.
    Busy,
    /// No adapter of that name in this build, or its system is not enabled in this world.
    UnknownAdapter,
    /// The operation is known, with a different seat or adapter.
    OperationConflict,
    /// No operation has that identity.
    UnknownOperation,
    /// The save could not take the record: the world stops.
    WorldStopped,
}

impl AdvanceRefusal {
    /// The wire code (`PROTOCOL.md` §11.4).
    pub(crate) const fn code(self) -> &'static str {
        match self {
            Self::NotSolo => "not_solo",
            Self::SeatMismatch => "seat_mismatch",
            Self::OtherControllerConnected => "other_controller_connected",
            Self::NotAsleep => "not_asleep",
            Self::Paused => "paused",
            Self::Busy => "busy",
            Self::UnknownAdapter => "unknown_adapter",
            Self::OperationConflict => "operation_conflict",
            Self::UnknownOperation => "unknown_operation",
            Self::WorldStopped => "world_stopped",
        }
    }

    /// What the operator is told.
    pub(crate) const fn detail(self) -> &'static str {
        match self {
            Self::NotSolo => "only a world started --solo may advance; this one may not",
            Self::SeatMismatch => "the seat is not driven by exactly one connection",
            Self::OtherControllerConnected => {
                "another seat is connected, or held for a connection that dropped"
            }
            Self::NotAsleep => "the seat's Person is not in the state the adapter stops on",
            Self::Paused => "the host clock is paused",
            Self::Busy => "another advance is running",
            Self::UnknownAdapter => "this world has no advance stop adapter of that name",
            Self::OperationConflict => {
                "that operation is already known with a different seat or adapter"
            }
            Self::UnknownOperation => "no advance has that operation",
            Self::WorldStopped => "the save could not take the record; the world has stopped",
        }
    }
}

/// An operation as `GET /admin/advance/{operation}` reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AdvanceStatus {
    pub(crate) operation: OperationId,
    pub(crate) seat: EntityKey,
    pub(crate) until: String,
    /// `None` while it runs.
    pub(crate) ended: Option<(AdvanceOutcome, Option<EndReason>)>,
}

/// The advance that runs now: what it was admitted with, held on the world thread only.
pub(super) struct Running {
    operation: OperationId,
    seat: EntityKey,
    /// The sleeper.
    observer: EntityId,
    /// The sleeper's connection: its departure ends the advance.
    subscription: SubscriptionId,
    stop: &'static dyn AdvanceStop,
    occurrence: StopIdentity,
    /// The last completed instant, once one is.
    cursor: Option<mineworld_contracts::WorldTime>,
    /// Waiting for a sweep to drain the sleeper's stream before the next slice (N2-I13, D-N2-6).
    awaiting_sweep: bool,
    /// What it did, for the operator's line when it ends.
    progress: slice::Progress,
}

impl Running {
    pub(super) const fn subscription(&self) -> SubscriptionId {
        self.subscription
    }

    pub(super) const fn seat(&self) -> &EntityKey {
        &self.seat
    }
}

/// The refusal a connection's `join` or `submit` meets while another seat's advance runs.
pub(super) fn in_progress() -> Refusal {
    Refusal::new(RefusalCode::AdvanceInProgress).detail(
        "the host is running a single-player advance; join or submit again once it has ended",
    )
}

impl WorldRuntime {
    /// Admits, starts and — in this build — at once ends an advance; or returns the recorded status of a
    /// known operation with the same payload (N2-I9). A refusal records nothing.
    pub(super) fn request_advance(
        &mut self,
        request: AdvanceRequest,
    ) -> Result<AdvanceStatus, AdvanceRefusal> {
        if let Some(known) = self.advance_status(request.operation) {
            return if known.seat == request.seat && known.until == request.until {
                Ok(known)
            } else {
                Err(AdvanceRefusal::OperationConflict)
            };
        }
        // A hold whose wall time has run out no longer counts against N2-I3, though no tick has reaped it.
        self.table.expire(Instant::now(), self.clock.now());
        let running = self.admit(&request)?;
        let prior_scale = self.clock.scale();
        let started = OperationEvent::AdvanceStarted {
            operation: request.operation,
            seat: request.seat,
            adapter: request.until,
            prior_scale,
        };
        if let Err(error) = self.record_operation(started) {
            self.stopped = Some(error.to_string());
            return Err(AdvanceRefusal::WorldStopped);
        }
        self.advancing = Some(running);
        // The clock is held at its reading: from here the advance alone moves it (N2-I12).
        self.clock.hold_at(Instant::now(), self.clock.now());
        self.post_slice();
        self.advance_status(request.operation)
            .ok_or(AdvanceRefusal::WorldStopped)
    }

    /// N2-I3's conditions, in order; the first that fails is the answer.
    fn admit(&self, request: &AdvanceRequest) -> Result<Running, AdvanceRefusal> {
        if !self.config.solo {
            return Err(AdvanceRefusal::NotSolo);
        }
        if self.advancing.is_some() {
            return Err(AdvanceRefusal::Busy);
        }
        let world = self.world.world();
        let stop = stop_named(&request.until)
            .filter(|stop| world.systems().is_enabled(&stop.system()))
            .ok_or(AdvanceRefusal::UnknownAdapter)?;
        let subscription = self
            .table
            .connection_of(&request.seat)
            .ok_or(AdvanceRefusal::SeatMismatch)?;
        if self.table.another_driven(&request.seat) {
            return Err(AdvanceRefusal::OtherControllerConnected);
        }
        if self.clock.paused() {
            return Err(AdvanceRefusal::Paused);
        }
        let observer = self
            .observer_of(&request.seat)
            .map_err(|_| AdvanceRefusal::SeatMismatch)?;
        let occurrence = stop
            .identify(&world.read(), observer)
            .ok_or(AdvanceRefusal::NotAsleep)?;
        Ok(Running {
            operation: request.operation,
            seat: request.seat.clone(),
            observer,
            subscription,
            stop,
            occurrence,
            cursor: None,
            awaiting_sweep: false,
            progress: slice::Progress::default(),
        })
    }

    /// Ends the running advance, if any, with `outcome`, at the world's instant. The clock runs on from the
    /// instant reached, at the scale it had, with no wall time folded in (N2-I10), and is announced.
    pub(super) fn end_advance(&mut self, outcome: AdvanceOutcome, reason: Option<EndReason>) {
        let Some(running) = self.advancing.take() else {
            return;
        };
        let wall = Instant::now();
        self.clock.hold_at(wall, self.world.world().now());
        self.clock.release_at(wall);
        self.announced.send_replace(self.clock_state(wall));
        println!(
            "[world] advance {} ended {outcome:?}: {}",
            running.operation, running.progress
        );
        let ended = OperationEvent::AdvanceEnded {
            operation: running.operation,
            outcome,
            reason,
        };
        if let Err(error) = self.record_operation(ended) {
            self.stopped = Some(error.to_string());
        }
    }

    /// `POST /admin/advance/{operation}/cancel`: the running advance stops and the sleeper stays as they
    /// are — the server submits nothing (N2-I10). An ended one answers its status unchanged.
    pub(super) fn cancel_advance(
        &mut self,
        operation: OperationId,
    ) -> Result<AdvanceStatus, AdvanceRefusal> {
        if self
            .advancing
            .as_ref()
            .is_some_and(|running| running.operation == operation)
        {
            self.end_advance(AdvanceOutcome::Cancelled, None);
        }
        self.advance_status(operation)
            .ok_or(AdvanceRefusal::UnknownOperation)
    }

    /// An operation's status from its records, or `None` for an unknown one.
    pub(super) fn advance_status(&self, operation: OperationId) -> Option<AdvanceStatus> {
        let mut status: Option<AdvanceStatus> = None;
        for record in &self.operations {
            match &record.event {
                OperationEvent::AdvanceStarted {
                    operation: started,
                    seat,
                    adapter,
                    ..
                } if *started == operation => {
                    status = Some(AdvanceStatus {
                        operation,
                        seat: seat.clone(),
                        until: adapter.clone(),
                        ended: None,
                    });
                }
                OperationEvent::AdvanceEnded {
                    operation: ended,
                    outcome,
                    reason,
                } if *ended == operation => {
                    if let Some(status) = &mut status {
                        status.ended = Some((*outcome, *reason));
                    }
                }
                _ => {}
            }
        }
        status
    }

    /// A connection is gone — departed, released, kicked or lagged. If it was the sleeper's, the advance
    /// ends `disconnected`: the night is never finished in the background (N2-I10).
    pub(super) fn connection_lost(&mut self, subscription: SubscriptionId, lagged: bool) {
        if self
            .advancing
            .as_ref()
            .is_some_and(|running| running.subscription() == subscription)
        {
            let reason = lagged.then_some(EndReason::Lagged);
            self.end_advance(AdvanceOutcome::Disconnected, reason);
        }
    }

    /// Whether a connection's `join`, or its `submit`, must be refused because another seat's advance
    /// runs: anything but the sleeper's own connection submitting.
    pub(super) fn refused_by_advance(&self, subscription: Option<SubscriptionId>) -> bool {
        self.advancing
            .as_ref()
            .is_some_and(|running| subscription != Some(running.subscription()))
    }

    /// At start: an advance a previous process started and never ended is recorded `interrupted` at the
    /// world's durable instant, and nothing resumes (N2-I8). A record that cannot be written stops the start.
    pub(super) fn close_interrupted(&mut self) -> Result<(), PersistError> {
        let Some(operation) = mineworld_persistence::unfinished(&self.operations) else {
            return Ok(());
        };
        self.record_operation(OperationEvent::AdvanceEnded {
            operation,
            outcome: AdvanceOutcome::Interrupted,
            reason: None,
        })
    }

    /// Writes one operation record — to the save first, for a persisted world — and keeps it. The caller
    /// stops the world when the save cannot take it (N2-I8: a persistence failure fail-stops).
    fn record_operation(&mut self, event: OperationEvent) -> Result<(), PersistError> {
        let at = self.world.world().now();
        let record = match &mut self.world {
            Hosted::Ephemeral(_) => OperationRecord {
                at,
                revision: WorldRevision::from_raw(0),
                event,
            },
            Hosted::Persisted(world) => world.record_operation(event, at)?,
        };
        self.operations.push(record);
        Ok(())
    }
}

mod slice;

#[cfg(test)]
mod parity;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod town;
