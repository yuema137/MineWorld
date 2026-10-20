//! A sleep as a Process: started when the alarm is set, ending at the alarm or earlier.

use mineworld_contracts::{PersonId, ProcessTypeId};
use mineworld_kernel::ProcessKind;
use serde::{Deserialize, Serialize};

use crate::system::SleepSystem;

/// The kind of process a sleep is, owned — as a type — by [`SleepSystem`], so no other system can
/// reschedule or end one (`INV-7`).
pub struct SleepProcess;

impl ProcessKind for SleepProcess {
    const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("sleep");
    type Owner = SleepSystem;
}

/// This pack's state for one sleep process: who sleeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SleepState {
    sleeper: PersonId,
}

impl SleepState {
    /// `sleeper`'s sleep.
    pub const fn new(sleeper: PersonId) -> Self {
        Self { sleeper }
    }

    /// Who sleeps.
    pub const fn sleeper(&self) -> PersonId {
        self.sleeper
    }
}
