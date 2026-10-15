//! `configure/calendar.yaml`, decoded by this pack's own types (`ARC-61`): deserializing is
//! validating, so an out-of-range value is refused at assembly with the file, its line and column, and
//! a message naming the key.
//!
//! ```yaml
//! epoch: 2026-10-08     # local civil date at instant 0, 1901-01-01 … 2099-12-31
//! utc_offset: -08:00    # ±HH:MM, within ±14:00; fixed — no daylight saving (QTW-9)
//! latitude: 32.7157     # decimal degrees, −90 … 90, kept as integer micro-degrees
//! longitude: -117.1611  # decimal degrees, −180 … 180
//! ```
//!
//! What the configuration becomes is [`CalendarConfigured`](crate::event::CalendarConfigured), all
//! integers; this type is only ever read from the file.

use serde::Deserialize;

use crate::civil::CalendarDate;
use crate::day::MicroDegrees;

/// The calendar a world keeps: which date the world begins on, its clock's offset from UTC, and where
/// on Earth it is, which sets sunrise and sunset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(example = serde_json::json!({
    "epoch": "2026-10-08", "utc_offset": "-08:00", "latitude": 32.7157, "longitude": -117.1611
}))]
pub struct CalendarConfiguration {
    /// The local date the world begins on, YYYY-MM-DD, from 1901-01-01 to 2099-12-31.
    epoch: Epoch,
    /// The local clock's fixed offset from UTC, ±HH:MM within ±14:00, such as "-08:00" (no daylight
    /// saving).
    utc_offset: UtcOffset,
    /// Latitude in decimal degrees, north positive, from -90 to 90.
    latitude: Latitude,
    /// Longitude in decimal degrees, east positive, from -180 to 180.
    longitude: Longitude,
}

impl CalendarConfiguration {
    /// The local date at instant 0.
    pub const fn epoch(&self) -> CalendarDate {
        self.epoch.0
    }

    /// Seconds the local time is ahead of UTC.
    pub const fn utc_offset(&self) -> i32 {
        self.utc_offset.0
    }

    /// Latitude, north positive.
    pub const fn latitude(&self) -> MicroDegrees {
        self.latitude.0
    }

    /// Longitude, east positive.
    pub const fn longitude(&self) -> MicroDegrees {
        self.longitude.0
    }
}

/// A date written YYYY-MM-DD, such as "2026-10-08" — quoted in YAML — from 1901-01-01 to 2099-12-31.
///
/// 1901 … 2099 is the range NOAA's equations are stated for (step-19 §5.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "String")]
#[schemars(with = "String", extend("pattern" = "^(19|20)[0-9]{2}-[0-9]{2}-[0-9]{2}$"))]
struct Epoch(CalendarDate);

impl TryFrom<String> for Epoch {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        let refused =
            || format!("epoch is a date YYYY-MM-DD from 1901-01-01 to 2099-12-31, not '{text}'");
        let mut parts = text.split('-');
        let (Some(year), Some(month), Some(day), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return Err(refused());
        };
        if year.len() != 4 || month.len() != 2 || day.len() != 2 {
            return Err(refused());
        }
        let (Ok(year), Ok(month), Ok(day)) =
            (year.parse::<i32>(), month.parse::<u8>(), day.parse::<u8>())
        else {
            return Err(refused());
        };
        if !(1901..=2099).contains(&year) {
            return Err(refused());
        }
        CalendarDate::new(year, month, day)
            .map(Self)
            .ok_or_else(refused)
    }
}

/// A fixed offset from UTC, written ±HH:MM such as "-08:00" — quoted in YAML — within ±14:00.
///
/// Held in seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "String")]
#[schemars(with = "String", extend("pattern" = "^[+-][0-9]{2}:[0-5][0-9]$"))]
struct UtcOffset(i32);

impl TryFrom<String> for UtcOffset {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        let refused = || {
            format!("utc_offset is ±HH:MM within ±14:00, such as -08:00 or +05:30, not '{text}'")
        };
        let (sign, rest) = match text.as_bytes().first() {
            Some(b'+') => (1, &text[1..]),
            Some(b'-') => (-1, &text[1..]),
            _ => return Err(refused()),
        };
        let Some((hours, minutes)) = rest.split_once(':') else {
            return Err(refused());
        };
        if hours.len() != 2 || minutes.len() != 2 {
            return Err(refused());
        }
        let (Ok(hours), Ok(minutes)) = (hours.parse::<i32>(), minutes.parse::<i32>()) else {
            return Err(refused());
        };
        let seconds = hours * 3_600 + minutes * 60;
        if minutes >= 60 || seconds > 14 * 3_600 {
            return Err(refused());
        }
        Ok(Self(sign * seconds))
    }
}

/// Decimal degrees to micro-degrees, rounded half away from zero, refused outside `±bound`.
#[allow(clippy::cast_possible_truncation)] // |degrees| ≤ 180: fits.
fn micro(key: &str, degrees: f64, bound: f64) -> Result<MicroDegrees, String> {
    if degrees.is_finite() && degrees.abs() <= bound {
        Ok(MicroDegrees::new((degrees * 1_000_000.0).round() as i32))
    } else {
        Err(format!(
            "{key} is decimal degrees from -{bound} to {bound}, not {degrees}"
        ))
    }
}

/// A latitude in decimal degrees, from -90 to 90, north positive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "f64")]
#[schemars(with = "f64", extend("minimum" = -90, "maximum" = 90))]
struct Latitude(MicroDegrees);

impl TryFrom<f64> for Latitude {
    type Error = String;

    fn try_from(degrees: f64) -> Result<Self, Self::Error> {
        micro("latitude", degrees, 90.0).map(Self)
    }
}

/// A longitude in decimal degrees, from -180 to 180, east positive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "f64")]
#[schemars(with = "f64", extend("minimum" = -180, "maximum" = 180))]
struct Longitude(MicroDegrees);

impl TryFrom<f64> for Longitude {
    type Error = String;

    fn try_from(degrees: f64) -> Result<Self, Self::Error> {
        micro("longitude", degrees, 180.0).map(Self)
    }
}
