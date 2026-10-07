//! The action this pack provides: give.

use mineworld_contracts::{
    Action, ActionTypeId, ItemId, Millimetres, SpatialRequirement, SystemId,
};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::system::ItemTransferSystem;

/// How close a giver must be: within arm's reach of a step, the reach `talk` already uses.
pub const GIVE_RANGE: Millimetres = Millimetres::new(3_000);

/// Give `count` of `item` to the request's target, who must be a Person.
///
/// A bounded choice — one of the kinds the giver holds — so the pack can offer every request it would
/// accept as a complete affordance (`ARC-34`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Give {
    item: ItemId,
    count: u32,
}

impl Action for Give {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("give");
    const OWNER: SystemId = ItemTransferSystem::ID;
}

impl Give {
    /// Give `count` of `item`.
    pub const fn new(item: ItemId, count: u32) -> Self {
        Self { item, count }
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// How many.
    pub const fn count(&self) -> u32 {
        self.count
    }
}

/// What a give requires of space: the same place, within [`GIVE_RANGE`], and a target available to
/// take it — "give item" as `CORE_CONCEPTS.md` §6.3 names it.
pub fn give_requirement() -> SpatialRequirement {
    SpatialRequirement::same_place()
        .within(GIVE_RANGE)
        .expect("a positive reach")
        .requiring_target_available()
}
