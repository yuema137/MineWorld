//! A job's shifts as a Process: started once, woken at every shift's start and end, never ended.

use mineworld_contracts::{PersonId, ProcessTypeId};
use mineworld_kernel::ProcessKind;
use serde::{Deserialize, Serialize};

use crate::system::EmploymentSystem;

/// The kind of process a job's shifts are, owned — as a type — by [`EmploymentSystem`], so no other
/// system can reschedule or end one (`INV-7`).
pub struct ShiftProcess;

impl ProcessKind for ShiftProcess {
    const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("shift");
    type Owner = EmploymentSystem;
}

/// This pack's state for one shift process: whose job it keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShiftState {
    employee: PersonId,
}

impl ShiftState {
    /// `employee`'s shifts.
    pub const fn new(employee: PersonId) -> Self {
        Self { employee }
    }

    /// Whose.
    pub const fn employee(&self) -> PersonId {
        self.employee
    }
}
