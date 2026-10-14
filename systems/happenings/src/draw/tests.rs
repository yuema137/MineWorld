//! T3: owns "the draw key is not FNV-1a".

use super::key;

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
