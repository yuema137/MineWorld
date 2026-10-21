//! The state this pack owns — a holder's money, a seller's asks, a place's shop, an item kind's
//! reference value — and the views it discloses from them.

use mineworld_contracts::{EntityId, ItemId, OrganizationId, PersonId, PlaceId};
use mineworld_happenings::HappeningRef;
use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::system::EconomySystem;

/// How many keepers one shop may name (`ARC-112`).
pub const MAX_KEEPERS: usize = 8;

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

/// One seller's standing offer to sell one kind at one unit price, in minor units (`ARC-112`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ask {
    item: ItemId,
    price: u64,
}

impl Ask {
    pub(crate) const fn new(item: ItemId, price: u64) -> Self {
        Self { item, price }
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// What one costs.
    pub const fn price(&self) -> u64 {
        self.price
    }
}

/// `asks`: what a seller — a Person for itself, an Organization through its shop — offers to sell, at
/// most one ask per kind, in item order (`ARC-112`). An ask covers every unit the seller holds; what is
/// in stock stays inventory's `Holdings`, read when shown (step-10 QS-45). Written only by this pack's
/// reductions of `listed`, `price-set` and `withdrawn`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Asks {
    asks: Vec<Ask>,
}

owned_component! {
    component = Asks,
    owner = EconomySystem,
    component_type = "asks",
    schema_version = 1,
}

impl Asks {
    /// Every ask, in item order.
    pub fn asks(&self) -> &[Ask] {
        &self.asks
    }

    /// The asking price of `item`, if the seller asks one.
    pub fn price(&self, item: ItemId) -> Option<u64> {
        self.asks
            .binary_search_by_key(&item, Ask::item)
            .ok()
            .map(|at| self.asks[at].price)
    }

    /// These asks with `ask` added; `None` when the kind already has one.
    pub(crate) fn listing(&self, ask: Ask) -> Option<Self> {
        let at = self.asks.binary_search_by_key(&ask.item, Ask::item).err()?;
        let mut asks = self.asks.clone();
        asks.insert(at, ask);
        Some(Self { asks })
    }

    /// These asks with `item` re-priced from `from` to `to`; `None` when the kind has no ask, its
    /// price is not `from`, or `to` equals it.
    pub(crate) fn repriced(&self, item: ItemId, from: u64, to: u64) -> Option<Self> {
        let at = self.asks.binary_search_by_key(&item, Ask::item).ok()?;
        if self.asks[at].price != from || from == to {
            return None;
        }
        let mut asks = self.asks.clone();
        asks[at].price = to;
        Some(Self { asks })
    }

    /// These asks without `item`'s; `None` when the kind has no ask.
    pub(crate) fn withdrawn(&self, item: ItemId) -> Option<Self> {
        let at = self.asks.binary_search_by_key(&item, Ask::item).ok()?;
        let mut asks = self.asks.clone();
        asks.remove(at);
        Some(Self { asks })
    }
}

/// A shop, on the Place it is in: the Organization that runs it and is paid, and the Persons it names
/// as keepers, who may change its asks while standing in it (`ARC-112`, Q-S31-8).
///
/// Authored on its operator's `economy:` section (a pack owns one section, step-10 F-47) and written
/// on the place by this pack's reduction of `shop-opened`. Its prices are not here: they are the
/// operator's [`Asks`], so the price at the counter and the price a keeper sets are one record. What is
/// in stock is the operator's `Holdings`, inventory's state (step-10 QS-45).
///
/// While a configured happening runs, every asking price is scaled by the shop's [`Adjustment`]s —
/// written only by this pack's reductions of `prices-adjusted` and `prices-restored` (WE-a design
/// §6.2). A shop never adjusted encodes without them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shop {
    operator: OrganizationId,
    keepers: Vec<PersonId>,
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
    pub(crate) const fn new(operator: OrganizationId, keepers: Vec<PersonId>) -> Self {
        Self {
            operator,
            keepers,
            adjustments: Vec::new(),
        }
    }

    /// The Organization that runs it and is paid.
    pub const fn operator(&self) -> OrganizationId {
        self.operator
    }

    /// Its keepers, in the order the operator named them.
    pub fn keepers(&self) -> &[PersonId] {
        &self.keepers
    }

    /// Whether `person` keeps this shop.
    pub fn kept_by(&self, person: EntityId) -> bool {
        self.keepers
            .iter()
            .any(|keeper| keeper.entity_id() == person)
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

    /// What an asking price of `base` costs here now: each adjustment in happening order,
    /// `p ← max(1, (p × permille + 500) ÷ 1 000)` — integers, rounded half up, never free. Every reader
    /// of a shop price applies it to the operator's ask: `buy`'s validation, its payment, the offers
    /// and the listing (WE-a design §6.2 over `ARC-112`'s asks).
    pub fn adjust(&self, base: u64) -> u64 {
        self.adjustments.iter().fold(base, |price, adjustment| {
            let scaled = (u128::from(price) * u128::from(adjustment.permille) + 500) / 1_000;
            u64::try_from(scaled).unwrap_or(u64::MAX).max(1)
        })
    }
}

owned_component! {
    component = Shop,
    owner = EconomySystem,
    component_type = "shop",
    schema_version = 2,
}

/// `reference-value`: an item kind's authored typical worth, in minor units (`ARC-113`). Never a price:
/// no settlement path reads it. Written only by this pack's reduction of `reference-declared`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceValue {
    value: u64,
}

impl ReferenceValue {
    pub(crate) const fn new(value: u64) -> Self {
        Self { value }
    }

    /// The value, in minor units.
    pub const fn value(&self) -> u64 {
        self.value
    }
}

owned_component! {
    component = ReferenceValue,
    owner = EconomySystem,
    component_type = "reference-value",
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

/// What a shop discloses to whoever perceives its place: who runs it, and per kind the operator asks
/// for, its price and how many the operator holds (step-10 QS-45). The `shop` component as an observer
/// sees it; its shape did not change when prices moved to the operator's asks (SD-S31a-12).
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
    pub(crate) fn of(shop: &Shop, asks: &[Ask], in_stock: impl Fn(ItemId) -> u32) -> Self {
        Self {
            operator: shop.operator,
            listed: asks
                .iter()
                .map(|ask| Listed {
                    item: ask.item,
                    price: shop.adjust(ask.price),
                    in_stock: in_stock(ask.item),
                })
                .collect(),
        }
    }

    /// Who runs the shop.
    pub const fn operator(&self) -> OrganizationId {
        self.operator
    }

    /// Every kind the operator asks for, in item order.
    pub fn listed(&self) -> &[Listed] {
        &self.listed
    }
}

/// One ask as a person seller's `asks` view shows it: the kind, its price, and whether the seller holds
/// one now — never how many (I-S31-9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AskLine {
    item: ItemId,
    price: u64,
    available: bool,
}

impl AskLine {
    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// What one costs.
    pub const fn price(&self) -> u64 {
        self.price
    }

    /// Whether the seller holds one now.
    pub const fn available(&self) -> bool {
        self.available
    }
}

/// What a person seller discloses to whoever perceives them: per ask, its price and whether it is
/// available (`ARC-112`). The `asks` component as an observer sees it; a view, never stored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AsksView {
    asks: Vec<AskLine>,
}

impl AsksView {
    pub(crate) fn of(asks: &Asks, holds: impl Fn(ItemId) -> bool) -> Self {
        Self {
            asks: asks
                .asks
                .iter()
                .map(|ask| AskLine {
                    item: ask.item,
                    price: ask.price,
                    available: holds(ask.item),
                })
                .collect(),
        }
    }

    /// Every ask, in item order.
    pub fn asks(&self) -> &[AskLine] {
        &self.asks
    }
}

/// `wealth`: what a holder is worth — its wallet plus, for each kind it holds that has a reference
/// value, count × value — and how many units it holds of kinds without one (`ARC-114`). Disclosed to
/// the holder only; computed when disclosed and never stored, so no settlement reads it (I-S31-6). A
/// declared component type so that perception keeps it; no entity carries it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wealth {
    amount: u64,
    unvalued: u64,
}

owned_component! {
    component = Wealth,
    owner = EconomySystem,
    component_type = "wealth",
    schema_version = 1,
}

impl Wealth {
    pub(crate) const fn new(amount: u64, unvalued: u64) -> Self {
        Self { amount, unvalued }
    }

    /// Wallet plus reference values of holdings, in minor units, saturating.
    pub const fn amount(&self) -> u64 {
        self.amount
    }

    /// Units held of kinds with no reference value.
    pub const fn unvalued(&self) -> u64 {
        self.unvalued
    }
}

#[cfg(test)]
mod tests {
    //! Effective-price rounding (WE-a design §6.2, §9.2 Unit): expected values worked by hand, applied
    //! to an asking price (`ARC-112`).

    use mineworld_contracts::{EntityId, EntityType, OrganizationId};
    use mineworld_happenings::HappeningRef;

    use super::Shop;

    fn happening(kind: &str, start: i64) -> HappeningRef {
        let place = serde_json::json!({ "entity": "9", "entity_type": "place" });
        serde_json::from_value(serde_json::json!({ "kind": kind, "place": place, "start": start }))
            .expect("a happening")
    }

    fn shop() -> Shop {
        let operator =
            OrganizationId::new(EntityId::from_raw(5), EntityType::Organization).expect("an org");
        Shop::new(operator, Vec::new())
    }

    /// 333 × 900 ‰ = 299.7 → 300; 5 × 900 ‰ = 4.5 → 5 (half up); 1 × 100 ‰ = 0.1 → 1 (never free).
    #[test]
    fn one_adjustment_rounds_half_up_and_never_to_zero() {
        let adjusted = shop().adjusted(happening("market-day", 100), 900);
        assert_eq!(adjusted.adjust(333), 300);
        assert_eq!(adjusted.adjust(5), 5);
        let cheap = shop().adjusted(happening("market-day", 100), 100);
        assert_eq!(cheap.adjust(1), 1);
    }

    /// Two adjustments multiply in happening order, each rounded: 333 → (by 900 at instant 100) 300 →
    /// (by 1 500 at instant 200) 450; and the other order of starts gives 333 → 500 → 450.
    /// Restoring one leaves the other.
    #[test]
    fn two_adjustments_multiply_in_happening_order() {
        let both = shop()
            .adjusted(happening("fair", 200), 1_500)
            .adjusted(happening("market-day", 100), 900);
        assert_eq!(both.adjust(333), 450);
        let restored = both.restored(&happening("market-day", 100));
        assert_eq!(restored.adjust(333), 500);
        let reversed = shop()
            .adjusted(happening("market-day", 200), 900)
            .adjusted(happening("fair", 100), 1_500);
        assert_eq!(reversed.adjust(333), 450);
    }

    /// An unadjusted shop encodes without adjustments.
    #[test]
    fn an_unadjusted_shop_encodes_without_adjustments() {
        let text = serde_json::to_string(&shop()).expect("encodes");
        assert!(!text.contains("adjustments"), "{text}");
        let adjusted = shop().adjusted(happening("market-day", 0), 900);
        let text = serde_json::to_string(&adjusted.restored(&happening("market-day", 0)))
            .expect("encodes");
        assert!(!text.contains("adjustments"), "{text}");
    }
}

/// `shop-stock`: every shop of the world, by its place, and the kinds its operator asks for and holds
/// at least one of — kinds only, never counts (S35 N-h R5, Q-Nh-4). What a shop window or a "sold out"
/// sign tells a passer-by, so a person can skip a shop that has nothing for them without walking in to
/// find out, and a controller that keeps no memory needs none (F-Ng-3).
///
/// Disclosed to each living Person about itself, wherever it stands: perception lists only the place one
/// stands in, so a shop's own record (its [`Listing`]) is seen from inside it alone. Computed when
/// disclosed and never stored; a declared component type so that perception keeps it, as [`Wealth`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShopStock {
    shops: Vec<StockedShop>,
}

/// One shop's place and the kinds it has in stock, in item order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StockedShop {
    place: PlaceId,
    stocked: Vec<ItemId>,
}

owned_component! {
    component = ShopStock,
    owner = EconomySystem,
    component_type = "shop-stock",
    schema_version = 1,
}

impl ShopStock {
    pub(crate) const fn new(shops: Vec<StockedShop>) -> Self {
        Self { shops }
    }

    /// Every shop, in place order.
    pub fn shops(&self) -> &[StockedShop] {
        &self.shops
    }
}

impl StockedShop {
    pub(crate) const fn new(place: PlaceId, stocked: Vec<ItemId>) -> Self {
        Self { place, stocked }
    }

    /// The shop's place.
    pub const fn place(&self) -> PlaceId {
        self.place
    }

    /// The asked kinds the operator holds at least one of, in item order.
    pub fn stocked(&self) -> &[ItemId] {
        &self.stocked
    }
}
