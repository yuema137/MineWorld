//! `curves_fit_atus_sleeping` (`pr-s35-nc3-motivated-sleep.md` SD-Nc3-4): the record of the two-process
//! defaults. One person with no commute, at `needs`' default rates (60 per hour awake, 125 per hour asleep),
//! from a midnight start at the genesis tiredness (700), under the default `sleep_from` and `wake_at`: sleep
//! begins when tiredness reaches `sleep_from` (where `pressure` is at its top) and lasts the model's
//! `minutes_to_wake`, as `motivated`'s `for`. The asleep share at each hour is held against the committed
//! ATUS *Sleeping* row's bounds (AC-S35-8). The early-riser case is the same person woken by an alarm.

use crate::settings::Settings;
use crate::two_process::{self, DAY_MINUTES, STEP_MINUTES};

/// `needs`' defaults (`systems/needs/src/interactions.rs`): tiredness per hour awake and rest per hour asleep.
const RISE_PER_HOUR: i64 = 60;
const REST_PER_HOUR: u16 = 125;
/// `needs`' genesis tiredness (`systems/needs/src/section.rs`).
const GENESIS: i64 = 700;
/// The longest sleep the default worlds allow, in minutes.
const LONGEST: i64 = 840;

/// One simulated person: each sleep as `(from, until)` in minutes since the start.
fn simulate(days: i64, alarm: Option<i64>) -> Vec<(i64, i64)> {
    let settings = Settings::defaults();
    let curves = &settings.curves;
    let mut tiredness = GENESIS;
    let mut asleep_until: Option<i64> = None;
    let mut sleeps = Vec::new();
    let mut t = 0;
    while t < days * DAY_MINUTES {
        let minute = t.rem_euclid(DAY_MINUTES);
        if asleep_until.is_some_and(|until| t >= until) {
            asleep_until = None;
        }
        if asleep_until.is_none() && tiredness >= i64::from(curves.sleep_from.at(minute)) {
            let level = u16::try_from(tiredness).expect("0 … 1 000");
            let mut length = two_process::minutes_to_wake(
                level,
                REST_PER_HOUR,
                minute,
                &curves.wake_at,
                LONGEST,
            );
            if let Some(alarm) = alarm {
                let to_alarm = (alarm - minute).rem_euclid(DAY_MINUTES);
                length = length.min(if to_alarm == 0 { DAY_MINUTES } else { to_alarm });
            }
            let length = length.max(STEP_MINUTES);
            asleep_until = Some(t + length);
            sleeps.push((t, t + length));
        }
        tiredness = if asleep_until.is_some() {
            (tiredness - i64::from(REST_PER_HOUR) * STEP_MINUTES / 60).max(0)
        } else {
            (tiredness + RISE_PER_HOUR * STEP_MINUTES / 60).min(1_000)
        };
        t += STEP_MINUTES;
    }
    sleeps
}

/// The share asleep (‰) at `hour` over the last `last` of `days`, the mean sleep per day in minutes, and the
/// mean count of sleeps begun per day.
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

fn assert_atus_bounds(case: &str, alarm: Option<i64>) {
    let (days, last) = (60, 28);
    let sleeps = simulate(days, alarm);
    let (at_three, mean, per_day) = measure(&sleeps, days, last, 3);
    let (at_two_pm, _, _) = measure(&sleeps, days, last, 14);
    let tail: Vec<String> = sleeps
        .iter()
        .rev()
        .take(4)
        .map(|&(from, until)| {
            let clock = |t: i64| format!("{:02}:{:02}", t.rem_euclid(DAY_MINUTES) / 60, t % 60);
            format!("{}–{}", clock(from), clock(until))
        })
        .collect();
    let report = format!(
        "{case}: asleep at 03:00 {at_three} ‰, at 14:00 {at_two_pm} ‰, {mean} min a day, {per_day} ‰ sleeps a day; last {tail:?}"
    );
    assert!(at_three >= 850, "{report}");
    assert!(at_two_pm <= 100, "{report}");
    assert!((420..=570).contains(&mean), "{report}");
    assert!(per_day <= 1_100, "one consolidated sleep a day: {report}");
}

#[test]
fn curves_fit_atus_sleeping() {
    assert_atus_bounds("free-running", None);
}

#[test]
fn curves_fit_atus_sleeping_for_an_alarm_bound_sleeper() {
    // A shift whose leave-by is 06:30, and an early riser's at 04:30 (§7).
    assert_atus_bounds("alarm 06:30", Some(390));
    assert_atus_bounds("alarm 04:30", Some(270));
}
