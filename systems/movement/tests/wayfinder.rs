//! Movement's wayfinder catalog (`DECISIONS.md` `ARC-75`, `ARC-62` item 4; step-11 SD-N3, NV-C2).
//!
//! This file is its own process, so it owns the catalog: it registers two synthetic wayfinders, listed
//! in the wrong order, and every test below sees that one registration (tests in one file share a
//! process, and registering the same ids again does nothing). Both plan only in movement's test town
//! and only for the goals named below, so they also show that the first answer in `SystemId` order is
//! the route, that `unreachable` is a `no-route` refusal, and that with no answer a leg is straight.

mod support;

use mineworld_contracts::{
    ActionRecord, ActionResult, LocalPosition, Millimetres, Rejection, SystemId,
};
use mineworld_kernel::WorldRead;
use mineworld_movement::{
    Destination, RouteAnswer, RouteAsk, WalkStep, WalkTo, Wayfinder, Waypoints,
    register_wayfinders, registered_wayfinders, require_wayfinder,
};
use support::{Town, at, encode};

fn p(x: i32, y: i32) -> LocalPosition {
    LocalPosition::on_ground(Millimetres::new(x), Millimetres::new(y))
}

/// Answers for one goal, (4 000, 5 000), with a detour north through (2 500, 6 000).
#[derive(Default)]
struct Alpha;

impl Wayfinder for Alpha {
    fn wayfinder_of(&self) -> SystemId {
        SystemId::from_static("alpha")
    }

    fn route(&self, _world: &WorldRead<'_>, ask: &RouteAsk) -> Option<RouteAnswer> {
        (ask.to() == p(4_000, 5_000)).then(|| {
            RouteAnswer::Waypoints(
                Waypoints::new(vec![p(2_500, 6_000), p(4_000, 5_000)]).expect("two"),
            )
        })
    }
}

/// Answers for the same goal with a detour south — never asked for it, because alpha answers first —
/// and answers that (1 000, 8 000) cannot be reached.
#[derive(Default)]
struct Zeta;

impl Wayfinder for Zeta {
    fn wayfinder_of(&self) -> SystemId {
        SystemId::from_static("zeta")
    }

    fn route(&self, _world: &WorldRead<'_>, ask: &RouteAsk) -> Option<RouteAnswer> {
        if ask.to() == p(4_000, 5_000) {
            return Some(RouteAnswer::Waypoints(
                Waypoints::new(vec![p(2_500, 4_000), p(4_000, 5_000)]).expect("two"),
            ));
        }
        (ask.to() == p(1_000, 8_000)).then_some(RouteAnswer::Unreachable)
    }
}

fn register() {
    register_wayfinders(vec![Box::new(Zeta), Box::new(Alpha)]);
}

#[test]
fn wayfinders_are_asked_in_system_id_order_whatever_order_they_are_listed_in() {
    register();
    assert_eq!(
        registered_wayfinders(),
        Some(vec![
            SystemId::from_static("alpha"),
            SystemId::from_static("zeta")
        ])
    );
    // The same ids again, in either order, change nothing: every composition registers.
    register_wayfinders(vec![Box::new(Alpha), Box::new(Zeta)]);
    require_wayfinder(&SystemId::from_static("zeta"));
}

#[test]
fn the_first_answer_is_the_route_unreachable_is_refused_and_no_answer_is_straight() {
    register();
    let mut town = Town::joined(|town| at(town.cafe, 1_000, 5_000));
    let (cafe, visitor) = (town.cafe, town.visitor);
    let walk_to = |town: &mut Town, x, y| {
        town.submit(
            visitor,
            ActionRecord::new::<WalkTo>(encode(&WalkTo::new(Destination::Place(at(cafe, x, y))))),
        )
        .0
    };
    let step = |town: &mut Town| {
        town.submit(
            visitor,
            ActionRecord::new::<WalkStep>(encode(&WalkStep::default())),
        );
        town.presence(visitor).expect("placed")
    };

    // Unreachable, by zeta: refused no-route at once.
    match walk_to(&mut town, 1_000, 8_000) {
        ActionResult::Rejected(Rejection::System { code, .. }) => {
            assert_eq!(code.as_str(), "no-route")
        }
        other => panic!("no-route, not {other:?}"),
    }

    // Alpha's detour, not zeta's: toward (2 500, 6 000) — (1 500, 1 000), ⌈1 802.8⌉ = 1 803, so
    // 1 500·1 340/1 803 = 1 114 and 1 000·1 340/1 803 = 743 → (2 114, 5 743); the rest of the leg,
    // 464 mm, to (2 500, 6 000); then (1 500, −1 000) the same way → (3 614, 5 257); then (4 000, 5 000).
    assert!(matches!(
        walk_to(&mut town, 4_000, 5_000),
        ActionResult::Accepted { .. }
    ));
    let trail: Vec<_> = (0..4).map(|_| step(&mut town)).collect();
    assert_eq!(
        trail,
        [
            at(cafe, 2_114, 5_743),
            at(cafe, 2_500, 6_000),
            at(cafe, 3_614, 5_257),
            at(cafe, 4_000, 5_000)
        ]
    );

    // A goal neither answers for: the straight segment, 1 000 mm north in one stride.
    assert!(matches!(
        walk_to(&mut town, 4_000, 6_000),
        ActionResult::Accepted { .. }
    ));
    assert_eq!(step(&mut town), at(cafe, 4_000, 6_000));
}

#[test]
#[should_panic(expected = "one build has one catalog of wayfinders")]
fn a_different_list_is_refused_naming_both() {
    register();
    register_wayfinders(vec![Box::new(Alpha)]);
}

#[test]
#[should_panic(expected = "list it on movement's extension line")]
fn a_pack_that_is_not_registered_refuses_to_join() {
    register();
    require_wayfinder(&SystemId::from_static("bodies"));
}

#[test]
#[should_panic(expected = "two wayfinders for 'alpha'")]
fn one_id_twice_is_refused_by_name() {
    register_wayfinders(vec![Box::new(Alpha), Box::new(Alpha)]);
}
