//! What kinds of things exist.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-37`). An authored Item is a kind (`ARC-36`): the file
//! `items/coffee.yaml` *is* the kind `coffee`. This pack owns what that kind is, so that the packs that
//! trade, give, sell or produce it share one vocabulary and none of them owns another's:
//!
//! ```text
//! section     item                 an item file's `item: { category: drink, name: Coffee }`, validated
//!                                  by [`Category`] and [`ItemName`]
//! emits       item-kind-declared   at genesis, from the section; public
//! owns        ItemKind             `item-kind`, reduced from `item-kind-declared` by this pack alone
//! discloses   ItemCatalogue        on a place, to whoever perceives it: every declared kind, its
//!                                  category and name, built at disclosure (`ARC-37` note); items are
//!                                  never perceived as entities
//! depends on  nothing
//! ```
//!
//! [`is_declared`] is the question the other packs ask. An Item entity whose file has no `item:`
//! section is inert: it exists, and no pack trades it.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;
mod section;

pub mod category;
pub mod component;
pub mod event;
pub mod name;
pub mod system;

pub use category::{CATEGORY_MAX_BYTES, Category, InvalidCategory};
pub use component::{Catalogued, ItemCatalogue, ItemKind};
pub use event::ItemKindDeclared;
pub use name::{ITEM_NAME_MAX_BYTES, InvalidItemName, ItemName};
pub use section::AuthoredItem;
pub use system::{ItemSystem, is_declared};
