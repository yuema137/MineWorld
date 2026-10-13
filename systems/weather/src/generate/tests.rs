//! Owns the generator's statistics (criteria 2 and 3), its determinism, and its edge cases. Expected
//! values come from the table's own parameters (the stationary probability of a two-state chain, the
//! mean of a geometric spell length), never from the generator.

use mineworld_calendar::CalendarDate;

use super::day;
use crate::day::{Chain, DailyWeather};
use crate::fixture::{constant, rules, san_diego, san_diego_text};
use crate::rules::Rules;

/// 100 world years, 2026-10-08 onward (step-19 §17.6 criterion 2).
const CENTURY: i64 = 36_524;

/// The days of a run from the market-town epoch: each day's month and summary.
fn run(rules: &Rules, seed: u64, days: i64) -> Vec<(u8, DailyWeather)> {
    let epoch = CalendarDate::new(2026, 10, 8).expect("a date").days();
    let mut chain = Chain::default();
    (0..days)
        .map(|index| {
            let month = CalendarDate::from_days(epoch + index)
                .expect("in range")
                .month();
            let (summary, next) = day(rules, seed, index, month, chain);
            chain = next;
            (month, summary)
        })
        .collect()
}

/// (b) Criterion 2: each month's wet-day frequency over 100 years is within ±3 points (30 per mille)
/// of its stationary probability p_wd / (1000 − p_ww + p_wd).
#[test]
fn each_months_wet_day_frequency_is_its_stationary_probability() {
    let table = san_diego();
    let days = run(&table, 19, CENTURY);
    for month in 1..=12_u8 {
        let m = table.month(month);
        let (p_wd, p_ww) = (m.p_wet_after_dry(), m.p_wet_after_wet());
        let ours: Vec<&DailyWeather> = days
            .iter()
            .filter(|(of, _)| *of == month)
            .map(|(_, summary)| summary)
            .collect();
        let n = i64::try_from(ours.len()).expect("small");
        let wet = i64::try_from(ours.iter().filter(|s| s.wet()).count()).expect("small");
        let denominator = 1_000 - p_ww + p_wd;
        // |wet/n − p_wd/denominator| ≤ 30/1000, in integers.
        let difference = (wet * 1_000 * denominator - p_wd * 1_000 * n).abs();
        println!(
            "month {month:2}: {wet:4} wet of {n:4} = {} ‰, stationary {} ‰",
            wet * 1_000 / n,
            p_wd * 1_000 / denominator
        );
        assert!(
            difference <= 30 * n * denominator,
            "month {month}: {wet} wet of {n}, stationary {p_wd}/{denominator}"
        );
    }
}

/// (c) Criterion 3: on a constant table (p_wd 200, p_ww 500) the mean wet spell is within ±10 % of
/// 1000 / (1000 − p_ww) = 2.0 days. A chain that ignored yesterday would give 1000 / (1000 − p_wd) =
/// 1.25 (M-TWb-3).
#[test]
fn the_mean_wet_spell_is_the_chains() {
    let days = run(&constant(200, 500), 19, CENTURY);
    let (mut wet, mut spells, mut before) = (0_i64, 0_i64, false);
    for (_, summary) in &days {
        if summary.wet() {
            wet += 1;
            if !before {
                spells += 1;
            }
        }
        before = summary.wet();
    }
    println!(
        "{wet} wet days in {spells} spells: mean {}.{:02} days",
        wet / spells,
        wet * 100 / spells % 100
    );
    assert!(
        (18 * spells..=22 * spells).contains(&(wet * 10)),
        "mean wet spell {wet}/{spells} outside 1.8 … 2.2"
    );
}

/// (d) The same inputs give the same bytes; another seed gives another year.
#[test]
fn the_same_seed_gives_the_same_weather_and_another_seed_another() {
    let table = san_diego();
    let bytes = |seed| serde_json::to_vec(&run(&table, seed, 365)).expect("encodes");
    assert_eq!(bytes(19), bytes(19));
    assert_ne!(bytes(19), bytes(20));
    let chain = Chain {
        wet: true,
        tmax_anomaly_dc: 13,
        tmin_anomaly_dc: -7,
    };
    assert_eq!(
        day(&table, 19, 400, 1, chain),
        day(&table, 19, 400, 1, chain)
    );
}

/// P(W|W) = 1000 is legal content — an always-rainy town once it starts — and the chain is absorbing.
#[test]
fn a_certain_wet_after_wet_is_an_absorbing_chain() {
    let days = run(&constant(200, 1_000), 7, 3_650);
    let first = days
        .iter()
        .position(|(_, s)| s.wet())
        .expect("it starts raining");
    assert!(days[first..].iter().all(|(_, s)| s.wet()));
}

/// P(W|D) = P(W|W) is the plain table: independent days, each wet with that probability.
#[test]
fn equal_branches_are_a_plain_independent_table() {
    let days = run(&constant(300, 300), 19, CENTURY);
    let wet = days.iter().filter(|(_, s)| s.wet()).count();
    let after_wet = days
        .windows(2)
        .filter(|pair| pair[0].1.wet())
        .collect::<Vec<_>>();
    let wet_after_wet = after_wet.iter().filter(|pair| pair[1].1.wet()).count();
    let permille = |a: usize, b: usize| a * 1_000 / b;
    assert!((270..=330).contains(&permille(wet, days.len())));
    assert!((270..=330).contains(&permille(wet_after_wet, after_wet.len())));
}

/// Noise that would put the minimum at or above the maximum lowers the minimum; a dry day is 0 mm and
/// a wet day at least 0.3 mm; thunder only when wet, a grey morning only when dry.
#[test]
fn every_day_is_physically_consistent() {
    let tight = rules(
        &san_diego_text()
            .replace("tmin_dc: 102", "tmin_dc: 190")
            .replace("t_noise_dc: 30", "t_noise_dc: 200")
            .replace("t_ar_permille: 600", "t_ar_permille: 999"),
    );
    let mut repaired = 0;
    for (_, s) in run(&tight, 3, CENTURY) {
        assert!(s.tmin_dc < s.tmax_dc, "{s:?}");
        if s.tmin_dc == s.tmax_dc - 1 {
            repaired += 1;
        }
        assert!(s.wet() || s.prcp_tenth_mm == 0);
        assert!(!s.wet() || s.prcp_tenth_mm >= 3);
        assert!(!s.thunder || s.wet());
        assert!(!s.overcast_morning || !s.wet());
        assert!(i64::from(s.tmax_dc).abs() <= 1_200 && i64::from(s.tmin_dc).abs() <= 1_300);
    }
    assert!(repaired > 0, "the repair was exercised");
}
