//! The day record (step-19 §5.4): one local day's date, weekday, light events and sun, all integers.

use mineworld_contracts::WorldTime;
use serde::{Deserialize, Serialize};

use crate::civil::CalendarDate;
use crate::sun::{self, SunUnavailable};

/// Seconds in a local day. Every multiple of it from instant 0 is a local midnight — `schedule`'s
/// convention, restated as this pack's own constant (`INV-TW-4`; pinned by `tests/calendar.rs`).
pub const DAY: i64 = 86_400;

/// The sun is sampled every this many seconds, from 00:00 to 24:00 inclusive.
pub const SAMPLE_INTERVAL: i64 = 900;

/// How many samples a day's track has: 00:00, 00:15, … 24:00.
pub const SAMPLES: usize = 97;

/// An angle in millionths of a degree: a latitude or a longitude as configured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MicroDegrees(i32);

impl MicroDegrees {
    /// The angle, in micro-degrees.
    pub const fn new(micro: i32) -> Self {
        Self(micro)
    }

    /// The angle, in micro-degrees.
    pub const fn micro(self) -> i32 {
        self.0
    }

    /// The angle in degrees, for the sun's model only.
    pub(crate) fn degrees(self) -> f64 {
        f64::from(self.0) / 1_000_000.0
    }
}

/// Where the sun appears: elevation above the horizon (standard refraction) and azimuth from north,
/// clockwise, both in millidegrees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SunSample {
    elevation: i32,
    azimuth: i32,
}

impl SunSample {
    pub(crate) const fn new(elevation: i32, azimuth: i32) -> Self {
        Self { elevation, azimuth }
    }

    /// Millidegrees above the horizon; negative below it.
    pub const fn elevation(self) -> i32 {
        self.elevation
    }

    /// Millidegrees from north, clockwise, 0 … 359 999.
    pub const fn azimuth(self) -> i32 {
        self.azimuth
    }
}

/// The day's light events, each in seconds after the day's start; `None` when it does not happen that
/// day (polar day or night).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(missing_docs)] // Each field is its event's name.
pub struct DayEvents {
    pub astronomical_dawn: Option<u32>,
    pub civil_dawn: Option<u32>,
    pub sunrise: Option<u32>,
    pub solar_noon: Option<u32>,
    pub sunset: Option<u32>,
    pub civil_dusk: Option<u32>,
    pub astronomical_dusk: Option<u32>,
}

impl DayEvents {
    /// The events that change the light, each with the phase it begins, in a fixed order: by time,
    /// then dawn before dusk. Solar noon changes no phase and is not among them.
    pub fn light_changes(&self) -> Vec<(u32, Phase)> {
        let mut changes: Vec<(u32, Phase)> = [
            (self.astronomical_dawn, Phase::AstronomicalTwilight),
            (self.civil_dawn, Phase::CivilTwilight),
            (self.sunrise, Phase::Day),
            (self.sunset, Phase::CivilTwilight),
            (self.civil_dusk, Phase::AstronomicalTwilight),
            (self.astronomical_dusk, Phase::Night),
        ]
        .into_iter()
        .filter_map(|(at, phase)| at.map(|at| (at, phase)))
        .collect();
        // Stable: two events at one second keep the listed order.
        changes.sort_by_key(|(at, _)| *at);
        changes
    }
}

/// How light it is: the sun's elevation against −18°, −6° and the sunrise horizon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Phase {
    /// Below −18°.
    Night,
    /// From −18° to −6° (nautical twilight included).
    AstronomicalTwilight,
    /// From −6° to the sunrise horizon.
    CivilTwilight,
    /// Above the sunrise horizon.
    Day,
}

/// One local day: its date, weekday, start, light events and sun every 15 minutes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalendarDay {
    date: CalendarDate,
    weekday: u8,
    day_start: WorldTime,
    events: DayEvents,
    track: Vec<SunSample>,
}

impl CalendarDay {
    /// The day `date`, which begins at `day_start` (a local midnight), at a position whose local
    /// time is `utc_offset` seconds ahead of UTC. `epoch_days` is the epoch date's day number, so
    /// `day_start`'s UTC instant is known without a clock.
    ///
    /// # Errors
    ///
    /// [`SunUnavailable`] when the sun cannot be computed for the day.
    pub fn compute(
        date: CalendarDate,
        day_start: WorldTime,
        epoch_days: i64,
        utc_offset: i32,
        latitude: MicroDegrees,
        longitude: MicroDegrees,
    ) -> Result<(Self, Phase), SunUnavailable> {
        let utc_start = epoch_days * DAY + day_start.seconds() - i64::from(utc_offset);
        let events = sun::day_events(latitude, longitude, utc_start)?;
        let mut track = Vec::with_capacity(SAMPLES);
        for sample in 0..SAMPLES {
            #[allow(clippy::cast_possible_wrap)] // < 97.
            let offset = sample as i64 * SAMPLE_INTERVAL;
            track.push(sun::sun_at(latitude, longitude, utc_start + offset)?);
        }
        let phase = sun::phase_at(latitude, longitude, utc_start)?;
        Ok((
            Self {
                date,
                weekday: date.weekday(),
                day_start,
                events,
                track,
            },
            phase,
        ))
    }

    /// The date.
    pub const fn date(&self) -> CalendarDate {
        self.date
    }

    /// The weekday, 0 = Monday.
    pub const fn weekday(&self) -> u8 {
        self.weekday
    }

    /// The instant of local midnight that begins it.
    pub const fn day_start(&self) -> WorldTime {
        self.day_start
    }

    /// The light events.
    pub const fn events(&self) -> &DayEvents {
        &self.events
    }

    /// The sun every [`SAMPLE_INTERVAL`] seconds, [`SAMPLES`] samples.
    pub fn track(&self) -> &[SunSample] {
        &self.track
    }
}
