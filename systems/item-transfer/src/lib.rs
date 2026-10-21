//! Giving things to people.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-37`) that owns nothing. It decides whether a give is
//! allowed and states the result in `inventory`'s vocabulary, through inventory's checked
//! constructor; inventory alone writes holdings, and still decides (`ARC-26`):
//!
//! ```text
//! provides    give { item, count }   to a Person in the same place, within 3 m, who can take it
//!             put-down { item }      one held item to the place I stand in (S35 N-h R4)
//!             pick-up { item }       one item from the place I stand in, when I can carry it
//! states      items-transferred      inventory's fact, through `mineworld_inventory::transfer`
//!                                    (`transfer_in_place` for a put-down or pick-up: heard there)
//! offers      one complete give per kind held, count 1, to each other person present (ARC-34);
//!             one complete put-down per kind held and one pick-up per kind the place holds
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

/// Putting a thing down and picking one up (S35 N-h R4).
pub mod hands;

pub mod action;
pub mod system;

pub use action::{GIVE_RANGE, Give, give_requirement};
pub use hands::{PickUp, PutDown};
pub use system::ItemTransferSystem;
