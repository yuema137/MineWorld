//! AC-Na-3: the four ways a sleep ends, on both test worlds. After each, `Asleep` is gone and no
//! `sleep` process runs; each sleep ends in exactly one `woke`.

mod support;

use mineworld_contracts::{EventEnvelope, WorldTime};
use mineworld_sleep::{AlarmSet, FellAsleep, Woke, WokeReason};
use support::{Kind, Town, accepted, facts_of};

const BOTH: [Kind; 2] = [Kind::FlatAndPark, Kind::Hostel];

/// The `woke` facts among `facts`, as (instant, reason, slept).
fn wokes(facts: &[EventEnvelope]) -> Vec<(WorldTime, WokeReason, u64)> {
    facts_of::<Woke>(facts)
        .into_iter()
        .map(|(at, woke)| (at, woke.reason(), woke.slept()))
        .collect()
}

/// `ana` falls asleep for `minutes`; returns the instant she did.
fn asleep_for(town: &mut Town, minutes: u16) -> WorldTime {
    let (result, facts) = town.sleep("ana", minutes);
    assert!(accepted(&result), "{result:?}");
    let fell = facts_of::<FellAsleep>(&facts);
    assert_eq!(fell.len(), 1);
    let set = facts_of::<AlarmSet>(&facts);
    assert_eq!(set.len(), 1);
    let since = fell[0].0;
    assert_eq!(
        set[0].1.alarm(),
        WorldTime::from_seconds(since.seconds() + i64::from(minutes) * 60)
    );
    let asleep = town.asleep("ana").expect("asleep");
    assert_eq!(asleep.since(), since);
    assert_eq!(asleep.alarm(), set[0].1.alarm());
    assert_eq!(town.sleeping_processes(), 1);
    since
}

fn awake_and_no_process(town: &Town) {
    assert!(town.asleep("ana").is_none(), "Asleep removed");
    assert_eq!(town.sleeping_processes(), 0, "the sleep process ended");
}

#[test]
fn the_alarm_wakes_the_sleeper_at_exactly_since_plus_for() {
    for kind in BOTH {
        let mut town = Town::new(kind);
        let since = asleep_for(&mut town, 90);
        let alarm = WorldTime::from_seconds(since.seconds() + 90 * 60);
        let before = town.pass_to(WorldTime::from_seconds(alarm.seconds() - 1));
        assert!(wokes(&before).is_empty(), "{kind:?}: not before the alarm");
        assert!(town.asleep("ana").is_some());
        let at = town.pass(1);
        assert_eq!(
            wokes(&at),
            [(alarm, WokeReason::Alarm, 90 * 60)],
            "{kind:?}"
        );
        awake_and_no_process(&town);
        assert!(wokes(&town.pass(86_400)).is_empty(), "exactly one woke");
    }
}

#[test]
fn wake_ends_the_sleep_asked_and_the_alarm_rings_for_nobody() {
    for kind in BOTH {
        let mut town = Town::new(kind);
        let since = asleep_for(&mut town, 60);
        town.pass(599);
        let (result, facts) = town.wake("ana");
        assert!(accepted(&result), "{result:?}");
        let woke = wokes(&facts);
        assert_eq!(woke.len(), 1);
        assert_eq!(woke[0].1, WokeReason::Asked);
        assert_eq!(woke[0].2, (woke[0].0.seconds() - since.seconds()) as u64);
        assert_eq!(woke[0].2, 600);
        awake_and_no_process(&town);
        assert!(wokes(&town.pass(7_200)).is_empty(), "the alarm is gone");
    }
}

/// An alarm and a `wake` request at one instant: whichever comes first ends the sleep, the other finds
/// nobody asleep, and there is exactly one `woke`.
#[test]
fn an_alarm_and_a_wake_at_one_instant_give_one_woke() {
    for kind in BOTH {
        let mut town = Town::new(kind);
        let since = asleep_for(&mut town, 15);
        let alarm = WorldTime::from_seconds(since.seconds() + 15 * 60);
        let mut facts = town.pass_to(alarm);
        let ana = town.person("ana");
        let (_, own) = town.submit_now(
            ana,
            None,
            support::record(&mineworld_sleep::Wake {}),
        );
        facts.extend(own);
        assert_eq!(
            wokes(&facts),
            [(alarm, WokeReason::Alarm, 900)],
            "{kind:?}"
        );
        awake_and_no_process(&town);
    }
}

/// A sleeper's place is told the sleep began and ended; the alarm is not in what it is told.
#[test]
fn fell_asleep_names_the_person_and_place_and_nothing_else() {
    let mut town = Town::new(Kind::FlatAndPark);
    let (_, facts) = town.sleep("ana", 60);
    let fell = facts
        .iter()
        .find(|fact| fact.event_type().as_str() == "fell-asleep")
        .expect("stated");
    let payload: serde_json::Value =
        serde_json::from_slice(fell.payload().payload()).expect("JSON");
    let mut keys: Vec<&str> = payload
        .as_object()
        .expect("an object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(keys, ["person", "place"]);
    assert_eq!(
        *fell.visibility(),
        mineworld_contracts::Visibility::Place(town.place("flat"))
    );
}
