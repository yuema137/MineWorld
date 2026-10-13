//! The world's climate as a Process: started once, from the configuration, woken at each hour the
//! condition changes, never ended (SD-TW-b-6, -10).
//!
//! Its state is the fold of this pack's three facts — the configuration, today's weather and the hour
//! of the condition in force — so a snapshot and a replay of the log agree, and a resumed world
//! continues the same chain without replaying from day 0. Every snapshot carries it whole, so its
//! encoded size has a pinned budget of 8 KB (F-TWbd-1; `tests/weather.rs`).

use mineworld_contracts::ProcessTypeId;
use mineworld_kernel::ProcessKind;
use serde::{Deserialize, Serialize};

use crate::day::{Chain, Condition, WeatherDay};
use crate::event::WeatherConfigured;
use crate::record::RecordSeries;
use crate::system::WeatherSystem;

/// The kind of process the world's climate is, owned — as a type — by [`WeatherSystem`], so no other
/// system can reschedule or end it (`INV-7`).
pub struct ClimateProcess;

impl ProcessKind for ClimateProcess {
    const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("climate");
    type Owner = WeatherSystem;
}

/// The kind of process a replayed record is: started once, from `weather-configured`, never woken and
/// never rewritten; its state is the series that fact carries (SD-TW-d-5). It is a Process of its own so
/// the climate state, which is rewritten several times a day, stays small, and the 66 KB series is
/// encoded once.
pub struct RecordProcess;

impl ProcessKind for RecordProcess {
    const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("weather-record");
    type Owner = WeatherSystem;
}

/// The record Process's state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordState {
    series: RecordSeries,
}

impl RecordState {
    pub(crate) const fn new(series: RecordSeries) -> Self {
        Self { series }
    }

    /// The series.
    pub const fn series(&self) -> &RecordSeries {
        &self.series
    }
}

/// The climate's state: what was configured (without the record's days, which the `weather-record`
/// Process holds), today (none before the first day), the hour whose condition is in force, the
/// generator's carry, and — in a world that replays a record — the year of its first day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClimateState {
    configured: WeatherConfigured,
    today: Option<WeatherDay>,
    now: u8,
    chain: Chain,
    /// Absent, not `null`, in a world of rules alone, so its state is byte for byte TW-b's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    epoch_year: Option<i32>,
}

impl ClimateState {
    /// Configured, no day yet. The record's days are not kept here (SD-TW-d-5).
    pub(crate) fn unstarted(configured: &WeatherConfigured) -> Self {
        Self {
            configured: configured.without_record(),
            today: None,
            now: 0,
            chain: Chain::default(),
            epoch_year: None,
        }
    }

    /// The year of the world's first day, once a record world has had one (SD-TW-d-4).
    pub const fn epoch_year(&self) -> Option<i32> {
        self.epoch_year
    }

    /// The first day's year folded, if not yet known.
    pub(crate) fn with_epoch(self, year: i32) -> Self {
        Self {
            epoch_year: self.epoch_year.or(Some(year)),
            ..self
        }
    }

    /// The configuration.
    pub const fn configured(&self) -> &WeatherConfigured {
        &self.configured
    }

    /// Today's weather, once the first day has begun.
    pub const fn today(&self) -> Option<&WeatherDay> {
        self.today.as_ref()
    }

    /// The hour whose condition is in force.
    pub const fn now(&self) -> u8 {
        self.now
    }

    /// The carry the next day is drawn from.
    pub const fn chain(&self) -> Chain {
        self.chain
    }

    /// The condition in force, if a day has begun.
    pub fn condition(&self) -> Option<Condition> {
        self.today
            .as_ref()
            .map(|day| day.hours()[usize::from(self.now)].condition)
    }

    pub(crate) fn with_day(self, day: WeatherDay, now: u8) -> Self {
        Self {
            chain: day.chain(),
            today: Some(day),
            now,
            ..self
        }
    }

    pub(crate) fn with_now(self, now: u8) -> Self {
        Self { now, ..self }
    }
}
