//! The facts this pack records, and reduces alone.
//!
//! None has a public constructor: this pack states all four. `money-transferred` is the only fact in
//! any pack that moves money.

use mineworld_contracts::{
    EntityId, Event, EventSchemaVersion, EventTypeId, OrganizationId, PersonId, PlaceId, SystemId,
};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::component::Price;
use crate::configuration::Currency;
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
