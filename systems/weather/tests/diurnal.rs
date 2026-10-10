//! The committed diurnal curve is the formula it claims to be (step-19 SD-TW-b-8a).
//!
//! `DIURNAL` is integers precomputed from Parton & Logan's shape; this test evaluates that formula in
//! floating point — here, outside the pack's source, which stays integer-only (`tests/no_float.rs`) —
//! and holds every entry equal. A table edited by hand, or a formula changed in the comment only,
//! fails here.

use mineworld_weather::HOURS;
use mineworld_weather::hours::{DIURNAL, PEAK_HOUR, SUNRISE_HOURS};

/// Parton & Logan's night coefficient at screen height.
const B: f64 = 2.2;

fn curve(sunrise: usize, hour: usize) -> u16 {
    let (s, h, peak) = (sunrise as f64, hour as f64, PEAK_HOUR as f64);
    let value = if (sunrise..=PEAK_HOUR).contains(&hour) {
        1000.0 * (core::f64::consts::FRAC_PI_2 * (h - s) / (peak - s)).sin()
    } else {
        let since_peak = if hour > PEAK_HOUR {
            h - peak
        } else {
            h + 24.0 - peak
        };
        let night = s + 24.0 - peak;
        1000.0 * ((-B * since_peak / night).exp() - (-B).exp()) / (1.0 - (-B).exp())
    };
    value.round() as u16
}

fn table() -> Vec<[u16; HOURS]> {
    (SUNRISE_HOURS.0..=SUNRISE_HOURS.1)
        .map(|sunrise| core::array::from_fn(|hour| curve(sunrise, hour)))
        .collect()
}

#[test]
fn the_committed_curve_is_parton_and_logans_formula() {
    let expected = table();
    for (row, sunrise) in expected.iter().zip(SUNRISE_HOURS.0..) {
        println!("    {row:?}, // sunrise {sunrise:02}:00");
    }
    assert_eq!(DIURNAL.to_vec(), expected);
    for (row, sunrise) in DIURNAL.iter().zip(SUNRISE_HOURS.0..) {
        assert_eq!(row[sunrise], 0, "the minimum is at sunrise {sunrise}");
        assert_eq!(row[PEAK_HOUR], 1000, "the maximum is at 15:00");
        assert!(row.iter().all(|c| *c <= 1000));
    }
}
