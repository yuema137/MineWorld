//! When a person falls asleep and wakes: Borbély's two-process model, as data (`pr-s35-nc3-motivated-sleep.md`
//! SD-Nc3-1, SD-Nc3-3).
//!
//! - A. A. Borbély, "A two process model of sleep regulation", *Human Neurobiology* 1 (1982) 195–204.
//! - S. Daan, D. G. M. Beersma, A. A. Borbély, "Timing of human sleep: recovery process gated by a circadian
//!   pacemaker", *American Journal of Physiology* 246 (1984) R161–R183 (the threshold form used here).
//! - A. A. Borbély, S. Daan, A. Wirz-Justice, T. Deboer, "The two-process model of sleep regulation: a
//!   reappraisal", *Journal of Sleep Research* 25 (2016) 131–143.
//!
//! Process S is sleep pressure: here the tiredness `needs` discloses, which rises while awake and falls while
//! asleep. Process C is one circadian wave `C(t)` that moves two thresholds together: the upper
//! `H(t) = H0 + a·C(t)` and the lower `L(t) = L0 + a·C(t)`. A person falls asleep when S reaches `H(t)` and
//! wakes when S falls to `L(t)`. The thresholds are two curves over the minute of the day, `sleep_from` (H)
//! and `wake_at` (L); a world may override either like any curve.
//!
//! The wave is a cosine with its peak at `peak_hour`, sampled at each whole hour from an integer table, so
//! the curves are the same bit for bit on every platform. The defaults are fitted once, against the committed
//! ATUS *Sleeping* row at `needs`' default rates (`fit_tests.rs`), not per world.

use mineworld_sleep::recovery::recovered;

use crate::curve::Curve;

/// Minutes in a day.
pub const DAY_MINUTES: i64 = 1_440;

/// The last minute of a day, where each threshold curve ends.
pub const LAST_MINUTE: i64 = DAY_MINUTES - 1;

/// `cos(k × 15°) × 1 000`, rounded, for `k` = 0 … 12; the other half of the circle is symmetric.
const COSINE: [i64; 13] = [
    1_000, 966, 866, 707, 500, 259, 0, -259, -500, -707, -866, -966, -1_000,
];

/// The wave `C`, × 1 000, `hours` after its peak.
fn wave(hours: i64) -> i64 {
    let k = hours.rem_euclid(24);
    COSINE[usize::try_from(k.min(24 - k)).expect("0 … 12")]
}

/// The two-process thresholds: the parameters of one circadian wave and its two thresholds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TwoProcess {
    /// `H0`: the upper threshold's mean, a tiredness level.
    pub upper: i64,
    /// `L0`: the lower threshold's mean, a tiredness level.
    pub lower: i64,
    /// `a`: how far the wave moves both thresholds, in tiredness.
    pub amplitude: i64,
    /// The wave's peak, an hour of the day.
    pub peak_hour: i64,
}

/// The fitted defaults (SD-Nc3-4): the values `fit_tests.rs` passes with, committed as numbers.
pub const DEFAULT: TwoProcess = TwoProcess {
    upper: 960,
    lower: 100,
    amplitude: 140,
    peak_hour: 15,
};

impl TwoProcess {
    /// `H(t)`, capped at 1 000 so a fully exhausted person can always sleep.
    pub fn sleep_from(&self) -> Curve {
        self.curve(self.upper)
    }

    /// `L(t)`, floored at 0.
    pub fn wake_at(&self) -> Curve {
        self.curve(self.lower)
    }

    /// `mean + a·C(t)`, clamped to 0 … 1 000, at each whole hour 00:00 … 22:00 and at the day's last minute
    /// (where the wave is all but back at its midnight value): 24 points, the most a curve holds.
    fn curve(&self, mean: i64) -> Curve {
        let level = |hour: i64| {
            let value = mean + self.amplitude * wave(hour - self.peak_hour) / 1_000;
            u32::try_from(value.clamp(0, 1_000)).expect("clamped to 0 … 1 000")
        };
        let mut points: Vec<(i64, u32)> = (0..23).map(|hour| (hour * 60, level(hour))).collect();
        points.push((LAST_MINUTE, level(24)));
        Curve::new(points).expect("24 increasing points in 0 … 1 000")
    }
}

/// The step a sleep length is counted in, in minutes.
pub const STEP_MINUTES: i64 = 15;

/// How long a sleep begun now lasts by the model (SD-Nc3-3): from `minute` (of the day), tiredness
/// `tiredness` recovers at the disclosed `rest_per_hour` the way `needs` recovers it (in proportion to its
/// level, `mineworld_sleep::recovery`, Q-Nc3-5), stepped every 15 minutes, and the sleeper wakes at the first
/// step where it is at or below `wake_at` there. At most `longest` minutes.
pub fn minutes_to_wake(
    tiredness: u16,
    rest_per_hour: u16,
    minute: i64,
    wake_at: &Curve,
    longest: i64,
) -> i64 {
    let mut minutes = 0;
    while minutes < longest {
        minutes += STEP_MINUTES;
        let level = i64::from(recovered(tiredness, rest_per_hour, minutes * 60));
        let at = (minute + minutes).rem_euclid(DAY_MINUTES);
        if level <= i64::from(wake_at.at(at)) {
            break;
        }
    }
    minutes.min(longest)
}
