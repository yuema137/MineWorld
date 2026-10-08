//! Loose objects at genesis (step-11 §18.4 PO-2's pack half, PO-8, PO-16's first bullet; SD-O4,
//! SD-O5): an item's `body-formed` is reduced into its shape and states `object-placed`, which is
//! reduced in the next generation under SD-O5's checks, in order.
//!
//! ```text
//! bodies-unshaped           a ball in a place without a shape
//! bodies-held-kind          a ball that is also a declared kind (the item pack installed)
//! bodies-objects-max        a 33rd object in one place (32 are placed)
//! bodies-object-outside     a ball 50 mm from the floor's edge
//! bodies-object-in-solid    a box overlapping the counter
//! bodies-object-overlap     two balls 100 mm apart
//! bodies-object-on-person   a ball 200 mm from a person
//! bodies-capacity           a room that holds its one person, but not with an object as well
//! positive                  objects lie where authored, at their half-height; two in one place in
//!                           key order; the place discloses its listing, and nothing of another's
//! ```
//!
//! The scenario room is 12b's café: floor (0, 0)–(8 320, 10 320), the counter (3 860, 6 570)–(8 320,
//! 7 170), 1 100 mm high. Every position is a literal from the plan (test rules §25).

mod support;

use mineworld_bodies::{BodiesSystem, ObjectPlaced};
use mineworld_contracts::{Event, Rejection};
use mineworld_kernel::{KernelError, SystemIdentity};
use support::{Plan, Thing, Yard, ball, cafe, cube, shape};

/// The café, with `people` and `objects` in it.
fn cafe_with(people: &[(&'static str, (i32, i32))], objects: Vec<Thing>) -> Plan {
    Plan {
        objects,
        ..Plan::room(cafe(), people)
    }
}

/// The refusal a plan's genesis must end in, at `object-placed`: its code and its detail.
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
            assert_eq!(event_type, ObjectPlaced::EVENT_TYPE, "at object-placed");
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
fn objects_lie_where_authored_and_the_place_discloses_them() {
    let plan = Plan {
        places: vec![("room", Some(cafe())), ("yard", Some(cafe()))],
        people: vec![("alice", "room", Some((2_000, 3_000)))],
        objects: vec![
            ("ball", ball(110), "room", (4_000, 3_000)),
            ("crate", cube(200), "room", (2_000, 5_000)),
            ("far", ball(110), "yard", (1_000, 1_000)),
        ],
        ..Plan::default()
    };
    let yard = Yard::new(&plan);
    assert_eq!(
        yard.objects("room"),
        [
            ("ball", (4_000, 3_000, 110)),
            ("crate", (2_000, 5_000, 200))
        ],
        "both lie in the room, in key order, at their half-heights"
    );
    assert_eq!(yard.objects("yard"), [("far", (1_000, 1_000, 110))]);

    // PO-8: alice, in the room, is told the room's listing — and nothing of the yard's.
    let observation = serde_json::to_value(yard.observation("alice")).expect("encodes");
    let text = observation.to_string();
    let listings: Vec<serde_json::Value> = observation["entities"]
        .as_array()
        .expect("entities")
        .iter()
        .flat_map(|entity| entity["components"].as_array().cloned().unwrap_or_default())
        .filter(|record| record["component_type"] == "loose-objects")
        .collect();
    assert_eq!(listings.len(), 1, "exactly one listing, the room's: {text}");
    let record = &listings[0];
    let objects = record["payload"].clone();
    let expected = serde_json::json!([
        {
            "object": serde_json::to_value(yard.items["ball"]).expect("an id"),
            "shape": { "ball": 110 },
            "at": { "x": 4000, "y": 3000, "z": 110 }
        },
        {
            "object": serde_json::to_value(yard.items["crate"]).expect("an id"),
            "shape": { "box": { "x": 200, "y": 200, "z": 200 } },
            "at": { "x": 2000, "y": 5000, "z": 200 }
        }
    ]);
    assert_eq!(objects, expected, "the listing, as authored: {record}");
    println!("disclosed: {objects}");
    let far = serde_json::to_value(yard.items["far"])
        .expect("an id")
        .to_string();
    assert!(
        !text.contains(&far),
        "nothing of the yard's object {far}: {text}"
    );
}

#[test]
fn an_object_in_a_place_without_a_shape_is_refused() {
    let plan = Plan {
        places: vec![("room", Some(cafe())), ("street", None)],
        people: vec![("alice", "room", Some((2_000, 3_000)))],
        objects: vec![("ball", ball(110), "street", (1_000, 1_000))],
        ..Plan::default()
    };
    let (code, detail) = refused(&plan);
    assert_eq!(code, "bodies-unshaped");
    assert_names(&detail, &["ball", "street"]);
}

#[test]
fn an_object_that_is_also_a_declared_kind_is_refused() {
    let plan = Plan {
        kinds: vec!["ball"],
        ..cafe_with(
            &[("alice", (2_000, 3_000))],
            vec![("ball", ball(110), "room", (4_000, 3_000))],
        )
    };
    let (code, detail) = refused(&plan);
    assert_eq!(code, "bodies-held-kind");
    assert_names(&detail, &["ball", "item:"]);
}

#[test]
fn an_object_beside_a_declared_kind_lies_as_authored() {
    // The item pack installed, a plain kind declared, the ball not one: placed.
    let plan = Plan {
        kinds: vec!["lantern"],
        ..cafe_with(
            &[("alice", (2_000, 3_000))],
            vec![("ball", ball(110), "room", (4_000, 3_000))],
        )
    };
    let yard = Yard::new(&plan);
    assert_eq!(yard.objects("room"), [("ball", (4_000, 3_000, 110))]);
}

/// 33 balls of radius 50 on a 300 mm grid: 11 across, 3 deep, far from alice.
fn balls(count: usize) -> Vec<Thing> {
    const KEYS: [&str; 33] = [
        "b00", "b01", "b02", "b03", "b04", "b05", "b06", "b07", "b08", "b09", "b10", "b11", "b12",
        "b13", "b14", "b15", "b16", "b17", "b18", "b19", "b20", "b21", "b22", "b23", "b24", "b25",
        "b26", "b27", "b28", "b29", "b30", "b31", "b32",
    ];
    (0..count)
        .map(|i| {
            let (column, row) = (
                i32::try_from(i % 11).expect("small"),
                i32::try_from(i / 11).expect("small"),
            );
            (
                KEYS[i],
                ball(50),
                "room",
                (500 + 300 * column, 1_000 + 300 * row),
            )
        })
        .collect()
}

#[test]
fn a_place_holds_thirty_two_objects_and_refuses_a_thirty_third() {
    let yard = Yard::new(&cafe_with(&[("alice", (7_000, 9_000))], balls(32)));
    assert_eq!(yard.objects("room").len(), 32);
    let (code, detail) = refused(&cafe_with(&[("alice", (7_000, 9_000))], balls(33)));
    assert_eq!(code, "bodies-objects-max");
    assert_names(&detail, &["b32", "room", "32"]);
}

#[test]
fn a_footprint_beyond_the_floor_is_refused() {
    // A ball r 110 whose centre is 50 mm from the west wall: its footprint reaches x = −60.
    let (code, detail) = refused(&cafe_with(
        &[("alice", (2_000, 3_000))],
        vec![("ball", ball(110), "room", (50, 3_000))],
    ));
    assert_eq!(code, "bodies-object-outside");
    assert_names(&detail, &["ball", "room", "(50, 3000)"]);
}

#[test]
fn a_footprint_meeting_a_solid_is_refused() {
    // A box half 200 at (5 000, 6 500): its footprint reaches y = 6 700, inside the counter (6 570).
    let (code, detail) = refused(&cafe_with(
        &[("alice", (2_000, 3_000))],
        vec![("crate", cube(200), "room", (5_000, 6_500))],
    ));
    assert_eq!(code, "bodies-object-in-solid");
    assert_names(&detail, &["crate", "room", "(5000, 6500)"]);
}

#[test]
fn two_overlapping_objects_are_refused() {
    let (code, detail) = refused(&cafe_with(
        &[("alice", (2_000, 3_000))],
        vec![
            ("a-ball", ball(110), "room", (4_000, 3_000)),
            ("b-ball", ball(110), "room", (4_100, 3_000)),
        ],
    ));
    assert_eq!(code, "bodies-object-overlap");
    assert_names(
        &detail,
        &["b-ball", "a-ball", "(4100, 3000)", "(4000, 3000)"],
    );
}

#[test]
fn an_object_under_a_person_is_refused() {
    // 200 mm from alice's centre: inside her 300 mm disc.
    let (code, detail) = refused(&cafe_with(
        &[("alice", (2_000, 3_000))],
        vec![("ball", ball(110), "room", (2_200, 3_000))],
    ));
    assert_eq!(code, "bodies-object-on-person");
    assert_names(&detail, &["ball", "alice", "(2000, 3000)", "room"]);
}

#[test]
fn a_room_that_cannot_hold_its_people_with_its_objects_is_refused() {
    // A 1 300 mm square room: its 650 mm grid, anchored a radius in, has 2 × 2 = 4 points. One
    // person needs 1; the ball r 50 can cover up to 2 × 2 = 4 more: 5 > 4.
    let plan = Plan::room(shape((0, 0, 1_300, 1_300), &[]), &[("solo", (350, 350))]);
    let plan = Plan {
        objects: vec![("ball", ball(50), "room", (1_000, 1_000))],
        ..plan
    };
    let (code, detail) = refused(&plan);
    assert_eq!(code, "bodies-capacity");
    assert_names(
        &detail,
        &["room", "4 points", "1 people", "1 objects", "needs 5"],
    );
}
