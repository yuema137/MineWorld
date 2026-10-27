//! The market packs' records and requests, as a controller reads and builds them: by slug, through
//! mirrors with the owners' labels (`ARC-182`; `pr-s31d-reporter-policies.md` §5.7).
//!
//! `rule-controller` may not link a market pack (`ARC-35` check 2), so each type here declares the
//! owner's component or action type, owner system id and schema version, and refuses unknown fields.
//! `payload_for` refuses a newer schema, so an owner that changes a shape makes a policy do nothing rather
//! than guess. `systems/economy/tests/controller_wire.rs` holds every mirror equal to its owner's type.

use mineworld_contracts::{
    Action, ActionCategory, ActionTypeId, Component, ComponentSchemaVersion, ComponentTypeId,
    ItemId, OrganizationId, SystemId,
};
use serde::{Deserialize, Serialize};

/// economy's system id.
pub const ECONOMY: SystemId = SystemId::from_static("economy");
/// inventory's system id.
pub const INVENTORY: SystemId = SystemId::from_static("inventory");
/// item's system id.
pub const ITEM: SystemId = SystemId::from_static("item");

/// economy's `buy` action type.
pub const BUY: ActionTypeId = ActionTypeId::from_static("buy");

/// One line of a person seller's `asks` view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskSeen {
    /// Which kind.
    pub item: ItemId,
    /// What one costs, in minor units.
    pub price: u64,
    /// Whether the seller holds one now.
    pub available: bool,
}

/// economy's `asks` view of a person seller: `{ asks: [{ item, price, available }] }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AsksSeen {
    /// Every ask, in item order.
    pub asks: Vec<AskSeen>,
}

impl Component for AsksSeen {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("asks");
    const OWNER: SystemId = ECONOMY;
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(1);
}

/// One line of a shop's listing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListedSeen {
    /// Which kind.
    pub item: ItemId,
    /// What it costs here now, in minor units (after any active adjustment).
    pub price: u64,
    /// How many the operator holds now.
    pub in_stock: u32,
}

/// economy's shop listing on a place: `{ operator, listed: [{ item, price, in_stock }] }`, under the
/// `shop` component's labels (schema 2 since WE-a).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListingSeen {
    /// Who runs the shop.
    pub operator: OrganizationId,
    /// Every kind the operator asks for, in item order.
    pub listed: Vec<ListedSeen>,
}

impl Component for ListingSeen {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("shop");
    const OWNER: SystemId = ECONOMY;
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(2);
}

/// Which side of an amount the currency's symbol is written on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolAtSeen {
    /// Before the amount.
    Before,
    /// After the amount.
    After,
}

/// One kind's reference value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceSeen {
    /// Which kind.
    pub item: ItemId,
    /// Its reference value, in minor units.
    pub value: u64,
}

/// economy's `currency` record: `{ name, symbol, exponent, symbol_at, references: [{ item, value }] }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CurrencySeen {
    /// The currency's name.
    pub name: String,
    /// Its symbol.
    pub symbol: String,
    /// How many digits of an amount are minor units.
    pub exponent: u8,
    /// Which side the symbol is written on.
    pub symbol_at: SymbolAtSeen,
    /// Every kind's reference value, in item order.
    pub references: Vec<ReferenceSeen>,
}

impl Component for CurrencySeen {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("currency");
    const OWNER: SystemId = ECONOMY;
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(1);
}

impl CurrencySeen {
    /// The reference value of `item`, if it has one.
    pub fn reference(&self, item: ItemId) -> Option<u64> {
        self.references
            .iter()
            .find(|line| line.item == item)
            .map(|line| line.value)
    }
}

/// One held kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeldSeen {
    /// Which kind.
    pub item: ItemId,
    /// How many.
    pub count: u32,
}

/// inventory's `holdings` record to its holder: `{ held: [{ item, count }], capacity? }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HoldingsSeen {
    /// Every kind held, in item order.
    pub held: Vec<HeldSeen>,
    /// The holder's capacity, when bounded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capacity: Option<u32>,
}

impl Component for HoldingsSeen {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("holdings");
    const OWNER: SystemId = INVENTORY;
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(1);
}

impl HoldingsSeen {
    /// How many of `item` — zero when none.
    pub fn count(&self, item: ItemId) -> u32 {
        self.held
            .iter()
            .find(|line| line.item == item)
            .map_or(0, |line| line.count)
    }
}

/// One declared kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CataloguedSeen {
    /// Which kind.
    pub item: ItemId,
    /// Its category slug.
    pub category: String,
    /// What people call it.
    pub name: String,
}

/// item's `item-catalogue` record on a place: `{ kinds: [{ item, category, name }] }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogueSeen {
    /// Every declared kind, in item order.
    pub kinds: Vec<CataloguedSeen>,
}

impl Component for CatalogueSeen {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("item-catalogue");
    const OWNER: SystemId = ITEM;
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(1);
}

impl CatalogueSeen {
    /// The category of `item`, if declared.
    pub fn category(&self, item: ItemId) -> Option<&str> {
        self.kinds
            .iter()
            .find(|kind| kind.item == item)
            .map(|kind| kind.category.as_str())
    }
}

/// Whose ask a seller action changes: the shop the actor keeps and stands in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShopSideSeen {
    /// The shop the actor keeps, at the place it stands in.
    Shop,
}

/// economy's `set-price { item, price, for? }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetPrice {
    /// Which kind.
    pub item: ItemId,
    /// The new asking price, in minor units.
    pub price: u64,
    /// `shop` for the kept shop's ask; absent for one's own.
    #[serde(rename = "for", default, skip_serializing_if = "Option::is_none")]
    pub side: Option<ShopSideSeen>,
}

impl Action for SetPrice {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("set-price");
    const OWNER: SystemId = ECONOMY;
    const CATEGORY: ActionCategory = ActionCategory::Commerce;
}

/// economy's `list-for-sale { item, price, for? }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListForSale {
    /// Which kind.
    pub item: ItemId,
    /// The asking price, in minor units.
    pub price: u64,
    /// `shop` for the kept shop's ask; absent for one's own.
    #[serde(rename = "for", default, skip_serializing_if = "Option::is_none")]
    pub side: Option<ShopSideSeen>,
}

impl Action for ListForSale {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("list-for-sale");
    const OWNER: SystemId = ECONOMY;
    const CATEGORY: ActionCategory = ActionCategory::Commerce;
}

/// The payload of an offered `buy`: `{ item, price }`. Decoded only; a buy is always submitted as the
/// complete affordance's own request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuySeen {
    /// Which kind.
    pub item: ItemId,
    /// The asking price the buyer sees, in minor units.
    pub price: u64,
}
