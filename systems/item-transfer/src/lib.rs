//! Giving things to people.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-37`) that owns nothing. It decides whether a give is
//! allowed and states the result in `inventory`'s vocabulary, through inventory's checked
//! constructor; inventory alone writes holdings, and still decides (`ARC-26`):
//!
//! ```text
//! provides    give { item, count }   to a Person in the same place, within 3 m, who can take it
//! states      items-transferred      inventory's fact, through `mineworld_inventory::transfer`
//! offers      one complete give per kind held, count 1, to each other person present (ARC-34)
//! owns        nothing
//! depends on  inventory, presence
//! ```
//!
//! Because each offer is **complete** — it carries the exact request — a controller that was never
//! compiled against this pack can give, and the paced controller does (`ARC-34`). Disabled, `give` is
//! answered `Unavailable`, is offered nowhere, and holdings never change (`INV-10`, `AC-2`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;
mod offer;

pub mod action;
pub mod system;

pub use action::{GIVE_RANGE, Give, give_requirement};
pub use system::ItemTransferSystem;
