//! The actions this pack provides: sleep for a while, sleep until an instant, and get up.

use mineworld_contracts::{Action, ActionTypeId, SystemId, WorldTime};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::system::SleepSystem;

/// Lie down to sleep where one is, for `for` minutes: the alarm is now + `for`. `for` must lie within
/// the bounds the world's list gives the sleeper there (`shortest_minutes … longest_minutes`, 15 … 840
/// by default).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sleep {
    #[serde(rename = "for")]
    minutes: u16,
}

impl Action for Sleep {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("sleep");
    const OWNER: SystemId = SleepSystem::ID;
}

impl Sleep {
    /// Sleep for `minutes`.
    pub const fn new(minutes: u16) -> Self {
        Self { minutes }
    }

    /// For how many minutes: the payload's `for`.
    pub const fn minutes(&self) -> u16 {
        self.minutes
    }
}

/// Lie down to sleep where one is until the instant `at`: the alarm is `at` itself (N2-I2). The sleep
/// is `sleep`'s in every other respect — the same admission, place rule, Interaction-List rules and
/// parameters (resolved with [`Sleep::ACTION_TYPE`]), Process, alarm privacy and four ways to end — so
/// a world's `{ action: sleep, … }` rules govern it and no world writes a rule for it. `at − now` must
/// lie within the same bounds as `for`, in seconds. The server offers the exact `at` for the world's
/// morning; a client copies it and never computes one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SleepUntil {
    at: WorldTime,
}

impl Action for SleepUntil {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("sleep-until");
    const OWNER: SystemId = SleepSystem::ID;
}

impl SleepUntil {
    /// Sleep until `at`.
    pub const fn new(at: WorldTime) -> Self {
        Self { at }
    }

    /// Until when: the payload's `at`, which becomes the alarm.
    pub const fn at(&self) -> WorldTime {
        self.at
    }
}

/// Get up: ends one's own sleep (`woke` with `reason: asked`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wake {}

impl Action for Wake {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("wake");
    const OWNER: SystemId = SleepSystem::ID;
}
