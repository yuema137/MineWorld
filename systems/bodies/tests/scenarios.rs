//! People with bodies, through the real `World::dispatch`: presence, movement and bodies, `[bodies]`
//! registered (step-11 PB-5 … PB-7, PB-17 … PB-19).
//!
//! ```text
//! walls      a walker stops at the east wall and at the counter, slides along a wall, and a stride
//!            that meets nothing is exactly as asked (the fast path)
//! nudging    n2 — a standing person is nudged aside, ≤ 310 mm a stride; n3 — a crowd of five, chains
//!            ≤ 2 generations and ≤ 4 people, some strides blocked; a chain needing a third
//!            generation is blocked at contact
//! walls win  a person backed against the east wall or the counter cannot be nudged through it: the
//!            stride is blocked, and nobody moves
//! the guard  an arrival that bypassed resolution is caught at the next arrival into the place
//! entries    a crossing lands on a free doorway point; nudges its occupant; is placed at the nearest
//!            free lattice point when the nudge fails or the point is inside a solid
//! inert      a place without a body, and an arrival with no position, are recorded exactly as asked
//! ```
//!
//! The room is the prototype's café: floor (0, 0)–(8 320, 10 320), counter (3 860, 6 570)–(8 320,
//! 7 170). Positions that pass through Rapier are asserted to within ±1 mm of a literal from the
//! layout; generations and degradation are read from the resolver's own account (`explain`), asked
//! before each request.

mod support;

use std::panic::{AssertUnwindSafe, catch_unwind};

use mineworld_bodies::{Degraded, Outcome, Route, explain};
use mineworld_contracts::{ActionRecord, EntityType, Location, PersonId};
use mineworld_presence::Arrived;
use support::{Fact, Moved, Plan, Put, Xy, Yard, cafe, distance2, encode, shape, xy};

/// The floor shrunk by 295 mm (a radius less the tolerance), and the counter grown by it.
fn inside_the_cafe((x, y): Xy) -> bool {
    let in_floor = (295..=8_320 - 295).contains(&x) && (295..=10_320 - 295).contains(&y);
    let dx = (3_860 - x).max(x - 8_320).max(0);
    let dy = (6_570 - y).max(y - 7_170).max(0);
    let clear_of_counter =
        i64::from(dx) * i64::from(dx) + i64::from(dy) * i64::from(dy) >= 295 * 295;
    in_floor && clear_of_counter
}

fn person(yard: &Yard, key: &str) -> PersonId {
    PersonId::new(yard.people[key], EntityType::Person).expect("a person")
}

/// What the resolver will do with `key`'s arrival at `to`, asked before the request.
fn foreseen(yard: &Yard, key: &str, to: Location) -> Outcome {
    explain(&yard.world.read(), person(yard, key), to).expect("an arrival into a shaped place")
}

fn within_a_millimetre(got: Xy, want: Xy) -> bool {
    (got.0 - want.0).abs() <= 1 && (got.1 - want.1).abs() <= 1
}

// ---------------------------------------------------------------------------------------------
// PB-5 — walls
// ---------------------------------------------------------------------------------------------

/// The walker's arrival and the stopped-short beside it, checked; returns where the walker ended.
fn stopped(yard: &Yard, moved: &Moved, walker: &str, wanted: Location, by: Option<&str>) -> Xy {
    assert!(moved.accepted(), "accepted: {:?}", moved.result);
    moved.assert_caused_by_the_request();
    let facts = moved.facts();
    let Fact::Arrived(who, reached) = facts[0] else {
        panic!("the walker's arrival first: {facts:?}");
    };
    assert_eq!(who, yard.people[walker]);
    assert_eq!(
        facts.last(),
        Some(&Fact::StoppedShort {
            person: yard.people[walker],
            wanted,
            reached,
            by: by.map(|key| yard.people[key]),
        }),
        "stopped short, by {by:?}: {facts:?}"
    );
    xy(reached)
}

#[test]
fn a_walker_stops_at_the_east_wall() {
    let mut yard = Yard::new(&Plan::room(cafe(), &[("walker", (7_000, 3_000))]));
    let wanted = yard.at("room", (8_500, 3_000));
    let moved = yard.walk("walker", wanted);
    let reached = stopped(&yard, &moved, "walker", wanted, None);
    println!("east wall: reached {reached:?}");
    // 8 320 − 300 − 10: the wall, a radius, the controller's gap.
    assert!(within_a_millimetre(reached, (8_010, 3_000)), "{reached:?}");
}

#[test]
fn a_walker_stops_at_the_counter() {
    let mut yard = Yard::new(&Plan::room(cafe(), &[("walker", (5_000, 5_500))]));
    let wanted = yard.at("room", (5_000, 7_000));
    let moved = yard.walk("walker", wanted);
    let reached = stopped(&yard, &moved, "walker", wanted, None);
    println!("counter: reached {reached:?}");
    // 6 570 − 310.
    assert!(within_a_millimetre(reached, (5_000, 6_260)), "{reached:?}");
}

#[test]
fn a_walker_slides_along_the_east_wall_and_is_never_carried_farther_than_asked() {
    let mut yard = Yard::new(&Plan::room(cafe(), &[("walker", (7_000, 3_000))]));
    let wanted = yard.at("room", (8_400, 4_400));
    let moved = yard.walk("walker", wanted);
    let reached = stopped(&yard, &moved, "walker", wanted, None);
    println!("slide: reached {reached:?}");
    assert!(
        (8_009..=8_011).contains(&reached.0),
        "against the wall: {reached:?}"
    );
    assert!(
        reached.1 > 4_010,
        "it slid north along the wall: {reached:?}"
    );
    assert!(
        distance2((7_000, 3_000), reached) <= distance2((7_000, 3_000), (8_400, 4_400)),
        "never longer than asked (ARC-39 rule (c))"
    );
}

#[test]
fn a_stride_that_meets_nothing_is_exactly_as_asked_without_a_scene() {
    let mut yard = Yard::new(&Plan::room(
        cafe(),
        &[("walker", (2_000, 2_000)), ("other", (6_000, 2_000))],
    ));
    let to = yard.at("room", (3_500, 2_500));
    assert_eq!(
        foreseen(&yard, "walker", to).route,
        Route::Clear,
        "the fast path"
    );
    let moved = yard.walk("walker", to);
    assert!(moved.accepted());
    moved.assert_caused_by_the_request();
    assert_eq!(moved.events.len(), 1, "one fact: {:?}", moved.facts());
    assert_eq!(
        moved.events[0].payload().payload(),
        &encode(&Arrived::new(person(&yard, "walker"), to)),
        "exactly Arrived::new's bytes"
    );
}

// ---------------------------------------------------------------------------------------------
// PB-6 — nudging
// ---------------------------------------------------------------------------------------------

/// One stride of `walker` by `(dx, dy)` from where presence says they stand, with every bound of
/// I-11 and I-12 checked on its facts and on the place after it. Returns the outcome and the move.
fn bounded_stride(yard: &mut Yard, walker: &str, (dx, dy): Xy) -> (Outcome, Moved) {
    let from = yard.point(walker).expect("placed");
    let to = yard.at("room", (from.0 + dx, from.1 + dy));
    let before = yard.standing("room");
    let outcome = foreseen(yard, walker, to);
    let moved = yard.walk(walker, to);
    assert!(moved.accepted(), "accepted: {:?}", moved.result);
    moved.assert_caused_by_the_request();
    let displaced = moved.displaced();
    assert!(displaced.len() <= 4, "at most four moved: {displaced:?}");
    assert_eq!(
        outcome.nudged,
        displaced.len(),
        "the account matches the facts"
    );
    assert!(
        outcome.generations <= 2,
        "at most two generations: {outcome:?}"
    );
    for (other, at) in &displaced {
        assert_eq!(
            at.place(),
            yard.places["room"],
            "a nudge never changes place"
        );
        let key = yard.key_of(*other);
        let was = before
            .iter()
            .find(|(standing, _)| *standing == key)
            .expect("stood in the room")
            .1;
        let length2 = distance2(was, xy(*at));
        assert!(
            length2 <= 310 * 310,
            "{key} nudged {} mm, more than 310 (I-11)",
            length2.isqrt()
        );
    }
    if let Some((a, b, closest)) = yard.closest("room") {
        assert!(
            closest >= 595 * 595,
            "{a} and {b} {} mm apart after {walker}'s stride (I-12)",
            closest.isqrt()
        );
    }
    for (key, at) in yard.standing("room") {
        assert!(
            inside_the_cafe(at),
            "{key} at {at:?} is outside the floor or in the counter"
        );
    }
    (outcome, moved)
}

#[test]
fn n2_a_standing_person_is_nudged_aside_a_little_at_a_time() {
    let mut yard = Yard::new(&Plan::room(
        cafe(),
        &[("a", (2_000, 5_000)), ("b", (4_000, 5_100))],
    ));
    let mut nudges = 0;
    for stride in 0..12 {
        let (outcome, moved) = bounded_stride(&mut yard, "a", (500, 0));
        nudges += moved.displaced().len();
        println!(
            "n2 stride {stride:2}: a {:?} b {:?} {outcome:?}",
            yard.point("a"),
            yard.point("b")
        );
    }
    let b = yard.point("b").expect("placed");
    let moved = distance2((4_000, 5_100), b).isqrt();
    println!("n2: b moved {moved} mm in all, in {nudges} nudges");
    assert!(
        moved >= 100,
        "the instrument sees a nudge: b moved {moved} mm"
    );
}

#[test]
fn n3_a_crowd_is_nudged_in_bounded_chains_and_sometimes_blocks() {
    let mut yard = Yard::new(&Plan::room(
        cafe(),
        &[
            ("a", (2_000, 5_000)),
            ("c1", (5_000, 5_000)),
            ("c2", (5_000, 5_650)),
            ("c3", (5_000, 4_350)),
            ("c4", (5_650, 5_000)),
            ("c5", (5_650, 5_650)),
        ],
    ));
    let (mut blocked, mut most, mut deepest) = (0, 0, 0);
    for stride in 0..12 {
        let (outcome, moved) = bounded_stride(&mut yard, "a", (500, 0));
        if outcome.nudge_failed || outcome.degraded != Degraded::No {
            blocked += 1;
        }
        most = most.max(moved.displaced().len());
        deepest = deepest.max(outcome.generations);
        println!("n3 stride {stride:2}: a {:?} {outcome:?}", yard.point("a"));
    }
    println!("n3: {blocked} strides blocked; at most {most} moved, {deepest} generations");
    assert!(blocked >= 1, "at least one stride is blocked");
    assert!(most >= 1, "the crowd was nudged");
}

#[test]
fn a_chain_that_needs_a_third_generation_is_blocked_at_contact() {
    // Each link 610 mm from the next along (0.8, 0.6): the walker's stride ends 500 mm from p1, so
    // p1 is nudged 110 mm into p2, p2 110 mm into p3, and p3 would need a third generation. p1 is
    // 300 mm off the walker's line: outside the head-on band.
    let mut yard = Yard::new(&Plan::room(
        cafe(),
        &[
            ("walker", (2_000, 5_000)),
            ("p1", (3_400, 5_300)),
            ("p2", (3_888, 5_666)),
            ("p3", (4_376, 6_032)),
        ],
    ));
    let wanted = yard.at("room", (3_000, 5_000));
    let outcome = foreseen(&yard, "walker", wanted);
    let moved = yard.walk("walker", wanted);
    println!("chain: {outcome:?} {:?}", moved.facts());
    let reached = stopped(&yard, &moved, "walker", wanted, Some("p1"));
    assert!(outcome.nudge_failed, "the pass failed: {outcome:?}");
    assert!(moved.displaced().is_empty(), "nobody else moves");
    for (key, at) in [
        ("p1", (3_400, 5_300)),
        ("p2", (3_888, 5_666)),
        ("p3", (4_376, 6_032)),
    ] {
        assert_eq!(yard.point(key), Some(at), "{key} did not move");
    }
    // Touching (600 mm) at x = 3 400 − √(600² − 300²) = 2 880.4, backed off the controller's 10 mm
    // along the stride: (2 870.4, 5 000), 608.7 mm from p1.
    at_contact(reached, (2_870, 5_000), (3_400, 5_300));
}

/// A blocked walker ends at contact: within a millimetre, on each axis, of the point `contact`
/// computed by hand from the layout — and touching, never overlapping, what stopped it.
///
/// Not simply 2R + GAP = 610 mm from it. The character controller keeps its 10 mm offset along the
/// direction it was moving, so a walker who meets a person at an angle θ off their line of centres
/// stops 600 + 10·cos θ from them (step-11 §17.11, DB-4); each caller computes the point that way.
fn at_contact(reached: Xy, contact: Xy, stopper: Xy) {
    assert!(
        within_a_millimetre(reached, contact),
        "the walker ends at contact, {contact:?} ± 1 mm: {reached:?}"
    );
    let apart = distance2(reached, stopper);
    assert!(
        (600 * 600..=611 * 611).contains(&apart),
        "touching, not overlapping: {} mm",
        apart.isqrt()
    );
}

// ---------------------------------------------------------------------------------------------
// PB-7 — nobody through a wall
// ---------------------------------------------------------------------------------------------

/// `walker` asks for `wanted`, which ends 300 mm short of `pinned`, who stands 320 mm from a wall or
/// the counter in the direction of the nudge: they can give only 10 mm, so the stride is blocked and
/// the walker ends at `contact`.
fn pinned_against(walker: Xy, pinned: Xy, wanted: Xy, contact: Xy) {
    let mut yard = Yard::new(&Plan::room(
        cafe(),
        &[("walker", walker), ("pinned", pinned)],
    ));
    let wanted = yard.at("room", wanted);
    let outcome = foreseen(&yard, "walker", wanted);
    let moved = yard.walk("walker", wanted);
    println!("pinned at {pinned:?}: {outcome:?} {:?}", moved.facts());
    let reached = stopped(&yard, &moved, "walker", wanted, Some("pinned"));
    assert!(moved.displaced().is_empty(), "nobody else moves");
    assert_eq!(
        yard.point("pinned"),
        Some(pinned),
        "the pinned person is unchanged"
    );
    at_contact(reached, contact, pinned);
}

// For both: the stride d is (700, 750) or, turned a quarter, (−750, 700); |d| = 1 025.91. pinned lies
// 1 230.61 mm along it and 219.32 mm off it. Touching (600 mm) at 1 230.61 − √(600² − 219.32²) =
// 672.13 mm along; backed off the 10 mm offset, 662.13 mm along: the start + d × 662.13 / 1 025.91.

#[test]
fn a_person_backed_against_the_east_wall_is_not_nudged_through_it() {
    // pinned: 8 320 − 320. The stride's line passes 219 mm from pinned: outside the head-on band.
    // Contact: (7 000 + 451.79, 2 500 + 484.06).
    pinned_against(
        (7_000, 2_500),
        (8_000, 3_250),
        (7_700, 3_250),
        (7_452, 2_984),
    );
}

#[test]
fn a_person_backed_against_the_counter_is_not_nudged_through_it() {
    // pinned: 6 570 − 320. Contact: (5 750 − 484.06, 5 250 + 451.79).
    pinned_against(
        (5_750, 5_250),
        (5_000, 6_250),
        (5_000, 5_950),
        (5_266, 5_702),
    );
}

// ---------------------------------------------------------------------------------------------
// PB-17 — the guard
// ---------------------------------------------------------------------------------------------

#[test]
fn an_arrival_that_bypassed_resolution_is_caught_at_the_next_arrival_into_the_place() {
    let mut yard = Yard::new(&Plan {
        with_bypass: true,
        ..Plan::room(
            cafe(),
            &[
                ("alice", (2_000, 2_000)),
                ("bob", (5_000, 5_000)),
                ("carol", (7_000, 9_000)),
            ],
        )
    });
    let put = Put {
        person: person(&yard, "bob"),
        to: yard.at("room", (2_200, 2_000)),
    };
    let bypassed = yard.submit("bob", ActionRecord::new::<Put>(encode(&put)));
    assert!(
        bypassed.accepted(),
        "the bypass is recorded: {:?}",
        bypassed.result
    );
    assert_eq!(
        yard.point("bob"),
        Some((2_200, 2_000)),
        "bob now overlaps alice"
    );

    let next = yard.at("room", (7_000, 8_500));
    let payload = catch_unwind(AssertUnwindSafe(|| yard.walk("carol", next)))
        .err()
        .expect("the next arrival into the place must panic");
    let message = payload
        .downcast_ref::<String>()
        .cloned()
        .expect("a formatted message");
    println!("guard: {message}");
    for word in ["alice", "bob", "room", "200 mm", "ARC-39"] {
        assert!(message.contains(word), "names {word}: {message}");
    }
}

// ---------------------------------------------------------------------------------------------
// PB-18 — entries
// ---------------------------------------------------------------------------------------------

/// A lane with no body, its doorway at (1 000, 1 000), opening onto the café at `door`; the walker
/// in the lane at the doorway, and whoever else stands in the café.
fn entering(door: Xy, inside: &[(&'static str, Xy)]) -> Yard {
    let mut people = vec![("walker", "lane", Some((1_000, 1_000)))];
    people.extend(inside.iter().map(|(key, at)| (*key, "hall", Some(*at))));
    Yard::new(&Plan {
        places: vec![("lane", None), ("hall", Some(cafe()))],
        people,
        passages: vec![(("lane", (1_000, 1_000)), ("hall", door))],
        ..Plan::default()
    })
}

/// The walker crosses into the hall at `wanted`; checked and returned.
fn cross(yard: &mut Yard, wanted: Xy) -> (Outcome, Moved) {
    let to = yard.at("hall", wanted);
    let outcome = foreseen(yard, "walker", to);
    let moved = yard.walk("walker", to);
    assert!(
        moved.accepted(),
        "the crossing is accepted: {:?}",
        moved.result
    );
    moved.assert_caused_by_the_request();
    println!("entering at {wanted:?}: {outcome:?} {:?}", moved.facts());
    if let Some((a, b, closest)) = yard.closest("hall") {
        assert!(
            closest >= 595 * 595,
            "{a} and {b} stand {} mm apart in the hall",
            closest.isqrt()
        );
    }
    (outcome, moved)
}

#[test]
fn a_crossing_onto_a_free_doorway_point_lands_exactly_there() {
    let mut yard = entering((900, 2_000), &[]);
    let (outcome, moved) = cross(&mut yard, (900, 2_000));
    assert_eq!(outcome.route, Route::Entered);
    assert_eq!(yard.point("walker"), Some((900, 2_000)));
    assert!(
        moved
            .facts()
            .iter()
            .all(|fact| !matches!(fact, Fact::StoppedShort { .. })),
        "no stopped-short"
    );
}

#[test]
fn a_crossing_onto_an_occupied_point_nudges_the_occupant_aside() {
    let mut yard = entering((900, 2_000), &[("occupant", (1_300, 2_000))]);
    let (outcome, moved) = cross(&mut yard, (900, 2_000));
    assert_eq!(outcome.route, Route::EnteredNudging);
    assert_eq!(
        yard.point("walker"),
        Some((900, 2_000)),
        "the crosser is at the point"
    );
    let displaced = moved.displaced();
    assert_eq!(displaced.len(), 1, "the occupant: {displaced:?}");
    let occupant = yard.point("occupant").expect("placed");
    // 610 − 400 = 210 mm east.
    assert!(
        within_a_millimetre(occupant, (1_510, 2_000)),
        "{occupant:?}"
    );
    assert!(distance2((1_300, 2_000), occupant) <= 310 * 310);
}

#[test]
fn a_crossing_whose_nudge_fails_is_placed_at_the_nearest_free_lattice_point() {
    // The occupant stands 400 mm west of the point and 500 mm from the west wall: pushed 210 mm west
    // it can give 190. The nearest free point of the 50 mm lattice anchored at (300, 300) is at
    // distance² 62 500 from (900, 2 000) three times — (1 100, 1 850), (1 150, 2 000), (1 100, 2 150)
    // — and the lowest y wins.
    let mut yard = entering((900, 2_000), &[("occupant", (500, 2_000))]);
    let wanted = yard.at("hall", (900, 2_000));
    let (outcome, moved) = cross(&mut yard, (900, 2_000));
    assert_eq!(outcome.route, Route::Placed);
    assert_eq!(yard.point("walker"), Some((1_100, 1_850)));
    assert_eq!(
        yard.point("occupant"),
        Some((500, 2_000)),
        "nobody else moves"
    );
    assert!(moved.facts().contains(&Fact::StoppedShort {
        person: yard.people["walker"],
        wanted,
        reached: yard.at("hall", (1_100, 1_850)),
        by: Some(yard.people["occupant"]),
    }));
}

#[test]
fn a_crossing_onto_a_point_inside_a_solid_is_placed_at_the_nearest_free_point() {
    // (5 000, 6 800) is inside the counter; the nearest free lattice point is (5 000, 6 250), 320 mm
    // south of its face.
    let mut yard = entering((5_000, 6_000), &[]);
    let wanted = yard.at("hall", (5_000, 6_800));
    let (outcome, moved) = cross(&mut yard, (5_000, 6_800));
    assert_eq!(outcome.route, Route::Placed);
    assert_eq!(yard.point("walker"), Some((5_000, 6_250)));
    assert!(moved.facts().contains(&Fact::StoppedShort {
        person: yard.people["walker"],
        wanted,
        reached: yard.at("hall", (5_000, 6_250)),
        by: None,
    }));
}

// ---------------------------------------------------------------------------------------------
// PB-19 — inert where absent
// ---------------------------------------------------------------------------------------------

/// 12a's RS-7 script (`tests/acceptance/tests/resolvers/mod.rs`): twelve moves for alice, two through
/// the doorway — here between two places with no body, past bob and carol, whom nobody resolves.
const SCRIPT: [(&str, Xy); 12] = [
    ("yard", (5_000, 2_000)),
    ("yard", (6_500, 2_000)),
    ("yard", (8_000, 2_000)),
    ("yard", (9_000, 2_500)),
    ("lane", (500, 2_000)),
    ("lane", (2_000, 2_000)),
    ("lane", (3_000, 3_000)),
    ("lane", (1_500, 2_000)),
    ("lane", (200, 1_800)),
    ("yard", (8_800, 2_000)),
    ("yard", (7_000, 1_500)),
    ("yard", (5_600, 2_000)),
];

#[test]
fn a_place_without_a_body_and_an_arrival_without_a_position_are_recorded_exactly_as_asked() {
    let mut yard = Yard::new(&Plan {
        places: vec![
            ("yard", None),
            ("lane", None),
            ("court", Some(shape((0, 0, 10_000, 10_000), &[]))),
        ],
        people: vec![
            ("alice", "yard", Some((4_000, 2_000))),
            ("bob", "yard", Some((5_100, 2_100))),
            ("carol", "yard", Some((5_400, 2_000))),
            ("dan", "court", Some((2_000, 2_000))),
        ],
        passages: vec![(("yard", (9_000, 2_000)), ("lane", (0, 2_000)))],
        ..Plan::default()
    });
    let mut place = "yard";
    for (step, (to_place, at)) in SCRIPT.into_iter().enumerate() {
        let to = yard.at(to_place, at);
        assert!(
            explain(&yard.world.read(), person(&yard, "alice"), to).is_none(),
            "inert"
        );
        let moved = yard.walk("alice", to);
        assert!(moved.accepted(), "move {step}: {:?}", moved.result);
        let crossing = to_place != place;
        assert_eq!(
            moved.events.len(),
            if crossing { 2 } else { 1 },
            "move {step}"
        );
        assert_eq!(
            moved.events[0].payload().payload(),
            &encode(&Arrived::new(person(&yard, "alice"), to)),
            "move {step}: exactly the arrival asked for, byte for byte"
        );
        assert!(
            moved
                .facts()
                .iter()
                .all(|fact| !matches!(fact, Fact::StoppedShort { .. })),
            "move {step}: never stopped-short"
        );
        place = to_place;
    }
    // In the shaped court, an arrival with no position: no body to place.
    let nowhere = Location::in_place(yard.places["court"]);
    let moved = yard.walk("dan", nowhere);
    assert!(moved.accepted(), "{:?}", moved.result);
    assert_eq!(
        moved.events[0].payload().payload(),
        &encode(&Arrived::new(person(&yard, "dan"), nowhere)),
        "exactly Arrived::new's bytes"
    );
    assert_eq!(moved.events.len(), 1);
}
