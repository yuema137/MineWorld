//! How a hosted world is configured: its clock's starting point, its stream's cadence and its bounds.
//!
//! Moved here unchanged from `host.rs` by TW-c (`pr-tw-c-host-clock.md` §5.9), so that `host.rs` stays
//! under its size review point. The request budget (S27-a, `ARC-97`) is the only addition since.

use std::num::NonZeroU32;
use std::time::Duration;

use mineworld_contracts::WorldTime;

/// How a world is hosted: the clock's starting point, the stream's cadence, and two bounds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostConfig {
    /// The instant the world's clock starts from.
    pub epoch: WorldTime,
    /// How often each connected client is sent a fresh observation.
    ///
    /// 10 Hz by default, as the spike ran. `NETWORKING.md` §4 is explicit that this is a life
    /// simulation rather than a competitive shooter, and that the rate of replication is not the
    /// rate of the simulation nor of rendering.
    pub observation_interval: Duration,
    /// How many observations may be queued for one client before the oldest are lost.
    ///
    /// Small on purpose. A slow client should receive the *current* world late, not a backlog of
    /// stale ones, and the world must never wait for it.
    pub observation_backlog: usize,
    /// How many recorded events perception may look back over.
    ///
    /// A window, not a log: the durable event log is S5's.
    pub recent_events: usize,
    /// How long a seat whose connection dropped is held for its resume, in wall time
    /// (`PROTOCOL.md` §4.2). Zero holds none.
    pub hold: Duration,
    /// Which time scale the world starts at, as its host states it (`ARC-69` item 8). The save's journal is
    /// the third source, read by the runtime itself.
    pub time_scale: TimeScaleChoice,
    /// Whether this world is a single-player world (`mineworld server --solo`, `ARC-69` item 5): the only
    /// kind whose time may be scaled above 24×, at start or live (`PROTOCOL.md` §11.2). The one source of
    /// this fact: [`WorldHost::spawn`](crate::WorldHost::spawn) refuses a start-time `time_scale.flag` above
    /// 24× without it, the admin surface refuses a live one, and `app::serve` refuses to serve a solo world on
    /// a listener that is not bound to a loopback address. The server selects no other code path for it.
    pub solo: bool,
    /// Learned facts that may wait for a connection's next frame; past it they drop (§5.2).
    pub event_backlog: usize,
    /// Facts that may wait for a connection's `perceived` stream; past it, `lagged` (§5.8).
    pub perceived_backlog: usize,
    /// Every Nth frame to a connection is a whole observation, the others deltas (§5.3, DEP-15).
    pub keyframe_every: NonZeroU32,
    /// How many `submit`s each seat may make, refused `rate_limited` beyond it (`PROTOCOL.md` §8,
    /// `ARC-97`); `None` sets no budget. It bounds every connection alike, and no in-server
    /// controller.
    pub request_budget: Option<RequestBudget>,
}

/// The time scale a hosted world starts at, as its host states it: a flag the operator gave, the world's own
/// default (`world.yaml` `hosting.time_scale`), or neither. The save's journal comes before the default, and
/// 1 after it (`ARC-69` item 8; `pacing.rs`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TimeScaleChoice {
    /// `--time-scale`, if given.
    pub flag: Option<NonZeroU32>,
    /// `world.yaml`'s `hosting.time_scale`, if the world declares one.
    pub pack: Option<NonZeroU32>,
}

/// A seat's request budget: `per_second` sustained, up to `burst` at once (a token bucket kept per
/// seat by the seat table, so reconnecting does not refill it; `docs/DECISIONS.md` `ARC-97`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestBudget {
    /// Requests per wall second, sustained.
    pub per_second: NonZeroU32,
    /// Requests at once, from a full bucket.
    pub burst: NonZeroU32,
}

impl RequestBudget {
    /// `R = 10`, `B = 20`: S27-a C0's measured rule, `R = max(10, 2 × p99)` over the 2D walker's
    /// recorded trace (p99 3 a second), `B = 2R` (`ARC-97`).
    pub const DEFAULT: Self = Self {
        per_second: nonzero(10),
        burst: nonzero(20),
    };
}

/// A non-zero literal, in a constant.
const fn nonzero(n: u32) -> NonZeroU32 {
    match NonZeroU32::new(n) {
        Some(n) => n,
        None => NonZeroU32::MIN,
    }
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            epoch: WorldTime::EPOCH,
            observation_interval: Duration::from_millis(100),
            observation_backlog: 8,
            recent_events: 64,
            hold: Duration::from_secs(30),
            time_scale: TimeScaleChoice::default(),
            solo: false,
            event_backlog: 256,
            perceived_backlog: 4_096,
            keyframe_every: NonZeroU32::new(50).unwrap_or(NonZeroU32::MIN),
            request_budget: Some(RequestBudget::DEFAULT),
        }
    }
}
