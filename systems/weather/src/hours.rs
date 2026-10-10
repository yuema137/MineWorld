//! A day's summary spread over its 24 hours, in integers (SD-TW-b-8): temperature on a diurnal curve,
//! the wet hours placed as seeded runs, and each hour's condition and cloud.
//!
//! Sources (read 2026-10-09):
//! - Temperature shape: Parton, W.J. & Logan, J.A. 1981, "A model for diurnal variation in soil and
//!   air temperature", *Agricultural Meteorology* 23:205–216 — a sine from the minimum at sunrise to the
//!   maximum, then an exponential decay through the night with their screen-height coefficient
//!   b = 2.2.
//! - Intensity: American Meteorological Society, *Glossary of Meteorology* — "drizzle … seldom exceeds
//!   1 mm per hour" (<https://glossary.ametsoc.org/wiki/Drizzle>); heavy rain "more than 7.6 mm per hour"
//!   (<https://glossary.ametsoc.org/wiki/Rain>).
//! - Cloud: the WMO okta scale, eighths of sky covered, 0 – 8 (WMO-No. 306, code table 2700).

use mineworld_calendar::DayEvents;

use crate::day::{Condition, DailyWeather, HOURS, WeatherHour};
use crate::draw::{draw, index};

#[cfg(test)]
mod tests;

/// The first and last sunrise hour [`DIURNAL`] has a row for. A sunrise outside them (near a polar day)
/// is taken as the nearest (TWb-D3).
pub const SUNRISE_HOURS: (usize, usize) = (1, 13);

/// The hour of the day's maximum temperature.
pub const PEAK_HOUR: usize = 15;

/// The sunrise hour taken when the day has no sunrise (polar day or night).
pub const POLAR_SUNRISE_HOUR: usize = 6;

/// The civil-dawn hour taken when the day has no civil dawn.
pub const POLAR_CIVIL_DAWN_HOUR: usize = 5;

/// The last hour a fog day is foggy (fog lifts at 10:00).
pub const FOG_LAST_HOUR: usize = 9;

/// The hours of a grey morning: 05:00 – 10:59.
pub const OVERCAST_MORNING_HOURS: (usize, usize) = (5, 10);

/// The diurnal temperature curve, per mille of the day's range above its minimum, for each sunrise hour
/// `s` in [`SUNRISE_HOURS`] (row `s − 1`) and each hour `h`. Precomputed once, offline, by
///
/// ```text
/// s ≤ h ≤ 15:   c = 1000 · sin(π/2 · (h − s) / (15 − s))
/// otherwise:    n = hours since 15:00 (h − 15, or h + 9 before sunrise), N = s + 9 (15:00 to the
///               next sunrise), c = 1000 · (e^(−b·n/N) − e^(−b)) / (1 − e^(−b)),  b = 2.2
/// ```
///
/// rounded half away from zero (Parton & Logan 1981, above). So every row is exactly 0 at its sunrise
/// hour and exactly 1000 at 15:00, and lies in 0 … 1000 elsewhere. `tests/diurnal.rs` recomputes the
/// table from the formula and holds it equal.
#[rustfmt::skip]
pub const DIURNAL: [[u16; HOURS]; SUNRISE_HOURS.1 - SUNRISE_HOURS.0 + 1] = [
    [31, 0, 112, 223, 330, 434, 532, 623, 707, 782, 847, 901, 944, 975, 994, 1000, 778, 600, 457, 342, 250, 176, 116, 69], // sunrise 01:00
    [61, 28, 0, 121, 239, 355, 465, 568, 663, 749, 823, 885, 935, 971, 993, 1000, 796, 629, 493, 381, 289, 214, 153, 102], // sunrise 02:00
    [91, 55, 25, 0, 131, 259, 383, 500, 609, 707, 793, 866, 924, 966, 991, 1000, 812, 655, 524, 416, 325, 250, 187, 135], // sunrise 03:00
    [121, 82, 50, 23, 0, 142, 282, 415, 541, 655, 756, 841, 910, 959, 990, 1000, 825, 677, 552, 447, 358, 283, 219, 166], // sunrise 04:00
    [149, 109, 75, 46, 21, 0, 156, 309, 454, 588, 707, 809, 891, 951, 988, 1000, 836, 697, 577, 475, 388, 313, 250, 195], // sunrise 05:00
    [176, 135, 99, 69, 42, 20, 0, 174, 342, 500, 643, 766, 866, 940, 985, 1000, 847, 714, 600, 501, 416, 342, 278, 223], // sunrise 06:00
    [202, 160, 123, 91, 64, 39, 18, 0, 195, 383, 556, 707, 831, 924, 981, 1000, 856, 730, 620, 524, 441, 368, 305, 250], // sunrise 07:00
    [226, 184, 146, 113, 84, 59, 37, 17, 0, 223, 434, 623, 782, 901, 975, 1000, 863, 744, 638, 546, 464, 393, 330, 275], // sunrise 08:00
    [250, 207, 169, 135, 105, 79, 55, 35, 16, 0, 259, 500, 707, 866, 966, 1000, 871, 756, 655, 565, 486, 416, 353, 298], // sunrise 09:00
    [272, 229, 190, 156, 125, 98, 73, 52, 32, 15, 0, 309, 588, 809, 951, 1000, 877, 768, 670, 583, 506, 437, 375, 321], // sunrise 10:00
    [293, 250, 211, 176, 145, 116, 91, 69, 49, 31, 14, 0, 383, 707, 924, 1000, 883, 778, 684, 600, 524, 457, 396, 342], // sunrise 11:00
    [313, 270, 231, 195, 163, 135, 109, 86, 65, 46, 29, 14, 0, 500, 866, 1000, 888, 787, 697, 615, 541, 475, 416, 362], // sunrise 12:00
    [333, 289, 250, 214, 182, 153, 126, 102, 81, 61, 44, 28, 13, 0, 707, 1000, 893, 796, 709, 629, 558, 493, 434, 381], // sunrise 13:00
];

/// Wet hours by the day's amount, 0.1 mm: up to 2.0 mm → 2 h, up to 10 mm → 4 h, up to 25 mm → 8 h,
/// more → 12 h. A stated design default, not a measured climatology; the hourly layer (TW-g) replaces it.
pub const WET_HOURS: [(u16, usize); 3] = [(20, 2), (100, 4), (250, 8)];

/// Wet hours above the last row of [`WET_HOURS`].
pub const WET_HOURS_MAX: usize = 12;

/// Above this amount (0.1 mm) the wet hours fall in two runs, one in each half of the day.
pub const TWO_RUNS_ABOVE: u16 = 100;

/// An hour with at most this much (0.1 mm) is drizzle: AMS, "seldom exceeds 1 mm per hour".
pub const DRIZZLE_MAX: u16 = 10;

/// An hour with more than this much (0.1 mm) is heavy rain: AMS, "more than 7.6 mm per hour".
pub const RAIN_MAX: u16 = 76;

/// Cloud on a clear hour, oktas.
pub const FAIR_OKTAS: u8 = 1;

/// Cloud on a grey, wet or foggy hour, oktas.
pub const OVERCAST_OKTAS: u8 = 8;

/// The condition a dry hour's cloud gives, by the okta scale: 0 – 2 clear, 3 – 6 partly cloudy,
/// 7 – 8 overcast.
pub const fn sky(oktas: u8) -> Condition {
    match oktas {
        0..=2 => Condition::Clear,
        3..=6 => Condition::PartlyCloudy,
        _ => Condition::Overcast,
    }
}

/// An hour of `seconds` after midnight, if it is one.
fn hour_of(seconds: Option<u32>) -> Option<usize> {
    seconds.and_then(|at| usize::try_from(at / 3_600).ok())
}

/// How many hours of a day with `prcp` (0.1 mm) are wet.
pub fn wet_hours(prcp: u16) -> usize {
    if prcp == 0 {
        return 0;
    }
    WET_HOURS
        .iter()
        .find(|(up_to, _)| prcp <= *up_to)
        .map_or(WET_HOURS_MAX, |(_, hours)| *hours)
}

/// Which hours are wet: one run, or two runs (one in each half of the day) above
/// [`TWO_RUNS_ABOVE`], each placed by its own draw. Runs never wrap past midnight and never overlap.
fn wet_mask(prcp: u16, seed: u64, day_index: i64) -> [bool; HOURS] {
    let mut wet = [false; HOURS];
    let length = wet_hours(prcp);
    if length == 0 {
        return wet;
    }
    // A run of `run` hours starting in first ..= last, by the draw `k`.
    let mut place = |run: usize, first: usize, last: usize, k: u64| {
        let span = last - first + 1;
        let offset = usize::from(draw(seed, day_index, k)) * span / 1_000;
        for flag in &mut wet[first + offset..first + offset + run] {
            *flag = true;
        }
    };
    if prcp > TWO_RUNS_ABOVE {
        let run = length / 2;
        place(run, 0, HOURS / 2 - run, index::FIRST_RUN);
        place(run, HOURS / 2, HOURS - run, index::SECOND_RUN);
    } else {
        place(length, 0, HOURS - length, index::FIRST_RUN);
    }
    wet
}

/// The 24 hours of a day with `summary`, whose light events are `events`, numbered `day_index`.
pub fn hours(
    summary: &DailyWeather,
    events: &DayEvents,
    seed: u64,
    day_index: i64,
) -> [WeatherHour; HOURS] {
    let sunrise = hour_of(events.sunrise)
        .unwrap_or(POLAR_SUNRISE_HOUR)
        .clamp(SUNRISE_HOURS.0, SUNRISE_HOURS.1);
    let curve = &DIURNAL[sunrise - SUNRISE_HOURS.0];
    let fog_from = hour_of(events.civil_dawn).unwrap_or(POLAR_CIVIL_DAWN_HOUR);

    let wet = wet_mask(summary.prcp_tenth_mm, seed, day_index);
    let wet_count = wet.iter().filter(|flag| **flag).count();
    // The rate per wet hour, the remainder on the first wet hour, so the hours sum to the day.
    let (rate, remainder) = match u16::try_from(wet_count) {
        Ok(count) if count > 0 => (summary.prcp_tenth_mm / count, summary.prcp_tenth_mm % count),
        _ => (0, 0),
    };
    let first_wet = wet.iter().position(|flag| *flag);
    let near_wet = |h: usize| wet[h] || (h > 0 && wet[h - 1]) || (h + 1 < HOURS && wet[h + 1]);

    let range = i32::from(summary.tmax_dc) - i32::from(summary.tmin_dc);
    core::array::from_fn(|h| {
        let precipitation = match (wet[h], first_wet == Some(h)) {
            (true, true) => rate + remainder,
            (true, false) => rate,
            (false, _) => 0,
        };
        let (condition, cloud_oktas) = if wet[h] {
            let condition = if summary.thunder {
                Condition::Thunderstorm
            } else if precipitation <= DRIZZLE_MAX {
                Condition::Drizzle
            } else if precipitation <= RAIN_MAX {
                Condition::Rain
            } else {
                Condition::HeavyRain
            };
            (condition, OVERCAST_OKTAS)
        } else if summary.fog && (fog_from..=FOG_LAST_HOUR).contains(&h) {
            (Condition::Fog, OVERCAST_OKTAS)
        } else {
            let grey_morning = summary.overcast_morning
                && (OVERCAST_MORNING_HOURS.0..=OVERCAST_MORNING_HOURS.1).contains(&h);
            let oktas = if grey_morning || (summary.wet() && near_wet(h)) {
                OVERCAST_OKTAS
            } else {
                FAIR_OKTAS
            };
            (sky(oktas), oktas)
        };
        // T(h) = TMIN + range × c / 1000; range > 0 and c ≥ 0, so the division rounds down, and c = 1000
        // gives TMAX exactly.
        let temperature = i32::from(summary.tmin_dc) + range * i32::from(curve[h]) / 1_000;
        WeatherHour {
            condition,
            cloud_oktas,
            temperature_dc: i16::try_from(temperature)
                .expect("between the day's minimum and maximum, both i16"),
            precipitation_tenth_mm: precipitation,
            wind_dms: summary.awnd_dms,
            wind_from_deg: summary.wind_from_deg,
        }
    })
}
