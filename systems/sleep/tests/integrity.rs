//! AC-Na-5 (AC-S35-7, property) and AC-Na-6 (kill and resume).
//!
//! A seeded script — test code, its own draws — drives every person of each test world: every 15 world
//! minutes each picks one of sleep (for drawn within the bounds) / wake / talk to someone present /
//! walk to another place / nothing. Over the run, per person: `fell-asleep` and `woke` strictly
//! alternate, starting with `fell-asleep`; every `woke.slept` is the instants' difference; every
//! `woke acted` is at the instant of an `arrived` or `spoke` of that person; `Asleep` is present
//! exactly between the pair at each sampled instant; running `sleep` processes equal sleepers.

mod support;

use std::collections::BTreeMap;

use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, ActionResult, EntityId, EventEnvelope, WorldTime,
};
use mineworld_conversation::Spoke;
use mineworld_persistence::{Creation, Durability, PersistentWorld, SqliteBackend};
use mineworld_presence::Arrived;
use mineworld_sleep::{Asleep, FellAsleep, Sleep, Woke, WokeReason};
use support::{Assembled, GENESIS, Kind, Town, assemble, compose, facts_of};

/// A test-local generator (SplitMix64): the script's draws, never the pack's.
struct Draws(u64);

impl Draws {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

const NAMES: [&str; 3] = ["ana", "ben", "cy"];

/// What a run saw, for the minimum counts.
#[derive(Default, Debug)]
struct Counts {
    sleeps: usize,
    addressed: usize,
    acted: usize,
}

fn drive(kind: Kind, seed: u64, days: i64) -> Counts {
    let mut town = Town::new(kind);
    let longest: u16 = if kind == Kind::Hostel { 600 } else { 840 };
    let places: Vec<&'static str> = town.places.keys().copied().collect();
    let mut draws = Draws(seed);
    let slots = days * 96;
    for slot in 1..=slots {
        for name in NAMES {
            match draws.below(20) {
                0..=5 => {
                    let span = u64::from(longest - 15) + 1;
                    let minutes = 15 + u16::try_from(draws.below(span)).expect("small");
                    town.sleep(name, minutes);
                }
                6..=7 => {
                    town.wake(name);
                }
                8..=12 => {
                    let here = town.place_of(name);
                    let present: Vec<&str> = NAMES
                        .into_iter()
                        .filter(|other| *other != name && town.place_of(other) == here)
                        .collect();
                    if !present.is_empty() {
                        let pick = present
                            [usize::try_from(draws.below(present.len() as u64)).expect("small")];
                        town.talk(name, pick);
                    }
                }
                13..=16 => {
                    let here = town.place_of(name);
                    let others: Vec<&str> = places
                        .iter()
                        .copied()
                        .filter(|place| Some(town.place(place)) != here)
                        .collect();
                    let pick =
                        others[usize::try_from(draws.below(others.len() as u64)).expect("small")];
                    town.walk(name, pick);
                }
                _ => {}
            }
        }
        town.pass_to(WorldTime::from_seconds(GENESIS.seconds() + slot * 900));
        sample(&town, &format!("{kind:?} seed {seed} slot {slot}"));
    }
    check_log(&town)
}

/// At this instant: `Asleep` exactly for those whose last sleep fact is `fell-asleep`, and one running
/// `sleep` process per sleeper.
fn sample(town: &Town, label: &str) {
    let mut sleepers = 0;
    for name in NAMES {
        let person = town.person(name);
        let last = town
            .log
            .iter()
            .rev()
            .find(|fact| {
                matches!(fact.event_type().as_str(), "fell-asleep" | "woke")
                    && fact.participants().first() == Some(&person)
            })
            .map(|fact| fact.event_type().as_str() == "fell-asleep");
        let expected = last.unwrap_or(false);
        assert_eq!(town.asleep(name).is_some(), expected, "{label}: {name}");
        sleepers += usize::from(expected);
    }
    assert_eq!(town.sleeping_processes(), sleepers, "{label}");
    assert_eq!(
        town.world.read().components::<Asleep>().count(),
        sleepers,
        "{label}"
    );
}

/// The whole log's sleep facts, per person.
fn check_log(town: &Town) -> Counts {
    let mut counts = Counts::default();
    let mut open: BTreeMap<EntityId, WorldTime> = BTreeMap::new();
    let acts = acts(&town.log);
    for fact in &town.log {
        match fact.event_type().as_str() {
            "fell-asleep" => {
                let fell = facts_of::<FellAsleep>(std::slice::from_ref(fact));
                let person = fell[0].1.person().entity_id();
                assert!(
                    open.insert(person, fact.at()).is_none(),
                    "fell-asleep twice without a woke"
                );
                counts.sleeps += 1;
            }
            "woke" => {
                let woke = facts_of::<Woke>(std::slice::from_ref(fact))[0].1;
                let person = woke.person().entity_id();
                let since = open.remove(&person).expect("woke follows a fell-asleep");
                assert_eq!(
                    woke.slept(),
                    u64::try_from(fact.at().seconds() - since.seconds()).expect("forward")
                );
                match woke.reason() {
                    WokeReason::Acted => {
                        assert!(
                            acts.contains(&(person, fact.at())),
                            "acted at an arrival or a line of the sleeper's"
                        );
                        counts.acted += 1;
                    }
                    WokeReason::Addressed => counts.addressed += 1,
                    WokeReason::Alarm | WokeReason::Asked => {}
                }
            }
            _ => {}
        }
    }
    counts
}

/// Every (person, instant) at which the person arrived somewhere or spoke.
fn acts(log: &[EventEnvelope]) -> Vec<(EntityId, WorldTime)> {
    let mut acts: Vec<(EntityId, WorldTime)> = facts_of::<Arrived>(log)
        .into_iter()
        .map(|(at, arrived)| (arrived.person().entity_id(), at))
        .collect();
    acts.extend(
        facts_of::<Spoke>(log)
            .into_iter()
            .map(|(at, spoke)| (spoke.speaker().entity_id(), at)),
    );
    acts
}

fn property(days: i64) {
    for kind in [Kind::FlatAndPark, Kind::Hostel] {
        for seed in [7, 42] {
            let counts = drive(kind, seed, days);
            println!("{kind:?} seed {seed}: {counts:?}");
            assert!(counts.sleeps >= 1, "{kind:?} {seed}: a vacuous run");
            assert!(counts.acted >= 1, "{kind:?} {seed}: nobody acted awake");
            if kind == Kind::FlatAndPark {
                assert!(counts.addressed >= 1, "{seed}: nobody woken by talk");
            } else {
                assert_eq!(counts.addressed, 0, "{seed}: woken_by_talk is false");
            }
        }
    }
}

#[test]
fn sleep_integrity_holds_over_three_days() {
    property(3);
}

/// The milestone run (AC-S35-7): 30 days per world and seed.
#[test]
#[ignore = "milestone batch: 30 simulated days per world and seed"]
fn sleep_integrity_holds_over_thirty_days() {
    property(30);
}

const INSTANCE: u128 = 0x5eed_0000_0000_0000_0000_0000_0035_000a;

fn create(scratch: &mineworld_test_support::Scratch) -> (PersistentWorld, EntityId) {
    let Assembled {
        world,
        people,
        facts,
        ..
    } = assemble(Kind::FlatAndPark, true, None);
    let backend = SqliteBackend::create(scratch, Durability::ProcessCrash).expect("creates");
    let (persisted, _) = PersistentWorld::create(
        Box::new(backend),
        world,
        Creation {
            instance: INSTANCE,
            pack: "sleep-test".to_owned(),
            at: GENESIS,
            facts,
        },
    )
    .expect("a save is created");
    (persisted, people["ana"])
}

fn at(seconds: i64) -> WorldTime {
    WorldTime::from_seconds(GENESIS.seconds() + seconds)
}

/// ana sleeps two hours at +60 s; the run stops at +30 min.
fn fall_asleep(persisted: &mut PersistentWorld, ana: EntityId) {
    let _ = persisted.advance_to(at(60)).expect("advances");
    let intent = ActionIntent::new(
        ActionId::from_raw(1),
        ana,
        ActionRecord::new::<Sleep>(serde_json::to_vec(&Sleep::new(120)).expect("encodes")),
        at(60),
    );
    let done = persisted.dispatch(&intent, at(60)).expect("dispatches");
    assert!(matches!(done.result(), ActionResult::Accepted { .. }));
    let _ = persisted.advance_to(at(1_800)).expect("advances");
}

fn until_after_the_alarm(persisted: &mut PersistentWorld) -> Vec<EventEnvelope> {
    persisted
        .advance_to(at(4 * 3_600))
        .expect("advances")
        .into_events()
}

#[test]
fn a_world_stopped_mid_sleep_wakes_as_the_uninterrupted_world_does() {
    let straight = mineworld_test_support::scratch!(empty "straight");
    let (mut uninterrupted, ana) = create(&straight);
    fall_asleep(&mut uninterrupted, ana);
    let straight_after = until_after_the_alarm(&mut uninterrupted);

    let stopped = mineworld_test_support::scratch!(empty "stopped");
    let (mut first, ana) = create(&stopped);
    fall_asleep(&mut first, ana);
    let asleep = first
        .world()
        .read()
        .component::<Asleep>(ana)
        .copied()
        .expect("asleep at the stop");
    drop(first);

    let backend = SqliteBackend::open(&stopped, Durability::ProcessCrash).expect("opens");
    let (mut resumed, _) =
        PersistentWorld::resume(Box::new(backend), compose(true).0).expect("resumes");
    assert_eq!(
        resumed.world().read().component::<Asleep>(ana).copied(),
        Some(asleep),
        "Asleep survived"
    );
    let resumed_after = until_after_the_alarm(&mut resumed);

    let wokes = |facts: &[EventEnvelope]| -> Vec<(WorldTime, Woke)> { facts_of::<Woke>(facts) };
    let woke = wokes(&resumed_after);
    assert_eq!(woke.len(), 1);
    assert_eq!(woke[0].0, at(60 + 120 * 60));
    assert_eq!(woke[0].1.reason(), WokeReason::Alarm);
    assert_eq!(woke[0].1.slept(), 7_200);
    assert_eq!(
        woke,
        wokes(&straight_after),
        "the same woke, instant and payload"
    );
}
