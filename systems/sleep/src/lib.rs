//! Sleeping, and getting up.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-38`, `ARC-283`). A person asks to sleep for a number of
//! minutes where they are; the world's Interaction List decides where that is allowed, and when it says
//! nothing, a place tagged `residential` is. While a person sleeps the pack holds [`Asleep`] and one
//! `sleep` Process that ends at the alarm.
//!
//! ```text
//! provides    sleep { for }      minutes, within the bounds in force (15 … 840 by default)
//!             wake  { }          get up
//! emits       fell-asleep        { person, place }: to the place; it carries no alarm
//!             alarm-set          { person, alarm }: to the sleeper alone
//!             woke               { person, slept, reason }: to the place the sleeper is in
//! owns        Asleep             { since, alarm, process } on a sleeping person
//!             SleepTerms         declared, never stored: the `sleep-terms` record, computed
//! process     sleep              one per sleeper, ending at the alarm
//! hears       arrived            presence's: a sleeper who walks wakes (`acted`)
//!             spoke              conversation's: the listener wakes (`addressed`), the speaker too
//!                                (`acted`) — decoded through its published type, no dependency
//! section     configure/sleep.yaml (ARC-63)
//! depends on  presence
//! ```
//!
//! A sleep ends in exactly four ways, each this pack's own reaction: the alarm, `wake`, being spoken
//! to, and acting. `movement` and `conversation` never learn that sleep exists.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;
mod offer;
mod place;

pub mod action;
pub mod component;
pub mod event;
pub mod interactions;
pub mod process;
pub mod recovery;
pub mod system;

pub use action::{Sleep, Wake};
pub use component::{Asleep, SleepTerms};
pub use event::{AlarmSet, FellAsleep, Woke, WokeReason};
pub use interactions::{SleepParameters, SleepPartial};
pub use place::SLEEPING_TAG;
pub use process::{SleepProcess, SleepState};
pub use system::{OUT_OF_BOUND, SleepSystem};
