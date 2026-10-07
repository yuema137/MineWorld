//! The state this pack owns: how many of each kind a holder holds.

use mineworld_contracts::ItemId;
use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::system::InventorySystem;

/// How many of one kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Held {
    item: ItemId,
    count: u32,
}

impl Held {
    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// How many: never zero.
    pub const fn count(&self) -> u32 {
        self.count
    }
}

/// What a Person or an Organization holds: a count per kind, in item order, never a zero entry.
///
/// A sorted list rather than a map keyed by item, because an [`ItemId`] encodes as `{ entity, type }`
/// and a JSON object key must be a string — and snapshots, observations and payloads are all JSON
/// (`DEP-5`; step-10 F-38). Written only by this pack's reductions; built only through
/// [`Holdings::adding`] and [`Holdings::removing`], which keep the order and drop a count that
/// reaches zero.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Holdings {
    held: Vec<Held>,
}

impl Holdings {
    /// Every kind held, in item order.
    pub fn held(&self) -> &[Held] {
        &self.held
    }

    /// How many of `item` — zero when none.
    pub fn count(&self, item: ItemId) -> u32 {
        self.held
            .binary_search_by_key(&item, |held| held.item)
            .map_or(0, |at| self.held[at].count)
    }

    /// How many items in all, every kind together.
    pub fn total(&self) -> u64 {
        self.held.iter().map(|held| u64::from(held.count)).sum()
    }

    /// These holdings with `count` more of `item`; [`None`] if the count would overflow.
    pub(crate) fn adding(&self, item: ItemId, count: u32) -> Option<Self> {
        let mut next = self.clone();
        match next.held.binary_search_by_key(&item, |held| held.item) {
            Ok(at) => next.held[at].count = next.held[at].count.checked_add(count)?,
            Err(at) => {
                if count == 0 {
                    return None;
                }
                next.held.insert(at, Held { item, count });
            }
        }
        Some(next)
    }

    /// These holdings with `count` fewer of `item`; [`None`] if fewer are held.
    pub(crate) fn removing(&self, item: ItemId, count: u32) -> Option<Self> {
        let mut next = self.clone();
        let at = next
            .held
            .binary_search_by_key(&item, |held| held.item)
            .ok()?;
        let left = next.held[at].count.checked_sub(count)?;
        if left == 0 {
            next.held.remove(at);
        } else {
            next.held[at].count = left;
        }
        Some(next)
    }
}

owned_component! {
    component = Holdings,
    owner = InventorySystem,
    component_type = "holdings",
    schema_version = 1,
}
