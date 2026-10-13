//! Owns "the restated mixer drifted" and "a draw leaves 0 … 999".

use super::{draw, mix, permille};

/// (a) The first three outputs of the reference `splitmix64.c` from seed 0
/// (<https://prng.di.unimi.it/splitmix64.c>, S. Vigna, read 2026-10-09): `mix(0, n)` is exactly the
/// `n`-th output, because the reference's state after `n` steps is `n·γ`.
#[test]
fn mix_is_splitmix64() {
    assert_eq!(mix(0, 1), 0xE220_A839_7B1D_CDAF);
    assert_eq!(mix(0, 2), 0x6E78_9E6A_A1B9_65F4);
    assert_eq!(mix(0, 3), 0x06C4_5D18_8009_454F);
}

#[test]
fn permille_spans_0_to_999_without_leaving_it() {
    assert_eq!(permille(0), 0);
    assert_eq!(permille(u64::MAX), 999);
    assert_eq!(permille(1 << 63), 500);
    let seen: std::collections::BTreeSet<u16> = (0..20_000).map(|day| draw(19, day, 0)).collect();
    assert_eq!(seen.first(), Some(&0));
    assert_eq!(seen.last(), Some(&999));
}
