//! Where and when a world is: its civil date, its weekday and its sun.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-67`, `DEP-30`; design
//! `.structured-coding/plans/mvp0/step-19-time-weather.md` §5 and §16). The world's one clock,
//! `WorldTime`, is calendar time (`docs/CORE_CONCEPTS.md` §17); this pack gives its seconds a date and
//! a sun. It reads nothing of the host — no time scale, no pause — and nothing of any other pack's
//! state but where an observer is, for disclosure.
//!
//! ```text
//! configure   configure/calendar.yaml   epoch: YYYY-MM-DD (local date at instant 0, 1901 … 2099)
//!                                       utc_offset: ±HH:MM (fixed, ±14 h; no DST — QTW-9)
//!                                       latitude, longitude: decimal degrees (±90, ±180), kept as
//!                                       integer micro-degrees
//! emits       calendar-configured       genesis, from the configuration; SystemInternal, no subjects
//!             day-began                 each local midnight: date, weekday (0 = Monday), day_start,
//!                                       the phase at day_start, the day's light events, the sun every
//!                                       900 s (97 samples); Public
//!             daylight-changed          at each light event: the new phase (night,
//!                                       astronomical-twilight, civil-twilight, day); Public
//! owns        one `calendar` Process    no place, no participants, uninterruptible; woken at each
//!                                       light event and at midnight; its state is the fold of the
//!                                       three facts above, so a snapshot and a replay agree
//! discloses   calendar-day              the current day record   ┐ about the observer's place only;
//!             calendar-light            the current phase        ┘ an observer in no place gets none
//! ```
//!
//! **One midnight.** Instant 0 is local midnight of the epoch date and every multiple of 86 400 s is a
//! local midnight — `schedule`'s convention, restated here as this pack's own constant and pinned by a
//! test, so the pack does not depend on `schedule` (`INV-TW-4`).
//!
//! **Integers only.** Sun angles are millidegrees, times are whole world seconds. The only file that
//! evaluates floating point is `sun.rs`, and it rounds before anything leaves it (`INV-TW-5`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod civil;
pub mod component;
pub mod configuration;
pub mod day;
pub mod event;
pub mod process;
mod sun;
pub mod system;

pub use civil::CalendarDate;
pub use component::{CalendarDayRecord, CalendarLight};
pub use configuration::CalendarConfiguration;
pub use day::{
    CalendarDay, DAY, DayEvents, MicroDegrees, Phase, SAMPLE_INTERVAL, SAMPLES, SunSample,
};
pub use event::{CalendarConfigured, DayBegan, DaylightChanged};
pub use process::{CalendarProcess, CalendarState};
pub use sun::SunUnavailable;
pub use system::CalendarSystem;
