//! S35 N-g: walkers do not jam each other (`DECISIONS.md` `ARC-333`; AC-Ng-1 … AC-Ng-3).
//!
//! ```text
//! same door      five walkers leave a one-door room for the street in the same step; every walk
//!                ends `arrived` within 60 steps each (F-Ng-1, AC-Ng-1)
//! head-on door   two walkers meet in a 1 m doorway, one going in and one going out; both arrive, and
//!                the one whose walk ranks lower gives way (AC-Ng-2)
//! queue          a walker queued behind a walker of higher rank waits rather than stalls, walks on
//!                when the queue clears, and still ends `stalled` within WAITS_MAX when it never does
//!                (ARC-333 R2)
//! street leg     Market Town's street, as its place file states it: walks to the store's door from
//!                the pavement end with no stop by a solid (F-Ng-4, AC-Ng-3)
//! ```
//!
//! Every request is made at the same instant (`support::NOW`), so walks that start together rank by
//! entity id: the person listed first in a plan has the lower id and ranks higher.

mod support;

use mineworld_bodies::PlaceShape;
use mineworld_contracts::{ActionRecord, Location};
use mineworld_movement::{Destination, Ended, WalkEnded, WalkStep, WalkTo};
use support::{CLEAR, Fact, GAP, Moved, Plan, R, Yard, distance2, encode, shape, xy};

/// How a walk ended, from the facts of one `walk-step`.
fn walk_outcome(moved: &Moved) -> Option<Ended> {
    moved.events.iter().find_map(|event| {
        event
            .payload()
            .payload_for::<WalkEnded>()
            .ok()
            .map(|payload| {
                let ended: WalkEnded =
                    serde_json::from_slice(payload).expect("movement's encoding");
                ended.outcome()
            })
    })
}

/// Starts a walk for `person` to `to`.
fn walk_to(yard: &mut Yard, person: &str, to: Location) {
    let walk = yard.submit(
        person,
        ActionRecord::new::<WalkTo>(encode(&WalkTo::new(Destination::Place(to)))),
    );
    assert!(walk.accepted(), "{person}'s walk-to: {:?}", walk.result);
}

/// Steps every walker in turn, in the order given, until each walk ends or `bound` rounds have run;
/// checks the invariant in `place` after every step. Each walker's outcome and the round it ended in,
/// and every stop by a solid (a `stopped-short` naming nobody).
fn step_all(
    yard: &mut Yard,
    walkers: &[&'static str],
    place: &str,
    bound: usize,
) -> (Vec<Option<(usize, Ended)>>, usize) {
    let mut ended: Vec<Option<(usize, Ended)>> = vec![None; walkers.len()];
    let mut by_solid = 0;
    for round in 1..=bound {
        for (index, walker) in walkers.iter().enumerate() {
            if ended[index].is_some() {
                continue;
            }
            let moved = yard.submit(
                walker,
                ActionRecord::new::<WalkStep>(encode(&WalkStep::default())),
            );
            assert!(
                moved.accepted(),
                "{walker}, round {round}: {:?}",
                moved.result
            );
            for fact in moved.facts() {
                if let Fact::StoppedShort {
                    by: None, reached, ..
                } = fact
                {
                    println!("round {round}: {walker} stopped by a solid at {reached:?}");
                    by_solid += 1;
                }
            }
            if let Some((a, b, closest)) = yard.closest(place) {
                assert!(
                    closest >= i64::from(CLEAR).pow(2),
                    "round {round}: {a} and {b} stand {} mm apart",
                    closest.isqrt()
                );
            }
            if let Some(outcome) = walk_outcome(&moved) {
                ended[index] = Some((round, outcome));
            }
        }
        if ended.iter().all(Option::is_some) {
            break;
        }
    }
    (ended, by_solid)
}

/// AC-Ng-1 (F-Ng-1): a one-door room, 4 × 3 m, its doorway point 0.4 m inside the south wall; five
/// walkers set off for the street in the same step, as Market Town's residents do at 06:00 — once
/// standing in a row across the room, once queued in a cross before the door. On N-f's head one walk
/// of each ends `stalled`; now every walk ends `arrived` within 60 `walk-step`s.
#[test]
fn five_walkers_leave_a_one_door_room_together() {
    let layouts: [[(i32, i32); 5]; 2] = [
        [
            (800, 1_000),
            (1_400, 1_000),
            (2_000, 1_000),
            (2_600, 1_000),
            (3_200, 1_000),
        ],
        [
            (2_000, 1_100),
            (1_400, 1_100),
            (2_600, 1_100),
            (2_000, 1_700),
            (2_000, 2_300),
        ],
    ];
    let walkers = ["a", "b", "c", "d", "e"];
    for layout in layouts {
        let room = shape((0, 0, 4_000, 3_000), &[]);
        let mut yard = Yard::new(&Plan {
            places: vec![("street", None), ("room", Some(room))],
            people: walkers
                .iter()
                .zip(layout)
                .map(|(walker, at)| (*walker, "room", Some(at)))
                .collect(),
            passages: vec![(("street", (1_000, 1_000)), ("room", (2_000, 400)))],
            ..Plan::default()
        });
        let street = Location::in_place(yard.places["street"]);
        for walker in walkers {
            walk_to(&mut yard, walker, street);
        }
        let (ended, _) = step_all(&mut yard, &walkers, "room", 60);
        println!("same door from {layout:?}: {ended:?}");
        for (walker, outcome) in walkers.iter().zip(&ended) {
            assert_eq!(
                outcome.map(|(_, outcome)| outcome),
                Some(Ended::Arrived),
                "{walker} leaves the room (F-Ng-1: walkers stall one another at the door)"
            );
            assert_eq!(yard.location(walker).place(), yard.places["street"]);
        }
    }
}

/// AC-Ng-2: a hall 4 × 6 m split by a wall 1 m thick with a doorway 1 m wide and 1 m deep in its
/// middle. "first" walks from the north half to the south and "second" from the south half to the
/// north, both through the doorway, started in the same step; "first" has the lower id, so its walk
/// ranks higher. Both arrive, and only "second" ever steps aside for the other farther than a nudge
/// (which every walker may still give anyone, step-11 SD-B7). On N-f's head this geometry already
/// passes by nudges and the head-on bias; the scenario guards that R1 keeps it so.
#[test]
fn two_walkers_meeting_in_a_narrow_doorway_both_pass() {
    let hall = shape(
        (0, 0, 4_000, 6_000),
        &[
            ((0, 2_500, 1_500, 3_500), 2_500),
            ((2_500, 2_500, 4_000, 3_500), 2_500),
        ],
    );
    let mut yard = Yard::new(&Plan {
        places: vec![("hall", Some(hall))],
        people: vec![
            ("first", "hall", Some((2_000, 5_000))),
            ("second", "hall", Some((2_000, 1_000))),
        ],
        ..Plan::default()
    });
    let south = yard.at("hall", (3_000, 800));
    let north = yard.at("hall", (1_000, 5_200));
    walk_to(&mut yard, "first", south);
    walk_to(&mut yard, "second", north);
    let walkers = ["first", "second"];
    let mut ended: Vec<Option<Ended>> = vec![None; 2];
    // A step aside longer than a nudge (NUDGE_MAX + GAP = 260 mm): a give-way move, not a nudge.
    let nudge_max = i64::from(R + GAP);
    let mut gave_way: Vec<(&'static str, i64)> = Vec::new();
    for round in 1..=40 {
        for (index, walker) in walkers.iter().enumerate() {
            if ended[index].is_some() {
                continue;
            }
            let before: Vec<Option<(i32, i32)>> =
                walkers.iter().map(|other| yard.point(other)).collect();
            let moved = yard.submit(
                walker,
                ActionRecord::new::<WalkStep>(encode(&WalkStep::default())),
            );
            assert!(
                moved.accepted(),
                "{walker}, round {round}: {:?}",
                moved.result
            );
            println!(
                "round {round} {walker}: first {:?}, second {:?}; {:?}",
                yard.point("first"),
                yard.point("second"),
                moved.facts()
            );
            for (person, at) in moved.displaced() {
                let key = yard.key_of(person);
                let other = walkers.iter().position(|w| *w == key).expect("a walker");
                // Only while both walks are in progress: a walker whose walk has ended is idle, and
                // the idle give way to every walker (ARC-330 R2).
                if let (Some(from), true) = (before[other], ended.iter().all(Option::is_none)) {
                    let stepped = distance2(from, xy(at)).isqrt();
                    if stepped > nudge_max {
                        gave_way.push((key, stepped));
                    }
                }
            }
            if let Some((a, b, closest)) = yard.closest("hall") {
                assert!(
                    closest >= i64::from(CLEAR).pow(2),
                    "round {round}: {a} and {b} stand {} mm apart",
                    closest.isqrt()
                );
            }
            ended[index] = walk_outcome(&moved);
        }
        if ended.iter().all(Option::is_some) {
            break;
        }
    }
    println!("head-on doorway: {ended:?}; stepped aside {gave_way:?}");
    assert_eq!(ended, vec![Some(Ended::Arrived), Some(Ended::Arrived)]);
    assert!(
        gave_way.iter().all(|(person, _)| *person == "second"),
        "while both walk, only the walk that ranks lower steps aside: {gave_way:?}"
    );
}

/// Market Town's street body, decoded from its place file exactly as the loader decodes a `body:`
/// section.
fn market_town_street() -> PlaceShape {
    #[derive(serde::Deserialize)]
    struct File {
        body: PlaceShape,
    }
    let text = include_str!("../../../worlds/market-town/places/street.yaml");
    serde_saphyr::from_str::<File>(text)
        .expect("the street's place file")
        .body
}

/// AC-Ng-3 (F-Ng-4): on Market Town's street, walks from the pavement to the store's doorway point
/// (8 390, 2 800) — including from the point where Market Town's walkers were stopped again and again,
/// (7 074, 1 214) — arrive with no stop by a solid.
#[test]
#[ignore = "F-Ng-4: red until ARC-333 R3"]
fn the_street_leg_to_the_store_meets_no_solid() {
    let starts = [(7_074, 1_214), (4_000, 0), (13_000, 1_500), (6_000, -6_000)];
    let mut stops = 0;
    for start in starts {
        let mut yard = Yard::new(&Plan {
            places: vec![("street", Some(market_town_street())), ("store", None)],
            people: vec![("walker", "street", Some(start))],
            passages: vec![(("street", (8_390, 2_800)), ("store", (1_040, 400)))],
            ..Plan::default()
        });
        let store = Location::in_place(yard.places["store"]);
        walk_to(&mut yard, "walker", store);
        let (ended, by_solid) = step_all(&mut yard, &["walker"], "street", 40);
        println!("street leg from {start:?}: {ended:?}, {by_solid} stops by a solid");
        assert_eq!(
            ended[0].map(|(_, outcome)| outcome),
            Some(Ended::Arrived),
            "from {start:?}"
        );
        stops += by_solid;
    }
    assert_eq!(stops, 0, "no stride on the leg is stopped by a solid");
}

/// A dead-end corridor 1 m wide and 8 m long — too narrow for two people side by side — whose one
/// door, 1 m from its north end, leads to the street. "ahead" stands 1.6 m from that end with a walk
/// to the street in progress; "behind", who ranks lower (same instant, higher id), walks to the
/// corridor's far end, past "ahead". How "behind"'s walk ended, and after how many of its own steps —
/// `None` when it is still walking after `steps`. When `ahead_goes_after` is given, "ahead" walks out
/// through the door after that many of "behind"'s steps.
fn queue_behind(steps: usize, ahead_goes_after: Option<usize>) -> Option<(usize, Ended)> {
    let corridor = shape((0, 0, 1_000, 8_000), &[]);
    let mut yard = Yard::new(&Plan {
        places: vec![("street", None), ("corridor", Some(corridor))],
        people: vec![
            ("ahead", "corridor", Some((500, 6_400))),
            ("behind", "corridor", Some((500, 3_000))),
        ],
        passages: vec![(("street", (1_000, 1_000)), ("corridor", (500, 7_000)))],
        ..Plan::default()
    });
    let street = Location::in_place(yard.places["street"]);
    let far_end = yard.at("corridor", (500, 7_700));
    walk_to(&mut yard, "ahead", street);
    walk_to(&mut yard, "behind", far_end);
    for step in 1..=steps {
        if ahead_goes_after == Some(step - 1) {
            let (ended, _) = step_all(&mut yard, &["ahead"], "corridor", 5);
            assert_eq!(ended[0].map(|(_, outcome)| outcome), Some(Ended::Arrived));
        }
        let moved = yard.submit(
            "behind",
            ActionRecord::new::<WalkStep>(encode(&WalkStep::default())),
        );
        if let Some(outcome) = walk_outcome(&moved) {
            return Some((step, outcome));
        }
    }
    None
}

/// ARC-333 R2: a walker queued behind a walker of higher rank who does not move is waiting, not
/// stalled — its walk is still in progress after more than STALLS_MAX stopped steps — and it walks on
/// when the one ahead goes; a queue that never clears still ends the walk `stalled`, after WAITS_MAX
/// waiting steps (and the few strides it took to reach the queue and settle into it).
#[test]
fn waiting_behind_a_higher_walker_is_not_a_stall() {
    let waits = mineworld_movement::WAITS_MAX as usize;
    assert_eq!(
        queue_behind(8, None),
        None,
        "still waiting after 8 steps (STALLS_MAX is {})",
        mineworld_movement::STALLS_MAX
    );
    let then = queue_behind(waits + 10, Some(8));
    println!("the queue clears after 8 steps: {then:?}");
    assert!(matches!(then, Some((_, Ended::Arrived))), "{then:?}");
    let never = queue_behind(waits + 10, None);
    println!("the queue never clears: {never:?}");
    assert!(
        matches!(never, Some((step, Ended::Stalled)) if step > waits && step <= waits + 8),
        "{never:?}"
    );
}
