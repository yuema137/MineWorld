//! The state this pack owns: what an item kind is.

use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::category::Category;
use crate::system::ItemSystem;

/// What an item kind is, on the Item entity that is the kind (`ARC-36`).
///
/// Its presence is what "a declared kind" means: another pack asks [`is_declared`](crate::is_declared)
/// rather than reading it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemKind {
    category: Category,
}

impl ItemKind {
    /// A kind of this category.
    pub const fn new(category: Category) -> Self {
        Self { category }
    }

    /// The sort of thing it is.
    pub const fn category(&self) -> &Category {
        &self.category
    }
}

owned_component! {
    component = ItemKind,
    owner = ItemSystem,
    component_type = "item-kind",
    schema_version = 1,
}
