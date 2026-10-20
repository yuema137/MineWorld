//! Leaving early for work (`pr-le-leave-early.md` §0.8): a person whose part of the day is punctual is at its
//! place when it begins, and an employee is present when the shift starts — in Market Town, in Social Café and
//! in the test-only variant `commute-town`, whose geometry nothing in any crate knows (AC-LE2-7).
//!
//! ```text
//! AC-LE2-1   rule, default policy: every punctual part's place reached at or before `from`; every shift present
//! AC-LE-3    nobody arrives absurdly early: at most an hour before `from` (a literal, fixed here). Only commute
//!            arrivals count (operator ruling F-Nd-7, 2026-10-19): an arrival after which the person received
//!            items (`items-transferred` to them) before `from` was an errand there, not a commute
//! AC-LE2-4   motivated, every seat: on-time share of punctual parts ≥ 900 ‰ (recorded, never retuned)
//! policy     the same world with `commute.mode: at-boundary` sets out at the boundary and so is late
//! ```
//!
//! A punctual part is read from `routine-assigned` (its `punctual` flag) and judged at the `agenda-changed` that
//! begins it: the person's place at that instant — every arrival at or before it applied — must be the part's.
//! Parts that begin at genesis are not judged (nobody could set out for them). The expected counts are literals,
//! so a run that measures nothing fails.
//!
//! The default test is one seven-day seed per world; the 30-day, three-seed acceptance runs and the motivated run
//! are `#[ignore]`d for run time and run at the milestone (SPEED FIRST):
//! `cargo test -p mineworld-cli --test punctuality -- --ignored --nocapture`.

mod headless;
mod social;

use std::collections::{BTreeMap, BTreeSet};

use mineworld_contracts::EventEnvelope;
use mineworld_presence::Arrived;
use mineworld_schedule::{AgendaChanged, RoutineAssigned, TimeOfDay};

use headless::{Tables, fresh, mineworld};
use social::decoded;

const MARKET: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/market-town");
const CAFE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/social-cafe");
const COMMUTE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worlds/commute-town");
const ALL_MOTIVATED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../cognition/motivated/examples/social-cafe-all-motivated.yaml"
);
const AT_BOUNDARY: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/decisions/rule-at-boundary.yaml"
);

/// AC-LE-3's bound, fixed before any run: an arrival more than an hour before `from` is absurdly early.
const EARLIEST: i64 = 3_600;

/// What one run measured.
#[derive(Debug, Default)]
struct Measured {
    /// Punctual parts judged, and those whose place was reached by `from`.
    parts: usize,
    on_time: usize,
    /// Commute arrivals more than [`EARLIEST`] before `from`.
    too_early: usize,
    /// Arrivals more than [`EARLIEST`] before `from` that were errands (F-Nd-7): recorded, not judged.
    errands: usize,
    /// `shift-started` facts, and those with the employee present.
    shifts: usize,
    present: usize,
    /// Walks to a place that ended `stalled` or `no-route` (`ARC-330`): one line each, for a reader.
    walks_failed: Vec<String>,
    /// One line per late part, for a reader.
    late: Vec<String>,
}

fn id_of(value: &serde_json::Value) -> Option<u64> {
    let entity = value.get("entity").unwrap_or(value);
    entity.as_u64().or_else(|| entity.as_str()?.parse().ok())
}

/// `shift-started` as JSON, so this test needs no employment crate (ARC-35 item 3), as `fx24.rs` reads it.
fn shift_present(fact: &EventEnvelope) -> Option<bool> {
    let value: serde_json::Value = serde_json::from_slice(fact.payload().payload()).ok()?;
    id_of(&value["employee"])?;
    value["present"].as_bool()
}

/// The receiver of `items-transferred`, as JSON, so this test needs no inventory crate (ARC-35 item 3).
fn items_received_by(fact: &EventEnvelope) -> Option<u64> {
    let value: serde_json::Value = serde_json::from_slice(fact.payload().payload()).ok()?;
    id_of(&value["to"])
}

fn measure(facts: &[EventEnvelope]) -> Measured {
    // Each person's punctual times of day and places, from their routine.
    let mut punctual: BTreeMap<u64, BTreeSet<(i64, u64)>> = BTreeMap::new();
    // Every arrival, in order: (instant, person, place).
    let mut arrivals: Vec<(i64, u64, u64)> = Vec::new();
    // Every punctual part begun after genesis: (instant, person, place).
    let mut begun: Vec<(i64, u64, u64)> = Vec::new();
    // Every receipt of items: (instant, receiver) — what makes an early arrival an errand (F-Nd-7).
    let mut receipts: Vec<(i64, u64)> = Vec::new();
    let genesis = facts.first().map_or(0, |fact| fact.at().seconds());
    let mut measured = Measured::default();
    for fact in facts {
        match fact.event_type().as_str() {
            "routine-assigned" => {
                let assigned: RoutineAssigned = decoded(fact).expect("schedule's fact");
                let person = assigned.person().entity_id().raw();
                for part in assigned.segments().segments() {
                    if part.punctual() {
                        punctual
                            .entry(person)
                            .or_default()
                            .insert((part.from().seconds(), part.place().entity_id().raw()));
                    }
                }
            }
            "arrived" => {
                let arrived: Arrived = decoded(fact).expect("presence's fact");
                arrivals.push((
                    fact.at().seconds(),
                    arrived.person().entity_id().raw(),
                    arrived.location().place().entity_id().raw(),
                ));
            }
            "agenda-changed" if fact.at().seconds() > genesis => {
                let changed: AgendaChanged = decoded(fact).expect("schedule's fact");
                let person = changed.person().entity_id().raw();
                let place = changed.place().entity_id().raw();
                let at = fact.at().seconds();
                let of_day = TimeOfDay::of(fact.at());
                if punctual
                    .get(&person)
                    .is_some_and(|parts| parts.contains(&(of_day, place)))
                {
                    begun.push((at, person, place));
                }
            }
            "items-transferred" => {
                if let Some(to) = items_received_by(fact) {
                    receipts.push((fact.at().seconds(), to));
                }
            }
            "walk-ended" => {
                let value: serde_json::Value =
                    serde_json::from_slice(fact.payload().payload()).unwrap_or_default();
                let outcome = value["outcome"].as_str().unwrap_or_default();
                // A walk to a person ends no-route when they leave the place: not a doorway jam.
                let to_a_place = value["destination"].get("person").is_none();
                if to_a_place && matches!(outcome, "stalled" | "no-route") {
                    measured.walks_failed.push(format!(
                        "t{}: person #{} {outcome}",
                        fact.at().seconds(),
                        id_of(&value["person"]).unwrap_or_default()
                    ));
                }
            }
            "shift-started" => {
                measured.shifts += 1;
                if shift_present(fact).expect("shift-started carries presence") {
                    measured.present += 1;
                }
            }
            _ => {}
        }
    }
    for (from, person, place) in begun {
        measured.parts += 1;
        let last = arrivals
            .iter()
            .rfind(|(at, who, _)| *who == person && *at <= from);
        match last {
            Some((at, _, there)) if *there == place => {
                measured.on_time += 1;
                // The arrival that put them there; an hour's bound on how early (AC-LE-3). Where genesis put
                // them is no departure.
                let errand = receipts
                    .iter()
                    .any(|(when, who)| *who == person && *at <= *when && *when <= from);
                if *at > genesis && from - at > EARLIEST && errand {
                    measured.errands += 1;
                } else if *at > genesis && from - at > EARLIEST {
                    measured.too_early += 1;
                    measured.late.push(format!(
                        "early: person #{person} at #{place} from t{from}: arrived t{at}"
                    ));
                }
            }
            _ => {
                let first = arrivals
                    .iter()
                    .find(|(at, who, there)| *who == person && *there == place && *at > from)
                    .map_or("never".to_owned(), |(at, ..)| {
                        format!("+{} min", (at - from) / 60)
                    });
                measured.late.push(format!(
                    "person #{person} at #{place} from t{from}: arrived {first}"
                ));
            }
        }
    }
    measured
}

/// `run` of `world` for `days` with `seed`, and what it measured; `decisions` names a decision list.
fn run(world: &str, days: u64, seed: u64, decisions: Option<&str>) -> Measured {
    // Named by the world too, so that two worlds run by concurrent tests never share a scratch.
    let tag = std::path::Path::new(world)
        .file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().into_owned());
    let save = fresh(&format!("punctuality-{tag}-{days}-{seed}"));
    let save_text = save.to_str().expect("a printable path").to_owned();
    let (days, seed) = (days.to_string(), seed.to_string());
    let mut arguments = vec![
        "run",
        world,
        "--headless",
        "--seed",
        &seed,
        "--days",
        &days,
        "--save",
        &save_text,
    ];
    if let Some(decisions) = decisions {
        arguments.extend(["--decisions", decisions]);
    }
    let output = mineworld(&arguments);
    assert!(
        output.status.success(),
        "run {world}: {}",
        headless::stderr(&output)
    );
    measure(&social::facts_of(&Tables::read(&save)))
}

/// AC-LE2-1 on one run: every punctual part on time, no arrival absurdly early, and the literal counts.
fn assert_punctual(world: &str, days: u64, seed: u64, parts: usize, shifts: usize) {
    let measured = run(world, days, seed, None);
    println!("{world} seed {seed} × {days} days: {measured:?}");
    assert_eq!(
        (measured.parts, measured.shifts),
        (parts, shifts),
        "{world}: the expected counts (a run that measures nothing fails)"
    );
    assert_eq!(
        measured.on_time, measured.parts,
        "{world} seed {seed}: late parts {:?}",
        measured.late
    );
    assert_eq!(
        measured.present, measured.shifts,
        "{world} seed {seed}: every shift present"
    );
    assert_eq!(
        measured.too_early, 0,
        "{world} seed {seed}: AC-LE-3 (commute arrivals; {} early errand arrival(s) not judged, F-Nd-7)",
        measured.errands
    );
}

// The literal counts: Market Town has two punctual parts a day (Alice's 05:30, Felix's 08:00) and two shifts;
// Social Café the same two parts and no employment; Commute Town five punctual parts a day (Dee has two) and
// four shifts.

#[test]
fn a_week_of_punctual_parts_in_market_town_and_commute_town_is_on_time() {
    assert_punctual(MARKET, 7, 1, 14, 14);
    assert_punctual(COMMUTE, 7, 1, 35, 28);
}

#[test]
#[ignore = "acceptance at the milestone (SPEED FIRST): 30 days × seeds 1, 7, 42 in three worlds, a few minutes"]
fn thirty_days_three_seeds_three_worlds_every_punctual_part_on_time() {
    for seed in [1, 7, 42] {
        assert_punctual(MARKET, 30, seed, 60, 60);
        assert_punctual(CAFE, 30, seed, 60, 0);
        assert_punctual(COMMUTE, 30, seed, 150, 120);
    }
}

#[test]
#[ignore = "acceptance at the milestone (SPEED FIRST): AC-LE2-4, recorded; a miss is a finding, never a retune"]
fn motivated_seats_are_on_time_at_least_nine_hundred_per_mille() {
    for world in [MARKET, COMMUTE] {
        for seed in [1, 7, 42] {
            let measured = run(world, 30, seed, Some(ALL_MOTIVATED));
            let share = measured.on_time * 1_000 / measured.parts.max(1);
            println!("motivated {world} seed {seed}: {share} ‰ on time; {measured:?}");
            assert!(measured.parts > 0, "{world}: punctual parts measured");
            assert!(share >= 900, "{world} seed {seed}: {share} ‰ < 900 ‰");
        }
    }
}

#[test]
#[ignore = "acceptance at the milestone (SPEED FIRST): the policy is data — at-boundary brings back the lateness"]
fn at_the_boundary_the_same_people_are_late() {
    for world in [MARKET, COMMUTE] {
        let measured = run(world, 7, 1, Some(AT_BOUNDARY));
        println!("at-boundary {world}: {measured:?}");
        assert!(
            measured.on_time < measured.parts,
            "{world}: setting out at the boundary is late at least once"
        );
    }
}

// ── ARC-330 (S35 N-f): a crowded one-door variant, generated at test time ─────────────────────────────

/// Commute Town rewritten to crowd its doorways (`pr-s35-nf-doorways.md` §4): `bodies` installed, every
/// resident living in `home-near`, and `home-near` and the bakery each a 4 × 3 m room with its one
/// doorway 0.4 m inside the south wall. Built in a scratch directory from the checked-in copy; never
/// checked in.
fn crowded_commute_town(into: &std::path::Path) {
    fn copy(from: &std::path::Path, to: &std::path::Path) {
        std::fs::create_dir_all(to).expect("a scratch directory");
        for entry in std::fs::read_dir(from).expect("the world's directory") {
            let entry = entry.expect("an entry");
            let target = to.join(entry.file_name());
            if entry.file_type().expect("a type").is_dir() {
                copy(&entry.path(), &target);
            } else {
                std::fs::copy(entry.path(), &target).expect("a copy");
            }
        }
    }
    fn rewrite(file: &std::path::Path, edits: &[(&str, &str)]) {
        let mut text = std::fs::read_to_string(file).expect("a world file");
        for (from, to) in edits {
            assert!(
                text.contains(from),
                "{}: '{from}' to rewrite",
                file.display()
            );
            text = text.replace(from, to);
        }
        std::fs::write(file, text).expect("rewritten");
    }
    copy(std::path::Path::new(COMMUTE), into);
    rewrite(
        &into.join("world.yaml"),
        &[
            ("  id: commute-town\n", "  id: crowded-commute-town\n"),
            ("  - presence\n", "  - presence\n  - bodies\n"),
        ],
    );
    let room =
        "\nbody:\n  floor: { min: { x: 0, y: 0 }, max: { x: 4000, y: 3000 } }\n  solids: []\n";
    for place in ["bakery", "home-near"] {
        let file = into.join("places").join(format!("{place}.yaml"));
        rewrite(
            &file,
            &[("here: { x: 1000, y: 0 }", "here: { x: 1000, y: 400 }")],
        );
        let mut text = std::fs::read_to_string(&file).expect("a place file");
        text.push_str(room);
        std::fs::write(&file, text).expect("a room");
    }
    for (person, at) in [
        ("ana", "{ x: 1000, y: 2000 }"),
        ("ben", "{ x: 2000, y: 2000 }"),
        ("cy", "{ x: 3000, y: 2000 }"),
        ("dee", "{ x: 2000, y: 1200 }"),
    ] {
        let file = into.join("people").join(format!("{person}.yaml"));
        let text = std::fs::read_to_string(&file)
            .expect("a person file")
            .replace("home-far", "home-near")
            .replace("home-mid", "home-near")
            .replace("position: { x: 1000, y: 2000 }", &format!("position: {at}"));
        std::fs::write(&file, text).expect("rewritten");
    }
}

/// AC-Nf-4's crowded row: 7 days × seeds 1 and 7 in the crowded variant — every punctual part on time,
/// every shift present, and no walk ends `stalled` or `no-route`.
#[test]
fn a_crowded_one_door_commute_town_keeps_its_doorways_passable() {
    let world = mineworld_test_support::scratch!(empty "crowded-commute-town");
    crowded_commute_town(&world);
    let world = world.to_str().expect("a printable path");
    for seed in [1, 7] {
        let measured = run(world, 7, seed, None);
        println!("crowded seed {seed}: {measured:?}");
        assert_eq!(
            measured.walks_failed,
            Vec::<String>::new(),
            "seed {seed}: walks that ended stalled or no-route"
        );
        assert_eq!(
            (measured.parts, measured.shifts),
            (35, 28),
            "seed {seed}: the expected counts"
        );
        assert_eq!(
            measured.on_time, measured.parts,
            "seed {seed}: {:?}",
            measured.late
        );
        assert_eq!(
            measured.present, measured.shifts,
            "seed {seed}: every shift present"
        );
    }
}
