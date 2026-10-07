//! Eating and drinking what one carries.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-38`) that owns nothing. It decides whether a person may
//! eat or drink a held kind, and states the result in `inventory`'s vocabulary, through inventory's
//! checked constructor; inventory alone writes holdings, and still decides (`ARC-26`):
//!
//! ```text
//! provides    eat { item }       a held kind whose category is `food`
//!             drink { item }     a held kind whose category is `drink`
//! states      items-consumed     inventory's fact, through `mineworld_inventory::consume`
//! offers      one complete eat or drink per edible or drinkable kind held, no target (ARC-34)
//! owns        nothing
//! depends on  inventory          (and reads item's ItemKind)
//! ```
//!
//! There is no spatial requirement: a person eats what they carry, wherever they are (step-10 QS-40).
//! Goods are never consumed. This is the interaction, not a need: nobody is hungry (step-10 QS-10).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;
mod offer;

pub mod action;
pub mod system;

pub use action::{DRUNK, Drink, EATEN, Eat};
pub use system::ConsumptionSystem;
