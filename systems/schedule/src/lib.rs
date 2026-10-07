//! A person's day: where they mean to be, and when — an agenda people may follow, never a mover.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-32`).
//!
//! ```text
//! section     routine              a person file's day, in segments { from, place, label }
//! emits       routine-assigned     at genesis, from the section
//!             agenda-changed       at each boundary, caused by the person's routine process
//! owns        Routine, Agenda      on the person; and one `routine` Process per person
//! discloses   Agenda               to its holder only
//! depends on  nothing              and states no other pack's facts — so it moves nobody
//! ```
//!
//! Following an agenda is a controller's choice: `mineworld run`'s paced controller walks there
//! through `move`; a human may ignore it; a person nobody drives stays where they are while their day
//! goes by. The time of day is world seconds since the epoch modulo 86 400 — this pack's convention,
//! not the kernel's (`INV-12`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;
mod section;

pub mod component;
pub mod error;
pub mod event;
pub mod label;
pub mod process;
pub mod segment;
pub mod system;
pub mod time;

pub use component::{Agenda, Routine};
pub use error::ScheduleError;
pub use event::{AgendaChanged, RoutineAssigned};
pub use label::{AgendaLabel, LABEL_MAX_BYTES};
pub use process::{RoutineProcess, RoutineState};
pub use segment::{AuthoredRoutine, InForce, MAX_SEGMENTS, MIN_SEGMENTS, Segment, Segments};
pub use system::{BIOGRAPHICAL, ScheduleSystem};
pub use time::{DAY, TimeOfDay};
