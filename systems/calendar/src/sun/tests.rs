//! The sun against independent references, and pinned.
//!
//! References, read once and never fetched at run time (step-19 §16.4 C2, §16.5):
//!
//! ```text
//! NOAA ESRL/GML Solar Calculator, sunrise/sunset/solar-noon table, San Diego (32.7157, −117.1611),
//!   https://gml.noaa.gov/grad/solcalc/table.php?lat=32.7157&lon=-117.1611&year=2026, read 2026-10-08.
//!   The page prints local time for America/Los_Angeles with daylight saving ("Time Zone Offset
//!   −7.0" for October): 2026-10-08 sunrise 06:48, sunset 18:24, solar noon 12:36:14 (PDT), i.e. at
//!   this world's fixed −08:00: sunrise 05:48, sunset 17:24, solar noon 11:36:14. (Its December 21
//!   row, 06:47 / 16:47 / 11:46:38, is printed in standard time, which confirms the reading.)
//! NOAA's elevation at an instant is computed in the browser and published as no table, so the
//!   elevation reference is NOAA's own published spreadsheet equations (NOAA_Solar_Calculations_day.xls,
//!   linked from https://gml.noaa.gov/grad/solcalc/calcdetails.html, read 2026-10-08; Meeus), with
//!   that page's refraction formulas — `noaa_elevation` below, an independent model (not SPA).
//! ```

use super::*;

/// San Diego, as configured in `worlds/market-town/configure/calendar.yaml`.
const SAN_DIEGO: (MicroDegrees, MicroDegrees) = (
    MicroDegrees::new(32_715_700),
    MicroDegrees::new(-117_161_100),
);

/// The fixed offset San Diego's world keeps: −08:00 (QTW-9, no DST).
const PST: i64 = -8 * 3_600;

/// 2026-10-08's day number.
const OCTOBER_8: i64 = 20_734;

/// The UTC instant of `hh:mm:ss` local on day `days` at offset `offset`.
const fn local(days: i64, offset: i64, hh: i64, mm: i64, ss: i64) -> i64 {
    days * DAY + hh * 3_600 + mm * 60 + ss - offset
}

/// NOAA's spreadsheet model: the refraction-corrected solar elevation, in degrees, at `utc`
/// (seconds from 1970-01-01T00:00Z), for latitude and longitude in degrees (east positive). Column
/// names follow the spreadsheet's.
fn noaa_elevation(latitude: f64, longitude: f64, utc: i64) -> f64 {
    let radians = f64::to_radians;
    let degrees = f64::to_degrees;
    #[allow(clippy::cast_precision_loss)]
    let julian_day = 2_440_587.5 + utc as f64 / 86_400.0;
    let century = (julian_day - 2_451_545.0) / 36_525.0;
    let mean_long =
        (280.466_46 + century * (36_000.769_83 + century * 0.000_303_2)).rem_euclid(360.0);
    let mean_anom = 357.529_11 + century * (35_999.050_29 - 0.000_153_7 * century);
    let eccent = 0.016_708_634 - century * (0.000_042_037 + 0.000_000_126_7 * century);
    let eq_of_ctr = radians(mean_anom).sin()
        * (1.914_602 - century * (0.004_817 + 0.000_014 * century))
        + radians(2.0 * mean_anom).sin() * (0.019_993 - 0.000_101 * century)
        + radians(3.0 * mean_anom).sin() * 0.000_289;
    let true_long = mean_long + eq_of_ctr;
    let omega = radians(125.04 - 1_934.136 * century);
    let app_long = true_long - 0.005_69 - 0.004_78 * omega.sin();
    let mean_obliq = 23.0
        + (26.0
            + (21.448 - century * (46.815 + century * (0.000_59 - century * 0.001_813))) / 60.0)
            / 60.0;
    let obliq = mean_obliq + 0.002_56 * omega.cos();
    let declination = degrees((radians(obliq).sin() * radians(app_long).sin()).asin());
    let y = radians(obliq / 2.0).tan().powi(2);
    let eq_of_time = 4.0
        * degrees(
            y * (2.0 * radians(mean_long)).sin() - 2.0 * eccent * radians(mean_anom).sin()
                + 4.0 * eccent * y * radians(mean_anom).sin() * (2.0 * radians(mean_long)).cos()
                - 0.5 * y * y * (4.0 * radians(mean_long)).sin()
                - 1.25 * eccent * eccent * (2.0 * radians(mean_anom)).sin(),
        );
    #[allow(clippy::cast_precision_loss)]
    let minutes_utc = utc.rem_euclid(86_400) as f64 / 60.0;
    let true_solar = (minutes_utc + eq_of_time + 4.0 * longitude).rem_euclid(1_440.0);
    let hour_angle = if true_solar / 4.0 < 0.0 {
        true_solar / 4.0 + 180.0
    } else {
        true_solar / 4.0 - 180.0
    };
    let zenith = degrees(
        (radians(latitude).sin() * radians(declination).sin()
            + radians(latitude).cos() * radians(declination).cos() * radians(hour_angle).cos())
        .acos(),
    );
    let h = 90.0 - zenith;
    let t = radians(h).tan();
    let refraction = if h > 85.0 {
        0.0
    } else if h > 5.0 {
        58.1 / t - 0.07 / t.powi(3) + 0.000_086 / t.powi(5)
    } else if h > -0.575 {
        1_735.0 + h * (-518.2 + h * (103.4 + h * (-12.79 + h * 0.711)))
    } else {
        -20.774 / t
    } / 3_600.0;
    h + refraction
}

fn day_start_utc() -> i64 {
    local(OCTOBER_8, PST, 0, 0, 0)
}

/// CP-TW-a: San Diego's sunrise and sunset on 2026-10-08 are within ±2 minutes of NOAA's published
/// table (rounded to the minute there, so the bound is 2 minutes plus the table's half minute), and
/// solar noon within ±2 minutes of its 11:36:14.
#[test]
fn san_diego_sunrise_sunset_and_noon_agree_with_noaas_table() {
    let (latitude, longitude) = SAN_DIEGO;
    let events = day_events(latitude, longitude, day_start_utc()).expect("computes");
    let within = |name: &str, found: Option<u32>, noaa: i64, bound: i64| {
        let found = i64::from(found.unwrap_or_else(|| panic!("{name} happens in San Diego")));
        assert!(
            (found - noaa).abs() <= bound,
            "{name}: {found} s after midnight here, NOAA {noaa} s (bound {bound} s)"
        );
        found - noaa
    };
    let rise = within("sunrise", events.sunrise, 5 * 3_600 + 48 * 60, 150);
    let set = within("sunset", events.sunset, 17 * 3_600 + 24 * 60, 150);
    let noon = within(
        "solar noon",
        events.solar_noon,
        11 * 3_600 + 36 * 60 + 14,
        120,
    );
    eprintln!("NOAA-DIFF sunrise {rise} s, sunset {set} s, solar noon {noon} s; events {events:?}");
    // Dawn, rise, noon, set and dusk are in order on an ordinary day.
    let order = [
        events.astronomical_dawn,
        events.civil_dawn,
        events.sunrise,
        events.solar_noon,
        events.sunset,
        events.civil_dusk,
        events.astronomical_dusk,
    ]
    .map(|event| event.expect("an ordinary latitude has every event"));
    assert!(order.windows(2).all(|pair| pair[0] < pair[1]), "{order:?}");
}

/// §16.5: the sun at San Diego, 12:00 local on 2026-10-08, is within ±0.5° of NOAA's model, and so are
/// 09:00 and 15:00 (the bound is the acceptance's; the models differ by far less).
#[test]
fn san_diego_elevation_agrees_with_noaas_model_within_half_a_degree() {
    let (latitude, longitude) = SAN_DIEGO;
    for hour in [9, 12, 15] {
        let at = local(OCTOBER_8, PST, hour, 0, 0);
        let here = f64::from(
            sun_at(latitude, longitude, at)
                .expect("computes")
                .elevation(),
        ) / 1_000.0;
        let noaa = noaa_elevation(latitude.degrees(), longitude.degrees(), at);
        eprintln!("NOAA-ELEV {hour:02}:00 here {here:.3}° NOAA model {noaa:.3}°");
        assert!(
            (here - noaa).abs() <= 0.5,
            "{hour}:00: {here}° here, {noaa}° NOAA"
        );
    }
}

/// §16.8 (a): three exact samples, pinned. They were written once, from the first green run, and are
/// the guard on determinism: a dependency bump or a platform difference that moves one is a
/// **material stop**, not a re-golden.
#[test]
fn three_sun_samples_are_pinned_exactly() {
    let (latitude, longitude) = SAN_DIEGO;
    let noon = sun_at(latitude, longitude, local(OCTOBER_8, PST, 12, 0, 0)).expect("computes");
    let evening = sun_at(latitude, longitude, local(OCTOBER_8, PST, 18, 0, 0)).expect("computes");
    // 78° N, 15° E (Svalbard), 2026-12-21 at 12:00 UTC+01:00.
    let polar = sun_at(
        MicroDegrees::new(78_000_000),
        MicroDegrees::new(15_000_000),
        local(20_808, 3_600, 12, 0, 0),
    )
    .expect("computes");
    eprintln!("PINNED noon {noon:?} evening {evening:?} polar {polar:?}");
    // Written 2026-10-08 from the first green run (macOS arm64, libm 0.2.16, solar-positioning 0.7.0)
    // and held equal on CI's Linux x86_64. A change to any of them is a MATERIAL STOP.
    assert_eq!(
        noon,
        SunSample::new(50_759, 189_418),
        "San Diego 2026-10-08 12:00 −08:00"
    );
    assert_eq!(
        evening,
        SunSample::new(-8_391, 267_971),
        "San Diego 2026-10-08 18:00 −08:00"
    );
    assert_eq!(
        polar,
        SunSample::new(-11_440, 180_458),
        "78° N 15° E 2026-12-21 12:00 +01:00"
    );
}

/// Polar night at 78° N on the winter solstice: no sunrise, sunset or civil twilight, and the light
/// is never day; the sun still transits.
#[test]
fn polar_night_has_no_sunrise() {
    let (latitude, longitude) = (MicroDegrees::new(78_000_000), MicroDegrees::new(15_000_000));
    let start = local(20_808, 3_600, 0, 0, 0);
    let events = day_events(latitude, longitude, start).expect("computes");
    assert_eq!(events.sunrise, None);
    assert_eq!(events.sunset, None);
    assert_eq!(events.civil_dawn, None);
    assert_eq!(events.civil_dusk, None);
    assert!(events.solar_noon.is_some());
    assert_eq!(phase_at(latitude, longitude, start), Ok(Phase::Night));
    eprintln!("POLAR {events:?}");
}

/// At 0° N 0° E on the March equinox the sun transits within a few minutes of 12:00 UTC (the
/// equation of time is about −7 minutes then) nearly overhead, and day and night are about equal.
#[test]
fn the_equator_at_the_equinox_has_a_noon_sun_overhead_and_an_even_day() {
    let origin = MicroDegrees::new(0);
    let march_20 = 20_532; // 2026-03-20
    let start = march_20 * DAY;
    let events = day_events(origin, origin, start).expect("computes");
    let noon = i64::from(events.solar_noon.expect("transits"));
    assert!((noon - 43_200).abs() <= 10 * 60, "solar noon {noon}");
    let overhead = sun_at(origin, origin, start + noon).expect("computes");
    assert!(overhead.elevation() > 88_000, "{overhead:?}");
    let length =
        i64::from(events.sunset.expect("sets")) - i64::from(events.sunrise.expect("rises"));
    assert!((length - 43_200).abs() <= 15 * 60, "day length {length}");
}

/// The azimuth convention: north = 0, clockwise. San Diego's morning sun is in the east (between 45°
/// and 135°), its evening sun in the west (225° … 315°), and its noon sun south (> 90° and < 270°).
#[test]
fn azimuth_is_from_north_clockwise() {
    let (latitude, longitude) = SAN_DIEGO;
    let at = |hh| sun_at(latitude, longitude, local(OCTOBER_8, PST, hh, 0, 0)).expect("computes");
    assert!((45_000..135_000).contains(&at(8).azimuth()), "{:?}", at(8));
    assert!(
        (225_000..315_000).contains(&at(16).azimuth()),
        "{:?}",
        at(16)
    );
    assert!(
        (90_000..270_000).contains(&at(12).azimuth()),
        "{:?}",
        at(12)
    );
}
