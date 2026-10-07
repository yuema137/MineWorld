//! CP-4 — **a person's day is a schedule-owned Process whose wakes cause the facts that begin each part
//! of it, and people follow it** (`step-09-social.md` §4.3.3 C8, `DECISIONS.md` `ARC-32`).
//!
//! ```text
//! mineworld run worlds/social-cafe --headless --seed 7 --days 30 --save S, then S read back:
//!
//! caused by the routine   every agenda-changed after genesis is caused by Process(p), p the routine
//!                         its own payload names, one p per person for the whole run; each genesis one
//!                         by that person's routine-assigned
//! on schedule             every one after genesis falls on a boundary of the person's authored day,
//!                         and there are exactly days × segments of them
//! followed                every seat reaches ≥ 90 % of its segments: in the agenda's place when the
//!                         segment begins, or entering it before the segment ends
//! moves nobody            otto, whom no seat names, never moves, and his agenda changes every day
//! ```
//!
//! The oracle for "on schedule" reads each person's file with a literal reader of its own, never
//! schedule's code (rules §25). The 90 % is a literal from the requirement — a routine followed less
//! than nine times in ten is not a routine — fixed before anything was measured (QC-7), and shown
//! load-bearing by setting the controller's agenda band to 0 (§9 E-C8).

mod headless;
mod social;

use std::collections::BTreeMap;

use headless::{PACK, Tables, fresh, run};
use mineworld_contracts::{Causation, EntityId, EntityKey, PlaceId, WorldTime};
use mineworld_presence::Arrived;
use mineworld_schedule::{AgendaChanged, RoutineAssigned};
use social::decoded;

const SEED: u64 = 7;
const DAYS: i64 = 30;
const DAY: i64 = 86_400;
/// Out of 100: the share of its segments every seat must reach. From the requirement, not the result.
const FOLLOWED_AT_LEAST: usize = 90;

/// A person's authored day, as (seconds after midnight, place key), read literally from their file.
fn authored_day(person: &str) -> Vec<(i64, String)> {
    let text = std::fs::read_to_string(format!("{PACK}/people/{person}.yaml")).expect("a file");
    text.lines()
        .filter(|line| line.trim_start().starts_with("- { from:"))
        .map(|line| {
            let at = line.find("from: \"").expect("from") + 7;
            let (hours, minutes) = line[at..at + 5].split_once(':').expect("HH:MM");
            let place_at = line.find("place: ").expect("place") + 7;
            let place = line[place_at..]
                .split([',', ' ', '}'])
                .next()
                .expect("a key")
                .to_owned();
            (
                hours.parse::<i64>().expect("hours") * 3_600
                    + minutes.parse::<i64>().expect("minutes") * 60,
                place,
            )
        })
        .collect()
}

#[test]
fn every_persons_day_is_kept_by_their_routine_process_and_the_seats_follow_it() {
    let save = fresh("routines-30");
    run(SEED, u64::try_from(DAYS).expect("fits"), Some(&save));
    let facts = social::facts_of(&Tables::read(&save));
    social::every_person_has_an_agenda_change_every_day(&facts, DAYS);

    let loaded = mineworld_worldpack::WorldPack::read(PACK)
        .expect("reads")
        .load(WorldTime::EPOCH)
        .expect("loads");
    let id = |key: &str| {
        loaded
            .id(&EntityKey::new(key).expect("a key"))
            .expect("a key of the pack")
    };
    let people: Vec<&str> = social::AUTHORED_NAMES.iter().map(|(key, _)| *key).collect();
    let by_id: BTreeMap<EntityId, &str> = people.iter().map(|key| (id(key), *key)).collect();
    let types: BTreeMap<u64, &str> = facts
        .iter()
        .map(|fact| (fact.id().raw(), fact.event_type().as_str()))
        .collect();

    // ── Caused by the routine, and on schedule. ──────────────────────────────────────────────
    let mut routine_of: BTreeMap<EntityId, mineworld_contracts::ProcessId> = BTreeMap::new();
    let mut changes: BTreeMap<EntityId, usize> = BTreeMap::new();
    for fact in facts
        .iter()
        .filter(|fact| fact.event_type().as_str() == "agenda-changed")
    {
        let changed: AgendaChanged = decoded(fact).expect("schedule's fact");
        let person = changed.person().entity_id();
        let key = by_id[&person];
        let routine = *routine_of.entry(person).or_insert(changed.routine());
        assert_eq!(
            changed.routine(),
            routine,
            "{key} has one routine process for the whole run"
        );
        match fact.caused_by() {
            Causation::Event(cause) => {
                assert_eq!(
                    fact.at().seconds(),
                    0,
                    "only the first agenda is caused by a fact"
                );
                assert_eq!(
                    types.get(&cause.raw()).copied(),
                    Some("routine-assigned"),
                    "{key}'s first agenda is caused by their routine's assignment"
                );
            }
            Causation::Process(process) => {
                assert_eq!(
                    *process, routine,
                    "{key}'s agenda changes are their routine's wakes"
                );
                let of_day = fact.at().seconds().rem_euclid(DAY);
                assert!(
                    authored_day(key).iter().any(|(from, _)| *from == of_day),
                    "{key}'s agenda changed at {:02}:{:02}, which is no boundary of their authored day",
                    of_day / 3_600,
                    of_day % 3_600 / 60
                );
                *changes.entry(person).or_default() += 1;
            }
            other => panic!("#{} agenda-changed caused by {other:?}", fact.id().raw()),
        }
    }
    for key in &people {
        let expected = usize::try_from(DAYS).expect("fits") * authored_day(key).len();
        assert_eq!(
            changes.get(&id(key)).copied().unwrap_or(0),
            expected,
            "{key}: every boundary of {DAYS} days, once"
        );
    }
    let assigned = facts.iter().filter_map(decoded::<RoutineAssigned>).count();
    assert_eq!(assigned, people.len(), "one routine assigned per person");

    // ── Followed: where each person was when each part of their day began, and after. ────────
    let mut place_of: BTreeMap<EntityId, PlaceId> = BTreeMap::new();
    // Per person: (segment's place, reached?) for every segment begun so far.
    let mut segments: BTreeMap<EntityId, Vec<(PlaceId, bool)>> = BTreeMap::new();
    let mut began: BTreeMap<EntityId, i64> = BTreeMap::new();
    let mut travel: Vec<i64> = Vec::new();
    for fact in &facts {
        match fact.event_type().as_str() {
            "arrived" => {
                let arrived: Arrived = decoded(fact).expect("presence's fact");
                let person = arrived.person().entity_id();
                let place = arrived.location().place();
                place_of.insert(person, place);
                if let Some((wanted, reached)) =
                    segments.get_mut(&person).and_then(|list| list.last_mut())
                    && *wanted == place
                {
                    if !*reached {
                        travel.push(fact.at().seconds() - began[&person]);
                    }
                    *reached = true;
                }
            }
            "agenda-changed" => {
                let changed: AgendaChanged = decoded(fact).expect("schedule's fact");
                let person = changed.person().entity_id();
                let already = place_of.get(&person) == Some(&changed.place());
                began.insert(person, fact.at().seconds());
                segments
                    .entry(person)
                    .or_default()
                    .push((changed.place(), already));
            }
            _ => {}
        }
    }
    travel.sort_unstable();
    if !travel.is_empty() {
        eprintln!(
            "time to reach the agenda's place, when it was elsewhere: median {} min, 90th percentile \
             {} min, longest {} min ({} journeys)",
            travel[travel.len() / 2] / 60,
            travel[travel.len() * 9 / 10] / 60,
            travel[travel.len() - 1] / 60,
            travel.len()
        );
    }
    let seats = headless::seats();
    let mut report = Vec::new();
    let mut short = Vec::new();
    for key in &people {
        let list = &segments[&id(key)];
        let reached = list.iter().filter(|(_, reached)| *reached).count();
        let mut missed: BTreeMap<u64, usize> = BTreeMap::new();
        for (place, reached) in list {
            if !reached {
                *missed.entry(place.entity_id().raw()).or_default() += 1;
            }
        }
        report.push(format!(
            "{key} {reached}/{} (missed by place {missed:?})",
            list.len()
        ));
        if seats.iter().any(|seat| seat == key) && reached * 100 < list.len() * FOLLOWED_AT_LEAST {
            short.push(format!("{key} reached {reached} of {}", list.len()));
        }
    }
    // Every seat measured and printed before any verdict, so a failure names all who fell short.
    eprintln!("segments reached, per person: {}", report.join(", "));
    assert!(
        short.is_empty(),
        "under {FOLLOWED_AT_LEAST} % of their segments reached — not a routine followed: {}",
        short.join("; ")
    );

    // ── It moves nobody: otto is no seat, and stays where he was put. ────────────────────────
    let otto = id("otto");
    let moved = facts
        .iter()
        .filter(|fact| fact.event_type().as_str() == "arrived")
        .filter_map(decoded::<Arrived>)
        .filter(|arrived| arrived.person().entity_id() == otto)
        .count();
    assert_eq!(moved, 1, "otto arrived once, at genesis, and never moved");
    let away: Vec<&(PlaceId, bool)> = segments[&otto]
        .iter()
        .filter(|(place, _)| Some(place) != place_of.get(&otto))
        .collect();
    assert!(
        !away.is_empty(),
        "located: otto's day does send him elsewhere"
    );
    assert!(
        away.iter().all(|(_, reached)| !reached),
        "and he reached none of it: an agenda moves nobody"
    );
}
