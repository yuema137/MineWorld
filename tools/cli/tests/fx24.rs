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
use mineworld_employment::ShiftStarted;
use mineworld_presence::Arrived;
use mineworld_schedule::AgendaChanged;

use headless::{Tables, fresh, mineworld};
use social::decoded;

const MARKET: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/market-town");
const DAYS: &str = "7";
const SEED: &str = "1";
const PACES: [(&str, u32, &str); 3] = [("120", 24, "24×"), ("60", 12, "12×"), ("30", 6, "6×")];

/// One agenda part of one person: where it is, when it began, and when they first arrived there.
struct Part {
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
    let mut parts: Vec<(u64, Part)> = Vec::new();
    let mut last: BTreeMap<u64, usize> = BTreeMap::new();
    for fact in facts {
        match fact.event_type().as_str() {
            "arrived" => {
                let arrived: Arrived = decoded(fact).expect("presence's fact");
                let person = arrived.person().entity_id().raw();
                let place = arrived.location().place().entity_id().raw();
                at_place.insert(person, place);
                if let Some(&index) = last.get(&person) {
                    let part = &mut parts[index].1;
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
                    parts[index].1.ends = Some(began);
                }
                let already = at_place.get(&person) == Some(&place);
                last.insert(person, parts.len());
                parts.push((
                    person,
                    Part {
                        place,
                        began,
                        already,
                        arrived: None,
                        ends: None,
                    },
                ));
            }
            _ => {}
        }
    }
    parts.into_iter().map(|(_, part)| part).collect()
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

    let parts = journeys(&facts);
    let measured: Vec<&Part> = parts
        .iter()
        .filter(|part| !part.already && part.ends.is_some())
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
#[ignore = "about 60 s: three seven-day runs (ARC-69 / AC-TWc-9); run with --ignored"]
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
