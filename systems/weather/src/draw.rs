//! The weather's randomness: counter-based, so every draw is a pure function of `(seed, day, index)`
//! and no draw depends on how many came before it (SD-TW-b-7).
//!
//! `mix` is SplitMix64's output function (Steele, Lea & Flood 2014; the reference `splitmix64.c` by
//! S. Vigna, <https://prng.di.unimi.it/splitmix64.c>, read 2026-10-09) applied to `a ^ b·γ`, where γ is
//! the generator's increment. With `a = 0` it is exactly the reference generator's `b`-th output from
//! seed 0, which a test pins. It restates the paced controller's private `mix`
//! (`cognition/rule-controller/src/paced.rs`): a System Pack never depends on cognition.

#[cfg(test)]
mod tests;

/// SplitMix64's increment, the golden ratio's 64-bit fraction.
const GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

/// SplitMix64's finalizer over two words: a pure, well-mixed function, the same on every machine.
pub(crate) const fn mix(a: u64, b: u64) -> u64 {
    let mut z = a ^ b.wrapping_mul(GAMMA);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A uniform draw in `0 … 999` from a 64-bit word, by multiply-shift on its high 32 bits: no modulo
/// bias beyond one part in 2^32, and always below 1000 because `(2^32 − 1) × 1000 >> 32 = 999`.
pub(crate) const fn permille(x: u64) -> u16 {
    #[allow(clippy::cast_possible_truncation)] // < 1000 by the shift above.
    let value = (((x >> 32) * 1_000) >> 32) as u16;
    value
}

/// The draw numbered `index` of world day `day`, per mille.
pub(crate) const fn draw(seed: u64, day: i64, index: u64) -> u16 {
    permille(mix(mix(seed, day.cast_unsigned()), index))
}

/// The fixed draw indices. A new draw takes a new index, so existing draws never shift.
pub(crate) mod index {
    /// Wet or dry.
    pub const WET: u64 = 0;
    /// A wet day's quintile.
    pub const AMOUNT: u64 = 1;
    /// The maximum temperature's noise.
    pub const TMAX_NOISE: u64 = 2;
    /// The minimum temperature's noise.
    pub const TMIN_NOISE: u64 = 3;
    /// Fog.
    pub const FOG: u64 = 4;
    /// Thunder, on a wet day.
    pub const THUNDER: u64 = 5;
    /// Morning overcast, on a dry day.
    pub const OVERCAST: u64 = 6;
    /// Where the first wet run begins.
    pub const FIRST_RUN: u64 = 16;
    /// Where the second wet run begins.
    pub const SECOND_RUN: u64 = 17;
}
