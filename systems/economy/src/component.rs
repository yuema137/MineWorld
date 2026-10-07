//! The state this pack owns — a holder's money and a place's shop — and the listing it discloses.

use mineworld_contracts::{ItemId, OrganizationId};
use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::system::EconomySystem;

/// How much money a Person or an Organization has, in integer minor units (I-6).
///
/// `u64`: a negative balance cannot be represented, so a payment larger than the balance is refused,
/// never recorded (`ARC-38` item 3). Written only by this pack's reductions of `funded` and
/// `money-transferred`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wallet {
    balance: u64,
}

impl Wallet {
    pub(crate) const fn new(balance: u64) -> Self {
        Self { balance }
    }

    /// The balance, in minor units.
    pub const fn balance(&self) -> u64 {
        self.balance
    }
}

owned_component! {
    component = Wallet,
    owner = EconomySystem,
    component_type = "wallet",
    schema_version = 1,
}

/// What one kind costs in a shop, in minor units; never zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Price {
    item: ItemId,
    price: u64,
}

impl Price {
    pub(crate) const fn new(item: ItemId, price: u64) -> Self {
        Self { item, price }
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// What it costs.
    pub const fn price(&self) -> u64 {
        self.price
    }
}

/// A shop, on the Place it is in: who runs it, and what it sells at what price.
///
/// Authored on its operator's `economy:` section (a pack owns one section, step-10 F-47) and written
/// on the place by this pack's reduction of `shop-opened`. What is in stock is not here: it is the
/// operator's `Holdings`, inventory's state, read when an offer or a listing is made — one truth about
/// holdings (step-10 QS-45).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shop {
    operator: OrganizationId,
    prices: Vec<Price>,
}

impl Shop {
    pub(crate) const fn new(operator: OrganizationId, prices: Vec<Price>) -> Self {
        Self { operator, prices }
    }

    /// The Organization that runs it and is paid.
    pub const fn operator(&self) -> OrganizationId {
        self.operator
    }

    /// Every priced kind, in item order.
    pub fn prices(&self) -> &[Price] {
        &self.prices
    }

    /// What `item` costs here, if it is sold here.
    pub fn price(&self, item: ItemId) -> Option<u64> {
        self.prices
            .binary_search_by_key(&item, Price::item)
            .ok()
            .map(|at| self.prices[at].price)
    }
}

owned_component! {
    component = Shop,
    owner = EconomySystem,
    component_type = "shop",
    schema_version = 1,
}

/// One line of a shop's listing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Listed {
    item: ItemId,
    price: u64,
    in_stock: u32,
}

impl Listed {
    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// What it costs.
    pub const fn price(&self) -> u64 {
        self.price
    }

    /// How many the operator holds now.
    pub const fn in_stock(&self) -> u32 {
        self.in_stock
    }
}

/// What a shop discloses to whoever perceives its place: who runs it, and per priced kind its price
/// and how many the operator holds (step-10 QS-45). The `shop` component as an observer sees it.
///
/// A view, built when it is disclosed, never stored: the stock count is read from inventory's
/// `Holdings`, which economy may read because it depends on inventory, and never writes. It is how a
/// second person in the shop perceives a purchase without anybody's holdings being disclosed to them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Listing {
    operator: OrganizationId,
    listed: Vec<Listed>,
}

impl Listing {
    pub(crate) fn of(shop: &Shop, in_stock: impl Fn(ItemId) -> u32) -> Self {
        Self {
            operator: shop.operator,
            listed: shop
                .prices
                .iter()
                .map(|price| Listed {
                    item: price.item,
                    price: price.price,
                    in_stock: in_stock(price.item),
                })
                .collect(),
        }
    }

    /// Who runs the shop.
    pub const fn operator(&self) -> OrganizationId {
        self.operator
    }

    /// Every priced kind, in item order.
    pub fn listed(&self) -> &[Listed] {
        &self.listed
    }
}
