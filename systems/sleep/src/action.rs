//! The actions this pack provides: sleep for a while, and get up.

use mineworld_contracts::{Action, ActionTypeId, SystemId};
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

/// Get up: ends one's own sleep (`woke` with `reason: asked`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wake {}

impl Action for Wake {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("wake");
    const OWNER: SystemId = SleepSystem::ID;
}
