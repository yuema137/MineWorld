//! Every refusal of `sit` and `stand`, each produced by a request built for it, and the observer's
//! offer for the same point carrying the same reason (`ARC-147` item 4, I-a-1; AC-a-2, AC-a-4).

mod support;

use mineworld_contracts::{ActionRecord, ActionResult, Rejection};
use mineworld_movement::{Destination, WalkTo};
use mineworld_posture::Sit;
use support::{Layout, Town, accepted, at, record, sit, stand};

/// Asks dispatch and the observation the same question at the same instant: the offer first, so
/// nothing the request does can change it.
fn both(town: &mut Town, when: i64, who: &str, point: &str) -> (Option<Rejection>, ActionResult) {
    let offer = town
        .sit_offer(who, point, when)
        .unwrap_or_else(|| panic!("{who} is offered sit on {point}"));
    let offered = offer.unavailable_reason().cloned();
    let (result, _) = town.submit(when, who, sit(point));
    (offered, result)
}

fn refused(reason: Rejection) -> ActionResult {
    ActionResult::Rejected(reason)
}

#[test]
fn each_sit_refusal_is_dispatched_and_offered_alike() {
    let mut town = Town::begin(&Layout::cafe(), "refusals");

    // Admissible: offered available, and accepted.
    let (offered, result) = both(&mut town, 10, "carol", "bench.2");
    assert_eq!(offered, None);
    assert!(accepted(&result), "{result:?}");

    // Too far: bob is 1 400 mm from bench.2 and farther from the rest.
    let (offered, result) = both(&mut town, 11, "bob", "bench.1");
    assert_eq!(offered, Some(Rejection::TooFarAway));
    assert_eq!(result, refused(Rejection::TooFarAway));

    // Taken and out of reach: reach is asked before occupancy, so alice (781 mm from bench.2, which
    // carol sits on) hears TooFarAway. The taken-and-reachable row is the contention test below.
    let (offered, result) = both(&mut town, 12, "alice", "bench.2");
    assert_eq!(
        offered,
        Some(Rejection::TooFarAway),
        "781 mm: out of reach first"
    );
    assert_eq!(result, refused(Rejection::TooFarAway));

    // Crowded: eve stands 300 mm from stool.1, which frank can reach.
    let (offered, result) = both(&mut town, 13, "frank", "stool.1");
    assert_eq!(offered, Some(Rejection::TargetUnavailable));
    assert_eq!(result, refused(Rejection::TargetUnavailable));

    // Busy, seated: carol asks for bench.2 again — she is offered stand, not sit.
    let (result, _) = town.submit(14, "carol", sit("bench.2"));
    assert_eq!(result, refused(Rejection::Busy));
    assert!(town.sit_offer("carol", "bench.2", 14).is_none());

    // Busy, walking.
    let cafe = town.place("cafe");
    let (result, _) = town.submit(
        15,
        "bob",
        record(&WalkTo::new(Destination::Place(at(cafe, (4_000, 3_000))))),
    );
    assert!(accepted(&result), "{result:?}");
    let (offered, result) = both(&mut town, 16, "bob", "bench.1");
    assert_eq!(offered, Some(Rejection::Busy));
    assert_eq!(result, refused(Rejection::Busy));

    // No such point in the actor's place: not offered, and not supported.
    assert!(town.sit_offer("alice", "window.9", 17).is_none());
    let (result, _) = town.submit(17, "alice", sit("window.9"));
    assert_eq!(result, refused(Rejection::NoSupportedInteraction));

    // A malformed payload is this pack's own refusal.
    let (result, _) = town.submit(
        18,
        "alice",
        ActionRecord::new::<Sit>(b"{\"point\":\"Not A Key\"}".to_vec()),
    );
    assert!(
        matches!(result, ActionResult::Rejected(Rejection::System { ref code, .. }) if code.as_str() == "malformed-payload"),
        "{result:?}"
    );

    // Standing when not seated is a precondition.
    let (result, _) = town.submit(19, "alice", stand());
    assert_eq!(result, refused(Rejection::PreconditionFailed));
}

#[test]
fn a_place_cannot_sit() {
    let mut town = Town::begin(&Layout::cafe(), "place-sits");
    let (result, _) = town.submit(10, "cafe", sit("bench.1"));
    assert_eq!(result, refused(Rejection::PreconditionFailed));
}

#[test]
fn two_requests_for_one_point_at_one_instant_seat_one_person() {
    // AC-a-4, X-S34-1's witness: alice (500 mm) and dave (600 mm) both reach bench.1.
    let mut town = Town::begin(&Layout::cafe(), "contention");
    let (first, _) = town.submit(10, "alice", sit("bench.1"));
    let (second, facts) = town.submit(10, "dave", sit("bench.1"));
    assert!(accepted(&first), "{first:?}");
    assert_eq!(second, refused(Rejection::TargetUnavailable));
    assert!(facts.is_empty());
    assert!(town.posture("dave").is_none());
    assert_eq!(
        town.sit_offer("dave", "bench.1", 10)
            .and_then(|offer| offer.unavailable_reason().cloned()),
        Some(Rejection::TargetUnavailable)
    );
}
