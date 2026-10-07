//! The fact this pack records: a kind exists, and what sort of thing it is.

use mineworld_contracts::{Event, EventSchemaVersion, EventTypeId, ItemId, SystemId, Visibility};
use mineworld_kernel::{Emission, SystemIdentity};
use serde::{Deserialize, Serialize};

use crate::category::Category;
use crate::codec;
use crate::system::ItemSystem;

/// `item` is a kind of thing, of `category`.
///
/// Stated at genesis from the item file's `item:` section and reduced by this pack alone into
/// [`ItemKind`](crate::ItemKind). Public: what sort of thing a kind is, is nobody's secret.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemKindDeclared {
    item: ItemId,
    category: Category,
}

impl Event for ItemKindDeclared {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("item-kind-declared");
    const OWNER: SystemId = ItemSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl ItemKindDeclared {
    /// States it.
    pub const fn new(item: ItemId, category: Category) -> Self {
        Self { item, category }
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// What sort of thing it is.
    pub const fn category(&self) -> &Category {
        &self.category
    }

    /// As a fact ready to record: public, about the kind.
    pub fn emission(&self) -> Emission {
        Emission::new::<Self>(codec::encode(self), Visibility::Public)
            .about(vec![self.item.entity_id()])
    }
}
