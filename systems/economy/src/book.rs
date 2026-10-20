//! The price book: where a world's configured price modifiers live between genesis and the happenings
//! they answer (WE-a design §6.2). A Process, because the configuration is seeded before any shop opens,
//! so no shop exists to carry it then — the precedent of calendar's and weather's configuration, kept
//! in a Process their own facts fold. Started once, from `economy-configured`; never woken, never
//! rewritten. A world with no `configure/economy.yaml` has none, and economy hears every happening
//! without answering it.

use mineworld_contracts::ProcessTypeId;
use mineworld_kernel::ProcessKind;
use serde::{Deserialize, Serialize};

use crate::configuration::ResolvedModifier;
use crate::system::EconomySystem;

/// The kind of process the price book is, owned — as a type — by [`EconomySystem`] (`INV-7`).
pub struct PriceBookProcess;

impl ProcessKind for PriceBookProcess {
    const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("price-book");
    type Owner = EconomySystem;
}

/// The price book's state: the configured modifiers, in file order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceBookState {
    price_modifiers: Vec<ResolvedModifier>,
}

impl PriceBookState {
    pub(crate) const fn new(price_modifiers: Vec<ResolvedModifier>) -> Self {
        Self { price_modifiers }
    }

    /// The modifiers, in file order.
    pub fn price_modifiers(&self) -> &[ResolvedModifier] {
        &self.price_modifiers
    }
}
