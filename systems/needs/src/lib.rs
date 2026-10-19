//! Hunger and tiredness.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-38`, `ARC-284`, `ARC-285`). A person whose file carries a
//! `needs:` section has two integer deficits, hunger and tiredness, from 0 (sated, rested) to 1 000
//! (famished, exhausted). They rise with world time and fall when the person eats (inventory's
//! `items-consumed`) or sleeps (sleep's `fell-asleep` … `woke`). Nothing is written while they rise:
//! the pack stores a level, the instant it was true and whether the person was awake, and computes the
//! current level wherever it is read (`ARC-86`).
//!
//! ```text
//! section     needs:        a person form { hunger, tiredness } and an item form { nourishment }
//! configure   configure/needs.yaml (ARC-63): rates, thresholds, restore
//! emits       needs-assigned        genesis, from a person's form; to the person
//!             nourishment-assigned  genesis, from an item's form; SystemInternal
//! owns        Needs                 on a person
//!             Nourishment           on an item kind
//! process     none
//! depends on  nothing (registry)
//! hears       items-consumed (inventory), fell-asleep, woke (sleep), through their owners' types
//! discloses   needs                 to its holder only: levels, words, mode, rates, restore and the
//!                                   nourishment of each kind they carry
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;

pub mod component;
pub mod event;
pub mod level;
pub mod section;
pub mod system;

pub use component::{Needs, Nourishment};
pub use event::{Levels, NeedsAssigned, NourishmentAssigned};
pub use level::{HungerWord, Level, Mode, Need, Rates, TirednessWord};
pub use section::{AuthoredNeeds, SECTION_KIND};
pub use system::NeedsSystem;
