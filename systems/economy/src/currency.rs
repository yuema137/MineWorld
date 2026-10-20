//! The world's currency as state: one never-woken Process holding what `currency-declared` declared,
//! and the `currency` record economy discloses from it (`ARC-111`).
//!
//! A Process rather than a component because no entity exists to carry world configuration when it is
//! seeded — calendar's precedent (`ARC-67`). It has no expected end, so it is never woken, and it is
//! uninterruptible: nothing about a currency is anybody else's to stop.

use mineworld_contracts::{EntityId, EntityType, ItemId, ProcessTypeId};
use mineworld_kernel::{
    KernelError, Process, ProcessKind, SystemIdentity, WorldRead, owned_component,
};
use serde::{Deserialize, Serialize};

use crate::component::ReferenceValue;
use crate::configuration::{Currency, SymbolAt};
use crate::system::EconomySystem;

/// The kind of process the world's currency is, owned — as a type — by [`EconomySystem`] (`INV-7`).
pub struct CurrencyProcess;

impl ProcessKind for CurrencyProcess {
    const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("economy-currency");
    type Owner = EconomySystem;
}

/// The world's currency process, if a currency has been declared.
pub(crate) fn currency_process<'a>(world: &WorldRead<'a>) -> Option<&'a Process> {
    world.processes().find(|process| {
        *process.owner() == EconomySystem::ID
            && *process.process_type() == CurrencyProcess::PROCESS_TYPE
    })
}

/// The declared currency, read back from its process.
pub(crate) fn declared(process: &Process) -> Result<Currency, KernelError> {
    let bytes = process.state_for::<CurrencyProcess>()?;
    serde_json::from_slice(bytes).map_err(|_| KernelError::ProcessNotRunning {
        process: process.id(),
    })
}

/// One kind's reference value, as the `currency` record lists it (`ARC-113`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceLine {
    item: ItemId,
    value: u64,
}

impl ReferenceLine {
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

/// `currency`: the world's currency as an observer is told it — its name, symbol, exponent and symbol
/// placement, and every item kind's reference value, in item order — so that every client and agent
/// formats amounts from the world, never from a constant.
///
/// A declared component type so that perception keeps it (`owned_by_an_enabled_system`); no entity
/// carries it. It is disclosed about the observer itself, to every living Person.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrencyRecord {
    name: String,
    symbol: String,
    exponent: u8,
    symbol_at: SymbolAt,
    references: Vec<ReferenceLine>,
}

owned_component! {
    component = CurrencyRecord,
    owner = EconomySystem,
    component_type = "currency",
    schema_version = 1,
}

impl CurrencyRecord {
    pub(crate) fn of(currency: &Currency, references: Vec<ReferenceLine>) -> Self {
        Self {
            name: currency.name().to_owned(),
            symbol: currency.symbol().to_owned(),
            exponent: currency.exponent(),
            symbol_at: currency.symbol_at(),
            references,
        }
    }

    /// The currency's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Its symbol.
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// How many digits of an amount are minor units.
    pub const fn exponent(&self) -> u8 {
        self.exponent
    }

    /// Which side of an amount the symbol is written on.
    pub const fn symbol_at(&self) -> SymbolAt {
        self.symbol_at
    }

    /// Every kind's reference value, in item order.
    pub fn references(&self) -> &[ReferenceLine] {
        &self.references
    }
}

/// The `currency` record as it stands, if a currency is declared: its references are every
/// `ReferenceValue` in the world, in item order.
pub(crate) fn record(world: &WorldRead<'_>) -> Option<CurrencyRecord> {
    let currency = declared(currency_process(world)?).ok()?;
    Some(CurrencyRecord::of(&currency, references(world)))
}

/// Every item kind's reference value, in item order (`ARC-113`).
fn references(world: &WorldRead<'_>) -> Vec<ReferenceLine> {
    world
        .components::<ReferenceValue>()
        .filter_map(|(entity, value)| {
            let item = ItemId::new(entity, EntityType::Item).ok()?;
            Some(ReferenceLine::new(item, value.value()))
        })
        .collect()
}

/// Whether `observer` is told the currency about itself: a living Person.
pub(crate) fn tells(world: &WorldRead<'_>, observer: EntityId, subject: EntityId) -> bool {
    observer == subject && crate::offer::living_person(world, observer)
}
