//! The action this pack provides: buy, and what decides whether one can happen.

use mineworld_contracts::{
    Action, ActionTypeId, EntityId, ItemId, PlaceId, SpatialRequirement, SystemId,
};
use mineworld_kernel::{SystemIdentity, WorldRead};
use serde::{Deserialize, Serialize};

use crate::component::Shop;
use crate::money::admit_payment;
use crate::system::EconomySystem;

/// Buy one `item` from the shop the buyer is standing in. No target: the shop is wherever the buyer
/// is.
///
/// A bounded choice — one of the kinds the shop prices — so the pack can offer every request it would
/// accept as a complete affordance (`ARC-34`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Buy {
    item: ItemId,
}

impl Action for Buy {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("buy");
    const OWNER: SystemId = EconomySystem::ID;
}

impl Buy {
    /// Buy one `item`.
    pub const fn new(item: ItemId) -> Self {
        Self { item }
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }
}

/// What a buy requires of space: the buyer in the shop's place, and the purchase available.
///
/// Target-less, because the shop is the place; an offer without a target can only say "at that
/// place" (step-10 F-48). Availability carries the three reasons a buy can fail — out of stock,
/// cannot pay, cannot carry — as one `TargetUnavailable` (step-10 QS-44).
pub const fn buy_requirement(shop: PlaceId) -> SpatialRequirement {
    SpatialRequirement::at_place(shop).requiring_target_available()
}

/// Whether `buyer` can buy one `item` from `shop` now: it is priced there, the buyer can pay the price
/// to the operator, and inventory would move one from the operator to the buyer (the operator holds
/// one, the buyer can carry it).
pub fn purchasable(world: &WorldRead<'_>, buyer: EntityId, shop: &Shop, item: ItemId) -> bool {
    let Some(price) = shop.price(item) else {
        return false;
    };
    let operator = shop.operator().entity_id();
    admit_payment(world, buyer, operator, price).is_ok()
        && mineworld_inventory::admit_transfer(world, operator, buyer, item, 1).is_ok()
}
