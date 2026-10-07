//! The facts this pack records, and reduces alone: a holder begins with some, some pass between two
//! holders, some come into being, and some are used up.
//!
//! None has a public unchecked constructor. `stocked` is built only by this pack's section seed;
//! `items-transferred` only by [`transfer`](crate::transfer), which asks
//! [`admit_transfer`](crate::admit_transfer) first (`ARC-26`); `items-produced` only by
//! [`produce`](crate::produce) and `items-consumed` only by [`consume`](crate::consume), each asking
//! its own admit function first (`ARC-38`).

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

/// `count` of `item` came into being in `holder`'s holdings.
///
/// Stated by whichever pack decided it — `employment`, when a shift ends — through
/// [`produce`](crate::produce). The fact says what happened to holdings, not why: its cause is the
/// stater's (`ARC-38` item 1). Visible to the holder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemsProduced {
    holder: EntityId,
    item: ItemId,
    count: u32,
}

impl Event for ItemsProduced {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("items-produced");
    const OWNER: SystemId = InventorySystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl ItemsProduced {
    pub(crate) const fn new(holder: EntityId, item: ItemId, count: u32) -> Self {
        Self {
            holder,
            item,
            count,
        }
    }

    /// Whose holdings they came into.
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

/// `count` of `item` left `holder`'s holdings by being used up.
///
/// Stated by whichever pack decided it — `consumption`, when somebody eats or drinks — through
/// [`consume`](crate::consume). Visible to the holder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemsConsumed {
    holder: EntityId,
    item: ItemId,
    count: u32,
}

impl Event for ItemsConsumed {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("items-consumed");
    const OWNER: SystemId = InventorySystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl ItemsConsumed {
    pub(crate) const fn new(holder: EntityId, item: ItemId, count: u32) -> Self {
        Self {
            holder,
            item,
            count,
        }
    }

    /// Whose holdings they left.
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
