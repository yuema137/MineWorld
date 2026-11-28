//! Food energy: hunger as a kcal reserve (`docs/DECISIONS.md` `ARC-370`; `pr-fe-a-food-energy-engine.md`
//! §3.1).
//!
//! ```text
//! D                      the person's effective daily use, kcal per 86 400 world seconds; capacity C = D
//! N                      the stored numerator, kcal × 86 400, true at `at`
//! N(t)                   max(0, N(at) − D·(t − at))                     the same rate asleep and awake
//! eat count × K kcal     min(C·86 400, N(t) + K·count·86 400)           excess discarded; K = 0 writes nothing
//! available_kcal         ⌊N/86 400⌋
//! bar                    ⌊1 000·N/(C·86 400)⌋ of 1 000
//! hunger                 ⌊1 000·(C·86 400 − N)/(C·86 400)⌋              the words read it, at or above
//! motivation of K        ⌈1 000·min(K, C)/C⌉, 0 for K = 0               ranks food; never changes N
//! ```
//!
//! Integer only: every product and time difference is `i128` before it is clamped and narrowed, since
//! `count × 100 000 kcal × 86 400` exceeds `u64`. Reading at an instant before `at` is an invariant
//! failure (`None`), never a saturated duration. No clock is read here; no Process, fact or draw.

use mineworld_contracts::WorldTime;
use serde::{Deserialize, Serialize};

/// World seconds in a world day: the period daily use is stated per.
pub const DAY_SECONDS: u32 = 86_400;

/// The largest kcal value a world may state: a daily use, a serving's energy, an initial reserve.
pub const MAX_KCAL: u32 = 100_000;

/// The bar's maximum.
pub const BAR_MAXIMUM: u16 = 1_000;

/// The unit every food-energy number is in.
pub const UNIT: &str = "kcal";

const DAY: i128 = DAY_SECONDS as i128;

/// A person's food-energy reserve as stored: `numerator` kcal × 86 400, true at `at`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reserve {
    numerator: u64,
    at: WorldTime,
}

/// What a reserve reads as at one instant, for one daily use: every number a reader is told.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reading {
    /// Whole kcal available: `⌊N/86 400⌋`.
    pub available_kcal: u32,
    /// The capacity, `C = D`.
    pub capacity_kcal: u32,
    /// The owner's bar: `⌊1 000·N/(C·86 400)⌋`.
    pub bar: u16,
    /// The normalized hunger 0 … 1 000 the words read.
    pub hunger: u16,
}

impl Reserve {
    /// `kcal` whole kcal at `at`: a person at genesis.
    pub const fn holding(kcal: u32, at: WorldTime) -> Self {
        Self {
            numerator: kcal as u64 * DAY_SECONDS as u64,
            at,
        }
    }

    /// The instant the stored numerator was true.
    pub const fn at(&self) -> WorldTime {
        self.at
    }

    /// The stored numerator, kcal × 86 400, at [`at`](Self::at).
    pub const fn numerator(&self) -> u64 {
        self.numerator
    }

    /// The numerator at `now` with daily use `daily`, or `None` when `now` is before `at`.
    pub fn numerator_at(&self, now: WorldTime, daily: u32) -> Option<u64> {
        let elapsed = i128::from(now.seconds()) - i128::from(self.at.seconds());
        if elapsed < 0 {
            return None;
        }
        let left = (i128::from(self.numerator) - i128::from(daily) * elapsed).max(0);
        u64::try_from(left).ok()
    }

    /// The reserve after eating `count` of a kind of `kcal` at `at`, capped at the capacity `daily`:
    /// `None` when `at` is before the stored instant. A kind of 0 kcal is the caller's to skip.
    pub fn fed(self, at: WorldTime, daily: u32, kcal: u32, count: u32) -> Option<Self> {
        let now = i128::from(self.numerator_at(at, daily)?);
        let capacity = i128::from(daily) * DAY;
        let added = i128::from(kcal) * i128::from(count) * DAY;
        let numerator = u64::try_from((now + added).min(capacity)).ok()?;
        Some(Self { numerator, at })
    }

    /// What the reserve reads as at `now` with daily use `daily`, or `None` when `now` is before `at`.
    pub fn reading(&self, now: WorldTime, daily: u32) -> Option<Reading> {
        let numerator = i128::from(self.numerator_at(now, daily)?);
        let capacity = i128::from(daily.max(1)) * DAY;
        let numerator = numerator.min(capacity);
        let narrow = |value: i128| u16::try_from(value).unwrap_or(BAR_MAXIMUM);
        Some(Reading {
            available_kcal: u32::try_from(numerator / DAY).unwrap_or(u32::MAX),
            capacity_kcal: daily,
            bar: narrow(i128::from(BAR_MAXIMUM) * numerator / capacity),
            hunger: narrow(i128::from(BAR_MAXIMUM) * (capacity - numerator) / capacity),
        })
    }
}

/// How strongly a kind of `kcal` per serving motivates a person whose capacity is `capacity`:
/// `⌈1 000·min(K, C)/C⌉`, 0 for 0 kcal. On the unitless scale a controller ranks food by.
pub fn motivation(kcal: u32, capacity: u32) -> u16 {
    if kcal == 0 || capacity == 0 {
        return 0;
    }
    let (kcal, capacity) = (i128::from(kcal.min(capacity)), i128::from(capacity));
    let score = (i128::from(BAR_MAXIMUM) * kcal + capacity - 1) / capacity;
    u16::try_from(score).unwrap_or(BAR_MAXIMUM)
}

/// The unit a serving is measured in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum QuantityUnit {
    /// Grams.
    G,
    /// Millilitres.
    Ml,
}

/// How much one serving is: an amount in a unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Quantity {
    /// Grams or millilitres.
    pub unit: QuantityUnit,
    /// The amount, 1 … 100 000.
    #[schemars(range(min = 1, max = 100_000))]
    pub amount: u32,
}

/// One inventory unit of a food-energy item kind: a whole serving, with a label for people and a
/// quantity. Display text only; never parsed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Serving {
    /// What a serving is called, 1 … 128 bytes, no control characters: "1 bowl", "2 slices".
    #[schemars(length(min = 1, max = 128))]
    pub label: String,
    /// How much it is.
    pub quantity: Quantity,
}

/// The longest serving label, in bytes.
pub const LABEL_MAX_BYTES: usize = 128;

impl Serving {
    /// Why this serving may not be authored, if it may not: an empty, overlong or control-character
    /// label, or an amount outside 1 … 100 000.
    pub fn refusal(&self) -> Option<String> {
        let label = &self.label;
        if label.is_empty() || label.len() > LABEL_MAX_BYTES {
            return Some(format!(
                "a serving's label is 1 … {LABEL_MAX_BYTES} bytes, not {}",
                label.len()
            ));
        }
        if label.chars().any(char::is_control) {
            return Some("a serving's label has no control characters".to_owned());
        }
        if !(1..=MAX_KCAL).contains(&self.quantity.amount) {
            return Some(format!(
                "a serving's amount is 1 … {MAX_KCAL}, not {}",
                self.quantity.amount
            ));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(hours: i64) -> WorldTime {
        WorldTime::from_seconds(hours * 3_600)
    }

    /// NE-2's literal oracle, D = C = 2 000, initial 1 500: 06:00 → 1 000; +500 → 1 500; 12:00
    /// → 1 000; a full 2 000 is empty after 24 h and stays at zero.
    #[test]
    fn ne_2_literal_oracle() {
        let start = Reserve::holding(1_500, at(0));
        assert_eq!(
            start.reading(at(6), 2_000).map(|r| r.available_kcal),
            Some(1_000)
        );
        let fed = start.fed(at(6), 2_000, 500, 1).expect("forward");
        assert_eq!(
            fed.reading(at(6), 2_000).map(|r| r.available_kcal),
            Some(1_500)
        );
        assert_eq!(
            fed.reading(at(12), 2_000).map(|r| r.available_kcal),
            Some(1_000)
        );
        let full = Reserve::holding(2_000, at(0));
        let empty = full.reading(at(24), 2_000).expect("forward");
        assert_eq!(
            (empty.available_kcal, empty.bar, empty.hunger),
            (0, 0, 1_000)
        );
        let later = full.reading(at(48), 2_000).expect("forward");
        assert_eq!(later.available_kcal, 0, "zero stays zero");
        let start_reading = full.reading(at(0), 2_000).expect("now");
        assert_eq!((start_reading.bar, start_reading.hunger), (1_000, 0));
        assert_eq!(
            start.reading(at(0), 2_000).map(|r| (r.bar, r.hunger)),
            Some((750, 250))
        );
    }

    /// Fractional seconds are kept: one second of 2 000 kcal/day is 2 000 numerator units, so 43 s
    /// shows 1 499 kcal (1 500·86 400 − 86 000 = 129 514 000 → ⌊…/86 400⌋ = 1 499), and 0 s none lost.
    /// Split equals unsplit: folding at a meal of 0 net change and reading on gives the unsplit value.
    #[test]
    fn fractional_seconds_survive_and_split_equals_unsplit() {
        let start = Reserve::holding(1_500, WorldTime::EPOCH);
        let second = WorldTime::from_seconds;
        assert_eq!(start.numerator_at(second(43), 2_000), Some(129_514_000));
        assert_eq!(
            start.reading(second(43), 2_000).map(|r| r.available_kcal),
            Some(1_499)
        );
        assert_eq!(
            start.reading(second(1), 2_000).map(|r| r.available_kcal),
            Some(1_499)
        );
        let folded = Reserve {
            numerator: start.numerator_at(second(12_345), 2_000).expect("forward"),
            at: second(12_345),
        };
        for later in [12_345, 20_000, 50_001, 86_400] {
            assert_eq!(
                folded.numerator_at(second(later), 2_000),
                start.numerator_at(second(later), 2_000),
                "split = unsplit at {later}"
            );
        }
    }

    /// The cap discards excess, zero discards deficit, two meals at one instant both count, and the
    /// widest authored product (count u32::MAX × 100 000 kcal × 86 400) does not overflow.
    #[test]
    fn cap_zero_same_instant_and_widest_products() {
        let half = Reserve::holding(1_000, at(0));
        let capped = half.fed(at(0), 2_000, 1_500, 1).expect("forward");
        assert_eq!(capped.numerator(), 2_000 * 86_400, "capped at C");
        let twice = half
            .fed(at(0), 2_000, 300, 1)
            .and_then(|once| once.fed(at(0), 2_000, 300, 1))
            .expect("forward");
        assert_eq!(
            twice.reading(at(0), 2_000).map(|r| r.available_kcal),
            Some(1_600)
        );
        let widest = Reserve::holding(0, at(0))
            .fed(at(0), MAX_KCAL, MAX_KCAL, u32::MAX)
            .expect("forward");
        assert_eq!(widest.numerator(), u64::from(MAX_KCAL) * 86_400);
        let empty = Reserve::holding(5, at(0));
        assert_eq!(empty.numerator_at(at(10_000), MAX_KCAL), Some(0));
    }

    /// Reading or feeding before the stored instant is an invariant failure, not zero elapsed time.
    #[test]
    fn backward_time_is_refused() {
        let later = Reserve::holding(1_000, at(5));
        assert_eq!(later.numerator_at(at(4), 2_000), None);
        assert_eq!(later.reading(at(4), 2_000), None);
        assert_eq!(later.fed(at(4), 2_000, 100, 1), None);
    }

    /// `⌈1 000·min(K, C)/C⌉`: 0 for water, capped at C, rounded up.
    #[test]
    fn motivation_ranks_food_on_the_unitless_scale() {
        assert_eq!(motivation(0, 2_000), 0);
        assert_eq!(motivation(1, 2_000), 1);
        assert_eq!(motivation(500, 2_000), 250);
        assert_eq!(motivation(501, 2_000), 251);
        assert_eq!(motivation(2_400, 2_000), 1_000);
        assert_eq!(motivation(600, 2_400), 250);
    }

    #[test]
    fn a_serving_is_refused_for_its_label_or_amount() {
        let serving = |label: &str, amount| Serving {
            label: label.to_owned(),
            quantity: Quantity {
                unit: QuantityUnit::G,
                amount,
            },
        };
        assert_eq!(serving("1 bowl", 300).refusal(), None);
        assert!(serving("", 300).refusal().is_some());
        assert!(serving(&"x".repeat(129), 300).refusal().is_some());
        assert_eq!(serving(&"x".repeat(128), 300).refusal(), None);
        assert!(serving("a\tb", 300).refusal().is_some());
        assert!(serving("cup", 0).refusal().is_some());
        assert!(serving("cup", 100_001).refusal().is_some());
    }
}
