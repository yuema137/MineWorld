//! Words of randomness (`pr-s30d-sampler.md` §5.3): FNV-1a 64 turns text into a 64-bit word, SplitMix64's
//! output function mixes two words, and `ppm` maps a word to parts per million. Integers only, the same
//! on every platform.

/// FNV-1a 64 of `bytes` (offset `0xcbf29ce484222325`, prime `0x100000001b3`).
pub const fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut at = 0;
    while at < bytes.len() {
        hash ^= bytes[at] as u64;
        hash = hash.wrapping_mul(0x0100_0000_01b3);
        at += 1;
    }
    hash
}

/// SplitMix64's output function applied to `a ^ b·γ` (Steele, Lea & Flood 2014; Vigna's
/// `splitmix64.c`): with `a = 0` it is the reference generator's `b`-th output from seed 0. The tree's
/// idiom, restated as `systems/weather/src/draw.rs` restates it (a model crate depends on no pack).
pub const fn mix(a: u64, b: u64) -> u64 {
    let mut z = a ^ b.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// A word as a uniform value in `0 ..= 999 999`, by multiply-shift on its high 32 bits.
pub const fn ppm(word: u64) -> u64 {
    ((word >> 32) * 1_000_000) >> 32
}

/// The world's stream: `mix(fnv1a64(seed), salt)`.
pub const fn stream(seed: &str, salt: u32) -> u64 {
    mix(fnv1a64(seed.as_bytes()), salt as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The published FNV-1a 64 vectors.
    #[test]
    fn fnv1a64_matches_its_test_vectors() {
        assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a64(b"foobar"), 0x8594_4171_f739_67e8);
    }

    /// SplitMix64 seeded at 0: its first outputs (the values `systems/weather` pins too).
    #[test]
    fn mix_from_zero_is_the_reference_generator() {
        assert_eq!(mix(0, 1), 0xe220_a839_7b1d_cdaf);
        assert_eq!(mix(0, 2), 0x6e78_9e6a_a1b9_65f4);
        assert_eq!(mix(0, 3), 0x06c4_5d18_8009_454f);
    }

    #[test]
    fn ppm_spans_zero_to_a_million_exclusive() {
        assert_eq!(ppm(0), 0);
        assert_eq!(ppm(u64::MAX), 999_999);
    }
}
