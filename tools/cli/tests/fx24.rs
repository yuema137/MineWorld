//! FX-24 (step-19 §9.2; `pr-tw-c-host-clock.md` §5.8, AC-TWc-9): are agendas feasible at 24× — and at 12× and 6×?
//!
//! `run worlds/market-town --days 7 --seed 1` at `--pace 120`, `60` and `30`, the equivalents of `cadence × scale`
//! for cadence 5 at 24×, 12× and 6×. Thresholds are step-19 §9.2's, fixed before anything was measured here, and are
//! not retuned: a FAIL is a material stop (§13), never a change to these numbers.
//!
//! ```text
//! (1) ≥ 90 % of agenda journeys end at the agenda's place within 25 % of the part of the day they serve,
//!     and every journey ends within 50 %
//! (2) ≥ 80 % of shift-started facts record the employee present
//! (3) both hold at all three paces
//! ```
//!
//! A journey is an agenda part whose place was elsewhere when it began; its time is from the part's start to the
//! first arrival at its place; its part is the time to the next agenda change. A journey that never arrives is a
//! failure of both clauses. A part that has no successor before the run ends is not measured.
//!
//! Ignored by default: three seven-day runs take about 60 s on a development machine, above the 30 s default-suite
//! budget. Run with `cargo test -p mineworld-cli --test fx24 -- --ignored --nocapture`.

mod headless;
mod social;

use std::collections::BTreeMap;

use mineworld_contracts::EventEnvelope;
use mineworld_employment::{Hired, ShiftStarted};
use mineworld_presence::Arrived;
use mineworld_schedule::AgendaChanged;
use mineworld_worldpack::WorldPack;

use headless::{Tables, fresh, mineworld};
use social::decoded;

const MARKET: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/market-town");
const DAYS: &str = "7";
const SEED: &str = "1";
const PACES: [(&str, u32, &str); 3] = [("120", 24, "24×"), ("60", 12, "12×"), ("30", 6, "6×")];

/// One agenda part of one person: where it is, when it began, and when they first arrived there.
struct Part {
    person: u64,
    place: u64,
    began: i64,
    /// Whether the person was already at the place when the part began: then there is no journey.
    already: bool,
    arrived: Option<i64>,
    /// The next agenda change, if the part has one before the run ends: the part's length.
    ends: Option<i64>,
}

/// The journeys of one run, as parts: every part of every person whose place was elsewhere when it began.
fn journeys(facts: &[EventEnvelope]) -> Vec<Part> {
    let mut at_place: BTreeMap<u64, u64> = BTreeMap::new();
    let mut parts: Vec<Part> = Vec::new();
    let mut last: BTreeMap<u64, usize> = BTreeMap::new();
    for fact in facts {
        match fact.event_type().as_str() {
            "arrived" => {
                let arrived: Arrived = decoded(fact).expect("presence's fact");
                let person = arrived.person().entity_id().raw();
                let place = arrived.location().place().entity_id().raw();
                at_place.insert(person, place);
                if let Some(&index) = last.get(&person) {
                    let part = &mut parts[index];
                    if part.place == place && !part.already && part.arrived.is_none() {
                        part.arrived = Some(fact.at().seconds());
                    }
                }
            }
            "agenda-changed" => {
                let changed: AgendaChanged = decoded(fact).expect("schedule's fact");
                let person = changed.person().entity_id().raw();
                let place = changed.place().entity_id().raw();
                let began = fact.at().seconds();
                if let Some(&index) = last.get(&person) {
                    parts[index].ends = Some(began);
                }
                let already = at_place.get(&person) == Some(&place);
                last.insert(person, parts.len());
                parts.push(Part {
                    person,
                    place,
                    began,
                    already,
                    arrived: None,
                    ends: None,
                });
            }
            _ => {}
        }
    }
    parts
}

/// The fraction of `numerator` in `denominator`, in per cent; 100 when there is nothing to judge.
fn per_cent(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        return 100.0;
    }
    #[allow(clippy::cast_precision_loss)]
    let fraction = numerator as f64 * 100.0 / denominator as f64;
    fraction
}

/// Entity ids of the market town, back to their authored keys, for a person reading the output.
fn names() -> BTreeMap<u64, String> {
    let loaded = WorldPack::read(MARKET)
        .expect("reads")
        .load(mineworld_contracts::WorldTime::EPOCH)
        .expect("loads");
    loaded
        .ids()
        .iter()
        .map(|(key, id)| (id.raw(), format!("{key}")))
        .collect()
}

/// The entity ids of the seats: the people some client or controller drives.
fn seated_ids() -> std::collections::BTreeSet<u64> {
    let pack = WorldPack::read(MARKET).expect("reads");
    let loaded = pack
        .load(mineworld_contracts::WorldTime::EPOCH)
        .expect("loads");
    pack.seats()
        .iter()
        .map(|key| loaded.id(key).expect("a seat is a key of the pack").raw())
        .collect()
}

/// `day D hh:mm` of a world instant, for reading.
fn clock(at: i64) -> String {
    let day = at.div_euclid(86_400) + 1;
    let minute = at.rem_euclid(86_400) / 60;
    format!("day {day} {:02}:{:02}", minute / 60, minute % 60)
}

/// Runs the world at one pace and measures the three clauses. Returns the printed line and whether it holds.
fn measure(pace: &str, scale: &str) -> (String, bool) {
    let save = fresh(&format!("fx24-{pace}"));
    let save_text = save.to_str().expect("a printable path").to_owned();
    let output = mineworld(&[
        "run",
        MARKET,
        "--headless",
        "--seed",
        SEED,
        "--days",
        DAYS,
        "--pace",
        pace,
        "--save",
        &save_text,
    ]);
    assert!(
        output.status.success(),
        "run --pace {pace}: {}",
        headless::stderr(&output)
    );
    let tables = Tables::read(&save);
    let facts = social::facts_of(&tables);
    let names = names();
    let name = |id: u64| names.get(&id).cloned().unwrap_or_else(|| format!("#{id}"));

    // Only the people someone drives are measured. A Person left out of the seats has no controller to walk
    // them, so an agenda cannot move them: `routines.rs` asserts that otto, who is no seat, never moves. Those
    // parts are excluded and counted, not judged.
    let seated = seated_ids();
    let parts = journeys(&facts);
    let undriven = parts
        .iter()
        .filter(|part| !part.already && part.ends.is_some() && !seated.contains(&part.person))
        .count();
    println!("  excluded {undriven} journeys of Persons no seat drives (they never move)");
    let measured: Vec<&Part> = parts
        .iter()
        .filter(|part| !part.already && part.ends.is_some() && seated.contains(&part.person))
        .collect();
    let within_25 = measured
        .iter()
        .filter(|part| {
            part.arrived.is_some_and(|arrived| {
                let part_length = part.ends.expect("measured") - part.began;
                (arrived - part.began) * 4 <= part_length
            })
        })
        .count();
    let within_50 = measured
        .iter()
        .filter(|part| {
            part.arrived.is_some_and(|arrived| {
                let part_length = part.ends.expect("measured") - part.began;
                (arrived - part.began) * 2 <= part_length
            })
        })
        .count();
    let journeys_total = measured.len();
    let clause_1 = per_cent(within_25, journeys_total) >= 90.0 && within_50 == journeys_total;
    for part in measured.iter().filter(|part| {
        !part.arrived.is_some_and(|arrived| {
            (arrived - part.began) * 2 <= part.ends.expect("measured") - part.began
        })
    }) {
        let length = part.ends.expect("measured") - part.began;
        match part.arrived {
            Some(arrived) => println!(
                "  over 50 %: {} began {} for {}; arrived {} after {} min of a {} min part ({:.0} %)",
                name(part.person),
                clock(part.began),
                name(part.place),
                clock(arrived),
                (arrived - part.began) / 60,
                length / 60,
                (arrived - part.began) as f64 * 100.0 / length as f64,
            ),
            None => println!(
                "  over 50 %: {} began {} for {}; never arrived in a {} min part",
                name(part.person),
                clock(part.began),
                name(part.place),
                length / 60,
            ),
        }
    }

    let shifts: Vec<ShiftStarted> = facts.iter().filter_map(decoded::<ShiftStarted>).collect();
    let present = shifts.iter().filter(|shift| shift.present()).count();
    let clause_2 = per_cent(present, shifts.len()) >= 80.0;

    let holds = clause_1 && clause_2;
    let line = format!(
        "FX-24 {scale} (--pace {pace}): journeys {journeys_total}, within 25 % {within_25} ({:.1} %), within 50 % \
         {within_50}; shifts {} of {} present ({:.1} %); {}",
        per_cent(within_25, journeys_total),
        present,
        shifts.len(),
        per_cent(present, shifts.len()),
        if holds { "HOLDS" } else { "FAILS" },
    );
    (line, holds)
}

#[test]
#[ignore = "FX-24 clause 2 fails for content reasons, F-TWc-1 (shift starts before the employee reaches the workplace); \
            the lead-time content PR \"FX-24 lead time\" (QTW-1 (a)) makes it pass. Run with --ignored to see the numbers. \
            Ruling R-TWc-FX24."]
fn agendas_stay_feasible_at_24_12_and_6_times() {
    let mut lines = Vec::new();
    let mut failing = Vec::new();
    for (pace, scale, label) in PACES {
        let (line, holds) = measure(pace, label);
        println!("{line}");
        lines.push(line);
        if !holds {
            failing.push(scale);
        }
    }
    assert!(
        failing.is_empty(),
        "FX-24 fails at {failing:?}× — a material stop (§13), never retuned:\n{}",
        lines.join("\n")
    );
}

/// Diagnostic for the shift clause (not a threshold), at the default pace: every `shift-started`, whether its
/// `present` flag is true, the employee's workplace (from `hired`), where the employee was at the shift's start
/// (their last arrival at or before it), and when they first arrived at the workplace afterwards.
#[test]
#[ignore = "diagnostic: one seven-day run at the default pace (ARC-69 item 11 investigation)"]
fn shift_presence_diagnostic_at_the_default_pace() {
    let save = fresh("fx24-shifts-900");
    let save_text = save.to_str().expect("a printable path").to_owned();
    let output = mineworld(&[
        "run",
        MARKET,
        "--headless",
        "--seed",
        SEED,
        "--days",
        DAYS,
        "--save",
        &save_text,
    ]);
    assert!(output.status.success(), "{}", headless::stderr(&output));
    let facts = social::facts_of(&Tables::read(&save));
    let names = names();
    let name = |id: u64| names.get(&id).cloned().unwrap_or_else(|| format!("#{id}"));

    let mut workplace: BTreeMap<u64, u64> = BTreeMap::new();
    let mut location: BTreeMap<u64, u64> = BTreeMap::new();
    let mut arrivals: Vec<(i64, u64, u64)> = Vec::new();
    let mut shifts: Vec<(i64, u64, bool, Option<u64>)> = Vec::new();
    for fact in &facts {
        match fact.event_type().as_str() {
            "hired" => {
                let hired: Hired = decoded(fact).expect("employment's fact");
                workplace.insert(
                    hired.employee().entity_id().raw(),
                    hired.job().workplace().entity_id().raw(),
                );
            }
            "arrived" => {
                let arrived: Arrived = decoded(fact).expect("presence's fact");
                let person = arrived.person().entity_id().raw();
                let place = arrived.location().place().entity_id().raw();
                location.insert(person, place);
                arrivals.push((fact.at().seconds(), person, place));
            }
            "shift-started" => {
                let shift: ShiftStarted = decoded(fact).expect("employment's fact");
                let person = shift.employee().entity_id().raw();
                shifts.push((
                    fact.at().seconds(),
                    person,
                    shift.present(),
                    location.get(&person).copied(),
                ));
            }
            _ => {}
        }
    }
    for (at, person, present, at_start) in &shifts {
        let work = workplace.get(person).copied();
        let first_there = work.and_then(|work| {
            arrivals
                .iter()
                .find(|(when, who, place)| who == person && place == &work && when >= at)
                .map(|(when, ..)| *when)
        });
        println!(
            "shift {} at {}: present={present}; workplace {}; at the start {}; first at the workplace {}",
            name(*person),
            clock(*at),
            work.map_or("none".to_owned(), name),
            at_start.map_or("unknown".to_owned(), name),
            first_there.map_or("never".to_owned(), |when| format!(
                "{} (+{} min)",
                clock(when),
                (when - at) / 60
            )),
        );
    }
}
