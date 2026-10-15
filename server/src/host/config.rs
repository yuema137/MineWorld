//! How a hosted world is configured: its clock's starting point, its stream's cadence and its bounds.
//!
//! Moved here unchanged from `host.rs` by TW-c (`pr-tw-c-host-clock.md` §5.9), so that `host.rs` stays
//! under its size review point. Nothing in this file is new.

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
    /// How many world seconds pass per wall second.
    pub time_scale: NonZeroU32,
    /// Learned facts that may wait for a connection's next frame; past it they drop (§5.2).
    pub event_backlog: usize,
    /// Facts that may wait for a connection's `perceived` stream; past it, `lagged` (§5.8).
    pub perceived_backlog: usize,
    /// Every Nth frame to a connection is a whole observation, the others deltas (§5.3, DEP-15).
    pub keyframe_every: NonZeroU32,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            epoch: WorldTime::EPOCH,
            observation_interval: Duration::from_millis(100),
            observation_backlog: 8,
            recent_events: 64,
            hold: Duration::from_secs(30),
            time_scale: NonZeroU32::MIN,
            event_backlog: 256,
            perceived_backlog: 4_096,
            keyframe_every: NonZeroU32::new(50).unwrap_or(NonZeroU32::MIN),
        }
    }
}
