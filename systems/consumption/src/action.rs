//! The actions this pack provides: eat and drink.

use mineworld_contracts::{Action, ActionTypeId, ItemId, SystemId};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::system::ConsumptionSystem;

/// The item category that is eaten.
pub const EATEN: &str = "food";

/// The item category that is drunk.
pub const DRUNK: &str = "drink";

/// Eat one `item` the eater carries; its kind's category must be [`EATEN`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Eat {
    item: ItemId,
}

impl Action for Eat {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("eat");
    const OWNER: SystemId = ConsumptionSystem::ID;
}

impl Eat {
    /// Eat one `item`.
    pub const fn new(item: ItemId) -> Self {
        Self { item }
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }
}

/// Drink one `item` the drinker carries; its kind's category must be [`DRUNK`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Drink {
    item: ItemId,
}

impl Action for Drink {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("drink");
    const OWNER: SystemId = ConsumptionSystem::ID;
}

impl Drink {
    /// Drink one `item`.
    pub const fn new(item: ItemId) -> Self {
        Self { item }
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }
}
