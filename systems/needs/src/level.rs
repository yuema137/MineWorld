//! A need's level, read lazily: pure integer arithmetic over a stored `(level, at, mode)` and world
//! time (step-35 §5.2; `pr-s35-nb-needs.md` SD-Nb-3; the `ARC-86` pattern).
//!
//! ```text
//! current(l, now, r) = clamp(l.level + rate(r, need, l.mode) × (now − l.at) ÷ 3 600, 0, 1 000)
//! fold(l, at, r)     = Level { level: current(l, at, r), at, mode: l.mode }
//! word(level, [a, b, c]) = ok < a ≤ peckish/sleepy < b ≤ hungry/tired < c ≤ famished/exhausted
//! ```
//!
//! Integer only, `i64` intermediates; the division truncates toward zero (F-Nb-3), so a fold loses at
//! most one unit. Folds happen only at facts — a meal, falling asleep, waking — so the loss is a few
//! units a day, bounded and deterministic. No clock is read here: `now` is the caller's world time.

use mineworld_contracts::WorldTime;
use serde::{Deserialize, Serialize};

/// The fullest deficit: famished, exhausted.
pub const MAX: u16 = 1_000;

/// Seconds in an hour: rates are per hour of world time.
const HOUR: i64 = 3_600;

/// Whether the person was awake or asleep when the level was stored: which rate it moves at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    /// Awake.
    Awake,
    /// Asleep, as `sleep`'s facts said.
    Asleep,
}

/// Which need.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Need {
    /// Hunger.
    Hunger,
    /// Tiredness.
    Tiredness,
}

/// One need as stored: its level at `at`, and the mode it has moved in since.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Level {
    level: u16,
    at: WorldTime,
    mode: Mode,
}

impl Level {
    /// A level of `level` (clamped to [`MAX`]) true at `at`, moving in `mode`.
    pub const fn new(level: u16, at: WorldTime, mode: Mode) -> Self {
        Self {
            level: if level > MAX { MAX } else { level },
            at,
            mode,
        }
    }

    /// The stored level: true at [`at`](Self::at), not now.
    pub const fn stored(&self) -> u16 {
        self.level
    }

    /// The instant the stored level was true.
    pub const fn at(&self) -> WorldTime {
        self.at
    }

    /// The mode it moves in.
    pub const fn mode(&self) -> Mode {
        self.mode
    }

    /// The same level, moving in another mode from now on (only after a fold to the same instant).
    pub const fn in_mode(self, mode: Mode) -> Self {
        Self { mode, ..self }
    }
}

/// The rates in force for one person, per hour, and whether sleep rests them: the parameters a world
/// chose, already looked up (`src/interactions.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rates {
    /// Hunger's rise while awake.
    pub hunger_per_hour: u16,
    /// Hunger's rise while asleep.
    pub hunger_asleep_per_hour: u16,
    /// Tiredness's rise while awake (and asleep when sleep does not rest).
    pub tiredness_per_hour: u16,
    /// Tiredness's fall while asleep, when sleep rests.
    pub rest_per_hour: u16,
    /// Whether `sleeping` restores tiredness in this world.
    pub sleep_rests: bool,
}

impl Rates {
    /// The signed rate, per hour, of `need` in `mode`.
    pub fn rate(&self, need: Need, mode: Mode) -> i64 {
        match (need, mode) {
            (Need::Hunger, Mode::Awake) => i64::from(self.hunger_per_hour),
            (Need::Hunger, Mode::Asleep) => i64::from(self.hunger_asleep_per_hour),
            (Need::Tiredness, Mode::Asleep) if self.sleep_rests => -i64::from(self.rest_per_hour),
            (Need::Tiredness, _) => i64::from(self.tiredness_per_hour),
        }
    }
}

/// `need`'s level at `now`, from its stored `level` and the `rates` in force.
pub fn current(level: &Level, need: Need, now: WorldTime, rates: &Rates) -> u16 {
    let elapsed = now.seconds() - level.at.seconds();
    debug_assert!(elapsed >= 0, "a need is never read before it was stored");
    let elapsed = elapsed.max(0);
    let moved = rates.rate(need, level.mode) * elapsed / HOUR;
    let value = (i64::from(level.level) + moved).clamp(0, i64::from(MAX));
    u16::try_from(value).unwrap_or(MAX)
}

/// The stored level brought to `at`: what a reaction stores before it applies its fact.
pub fn fold(level: &Level, need: Need, at: WorldTime, rates: &Rates) -> Level {
    Level::new(current(level, need, at, rates), at, level.mode)
}

/// A need's word, step-35 §5.4's bands: `0` ok, `1` peckish / sleepy, `2` hungry / tired, `3`
/// famished / exhausted.
pub const fn band(level: u16, thresholds: [u16; 3]) -> u8 {
    if level >= thresholds[2] {
        3
    } else if level >= thresholds[1] {
        2
    } else if level >= thresholds[0] {
        1
    } else {
        0
    }
}

/// How hungry, in the pack's words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HungerWord {
    /// Below the first threshold.
    Ok,
    /// From the first.
    Peckish,
    /// From the second.
    Hungry,
    /// From the third.
    Famished,
}

/// How tired, in the pack's words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TirednessWord {
    /// Below the first threshold.
    Ok,
    /// From the first.
    Sleepy,
    /// From the second.
    Tired,
    /// From the third.
    Exhausted,
}

impl HungerWord {
    /// The word for `level` under `thresholds`.
    pub const fn of(level: u16, thresholds: [u16; 3]) -> Self {
        match band(level, thresholds) {
            0 => Self::Ok,
            1 => Self::Peckish,
            2 => Self::Hungry,
            _ => Self::Famished,
        }
    }
}

impl TirednessWord {
    /// The word for `level` under `thresholds`.
    pub const fn of(level: u16, thresholds: [u16; 3]) -> Self {
        match band(level, thresholds) {
            0 => Self::Ok,
            1 => Self::Sleepy,
            2 => Self::Tired,
            _ => Self::Exhausted,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// step-35 §5.5's defaults, with sleep resting.
    const DEFAULT: Rates = Rates {
        hunger_per_hour: 120,
        hunger_asleep_per_hour: 30,
        tiredness_per_hour: 60,
        rest_per_hour: 125,
        sleep_rests: true,
    };

    const THRESHOLDS: [u16; 3] = [400, 600, 900];

    fn hour(h: i64) -> WorldTime {
        WorldTime::from_seconds(h * HOUR)
    }

    /// AC-Nb-2's flat-and-cafe script, by hand: 00:00 (300, 700); 05:00 awake (900, 1 000); one soup
    /// (600) at 05:00 → 300; asleep at 06:00; at 14:00 hunger 300 + 120 + 30 × 8 = 660, tiredness
    /// 1 000 − 125 × 8 = 0; words hungry / ok.
    #[test]
    fn ac_nb_2_by_hand() {
        let hunger = Level::new(300, hour(0), Mode::Awake);
        let tiredness = Level::new(700, hour(0), Mode::Awake);
        assert_eq!(current(&hunger, Need::Hunger, hour(0), &DEFAULT), 300);
        assert_eq!(current(&tiredness, Need::Tiredness, hour(0), &DEFAULT), 700);
        assert_eq!(current(&hunger, Need::Hunger, hour(5), &DEFAULT), 900);
        assert_eq!(
            current(&tiredness, Need::Tiredness, hour(5), &DEFAULT),
            1_000
        );

        let eaten = fold(&hunger, Need::Hunger, hour(5), &DEFAULT);
        let hunger = Level::new(eaten.stored().saturating_sub(600), hour(5), Mode::Awake);
        assert_eq!(hunger.stored(), 300);

        let hunger = fold(&hunger, Need::Hunger, hour(6), &DEFAULT).in_mode(Mode::Asleep);
        let tiredness = fold(&tiredness, Need::Tiredness, hour(6), &DEFAULT).in_mode(Mode::Asleep);
        assert_eq!((hunger.stored(), tiredness.stored()), (420, 1_000));

        let at_14 = (
            current(&hunger, Need::Hunger, hour(14), &DEFAULT),
            current(&tiredness, Need::Tiredness, hour(14), &DEFAULT),
        );
        assert_eq!(at_14, (660, 0));
        assert_eq!(HungerWord::of(at_14.0, THRESHOLDS), HungerWord::Hungry);
        assert_eq!(TirednessWord::of(at_14.1, THRESHOLDS), TirednessWord::Ok);
    }

    /// Clamped at both ends; a 14-hour sleep from 1 000 reaches 0, not −750.
    #[test]
    fn levels_clamp_at_zero_and_at_one_thousand() {
        let full = Level::new(1_000, hour(0), Mode::Awake);
        assert_eq!(current(&full, Need::Hunger, hour(100), &DEFAULT), 1_000);
        let asleep = Level::new(1_000, hour(0), Mode::Asleep);
        assert_eq!(current(&asleep, Need::Tiredness, hour(14), &DEFAULT), 0);
        assert_eq!(Level::new(5_000, hour(0), Mode::Awake).stored(), 1_000);
    }

    /// F-Nb-3: the division truncates toward zero, rising or falling. Falling at 125/h, one second
    /// moves nothing, 29 s moves −1 (−3 625 ÷ 3 600), 28 s still −0 (−3 500 ÷ 3 600). Rising at
    /// 120/h, 29 s is +0 (3 480 ÷ 3 600), 30 s +1.
    #[test]
    fn folds_truncate_toward_zero() {
        let asleep = Level::new(500, WorldTime::EPOCH, Mode::Asleep);
        let at = WorldTime::from_seconds;
        assert_eq!(current(&asleep, Need::Tiredness, at(1), &DEFAULT), 500);
        assert_eq!(current(&asleep, Need::Tiredness, at(28), &DEFAULT), 500);
        assert_eq!(current(&asleep, Need::Tiredness, at(29), &DEFAULT), 499);
        let awake = Level::new(500, WorldTime::EPOCH, Mode::Awake);
        assert_eq!(current(&awake, Need::Hunger, at(29), &DEFAULT), 500);
        assert_eq!(current(&awake, Need::Hunger, at(30), &DEFAULT), 501);
    }

    /// `restore` without `sleeping`: asleep, tiredness rises at the awake rate; hunger takes its asleep
    /// rate either way.
    #[test]
    fn without_sleeping_as_a_restorer_tiredness_rises_in_sleep() {
        let restless = Rates {
            sleep_rests: false,
            ..DEFAULT
        };
        let asleep = Level::new(500, hour(0), Mode::Asleep);
        assert_eq!(restless.rate(Need::Tiredness, Mode::Asleep), 60);
        assert_eq!(current(&asleep, Need::Tiredness, hour(2), &restless), 620);
        assert_eq!(current(&asleep, Need::Hunger, hour(2), &restless), 560);
    }

    /// Each band's boundary, one either side.
    #[test]
    fn words_change_exactly_at_the_thresholds() {
        for (level, hunger, tiredness) in [
            (0, HungerWord::Ok, TirednessWord::Ok),
            (399, HungerWord::Ok, TirednessWord::Ok),
            (400, HungerWord::Peckish, TirednessWord::Sleepy),
            (599, HungerWord::Peckish, TirednessWord::Sleepy),
            (600, HungerWord::Hungry, TirednessWord::Tired),
            (899, HungerWord::Hungry, TirednessWord::Tired),
            (900, HungerWord::Famished, TirednessWord::Exhausted),
            (1_000, HungerWord::Famished, TirednessWord::Exhausted),
        ] {
            assert_eq!(HungerWord::of(level, THRESHOLDS), hunger, "{level}");
            assert_eq!(TirednessWord::of(level, THRESHOLDS), tiredness, "{level}");
        }
    }
}
