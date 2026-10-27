//! The facts this pack records, and reduces alone.
//!
//! None has a public constructor: this pack states every one. `money-transferred` is the only fact in
//! any pack that moves money; `listed` is the only fact that sets an asking price; `prices-adjusted`
//! and `prices-restored` are the only facts that scale a shop's prices, each caused by the happening
//! fact it answers (WE-a design §6.2).

use mineworld_contracts::{
    EntityId, Event, EventSchemaVersion, EventTypeId, ItemId, OrganizationId, PersonId, PlaceId,
    SystemId, WorldTime,
};
use mineworld_happenings::HappeningRef;
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::configuration::{Currency, ResolvedModifier};
use crate::proposal::ProposalId;
use crate::system::EconomySystem;

/// The world's currency, as `configure/economy.yaml` declared it (`ARC-111`): a genesis fact,
/// `SystemInternal`, about nobody (`ARC-61` item 8). Its payload is the currency's four fields, decoded
/// through their bounded types, so a value out of bounds cannot be reduced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CurrencyDeclared {
    currency: Currency,
}

impl Event for CurrencyDeclared {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("currency-declared");
    const OWNER: SystemId = EconomySystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl CurrencyDeclared {
    pub(crate) fn new(currency: &Currency) -> Self {
        Self {
            currency: currency.clone(),
        }
    }

    /// The declared currency.
    pub const fn currency(&self) -> &Currency {
        &self.currency
    }
}

/// `holder` begins with `balance` minor units: a genesis fact, from the holder's `economy:` section.
/// Visible to the holder only (`INV-13`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Funded {
    holder: EntityId,
    balance: u64,
}

impl Event for Funded {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("funded");
    const OWNER: SystemId = EconomySystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl Funded {
    pub(crate) const fn new(holder: EntityId, balance: u64) -> Self {
        Self { holder, balance }
    }

    /// Whose wallet.
    pub const fn holder(&self) -> EntityId {
        self.holder
    }

    /// The opening balance.
    pub const fn balance(&self) -> u64 {
        self.balance
    }
}

/// A shop opened in `place`, run by `operator`, with `keepers` who may change its asks: a genesis fact
/// from the operator's `economy:` section. Public: a shop is there for anyone to see. Its prices follow
/// as `listed` facts, the operator's asks (schema 2: `prices` left it, `ARC-112`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShopOpened {
    place: PlaceId,
    operator: OrganizationId,
    keepers: Vec<PersonId>,
}

impl Event for ShopOpened {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("shop-opened");
    const OWNER: SystemId = EconomySystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(2);
}

impl ShopOpened {
    pub(crate) const fn new(
        place: PlaceId,
        operator: OrganizationId,
        keepers: Vec<PersonId>,
    ) -> Self {
        Self {
            place,
            operator,
            keepers,
        }
    }

    /// Where.
    pub const fn place(&self) -> PlaceId {
        self.place
    }

    /// Who runs it.
    pub const fn operator(&self) -> OrganizationId {
        self.operator
    }

    /// Who keeps it, in the order the operator named them.
    pub fn keepers(&self) -> &[PersonId] {
        &self.keepers
    }
}

/// `seller` asks `price` for `item` (`ARC-112`): at genesis from a shop's `prices` or a person's
/// `asks`, and later from `list-for-sale`. Reduced into the seller's [`Asks`](crate::component::Asks).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Listed {
    seller: EntityId,
    item: ItemId,
    price: u64,
}

impl Event for Listed {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("listed");
    const OWNER: SystemId = EconomySystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl Listed {
    pub(crate) const fn new(seller: EntityId, item: ItemId, price: u64) -> Self {
        Self {
            seller,
            item,
            price,
        }
    }

    /// Who sells.
    pub const fn seller(&self) -> EntityId {
        self.seller
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// The asking price, in minor units.
    pub const fn price(&self) -> u64 {
        self.price
    }
}

/// `item`'s authored reference value is `value` (`ARC-113`): a genesis fact from an item file's
/// `economy: { reference }`, SystemInternal (Q-S31a-2). Reduced into the kind's
/// [`ReferenceValue`](crate::component::ReferenceValue).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceDeclared {
    item: ItemId,
    value: u64,
}

impl Event for ReferenceDeclared {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("reference-declared");
    const OWNER: SystemId = EconomySystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl ReferenceDeclared {
    pub(crate) const fn new(item: ItemId, value: u64) -> Self {
        Self { item, value }
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// Its reference value, in minor units.
    pub const fn value(&self) -> u64 {
        self.value
    }
}

/// `amount` minor units passed from `from` to `to`: a purchase (caused by the `buy`) or a wage
/// (caused by employment's `wage-due`). The only fact that moves money.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoneyTransferred {
    from: EntityId,
    to: EntityId,
    amount: u64,
}

impl Event for MoneyTransferred {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("money-transferred");
    const OWNER: SystemId = EconomySystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl MoneyTransferred {
    pub(crate) const fn new(from: EntityId, to: EntityId, amount: u64) -> Self {
        Self { from, to, amount }
    }

    /// Who paid.
    pub const fn from(&self) -> EntityId {
        self.from
    }

    /// Who was paid.
    pub const fn to(&self) -> EntityId {
        self.to
    }

    /// How much, in minor units.
    pub const fn amount(&self) -> u64 {
        self.amount
    }
}

/// `employer` could not pay `employee` the `amount` a `wage-due` named: nothing moved. Caused by the
/// `wage-due`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WageUnpaid {
    employee: PersonId,
    employer: OrganizationId,
    amount: u64,
}

impl Event for WageUnpaid {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("wage-unpaid");
    const OWNER: SystemId = EconomySystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl WageUnpaid {
    pub(crate) const fn new(employee: PersonId, employer: OrganizationId, amount: u64) -> Self {
        Self {
            employee,
            employer,
            amount,
        }
    }

    /// Who was not paid.
    pub const fn employee(&self) -> PersonId {
        self.employee
    }

    /// Who could not pay.
    pub const fn employer(&self) -> OrganizationId {
        self.employer
    }

    /// How much was due.
    pub const fn amount(&self) -> u64 {
        self.amount
    }
}

/// `employee` could not pay `employer` the `amount` a `wage-deduction-due` named: nothing moved. Caused
/// by the `wage-deduction-due` (`ARC-287`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeductionUnpaid {
    employee: PersonId,
    employer: OrganizationId,
    amount: u64,
}

impl Event for DeductionUnpaid {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("deduction-unpaid");
    const OWNER: SystemId = EconomySystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// The world's price modifiers, as configured, each shop resolved (`ARC-61`): a genesis fact,
/// SystemInternal, about nobody. A world with no `configure/economy.yaml` states none.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EconomyConfigured {
    price_modifiers: Vec<ResolvedModifier>,
}

impl Event for EconomyConfigured {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("economy-configured");
    const OWNER: SystemId = EconomySystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl DeductionUnpaid {
    pub(crate) const fn new(employee: PersonId, employer: OrganizationId, amount: u64) -> Self {
        Self {
            employee,
            employer,
            amount,
        }
    }

    /// Who could not pay.
    pub const fn employee(&self) -> PersonId {
        self.employee
    }

    /// Who was not paid.
    pub const fn employer(&self) -> OrganizationId {
        self.employer
    }

    /// How much was due.
    pub const fn amount(&self) -> u64 {
        self.amount
    }
}

impl EconomyConfigured {
    pub(crate) const fn new(price_modifiers: Vec<ResolvedModifier>) -> Self {
        Self { price_modifiers }
    }

    /// The modifiers, in file order.
    pub fn price_modifiers(&self) -> &[ResolvedModifier] {
        &self.price_modifiers
    }
}

/// A happening began that the world's economy prices by: the shop at `place` multiplies its prices by
/// `permille` while `happening` runs. Caused by the `happening-began`; public, at the shop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PricesAdjusted {
    place: PlaceId,
    happening: HappeningRef,
    permille: u16,
}

impl Event for PricesAdjusted {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("prices-adjusted");
    const OWNER: SystemId = EconomySystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl PricesAdjusted {
    pub(crate) const fn new(place: PlaceId, happening: HappeningRef, permille: u16) -> Self {
        Self {
            place,
            happening,
            permille,
        }
    }

    /// The shop's place.
    pub const fn place(&self) -> PlaceId {
        self.place
    }

    /// The happening it lasts for.
    pub const fn happening(&self) -> &HappeningRef {
        &self.happening
    }

    /// The multiplier, per mille.
    pub const fn permille(&self) -> u16 {
        self.permille
    }
}

/// The happening a shop's adjustment lasted for ended: its prices no longer carry it. Caused by the
/// `happening-ended`; public, at the shop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PricesRestored {
    place: PlaceId,
    happening: HappeningRef,
}

impl Event for PricesRestored {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("prices-restored");
    const OWNER: SystemId = EconomySystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl PricesRestored {
    pub(crate) const fn new(place: PlaceId, happening: HappeningRef) -> Self {
        Self { place, happening }
    }

    /// The shop's place.
    pub const fn place(&self) -> PlaceId {
        self.place
    }

    /// The happening that ended.
    pub const fn happening(&self) -> &HappeningRef {
        &self.happening
    }
}

/// `seller` changed its asking price for `item` from `from` to `to` (`ARC-112`): from `set-price`.
/// Reduced into the seller's [`Asks`](crate::component::Asks).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceSet {
    seller: EntityId,
    item: ItemId,
    from: u64,
    to: u64,
}

impl Event for PriceSet {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("price-set");
    const OWNER: SystemId = EconomySystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl PriceSet {
    pub(crate) const fn new(seller: EntityId, item: ItemId, from: u64, to: u64) -> Self {
        Self {
            seller,
            item,
            from,
            to,
        }
    }

    /// Who sells.
    pub const fn seller(&self) -> EntityId {
        self.seller
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// The asking price before.
    pub const fn from(&self) -> u64 {
        self.from
    }

    /// The asking price now.
    pub const fn to(&self) -> u64 {
        self.to
    }
}

/// `seller` no longer asks a price for `item` (`ARC-112`): from `withdraw`. Reduced into the seller's
/// [`Asks`](crate::component::Asks).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Withdrawn {
    seller: EntityId,
    item: ItemId,
}

impl Event for Withdrawn {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("withdrawn");
    const OWNER: SystemId = EconomySystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl Withdrawn {
    pub(crate) const fn new(seller: EntityId, item: ItemId) -> Self {
        Self { seller, item }
    }

    /// Who sold.
    pub const fn seller(&self) -> EntityId {
        self.seller
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }
}

/// `seller` sold `count` of `item` to `buyer` at `price` each, in `place` (`ARC-112`): the first of a
/// sale's three facts, stated by the one settling owner beside its `money-transferred` and inventory's
/// `items-transferred` (§5.7). It reduces to nothing: the money and the goods move by the other two.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sold {
    seller: EntityId,
    buyer: EntityId,
    item: ItemId,
    count: u32,
    price: u64,
    place: PlaceId,
}

impl Event for Sold {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("sold");
    const OWNER: SystemId = EconomySystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl Sold {
    pub(crate) const fn new(
        seller: EntityId,
        buyer: EntityId,
        item: ItemId,
        price: u64,
        place: PlaceId,
    ) -> Self {
        Self {
            seller,
            buyer,
            item,
            count: 1,
            price,
            place,
        }
    }

    /// Who sold and was paid.
    pub const fn seller(&self) -> EntityId {
        self.seller
    }

    /// Who bought and paid.
    pub const fn buyer(&self) -> EntityId {
        self.buyer
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// How many: one.
    pub const fn count(&self) -> u32 {
        self.count
    }

    /// The price of one, in minor units.
    pub const fn price(&self) -> u64 {
        self.price
    }

    /// Where the sale happened.
    pub const fn place(&self) -> PlaceId {
        self.place
    }
}

/// `buyer` proposes to buy one `item` from `seller` at `price` (`ARC-118`): from a shop's operator
/// (with the shop's `place`) or from a person seller (no place), open until `valid_until`. Heard by
/// the buyer and whoever may answer it, nobody else. Reduced into the seller's
/// [`Proposals`](crate::proposal::Proposals).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceProposed {
    proposal: ProposalId,
    seller: EntityId,
    buyer: PersonId,
    item: ItemId,
    price: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    place: Option<PlaceId>,
    valid_until: WorldTime,
}

impl Event for PriceProposed {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("price-proposed");
    const OWNER: SystemId = EconomySystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl PriceProposed {
    pub(crate) const fn new(
        terms: ProposalTerms,
        place: Option<PlaceId>,
        valid_until: WorldTime,
    ) -> Self {
        Self {
            proposal: terms.proposal,
            seller: terms.seller,
            buyer: terms.buyer,
            item: terms.item,
            price: terms.price,
            place,
            valid_until,
        }
    }

    /// The proposal.
    pub const fn proposal(&self) -> ProposalId {
        self.proposal
    }

    /// Who would sell: a Person, or the Organization running the shop.
    pub const fn seller(&self) -> EntityId {
        self.seller
    }

    /// Who proposes.
    pub const fn buyer(&self) -> PersonId {
        self.buyer
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// The proposed price of one, in minor units.
    pub const fn price(&self) -> u64 {
        self.price
    }

    /// The shop's place, for a shop's proposal.
    pub const fn place(&self) -> Option<PlaceId> {
        self.place
    }

    /// The first instant at which the proposal is dead.
    pub const fn valid_until(&self) -> WorldTime {
        self.valid_until
    }
}

/// The terms an answer or a withdrawal names (SD-S31h-6): which proposal, who would sell, who
/// proposed, the kind and the price — so the reduction finds the seller directly and a reader can say
/// who answered what.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalTerms {
    proposal: ProposalId,
    seller: EntityId,
    buyer: PersonId,
    item: ItemId,
    price: u64,
}

impl ProposalTerms {
    pub(crate) const fn new(
        proposal: ProposalId,
        seller: EntityId,
        buyer: PersonId,
        item: ItemId,
        price: u64,
    ) -> Self {
        Self {
            proposal,
            seller,
            buyer,
            item,
            price,
        }
    }

    /// The proposal.
    pub const fn proposal(&self) -> ProposalId {
        self.proposal
    }

    /// Who would sell.
    pub const fn seller(&self) -> EntityId {
        self.seller
    }

    /// Who proposed.
    pub const fn buyer(&self) -> PersonId {
        self.buyer
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// The proposed price, in minor units.
    pub const fn price(&self) -> u64 {
        self.price
    }
}

/// Declares one of the three facts that close or decline a proposal: the same terms, its own type.
macro_rules! proposal_fact {
    ($(#[$doc:meta])* $name:ident, $event_type:literal) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name {
            terms: ProposalTerms,
        }

        impl Event for $name {
            const EVENT_TYPE: EventTypeId = EventTypeId::from_static($event_type);
            const OWNER: SystemId = EconomySystem::ID;
            const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
        }

        impl $name {
            pub(crate) const fn new(terms: ProposalTerms) -> Self {
                Self { terms }
            }

            /// The proposal's terms.
            pub const fn terms(&self) -> &ProposalTerms {
                &self.terms
            }
        }
    };
}

proposal_fact!(
    /// The seller, or a keeper of the proposal's shop, declined it (`ARC-118`): it stays, marked
    /// declined, until it expires, so the buyer sees the "no" (Q-S31h-6).
    ProposalDeclined,
    "proposal-declined"
);

proposal_fact!(
    /// The buyer took its proposal back (`ARC-118`): it is removed.
    ProposalWithdrawn,
    "proposal-withdrawn"
);

proposal_fact!(
    /// The seller, or a keeper of the proposal's shop, accepted it (`ARC-118`): it is removed, and the
    /// same dispatch states the sale at the proposed price — `sold`, `money-transferred` and
    /// `items-transferred`.
    ProposalAccepted,
    "proposal-accepted"
);
