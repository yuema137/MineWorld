//! The seeded, counter-based draws every in-process model takes its randomness from.
//!
//! Moved verbatim from `cognition/rule-controller/src/paced.rs` (`pr-dl-a-decision-list.md` SD-DLa-2), so
//! every model shares one well-tested mixer: a draw is a pure function of `(salt, seed, observer,
//! instant, index)`, the same on every machine, with no generator state to persist across a restart
//! (`DECISIONS.md` `ARC-27`). [`Draw::new`] is the rule controller's stream, its salt unchanged; a model
//! that wants an independent stream calls [`Draw::salted`] with its own salt.
//!
//! The golden values in `draw_tests.rs` were printed by the base's private `Draw` before the move
//! (test rules §25), so a change to a draw here is seen there.

use mineworld_contracts::Observation;
use serde_json::Value;

/// The rule controller's salt, unchanged since S7: "mineworl" in ASCII.
const RULE_SALT: u64 = 0x6d69_6e65_776f_726c;

/// The seeded draws for one decision: a pure function of the salt, the seed, the observer and the
/// instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Draw {
    mixed: u64,
}

impl Draw {
    /// The rule controller's draws for `observation` under `seed`.
    pub fn new(seed: u64, observation: &Observation<Value>) -> Self {
        Self::salted(RULE_SALT, seed, observation)
    }

    /// A stream of draws independent of every other salt's, for `observation` under `seed`.
    pub fn salted(salt: u64, seed: u64, observation: &Observation<Value>) -> Self {
        let observer = observation.observer().raw();
        // Two's complement bits of the instant: any i64 is a distinct input.
        let at = observation.at().seconds().cast_unsigned();
        Self {
            mixed: mix(mix(seed ^ salt, observer), at),
        }
    }

    /// The `n`th independent draw, in `0..bound`. `bound` must not be zero.
    pub fn below(&self, bound: u64, n: u64) -> u64 {
        mix(self.mixed, n) % bound
    }

    /// The `n`th draw, in `-reach..=reach`. `reach` must not be negative.
    pub fn offset(&self, reach: i64, n: u64) -> i64 {
        let span = reach.cast_unsigned() * 2 + 1;
        self.below(span, n).cast_signed() - reach
    }
}

/// SplitMix64's finalizer over two words: a pure, well-mixed function, the same on every machine.
pub fn mix(a: u64, b: u64) -> u64 {
    let mut z = a ^ b.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
