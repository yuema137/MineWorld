//! How sleep pressure recovers while asleep: the two-process model's Process S, in integers
//! (`pr-s35-nc3-motivated-sleep.md` Q-Nc3-5, operator ruling 2026-10-20; amends step-35 I-S35-7).
//!
//! In Borbély's model sleep pressure falls exponentially while asleep: the fall per hour is proportional to
//! the level (A. A. Borbély, *Human Neurobiology* 1 (1982) 195–204; S. Daan, D. G. M. Beersma,
//! A. A. Borbély, *Am. J. Physiol.* 246 (1984) R161–R183). A high level falls fast and a low one slowly, so a
//! sleeper woken early still sheds most of the night's pressure, and a person never sleeps "to zero".
//!
//! Here the level is a tiredness of 0 … 1 000 and the rate is a world's `rest_per_hour`: the fall per hour
//! at the [`REFERENCE`] level. So
//!
//! ```text
//! fall per hour = rest_per_hour × level ÷ REFERENCE
//! level(t)      = level(0) × 2^(−t ÷ half_life),  half_life = ln 2 × REFERENCE ÷ rest_per_hour hours
//! ```
//!
//! At the default 125 per hour the time constant is 2.4 h (half-life 1 h 40 min), and an 8-hour night from
//! 1 000 ends at 35: a fall of 965, 121 per hour on average — the old linear 125 per hour, as the ruling asks.
//!
//! The owner of tiredness is still `needs`; this module holds only the formula, so that `needs` and every
//! controller that predicts a sleep's length compute the same number (the controllers read `sleep`'s types
//! already and never link `needs`). Integer only: `2^(−x)` is a committed table at 1/64 steps, linearly
//! interpolated, so the result is the same bit for bit on every platform.

/// The level at which tiredness falls by exactly `rest_per_hour` per hour asleep.
pub const REFERENCE: i64 = 300;

/// `ln 2`, in millionths.
const LN2_MILLIONTHS: i64 = 693_147;

/// `2^(−i ÷ 64) × 1 000 000`, rounded, for `i` = 0 … 64.
const HALVING: [i64; 65] = [
    1_000_000, 989_228, 978_572, 968_031, 957_603, 947_288, 937_084, 926_990, //
    917_004, 907_126, 897_355, 887_688, 878_126, 868_667, 859_310, 850_053, //
    840_896, 831_838, 822_878, 814_014, 805_245, 796_571, 787_990, 779_502, //
    771_105, 762_799, 754_582, 746_454, 738_413, 730_459, 722_590, 714_807, //
    707_107, 699_490, 691_955, 684_501, 677_128, 669_834, 662_618, 655_481, //
    648_420, 641_435, 634_525, 627_690, 620_929, 614_240, 607_624, 601_078, //
    594_604, 588_198, 581_862, 575_595, 569_394, 563_261, 557_193, 551_191, //
    545_254, 539_380, 533_570, 527_823, 522_137, 516_512, 510_949, 505_445, //
    500_000,
];

/// The half-life of tiredness asleep at `rest_per_hour`, in seconds (at least one).
pub fn half_life_seconds(rest_per_hour: u16) -> i64 {
    let rest = i64::from(rest_per_hour.max(1));
    (LN2_MILLIONTHS * 3_600 * REFERENCE / (rest * 1_000_000)).max(1)
}

/// The tiredness `seconds` after it was `level`, asleep, at `rest_per_hour`. Never below 0; `level` itself
/// for no time or negative time.
pub fn recovered(level: u16, rest_per_hour: u16, seconds: i64) -> u16 {
    if seconds <= 0 {
        return level;
    }
    let half_life = half_life_seconds(rest_per_hour);
    let steps = seconds.saturating_mul(64);
    let whole = steps / half_life / 64;
    if whole >= 16 {
        return 0;
    }
    let index = usize::try_from(steps / half_life % 64).expect("0 … 63");
    let within = steps % half_life;
    let fraction = HALVING[index] - (HALVING[index] - HALVING[index + 1]) * within / half_life;
    let value = (i64::from(level) * fraction / 1_000_000) >> whole;
    u16::try_from(value).expect("at most the level")
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOUR: i64 = 3_600;

    /// The ruling's average: an 8-hour night from 1 000 at the default rate falls about 125 per hour.
    #[test]
    fn a_night_at_the_default_rate_falls_about_125_an_hour() {
        let left = recovered(1_000, 125, 8 * HOUR);
        assert_eq!(left, 35);
        let mean = (1_000 - i64::from(left)) / 8;
        assert!((115..=125).contains(&mean), "{mean}");
    }

    /// Proportional to the level: one half-life halves any level, and at the reference level the first
    /// minutes fall at `rest_per_hour`.
    #[test]
    fn the_fall_is_proportional_to_the_level() {
        let half = half_life_seconds(125);
        assert_eq!(half, 5_988);
        assert_eq!(recovered(1_000, 125, half), 500);
        assert_eq!(recovered(400, 125, half), 200);
        // Six minutes at the reference level: 300 × (1 − e^(−0.1 ÷ 2.4)) ≈ 12.2 (13 after truncation), the
        // linear 12.5.
        assert_eq!(300 - recovered(300, 125, 360), 13);
        // A faster world halves sooner.
        assert!(half_life_seconds(250) < half);
    }

    #[test]
    fn it_never_rises_and_reaches_zero_in_the_end() {
        let mut last = 1_000;
        for minutes in (0..=24 * 60).step_by(15) {
            let now = recovered(1_000, 125, minutes * 60);
            assert!(now <= last, "{minutes}");
            last = now;
        }
        assert_eq!(recovered(1_000, 125, 30 * HOUR), 0);
        assert_eq!(recovered(700, 125, 0), 700);
    }
}
