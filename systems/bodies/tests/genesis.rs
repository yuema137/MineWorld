//! A place's shape at genesis: the people authored into it must fit it (step-11 SD-B4, QR-7), and
//! whoever perceives the place is told its shape (PB-16).
//!
//! ```text
//! bodies-overlap   two people 420 mm apart
//! bodies-outside   a centre 200 mm from the floor's edge
//! bodies-in-solid  a centre 100 mm inside the counter
//! bodies-capacity  a 1 300 × 1 300 mm room in a world of twelve people: 4 grid points, 45 needed
//! positive         the same people, apart and inside, load; the shape is written exactly as stated
//! ```
//!
//! Each refusal is the owner's, `FactRefusedByOwner { bodies, place-shaped, System { code, detail } }`,
//! and its detail names the people by key, the place by key, and the numbers. The real binary's half
//! of this — `mineworld validate` on copies of `worlds/bodies-yard` — is `tools/cli/tests/bodies_yard.rs`.

mod support;

use mineworld_bodies::{BodiesSystem, PlaceShape, PlaceShaped};
use mineworld_contracts::{Event, Rejection};
use mineworld_kernel::{KernelError, SystemIdentity};
use support::{CLEAR, Plan, Yard, cafe, shape};

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
            assert_eq!(event_type, PlaceShaped::EVENT_TYPE, "at its own fact");
            let detail = detail.expect("a detail");
            println!("refused {code}: {detail}");
            (code.as_str().to_owned(), detail)
        }
        other => panic!("refused by the owner with a system code, not {other:?}"),
    }
}

#[test]
fn two_people_closer_than_the_clearance_are_refused_naming_both_the_place_and_the_distance() {
    let (code, detail) = refused(&Plan::room(
        cafe(),
        &[("alice", (2_000, 3_000)), ("bob", (2_420, 3_000))],
    ));
    assert_eq!(code, "bodies-overlap");
    for word in ["alice", "bob", "room", "420 mm"] {
        assert!(detail.contains(word), "names {word}: {detail}");
    }
}

#[test]
fn a_centre_too_close_to_the_floors_edge_is_refused() {
    let (code, detail) = refused(&Plan::room(cafe(), &[("carol", (200, 3_000))]));
    assert_eq!(code, "bodies-outside");
    for word in ["carol", "room", "(200, 3000)"] {
        assert!(detail.contains(word), "names {word}: {detail}");
    }
}

#[test]
fn a_centre_inside_a_solid_is_refused() {
    // 100 mm inside the counter's south face (y 6 570).
    let (code, detail) = refused(&Plan::room(cafe(), &[("dan", (5_000, 6_670))]));
    assert_eq!(code, "bodies-in-solid");
    for word in ["dan", "room", "0 mm from a solid"] {
        assert!(detail.contains(word), "names {word}: {detail}");
    }
}

#[test]
fn a_floor_that_cannot_hold_the_population_is_refused_with_the_counts() {
    // Twelve people in an unshaped street; a 1 300 mm square room whose 650 mm grid, anchored a
    // radius in, has 2 × 2 points where a person fits, against 4 × 11 + 1 = 45 needed.
    const KEYS: [&str; 12] = [
        "p01", "p02", "p03", "p04", "p05", "p06", "p07", "p08", "p09", "p10", "p11", "p12",
    ];
    let plan = Plan {
        places: vec![
            ("room", Some(shape((0, 0, 1_300, 1_300), &[]))),
            ("street", None),
        ],
        people: KEYS
            .iter()
            .enumerate()
            .map(|(i, key)| {
                (
                    *key,
                    "street",
                    Some((i32::try_from(i).expect("small") * 1_000, 0)),
                )
            })
            .collect(),
        ..Plan::default()
    };
    let (code, detail) = refused(&plan);
    assert_eq!(code, "bodies-capacity");
    for word in ["room", "4 points", "12 people", "needs 45"] {
        assert!(detail.contains(word), "names {word}: {detail}");
    }
}

#[test]
fn people_who_fit_load_and_the_shape_is_written_as_stated() {
    let yard = Yard::new(&Plan::room(
        cafe(),
        // Exactly the clearance apart, 2R − TOL = 495 mm (595 while R was 300): the bound is inclusive.
        &[("alice", (2_000, 3_000)), ("bob", (2_000 + CLEAR, 3_000))],
    ));
    let shape: &PlaceShape = yard
        .world
        .components()
        .get::<PlaceShape>(yard.places["room"].entity_id())
        .expect("the room is shaped");
    assert_eq!(*shape, cafe(), "exactly the stated shape");
    let facts: Vec<&str> = yard
        .genesis
        .iter()
        .map(|fact| fact.event_type().as_str())
        .collect();
    assert_eq!(facts, ["arrived", "arrived", "place-shaped"]);
}

/// PB-16: an observer in the hall is told the hall's shape, as authored literals; the court's shape is
/// not in that observation.
#[test]
fn whoever_perceives_a_place_is_told_its_shape_and_no_other_places() {
    let hall = shape(
        (0, 0, 12_000, 9_000),
        &[((5_000, 4_000, 7_000, 5_000), 750)],
    );
    let court = shape(
        (0, 0, 10_000, 10_000),
        &[((4_700, 4_700, 5_300, 5_300), 3_000)],
    );
    let yard = Yard::new(&Plan {
        places: vec![("hall", Some(hall)), ("court", Some(court))],
        people: vec![
            ("alice", "hall", Some((2_000, 2_000))),
            ("bob", "court", Some((2_000, 2_000))),
        ],
        ..Plan::default()
    });
    let observation = yard.observation("alice");
    let shapes: Vec<(mineworld_contracts::EntityId, serde_json::Value)> = observation
        .entities()
        .iter()
        .flat_map(|entity| entity.components())
        .filter(|record| record.component_type().as_str() == "place-shape")
        .map(|record| (record.entity(), record.payload().clone()))
        .collect();
    println!("place-shape records told to alice: {shapes:?}");
    assert_eq!(
        shapes,
        [(
            yard.places["hall"].entity_id(),
            serde_json::json!({
                "floor": { "min": { "x": 0, "y": 0 }, "max": { "x": 12000, "y": 9000 } },
                "solids": [
                    { "min": { "x": 5000, "y": 4000 }, "max": { "x": 7000, "y": 5000 }, "height": 750 }
                ]
            })
        )],
        "the hall's shape, as authored, and nothing of the court"
    );
}
