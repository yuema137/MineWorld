//! `kick`, `throw` and `shove` through the real `World::dispatch`, and the offers through presence's
//! `observe` (step-11 §18.4 PO-4, PO-5, PO-6, PO-7, PO-17; SD-O11 … SD-O15).
//!
//! The room is the prototype's café: floor (0, 0)–(8 320, 10 320), counter (3 860, 6 570)–(8 320,
//! 7 170), 1 100 mm high. A flight cannot be hand-computed to the millimetre: its end is asserted
//! against bounds fixed in §18.4 before measuring, and every invariant is checked after every request
//! (`assert_holds`). PO-4 f, the landing ladder on a flight cut short, is a unit test of `flight.rs`.

mod support;

use mineworld_bodies::{How, Kick, Throw, Toward};
use mineworld_contracts::{
    ActionRecord, ActionResult, Causation, LocalPosition, Millimetres, Rejection,
};
use support::{Moved, Plan, Thing, Xy, Yard, assert_holds, ball, cafe, cube, encode, object_moved};

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

fn kick(yard: &mut Yard, who: &str, object: &str) -> Moved {
    let record = ActionRecord::new::<Kick>(encode(&Kick::new(yard.items[object])));
    yard.submit(who, record)
}

fn throw(yard: &mut Yard, who: &str, object: &str, toward: Option<Xy>) -> Moved {
    let toward = toward.map(|(x, y)| Toward::new(Millimetres::new(x), Millimetres::new(y)));
    let record = ActionRecord::new::<Throw>(encode(&Throw::new(yard.items[object], toward)));
    yard.submit(who, record)
}

fn rejected(moved: &Moved) -> Rejection {
    match &moved.result {
        ActionResult::Rejected(reason) => reason.clone(),
        other => panic!("rejected, not {other:?}"),
    }
}

/// The one `object-moved` of an accepted kick or throw: checked as the request's own fact, stated by
/// bodies; returns (how, from, to, path).
fn flight(yard: &Yard, moved: &Moved) -> (How, Xyz, Xyz, Vec<Xyz>) {
    assert!(moved.accepted(), "accepted: {:?}", moved.result);
    assert_eq!(moved.events.len(), 1, "one fact: {:?}", moved.facts());
    let event = &moved.events[0];
    assert_eq!(
        *event.caused_by(),
        Causation::Action(moved.id),
        "caused by the request"
    );
    assert_eq!(event.provenance().emitted_by().as_str(), "bodies");
    let fact = object_moved(event).expect("an object-moved");
    let xyz = |p: LocalPosition| (p.x().value(), p.y().value(), p.z().value());
    holds(yard);
    (
        fact.how(),
        xyz(fact.from()),
        xyz(fact.to()),
        fact.path().iter().copied().map(xyz).collect(),
    )
}

type Xyz = (i32, i32, i32);

fn inside_the_floor((x, y, _): Xyz) -> bool {
    (0..=8_320).contains(&x) && (0..=10_320).contains(&y)
}

// ---------------------------------------------------------------------------------------------
// PO-4 — kick
// ---------------------------------------------------------------------------------------------

#[test]
fn a_kick_reaches_800_mm_and_no_farther() {
    let mut near = yard(
        &[("kicker", (2_000, 5_000))],
        vec![("ball", ball(110), "room", (2_800, 5_000))],
    );
    assert!(
        kick(&mut near, "kicker", "ball").accepted(),
        "800 mm: in reach"
    );
    let mut far = yard(
        &[("kicker", (2_000, 5_000))],
        vec![("ball", ball(110), "room", (2_801, 5_000))],
    );
    assert_eq!(
        rejected(&kick(&mut far, "kicker", "ball")),
        Rejection::TooFarAway
    );
}

#[test]
fn a_kick_is_refused_for_what_cannot_be_kicked() {
    // A ball in the other room, an item that lies nowhere, a kicker with no position, a malformed
    // payload. (A box on the counter is PO-5 d's, which puts one there.)
    let plan = Plan {
        places: vec![("room", Some(cafe())), ("yard", Some(cafe()))],
        people: vec![
            ("kicker", "room", Some((2_000, 5_000))),
            ("ghost", "room", None),
        ],
        objects: vec![("far", ball(110), "yard", (2_700, 5_000))],
        kinds: vec!["lantern"],
        ..Plan::default()
    };
    let mut yard = Yard::new(&plan);
    assert_eq!(
        rejected(&kick(&mut yard, "kicker", "far")),
        Rejection::TooFarAway
    );
    assert_eq!(
        rejected(&kick(&mut yard, "kicker", "lantern")),
        Rejection::TargetUnavailable
    );
    assert_eq!(
        rejected(&kick(&mut yard, "ghost", "far")),
        Rejection::PreconditionFailed
    );
    let malformed = ActionRecord::new::<Kick>(b"{\"object\": 7}".to_vec());
    assert_eq!(
        rejected(&yard.submit("kicker", malformed)),
        Rejection::PreconditionFailed
    );
}

#[test]
fn a_kick_on_open_floor_sends_the_ball_away_from_the_kicker() {
    let mut yard = yard(
        &[("kicker", (2_000, 5_000))],
        vec![("ball", ball(110), "room", (2_700, 5_000))],
    );
    let moved = kick(&mut yard, "kicker", "ball");
    let (how, from, to, path) = flight(&yard, &moved);
    println!("kicked: from {from:?} to {to:?}, {} keyframes", path.len());
    assert_eq!(how, How::Kicked);
    assert_eq!(from, (2_700, 5_000, 110));
    // Bounds fixed in §18.4 before measuring.
    assert!(
        (1_500..=3_500).contains(&(to.0 - from.0)),
        "x grows by 1 500 … 3 500: {to:?}"
    );
    assert!((to.1 - from.1).abs() <= 50, "|Δy| ≤ 50: {to:?}");
    assert!((to.2 - 110).abs() <= 5, "z = 110 ± 5: {to:?}");
    assert!((1..=40).contains(&path.len()), "1 … 40 keyframes");
    assert_eq!(path[0], from, "the first keyframe is where it lay");
    assert!(
        path.iter().all(|p| inside_the_floor(*p)),
        "every keyframe on the floor"
    );
    assert_eq!(yard.point("kicker"), Some((2_000, 5_000)), "nobody moved");
}

#[test]
fn a_kick_at_the_wall_keeps_the_ball_inside() {
    let mut yard = yard(
        &[("kicker", (6_800, 5_000))],
        vec![("ball", ball(110), "room", (7_500, 5_000))],
    );
    let moved = kick(&mut yard, "kicker", "ball");
    let (_, _, to, _) = flight(&yard, &moved);
    println!("against the wall: {to:?}");
    assert!(to.0 <= 8_320 - 110 + 5, "inside the floor: {to:?}");
}

#[test]
fn a_kicked_ball_does_not_end_in_a_person() {
    let mut yard = yard(
        &[("kicker", (2_000, 5_000)), ("c", (4_000, 5_000))],
        vec![("ball", ball(110), "room", (2_700, 5_000))],
    );
    let moved = kick(&mut yard, "kicker", "ball");
    let (_, _, to, _) = flight(&yard, &moved);
    println!("toward c: {to:?}");
    let d2 = i64::from(to.0 - 4_000).pow(2) + i64::from(to.1 - 5_000).pow(2);
    assert!(d2 >= 405 * 405, "≥ 405 mm from c's centre: {to:?}");
    assert_eq!(yard.point("c"), Some((4_000, 5_000)), "c has no new fact");
}

#[test]
fn without_bodies_kick_and_throw_are_unavailable() {
    let plan = Plan {
        without_bodies: true,
        objects: vec![("ball", ball(110), "room", (2_700, 5_000))],
        ..Plan::room(cafe(), &[("kicker", (2_000, 5_000))])
    };
    let mut yard = Yard::new(&plan);
    assert_eq!(
        kick(&mut yard, "kicker", "ball").result,
        ActionResult::Unavailable
    );
    assert_eq!(
        throw(&mut yard, "kicker", "ball", None).result,
        ActionResult::Unavailable
    );
}

// ---------------------------------------------------------------------------------------------
// PO-5 — throw
// ---------------------------------------------------------------------------------------------

#[test]
fn an_unaimed_throw_lands_about_three_metres_on() {
    let mut yard = yard(
        &[("thrower", (2_000, 5_000))],
        vec![("ball", ball(110), "room", (2_600, 5_000))],
    );
    let moved = throw(&mut yard, "thrower", "ball", None);
    let (how, from, to, _) = flight(&yard, &moved);
    println!("thrown, unaimed: {from:?} → {to:?}");
    assert_eq!(how, How::Thrown);
    // The default aim is (5 600, 5 000); bounds fixed in §18.4.
    assert!(
        (4_600..=7_600).contains(&to.0),
        "x in 4 600 … 7 600: {to:?}"
    );
    assert!((to.1 - 5_000).abs() <= 100, "|Δy| ≤ 100: {to:?}");
    assert!((to.2 - 110).abs() <= 5, "at rest on the floor: {to:?}");
}

#[test]
fn an_aimed_throw_lands_near_its_point() {
    let mut yard = yard(
        &[("thrower", (2_000, 5_000))],
        vec![("ball", ball(110), "room", (2_600, 5_000))],
    );
    let moved = throw(&mut yard, "thrower", "ball", Some((2_600, 8_600)));
    let (_, _, to, _) = flight(&yard, &moved);
    println!("thrown at (2600, 8600): {to:?}");
    let d2 = i64::from(to.0 - 2_600).pow(2) + i64::from(to.1 - 8_600).pow(2);
    assert!(d2 <= 1_500 * 1_500, "within 1 500 mm of the point: {to:?}");
}

#[test]
fn a_throw_is_refused_outside_the_floor_and_beyond_six_metres() {
    let people = [("thrower", (2_000, 5_000))];
    let objects = || vec![("ball", ball(110), "room", (2_600, 5_000))];
    let mut yard_ = yard(&people, objects());
    assert_eq!(
        rejected(&throw(&mut yard_, "thrower", "ball", Some((2_600, 11_000)))),
        Rejection::PreconditionFailed,
        "outside the floor"
    );
    // (3 600, 4 801) from the ball: 6 000.8 mm.
    assert_eq!(
        rejected(&throw(&mut yard_, "thrower", "ball", Some((6_200, 9_801)))),
        Rejection::PreconditionFailed,
        "6 001 mm"
    );
    // (3 600, 4 800): exactly 6 000 mm.
    assert!(
        throw(&mut yard_, "thrower", "ball", Some((6_200, 9_800))).accepted(),
        "6 000 mm"
    );
    let mut far = yard(&people, vec![("ball", ball(110), "room", (2_801, 5_000))]);
    assert_eq!(
        rejected(&throw(&mut far, "thrower", "ball", None)),
        Rejection::TooFarAway
    );
}

#[test]
fn a_box_thrown_at_the_counter_lands_on_its_top_or_the_floor_never_inside_it() {
    let mut yard = yard(
        &[("thrower", (4_500, 5_800))],
        vec![("box", cube(100), "room", (4_950, 5_800))],
    );
    let moved = throw(&mut yard, "thrower", "box", Some((4_950, 6_870)));
    let (_, _, to, _) = flight(&yard, &moved);
    let on_top = (to.2 - 1_200).abs() <= 5;
    let on_floor = (to.2 - 100).abs() <= 5;
    println!("thrown at the counter: {to:?} — on its top: {on_top}, on the floor: {on_floor}");
    assert!(
        on_top || on_floor,
        "on the counter's top or the floor: {to:?}"
    );
    if on_top {
        // PO-4 b's last refusal: a box lying on the counter cannot be kicked; it can be thrown.
        assert_eq!(
            rejected(&kick(&mut yard, "thrower", "box")),
            Rejection::TargetUnavailable
        );
    }
}

#[test]
fn a_ball_thrown_among_people_ends_clear_of_them_and_moves_nobody() {
    let mut yard = yard(
        &[
            ("thrower", (2_000, 5_000)),
            ("p1", (4_000, 4_600)),
            ("p2", (4_000, 5_400)),
            ("p3", (4_600, 5_000)),
        ],
        vec![("ball", ball(110), "room", (2_600, 5_000))],
    );
    let moved = throw(&mut yard, "thrower", "ball", Some((4_000, 5_000)));
    let (_, _, to, _) = flight(&yard, &moved);
    println!("thrown among people: {to:?}");
    for (person, at) in [
        ("p1", (4_000, 4_600)),
        ("p2", (4_000, 5_400)),
        ("p3", (4_600, 5_000)),
    ] {
        let d2 = i64::from(to.0 - at.0).pow(2) + i64::from(to.1 - at.1).pow(2);
        assert!(d2 >= 405 * 405, "≥ 405 mm from {person}: {to:?}");
        assert_eq!(yard.point(person), Some(at), "{person} unmoved");
    }
}

/// The kicks and throws above, in one world, as bytes: every fact and every object's end.
fn flights_bytes() -> String {
    let mut yard = yard(
        &[
            ("kicker", (2_000, 5_000)),
            ("c", (4_000, 5_000)),
            ("thrower", (4_500, 5_800)),
        ],
        vec![
            ("ball", ball(110), "room", (2_700, 5_000)),
            ("box", cube(100), "room", (4_950, 5_800)),
        ],
    );
    let mut facts = Vec::new();
    facts.extend(kick(&mut yard, "kicker", "ball").events);
    facts.extend(throw(&mut yard, "thrower", "box", Some((4_950, 6_870))).events);
    facts.extend(throw(&mut yard, "c", "ball", None).events);
    serde_json::to_string(&(facts, yard.objects("room"))).expect("encodes")
}

const SECOND_PROCESS: &str = "BODIES_FLIGHTS_SECOND_PROCESS";

#[test]
fn flights_are_byte_identical_in_a_second_process() {
    if std::env::var_os(SECOND_PROCESS).is_some() {
        println!("FLIGHTS {}", flights_bytes());
        return;
    }
    let ours = flights_bytes();
    let output = std::process::Command::new(std::env::current_exe().expect("this test binary"))
        .args([
            "--exact",
            "flights_are_byte_identical_in_a_second_process",
            "--nocapture",
        ])
        .env(SECOND_PROCESS, "1")
        .output()
        .expect("the second process runs");
    assert!(output.status.success(), "the second process succeeded");
    let printed = String::from_utf8(output.stdout).expect("utf-8");
    let theirs = printed
        .lines()
        .find_map(|line| {
            line.find("FLIGHTS ")
                .map(|at| &line[at + "FLIGHTS ".len()..])
        })
        .expect("the second process printed its flights");
    println!("flights: {} bytes here, {} there", ours.len(), theirs.len());
    assert_eq!(ours, theirs, "byte-identical in two processes");
}

// ---------------------------------------------------------------------------------------------
// PO-7 — the offers (kick and throw)
// ---------------------------------------------------------------------------------------------

/// PO-7's layout: the observer; a ball 700 mm east (in reach); a box 900 mm north (beyond). P and Q
/// are PO-7's shove rows (PO-C5); Q stands at (3 200, 4 600), not (3 200, 5 000) — §18.11 DO-8.
fn offered() -> Yard {
    yard(
        &[
            ("observer", (2_000, 5_000)),
            ("p", (2_900, 5_400)),
            ("q", (3_200, 4_600)),
        ],
        vec![
            ("ball", ball(110), "room", (2_700, 5_000)),
            ("box", cube(200), "room", (2_000, 5_900)),
        ],
    )
}

/// Every affordance of bodies' actions in `observer`'s observation: (action, target key, payload,
/// available, reason).
fn bodies_affordances(
    yard: &Yard,
    observer: &str,
) -> Vec<(String, Option<&'static str>, String, bool)> {
    yard.observation(observer)
        .affordances()
        .iter()
        .filter(|affordance| {
            ["kick", "throw", "shove"].contains(&affordance.action_type().as_str())
        })
        .map(|affordance| {
            (
                affordance.action_type().as_str().to_owned(),
                affordance.target().map(|target| yard.key_of(target)),
                affordance
                    .payload()
                    .map_or_else(String::new, ToString::to_string),
                affordance.is_available(),
            )
        })
        .collect()
}

#[test]
fn the_kicks_and_throws_offered_are_the_objects_within_reach() {
    let yard = offered();
    let ball = serde_json::to_value(yard.items["ball"]).expect("an id");
    let target_less: Vec<_> = bodies_affordances(&yard, "observer")
        .into_iter()
        .filter(|(_, target, _, _)| target.is_none())
        .collect();
    println!("offered: {target_less:?}");
    assert_eq!(
        target_less,
        [
            (
                "kick".to_owned(),
                None,
                serde_json::json!({ "object": ball }).to_string(),
                true
            ),
            (
                "throw".to_owned(),
                None,
                serde_json::json!({ "object": ball, "toward": null }).to_string(),
                true
            ),
        ],
        "kick and throw of the ball, nothing for the box"
    );
    let observation = yard.observation("observer");
    for affordance in observation
        .affordances()
        .iter()
        .filter(|a| a.target().is_none())
    {
        if ["kick", "throw"].contains(&affordance.action_type().as_str()) {
            let room = yard.places["room"];
            assert_eq!(
                *affordance.requirement(),
                mineworld_contracts::SpatialRequirement::at_place(room)
                    .requiring_target_available()
            );
        }
    }
}

#[test]
fn each_offered_kick_and_throw_is_accepted_as_offered() {
    let observation = offered().observation("observer");
    for affordance in observation.affordances().iter().filter(|affordance| {
        ["kick", "throw"].contains(&affordance.action_type().as_str()) && affordance.is_available()
    }) {
        let mut fresh = offered();
        let request = affordance
            .request(fresh.people["observer"], |value| {
                serde_json::to_vec(value).expect("encodes")
            })
            .expect("a complete affordance");
        let moved = fresh.submit_request(request);
        assert!(
            moved.accepted(),
            "{} accepted as offered: {:?}",
            affordance.action_type(),
            moved.result
        );
    }
}

#[test]
fn nothing_is_offered_in_a_place_without_a_shape_or_to_a_positionless_observer() {
    let plan = Plan {
        places: vec![("room", Some(cafe())), ("street", None)],
        people: vec![
            ("walker", "street", Some((1_000, 1_000))),
            ("ghost", "room", None),
            ("other", "room", Some((5_000, 2_000))),
        ],
        objects: vec![("ball", ball(110), "room", (5_000, 3_000))],
        ..Plan::default()
    };
    let yard = Yard::new(&plan);
    assert!(
        bodies_affordances(&yard, "walker").is_empty(),
        "unshaped place"
    );
    assert!(
        bodies_affordances(&yard, "ghost").is_empty(),
        "positionless"
    );
}
