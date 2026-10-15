//! What people call a kind: `Flat White`, `Sourdough Loaf`.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The longest item name, in bytes.
pub const ITEM_NAME_MAX_BYTES: usize = 64;

/// What an item name may not be.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error(
    "an item name must be 1 to {max} bytes, with no control characters and no space at either end, \
     but got {name:?}"
)]
pub struct InvalidItemName {
    /// What was offered.
    pub name: String,
    /// The longest name this pack records.
    pub max: usize,
}

/// A kind's display name, as a person reads it (step-13 R-PK-2, F-41; `ARC-37` note).
///
/// `naming`'s display-name rule, restated here because `item` depends on no pack: free text in any
/// script, 1–64 bytes, no control character (a name cannot break a line of a transcript), no
/// surrounding space (two spellings cannot differ invisibly). Display, never identity: two kinds may
/// share a name, and every pack still names a kind by its [`ItemId`](mineworld_contracts::ItemId).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ItemName(String);

impl ItemName {
    /// A name, refused unless it keeps the rule above.
    ///
    /// # Errors
    ///
    /// [`InvalidItemName`] naming what was offered.
    pub fn new(name: impl Into<String>) -> Result<Self, InvalidItemName> {
        let name = name.into();
        let valid = !name.is_empty()
            && name.len() <= ITEM_NAME_MAX_BYTES
            && !name.chars().any(char::is_control)
            && name.trim() == name;
        if valid {
            Ok(Self(name))
        } else {
            Err(InvalidItemName {
                name,
                max: ITEM_NAME_MAX_BYTES,
            })
        }
    }

    /// The name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ItemName {
    type Error = InvalidItemName;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<ItemName> for String {
    fn from(value: ItemName) -> Self {
        value.0
    }
}

impl core::fmt::Display for ItemName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}
