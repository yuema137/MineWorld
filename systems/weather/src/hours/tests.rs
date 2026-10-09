//! Owns the daily-to-hourly derivation: exact extremes, exact sums, the classification boundaries, fog
//! hours and the polar fallbacks.

use mineworld_calendar::{CalendarDate, DayEvents};

use super::{FOG_LAST_HOUR, hours, wet_hours};
use crate::day::{Chain, Condition, DailyWeather};
use crate::fixture::{SAN_DIEGO, rules};
use crate::generate::day;

/// San Diego 2026-10-08's events (E-TWa-2): civil dawn 05:23, sunrise 05:47.
const SAN_DIEGO_EVENTS: DayEvents = DayEvents {
    astronomical_dawn: Some(15_956),
    civil_dawn: Some(19_384),
    sunrise: Some(20_864),
    solar_noon: Some(41_766),
    sunset: Some(62_639),
    civil_dusk: Some(64_118),
    astronomical_dusk: Some(67_540),
};

const POLAR: DayEvents = DayEvents {
    astronomical_dawn: None,
    civil_dawn: None,
    sunrise: None,
    solar_noon: None,
    sunset: None,
    civil_dusk: None,
    astronomical_dusk: None,
};

fn dry() -> DailyWeather {
    DailyWeather {
        tmax_dc: 220,
        tmin_dc: 150,
        prcp_tenth_mm: 0,
        awnd_dms: 30,
        wind_from_deg: 290,
        fog: false,
        thunder: false,
        overcast_morning: false,
    }
}

/// (e) Over every day of a 100-year run the hourly extremes are the day's TMAX and TMIN exactly, the
/// hours' precipitation sums to the day's, and every hour carries the day's wind.
#[test]
fn the_hours_keep_the_days_extremes_and_its_amount_exactly() {
    let table = rules(SAN_DIEGO);
    let epoch = CalendarDate::new(2026, 10, 8).expect("a date").days();
    let mut chain = Chain::default();
    let mut wet_days = 0;
    for index in 0..36_524 {
        let month = CalendarDate::from_days(epoch + index).expect("ok").month();
        let (summary, next) = day(&table, 19, index, month, chain);
        chain = next;
        let hours = hours(&summary, &SAN_DIEGO_EVENTS, 19, index);
        let temperatures = hours.iter().map(|h| h.temperature_dc);
        assert_eq!(
            temperatures.clone().max(),
            Some(summary.tmax_dc),
            "day {index}"
        );
        assert_eq!(temperatures.min(), Some(summary.tmin_dc), "day {index}");
        let sum: u32 = hours
            .iter()
            .map(|h| u32::from(h.precipitation_tenth_mm))
            .sum();
        assert_eq!(sum, u32::from(summary.prcp_tenth_mm), "day {index}");
        let wet = hours
            .iter()
            .filter(|h| h.precipitation_tenth_mm > 0)
            .count();
        assert_eq!(wet, wet_hours(summary.prcp_tenth_mm), "day {index}");
        assert!(hours.iter().all(|h| h.wind_dms == summary.awnd_dms
            && h.wind_from_deg == summary.wind_from_deg
            && h.cloud_oktas <= 8));
        if summary.wet() {
            wet_days += 1;
        }
    }
    assert!(
        wet_days > 1_000,
        "the run had wet days to check: {wet_days}"
    );
}

/// The condition of a day whose amount falls in two equal hours (2 wet hours below 2.0 mm).
fn wet_condition(prcp: u16, thunder: bool) -> Vec<Condition> {
    let summary = DailyWeather {
        prcp_tenth_mm: prcp,
        thunder,
        ..dry()
    };
    hours(&summary, &SAN_DIEGO_EVENTS, 19, 0)
        .iter()
        .filter(|h| h.precipitation_tenth_mm > 0)
        .map(|h| h.condition)
        .collect()
}

/// (f) Drizzle up to 1.0 mm/h, rain to 7.6 mm/h, heavy rain above (AMS); a thunder day's wet hours are
/// thunderstorms.
#[test]
fn the_rate_boundaries_are_the_ams_thresholds() {
    // 20 tenths in 2 hours: 10 each — drizzle at the boundary.
    assert_eq!(wet_condition(20, false), [Condition::Drizzle; 2]);
    // 22 tenths in 4 hours (above 2.0 mm): 5, 5, 5, 5 plus 2 on the first.
    assert_eq!(wet_condition(22, false), [Condition::Drizzle; 4]);
    // 44 in 4: 11 each — just rain.
    assert_eq!(wet_condition(44, false), [Condition::Rain; 4]);
    // 304 in 12 (above 25 mm): 25 each, 4 extra on the first — rain; 912 in 12: 76 each — still rain.
    assert_eq!(wet_condition(912, false), [Condition::Rain; 12]);
    // 924 in 12: 77 each — heavy rain.
    assert_eq!(wet_condition(924, false), [Condition::HeavyRain; 12]);
    // A remainder can tip only the first hour: 913 in 12 → 77 then eleven 76s.
    let tipped = wet_condition(913, false);
    assert_eq!(tipped[0], Condition::HeavyRain);
    assert!(tipped[1..].iter().all(|c| *c == Condition::Rain));
    assert_eq!(wet_condition(20, true), [Condition::Thunderstorm; 2]);
}

/// (f) A fog day is foggy from the civil-dawn hour to 09:59; with no civil dawn (polar) from 05:00.
/// A polar day's temperature curve takes its minimum at 06:00.
#[test]
fn fog_lifts_at_ten_and_polar_days_fall_back_to_the_stated_hours() {
    let foggy = DailyWeather { fog: true, ..dry() };
    let fog_hours = |events: &DayEvents| -> Vec<usize> {
        hours(&foggy, events, 19, 0)
            .iter()
            .enumerate()
            .filter(|(_, h)| h.condition == Condition::Fog)
            .map(|(hour, _)| hour)
            .collect()
    };
    assert_eq!(
        fog_hours(&SAN_DIEGO_EVENTS),
        (5..=FOG_LAST_HOUR).collect::<Vec<_>>()
    );
    assert_eq!(fog_hours(&POLAR), (5..=FOG_LAST_HOUR).collect::<Vec<_>>());
    let late_dawn = DayEvents {
        civil_dawn: Some(7 * 3_600 + 10),
        ..SAN_DIEGO_EVENTS
    };
    assert_eq!(fog_hours(&late_dawn), vec![7, 8, 9]);

    let polar = hours(&dry(), &POLAR, 19, 0);
    assert_eq!(
        polar[6].temperature_dc, 150,
        "minimum at the fallback sunrise, 06:00"
    );
    assert_eq!(polar[15].temperature_dc, 220, "maximum at 15:00");
    let san_diego = hours(&dry(), &SAN_DIEGO_EVENTS, 19, 0);
    assert_eq!(
        san_diego[5].temperature_dc, 150,
        "sunrise 05:47 → the 05:00 hour"
    );
}

/// A dry day: clear, or overcast through a grey morning (05:00 – 10:59); a wet day is overcast in and
/// next to its wet hours and clear elsewhere.
#[test]
fn a_dry_hour_takes_its_cloud_from_the_day() {
    let clear = hours(&dry(), &SAN_DIEGO_EVENTS, 19, 0);
    assert!(
        clear
            .iter()
            .all(|h| h.condition == Condition::Clear && h.cloud_oktas == 1)
    );
    let grey = hours(
        &DailyWeather {
            overcast_morning: true,
            ..dry()
        },
        &SAN_DIEGO_EVENTS,
        19,
        0,
    );
    let overcast: Vec<usize> = grey
        .iter()
        .enumerate()
        .filter(|(_, h)| h.condition == Condition::Overcast)
        .map(|(hour, _)| hour)
        .collect();
    assert_eq!(overcast, (5..=10).collect::<Vec<_>>());

    let wet = hours(
        &DailyWeather {
            prcp_tenth_mm: 50,
            ..dry()
        },
        &SAN_DIEGO_EVENTS,
        19,
        0,
    );
    for (hour, h) in wet.iter().enumerate() {
        let near = (hour.saturating_sub(1)..=(hour + 1).min(23))
            .any(|other| wet[other].precipitation_tenth_mm > 0);
        let expected = if near { 8 } else { 1 };
        assert_eq!(h.cloud_oktas, expected, "hour {hour}");
    }
}
