//! The facts this pack records, and reduces alone.
//!
//! None has a public constructor: this pack states all seven. `money-transferred` is the only fact in
//! any pack that moves money; `prices-adjusted` and `prices-restored` are the only facts that change a
//! shop's prices, each caused by the happening fact it answers (WE-a design §6.2).

use mineworld_contracts::{
    EntityId, Event, EventSchemaVersion, EventTypeId, OrganizationId, PersonId, PlaceId, SystemId,
};
use mineworld_happenings::HappeningRef;
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::component::Price;
use crate::configuration::ResolvedModifier;
use crate::system::EconomySystem;

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

/// A shop opened in `place`, run by `operator`, selling at `prices`: a genesis fact from the
/// operator's `economy:` section. Public: a shop is there for anyone to see.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShopOpened {
    place: PlaceId,
    operator: OrganizationId,
    prices: Vec<Price>,
}

impl Event for ShopOpened {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("shop-opened");
    const OWNER: SystemId = EconomySystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl ShopOpened {
    pub(crate) const fn new(place: PlaceId, operator: OrganizationId, prices: Vec<Price>) -> Self {
        Self {
            place,
            operator,
            prices,
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

    /// What it sells, in item order.
    pub fn prices(&self) -> &[Price] {
        &self.prices
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
