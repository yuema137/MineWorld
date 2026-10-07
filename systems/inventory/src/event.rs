//! The facts this pack records, and reduces alone: a holder begins with some, and some pass between
//! two holders.
//!
//! Neither has a public unchecked constructor. `stocked` is built only by this pack's section seed;
//! `items-transferred` only by [`transfer`](crate::transfer), which asks
//! [`admit_transfer`](crate::admit_transfer) first (`ARC-26`).

use mineworld_contracts::{EntityId, Event, EventSchemaVersion, EventTypeId, ItemId, SystemId};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::system::InventorySystem;

/// `holder` begins with `count` of `item`: a genesis fact, from the holder's `holdings:` section.
///
/// Visible to the holder only: what somebody carries is theirs to know (`INV-13`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stocked {
    holder: EntityId,
    item: ItemId,
    count: u32,
}

impl Event for Stocked {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("stocked");
    const OWNER: SystemId = InventorySystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl Stocked {
    pub(crate) const fn new(holder: EntityId, item: ItemId, count: u32) -> Self {
        Self {
            holder,
            item,
            count,
        }
    }

    /// Who holds them.
    pub const fn holder(&self) -> EntityId {
        self.holder
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// How many.
    pub const fn count(&self) -> u32 {
        self.count
    }
}

/// `count` of `item` passed from `from` to `to`.
///
/// Stated by whichever pack decided it — a give, and later a purchase — through
/// [`transfer`](crate::transfer). Visible to its two participants: a bystander is not told what
/// changed hands (step-10 QS-30).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemsTransferred {
    from: EntityId,
    to: EntityId,
    item: ItemId,
    count: u32,
}

impl Event for ItemsTransferred {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("items-transferred");
    const OWNER: SystemId = InventorySystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl ItemsTransferred {
    pub(crate) const fn new(from: EntityId, to: EntityId, item: ItemId, count: u32) -> Self {
        Self {
            from,
            to,
            item,
            count,
        }
    }

    /// Who gave them up.
    pub const fn from(&self) -> EntityId {
        self.from
    }

    /// Who received them.
    pub const fn to(&self) -> EntityId {
        self.to
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// How many.
    pub const fn count(&self) -> u32 {
        self.count
    }
}
