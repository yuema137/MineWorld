//! Walking pushes objects, through the real `World::dispatch`: presence, movement and bodies,
//! `[bodies]` registered (step-11 §18.4 PO-3; SD-O8 … SD-O10).
//!
//! ```text
//! a  push            a stride ending 300 mm past contact with a box pushes it clear, R + GAP = 260 mm
//! b  jam             a box against the east wall cannot be pushed: the stride is resolved with
//!                    objects solid, and the walker stops at contact
//! c  no tunnelling   a long stride at a ball stops 300 mm past contact and pushes it; never across
//! d  fast path       a stride R + GAP + 1 = 261 mm clear of every footprint is exactly as asked
//! e  nudged pusher   a person nudged by the walker pushes a ball, caused by their own arrival
//! f  conflict        two nudged people would both push one ball: objects solid, the ball unmoved
//! g  solids          a box pushed toward the counter jams like (b)
//! ```
//!
//! The room is the prototype's café: floor (0, 0)–(8 320, 10 320), counter (3 860, 6 570)–(8 320,
//! 7 170), 1 100 mm high. Every expected position is a literal hand-computed from the layout, asserted
//! within ±1 mm where it passes through a Rapier sweep or cast (test rules §25). After every request
//! every invariant is checked from the test's own literals (`assert_holds`).

mod support;

use mineworld_bodies::{How, Objects, explain};
use mineworld_contracts::{Causation, EntityType, PersonId};
use support::{
    Fact, GAP, Moved, Plan, R, Thing, Xy, Yard, assert_holds, ball, cafe, cube, object_moved, xy,
};

/// How far a stride may carry on past contact (`NUDGE_MAX`, a policy, not a person dimension).
const NUDGE_MAX: i32 = 300;

const FLOOR: (i32, i32, i32, i32) = (0, 0, 8_320, 10_320);
const COUNTER: ((i32, i32, i32, i32), i32) = ((3_860, 6_570, 8_320, 7_170), 1_100);

fn yard(people: &[(&'static str, Xy)], objects: Vec<Thing>) -> Yard {
    Yard::new(&Plan {
        objects,
        ..Plan::room(cafe(), people)
    })
}

fn holds(yard: &Yard) {
    assert_holds(yard, "room", FLOOR, &[COUNTER]);
}

fn near(got: Xy, want: Xy) -> bool {
    (got.0 - want.0).abs() <= 1 && (got.1 - want.1).abs() <= 1
}

/// Walks `walker` to `to`, accepted, and checks the invariants afterwards.
fn walk(yard: &mut Yard, walker: &str, to: Xy) -> Moved {
    let at = yard.at("room", to);
    let moved = yard.walk(walker, at);
    assert!(moved.accepted(), "{walker} to {to:?}: {:?}", moved.result);
    holds(yard);
    moved
}

/// The request's `object-moved` facts: (object key, from, to, how, by key, causing event's index in
/// the request's facts).
fn pushes(yard: &Yard, moved: &Moved) -> Vec<(&'static str, Xy, Xy, How, &'static str, usize)> {
    moved
        .events
        .iter()
        .filter_map(|event| {
            let fact = object_moved(event)?;
            let key = yard
                .items
                .iter()
                .find(|(_, id)| **id == fact.object())
                .map(|(key, _)| *key)
                .expect("one of the yard's objects");
            let Causation::Event(cause) = event.caused_by() else {
                panic!(
                    "a push is caused by an arrival, not {:?}",
                    event.caused_by()
                );
            };
            let index = moved
                .events
                .iter()
                .position(|other| other.id() == *cause)
                .expect("the cause is one of the request's facts");
            assert_eq!(
                event.provenance().controller_decision(),
                Some(moved.id),
                "the walker's decision (AC-9)"
            );
            assert_eq!(event.provenance().emitted_by().as_str(), "bodies");
            let ground = |p: mineworld_contracts::LocalPosition| (p.x().value(), p.y().value());
            Some((
                key,
                ground(fact.from()),
                ground(fact.to()),
                fact.how(),
                yard.key_of(fact.by().entity_id()),
                index,
            ))
        })
        .collect()
}

/// The walker's stopped-short, if any: (reached, by).
fn stopped(moved: &Moved) -> Option<(Xy, Option<mineworld_contracts::EntityId>)> {
    moved.facts().into_iter().find_map(|fact| match fact {
        Fact::StoppedShort { reached, by, .. } => Some((xy(reached), by)),
        _ => None,
    })
}

#[test]
fn a_push_moves_a_box_out_of_the_walkers_way() {
    let mut yard = yard(
        &[("walker", (2_800, 5_000))],
        vec![("box", cube(200), "room", (3_500, 5_000))],
    );
    // Contact at the box's face (3 300) − (R + GAP) = 3 300 − 260 = 3 040; the candidate rule allows
    // NUDGE_MAX = 300 mm more, to 3 340. The walker asks for 10 mm beyond that, 3 350 (3 300 while R
    // was 300: 2 990 + 300 + 10), so the box stops them; the box goes until its face is R + GAP from
    // the walker: 3 340 + 260 = 3 600, its centre 3 800.
    let contact = 3_300 - R - GAP;
    let moved = walk(&mut yard, "walker", (contact + NUDGE_MAX + 10, 5_000));
    let walker = yard.point("walker").expect("placed");
    assert!(
        near(walker, (contact + NUDGE_MAX, 5_000)),
        "walker at {walker:?}"
    );
    let (x, y, z) = yard.object("box");
    let pushed_to = (contact + NUDGE_MAX + R + GAP + 200, 5_000);
    assert!(
        near((x, y), pushed_to) && z == 200,
        "box at ({x}, {y}, {z})"
    );
    let (_, by) = stopped(&moved).expect("stopped short");
    assert_eq!(
        by,
        Some(yard.items["box"].entity_id()),
        "stopped by the box"
    );
    let facts: Vec<String> = moved
        .events
        .iter()
        .map(|event| event.event_type().as_str().to_owned())
        .collect();
    assert_eq!(
        facts,
        ["arrived", "stopped-short", "object-moved"],
        "in order"
    );
    let pushed = pushes(&yard, &moved);
    assert_eq!(pushed.len(), 1);
    let (key, from, to, how, by, cause) = pushed[0];
    assert_eq!(
        (key, from, how, by, cause),
        ("box", (3_500, 5_000), How::Pushed, "walker", 0)
    );
    assert!(near(to, pushed_to), "pushed to {to:?}");
}

#[test]
fn a_box_against_the_wall_jams_and_the_walker_stops_at_it() {
    let mut yard = yard(
        &[("walker", (7_300, 5_000))],
        vec![("box", cube(200), "room", (8_120, 5_000))],
    );
    let id = PersonId::new(yard.people["walker"], EntityType::Person).expect("a person");
    let to = yard.at("room", (7_800, 5_000));
    let outcome = explain(&yard.world.read(), id, to).expect("shaped");
    assert_eq!(outcome.objects, Objects::Solid, "{outcome:?}");
    let moved = walk(&mut yard, "walker", (7_800, 5_000));
    let walker = yard.point("walker").expect("placed");
    // Contact: the box's west face (8 120 − 200 = 7 920) − (R + GAP) = 7 920 − 260 = 7 660.
    assert!(
        near(walker, (8_120 - 200 - R - GAP, 5_000)),
        "walker at contact: {walker:?}"
    );
    assert_eq!(
        yard.object("box"),
        (8_120, 5_000, 200),
        "the box is unmoved"
    );
    let (_, by) = stopped(&moved).expect("stopped short");
    assert_eq!(
        by,
        Some(yard.items["box"].entity_id()),
        "stopped by the box"
    );
    assert!(pushes(&yard, &moved).is_empty(), "nothing pushed");
}

#[test]
fn a_long_stride_at_a_ball_never_passes_over_it() {
    let mut yard = yard(
        &[("walker", (3_000, 5_000))],
        vec![("ball", ball(110), "room", (4_000, 5_000))],
    );
    let moved = walk(&mut yard, "walker", (5_000, 5_000));
    // Contact at 4 000 − 110 − (R + GAP) = 4 000 − 110 − 260 = 3 630; NUDGE_MAX = 300 mm more is
    // 3 930; the ball goes until its edge is R + GAP from the walker: 3 930 + 260 + 110 = 4 300.
    let walker_at = 4_000 - 110 - R - GAP + NUDGE_MAX;
    let walker = yard.point("walker").expect("placed");
    assert!(near(walker, (walker_at, 5_000)), "walker at {walker:?}");
    let (x, y, _) = yard.object("ball");
    assert!(
        near((x, y), (walker_at + R + GAP + 110, 5_000)),
        "ball at ({x}, {y})"
    );
    let (_, by) = stopped(&moved).expect("stopped short");
    assert_eq!(by, Some(yard.items["ball"].entity_id()));
}

#[test]
fn a_stride_clear_of_every_footprint_is_exactly_as_asked() {
    // The ball's edge R + GAP + 1 = 261 mm from the walker's line (311 while R was 300): 5 000 + 261 +
    // 110 = 5 371.
    let mut yard = yard(
        &[("walker", (2_000, 5_000))],
        vec![(
            "ball",
            ball(110),
            "room",
            (2_250, 5_000 + R + GAP + 1 + 110),
        )],
    );
    let id = PersonId::new(yard.people["walker"], EntityType::Person).expect("a person");
    let to = yard.at("room", (2_500, 5_000));
    let outcome = explain(&yard.world.read(), id, to).expect("shaped");
    assert_eq!(outcome.route, mineworld_bodies::Route::Clear, "{outcome:?}");
    let moved = walk(&mut yard, "walker", (2_500, 5_000));
    assert_eq!(moved.events.len(), 1, "one fact");
    let expected = support::encode(&mineworld_presence::Arrived::new(id, to));
    assert_eq!(
        moved.events[0].payload().payload(),
        expected.as_slice(),
        "exactly Arrived::new's bytes"
    );
}

#[test]
fn a_nudged_person_pushes_a_ball_by_their_own_arrival() {
    // b stands 250 mm off the walker's line (beyond the head-on band); the walker's end nudges b
    // north-east; b's new disc overlaps the ball beside b, which b then pushes.
    //
    // The walker's end (2 600, 5 000) is (400, 250) from b, ⌊√222 500⌋ = 471 mm: b is nudged by
    // at_least((400, 250), 2R + GAP − 471 = 510 − 471 = 39) = (⌈33.12⌉, ⌈20.70⌉) = (34, 21), to
    // (3 034, 5 271). The ball (r 110) lies 340 mm east of there, 30 mm inside b's push reach
    // R + GAP + 110 = 370; from b's start it is (374, 21) away, 374.6 mm, clear of b's disc
    // (R − TOL + 110 = 355), so the place loads. (While R was 300 b went 139 mm to (3 119, 5 324) and
    // the ball lay 369 mm east of there, at (3 488, 5 324); at R = 250 that ball is 457 mm from b's
    // new position, beyond its reach, and b pushes nothing.)
    let nudged = (3_000 + 34, 5_250 + 21);
    let mut yard = yard(
        &[("walker", (2_000, 5_000)), ("b", (3_000, 5_250))],
        vec![("ball", ball(110), "room", (nudged.0 + 340, nudged.1))],
    );
    let moved = walk(&mut yard, "walker", (2_600, 5_000));
    let displaced = moved.displaced();
    assert_eq!(displaced.len(), 1, "b was nudged: {:?}", moved.facts());
    let b = xy(displaced[0].1);
    assert!(near(b, nudged), "b nudged 39 mm, to {b:?}");
    let pushed = pushes(&yard, &moved);
    assert_eq!(pushed.len(), 1, "one push: {:?}", moved.facts());
    let (key, _, _, how, by, cause) = pushed[0];
    assert_eq!((key, how, by), ("ball", How::Pushed, "b"));
    match moved.facts()[cause] {
        Fact::Arrived(person, _) => assert_eq!(person, yard.people["b"], "caused by b's arrival"),
        ref other => panic!("caused by an arrival, not {other:?}"),
    }
}

#[test]
fn two_people_who_would_both_push_one_ball_leave_it_and_the_stride_is_resolved_with_objects_solid()
{
    // b and c stand 488 mm from the walker's end (2 000, 5 000), either side of its line: (377, ±310),
    // ⌊√238 229⌋ = 488, 22 mm inside 2R + GAP = 510 (as 588 was inside 610 while R was 300). Nudged by
    // at_least((377, ±310), 22) = (⌈16.99⌉, ±⌈13.97⌉) = (17, ±14) to (2 394, 5 324) and (2 394, 4 676).
    // The ball, r 375 at (2 916, 5 000), is then (522, ∓324) from each, ⌊√377 460⌋ = 614 mm: 20 mm
    // inside the push reach R + GAP + 375 = 635 — one ball pushed twice. At genesis b and c are
    // (539, ±310) from it, 621.8 mm, clear of their discs (R − TOL + 375 = 620), so the place loads.
    // Each push alone would be valid — b's moves the ball by at_least((522, −324), 20) = (18, −11)
    // to (2 934, 4 989), (540, 313) from c', 624.2 mm, clear of c's disc (620), and c's symmetrically
    // — so only the "pushed twice" rule refuses it (M-PO10). (While R was 300: b, c at (2 500, 5 000 ± 310)
    // and the ball at (3 100, 5 000).)
    let mut yard = yard(
        &[
            ("walker", (1_400, 5_000)),
            ("b", (2_000 + 377, 5_000 + 310)),
            ("c", (2_000 + 377, 5_000 - 310)),
        ],
        vec![("ball", ball(375), "room", (2_916, 5_000))],
    );
    let id = PersonId::new(yard.people["walker"], EntityType::Person).expect("a person");
    let to = yard.at("room", (2_000, 5_000));
    let outcome = explain(&yard.world.read(), id, to).expect("shaped");
    println!("conflict: {outcome:?}");
    assert_eq!(
        outcome.objects,
        Objects::Solid,
        "the resolver's own account"
    );
    let moved = walk(&mut yard, "walker", (2_000, 5_000));
    assert_eq!(
        yard.object("ball"),
        (2_916, 5_000, 375),
        "the ball is unmoved"
    );
    assert!(pushes(&yard, &moved).is_empty(), "nothing pushed");
}

#[test]
fn a_box_pushed_toward_the_counter_jams_and_the_walker_stops_at_it() {
    // The box's north face touches the counter's south face (6 570).
    let mut yard = yard(
        &[("walker", (5_000, 5_600))],
        vec![("box", cube(200), "room", (5_000, 6_370))],
    );
    let moved = walk(&mut yard, "walker", (5_000, 6_000));
    let walker = yard.point("walker").expect("placed");
    // Contact: the box's south face (6 370 − 200 = 6 170) − (R + GAP) = 6 170 − 260 = 5 910.
    assert!(
        near(walker, (5_000, 6_370 - 200 - R - GAP)),
        "walker at contact: {walker:?}"
    );
    assert_eq!(
        yard.object("box"),
        (5_000, 6_370, 200),
        "the box is unmoved"
    );
    let (_, by) = stopped(&moved).expect("stopped short");
    assert_eq!(by, Some(yard.items["box"].entity_id()));
}
