//! `admit`'s table of reasons (design §5.3): each check, in its order, against hand-built drafts.
//! Owns "a check is missing, reordered, or off by one at a horizon or a limit".

use mineworld_contracts::{EntityId, EntityKey, EntityType, PlaceId, WorldTime};

use super::{Admission, WithheldReason};
use crate::HappeningsConfiguration;
use crate::event::HappeningsConfigured;
use crate::generate::{DayIndex, HappeningDraft};
use crate::kind::HappeningKindId;

const DAY: i64 = 86_400;

/// Two places: street (10) and park (11).
fn place(key: &EntityKey) -> Result<PlaceId, ()> {
    let raw = match key.as_str() {
        "street" => 10,
        "park" => 11,
        _ => return Err(()),
    };
    PlaceId::new(EntityId::from_raw(raw), EntityType::Place).map_err(|_| ())
}

fn street() -> PlaceId {
    place(&EntityKey::new("street").expect("a key")).expect("a place")
}

fn park() -> PlaceId {
    place(&EntityKey::new("park").expect("a key")).expect("a place")
}

/// A market (street, 08:00–14:00), a fair (street or park, 10:00–12:00, announced 3 days ahead), a
/// two-day fete (park, 09:00 day 1 – 17:00 day 2); `per_day: 2`, `per_place_running: 1`.
fn configured() -> HappeningsConfigured {
    let text = "seed: 7
kinds:
  - { id: market, places: [street], hours: \"08:00-14:00\" }
  - { id: fair, places: [street, park], hours: \"10:00-12:00\", announce_days: 3 }
  - { id: fete, places: [park], hours: \"09:00-17:00\", days: 2 }
limits: { per_day: 2, per_place_running: 1 }
";
    let configuration: HappeningsConfiguration = serde_saphyr::from_str(text).expect("decodes");
    HappeningsConfigured::of(&configuration, place).expect("resolves")
}

fn kind(id: &str) -> HappeningKindId {
    HappeningKindId::new(id).expect("an id")
}

fn draft(id: &str, at: PlaceId, day: i64) -> HappeningDraft {
    HappeningDraft::new(kind(id), at, DayIndex::new(day))
}

/// Day 5's pass, at its midnight.
fn admission(configured: &HappeningsConfigured) -> Admission<'_> {
    Admission::new(
        DayIndex::new(5),
        WorldTime::from_seconds(5 * DAY),
        configured.kinds(),
        configured.limits(),
        Vec::new(),
    )
}

#[test]
fn an_admitted_draft_spans_its_kinds_hours_and_days() {
    let configured = configured();
    let mut pass = admission(&configured);
    let market = pass.admit(&draft("market", street(), 5)).expect("admitted");
    assert_eq!(market.happening().start().seconds(), 5 * DAY + 8 * 3_600);
    assert_eq!(market.until().seconds(), 5 * DAY + 14 * 3_600);
    let fete = pass.admit(&draft("fete", park(), 5)).expect("admitted");
    assert_eq!(fete.happening().start().seconds(), 5 * DAY + 9 * 3_600);
    assert_eq!(
        fete.until().seconds(),
        6 * DAY + 17 * 3_600,
        "day 2's until"
    );
}

#[test]
fn each_check_withholds_with_its_own_reason() {
    let configured = configured();
    let mut pass = admission(&configured);
    assert_eq!(
        pass.admit(&draft("concert", street(), 5)),
        Err(WithheldReason::UnknownKind)
    );
    assert_eq!(
        pass.admit(&draft("market", park(), 5)),
        Err(WithheldReason::PlaceNotAdmitted)
    );
    assert_eq!(
        pass.admit(&draft("market", street(), 4)),
        Err(WithheldReason::OutsideWindow),
        "the past"
    );
    assert_eq!(
        pass.admit(&draft("market", street(), 6)),
        Err(WithheldReason::OutsideWindow),
        "beyond a horizon of 0"
    );
    assert!(
        pass.admit(&draft("fair", park(), 8)).is_ok(),
        "d + 3 is inside"
    );
    assert_eq!(
        pass.admit(&draft("fair", park(), 9)),
        Err(WithheldReason::OutsideWindow),
        "d + 4 is beyond"
    );
    assert_eq!(
        pass.admit(&draft("fair", park(), 8)),
        Err(WithheldReason::Duplicate)
    );
}

#[test]
fn the_daily_limit_and_the_place_limit_hold() {
    let configured = configured();
    let mut pass = admission(&configured);
    assert!(pass.admit(&draft("market", street(), 5)).is_ok());
    // The fair at the street, 10:00–12:00, lies inside the market's 08:00–14:00.
    assert_eq!(
        pass.admit(&draft("fair", street(), 5)),
        Err(WithheldReason::PlaceBusy)
    );
    assert!(
        pass.admit(&draft("fair", park(), 5)).is_ok(),
        "the park is free"
    );
    assert_eq!(
        pass.admit(&draft("fete", park(), 5)),
        Err(WithheldReason::OverDailyLimit),
        "two already begin on day 5 (checked before the place, which is busy too)"
    );
    // The limit is per day: day 6 has room.
    assert!(pass.admit(&draft("fair", park(), 6)).is_ok());
}

/// A happening that ends exactly when the next begins does not overlap it: spans are `[start, until)`.
#[test]
fn back_to_back_happenings_do_not_overlap() {
    let text = "seed: 7
kinds:
  - { id: morning, places: [street], hours: \"08:00-12:00\" }
  - { id: afternoon, places: [street], hours: \"12:00-16:00\" }
";
    let configuration: HappeningsConfiguration = serde_saphyr::from_str(text).expect("decodes");
    let configured = HappeningsConfigured::of(&configuration, place).expect("resolves");
    let mut pass = admission(&configured);
    assert!(pass.admit(&draft("morning", street(), 5)).is_ok());
    assert!(pass.admit(&draft("afternoon", street(), 5)).is_ok());
}

/// A pass that opens mid-day (a world begun at 10:00) withholds a happening that would already have
/// begun: never the past.
#[test]
fn a_happening_that_would_already_have_begun_is_outside_the_window() {
    let configured = configured();
    let mut pass = Admission::new(
        DayIndex::new(5),
        WorldTime::from_seconds(5 * DAY + 10 * 3_600),
        configured.kinds(),
        configured.limits(),
        Vec::new(),
    );
    assert_eq!(
        pass.admit(&draft("market", street(), 5)),
        Err(WithheldReason::OutsideWindow)
    );
}
