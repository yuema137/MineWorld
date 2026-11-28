//! The advance's loop: every due instant in order, in slices (pr-nightlife N2-I5, N2-I7, N2-I12, N2-I13).
//!
//! ```text
//! a slice    up to SLICE_INSTANTS completed instants or SLICE_TIME of wall time, whichever first
//!   t        = min(the kernel's next instant, the earliest hosted consult, the adapter's horizon),
//!              never before the world's instant, never an instant already completed
//!   step     the ordinary tick's own step to t (`WorldRuntime::step_to`): every seat due by t consulted at
//!              its instant through the ordinary path, then the world advanced to t; each input committed
//!              to the save before anything is published
//!   check    after the instant is complete, the adapter: the first completed instant at which the
//!              occurrence no longer holds ends the advance `woke`
//! between    one command: the slice re-posts itself on the world's bounded channel, so a sweep, a status,
//!              a cancel, a departure or a stop is served between two slices; each slice ends with a sweep
//!              and one `clock` frame
//! ```
//!
//! No rescale and no skipping: the scale, every hosted cadence and every speech window are those of the
//! ordinary host, which is what makes the night equal to the ordinary host ticking at every due instant
//! (N2-I6). Delivery is the ordinary bounded delivery: a slice ends early when the sleeper's stream reaches
//! half its bound, and the next waits — never blocking — for a sweep to drain it (D-N2-6).

use std::fmt;
use std::time::{Duration, Instant};

use mineworld_contracts::WorldTime;
use mineworld_persistence::AdvanceOutcome;
use tokio::sync::mpsc;

use super::super::WorldRuntime;
use crate::host::Command;

/// The most instants one slice completes (N2-I7): an implementation constant, not gameplay time.
const SLICE_INSTANTS: u32 = 128;
/// The most wall time one slice spends before it lets the thread serve other commands (N2-I7).
const SLICE_TIME: Duration = Duration::from_millis(10);

/// What an advance did, for the operator's line when it ends and for the speed record (N2-C3 item 5).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Progress {
    /// Completed instants.
    pub(crate) instants: u64,
    /// Slices run.
    pub(crate) slices: u64,
    /// The deepest the sleeper's stream was at the end of an instant: waiting perceived facts and queued
    /// frames (N2-I13's high-water mark).
    pub(crate) high_water: usize,
    /// Slices ended early to let a sweep drain the sleeper's stream.
    pub(crate) yielded: u64,
}

impl fmt::Display for Progress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} instants in {} slices, {} yielded to delivery, sleeper stream high-water {}",
            self.instants, self.slices, self.yielded, self.high_water
        )
    }
}

/// How a slice ended.
enum Ended {
    /// The advance is over.
    With(AdvanceOutcome),
    /// The slice's budget is spent: the next one is posted.
    Budget,
    /// The sleeper's stream is half full: the next slice waits for a sweep.
    Delivery,
}

impl WorldRuntime {
    /// Runs one slice of the running advance, if one runs.
    pub(in crate::runtime) fn advance_slice(&mut self) {
        let Some(running) = &mut self.advancing else {
            return;
        };
        running.awaiting_sweep = false;
        running.progress.slices += 1;
        let ended = self.run_slice();
        match ended {
            Ended::With(outcome) => {
                self.end_advance(outcome, None);
                if self.stopped.is_none() {
                    self.sweep();
                }
                return;
            }
            Ended::Budget => {}
            Ended::Delivery => {
                if let Some(running) = &mut self.advancing {
                    running.progress.yielded += 1;
                    running.awaiting_sweep = true;
                }
            }
        }
        self.sweep();
        let wall = Instant::now();
        self.announced.send_replace(self.clock_state(wall));
        let waiting = self
            .advancing
            .as_ref()
            .is_some_and(|running| running.awaiting_sweep);
        if waiting {
            // The sweep just run may already have drained the stream; if not, a later one will.
            self.resume_after_sweep();
        } else {
            self.post_slice();
        }
    }

    /// The instants of one slice. Stops at the budget, at a half-full stream, or at the advance's end.
    fn run_slice(&mut self) -> Ended {
        let started = Instant::now();
        let mut completed = 0;
        // Whatever happened between two slices — the sleeper's own `wake` among it — is judged first, at the
        // boundary already reached (N2-I12).
        if !self.occurrence_holds() {
            return Ended::With(AdvanceOutcome::Woke);
        }
        loop {
            if self.stopped.is_some() {
                return Ended::With(AdvanceOutcome::Failed);
            }
            let Some(next) = self.next_instant() else {
                return Ended::With(self.unbounded_end());
            };
            self.step_to(next);
            if self.stopped.is_some() {
                return Ended::With(AdvanceOutcome::Failed);
            }
            let depth = self.sleeper_depth();
            let Some(running) = &mut self.advancing else {
                // The sleeper's connection went during the instant (lagged): already ended.
                return Ended::Budget;
            };
            running.cursor = Some(next);
            running.progress.instants += 1;
            running.progress.high_water = running.progress.high_water.max(depth.0);
            self.clock.hold_at(Instant::now(), next);
            if !self.occurrence_holds() {
                return Ended::With(AdvanceOutcome::Woke);
            }
            completed += 1;
            if depth.1 {
                return Ended::Delivery;
            }
            if completed >= SLICE_INSTANTS || started.elapsed() >= SLICE_TIME {
                return Ended::Budget;
            }
        }
    }

    /// The next instant to complete (N2-I5): the earliest of the kernel's queue, the hosted consults and the
    /// adapter's horizon; never before the world's instant and never one already completed. `None` when the
    /// occurrence has ended or nothing at all is due.
    fn next_instant(&self) -> Option<WorldTime> {
        let running = self.advancing.as_ref()?;
        let world = self.world.world();
        let horizon = running
            .stop
            .horizon(&world.read(), running.observer, running.occurrence);
        let due = [world.next_instant(), self.table.earliest_due(), horizon]
            .into_iter()
            .flatten()
            .min()?;
        let mut next = due.max(world.now());
        if let Some(cursor) = running.cursor
            && next <= cursor
        {
            // A seat that cannot be consulted stays due; the night still moves one second on, as the ordinary
            // host's next tick would, and never spins on one instant.
            next = WorldTime::from_seconds(cursor.seconds().saturating_add(1));
        }
        Some(next)
    }

    /// Whether the sleeper is still in the occurrence identified at the start.
    fn occurrence_holds(&self) -> bool {
        self.advancing.as_ref().is_some_and(|running| {
            running.stop.holds(
                &self.world.world().read(),
                running.observer,
                running.occurrence,
            )
        })
    }

    /// Nothing is due and the adapter names no horizon: `woke` if the occurrence has ended, else `stopped` —
    /// nothing could ever end it.
    fn unbounded_end(&self) -> AdvanceOutcome {
        if self.occurrence_holds() {
            AdvanceOutcome::Stopped
        } else {
            AdvanceOutcome::Woke
        }
    }

    /// The sleeper's stream depth, and whether it has reached half its bound (N2-I13, D-N2-6). Reads only.
    fn sleeper_depth(&self) -> (usize, bool) {
        let Some(running) = &self.advancing else {
            return (0, false);
        };
        self.stream_depth(
            running.subscription(),
            self.config.perceived_backlog,
            self.config.observation_backlog,
        )
    }

    /// After a sweep: an advance that waited for one continues if the sleeper's stream has drained below half.
    pub(in crate::runtime) fn resume_after_sweep(&mut self) {
        let waiting = self
            .advancing
            .as_ref()
            .is_some_and(|running| running.awaiting_sweep);
        if waiting && !self.sleeper_depth().1 {
            if let Some(running) = &mut self.advancing {
                running.awaiting_sweep = false;
            }
            self.post_slice();
        }
    }

    /// Posts the next slice to this thread's own channel. A full channel leaves it to the next sweep.
    pub(in crate::runtime) fn post_slice(&mut self) {
        let posted = self
            .continuation
            .as_ref()
            .and_then(mpsc::WeakSender::upgrade)
            .is_some_and(|commands| commands.try_send(Command::AdvanceSlice).is_ok());
        if !posted && let Some(running) = &mut self.advancing {
            running.awaiting_sweep = true;
        }
    }

    /// This runtime, posting an advance's slices to `commands` (`WorldHost::spawn`).
    pub(crate) fn continuing_through(mut self, commands: mpsc::WeakSender<Command>) -> Self {
        self.continuation = Some(commands);
        self
    }
}
