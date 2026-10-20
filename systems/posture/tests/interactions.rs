//! The `posture` section of the World's Interaction List, read from `configure/posture.yaml` by the
//! real loader (`ARC-63`, `ARC-147` item 9; AC-a-7), and the `sit-points:` section's load refusals
//! (AC-a-6's loader half).

mod support;

use mineworld_contracts::{ActionResult, Rejection};
use mineworld_worldpack::WorldPack;
use support::{Layout, Town, accepted, sit};

#[test]
fn a_list_that_forbids_sit_is_answered_permission_denied_by_offer_and_dispatch() {
    let layout =
        Layout::cafe().configured("rules:\n  - { action: sit, actor: person, effect: forbid }\n");
    let mut town = Town::begin(&layout, "forbidden");
    let offer = town
        .sit_offer("alice", "bench.1", 10)
        .expect("still offered");
    assert_eq!(
        offer.unavailable_reason(),
        Some(&Rejection::PermissionDenied)
    );
    let (result, _) = town.submit(10, "alice", sit("bench.1"));
    assert_eq!(result, ActionResult::Rejected(Rejection::PermissionDenied));
}

#[test]
fn a_shorter_reach_makes_a_near_point_too_far() {
    // alice is 500 mm from bench.1: inside the default 750, outside a configured 300.
    let short = Layout::cafe().configured("parameters:\n  - { reach: 300 }\n");
    let mut town = Town::begin(&short, "short-reach");
    let offer = town.sit_offer("alice", "bench.1", 10).expect("offered");
    assert_eq!(offer.unavailable_reason(), Some(&Rejection::TooFarAway));
    let (result, _) = town.submit(10, "alice", sit("bench.1"));
    assert_eq!(result, ActionResult::Rejected(Rejection::TooFarAway));

    let mut town = Town::begin(&Layout::cafe(), "default-reach");
    let (result, _) = town.submit(10, "alice", sit("bench.1"));
    assert!(accepted(&result), "{result:?}");
}

#[test]
fn a_list_cannot_govern_stand_nor_set_reach_beyond_its_bound() {
    let stand = Layout::cafe().configured("rules:\n  - { action: stand, effect: forbid }\n");
    let refusal = stand.refusal("governs-stand");
    assert!(refusal.contains("stand"), "{refusal}");

    let far = Layout::cafe().configured("parameters:\n  - { reach: 5000 }\n");
    let refusal = far.refusal("reach-too-far");
    assert!(refusal.contains("reach"), "{refusal}");
}

#[test]
fn sit_points_too_close_are_refused_at_load_naming_both() {
    let mut layout = Layout::cafe();
    layout.places[0].1 = "tags: [cafe]\nsit-points:\n  - { key: a, at: { x: 0, y: 0 }, facing: 0 }\n  - { key: b, at: { x: 509, y: 0 }, facing: 0 }\n";
    let refusal = layout.refusal("too-close");
    assert!(
        refusal.contains("closer than 510") && refusal.contains(" a ") && refusal.contains(" b "),
        "{refusal}"
    );
}

#[test]
fn a_sit_points_section_in_a_person_file_is_refused_at_load() {
    let layout = Layout::cafe();
    let scratch = layout.write("person-sits");
    std::fs::write(
        scratch.path().join("people/alice.yaml"),
        "location:\n  place: cafe\n  position: { x: 2000, y: 1500 }\nsit-points:\n  - { key: lap, at: { x: 0, y: 0 }, facing: 0 }\n",
    )
    .expect("writable");
    let refusal = WorldPack::read(scratch.path())
        .and_then(|pack| pack.assemble().map(|_| ()))
        .expect_err("refused")
        .to_string();
    assert!(refusal.contains("sit-points"), "{refusal}");
}
