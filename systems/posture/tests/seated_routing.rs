//! S34-b amendment A1 (`pr-s34b-bodies-anchoring.md` §13, SD-b-A1): a walker's route goes round a
//! seated person rather than through them, and still exists when a sitter blocks the only way.
//!
//! ```text
//! AC-b-A1  café: dave walks north past alice on bench.1; he arrives, never stopped short, and alice
//!          is where she sat, still seated
//! AC-b-A2  a 1 m corridor with a sit point in its middle: the walk past the sitter is still accepted
//!          (the plain plan, as before A1), and the walker is stopped short at the sitter
//! ```

mod support;

use mineworld_contracts::EventEnvelope;
use mineworld_movement::{Destination, WalkStep, WalkTo};
use support::{Layout, Town, accepted, at, record, sit, types};

/// The café of `anchoring.rs`, 8 × 3 m, with its bench and stool.
const CAFE_SHAPED: &str = "\
tags: [cafe]
body:
  floor:
    min: { x: 0, y: 0 }
    max: { x: 8000, y: 3000 }
passages:
  - to: street
    here: { x: 2000, y: 2500 }
    there: { x: 500, y: 500 }
sit-points:
  - { key: bench.1, group: bench, at: { x: 2000, y: 1000 }, facing: 180000 }
  - { key: bench.2, group: bench, at: { x: 2600, y: 1000 }, facing: 180000 }
  - { key: stool.1, at: { x: 6000, y: 1000 }, facing: 0 }
";

const CAFE_PEOPLE: &[(&str, &str, i32, i32)] = &[
    ("alice", "cafe", 2_000, 1_500),
    ("dave", "cafe", 2_000, 400),
    ("ivy", "street", 500, 500),
];

/// A dead-end corridor 1 m wide and 8 m long, a sit point in its middle: nobody passes a sitter there.
const CORRIDOR: &str = "\
tags: [corridor]
body:
  floor:
    min: { x: 0, y: 0 }
    max: { x: 1000, y: 8000 }
passages:
  - to: street
    here: { x: 500, y: 7600 }
    there: { x: 500, y: 500 }
sit-points:
  - { key: stool.1, at: { x: 500, y: 4000 }, facing: 0 }
";

const CORRIDOR_PEOPLE: &[(&str, &str, i32, i32)] = &[
    ("gus", "corridor", 500, 3_500),
    ("hal", "corridor", 500, 1_000),
    ("ivy", "street", 500, 500),
];

fn layout(
    place: &'static str,
    text: &'static str,
    people: &'static [(&'static str, &'static str, i32, i32)],
) -> Layout {
    Layout {
        systems: vec!["presence", "movement", "bodies", "posture"],
        places: vec![(place, text), ("street", "tags: [street]\n")],
        people,
        posture: None,
    }
}

/// `walker`'s walk to `to` in its own place, stepped until it ends or `steps` steps have run: every
/// fact, and whether the walk-to was accepted.
fn walk(town: &mut Town, walker: &str, to: (i32, i32), steps: i64) -> (bool, Vec<EventEnvelope>) {
    let place = town.location(walker).place();
    let (result, mut facts) = town.submit(
        100,
        walker,
        record(&WalkTo::new(Destination::Place(at(place, to)))),
    );
    if !accepted(&result) {
        println!("{walker}'s walk-to: {result:?}");
        return (false, facts);
    }
    for step in 1..=steps {
        let (_, more) = town.submit(100 + step, walker, record(&WalkStep {}));
        let ended = types(&more).iter().any(|kind| kind == "walk-ended");
        facts.extend(more);
        if ended {
            break;
        }
    }
    (true, facts)
}

fn count(facts: &[EventEnvelope], kind: &str) -> usize {
    types(facts).iter().filter(|seen| *seen == kind).count()
}

#[test]
fn a_walker_goes_round_a_seated_person() {
    // AC-b-A1: alice sits on bench.1; dave, 600 mm south of it, walks to 1.2 m north of it.
    let mut town = Town::begin(
        &layout("cafe", CAFE_SHAPED, CAFE_PEOPLE),
        "seated-routing-cafe",
    );
    let (result, _) = town.submit(10, "alice", sit("bench.1"));
    assert!(accepted(&result), "{result:?}");
    let (seated_at, row) = (town.location("alice"), town.posture("alice"));
    let (began, facts) = walk(&mut town, "dave", (2_000, 2_200), 30);
    println!("dave: {:?}", types(&facts));
    assert!(began, "dave's walk is accepted");
    assert_eq!(
        count(&facts, "stopped-short"),
        0,
        "dave goes round alice instead of being stopped short: {:?}",
        types(&facts)
    );
    let cafe = town.place("cafe");
    assert_eq!(
        town.location("dave"),
        at(cafe, (2_000, 2_200)),
        "dave arrives"
    );
    assert_eq!(town.location("alice"), seated_at, "alice was not moved");
    assert_eq!(town.posture("alice"), row, "alice is still seated");
}

#[test]
fn a_sitter_who_blocks_the_only_way_still_lets_the_walk_begin() {
    // AC-b-A2: gus sits in the corridor's middle; hal walks to its far end, past him.
    let mut town = Town::begin(
        &layout("corridor", CORRIDOR, CORRIDOR_PEOPLE),
        "seated-routing-corridor",
    );
    let (result, _) = town.submit(10, "gus", sit("stool.1"));
    assert!(accepted(&result), "{result:?}");
    let seated_at = town.location("gus");
    let (began, facts) = walk(&mut town, "hal", (500, 7_000), 10);
    println!("hal: {:?}", types(&facts));
    assert!(began, "the walk is accepted, planned without the sitter");
    assert!(
        count(&facts, "stopped-short") > 0,
        "hal is stopped short at gus, as before A1: {:?}",
        types(&facts)
    );
    assert_eq!(town.location("gus"), seated_at, "gus was not moved");
    assert!(town.posture("gus").is_some(), "gus is still seated");
}
