//! SD-Z4 — a stride only the floor's edge can stop is answered by integers (step-11 §20.3, TZ-5).
//!
//! Through the real `World::dispatch`, in the prototype's café: floor (0, 0)–(8 320, 10 320), counter
//! (3 860, 6 570)–(8 320, 7 170). `C`, the centre's free region against the floor alone, is the floor
//! shrunk by R + GAP = 310: (310, 310)–(8 010, 10 010). Every literal below is exact, by hand: a
//! Walled stride ends at its target clamped into `C`. `Route::Walled` is the resolver's own account
//! (`explain`, asked before the request); it is returned before any scene is built.

mod support;

use mineworld_bodies::{Route, explain};
use mineworld_contracts::{EntityType, Location, PersonId};
use support::{Fact, Plan, Yard, cafe, xy};

fn route(yard: &Yard, key: &str, to: Location) -> Route {
    let person = PersonId::new(yard.people[key], EntityType::Person).expect("a person");
    explain(&yard.world.read(), person, to)
        .expect("an arrival into a shaped place")
        .route
}

/// The walker's stride to `to`, foreseen as Walled and ending exactly at `end`, stopped short by
/// nothing (the wall).
fn walled(start: (i32, i32), to: (i32, i32), end: (i32, i32)) {
    let mut yard = Yard::new(&Plan::room(cafe(), &[("walker", start)]));
    let wanted = yard.at("room", to);
    assert_eq!(
        route(&yard, "walker", wanted),
        Route::Walled,
        "the integer route"
    );
    let moved = yard.walk("walker", wanted);
    assert!(moved.accepted(), "{:?}", moved.result);
    moved.assert_caused_by_the_request();
    let facts = moved.facts();
    let Fact::Arrived(_, reached) = facts[0] else {
        panic!("the walker's arrival first: {facts:?}");
    };
    assert_eq!(xy(reached), end, "{start:?} → {to:?}");
    assert_eq!(
        facts.last(),
        Some(&Fact::StoppedShort {
            person: yard.people["walker"],
            wanted,
            reached,
            by: None,
        }),
        "stopped short by the wall: {facts:?}"
    );
}

/// TZ-5 a: east into the wall — 8 320 − 300 − 10, PB-5 a's stop, exactly. M-Z4 (C shrunk by R alone)
/// ends at 8 020.
#[test]
fn a_stride_into_the_east_wall_ends_at_8010_by_integers() {
    walled((7_000, 3_000), (8_500, 3_000), (8_010, 3_000));
}

/// TZ-5 b: diagonally into the wall, sliding: x stops at 8 010, y keeps all of its motion.
#[test]
fn a_diagonal_into_the_wall_keeps_its_slide() {
    walled((7_000, 3_000), (8_400, 4_400), (8_010, 4_400));
}

/// TZ-5 c: into the north-east corner, north of the counter.
#[test]
fn a_stride_into_the_corner_stops_in_it() {
    walled((7_200, 9_200), (8_500, 10_500), (8_010, 10_010));
}

/// TZ-5 d: the same stride as a, with somebody 600 mm off its box (inside 2R + GAP = 610): Rapier's,
/// as before.
#[test]
fn a_person_near_the_box_sends_the_stride_to_rapier() {
    let yard = Yard::new(&Plan::room(
        cafe(),
        &[("walker", (7_000, 3_000)), ("other", (7_500, 3_600))],
    ));
    let wanted = yard.at("room", (8_500, 3_000));
    assert_eq!(route(&yard, "walker", wanted), Route::Swept);
}

/// TZ-5 e: a start 305 mm from the wall lies outside `C`: Rapier's.
#[test]
fn a_start_outside_c_is_swept() {
    let yard = Yard::new(&Plan::room(cafe(), &[("walker", (8_015, 3_000))]));
    let wanted = yard.at("room", (8_500, 3_000));
    assert_eq!(route(&yard, "walker", wanted), Route::Swept);
}
