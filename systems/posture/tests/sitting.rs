//! Sitting down, being seen seated, standing up, and standing up because one moved (`ARC-147`;
//! AC-a-1, AC-a-3, AC-a-8). AC-a-9 — a seated person nudged or shoved off their point stands up — was
//! replaced by S34-b (SD-b-8): a seated person is anchored, and `tests/anchoring.rs` holds the walker and
//! the shove.
//!
//! What each test would pass without proving, and how it is excluded (`ARC-23`): a seat taken without
//! moving (the sitter starts 500 mm away and the position is checked to be the point's); a bystander
//! who sees the row only because they are the sitter (a second observer is asked); an auto-stand that
//! fires on the sitter's own arrival (sitting is checked to leave them seated); a world that hides `sit`
//! merely by having no sit points (the posture-less world keeps the café's layout).

mod support;

use mineworld_contracts::{ActionResult, Millidegrees, Orientation};
use mineworld_movement::{Destination, Move, WalkTo};
use mineworld_posture::Posture;
use support::{Layout, Town, accepted, at, key, record, sit, stand, stood_up_cause, t, types};

#[test]
fn a_person_within_reach_sits_on_the_point_is_seen_seated_and_stands_where_they_sat() {
    let mut town = Town::begin(&Layout::cafe(), "sit-and-stand");
    let cafe = town.place("cafe");

    let (result, facts) = town.submit(10, "alice", sit("bench.1"));
    assert!(accepted(&result), "{result:?}");
    assert_eq!(types(&facts), ["arrived", "sat-down"]);
    let seated_at = town.location("alice");
    assert_eq!(seated_at.place(), cafe);
    assert_eq!(
        seated_at,
        at(cafe, (2_000, 1_000)).with_facing(Orientation::facing(Millidegrees::new(180_000)))
    );
    assert_eq!(
        town.posture("alice"),
        Some(Posture::seated(cafe, key("bench.1"), t(10))),
        "seated by her own arrival, not stood up by it"
    );

    // The sitter and a bystander both see her posture-state; the bystander also sees the café's sit
    // points as place data.
    for observer in ["alice", "bob"] {
        let seen = town.observation(observer, 11);
        let alice = seen.entity(town.id("alice")).expect("alice is perceived");
        assert!(
            alice
                .components()
                .iter()
                .any(|record| record.component_type().as_str() == "posture-state"),
            "{observer} sees alice seated"
        );
        let place = seen
            .entity(cafe.entity_id())
            .expect("the café is perceived");
        assert!(
            place
                .components()
                .iter()
                .any(|record| record.component_type().as_str() == "sit-points"),
            "{observer} sees the café's sit points"
        );
    }
    // Seated, she is offered stand and nothing to sit on.
    let offered: Vec<String> = town
        .observation("alice", 11)
        .affordances()
        .iter()
        .filter(|offer| matches!(offer.action_type().as_str(), "sit" | "stand"))
        .map(|offer| offer.action_type().as_str().to_owned())
        .collect();
    assert_eq!(offered, ["stand"]);

    let (result, facts) = town.submit(20, "alice", stand());
    assert!(accepted(&result), "{result:?}");
    assert_eq!(types(&facts), ["stood-up"]);
    assert_eq!(stood_up_cause(&facts).as_deref(), Some("requested"));
    assert_eq!(
        town.location("alice"),
        seated_at,
        "standing does not move her"
    );
    assert_eq!(town.posture("alice"), None);
}

#[test]
fn a_seated_person_who_steps_away_stands_up_in_the_same_dispatch() {
    let mut town = Town::begin(&Layout::cafe(), "step-away");
    let cafe = town.place("cafe");
    town.submit(10, "alice", sit("bench.1"));

    let (result, facts) = town.submit(20, "alice", record(&Move::new(at(cafe, (2_000, 1_600)))));
    assert!(accepted(&result), "{result:?}");
    assert_eq!(types(&facts), ["arrived", "stood-up"]);
    assert_eq!(stood_up_cause(&facts).as_deref(), Some("moved"));
    assert_eq!(town.posture("alice"), None);
}

#[test]
fn turning_on_the_spot_keeps_a_seated_person_seated() {
    let mut town = Town::begin(&Layout::cafe(), "turn");
    let cafe = town.place("cafe");
    town.submit(10, "alice", sit("bench.1"));

    let turned =
        at(cafe, (2_000, 1_000)).with_facing(Orientation::facing(Millidegrees::new(90_000)));
    let (result, facts) = town.submit(20, "alice", record(&Move::new(turned)));
    assert!(accepted(&result), "{result:?}");
    assert_eq!(
        types(&facts),
        ["arrived"],
        "a new facing alone stands nobody up (BC-2)"
    );
    assert!(town.posture("alice").is_some());
}

#[test]
fn a_seated_person_who_leaves_the_place_or_sets_off_on_a_walk_stands_up() {
    let mut town = Town::begin(&Layout::cafe(), "leave");
    let street = town.place("street");
    town.submit(10, "alice", sit("bench.1"));
    let (result, facts) = town.submit(20, "alice", record(&Move::new(at(street, (500, 900)))));
    assert!(accepted(&result), "{result:?}");
    assert!(
        types(&facts).contains(&"stood-up".to_owned()),
        "{:?}",
        types(&facts)
    );
    assert_eq!(town.posture("alice"), None);

    let mut town = Town::begin(&Layout::cafe(), "walk-off");
    let cafe = town.place("cafe");
    let (result, _) = town.submit(10, "alice", sit("bench.1"));
    assert!(accepted(&result), "{result:?}");
    let (result, facts) = town.submit(
        20,
        "alice",
        record(&WalkTo::new(Destination::Place(at(cafe, (4_000, 3_000))))),
    );
    assert!(accepted(&result), "{result:?}");
    assert_eq!(types(&facts), ["walk-started", "stood-up"]);
    assert_eq!(stood_up_cause(&facts).as_deref(), Some("moved"));
}

#[test]
fn a_world_without_posture_offers_no_sit_and_refuses_it_as_unavailable() {
    let mut town = Town::begin(&Layout::cafe().without_posture(), "no-posture");
    assert!(
        town.observation("alice", 1)
            .affordances()
            .iter()
            .all(|offer| offer.action_type().as_str() != "sit")
    );
    let (result, facts) = town.submit(10, "alice", sit("bench.1"));
    assert_eq!(result, ActionResult::Unavailable);
    assert!(facts.is_empty());
}
