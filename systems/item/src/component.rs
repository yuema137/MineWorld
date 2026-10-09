//! The state this pack owns — what an item kind is — and the catalogue it discloses on a place.

use mineworld_contracts::ItemId;
use mineworld_kernel::{WorldRead, owned_component};
use serde::{Deserialize, Serialize};

use crate::category::Category;
use crate::name::ItemName;
use crate::system::ItemSystem;

/// What an item kind is, on the Item entity that is the kind (`ARC-36`).
///
/// Its presence is what "a declared kind" means: another pack asks [`is_declared`](crate::is_declared)
/// rather than reading it. Schema 2 added `name` (step-13 R-PK-2; `ARC-37` note).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemKind {
    category: Category,
    name: ItemName,
}

impl ItemKind {
    /// A kind of this category, called `name`.
    pub const fn new(category: Category, name: ItemName) -> Self {
        Self { category, name }
    }

    /// The sort of thing it is.
    pub const fn category(&self) -> &Category {
        &self.category
    }

    /// What people call it.
    pub const fn name(&self) -> &ItemName {
        &self.name
    }
}

owned_component! {
    component = ItemKind,
    owner = ItemSystem,
    component_type = "item-kind",
    schema_version = 2,
}

/// One entry of the catalogue: a declared kind, its category and its name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Catalogued {
    item: ItemId,
    category: Category,
    name: ItemName,
}

impl Catalogued {
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
}

/// What a place discloses about the world's kinds, to whoever perceives it: every declared kind, in
/// `ItemId` order (step-11 SD-D10; `ARC-37` note).
///
/// A view, built from [`ItemKind`] when it is disclosed and **never stored** — economy's listing is
/// the precedent (`ARC-38`). It is a declared component type only because perception discloses a
/// record only when an enabled system owns its type (`mineworld_presence`'s `observe`, step-11
/// QD-12): this pack installs its table and never writes a row into it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemCatalogue {
    kinds: Vec<Catalogued>,
}

impl ItemCatalogue {
    /// The world's declared kinds, now; `None` when there are none, so a world with no kinds discloses
    /// nothing.
    pub fn of(world: &WorldRead<'_>) -> Option<Self> {
        let kinds: Vec<Catalogued> = world
            .components::<ItemKind>()
            .filter_map(|(entity, kind)| {
                let item = ItemId::new(entity, mineworld_contracts::EntityType::Item).ok()?;
                Some(Catalogued {
                    item,
                    category: kind.category.clone(),
                    name: kind.name.clone(),
                })
            })
            .collect();
        (!kinds.is_empty()).then_some(Self { kinds })
    }

    /// Every declared kind, in `ItemId` order.
    pub fn kinds(&self) -> &[Catalogued] {
        &self.kinds
    }
}

owned_component! {
    component = ItemCatalogue,
    owner = ItemSystem,
    component_type = "item-catalogue",
    schema_version = 1,
}
