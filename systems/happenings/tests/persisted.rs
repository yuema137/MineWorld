//! T10 (AC-WE-1): a persisted world stopped during a running happening, and again between a
//! festival's announcement and its start, then resumed, states exactly what the uninterrupted world
//! states — the almanac, the happenings' processes and the first pass all ride the save.

mod support;

use mineworld_contracts::EventEnvelope;
use mineworld_happenings::{HappeningAnnounced, HappeningBegan};
use mineworld_persistence::{
    Creation, Durability, PersistenceBackend, PersistentWorld, SqliteBackend,
};
use support::{DAY, HOUR, SAN_DIEGO, assemble, compose, of, t};

const TOWN: &str = "seed: 7
kinds:
  - { id: market-day, title: Saturday market, places: [street], hours: \"08:00-14:00\" }
  - { id: festival, title: Harvest festival, places: [park], hours: \"16:00-22:00\", announce_days: 7 }
  - { id: street-musician, places: [street, park], hours: \"11:00-13:00\", audience: place }
  - { id: fete, places: [square], hours: \"09:00-21:00\", days: 3 }
calendar:
  - { kind: market-day, place: street, weekdays: [saturday] }
  - { kind: festival, place: park, date: \"10-31\" }
  - { kind: fete, place: square, date: \"10-09\" }
random:
  - { kind: street-musician, place: street, per_mille_per_day: 150 }
  - { kind: street-musician, place: park, per_mille_per_day: 300 }
";

/// A save of this test's own, removed when the test ends (DEP-29).
struct Scratch(mineworld_test_support::Scratch);

fn open(scratch: &Scratch) -> Box<dyn PersistenceBackend> {
    Box::new(SqliteBackend::open(&scratch.0, Durability::ProcessCrash).expect("opens"))
}

fn create(scratch: &Scratch) -> PersistentWorld {
    let (world, _, facts) = assemble(compose(true), Some(SAN_DIEGO), Some(TOWN));
    let backend = SqliteBackend::create(&scratch.0, Durability::ProcessCrash).expect("creates");
    PersistentWorld::create(
        Box::new(backend),
        world,
        Creation {
            instance: 0x5eed_0000_0000_0000_0000_0000_00a0_0b01,
            pack: "happenings-test".to_owned(),
            at: t(0),
            facts,
        },
    )
    .expect("a save is created")
    .0
}

#[test]
fn stopped_mid_happening_continues_as_uninterrupted() {
    // Day 2 (2026-10-10, a Saturday) at 10:00, the market running and the three-day fete (from day 1,
    // 2026-10-09, at 09:00 until day 3 at 21:00) on its second day; day 20 at 12:00, the festival of
    // day 23 announced on day 16 and not begun.
    let (first_stop, second_stop, end) = (2 * DAY + 10 * HOUR, 20 * DAY + 12 * HOUR, 30 * DAY);

    let straight = Scratch(mineworld_test_support::scratch!(empty "straight"));
    let mut uninterrupted = create(&straight);
    let _ = uninterrupted.advance_to(t(first_stop)).expect("advances");
    let straight_after = uninterrupted
        .advance_to(t(end))
        .expect("advances")
        .into_events();

    let stopped = Scratch(mineworld_test_support::scratch!(empty "stopped"));
    let mut first = create(&stopped);
    let before = first
        .advance_to(t(first_stop))
        .expect("advances")
        .into_events();
    assert!(
        of::<HappeningBegan>(&before)
            .iter()
            .any(|(at, _)| at.seconds() == 2 * DAY + 8 * HOUR),
        "the market is running at the first stop"
    );
    assert!(
        of::<HappeningBegan>(&before)
            .iter()
            .any(|(at, _)| at.seconds() == DAY + 9 * HOUR),
        "the fete is running at the first stop"
    );
    assert!(
        of::<mineworld_happenings::HappeningEnded>(&straight_after)
            .iter()
            .any(|(at, _)| at.seconds() == 3 * DAY + 21 * HOUR),
        "the fete ends on its third day"
    );
    drop(first);
    let (mut second, _) = PersistentWorld::resume(open(&stopped), compose(true)).expect("resumes");
    let mut resumed_after: Vec<EventEnvelope> = second
        .advance_to(t(second_stop))
        .expect("advances")
        .into_events();
    drop(second);
    let (mut third, _) = PersistentWorld::resume(open(&stopped), compose(true)).expect("resumes");
    resumed_after.extend(third.advance_to(t(end)).expect("advances").into_events());

    let festival = |facts: &[EventEnvelope]| {
        of::<HappeningAnnounced>(facts)
            .iter()
            .filter(|(_, fact)| fact.happening().kind().as_str() == "festival")
            .count()
    };
    assert_eq!(festival(&resumed_after), 1, "announced once, on day 16");
    assert!(
        of::<HappeningBegan>(&resumed_after)
            .iter()
            .any(|(at, _)| at.seconds() == 23 * DAY + 16 * HOUR),
        "the festival begins after the second resume"
    );
    assert_eq!(
        serde_json::to_vec(&resumed_after).expect("encodes"),
        serde_json::to_vec(&straight_after).expect("encodes"),
        "the resumed world's facts after the first stop are the uninterrupted world's, byte for byte"
    );
}
