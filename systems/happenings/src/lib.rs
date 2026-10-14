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
//!             happening-announced         the daily pass admitted a draft; Public, about its place
//!             happening-began             at its start; Public or its place's, per the kind's audience
//!             happening-ended             at its end; as began
//!             happening-withheld          the daily pass refused a draft, and why; SystemInternal
//! subscribes  calendar's day-began, and its own five facts
//! owns        one `almanac` Process        the configuration and the open happenings, folded from facts
//!             one `happening` Process      per announced happening, at its place: announced → running
//! discloses   happenings-here             running at the observer's place   ┐ about the observer's
//!             happenings-announced        announced, not begun, anywhere    ┘ place only
//! ```
//!
//! **The daily pass** (design §5.3): at each `day-began`, every source drafts for its target days —
//! `calendar:` entries from the date, `random:` entries from a counter-based draw over (seed, day, the
//! entry's key) — and the host admits or withholds each draft in a fixed order against the world's
//! limits. Same configuration, same history: no random state is kept anywhere.
//!
//! **Integers only** (INV-WE-7): minutes, days, per mille. No floating-point type appears in this
//! crate's source, and `tests/no_float.rs` scans for one.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod component;
pub mod configuration;
mod draw;
pub mod entry;
pub mod event;
pub mod generate;
pub mod host;
pub mod kind;
mod pass;
pub mod process;
pub mod system;

pub use component::{HappeningsAnnounced, HappeningsHere, ShownHappening};
pub use configuration::{HappeningsConfiguration, Limits};
pub use entry::{CalendarEntry, MonthSet, RandomEntry, Recurrence, WeekdaySet};
pub use event::{
    HappeningAnnounced, HappeningBegan, HappeningEnded, HappeningWithheld, HappeningsConfigured,
    KeyedRandom,
};
pub use generate::{DayIndex, DraftAsk, DraftSource, HappeningDraft, HappeningGenerator};
pub use host::{HappeningRef, MAX_DRAFTS_PER_GENERATOR, OpenHappening, WithheldReason};
pub use kind::{Audience, HappeningKind, HappeningKindId, Hours, KindCatalog, Title};
pub use process::{AlmanacProcess, AlmanacState, HappeningPhase, HappeningProcess, HappeningState};
pub use system::HappeningsSystem;
