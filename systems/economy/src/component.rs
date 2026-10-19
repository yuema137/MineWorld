//! The state this pack owns — a holder's money and a place's shop — and the listing it discloses.

use mineworld_contracts::{ItemId, OrganizationId};
use mineworld_happenings::HappeningRef;
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
///
/// Its prices are the authored ones adjusted, while a configured happening runs, by
/// [`Adjustment`]s — written only by this pack's reductions of `prices-adjusted` and
/// `prices-restored`. A shop never adjusted encodes exactly as it did before adjustments existed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shop {
    operator: OrganizationId,
    prices: Vec<Price>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    adjustments: Vec<Adjustment>,
}

/// A multiplier on every price of a shop for as long as one happening runs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Adjustment {
    happening: HappeningRef,
    permille: u16,
}

impl Adjustment {
    /// The happening it lasts for.
    pub const fn happening(&self) -> &HappeningRef {
        &self.happening
    }

    /// The multiplier, per mille.
    pub const fn permille(&self) -> u16 {
        self.permille
    }
}

impl Shop {
    pub(crate) const fn new(operator: OrganizationId, prices: Vec<Price>) -> Self {
        Self {
            operator,
            prices,
            adjustments: Vec::new(),
        }
    }

    /// The adjustments in force, in happening order.
    pub fn adjustments(&self) -> &[Adjustment] {
        &self.adjustments
    }

    /// The shop with `happening`'s adjustment added (once).
    pub(crate) fn adjusted(mut self, happening: HappeningRef, permille: u16) -> Self {
        let adjustment = Adjustment {
            happening,
            permille,
        };
        if let Err(at) = self.adjustments.binary_search(&adjustment) {
            self.adjustments.insert(at, adjustment);
        }
        self
    }

    /// The shop without `happening`'s adjustment.
    pub(crate) fn restored(mut self, happening: &HappeningRef) -> Self {
        self.adjustments
            .retain(|adjustment| adjustment.happening != *happening);
        self
    }

    /// Whether `happening` adjusts this shop now.
    pub fn is_adjusted_by(&self, happening: &HappeningRef) -> bool {
        self.adjustments
            .iter()
            .any(|adjustment| adjustment.happening == *happening)
    }

    /// What `item` costs here now, if it is sold here: its authored price, then each adjustment in
    /// happening order, `p ← max(1, (p × permille + 500) ÷ 1 000)` — integers, rounded half up, never
    /// free. The one price every reader uses: `buy`'s validation, its payment, the offers and the
    /// listing.
    pub fn effective_price(&self, item: ItemId) -> Option<u64> {
        let base = self.price(item)?;
        Some(self.adjustments.iter().fold(base, |price, adjustment| {
            let scaled = (u128::from(price) * u128::from(adjustment.permille) + 500) / 1_000;
            u64::try_from(scaled).unwrap_or(u64::MAX).max(1)
        }))
    }

    /// The Organization that runs it and is paid.
    pub const fn operator(&self) -> OrganizationId {
        self.operator
    }

    /// Every priced kind, in item order.
    pub fn prices(&self) -> &[Price] {
        &self.prices
    }

    /// What `item` costs here as authored, before any adjustment, if it is sold here. Readers of what a
    /// purchase costs use [`Shop::effective_price`].
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
                    price: shop.effective_price(price.item).unwrap_or(price.price),
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

#[cfg(test)]
mod tests {
    //! Effective-price rounding (WE-a design §6.2, §9.2 Unit): expected values worked by hand.

    use mineworld_contracts::{EntityId, EntityType, ItemId, OrganizationId};
    use mineworld_happenings::HappeningRef;

    use super::{Price, Shop};

    fn happening(kind: &str, start: i64) -> HappeningRef {
        let place = serde_json::json!({ "entity": "9", "entity_type": "place" });
        serde_json::from_value(serde_json::json!({ "kind": kind, "place": place, "start": start }))
            .expect("a happening")
    }

    fn shop(prices: &[u64]) -> (Shop, Vec<ItemId>) {
        let items: Vec<ItemId> = (0..prices.len())
            .map(|index| {
                ItemId::new(EntityId::from_raw(20 + index as u64), EntityType::Item)
                    .expect("an item")
            })
            .collect();
        let operator =
            OrganizationId::new(EntityId::from_raw(5), EntityType::Organization).expect("an org");
        let priced = items
            .iter()
            .zip(prices)
            .map(|(item, price)| Price::new(*item, *price))
            .collect();
        (Shop::new(operator, priced), items)
    }

    /// 333 × 900 ‰ = 299.7 → 300; 5 × 900 ‰ = 4.5 → 5 (half up); 1 × 100 ‰ = 0.1 → 1 (never free).
    #[test]
    fn one_adjustment_rounds_half_up_and_never_to_zero() {
        let (unadjusted, items) = shop(&[333, 5, 1]);
        let adjusted = unadjusted.adjusted(happening("market-day", 100), 900);
        assert_eq!(adjusted.effective_price(items[0]), Some(300));
        assert_eq!(adjusted.effective_price(items[1]), Some(5));
        let cheap = shop(&[1]).0.adjusted(happening("market-day", 100), 100);
        assert_eq!(cheap.effective_price(items[0]), Some(1));
    }

    /// Two adjustments multiply in happening order, each rounded: 333 → (by 900 at instant 100) 300 →
    /// (by 1 500 at instant 200) 450; and the other order of starts gives 333 → 500 → 450.
    /// Restoring one leaves the other.
    #[test]
    fn two_adjustments_multiply_in_happening_order() {
        let (unadjusted, items) = shop(&[333]);
        let both = unadjusted
            .clone()
            .adjusted(happening("fair", 200), 1_500)
            .adjusted(happening("market-day", 100), 900);
        assert_eq!(both.effective_price(items[0]), Some(450));
        let restored = both.restored(&happening("market-day", 100));
        assert_eq!(restored.effective_price(items[0]), Some(500));
        let reversed = unadjusted
            .adjusted(happening("market-day", 200), 900)
            .adjusted(happening("fair", 100), 1_500);
        assert_eq!(reversed.effective_price(items[0]), Some(450));
    }

    /// An unadjusted shop encodes exactly as it did before adjustments existed.
    #[test]
    fn an_unadjusted_shop_encodes_without_adjustments() {
        let (unadjusted, _) = shop(&[100]);
        let text = serde_json::to_string(&unadjusted).expect("encodes");
        assert!(!text.contains("adjustments"), "{text}");
        let adjusted = unadjusted.adjusted(happening("market-day", 0), 900);
        let text = serde_json::to_string(&adjusted.restored(&happening("market-day", 0)))
            .expect("encodes");
        assert!(!text.contains("adjustments"), "{text}");
    }
}
