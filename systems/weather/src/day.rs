//! The weather's vocabulary: a condition, an hour, a day's summary, the generator's carry and the day
//! record (step-19 §6.6, SD-TW-b-9). All integers (`INV-TW-5`).

use mineworld_calendar::CalendarDate;
use mineworld_contracts::WorldTime;
use serde::{Deserialize, Serialize};

/// Hours in a local day.
pub const HOURS: usize = 24;

/// What the weather is doing in an hour. Closed: a new condition is a schema change of this pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Condition {
    /// Dry, 0 – 2 oktas of cloud.
    Clear,
    /// Dry, 3 – 6 oktas.
    PartlyCloudy,
    /// Dry, 7 – 8 oktas.
    Overcast,
    /// Dry, sky obscured by fog.
    Fog,
    /// Wet, at most 1.0 mm in the hour.
    Drizzle,
    /// Wet, more than 1.0 and at most 7.6 mm in the hour.
    Rain,
    /// Wet, more than 7.6 mm in the hour.
    HeavyRain,
    /// Wet, on a day with thunder.
    Thunderstorm,
}

/// One hour of weather.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WeatherHour {
    /// What the weather is doing.
    pub condition: Condition,
    /// Cloud cover, 0 – 8 oktas.
    pub cloud_oktas: u8,
    /// Air temperature, 0.1 °C.
    pub temperature_dc: i16,
    /// Precipitation in the hour, 0.1 mm.
    pub precipitation_tenth_mm: u16,
    /// Mean wind speed, 0.1 m/s.
    pub wind_dms: u16,
    /// The direction the wind comes from, degrees from north, clockwise, 0 – 359.
    pub wind_from_deg: u16,
}

/// A day's weather as a station would summarize it (the GHCN-Daily elements' units, step-19 §17.2
/// E-TWb-pre-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DailyWeather {
    /// The day's maximum temperature, 0.1 °C.
    pub tmax_dc: i16,
    /// The day's minimum temperature, 0.1 °C; always below `tmax_dc`.
    pub tmin_dc: i16,
    /// The day's precipitation, 0.1 mm; 0 on a dry day.
    pub prcp_tenth_mm: u16,
    /// The day's mean wind speed, 0.1 m/s.
    pub awnd_dms: u16,
    /// The direction the wind comes from, degrees.
    pub wind_from_deg: u16,
    /// Fog in the morning.
    pub fog: bool,
    /// Thunder (only on a wet day).
    pub thunder: bool,
    /// A grey morning (only on a dry day): the marine layer.
    pub overcast_morning: bool,
}

impl DailyWeather {
    /// Whether any precipitation fell.
    pub const fn wet(&self) -> bool {
        self.prcp_tenth_mm > 0
    }
}

/// The generator's carry from one day to the next: whether the day was wet, and the temperature
/// anomalies of its AR(1) process (SD-TW-b-5, -6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Chain {
    /// Whether the day was wet.
    pub wet: bool,
    /// The maximum temperature's departure from the month's mean, 0.1 °C.
    pub tmax_anomaly_dc: i16,
    /// The minimum temperature's departure from the month's mean, 0.1 °C.
    pub tmin_anomaly_dc: i16,
}

/// Where a day's weather came from. TW-d adds `Record` and `Filled`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Origin {
    /// Drawn by the rules.
    Rule,
}

/// One local day's weather: when it began, its date, its origin, its summary, the carry it leaves,
/// and its hours.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeatherDay {
    day_start: WorldTime,
    date: CalendarDate,
    origin: Origin,
    summary: DailyWeather,
    chain: Chain,
    hours: [WeatherHour; HOURS],
}

impl WeatherDay {
    pub(crate) const fn new(
        day_start: WorldTime,
        date: CalendarDate,
        origin: Origin,
        summary: DailyWeather,
        chain: Chain,
        hours: [WeatherHour; HOURS],
    ) -> Self {
        Self {
            day_start,
            date,
            origin,
            summary,
            chain,
            hours,
        }
    }

    /// The local midnight that begins it.
    pub const fn day_start(&self) -> WorldTime {
        self.day_start
    }

    /// The date.
    pub const fn date(&self) -> CalendarDate {
        self.date
    }

    /// Where it came from.
    pub const fn origin(&self) -> Origin {
        self.origin
    }

    /// The day's summary.
    pub const fn summary(&self) -> &DailyWeather {
        &self.summary
    }

    /// The carry this day leaves for the next.
    pub const fn chain(&self) -> Chain {
        self.chain
    }

    /// Its 24 hours, 00:00 first.
    pub const fn hours(&self) -> &[WeatherHour; HOURS] {
        &self.hours
    }
}
