//! The world's weather, hour by hour: the same for every observer, reproducible from the world's facts,
//! and changeable by content.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-68`; design
//! `.structured-coding/plans/mvp0/step-19-time-weather.md` §6 and §17). It depends on `calendar`, never
//! the reverse (`INV-TW-10`): each local midnight it reads the date and the day's light events from
//! calendar's `day-began` and draws that day's weather. It reads nothing of the host — no time scale, no
//! pause (`INV-TW-2`) — and nothing of any other pack's state but where an observer is, for disclosure.
//!
//! ```text
//! depends on  presence, calendar        weather without calendar is refused at assembly
//! configure   configure/weather.yaml    source: rules (TW-b admits only `rules`); seed: u64; rules:
//!                                       twelve months of integer climate (SD-TW-b-3, step-19 §17.4)
//! emits       weather-configured        genesis, from the configuration: { seed, rules, record: None };
//!                                       SystemInternal, no subjects (SD-TW-b-4)
//!             weather-day               reacting to calendar's day-began: the day's summary (TMAX,
//!                                       TMIN, PRCP, wind, fog, thunder, overcast morning), the
//!                                       generator's carry, and its 24 hours; SystemInternal
//!             weather-changed           at each hour whose condition differs from the one in force:
//!                                       { hour, condition, cloud_oktas, precipitation_tenth_mm };
//!                                       Public, no subjects (SD-TW-b-9)
//! subscribes  day-began, and its own three facts
//! owns        one `climate` Process     no place, no participants, uninterruptible; woken at each
//!                                       condition change; its state { configured, today, now, chain }
//!                                       is the fold of the facts above (SD-TW-b-6, -10)
//! discloses   weather-today             the day's summary and its 24 hours  ┐ about the observer's
//!             weather-now               { hour, condition }                 ┘ place only (SD-TW-b-11)
//! ```
//!
//! **The generator** (SD-TW-b-5, WGEN-lite after Richardson 1981 and Richardson & Wright 1984): per world
//! day, wet or dry by a two-state Markov chain with monthly per-mille probabilities; a wet day's amount
//! from five quintile amounts; TMAX and TMIN as the month's means plus an integer AR(1) anomaly; fog,
//! thunder and morning overcast by monthly per-mille chances; the month's wind. Each draw is
//! counter-based SplitMix64 keyed by `(seed, day, draw index)` (SD-TW-b-7), so no draw depends on how
//! many came before.
//!
//! **The hours** (SD-TW-b-8): temperature on a fixed diurnal curve from TMIN at sunrise to TMAX at 15:00
//! (Parton & Logan 1981); wet hours as one or two seeded runs whose length grows with the amount; the
//! condition from the hourly rate (AMS Glossary drizzle and heavy-rain thresholds), thunder, fog and the
//! cloud's oktas (WMO).
//!
//! **Integers only** (`INV-TW-5`): 0.1 °C, 0.1 mm, 0.1 m/s, degrees, per mille, oktas. No floating-point
//! type appears in this crate's source, and `tests/no_float.rs` scans for one.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod component;
pub mod configuration;
pub mod day;
mod draw;
pub mod event;
#[cfg(test)]
mod fixture;
pub mod generate;
pub mod hours;
pub mod process;
pub mod rules;
pub mod system;

pub use component::{WeatherNow, WeatherToday};
pub use configuration::WeatherConfiguration;
pub use day::{Chain, Condition, DailyWeather, HOURS, Origin, WeatherDay, WeatherHour};
pub use event::{RecordRef, WeatherChanged, WeatherConfigured};
pub use process::{ClimateProcess, ClimateState};
pub use rules::{Month, Rules};
pub use system::WeatherSystem;
