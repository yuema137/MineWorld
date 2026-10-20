use super::*;

/// F-S30c-8 at hand-computed instants. Born at −1 072 915 200 (1936-01-01): at the epoch that is
/// 1 072 915 200 s × 400 / 12 622 780 800 = 33.9993 mean years, so 33 — a mean-year age, not a
/// calendar one (D-c-6). The 35th mean year begins at birth + 35 × 12 622 780 800 / 400 =
/// birth + 1 104 493 320 s.
#[test]
fn age_is_whole_mean_years_floored() {
    let birth = WorldTime::from_seconds(-1_072_915_200);
    assert_eq!(age(birth, WorldTime::from_seconds(0)), 33);
    let thirty_fifth = -1_072_915_200 + 1_104_493_320;
    assert_eq!(age(birth, WorldTime::from_seconds(thirty_fifth - 1)), 34);
    assert_eq!(age(birth, WorldTime::from_seconds(thirty_fifth)), 35);
    assert_eq!(age(birth, WorldTime::from_seconds(-1_072_915_201)), -1);
}

#[test]
fn a_band_is_the_first_threshold_below_else_the_last() {
    let below = [
        Threshold::Quantity(ScaledInt::new(1550, 3)),
        Threshold::Quantity(ScaledInt::new(1850, 3)),
    ];
    let at = |value| band_index(&AttributeValue::Quantity(ScaledInt::new(value, 3)), &below);
    assert_eq!(
        (at(1549), at(1550), at(1849), at(1850)),
        (Some(0), Some(1), Some(1), Some(2))
    );
    let years = AttributeValue::Quantity(ScaledInt::new(34, 0));
    assert_eq!(
        band_index(&years, &[Threshold::Quantity(ScaledInt::new(30_000, 3))]),
        Some(1)
    );
}
