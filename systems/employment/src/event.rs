//! The facts this pack records.
//!
//! `hired`, `shift-started` and `shift-ended` are this pack's own state changes, reduced by it alone.
//! `wage-due` is a statement for another pack to act on: this pack never reduces it, and `economy`
//! answers it by moving money, or by recording that it could not (`ARC-28`, `CORE_CONCEPTS.md`
//! §13.1). None has a public constructor: only this pack states them.

use mineworld_contracts::{
    Event, EventSchemaVersion, EventTypeId, OrganizationId, PersonId, SystemId,
};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::component::Job;
use crate::system::EmploymentSystem;

/// `employee` holds `job`: a genesis fact from the person's `job:` section. Biographical — once per
/// job, a life event (step-10 QS-49).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hired {
    employee: PersonId,
    job: Job,
}

impl Event for Hired {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("hired");
    const OWNER: SystemId = EmploymentSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl Hired {
    pub(crate) const fn new(employee: PersonId, job: Job) -> Self {
        Self { employee, job }
    }

    /// Who was hired.
    pub const fn employee(&self) -> PersonId {
        self.employee
    }

    /// The job.
    pub const fn job(&self) -> &Job {
        &self.job
    }
}

/// `employee`'s shift began; `present` says whether they were at the workplace when it did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShiftStarted {
    employee: PersonId,
    present: bool,
}

impl Event for ShiftStarted {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("shift-started");
    const OWNER: SystemId = EmploymentSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl ShiftStarted {
    pub(crate) const fn new(employee: PersonId, present: bool) -> Self {
        Self { employee, present }
    }

    /// Whose shift.
    pub const fn employee(&self) -> PersonId {
        self.employee
    }

    /// Whether they were at the workplace when it began.
    pub const fn present(&self) -> bool {
        self.present
    }
}

/// `employee`'s shift ended, `worked` seconds of it at the workplace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShiftEnded {
    employee: PersonId,
    worked: u64,
}

impl Event for ShiftEnded {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("shift-ended");
    const OWNER: SystemId = EmploymentSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl ShiftEnded {
    pub(crate) const fn new(employee: PersonId, worked: u64) -> Self {
        Self { employee, worked }
    }

    /// Whose shift.
    pub const fn employee(&self) -> PersonId {
        self.employee
    }

    /// Seconds spent at the workplace during it.
    pub const fn worked(&self) -> u64 {
        self.worked
    }
}

/// `employer` owes `employee` `amount` minor units for a shift worked.
///
/// The fact `economy` subscribes to, decoding it through this published type with no system
/// dependency on this pack (`ARC-28`). Stating it moves no money: economy decides, and records either
/// the payment, caused by this fact, or `wage-unpaid`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WageDue {
    employee: PersonId,
    employer: OrganizationId,
    amount: u64,
}

impl Event for WageDue {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("wage-due");
    const OWNER: SystemId = EmploymentSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl WageDue {
    pub(crate) const fn new(employee: PersonId, employer: OrganizationId, amount: u64) -> Self {
        Self {
            employee,
            employer,
            amount,
        }
    }

    /// Who is owed.
    pub const fn employee(&self) -> PersonId {
        self.employee
    }

    /// Who owes.
    pub const fn employer(&self) -> OrganizationId {
        self.employer
    }

    /// How much, in integer minor units; never zero.
    pub const fn amount(&self) -> u64 {
        self.amount
    }
}
