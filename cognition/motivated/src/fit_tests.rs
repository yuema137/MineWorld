//! `curves_fit_atus_sleeping` (`pr-s35-nc3-motivated-sleep.md` SD-Nc3-4): the record of the two-process
//! defaults. One person with no commute, at `needs`' default rates (60 per hour awake; asleep, 125 per hour at
//! level 300, in proportion to the level — `mineworld_sleep::recovery`, Q-Nc3-5), from a midnight start at the
//! genesis tiredness (700), under the thresholds `sleep_from` and `wake_at`: sleep begins when tiredness reaches
//! `sleep_from` (where `pressure` is at its top) and lasts the model's `minutes_to_wake`, as `motivated`'s
//! `for`. Over 60 days, the last 28 measured, the asleep share is held against the committed ATUS *Sleeping*
//! row's bounds (AC-S35-8): ≥ 850 ‰ at 03:00, ≤ 100 ‰ at 14:00, 7.0 – 9.5 h a day, and at most 1.1 sleeps
//! begun a day. Four cases: free-running, and woken by an alarm at 06:30, 05:30 and 04:30 (the early riser).
//!
//! The defaults are the fit's choice (the grid `H0` 800 … 1 000 step 20, `L0` 0 … 400 step 20, `a` 60 … 340
//! step 40, peak 14:00 … 22:00 hourly; the set closest to the ATUS row, by L1 distance over the 24 hours, among
//! those whose every one-step neighbour also passes). `the_fit_is_not_a_knife_edge` holds that last property.

use mineworld_sleep::recovery::recovered;

use crate::two_process::{self, DAY_MINUTES, DEFAULT, STEP_MINUTES, TwoProcess};

/// `needs`' defaults (`systems/needs/src/interactions.rs`): tiredness per hour awake and rest per hour asleep.
const RISE_PER_HOUR: i64 = 60;
const REST_PER_HOUR: u16 = 125;
/// `needs`' genesis tiredness (`systems/needs/src/section.rs`).
const GENESIS: i64 = 700;
/// The longest sleep the default worlds allow, in minutes.
const LONGEST: i64 = 840;
/// The four cases: no alarm, and alarms at 06:30, 05:30 and 04:30.
const CASES: [(&str, Option<i64>); 4] = [
    ("free-running", None),
    ("alarm 06:30", Some(390)),
    ("alarm 05:30", Some(330)),
    ("alarm 04:30", Some(270)),
];

/// One simulated person under `model`: each sleep as `(from, until)` in minutes since the start.
fn simulate(model: &TwoProcess, days: i64, alarm: Option<i64>) -> Vec<(i64, i64)> {
    let (sleep_from, wake_at) = (model.sleep_from(), model.wake_at());
    let mut tiredness = GENESIS;
    let mut asleep_until: Option<i64> = None;
    let mut sleeps = Vec::new();
    let mut fell: (i64, u16) = (0, 0);
    let mut t = 0;
    while t < days * DAY_MINUTES {
        let minute = t.rem_euclid(DAY_MINUTES);
        if asleep_until.is_some_and(|until| t >= until) {
            asleep_until = None;
        }
        if asleep_until.is_none() && tiredness >= i64::from(sleep_from.at(minute)) {
            let level = u16::try_from(tiredness).expect("0 … 1 000");
            let mut length =
                two_process::minutes_to_wake(level, REST_PER_HOUR, minute, &wake_at, LONGEST);
            if let Some(alarm) = alarm {
                let to_alarm = (alarm - minute).rem_euclid(DAY_MINUTES);
                length = length.min(if to_alarm == 0 { DAY_MINUTES } else { to_alarm });
            }
            let length = length.max(STEP_MINUTES);
            asleep_until = Some(t + length);
            sleeps.push((t, t + length));
            fell = (t, level);
        }
        // As `needs` reads it: asleep, recovered from the level at falling asleep (Q-Nc3-5).
        tiredness = if asleep_until.is_some() {
            i64::from(recovered(
                fell.1,
                REST_PER_HOUR,
                (t + STEP_MINUTES - fell.0) * 60,
            ))
        } else {
            (tiredness + RISE_PER_HOUR * STEP_MINUTES / 60).min(1_000)
        };
        t += STEP_MINUTES;
    }
    sleeps
}

/// The share asleep (‰) at `hour` over the last `last` of `days`, the mean sleep per day in minutes, and the
/// mean count of sleeps begun per day (‰).
fn measure(sleeps: &[(i64, i64)], days: i64, last: i64, hour: i64) -> (i64, i64, i64) {
    let start = (days - last) * DAY_MINUTES;
    let asleep_at = |t: i64| sleeps.iter().any(|&(from, until)| from <= t && t < until);
    let share = (days - last..days)
        .filter(|day| asleep_at(day * DAY_MINUTES + hour * 60))
        .count();
    let share = i64::try_from(share).expect("a count") * 1_000 / last;
    let slept: i64 = sleeps
        .iter()
        .filter(|&&(_, until)| until > start)
        .map(|&(from, until)| until.min(days * DAY_MINUTES) - from.max(start))
        .sum();
    let begun = sleeps.iter().filter(|&&(from, _)| from >= start).count();
    (
        share,
        slept / last,
        i64::try_from(begun).expect("a count") * 1_000 / last,
    )
}

/// The case's verdict under `model`: `Ok` with its numbers, or `Err` with them.
fn judged(model: &TwoProcess, case: &str, alarm: Option<i64>) -> Result<String, String> {
    let (days, last) = (60, 28);
    let sleeps = simulate(model, days, alarm);
    let (at_three, mean, per_day) = measure(&sleeps, days, last, 3);
    let (at_two_pm, _, _) = measure(&sleeps, days, last, 14);
    let clock = |t: i64| {
        let minute = t.rem_euclid(DAY_MINUTES);
        format!("{:02}:{:02}", minute / 60, minute % 60)
    };
    let last_sleep = sleeps
        .last()
        .map(|&(from, until)| format!("{}–{}", clock(from), clock(until)))
        .unwrap_or_default();
    let report = format!(
        "{model:?} {case}: asleep at 03:00 {at_three} ‰, at 14:00 {at_two_pm} ‰, {mean} min a day, \
         {per_day} ‰ sleeps a day, last {last_sleep}"
    );
    let holds =
        at_three >= 850 && at_two_pm <= 100 && (420..=570).contains(&mean) && per_day <= 1_100;
    if holds { Ok(report) } else { Err(report) }
}

#[test]
fn curves_fit_atus_sleeping() {
    for (case, alarm) in CASES {
        let report = judged(&DEFAULT, case, alarm).unwrap_or_else(|report| panic!("{report}"));
        println!("{report}");
    }
}

/// The fit's choice is not an isolated point (the linear recovery's failure, ledger 2026-10-20): every
/// one-step neighbour on the fit's grid passes every case too.
#[test]
fn the_fit_is_not_a_knife_edge() {
    let step = |upper: i64, lower: i64, amplitude: i64, peak: i64| TwoProcess {
        upper: DEFAULT.upper + upper,
        lower: DEFAULT.lower + lower,
        amplitude: DEFAULT.amplitude + amplitude,
        peak_hour: DEFAULT.peak_hour + peak,
    };
    let neighbours = [
        step(20, 0, 0, 0),
        step(-20, 0, 0, 0),
        step(0, 20, 0, 0),
        step(0, -20, 0, 0),
        step(0, 0, 40, 0),
        step(0, 0, -40, 0),
        step(0, 0, 0, 1),
        step(0, 0, 0, -1),
    ];
    let failed: Vec<String> = neighbours
        .iter()
        .flat_map(|model| {
            CASES
                .iter()
                .map(move |(case, alarm)| judged(model, case, *alarm))
        })
        .filter_map(Result::err)
        .collect();
    assert!(failed.is_empty(), "{failed:#?}");
}
