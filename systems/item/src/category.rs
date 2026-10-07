//! What sort of thing a kind is: `drink`, `food`, `goods`.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The longest category, in bytes.
pub const CATEGORY_MAX_BYTES: usize = 32;

/// What a category may not be.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error(
    "a category must be 1 to {max} bytes of a-z, 0-9 and '-', not beginning or ending with '-', \
     but got {category:?}"
)]
pub struct InvalidCategory {
    /// What was offered.
    pub category: String,
    /// The longest category this pack records.
    pub max: usize,
}

/// The sort of thing an item kind is, as a slug.
///
/// The smallest real attribute a kind has (step-10 QS-32): enough for a pack or a client to group
/// kinds, and free to ignore. A slug rather than free text because it is a value packs compare, not
/// something a person reads; no leading or trailing `-`, so two spellings of one category cannot
/// differ by a stray dash.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Category(String);

impl Category {
    /// A category, refused unless it keeps the rule above.
    ///
    /// # Errors
    ///
    /// [`InvalidCategory`] naming what was offered.
    pub fn new(category: impl Into<String>) -> Result<Self, InvalidCategory> {
        let category = category.into();
        let valid = !category.is_empty()
            && category.len() <= CATEGORY_MAX_BYTES
            && category
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            && !category.starts_with('-')
            && !category.ends_with('-');
        if valid {
            Ok(Self(category))
        } else {
            Err(InvalidCategory {
                category,
                max: CATEGORY_MAX_BYTES,
            })
        }
    }

    /// The category.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Category {
    type Error = InvalidCategory;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Category> for String {
    fn from(value: Category) -> Self {
        value.0
    }
}

impl core::fmt::Display for Category {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}
