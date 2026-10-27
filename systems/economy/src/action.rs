//! The actions this pack provides — buy, and a seller's list-for-sale, set-price and withdraw — and
//! what each requires of space (`ARC-112`).

use mineworld_contracts::{
    Action, ActionCategory, ActionTypeId, ItemId, Millimetres, PlaceId, SpatialRequirement,
    SystemId,
};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::system::EconomySystem;

/// How close a buyer must stand to a person seller: within arm's reach of a step, the reach `give`
/// and `talk` already use.
pub const TRADE_RANGE: Millimetres = Millimetres::new(3_000);

/// Buy one `item` at `price`, the asking price the buyer saw (`ARC-112`). With no target the seller is
/// the shop the buyer stands in; with a target, that person seller.
///
/// A bounded choice — one per ask the seller has — so the pack offers every request it would accept as
/// a complete affordance (`ARC-34`). A price that is not the current asking price is refused
/// `price-changed`, and nothing moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Buy {
    item: ItemId,
    price: u64,
}

impl Action for Buy {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("buy");
    const OWNER: SystemId = EconomySystem::ID;
    const CATEGORY: ActionCategory = ActionCategory::Commerce;
}

impl Buy {
    /// Buy one `item` at `price`.
    pub const fn new(item: ItemId, price: u64) -> Self {
        Self { item, price }
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// The price the buyer agrees to pay, in minor units.
    pub const fn price(&self) -> u64 {
        self.price
    }
}

/// Whose ask a seller action changes: the actor's own when absent, the shop the actor keeps and
/// stands in with `for: shop`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShopSide {
    /// The shop the actor keeps, at the place it stands in.
    Shop,
}

/// Start asking `price` for `item`: for oneself, or — `for: shop` — for the shop one keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListForSale {
    item: ItemId,
    price: u64,
    #[serde(rename = "for", default, skip_serializing_if = "Option::is_none")]
    side: Option<ShopSide>,
}

impl Action for ListForSale {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("list-for-sale");
    const OWNER: SystemId = EconomySystem::ID;
    const CATEGORY: ActionCategory = ActionCategory::Commerce;
}

impl ListForSale {
    /// Ask `price` for `item`, for oneself (`side` absent) or for the shop.
    pub const fn new(item: ItemId, price: u64, side: Option<ShopSide>) -> Self {
        Self { item, price, side }
    }
}

/// Change the asking price of `item` to `price`: one's own, or — `for: shop` — the shop's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetPrice {
    item: ItemId,
    price: u64,
    #[serde(rename = "for", default, skip_serializing_if = "Option::is_none")]
    side: Option<ShopSide>,
}

impl Action for SetPrice {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("set-price");
    const OWNER: SystemId = EconomySystem::ID;
    const CATEGORY: ActionCategory = ActionCategory::Commerce;
}

impl SetPrice {
    /// Re-price `item` to `price`, for oneself (`side` absent) or for the shop.
    pub const fn new(item: ItemId, price: u64, side: Option<ShopSide>) -> Self {
        Self { item, price, side }
    }
}

/// Stop asking for `item`: one's own ask, or — `for: shop` — the shop's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Withdraw {
    item: ItemId,
    #[serde(rename = "for", default, skip_serializing_if = "Option::is_none")]
    side: Option<ShopSide>,
}

impl Action for Withdraw {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("withdraw");
    const OWNER: SystemId = EconomySystem::ID;
    const CATEGORY: ActionCategory = ActionCategory::Commerce;
}

impl Withdraw {
    /// Withdraw the ask for `item`, one's own (`side` absent) or the shop's.
    pub const fn new(item: ItemId, side: Option<ShopSide>) -> Self {
        Self { item, side }
    }
}

/// What a seller action asks for, whichever of the three it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Change {
    /// `list-for-sale` at this price.
    List(u64),
    /// `set-price` to this price.
    Reprice(u64),
    /// `withdraw`.
    Withdraw,
}

/// One seller action, decoded: the kind, the change, and whose ask.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SellerRequest {
    pub(crate) item: ItemId,
    pub(crate) change: Change,
    pub(crate) for_shop: bool,
}

impl From<ListForSale> for SellerRequest {
    fn from(action: ListForSale) -> Self {
        Self {
            item: action.item,
            change: Change::List(action.price),
            for_shop: action.side.is_some(),
        }
    }
}

impl From<SetPrice> for SellerRequest {
    fn from(action: SetPrice) -> Self {
        Self {
            item: action.item,
            change: Change::Reprice(action.price),
            for_shop: action.side.is_some(),
        }
    }
}

impl From<Withdraw> for SellerRequest {
    fn from(action: Withdraw) -> Self {
        Self {
            item: action.item,
            change: Change::Withdraw,
            for_shop: action.side.is_some(),
        }
    }
}

/// What a shop buy requires of space: the buyer in the shop's place (step-10 F-48). Availability is
/// asked too, and always answered `true` once every reason a buy can fail has its own code
/// (`pr-s31a-currency-asks.md` §5.5).
pub const fn buy_requirement(shop: PlaceId) -> SpatialRequirement {
    SpatialRequirement::at_place(shop).requiring_target_available()
}

/// What a buy from a person seller requires of space: the same place, within [`TRADE_RANGE`].
pub fn person_buy_requirement() -> SpatialRequirement {
    SpatialRequirement::same_place()
        .within(TRADE_RANGE)
        .expect("a positive reach")
        .requiring_target_available()
}

/// What a seller action requires of space: nothing for one's own ask; standing in the shop for the
/// shop's (SD-S31a-3: the action names no target, so the requirement names the place).
pub const fn seller_requirement(shop: Option<PlaceId>) -> SpatialRequirement {
    match shop {
        Some(place) => SpatialRequirement::at_place(place),
        None => SpatialRequirement::NONE,
    }
}
