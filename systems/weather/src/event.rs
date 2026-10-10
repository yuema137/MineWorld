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

use crate::configuration::WeatherConfiguration;
use crate::day::{Condition, WeatherDay};
use crate::rules::Rules;
use crate::system::WeatherSystem;

/// A reference to a station record. TW-d gives it its variants; in TW-b it has none, so no world can
/// claim a record, and a TW-b save's `record: null` stays readable when it gains them (SD-TW-b-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordRef {}

/// The world's weather, as configured: its seed, its climate, and (from TW-d) its record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeatherConfigured {
    seed: u64,
    rules: Rules,
    record: Option<RecordRef>,
}

impl Event for WeatherConfigured {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("weather-configured");
    const OWNER: SystemId = WeatherSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl WeatherConfigured {
    pub(crate) fn of(configuration: &WeatherConfiguration) -> Self {
        Self {
            seed: configuration.seed(),
            rules: configuration.rules().clone(),
            record: None,
        }
    }

    /// The seed.
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// The climate.
    pub const fn rules(&self) -> &Rules {
        &self.rules
    }

    /// The record, if any; always none in TW-b.
    pub const fn record(&self) -> Option<RecordRef> {
        self.record
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
