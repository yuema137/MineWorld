//! A person's day as a Process: started once, woken at every boundary, never ended.

use mineworld_contracts::{PersonId, ProcessTypeId};
use mineworld_kernel::ProcessKind;
use serde::{Deserialize, Serialize};

use crate::system::ScheduleSystem;

/// The kind of process a routine is, owned — as a type — by [`ScheduleSystem`], so no other system can
/// reschedule or end one (`INV-7`).
pub struct RoutineProcess;

impl ProcessKind for RoutineProcess {
    const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("routine");
    type Owner = ScheduleSystem;
}

/// This pack's state for one routine process: whose day it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutineState {
    person: PersonId,
}

impl RoutineState {
    /// `person`'s day.
    pub const fn new(person: PersonId) -> Self {
        Self { person }
    }

    /// Whose.
    pub const fn person(&self) -> PersonId {
        self.person
    }
}
