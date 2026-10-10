//! The facts this pack records (SD-TW-b-4, -9). Only this pack states them.
//!
//! `weather-configured` is the world's configuration (`ARC-61`), SystemInternal with no subjects.
//! `weather-day` is the day's weather record, SystemInternal: it is how the climate is kept, not
//! something anybody perceives as an event. `weather-changed` is Public with no subjects — the weather
//! turning is the world's, like "a change of season" (`contracts/src/event.rs`) — so other packs may
//! react to it (`ARC-26`, `ARC-28`) without this pack knowing them.

use mineworld_contracts::{Event, EventSchemaVersion, EventTypeId, SystemId};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::day::{Condition, WeatherDay};
use crate::record::RecordSeries;
use crate::rules::Rules;
use crate::system::WeatherSystem;

/// The world's weather, as configured: its seed, its climate, and, when it replays one, its record —
/// every day of it, decoded and packed (SD-TW-d-5), so the world is a fold of its facts and a changed
/// file is configuration drift. `record` is `null` for a world of rules alone, exactly as in TW-b
/// (SD-TW-b-4), so such a world's facts are unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeatherConfigured {
    seed: u64,
    rules: Rules,
    record: Option<RecordSeries>,
}

impl Event for WeatherConfigured {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("weather-configured");
    const OWNER: SystemId = WeatherSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl WeatherConfigured {
    /// What a configuration and its decoded record (if any) state.
    pub(crate) const fn new(seed: u64, rules: Rules, record: Option<RecordSeries>) -> Self {
        Self {
            seed,
            rules,
            record,
        }
    }

    /// The same configuration without its record: what the climate Process keeps, since the series
    /// lives in its own Process (SD-TW-d-5) and the climate state stays small (SD-TW-b-6).
    pub(crate) fn without_record(&self) -> Self {
        Self {
            seed: self.seed,
            rules: self.rules.clone(),
            record: None,
        }
    }

    /// The record, taken out.
    pub(crate) fn into_record(self) -> Option<RecordSeries> {
        self.record
    }

    /// The seed.
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// The climate.
    pub const fn rules(&self) -> &Rules {
        &self.rules
    }

    /// The record, if the world replays one.
    pub const fn record(&self) -> Option<&RecordSeries> {
        self.record.as_ref()
    }
}

/// `weather-day`: a day's weather is its own fact.
impl Event for WeatherDay {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("weather-day");
    const OWNER: SystemId = WeatherSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// The weather turned: the hour it turned at, and what it is now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeatherChanged {
    hour: u8,
    condition: Condition,
    cloud_oktas: u8,
    precipitation_tenth_mm: u16,
}

impl Event for WeatherChanged {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("weather-changed");
    const OWNER: SystemId = WeatherSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl WeatherChanged {
    pub(crate) fn at_hour(day: &WeatherDay, hour: u8) -> Self {
        let weather = day.hours()[usize::from(hour)];
        Self {
            hour,
            condition: weather.condition,
            cloud_oktas: weather.cloud_oktas,
            precipitation_tenth_mm: weather.precipitation_tenth_mm,
        }
    }

    /// The hour of the day it turned at, 0 … 23.
    pub const fn hour(&self) -> u8 {
        self.hour
    }

    /// What the weather is doing now.
    pub const fn condition(&self) -> Condition {
        self.condition
    }

    /// Cloud now, oktas.
    pub const fn cloud_oktas(&self) -> u8 {
        self.cloud_oktas
    }

    /// Precipitation in this hour, 0.1 mm.
    pub const fn precipitation_tenth_mm(&self) -> u16 {
        self.precipitation_tenth_mm
    }
}
