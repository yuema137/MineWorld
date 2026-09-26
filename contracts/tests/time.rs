//! Contract tests for simulated time.
//!
//! Two value types with an order and a difference: that is all this PR provides, so that is all
//! there is to check. How time advances belongs to the scheduler.

use mineworld_contracts::{SimDuration, WorldTime};

/// A world's clock reaches before its own epoch, and comparing two moments orders them
/// chronologically across that boundary. An unsigned clock would make the first of these values
/// the largest instead of the smallest.
#[test]
fn moments_order_chronologically_including_before_the_epoch() {
    let long_ago = WorldTime::from_seconds(-6_307_200_000);
    let a_minute_early = WorldTime::from_seconds(-60);
    let later = WorldTime::from_seconds(60);

    assert!(long_ago < a_minute_early);
    assert!(a_minute_early < WorldTime::EPOCH);
    assert!(WorldTime::EPOCH < later);

    let mut moments = [later, long_ago, WorldTime::EPOCH, a_minute_early];
    moments.sort();
    assert_eq!(moments, [long_ago, a_minute_early, WorldTime::EPOCH, later]);
}

/// The difference between two moments is a duration, and it is signed: asking how long after a
/// later moment an earlier one is gives a negative answer rather than a wrong positive one.
#[test]
fn the_difference_between_two_moments_is_signed() {
    let start = WorldTime::from_seconds(1_000);
    let end = WorldTime::from_seconds(1_090);

    assert_eq!(
        end.duration_since(start),
        Some(SimDuration::from_seconds(90))
    );
    assert_eq!(
        start.duration_since(end),
        Some(SimDuration::from_seconds(-90))
    );
    assert_eq!(start.duration_since(start), Some(SimDuration::ZERO));

    // A difference that cannot be expressed is refused rather than wrapped: a contract type that
    // returned a wrong duration here would corrupt anything scheduling on it.
    assert_eq!(
        WorldTime::from_seconds(i64::MAX).duration_since(WorldTime::from_seconds(-1)),
        None
    );
}

/// Both types are stored as bare integers — the shape persistence and the event log read back.
#[test]
fn simulated_time_is_stored_as_bare_seconds() {
    assert_eq!(
        serde_json::to_string(&WorldTime::from_seconds(-60)).unwrap(),
        "-60"
    );
    assert_eq!(
        serde_json::to_string(&SimDuration::from_seconds(90)).unwrap(),
        "90"
    );
    assert_eq!(
        serde_json::from_str::<WorldTime>("0").unwrap(),
        WorldTime::EPOCH
    );
    assert_eq!(
        serde_json::from_str::<SimDuration>("-90").unwrap(),
        SimDuration::from_seconds(-90)
    );
}
