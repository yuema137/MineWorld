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
///
/// The host may pause it (`PROTOCOL.md` §11.3; step-12 SD-D6, S19 §4.2 option A). The clock is one
/// segment `(world_anchor, wall_anchor, scale, paused)`: running, it reads `world_anchor + ⌊elapsed
/// wall ms × scale / 1000⌋`; paused, it reads `world_anchor`. A pause folds the reading into the
/// anchor; a resume moves only the wall anchor. So it never decreases and never jumps — not by the
/// paused duration, not backwards (INV-TW-8). Every reading takes the wall instant as an argument, so
/// the arithmetic never reads a clock of its own; [`HostClock::now`] is the one place that does.
pub(super) struct HostClock {
    world_anchor: WorldTime,
    wall_anchor: Instant,
    scale: NonZeroU32,
    paused: bool,
}

impl HostClock {
    pub(super) fn new(epoch: WorldTime, scale: NonZeroU32, wall: Instant) -> Self {
        Self {
            world_anchor: epoch,
            wall_anchor: wall,
            scale,
            paused: false,
        }
    }

    /// The instant the world should have reached by now.
    pub(super) fn now(&self) -> WorldTime {
        self.now_at(Instant::now())
    }

    /// The instant the world should have reached at the wall instant `wall`.
    pub(super) fn now_at(&self, wall: Instant) -> WorldTime {
        if self.paused {
            return self.world_anchor;
        }
        let elapsed = wall.saturating_duration_since(self.wall_anchor).as_millis();
        let scaled = elapsed * u128::from(self.scale.get()) / 1_000;
        let scaled = i64::try_from(scaled).unwrap_or(i64::MAX);
        WorldTime::from_seconds(self.world_anchor.seconds().saturating_add(scaled))
    }

    /// Stops the clock where it stands at `wall`. Pausing a paused clock changes nothing.
    pub(super) fn pause_at(&mut self, wall: Instant) {
        if !self.paused {
            self.world_anchor = self.now_at(wall);
            self.paused = true;
        }
    }

    /// Starts the clock again from the instant it stopped at. Resuming a running clock changes
    /// nothing.
    pub(super) fn resume_at(&mut self, wall: Instant) {
        if self.paused {
            self.wall_anchor = wall;
            self.paused = false;
        }
    }

    pub(super) const fn paused(&self) -> bool {
        self.paused
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

    /// DA-7 (INV-TW-8): under 10 000 seeded random steps of pause, resume and wall advances of 0 to
    /// 5 000 ms, the clock never decreases, is constant while paused, and right after a resume reads
    /// exactly the instant it was paused at. Independently, it never runs ahead of the wall time it
    /// was running for.
    #[test]
    fn the_host_clock_is_monotonic_and_jump_free_across_pauses() {
        let start = Instant::now();
        let scale = NonZeroU32::new(60).expect("non-zero");
        let mut clock = HostClock::new(WorldTime::from_seconds(1_000), scale, start);
        let mut wall = start;
        let mut running_ms: u128 = 0;
        let mut previous = clock.now_at(wall);
        // A small linear congruential generator: seeded, reproducible, no dependency.
        let mut seed: u64 = 7;
        let mut next = |bound: u64| {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (seed >> 33) % bound
        };
        for _ in 0..10_000 {
            match next(3) {
                0 => {
                    let before = clock.now_at(wall);
                    clock.pause_at(wall);
                    assert_eq!(clock.now_at(wall), before, "pausing does not move the clock");
                }
                1 => {
                    let frozen = clock.now_at(wall);
                    clock.resume_at(wall);
                    assert_eq!(clock.now_at(wall), frozen, "a resume continues where it stopped");
                }
                _ => {
                    let step = Duration::from_millis(next(5_001));
                    let before = clock.now_at(wall);
                    wall += step;
                    if clock.paused() {
                        assert_eq!(clock.now_at(wall), before, "a paused clock stands still");
                    } else {
                        running_ms += step.as_millis();
                    }
                }
            }
            let now = clock.now_at(wall);
            assert!(now >= previous, "the clock went back: {previous:?} → {now:?}");
            let ceiling = 1_000 + i64::try_from(running_ms * 60 / 1_000).expect("small");
            assert!(
                now.seconds() <= ceiling,
                "the clock ran ahead of its running wall time: {now:?} > {ceiling}"
            );
            previous = now;
        }
    }

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
