//! The facts this pack records: both at genesis only, both reduced by it alone. Neither has a public
//! constructor: only this pack states them.

use mineworld_contracts::{Event, EventSchemaVersion, EventTypeId, ItemId, PersonId, SystemId};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::system::NeedsSystem;

/// Two needs' levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Levels {
    /// Hunger, 0 … 1 000.
    pub hunger: u16,
    /// Tiredness, 0 … 1 000.
    pub tiredness: u16,
}

/// `person` has needs, starting at `levels`: the person form of the `needs:` section, at genesis.
/// Perceived by the person alone; never biographical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeedsAssigned {
    person: PersonId,
    levels: Levels,
}

impl Event for NeedsAssigned {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("needs-assigned");
    const OWNER: SystemId = NeedsSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl NeedsAssigned {
    pub(crate) const fn new(person: PersonId, levels: Levels) -> Self {
        Self { person, levels }
    }

    /// Whose.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// The starting levels.
    pub const fn levels(&self) -> Levels {
        self.levels
    }
}

/// One of the item kind `item` lowers hunger by `hunger`: the item form of the `needs:` section, at
/// genesis (`ARC-285`). Nobody's perception (`SystemInternal`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NourishmentAssigned {
    item: ItemId,
    hunger: u16,
}

impl Event for NourishmentAssigned {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("nourishment-assigned");
    const OWNER: SystemId = NeedsSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl NourishmentAssigned {
    pub(crate) const fn new(item: ItemId, hunger: u16) -> Self {
        Self { item, hunger }
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// Hunger one lowers by.
    pub const fn hunger(&self) -> u16 {
        self.hunger
    }
}
