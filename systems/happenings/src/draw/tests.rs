//! T3: owns "the restated mixer drifted", "a draw leaves 0 … 999" and "the draw key is not FNV-1a".

use super::{draw, key, mix, permille};

/// The first three outputs of the reference `splitmix64.c` from seed 0
/// (<https://prng.di.unimi.it/splitmix64.c>), the same vector weather's mixer is pinned to: `mix(0, n)`
/// is exactly the `n`-th output, because the reference's state after `n` steps is `n·γ`.
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
    let seen: std::collections::BTreeSet<u16> =
        (0..20_000).map(|day| draw(7, day, 11, 0)).collect();
    assert_eq!(seen.first(), Some(&0));
    assert_eq!(seen.last(), Some(&999));
}

/// FNV-1a 64's published test vectors (Fowler/Noll/Vo, `isthe.com/chongo/tech/comp/fnv`): the empty
/// string is the offset basis, "a" is `af63dc4c8601ec8c`, "foobar" is `85944171f73967e8`. Parts are
/// joined by a zero byte, so ("a", "b") is not ("ab").
#[test]
fn the_draw_key_is_fnv1a_over_zero_joined_parts() {
    assert_eq!(key(&[""]), 0xCBF2_9CE4_8422_2325);
    assert_eq!(key(&["a"]), 0xAF63_DC4C_8601_EC8C);
    assert_eq!(key(&["foobar"]), 0x8594_4171_F739_67E8);
    assert_ne!(key(&["a", "b"]), key(&["ab"]));
    assert_ne!(
        key(&["random", "street-musician", "street"]),
        key(&["random", "street-musician", "park"])
    );
}
