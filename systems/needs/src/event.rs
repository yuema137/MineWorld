//! The facts this pack records: both at genesis only, both reduced by it alone. Neither has a public
//! constructor: only this pack states them. Schema 2 (`ARC-370`).

use mineworld_contracts::{Event, EventSchemaVersion, EventTypeId, ItemId, PersonId, SystemId};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::component::Nourishment;
use crate::system::NeedsSystem;

/// A person's hunger at genesis, in the form their file wrote it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StartingHunger {
    /// A unitless deficit, 0 … 1 000.
    Unitless {
        /// The deficit.
        hunger: u16,
    },
    /// A kcal reserve, 0 … the person's daily use (checked when genesis is reduced).
    FoodEnergy {
        /// Whole kcal.
        initial_kcal: u32,
    },
}

/// `person` has needs, starting at `hunger` and `tiredness`: the person form of the `needs:` section,
/// at genesis. Perceived by the person alone; never biographical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeedsAssigned {
    person: PersonId,
    hunger: StartingHunger,
    tiredness: u16,
}

impl Event for NeedsAssigned {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("needs-assigned");
    const OWNER: SystemId = NeedsSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(2);
}

impl NeedsAssigned {
    pub(crate) const fn new(person: PersonId, hunger: StartingHunger, tiredness: u16) -> Self {
        Self {
            person,
            hunger,
            tiredness,
        }
    }

    /// Whose.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// The starting hunger.
    pub const fn hunger(&self) -> StartingHunger {
        self.hunger
    }

    /// The starting tiredness, 0 … 1 000.
    pub const fn tiredness(&self) -> u16 {
        self.tiredness
    }
}

/// One of the item kind `item` nourishes as `nourishment` says: the item form of the `needs:` section,
/// at genesis (`ARC-285`). Nobody's perception (`SystemInternal`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NourishmentAssigned {
    item: ItemId,
    nourishment: Nourishment,
}

impl Event for NourishmentAssigned {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("nourishment-assigned");
    const OWNER: SystemId = NeedsSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(2);
}

impl NourishmentAssigned {
    pub(crate) const fn new(item: ItemId, nourishment: Nourishment) -> Self {
        Self { item, nourishment }
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// What one does.
    pub const fn nourishment(&self) -> &Nourishment {
        &self.nourishment
    }
}
