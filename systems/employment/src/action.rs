//! The action this pack provides: `penalize`, a manager docking an employee's pay for a lateness
//! (`docs/DECISIONS.md` `ARC-287`).

use mineworld_contracts::{Action, ActionTypeId, PersonId, SpatialRequirement, SystemId};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::system::EmploymentSystem;

/// Dock `amount` minor units from `employee` for their open lateness.
///
/// The employee is named in the payload, not as the request's target, so a manager may penalize from
/// anywhere: perception asks for targeted offers only of the people an observer perceives. Offered
/// **complete** to the manager, one per open lateness, with `amount` the bound; a requester that wants
/// less submits the same payload with a smaller amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Penalize {
    employee: PersonId,
    amount: u64,
}

impl Action for Penalize {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("penalize");
    const OWNER: SystemId = EmploymentSystem::ID;
}

impl Penalize {
    /// Dock `amount` from `employee`.
    pub const fn new(employee: PersonId, amount: u64) -> Self {
        Self { employee, amount }
    }

    /// Whose pay.
    pub const fn employee(&self) -> PersonId {
        self.employee
    }

    /// How much, in minor units.
    pub const fn amount(&self) -> u64 {
        self.amount
    }
}

/// What a penalty requires of space: nothing. A manager docks pay from anywhere (§0.6).
pub const PENALIZE_REQUIREMENT: SpatialRequirement = SpatialRequirement::NONE;
