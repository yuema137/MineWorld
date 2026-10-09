//! The world thread's small instruments: its pacing, its request allocator, and the one wrapper that
//! dispatches to whichever kind of world it hosts.
//!
//! None of these decides anything about the world. They answer "how far should the world be
//! advanced", "which identity does the next request get", and "did that input complete" — the
//! bookkeeping a hosted world needs that the kernel deliberately does not own.

use std::num::NonZeroU32;
use std::time::{Duration, Instant};

use mineworld_contracts::{ActionId, ActionIntent, WorldTime};
use mineworld_kernel::{Advanced, Dispatched, KernelError, World};
use mineworld_persistence::{PersistError, WorldRevision};

use crate::host::Hosted;

/// The host's pacing: which instant a hosted world should have reached by now.
///
/// Wall time in whole simulated seconds from an epoch. Not the world's clock — that is the kernel's,
/// moved only by advancing and dispatching (S4) — and not a scheduler: it answers "how far should the
/// world be advanced", which is a deployment decision rather than a simulation one. A headless run
/// that wants a hundred days in a second advances the kernel directly and has no use for this.
///
/// `scale` world seconds pass per wall second (`--time-scale`): the elapsed wall time is scaled in
/// milliseconds before it is cut to whole seconds, so a scaled clock moves smoothly rather than in
/// jumps of `scale` seconds.
pub(super) struct HostClock {
    epoch: WorldTime,
    started: Instant,
    scale: NonZeroU32,
}

impl HostClock {
    pub(super) fn new(epoch: WorldTime, scale: NonZeroU32) -> Self {
        Self {
            epoch,
            started: Instant::now(),
            scale,
        }
    }

    pub(super) fn now(&self) -> WorldTime {
        let scaled = self.started.elapsed().as_millis() * u128::from(self.scale.get()) / 1_000;
        let elapsed = i64::try_from(scaled).unwrap_or(i64::MAX);
        WorldTime::from_seconds(self.epoch.seconds().saturating_add(elapsed))
    }

    pub(super) const fn scale(&self) -> NonZeroU32 {
        self.scale
    }
}

/// How long the world thread's ticks took: counted, and the longest kept, for the operator's
/// statistics line on shutdown (`ARC-42`, step-12 CP-B4).
///
/// Kept as a histogram of 0.1 ms buckets up to one second (and one bucket beyond), so a server that
/// runs for months holds a fixed 80 KB rather than a sample per tick. A percentile is reported as its
/// bucket's upper edge: never under the true value, by at most 0.1 ms. CP-B4 bounds the p99 and
/// reports the maximum beside it (step-12 §16.4, operator ruling on D-SB12).
pub(super) struct TickTimes {
    ticks: u64,
    longest: Duration,
    buckets: Vec<u64>,
}

/// The histogram's resolution, in microseconds, and how many buckets it has before the overflow one.
const BUCKET_MICROS: u128 = 100;
const BUCKETS: usize = 10_000;

impl Default for TickTimes {
    fn default() -> Self {
        Self {
            ticks: 0,
            longest: Duration::ZERO,
            buckets: vec![0; BUCKETS + 1],
        }
    }
}

impl TickTimes {
    pub(super) fn record(&mut self, took: Duration) {
        self.ticks += 1;
        self.longest = self.longest.max(took);
        let bucket = usize::try_from(took.as_micros() / BUCKET_MICROS).unwrap_or(BUCKETS);
        self.buckets[bucket.min(BUCKETS)] += 1;
    }

    /// The `per_mille`th tick duration in milliseconds, as its bucket's upper edge; the overflow
    /// bucket answers with the longest tick.
    fn percentile(&self, per_mille: u64) -> f64 {
        if self.ticks == 0 {
            return 0.0;
        }
        let rank = (self.ticks * per_mille).div_ceil(1_000).max(1);
        let mut seen = 0;
        for (bucket, count) in self.buckets.iter().enumerate() {
            seen += count;
            if seen >= rank {
                if bucket == BUCKETS {
                    break;
                }
                return f64::from(u32::try_from(bucket + 1).unwrap_or(u32::MAX)) / 10.0;
            }
        }
        self.longest.as_secs_f64() * 1_000.0
    }

    pub(super) fn report(&self) -> String {
        format!(
            "[world] ticks {}, p50 {:.1} ms, p99 {:.1} ms, longest tick {} ms",
            self.ticks,
            self.percentile(500),
            self.percentile(990),
            self.longest.as_millis()
        )
    }
}

/// The server's request-identity allocator: monotonic, never reused, consulted by nothing else.
pub(super) struct ActionIds {
    next: u64,
}

impl ActionIds {
    /// An allocator whose first identity is `first` — one for a world with no requests behind it,
    /// past the journal's highest for a resumed one.
    pub(super) const fn starting_at(first: u64) -> Self {
        Self { next: first }
    }

    /// Allocates the next identity, or `None` when the space is spent — which no world reaches, and
    /// which must still not wrap, because a reused `ActionId` would make two requests
    /// indistinguishable in the event log's causal chain.
    pub(super) fn allocate(&mut self) -> Option<ActionId> {
        let id = ActionId::from_raw(self.next);
        self.next = self.next.checked_add(1)?;
        Some(id)
    }
}

/// Why an input to the hosted world did not complete.
pub(super) enum Failure {
    /// A system broke its own contract. Counted and reported; the world goes on. For a persisted
    /// world the fault is already journaled as the input's outcome.
    Fault(KernelError),
    /// The save could not be written: the world is ahead of it and must stop.
    Stopped(String),
}

impl Failure {
    fn from_persistence(error: PersistError) -> Self {
        match error {
            PersistError::Kernel(fault) => Self::Fault(fault),
            other => Self::Stopped(other.to_string()),
        }
    }
}

impl Hosted {
    pub(super) fn world(&self) -> &World {
        match self {
            Self::Ephemeral(world) => world,
            Self::Persisted(world) => world.world(),
        }
    }

    pub(super) fn revision(&self) -> Option<WorldRevision> {
        match self {
            Self::Ephemeral(_) => None,
            Self::Persisted(world) => Some(world.revision()),
        }
    }

    pub(super) fn dispatch(
        &mut self,
        intent: &ActionIntent,
        at: WorldTime,
    ) -> Result<Dispatched, Failure> {
        match self {
            Self::Ephemeral(world) => world.dispatch(intent, at).map_err(Failure::Fault),
            Self::Persisted(world) => world
                .dispatch(intent, at)
                .map_err(Failure::from_persistence),
        }
    }

    pub(super) fn advance_to(&mut self, at: WorldTime) -> Result<Advanced, Failure> {
        match self {
            Self::Ephemeral(world) => world.advance_to(at).map_err(Failure::Fault),
            Self::Persisted(world) => world.advance_to(at).map_err(Failure::from_persistence),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CP-B4's statistic: one stall in a hundred ticks moves the maximum and not the p99; two do.
    #[test]
    fn the_p99_ignores_one_stall_in_a_hundred_and_not_two() {
        let mut times = TickTimes::default();
        for _ in 0..99 {
            times.record(Duration::from_micros(1_050));
        }
        times.record(Duration::from_millis(160));
        assert_eq!(
            times.report(),
            "[world] ticks 100, p50 1.1 ms, p99 1.1 ms, longest tick 160 ms"
        );
        times.record(Duration::from_millis(170));
        assert!(
            times.report().contains("p99 160.1 ms"),
            "{}",
            times.report()
        );
        let mut beyond = TickTimes::default();
        beyond.record(Duration::from_secs(3));
        assert!(
            beyond.report().contains("p99 3000.0 ms"),
            "past the histogram, the longest: {}",
            beyond.report()
        );
    }
}
