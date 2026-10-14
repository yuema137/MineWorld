//! Happenings: a Saturday market on the street, a festival in the park announced a week ahead, a street
//! musician one day in seven — generated from the world's calendar and a seeded integer draw, with no
//! language model anywhere.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-88`; design
//! `.structured-coding/plans/mvp0/pr-we-a-happenings.md`). It depends on `calendar`, never the reverse:
//! each local midnight it reads the date from calendar's `day-began`. It owns only its happenings: it
//! announces, begins and ends each one as a fact, and every other System that responds does so in its
//! own reduction, from those facts, under its own opt-in configuration.
//!
//! ```text
//! depends on  presence, calendar          happenings without calendar is refused at assembly
//! configure   configure/happenings.yaml   seed, kinds, calendar and random entries, limits, generators
//! emits       happenings-configured       genesis, the resolved configuration; SystemInternal
//! owns        one `almanac` Process        the configuration, folded from its own facts
//! ```
//!
//! **Integers only** (INV-WE-7): minutes, days, per mille. No floating-point type appears in this
//! crate's source, and `tests/no_float.rs` scans for one.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod configuration;
mod draw;
pub mod entry;
pub mod event;
pub mod kind;
pub mod process;
pub mod system;

pub use configuration::{HappeningsConfiguration, Limits};
pub use entry::{CalendarEntry, MonthSet, RandomEntry, Recurrence, WeekdaySet};
pub use event::{HappeningsConfigured, KeyedRandom};
pub use kind::{Audience, HappeningKind, HappeningKindId, Hours, KindCatalog, Title};
pub use process::{AlmanacProcess, AlmanacState};
pub use system::HappeningsSystem;
