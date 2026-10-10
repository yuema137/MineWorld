//! Walking with no wayfinder registered: the route of every leg is the straight segment (step-11
//! SD-N1 … SD-N10, NV-C3; `DECISIONS.md` `ARC-73`).
//!
//! This file never registers a wayfinder, so it is the world without a geometry pack: what NV-3 claims
//! of a world without `bodies` (a straight route, one waypoint per leg, every stride accepted where it
//! was asked) is shown here on movement's own small town, and every refusal, the superseding rules,
//! "no calendar time" and a resume mid-walk with it.
//!
//! ```text
//! cafe     the doorway at (4600, 2000)  ─┐   Alice at (1200, 1000)
//! street   the doorway at (0, 2000)     ─┘
//! attic    no passage
//! ```
//!
//! Every expected position is hand-computed in the comments from the layout and `WALK_STRIDE`
//! 1 340 mm, never read back from the code under test (`ARC-23` rule 2).

mod support;

use mineworld_contracts::{
    ActionRecord, ActionResult, Event, EventEnvelope, Location, PersonId, Rejection, SimDuration,
    WorldTime,
};
use mineworld_kernel::World;
use mineworld_movement::{Destination, Ended, WalkEnded, WalkStep, WalkTo, Walking, is_walking};
use support::{Layout, Movement, NOW, Town, at, encode};

fn as_person(entity: mineworld_contracts::EntityId) -> PersonId {
    PersonId::new(entity, mineworld_contracts::EntityType::Person).expect("a person")
}

fn walk_to(
    town: &mut Town,
    who: mineworld_contracts::EntityId,
    to: Destination,
) -> (ActionResult, Vec<EventEnvelope>) {
    town.submit(who, ActionRecord::new::<WalkTo>(encode(&WalkTo::new(to))))
}

fn step(town: &mut Town, who: mineworld_contracts::EntityId) -> (ActionResult, Vec<EventEnvelope>) {
    town.submit(
        who,
        ActionRecord::new::<WalkStep>(encode(&WalkStep::default())),
    )
}

/// The outcome of the `walk-ended` among `events`, if one is there.
fn ended(events: &[EventEnvelope]) -> Option<Ended> {
    events
        .iter()
        .find(|fact| *fact.event_type() == WalkEnded::EVENT_TYPE)
        .map(|fact| {
            let payload = fact
                .payload()
                .payload_for::<WalkEnded>()
                .expect("movement's fact");
            serde_json::from_slice::<WalkEnded>(payload)
                .expect("it decodes")
                .outcome()
        })
}

fn kinds(events: &[EventEnvelope]) -> Vec<&str> {
    events
        .iter()
        .map(|fact| fact.event_type().as_str())
        .collect()
}

/// Steps until the walk ends; returns where the walker was after each accepted step, and how it ended.
fn walk_out(town: &mut Town, who: mineworld_contracts::EntityId) -> (Vec<Location>, Ended) {
    let mut trail = Vec::new();
    for _ in 0..40 {
        let (result, events) = step(town, who);
        assert!(
            matches!(result, ActionResult::Accepted { .. }),
            "{result:?}"
        );
        trail.push(town.presence(who).expect("placed"));
        if let Some(outcome) = ended(&events) {
            return (trail, outcome);
        }
    }
    panic!("the walk did not end in 40 steps: {trail:?}");
}

#[test]
fn a_straight_walk_takes_strides_of_at_most_1340_mm_and_ends_arrived() {
    let mut town = Town::joined(|town| at(town.cafe, 1_000, 5_000));
    let (cafe, visitor) = (town.cafe, town.visitor);
    let (result, events) = walk_to(
        &mut town,
        visitor,
        Destination::Place(at(cafe, 4_000, 5_000)),
    );
    assert!(
        matches!(result, ActionResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(kinds(&events), ["walk-started"], "no stride at walk-to");
    assert!(is_walking(&town.world.read(), as_person(visitor)));
    // 3 000 mm east: 1 340, 1 340, then the last 320.
    let (trail, outcome) = walk_out(&mut town, visitor);
    assert_eq!(
        trail,
        [
            at(cafe, 2_340, 5_000),
            at(cafe, 3_680, 5_000),
            at(cafe, 4_000, 5_000)
        ]
    );
    assert_eq!(outcome, Ended::Arrived);
    assert!(!is_walking(&town.world.read(), as_person(visitor)));
    let (result, _) = step(&mut town, visitor);
    assert_eq!(
        result,
        ActionResult::Rejected(Rejection::PreconditionFailed),
        "no walk, no step"
    );
}

#[test]
fn a_walk_into_the_next_place_crosses_at_the_doorway() {
    let mut town = Town::joined(|town| at(town.cafe, 2_600, 2_000));
    let (cafe, street, visitor) = (town.cafe, town.street, town.visitor);
    walk_to(
        &mut town,
        visitor,
        Destination::Place(at(street, 1_000, 2_000)),
    );
    // To the café's doorway (4 600, 2 000): 1 340 then 660; the crossing to the street's (0, 2 000);
    // then 1 000 east.
    let mut entered = 0;
    let mut trail = Vec::new();
    let outcome = loop {
        let (_, events) = step(&mut town, visitor);
        entered += kinds(&events)
            .iter()
            .filter(|kind| **kind == "person-entered-place")
            .count();
        trail.push(town.presence(visitor).expect("placed"));
        if let Some(outcome) = ended(&events) {
            break outcome;
        }
        assert!(trail.len() < 10, "{trail:?}");
    };
    assert_eq!(
        trail,
        [
            at(cafe, 3_940, 2_000),
            at(cafe, 4_600, 2_000),
            at(street, 0, 2_000),
            at(street, 1_000, 2_000)
        ]
    );
    assert_eq!((outcome, entered), (Ended::Arrived, 1));
}

#[test]
fn a_place_without_a_position_is_reached_on_entering_it() {
    let mut town = Town::joined(|town| at(town.cafe, 4_000, 2_000));
    let (street, visitor) = (town.street, town.visitor);
    walk_to(
        &mut town,
        visitor,
        Destination::Place(Location::in_place(street)),
    );
    let (trail, outcome) = walk_out(&mut town, visitor);
    // 600 mm to the doorway, then the crossing — which is the arrival.
    assert_eq!(trail.last().map(Location::place), Some(street));
    assert_eq!((trail.len(), outcome), (2, Ended::Arrived));
}

#[test]
fn a_semantic_walk_crosses_without_positions() {
    let mut town = Town::new(Movement::Enabled, Layout::Semantic, |town| {
        Location::in_place(town.cafe)
    });
    let (street, visitor) = (town.street, town.visitor);
    walk_to(
        &mut town,
        visitor,
        Destination::Place(Location::in_place(street)),
    );
    let (trail, outcome) = walk_out(&mut town, visitor);
    assert_eq!(trail, [Location::in_place(street)]);
    assert_eq!(outcome, Ended::Arrived);
}

#[test]
fn a_walk_to_a_person_ends_at_personal_distance_and_follows_them() {
    let mut town = Town::joined(|town| at(town.cafe, 1_200, 6_000));
    let (cafe, visitor, alice) = (town.cafe, town.visitor, town.alice);
    let target = as_person(alice);
    walk_to(&mut town, visitor, Destination::Person(target));
    // Alice at (1 200, 1 000), 5 000 mm south: aim at (1 200, 2 200), 1 200 short of her.
    let (trail, outcome) = walk_out(&mut town, visitor);
    assert_eq!(
        trail,
        [
            at(cafe, 1_200, 4_660),
            at(cafe, 1_200, 3_320),
            at(cafe, 1_200, 2_200)
        ]
    );
    assert_eq!(outcome, Ended::Arrived);

    // Again, and Alice walks 2 000 mm east after the first stride: the walk re-plans and follows.
    let mut town = Town::joined(|town| at(town.cafe, 1_200, 6_000));
    let (cafe, visitor, alice) = (town.cafe, town.visitor, town.alice);
    walk_to(&mut town, visitor, Destination::Person(as_person(alice)));
    step(&mut town, visitor);
    assert_eq!(town.presence(visitor), Some(at(cafe, 1_200, 4_660)));
    let (moved, _) = town.r#move(alice, at(cafe, 3_200, 1_000));
    assert!(matches!(moved, ActionResult::Accepted { .. }), "{moved:?}");
    let (_, outcome) = walk_out(&mut town, visitor);
    let (here, there) = (
        town.presence(visitor)
            .and_then(|l| l.local())
            .expect("positioned"),
        (3_200_i64, 1_000_i64),
    );
    let (dx, dy) = (
        i64::from(here.x().value()) - there.0,
        i64::from(here.y().value()) - there.1,
    );
    assert_eq!(outcome, Ended::Arrived);
    assert!(
        dx * dx + dy * dy <= 1_200 * 1_200,
        "within 1 200 mm of where she went: {here:?}"
    );
}

#[test]
fn every_refusal_is_the_one_its_rule_names() {
    let mut town = Town::joined(|town| at(town.street, 1_000, 2_000));
    let (attic, cafe, visitor, alice) = (town.attic, town.cafe, town.visitor, town.alice);
    let target = as_person(alice);
    let refused = |result: ActionResult| match result {
        ActionResult::Rejected(rejection) => rejection,
        other => panic!("refused, not {other:?}"),
    };
    // Alice is in the café, the visitor in the street.
    let (result, events) = walk_to(&mut town, visitor, Destination::Person(target));
    assert_eq!((refused(result), events.len()), (Rejection::TooFarAway, 0));
    // The attic joins nothing: no chain of passages.
    let (result, _) = walk_to(&mut town, visitor, Destination::Place(at(attic, 0, 0)));
    match refused(result) {
        Rejection::System { code, .. } => assert_eq!(code.as_str(), "no-route"),
        other => panic!("no-route, not {other:?}"),
    }
    // Oneself.
    let me = as_person(visitor);
    let (result, _) = walk_to(&mut town, visitor, Destination::Person(me));
    assert_eq!(refused(result), Rejection::PreconditionFailed);
    // A payload that is not a walk-to.
    let garbage = ActionRecord::new::<WalkTo>(br#"{"to":{"somewhere":1}}"#.to_vec());
    let (result, _) = town.submit(visitor, garbage);
    match refused(result) {
        Rejection::System { code, .. } => assert_eq!(code.as_str(), "malformed-payload"),
        other => panic!("malformed-payload, not {other:?}"),
    }
    // A step with no walk.
    let (result, _) = step(&mut town, visitor);
    assert_eq!(refused(result), Rejection::PreconditionFailed);
    // And a reachable place is accepted: the café through the doorway.
    let (result, _) = walk_to(
        &mut town,
        visitor,
        Destination::Place(at(cafe, 3_000, 2_000)),
    );
    assert!(
        matches!(result, ActionResult::Accepted { .. }),
        "{result:?}"
    );
}

#[test]
fn a_new_walk_replaces_the_old_and_a_move_stops_it() {
    let mut town = Town::joined(|town| at(town.cafe, 1_000, 5_000));
    let (cafe, visitor) = (town.cafe, town.visitor);
    walk_to(
        &mut town,
        visitor,
        Destination::Place(at(cafe, 4_000, 5_000)),
    );
    let (_, events) = walk_to(
        &mut town,
        visitor,
        Destination::Place(at(cafe, 1_000, 8_000)),
    );
    assert_eq!(kinds(&events), ["walk-ended", "walk-started"]);
    assert_eq!(ended(&events), Some(Ended::Replaced));
    step(&mut town, visitor);
    assert_eq!(
        town.presence(visitor),
        Some(at(cafe, 1_000, 6_340)),
        "the new walk's stride"
    );
    let (_, events) = town.r#move(visitor, at(cafe, 2_000, 6_340));
    assert_eq!(kinds(&events), ["walk-ended", "arrived"]);
    assert_eq!(ended(&events), Some(Ended::Stopped));
    assert!(!is_walking(&town.world.read(), as_person(visitor)));
}

/// NV-2 (g) at movement's level, and NV-11's "the same request at two instants yields the same
/// stride": a walk takes no calendar time.
#[test]
fn a_walk_takes_no_calendar_time() {
    let mut town = Town::joined(|town| at(town.cafe, 1_000, 5_000));
    let (cafe, visitor) = (town.cafe, town.visitor);
    walk_to(
        &mut town,
        visitor,
        Destination::Place(at(cafe, 4_000, 5_000)),
    );
    let before = town.world.components().get::<Walking>(visitor).cloned();
    let day = WorldTime::from_seconds(NOW.seconds() + SimDuration::from_seconds(86_400).seconds());
    let fired = town.world.advance_to(day).expect("the clock advances");
    println!("a day later: {fired:?}");
    assert_eq!(
        town.presence(visitor),
        Some(at(cafe, 1_000, 5_000)),
        "nobody stepped, nobody moved"
    );
    assert_eq!(
        town.world.components().get::<Walking>(visitor).cloned(),
        before,
        "the walk waits"
    );
    // The step a day later is the step a moment later would have been.
    let id = town.next_id();
    let intent = mineworld_contracts::ActionIntent::new(
        id,
        visitor,
        ActionRecord::new::<WalkStep>(encode(&WalkStep::default())),
        day,
    );
    let _ = town.world.dispatch(&intent, day).expect("dispatches");
    assert_eq!(town.presence(visitor), Some(at(cafe, 2_340, 5_000)));
}

#[test]
fn a_walker_discloses_the_walk_and_is_offered_the_step() {
    let mut town = Town::joined(|town| at(town.cafe, 1_000, 5_000));
    let (cafe, visitor, alice) = (town.cafe, town.visitor, town.alice);
    let offered = |town: &Town, who| {
        town.observation(who)
            .affordances()
            .iter()
            .map(|affordance| affordance.action_type().as_str().to_owned())
            .collect::<Vec<_>>()
    };
    assert!(offered(&town, visitor).contains(&"walk-to".to_owned()));
    assert!(!offered(&town, visitor).contains(&"walk-step".to_owned()));
    walk_to(
        &mut town,
        visitor,
        Destination::Place(at(cafe, 4_000, 5_000)),
    );
    assert!(offered(&town, visitor).contains(&"walk-step".to_owned()));
    let seen = town.observation(alice);
    let record = seen
        .entity(visitor)
        .and_then(|entity| {
            entity
                .components()
                .iter()
                .find(|record| record.component_type().as_str() == "walking")
        })
        .map(|record| record.payload().clone())
        .expect("Alice sees the visitor's walk");
    println!("disclosed: {record}");
    assert_eq!(
        record["next"],
        serde_json::json!([{ "x": 4_000, "y": 5_000, "z": 0 }]),
        "one waypoint: the straight leg"
    );
    assert!(record["destination"]["place"].is_object());
}

/// A world restored from a snapshot mid-walk continues the walk exactly as the original does.
#[test]
fn a_restored_world_continues_the_walk() {
    let mut town = Town::joined(|town| at(town.cafe, 2_600, 2_000));
    let (street, visitor) = (town.street, town.visitor);
    walk_to(
        &mut town,
        visitor,
        Destination::Place(at(street, 1_000, 2_000)),
    );
    step(&mut town, visitor);
    let snapshot = town.world.snapshot().expect("a snapshot");
    let (mut restored, _) = support::compose(Movement::Enabled);
    restored.restore(snapshot).expect("restores");
    let mut original_trail = Vec::new();
    let mut restored_trail = Vec::new();
    for raw in 0..3 {
        let intent = |world: &World| {
            let _ = world;
            mineworld_contracts::ActionIntent::new(
                mineworld_contracts::ActionId::from_raw(100 + raw),
                visitor,
                ActionRecord::new::<WalkStep>(encode(&WalkStep::default())),
                NOW,
            )
        };
        let a = town
            .world
            .dispatch(&intent(&town.world), NOW)
            .expect("dispatches");
        let b = restored
            .dispatch(&intent(&restored), NOW)
            .expect("dispatches");
        original_trail.push(encode(&a.events().to_vec()));
        restored_trail.push(encode(&b.events().to_vec()));
    }
    assert_eq!(original_trail, restored_trail, "byte for byte");
    assert_eq!(
        restored
            .components()
            .get::<mineworld_presence::Presence>(visitor)
            .map(|p| p.location()),
        Some(at(street, 1_000, 2_000))
    );
}
