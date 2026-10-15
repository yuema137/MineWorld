//! `Draw` and `mix` against golden values frozen before the move (test rules §25).
//!
//! Printed on `origin/main @ 6306977` by a temporary `#[cfg(test)]` module placed inside
//! `cognition/rule-controller/src/paced.rs`, calling that file's private `Draw::new`, `Draw::below`,
//! `Draw::offset` and `mix` — the code these replace — and deleted uncommitted
//! (`pr-dl-a-decision-list.md` §15, E-DLa-0). Nothing here is computed by the code under test.

use mineworld_contracts::{EntityId, Observation, WorldTime};
use serde_json::Value;

use crate::draw::{Draw, mix};

/// `(seed, observer, instant, n, bound, below(bound, n), offset(1 400, n))` under the rule's salt.
const GOLDEN: [(u64, u64, i64, u64, u64, u64, i64); 32] = [
    (0x0, 1, 0, 0, 100, 82, 942),
    (0x7, 1, 901, 5, 100, 86, 1083),
    (0x6d69_6e65_776f_726c, 1, -86400, 10, 100, 38, 551),
    (0xffff_ffff_ffff_ffff, 1, 0, 15, 100, 86, -385),
    (0x0, 3, 2_592_017, 4, 100, 93, -906),
    (0x7, 3, -86400, 9, 100, 81, 650),
    (0x6d69_6e65_776f_726c, 3, 901, 14, 100, 50, -280),
    (0xffff_ffff_ffff_ffff, 3, 2_592_017, 3, 100, 26, -975),
    (0x0, 12, 0, 8, 6, 2, 181),
    (0x7, 12, 901, 13, 6, 3, 443),
    (0x6d69_6e65_776f_726c, 12, -86400, 2, 6, 0, 1079),
    (0xffff_ffff_ffff_ffff, 12, 0, 7, 6, 3, -1014),
    (0x0, 4_000_000_007, 2_592_017, 12, 6, 3, 621),
    (0x7, 4_000_000_007, -86400, 1, 6, 3, 657),
    (0x6d69_6e65_776f_726c, 4_000_000_007, 901, 6, 6, 4, -1021),
    (
        0xffff_ffff_ffff_ffff,
        4_000_000_007,
        2_592_017,
        11,
        6,
        1,
        1077,
    ),
    (0x0, 1, 0, 0, 2801, 2342, 942),
    (0x7, 1, 901, 5, 2801, 2483, 1083),
    (0x6d69_6e65_776f_726c, 1, -86400, 10, 2801, 1951, 551),
    (0xffff_ffff_ffff_ffff, 1, 0, 15, 2801, 1015, -385),
    (0x0, 3, 2_592_017, 4, 2801, 494, -906),
    (0x7, 3, -86400, 9, 2801, 2050, 650),
    (0x6d69_6e65_776f_726c, 3, 901, 14, 2801, 1120, -280),
    (0xffff_ffff_ffff_ffff, 3, 2_592_017, 3, 2801, 425, -975),
    (0x0, 12, 0, 8, 1, 0, 181),
    (0x7, 12, 901, 13, 1, 0, 443),
    (0x6d69_6e65_776f_726c, 12, -86400, 2, 1, 0, 1079),
    (0xffff_ffff_ffff_ffff, 12, 0, 7, 1, 0, -1014),
    (0x0, 4_000_000_007, 2_592_017, 12, 1, 0, 621),
    (0x7, 4_000_000_007, -86400, 1, 1, 0, 657),
    (0x6d69_6e65_776f_726c, 4_000_000_007, 901, 6, 1, 0, -1021),
    (
        0xffff_ffff_ffff_ffff,
        4_000_000_007,
        2_592_017,
        11,
        1,
        0,
        1077,
    ),
];

/// `(a, b, mix(a, b))`.
const MIXED: [(u64, u64, u64); 4] = [
    (0x0, 0x0, 0x0),
    (0x1, 0x2, 0xbeeb_8da1_658e_ec67),
    (0xffff_ffff_ffff_ffff, 0x7, 0x8bde_40ab_8762_3c48),
    (0x646f_6f72, 0xc, 0x34b8_b18a_796b_8f85),
];

fn observed(observer: u64, at: i64) -> Observation<Value> {
    Observation::new(EntityId::from_raw(observer), WorldTime::from_seconds(at))
}

#[test]
fn the_rule_s_draws_are_the_base_s_draws() {
    for (seed, observer, at, n, bound, below, offset) in GOLDEN {
        let draw = Draw::new(seed, &observed(observer, at));
        assert_eq!(
            (draw.below(bound, n), draw.offset(1_400, n)),
            (below, offset),
            "seed {seed:#x}, observer {observer}, at {at}, n {n}, bound {bound}"
        );
    }
}

#[test]
fn mix_is_the_base_s_mix() {
    for (a, b, mixed) in MIXED {
        assert_eq!(mix(a, b), mixed, "mix({a:#x}, {b:#x})");
    }
}

/// `Draw::new` is the rule's salt and nothing else: another salt is another stream.
#[test]
fn a_salted_draw_is_a_different_stream() {
    let observation = observed(3, 901);
    let rule = Draw::new(7, &observation);
    assert_eq!(rule, Draw::salted(0x6d69_6e65_776f_726c, 7, &observation));
    let other = Draw::salted(0x6d6f_7469_7661_7465, 7, &observation);
    let differ = (0..16).filter(|n| rule.below(1_000_000, *n) != other.below(1_000_000, *n));
    assert_eq!(
        differ.count(),
        16,
        "an independent salt shares no draw here"
    );
}
