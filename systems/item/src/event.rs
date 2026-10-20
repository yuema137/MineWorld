//! The fact this pack records: a kind exists, what sort of thing it is, and what people call it.

use mineworld_contracts::{Event, EventSchemaVersion, EventTypeId, ItemId, SystemId, Visibility};
use mineworld_kernel::{Emission, SystemIdentity};
use serde::{Deserialize, Serialize};

use crate::category::Category;
use crate::codec;
use crate::name::ItemName;
use crate::system::ItemSystem;

/// `item` is a kind of thing, of `category`, called `name`.
///
/// Stated at genesis from the item file's `item:` section and reduced by this pack alone into
/// [`ItemKind`](crate::ItemKind). Public: what sort of thing a kind is, and its name, are nobody's
/// secret. Schema 2 added `name` (step-13 R-PK-2; `ARC-37` note).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemKindDeclared {
    item: ItemId,
    category: Category,
    name: ItemName,
}

impl Event for ItemKindDeclared {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("item-kind-declared");
    const OWNER: SystemId = ItemSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(2);
}

impl ItemKindDeclared {
    /// States it.
    pub const fn new(item: ItemId, category: Category, name: ItemName) -> Self {
        Self {
            item,
            category,
            name,
        }
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// What sort of thing it is.
    pub const fn category(&self) -> &Category {
        &self.category
    }

    /// What people call it.
    pub const fn name(&self) -> &ItemName {
        &self.name
    }

    /// As a fact ready to record: public, about the kind.
    pub fn emission(&self) -> Emission {
        Emission::new::<Self>(codec::encode(self), Visibility::Public)
            .about(vec![self.item.entity_id()])
    }
}
