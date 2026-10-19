//! The happenings' randomness: counter-based, so every draw is a pure function of `(seed, day, key,
//! index)` and no draw depends on how many came before it (INV-WE-2; design §5.3).
//!
//! `mix` and `permille` restate weather's `draw.rs` exactly — SplitMix64's output function (Steele, Lea
//! & Flood 2014; the reference `splitmix64.c` by S. Vigna, <https://prng.di.unimi.it/splitmix64.c>)
//! applied to `a ^ b·γ`, and a multiply-shift to 0 … 999 — pinned to the same reference vector. A System
//! Pack never depends on another pack's private code or on cognition, so this is a third copy; its
//! extraction into a shared engine-neutral crate is follow-up F-WE-a-8 (QWE-8).
//!
//! An entry's draw key is FNV-1a 64 (Fowler, Noll, Vo; public domain) over its own names, so reordering
//! or adding entries moves no other entry's draws.

#[cfg(test)]
mod tests;

/// SplitMix64's increment, the golden ratio's 64-bit fraction.
const GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

/// FNV-1a 64's offset basis and prime.
const FNV_OFFSET: u64 = 0xCBF2_9CE4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01B3;

/// The draw index of "does this entry fire". A new draw takes a new index, so existing draws never
/// shift.
pub(crate) const DRAW_FIRES: u64 = 0;

/// SplitMix64's finalizer over two words: a pure, well-mixed function, the same on every machine.
pub(crate) const fn mix(a: u64, b: u64) -> u64 {
    let mut z = a ^ b.wrapping_mul(GAMMA);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A uniform draw in `0 … 999` from a 64-bit word, by multiply-shift on its high 32 bits: always below
/// 1000 because `(2^32 − 1) × 1000 >> 32 = 999`.
pub(crate) const fn permille(x: u64) -> u16 {
    #[allow(clippy::cast_possible_truncation)] // < 1000 by the shift above.
    let value = (((x >> 32) * 1_000) >> 32) as u16;
    value
}

/// The draw numbered `index` for `key` on world day `day`, per mille.
pub(crate) const fn draw(seed: u64, day: i64, key: u64, index: u64) -> u16 {
    permille(mix(mix(mix(seed, day.cast_unsigned()), key), index))
}

/// FNV-1a 64 over `parts` joined by zero bytes: `"random" 0x00 kind 0x00 place`.
pub(crate) fn key(parts: &[&str]) -> u64 {
    let mut hash = FNV_OFFSET;
    for (index, part) in parts.iter().enumerate() {
        let separator = (index > 0).then_some(0_u8);
        for byte in separator.into_iter().chain(part.bytes()) {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }
    hash
}
