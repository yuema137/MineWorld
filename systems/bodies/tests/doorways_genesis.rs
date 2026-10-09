//! A doorway point must lie where a person fits (step-11 §19, SD-D5, F-B7; `ARC-39` note 5): at
//! genesis, every doorway point of a shaped place — the `here` of each passage out of it and the
//! `there` of each passage into it — lies inside the floor shrunk by R + GAP (250 + 10 = 260 mm; 310
//! while R was 300) and at least 260 mm from every solid, or the place is refused, `bodies-doorway`,
//! naming both places, the point and the distance.
//!
//! ```text
//! here, 200 mm from the front wall          the café's door where the towns had it  → refused
//! there, 100 mm from the street's façade    a street side the street does not hold   → refused
//! here, inside the counter                                                           → refused
//! here, outside the floor                                                            → refused
//! the bounds                                259 refused, 260 loads; from a solid too
//! here and there 400 mm inside              the towns' new points                    → loads
//! an unshaped place                         nothing to check                          → loads
//! ```
//!
//! The real binary's half — `mineworld validate` on copies of the towns — is
//! `tools/cli/tests/town_bodies.rs` (TD-2).

mod support;

use mineworld_bodies::{BodiesSystem, PlaceShape, PlaceShaped};
use mineworld_contracts::{Event, Rejection};
use mineworld_kernel::{KernelError, SystemIdentity};
use support::{GAP, Plan, R, Side, Yard, cafe, shape};

/// A street 10 m wide whose north façade is y 3 200, like the towns'.
fn street() -> PlaceShape {
    shape((-5_000, -3_000, 5_000, 3_200), &[])
}

/// The café (shaped as `support::cafe`: floor 8 320 × 10 320, the counter (3 860, 6 570)–(8 320,
/// 7 170)) opening onto `street` through one doorway, with alice standing clear of everything.
fn plan(street: Option<PlaceShape>, here: (i32, i32), there: (i32, i32)) -> Plan {
    let doorway: (Side, Side) = (("cafe", here), ("street", there));
    Plan {
        places: vec![("cafe", Some(cafe())), ("street", street)],
        people: vec![("alice", "cafe", Some((5_000, 3_000)))],
        passages: vec![doorway],
        ..Plan::default()
    }
}

/// The refusal a plan's genesis must end in: its code and its detail.
fn refused(plan: &Plan) -> (String, String) {
    let Err(error) = Yard::try_new(plan) else {
        panic!("this genesis must be refused");
    };
    match error {
        KernelError::FactRefusedByOwner {
            system,
            event_type,
            reason: Rejection::System { code, detail },
        } => {
            assert_eq!(system, BodiesSystem::ID, "refused by the owner");
            assert_eq!(event_type, PlaceShaped::EVENT_TYPE, "at the place's shape");
            let detail = detail.expect("a detail");
            println!("refused {code}: {detail}");
            (code.as_str().to_owned(), detail)
        }
        other => panic!("refused by the owner with a system code, not {other:?}"),
    }
}

fn assert_names(detail: &str, words: &[&str]) {
    for word in words {
        assert!(detail.contains(word), "names {word}: {detail}");
    }
}

#[test]
fn a_here_200_mm_from_the_front_wall_is_refused_naming_both_places_the_point_and_the_distance() {
    let (code, detail) = refused(&plan(None, (1_610, 200), (0, 2_800)));
    assert_eq!(code, "bodies-doorway");
    assert_names(
        &detail,
        &[
            "cafe",
            "street",
            "(1610, 200)",
            "200 mm from its floor's edge",
        ],
    );
}

#[test]
fn a_there_100_mm_from_the_facade_is_refused_in_the_place_it_lies_in() {
    let (code, detail) = refused(&plan(Some(street()), (1_610, 400), (0, 3_100)));
    assert_eq!(code, "bodies-doorway");
    assert_names(
        &detail,
        &[
            "street",
            "cafe",
            "(0, 3100) in street",
            "100 mm from its floor's edge",
        ],
    );
}

#[test]
fn a_doorway_inside_a_solid_is_refused() {
    // 130 mm inside the counter's south face (y 6 570).
    let (code, detail) = refused(&plan(None, (5_000, 6_700), (0, 2_800)));
    assert_eq!(code, "bodies-doorway");
    assert_names(&detail, &["cafe", "(5000, 6700)", "0 mm from a solid"]);
}

#[test]
fn a_doorway_outside_the_floor_is_refused() {
    let (code, detail) = refused(&plan(None, (1_610, -50), (0, 2_800)));
    assert_eq!(code, "bodies-doorway");
    assert_names(
        &detail,
        &["cafe", "(1610, -50)", "outside its floor by 50 mm"],
    );
}

#[test]
fn the_bounds_are_a_radius_and_the_gap_inclusive() {
    // The bound is R + GAP = 250 + 10 = 260 mm (310 while R was 300).
    const BOUND: i32 = R + GAP;
    // From the floor's edge: 259 mm is refused, 260 loads.
    let (code, detail) = refused(&plan(None, (1_610, BOUND - 1), (0, 2_800)));
    assert_eq!(code, "bodies-doorway");
    assert_names(&detail, &["259 mm from its floor's edge"]);
    Yard::try_new(&plan(None, (1_610, BOUND), (0, 2_800))).expect("260 mm from the wall loads");

    // From a solid: 259 mm below the counter's south face (y 6 570) is refused, 260 loads.
    let (code, detail) = refused(&plan(None, (5_000, 6_570 - (BOUND - 1)), (0, 2_800)));
    assert_eq!(code, "bodies-doorway");
    assert_names(&detail, &["259 mm from a solid"]);
    Yard::try_new(&plan(None, (5_000, 6_570 - BOUND), (0, 2_800)))
        .expect("260 mm from the counter loads");
}

#[test]
fn doorway_points_400_mm_inside_load_and_an_unshaped_place_has_nothing_to_check() {
    Yard::try_new(&plan(Some(street()), (1_610, 400), (0, 2_800)))
        .expect("the towns' new points load");
    // The street unshaped: its side of the doorway is outside any floor, and nothing is checked.
    Yard::try_new(&plan(None, (1_610, 400), (0, 3_100)))
        .expect("an unshaped street is not checked");
}
