//! Hunger and tiredness.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-38`, `ARC-284`, `ARC-285`, `ARC-370`). A person whose file
//! carries a `needs:` section has hunger and tiredness. Tiredness is an integer deficit from 0 (rested)
//! to 1 000 (exhausted). Hunger is, in the model the world chooses, the same kind of deficit
//! (`unitless`, the default) or a kcal reserve full at the person's daily use (`food-energy`). Both move
//! with world time and change when the person eats (inventory's `items-consumed`) or sleeps (sleep's
//! `fell-asleep` … `woke`). Nothing is written while they move: the pack stores a value and the instant
//! it was true, and computes the current value wherever it is read (`ARC-86`).
//!
//! ```text
//! section     needs:        a person form and an item form, each unitless or food-energy
//! configure   configure/needs.yaml (ARC-63): hunger model, daily use, rates, thresholds, restore
//! emits       needs-assigned        genesis, from a person's form; to the person
//!             nourishment-assigned  genesis, from an item's form; SystemInternal
//! owns        Needs                 on a person
//!             Nourishment           on an item kind
//! process     none
//! depends on  nothing (registry)
//! hears       items-consumed (inventory), fell-asleep, woke (sleep), through their owners' types
//! discloses   needs                 to its holder only: levels, words, mode, rates, restore, the
//!                                   nourishment of each kind they carry and, for a reserve, its kcal
//! refuses     a form of the other model, or a reserve above daily use, when genesis is reduced
//!             (InvalidGenesisContent)
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;
mod disclose;

pub mod component;
pub mod event;
pub mod food_energy;
pub mod interactions;
pub mod level;
pub mod section;
pub mod system;

pub use component::{Hunger, Needs, Nourishment};
pub use event::{NeedsAssigned, NourishmentAssigned, StartingHunger};
pub use food_energy::{Quantity, QuantityUnit, Reserve, Serving};
pub use level::{HungerWord, Level, Mode, Need, Rates, TirednessWord};
pub use section::{AuthoredNeeds, SECTION_KIND};
pub use system::{INITIAL_ENERGY_OUT_OF_RANGE, MODEL_MISMATCH, NeedsSystem};
