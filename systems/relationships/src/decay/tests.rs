//! FA-4: the curve arithmetic against literal oracles from hand arithmetic, the table, the host clamp,
//! and monotonicity (`pr-fg-a-relationship-forgetting.md` §5.1, §6).

use mineworld_contracts::{SimDuration, SystemId};

use super::{Curve, CurveName, DAY, Decay, DecayCurve, HALF};

fn days(n: i64) -> SimDuration {
    SimDuration::from_seconds(n * DAY)
}

fn seconds(n: i64) -> SimDuration {
    SimDuration::from_seconds(n)
}

/// The excess left by `curve` for a value of 1 000 over a floor of 0.
fn left(curve: Curve, elapsed: SimDuration, time_constant: SimDuration) -> u32 {
    curve.remaining(1_000, elapsed, time_constant)
}

#[test]
fn half_life_halves_at_every_whole_step_and_reads_the_table_between() {
    let t = days(30);
    assert_eq!(left(Curve::HalfLife, days(30), t), 500);
    assert_eq!(left(Curve::HalfLife, days(60), t), 250);
    // 1 000 × 46 341 >> 16 = 707.1…
    assert_eq!(left(Curve::HalfLife, days(15), t), 707);
    // At the table's own scale the step is read exactly: 65 536 × 46 341 >> 16 = 46 341.
    assert_eq!(Curve::HalfLife.remaining(65_536, days(15), t), 46_341);
    assert_eq!(left(Curve::HalfLife, days(30 * 16), t), 0);
    assert_eq!(left(Curve::HalfLife, seconds(0), t), 1_000);
}

#[test]
fn linear_loses_the_whole_range_in_one_time_constant() {
    let t = days(10);
    assert_eq!(left(Curve::Linear, days(5), t), 500);
    assert_eq!(left(Curve::Linear, days(10), t), 0);
    assert_eq!(left(Curve::Linear, days(20), t), 0);
}

#[test]
fn power_is_t_over_t_plus_elapsed() {
    let t = days(7);
    assert_eq!(left(Curve::Power, days(7), t), 500);
    assert_eq!(left(Curve::Power, days(21), t), 250);
}

#[test]
fn none_keeps_everything() {
    assert_eq!(left(Curve::None, days(3_650), days(1)), 1_000);
}

#[test]
fn floors_keep_their_part_and_regard_keeps_its_sign() {
    let half_life = |floor| Decay {
        curve: Curve::HalfLife,
        time_constant: days(1),
        floor,
    };
    // Familiarity 600 over a floor of 200: the excess 400 halves to 200, so 400.
    assert_eq!(half_life(200).familiarity(600, days(1)), 400);
    // Regard −800, floor 100: the excess 700 halves to 350, so −(100 + 350).
    assert_eq!(half_life(100).regard(-800, days(1)), -450);
    assert_eq!(half_life(100).regard(800, days(1)), 450);
    // At or below the floor nothing fades.
    assert_eq!(half_life(200).familiarity(150, days(100)), 150);
    assert_eq!(half_life(100).regard(-60, days(100)), -60);
}

#[test]
fn the_table_has_its_literal_ends_and_middle_and_strictly_decreases() {
    assert_eq!((HALF[0], HALF[32], HALF[64]), (65_536, 46_341, 32_768));
    assert!(HALF.windows(2).all(|pair| pair[0] > pair[1]));
}

#[test]
fn every_bundled_curve_is_non_increasing_and_never_above_the_excess() {
    for curve in [Curve::None, Curve::Linear, Curve::HalfLife, Curve::Power] {
        for excess in [0, 1, 7, 500, 1_000] {
            for time_constant in [days(1), days(7), days(3_650)] {
                let step = time_constant.seconds() / 7;
                let mut previous = excess;
                for k in 0..=140 {
                    let now = curve.remaining(excess, seconds(k * step), time_constant);
                    assert!(
                        now <= previous && now <= excess,
                        "excess {excess}, T {time_constant}, step {k}: {now} after {previous}"
                    );
                    previous = now;
                }
            }
        }
    }
}

/// A curve that breaks its contract: more than it was given, and nothing at all at once.
struct Hostile {
    give_back: bool,
}

impl DecayCurve for Hostile {
    fn name(&self) -> CurveName {
        CurveName::from_static("hostile")
    }

    fn remaining(&self, excess: u32, _: SimDuration, _: SimDuration) -> u32 {
        if self.give_back { excess + 5_000 } else { 0 }
    }

    fn curve_of(&self) -> SystemId {
        SystemId::from_static("test-hostile")
    }
}

static GREEDY: Hostile = Hostile { give_back: true };
static EAGER: Hostile = Hostile { give_back: false };

/// The host clamps whatever a curve returns to `[floor, value]`: a value never rises, and never falls
/// below its floor — even at no elapsed time (`ARC-86`).
#[test]
fn the_host_clamps_a_curve_to_the_floor_and_the_value() {
    let decay = |curve: &'static Hostile, floor| Decay {
        curve: Curve::Catalog(curve),
        time_constant: days(1),
        floor,
    };
    assert_eq!(decay(&GREEDY, 0).familiarity(600, days(3)), 600);
    assert_eq!(decay(&GREEDY, 100).regard(-700, days(3)), -700);
    assert_eq!(decay(&EAGER, 250).familiarity(600, seconds(0)), 250);
    assert_eq!(decay(&EAGER, 100).regard(-700, seconds(0)), -100);
}

/// A name's form is checked when it is read; the bundled names are of that form.
#[test]
fn a_curve_name_is_lowercase_letters_digits_and_hyphens_up_to_32_bytes() {
    assert!(CurveName::new("step-down-2").is_ok());
    assert!(CurveName::new(&"a".repeat(32)).is_ok());
    for bad in ["", "Half-Life", "half life", "x_y", &"a".repeat(33)] {
        assert!(CurveName::new(bad).is_err(), "'{bad}'");
    }
}
