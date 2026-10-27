//! Slot arithmetic on world seconds, shared by the pricing bands and the price clerk
//! (`pr-s31d-reporter-policies.md` §5.3).
//!
//! A model keeps no state (`ARC-27`), so "once per slot" is a window rule: a seat consulted every `pace`
//! seconds holds the instant `s` in exactly one consult, the one at `t` with `t − pace < s ≤ t` — the
//! rule's own answer window. World time 0 is midnight of the calendar's epoch, so `t mod 86 400` is the
//! time of day.

/// Seconds in a day.
pub const DAY: i64 = 86_400;

/// Whether the consult at `t`, consulted every `pace` seconds, holds the instant `s`: `t − pace < s ≤ t`.
pub const fn in_window(t: i64, pace: i64, s: i64) -> bool {
    t - pace < s && s <= t
}

/// Whether a publish slot — minute `first`, `first + every`, … up to `last` of some day — lies in the
/// consult window at `t`. `None`-free: a zero `every` or `pace` holds no slot.
pub fn publish_due(t: i64, pace: i64, first: u32, last: u32, every: u32) -> bool {
    if pace <= 0 || every == 0 || last < first {
        return false;
    }
    let opened = t - pace;
    let days = opened.div_euclid(DAY)..=t.div_euclid(DAY);
    days.into_iter().any(|day| {
        (first..=last)
            .step_by(every as usize)
            .any(|minute| in_window(t, pace, day * DAY + i64::from(minute) * 60))
    })
}

/// The index of the consult at `t` within its period of `every_minutes`: `floor((t mod period) / pace)`.
/// Consecutive consults of one period take consecutive indices, so each index is handled at most once
/// per period, and exactly once when `pace` divides the period. `None` for a zero pace or period.
pub fn period_index(t: i64, pace: i64, every_minutes: u32) -> Option<u64> {
    let period = i64::from(every_minutes) * 60;
    if pace <= 0 || period <= 0 {
        return None;
    }
    u64::try_from(t.rem_euclid(period) / pace).ok()
}
