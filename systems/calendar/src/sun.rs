//! The sun, in integers: the only file that names `solar-positioning` or evaluates floating point
//! (`DEP-30`).
//!
//! Two questions, each answered for a position and a UTC instant (seconds from 1970-01-01T00:00Z):
//!
//! ```text
//! sun_at       where the sun appears: elevation (standard atmospheric refraction) and azimuth from
//!              north, clockwise, both in millidegrees
//! day_events   when, within one local day, the sun crosses −18°, −6° and the sunrise horizon
//!              (−0.833°, the library's and NOAA's convention) rising and setting, and transits;
//!              seconds after the day's start
//! phase_at     which light phase an instant is in, by the same three horizons
//! ```
//!
//! The library is built with `libm` and without `std`, so its transcendental functions are pure Rust
//! and give the same bits on every host. Every value is rounded half away from zero to an integer
//! before it leaves this file; a moved rounding boundary is caught by the pinned samples in this
//! file's tests (step-19 §16.8 (a)), and is a material stop, never a re-golden.
//!
//! ΔT (TT − UT) is the library's estimate at the middle of the instant's UTC month: fixed per month,
//! deterministic, and far below what a quantized degree can see.

use solar_positioning::time::JulianDate;
use solar_positioning::{Horizon, Location, RefractionCorrection, SolarEvents, SolarPositions};

use crate::civil::CalendarDate;
use crate::day::{DayEvents, MicroDegrees, Phase, SunSample};

/// Seconds in a day: a UTC day here, a local day where the caller adds its offset.
const DAY: i64 = 86_400;

/// The Julian date of 1970-01-01T00:00Z.
const UNIX_EPOCH_JD: f64 = 2_440_587.5;

/// Why the sun could not be computed: the library refused (a year outside its range). Unreachable
/// for a configuration this pack accepts, and still never a panic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SunUnavailable(pub String);

impl core::fmt::Display for SunUnavailable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "the sun could not be computed: {}", self.0)
    }
}

fn unavailable(error: solar_positioning::Error) -> SunUnavailable {
    SunUnavailable(error.to_string())
}

/// A position as the library takes it.
fn location(latitude: MicroDegrees, longitude: MicroDegrees) -> Location {
    Location {
        latitude: latitude.degrees(),
        longitude: longitude.degrees(),
    }
}

/// The UT Julian date of a UTC instant.
#[allow(clippy::cast_precision_loss)] // |utc| < 2^53: exact.
fn julian(utc: i64) -> f64 {
    UNIX_EPOCH_JD + utc as f64 / DAY as f64
}

/// ΔT at the middle of `utc`'s UTC month.
fn delta_t(utc: i64) -> Result<f64, SunUnavailable> {
    let date = CalendarDate::from_days(utc.div_euclid(DAY))
        .ok_or_else(|| SunUnavailable(format!("no date for instant {utc}")))?;
    solar_positioning::delta_t::estimate_from_date(date.year(), u32::from(date.month()))
        .map_err(unavailable)
}

/// Degrees to millidegrees, rounded half away from zero.
#[allow(clippy::cast_possible_truncation)] // |degrees| ≤ 360: fits.
fn millidegrees(degrees: f64) -> i32 {
    (degrees * 1_000.0).round() as i32
}

/// Where the sun appears at `utc`, seen from the position.
///
/// # Errors
///
/// [`SunUnavailable`] when the library refuses the instant.
pub fn sun_at(
    latitude: MicroDegrees,
    longitude: MicroDegrees,
    utc: i64,
) -> Result<SunSample, SunUnavailable> {
    let time = JulianDate::new(julian(utc), delta_t(utc)?).map_err(unavailable)?;
    let position = SolarPositions::new()
        .at_from_julian(
            time,
            location(latitude, longitude),
            0.0,
            Some(RefractionCorrection::standard()),
        )
        .map_err(unavailable)?;
    let azimuth = millidegrees(position.azimuth());
    Ok(SunSample::new(
        millidegrees(position.elevation_angle()),
        if azimuth >= 360_000 {
            azimuth - 360_000
        } else {
            azimuth
        },
    ))
}

/// The light phase at `utc`: the sun's unrefracted elevation against the three horizons the events
/// are found with, so a phase and its events agree.
///
/// # Errors
///
/// [`SunUnavailable`] when the library refuses the instant.
pub fn phase_at(
    latitude: MicroDegrees,
    longitude: MicroDegrees,
    utc: i64,
) -> Result<Phase, SunUnavailable> {
    let time = JulianDate::new(julian(utc), delta_t(utc)?).map_err(unavailable)?;
    let elevation = SolarPositions::new()
        .at_from_julian(time, location(latitude, longitude), 0.0, None)
        .map_err(unavailable)?
        .elevation_angle();
    Ok(if elevation >= Horizon::SunriseSunset.elevation_angle() {
        Phase::Day
    } else if elevation >= Horizon::CivilTwilight.elevation_angle() {
        Phase::CivilTwilight
    } else if elevation >= Horizon::AstronomicalTwilight.elevation_angle() {
        Phase::AstronomicalTwilight
    } else {
        Phase::Night
    })
}

/// The day's light events, in seconds after `day_start` (a UTC instant): the first rising and the
/// first setting crossing of each horizon within `(day_start, day_start + 86 400)`, and the first
/// transit. An event that does not happen that day — polar day or night, or a crossing that rounds
/// onto either midnight — is `None`.
///
/// # Errors
///
/// [`SunUnavailable`] when the library refuses the day.
pub fn day_events(
    latitude: MicroDegrees,
    longitude: MicroDegrees,
    day_start: i64,
) -> Result<DayEvents, SunUnavailable> {
    let events = SolarEvents::new();
    let place = location(latitude, longitude);
    let delta_t = delta_t(day_start)?;
    let start = julian(day_start);
    let end = julian(day_start + DAY);
    let offset = |found: Option<f64>| -> Option<u32> {
        let jd = found?;
        let seconds = ((jd - start) * DAY as f64).round();
        // Strictly inside the day: an event on a midnight belongs to that midnight's own record.
        (seconds >= 1.0 && seconds < DAY as f64).then(|| {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // 1 … 86 399.
            let seconds = seconds as u32;
            seconds
        })
    };
    let rise = |horizon| {
        events
            .next_rise_from_julian(start, end, place, delta_t, horizon)
            .map(offset)
            .map_err(unavailable)
    };
    let set = |horizon| {
        events
            .next_set_from_julian(start, end, place, delta_t, horizon)
            .map(offset)
            .map_err(unavailable)
    };
    Ok(DayEvents {
        astronomical_dawn: rise(Horizon::AstronomicalTwilight)?,
        civil_dawn: rise(Horizon::CivilTwilight)?,
        sunrise: rise(Horizon::SunriseSunset)?,
        solar_noon: events
            .next_transit_from_julian(start, end, place.longitude, delta_t)
            .map(offset)
            .map_err(unavailable)?,
        sunset: set(Horizon::SunriseSunset)?,
        civil_dusk: set(Horizon::CivilTwilight)?,
        astronomical_dusk: set(Horizon::AstronomicalTwilight)?,
    })
}

#[cfg(test)]
mod tests;
