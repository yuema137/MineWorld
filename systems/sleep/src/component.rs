//! The state this pack owns, and the record it discloses without storing.

use mineworld_contracts::{ProcessId, WorldTime};
use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::system::SleepSystem;

/// A person is asleep: since when, until which alarm, and the `sleep` Process that ends at it.
///
/// Written only while reducing this pack's own `alarm-set` (inserted) and `woke` (removed), so it is a
/// projection of the pack's log and replay rebuilds it. Present exactly between a `fell-asleep` and
/// the `woke` that follows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Asleep {
    since: WorldTime,
    alarm: WorldTime,
    process: ProcessId,
}

owned_component! {
    component = Asleep,
    owner = SleepSystem,
    component_type = "asleep",
    schema_version = 1,
}

impl Asleep {
    pub(crate) const fn new(since: WorldTime, alarm: WorldTime, process: ProcessId) -> Self {
        Self {
            since,
            alarm,
            process,
        }
    }

    /// When the sleep began.
    pub const fn since(&self) -> WorldTime {
        self.since
    }

    /// When the alarm rings.
    pub const fn alarm(&self) -> WorldTime {
        self.alarm
    }

    /// The `sleep` Process that ends at the alarm.
    pub const fn process(&self) -> ProcessId {
        self.process
    }
}

/// The bounds of `sleep`'s `for` in force for a person where they are: what a controller, an agent or
/// a client reads instead of hard-coding them. Declared so perception keeps the record, never stored
/// on any entity: computed at disclosure from the world's list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SleepTerms {
    /// The fewest minutes one may ask to sleep.
    pub shortest_minutes: u16,
    /// The most minutes one may ask to sleep.
    pub longest_minutes: u16,
}

owned_component! {
    component = SleepTerms,
    owner = SleepSystem,
    component_type = "sleep-terms",
    schema_version = 1,
}
