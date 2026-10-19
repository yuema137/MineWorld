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
        let (_, own) = town.submit_now(ana, None, support::record(&mineworld_sleep::Wake {}));
        facts.extend(own);
        assert_eq!(wokes(&facts), [(alarm, WokeReason::Alarm, 900)], "{kind:?}");
        awake_and_no_process(&town);
    }
}

/// Talking to a sleeper wakes them `addressed` where the world says talk wakes (flat-and-park); in
/// the hostel (`woken_by_talk: false`) the line is heard and the sleeper sleeps on.
#[test]
fn being_spoken_to_wakes_the_sleeper_unless_the_world_says_otherwise() {
    let mut town = Town::new(Kind::FlatAndPark);
    let since = asleep_for(&mut town, 120);
    town.pass(299);
    let (result, facts) = town.talk("ben", "ana");
    assert!(accepted(&result), "{result:?}");
    let woke = wokes(&facts);
    assert_eq!(woke.len(), 1);
    assert_eq!(woke[0].1, WokeReason::Addressed);
    assert_eq!(woke[0].2, (woke[0].0.seconds() - since.seconds()) as u64);
    assert_eq!(woke[0].2, 300);
    awake_and_no_process(&town);

    let mut hostel = Town::new(Kind::Hostel);
    asleep_for(&mut hostel, 120);
    let (result, facts) = hostel.talk("ben", "ana");
    assert!(accepted(&result), "the line is still spoken: {result:?}");
    assert!(support::kinds(&facts).contains(&"spoke"));
    assert!(wokes(&facts).is_empty(), "nobody wakes");
    assert!(hostel.asleep("ana").is_some());
    assert_eq!(hostel.sleeping_processes(), 1);
}

/// A sleeper who speaks wakes `acted`, at the instant of their own `spoke`, in both worlds.
#[test]
fn a_sleeper_who_speaks_wakes_acted() {
    for kind in BOTH {
        let mut town = Town::new(kind);
        asleep_for(&mut town, 120);
        let (result, facts) = town.talk("ana", "ben");
        assert!(accepted(&result), "{result:?}");
        let spoke = facts
            .iter()
            .find(|fact| fact.event_type().as_str() == "spoke")
            .expect("spoken")
            .at();
        let woke = wokes(&facts);
        assert_eq!(woke.len(), 1, "{kind:?}");
        assert_eq!(woke[0].0, spoke);
        assert_eq!(woke[0].1, WokeReason::Acted);
        awake_and_no_process(&town);
    }
}

/// A sleeper who walks wakes `acted` at the instant of their first `arrived`, and only once.
#[test]
fn a_sleeper_who_walks_wakes_acted_at_the_first_arrival() {
    for (kind, to) in [(Kind::FlatAndPark, "street"), (Kind::Hostel, "hall")] {
        let mut town = Town::new(kind);
        asleep_for(&mut town, 120);
        let (result, facts) = town.walk("ana", to);
        assert!(accepted(&result), "{result:?}");
        let first = facts
            .iter()
            .find(|fact| fact.event_type().as_str() == "arrived")
            .expect("a stride")
            .at();
        let woke = wokes(&facts);
        assert_eq!(woke.len(), 1, "{kind:?}: one woke for the whole walk");
        assert_eq!(woke[0].0, first);
        assert_eq!(woke[0].1, WokeReason::Acted);
        awake_and_no_process(&town);
        assert_eq!(town.place_of("ana"), Some(town.place(to)));
    }
}

/// Two sleepers talk: the speaker wakes `acted`, the listener `addressed`, speaker first.
#[test]
fn two_sleepers_talking_each_wake_once_speaker_first() {
    let mut town = Town::new(Kind::FlatAndPark);
    let (result, _) = town.sleep("ana", 60);
    assert!(accepted(&result));
    let (result, _) = town.sleep("ben", 60);
    assert!(accepted(&result));
    assert_eq!(town.sleeping_processes(), 2);
    let (_, facts) = town.talk("ana", "ben");
    let woke: Vec<(WokeReason, mineworld_contracts::PersonId)> = facts_of::<Woke>(&facts)
        .into_iter()
        .map(|(_, woke)| (woke.reason(), woke.person()))
        .collect();
    let ana = mineworld_contracts::PersonId::new(
        town.person("ana"),
        mineworld_contracts::EntityType::Person,
    )
    .expect("a person");
    let ben = mineworld_contracts::PersonId::new(
        town.person("ben"),
        mineworld_contracts::EntityType::Person,
    )
    .expect("a person");
    assert_eq!(
        woke,
        [(WokeReason::Acted, ana), (WokeReason::Addressed, ben)]
    );
    assert_eq!(town.sleeping_processes(), 0);
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
