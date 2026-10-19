//! Lateness has consequences (`pr-le-leave-early.md` §0.6, §0.8; `ARC-287`), in Market Town and in the
//! test-only variant `commute-town`, whose geometry and staff nothing in any crate knows (AC-LE2-7).
//!
//! ```text
//! AC-LE2-1   default policy: zero `late-for-shift` in either world (seven days, seed 1)
//! AC-LE2-2   `commute.mode: at-boundary`: every employee late at least once in seven days, each fact's
//!            count and level right against its own history; in commute-town the mill (no manager,
//!            deduction on) follows each with `wage-deduction-due` of the formula's amount, and economy
//!            answers it — a transfer or `deduction-unpaid` caused by it
//! AC-LE2-3   a strict `rule` boss in commute-town: the bakery's lateness is told to Ana, nothing is
//!            deducted automatically, and her accepted `penalize` states a deduction money then answers
//! ```
//!
//! The refusals of AC-LE2-3 (a second penalty, one above the bound, a non-manager's) are dispatch
//! answers and are asserted in `systems/employment/tests/lateness.rs`. Facts are read as JSON by their
//! slugs: no market crate is named here (`ARC-35` item 3).

mod headless;
mod social;

use std::collections::BTreeMap;

use mineworld_contracts::{Causation, EventEnvelope};
use serde_json::Value;

use headless::{Tables, fresh, mineworld};

const MARKET: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/market-town");
const COMMUTE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worlds/commute-town");
const AT_BOUNDARY: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/decisions/rule-at-boundary.yaml"
);
const STRICT_BOSS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/decisions/rule-at-boundary-strict-boss.yaml"
);

/// The defaults of §0.6, fixed before any run: a 7-day window, warning at 2, final at 3; 10 per started
/// late minute, at most 600 a shift.
const WINDOW: i64 = 7 * 86_400;
const PER_MINUTE: u64 = 10;
const CAP: u64 = 600;

fn json(fact: &EventEnvelope) -> Value {
    serde_json::from_slice(fact.payload().payload()).expect("a JSON payload")
}

fn person(value: &Value) -> u64 {
    let entity = value.get("entity").unwrap_or(value);
    entity
        .as_u64()
        .or_else(|| entity.as_str()?.parse().ok())
        .expect("an entity id")
}

/// Every fact of `run` of `world` for `days` with seed 1 and `decisions`.
fn run(world: &str, days: u64, decisions: Option<&str>, name: &str) -> Vec<EventEnvelope> {
    let save = fresh(&format!("lateness-{name}"));
    let save_text = save.to_str().expect("a printable path").to_owned();
    let days = days.to_string();
    let mut arguments = vec![
        "run",
        world,
        "--headless",
        "--seed",
        "1",
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
    social::facts_of(&Tables::read(&save))
}

fn of<'a>(facts: &'a [EventEnvelope], slug: &str) -> Vec<&'a EventEnvelope> {
    facts
        .iter()
        .filter(|fact| fact.event_type().as_str() == slug)
        .collect()
}

/// Every `late-for-shift`'s count and level, checked against the employee's own earlier complaints.
fn assert_counted(facts: &[EventEnvelope]) {
    let mut history: BTreeMap<u64, Vec<i64>> = BTreeMap::new();
    for fact in of(facts, "late-for-shift") {
        let late = json(fact);
        let who = person(&late["employee"]);
        let shift = late["shift"].as_i64().expect("an instant");
        let earlier = history.entry(who).or_default();
        let count = earlier.iter().filter(|at| shift - **at < WINDOW).count() + 1;
        let level = match count {
            1 => "notice",
            2 => "warning",
            _ => "final",
        };
        assert_eq!(
            (late["count"].as_u64(), late["level"].as_str()),
            (Some(count as u64), Some(level)),
            "{late}"
        );
        earlier.push(shift);
    }
}

#[test]
fn on_the_default_policy_nobody_is_late() {
    for (world, name) in [(MARKET, "market"), (COMMUTE, "commute")] {
        let facts = run(world, 7, None, &format!("default-{name}"));
        assert!(
            !of(&facts, "shift-started").is_empty(),
            "{world}: shifts ran"
        );
        assert!(
            of(&facts, "late-for-shift").is_empty(),
            "{world}: AC-LE2-1, zero complaints"
        );
        assert!(of(&facts, "wage-deduction-due").is_empty());
    }
}

#[test]
fn setting_out_at_the_boundary_is_late_counted_and_on_the_mill_deducted() {
    for (world, name, employees) in [(MARKET, "market", 2), (COMMUTE, "commute", 4)] {
        let facts = run(world, 7, Some(AT_BOUNDARY), &format!("boundary-{name}"));
        let late = of(&facts, "late-for-shift");
        let who: std::collections::BTreeSet<u64> = late
            .iter()
            .map(|fact| person(&json(fact)["employee"]))
            .collect();
        println!(
            "{world}: {} late-for-shift, {} employees",
            late.len(),
            who.len()
        );
        assert_eq!(
            who.len(),
            employees,
            "{world}: every employee late at least once"
        );
        assert_counted(&facts);
        if world == MARKET {
            assert!(
                of(&facts, "wage-deduction-due").is_empty(),
                "Market Town leaves deduction off"
            );
            continue;
        }
        // Commute Town: the mill has no manager, so every one of its complaints is deducted at once;
        // the bakery's go to Ana, and nothing is deducted for them without her.
        let deductions = of(&facts, "wage-deduction-due");
        // Employers with a manager: their complaints name one, except the manager's own.
        let managed: std::collections::BTreeSet<u64> = late
            .iter()
            .map(|fact| json(fact))
            .filter(|late| late.get("manager").is_some())
            .map(|late| person(&late["employer"]))
            .collect();
        for fact in &late {
            let complaint = json(fact);
            let caused: Vec<&&EventEnvelope> = deductions
                .iter()
                .filter(|due| *due.caused_by() == Causation::Event(fact.id()))
                .collect();
            if managed.contains(&person(&complaint["employer"])) {
                assert!(caused.is_empty(), "a manager decides: {complaint}");
                continue;
            }
            assert_eq!(caused.len(), 1, "the author's numbers decide: {complaint}");
            let late_by = complaint["late_by"].as_u64().expect("seconds");
            let expected = (PER_MINUTE * late_by.div_ceil(60)).min(CAP);
            assert_eq!(json(caused[0])["amount"].as_u64(), Some(expected));
            assert_answered(&facts, caused[0]);
        }
    }
}

/// Economy answered `due`: a transfer from the employee, or `deduction-unpaid`, caused by it.
fn assert_answered(facts: &[EventEnvelope], due: &EventEnvelope) {
    let answers = facts
        .iter()
        .filter(|fact| *fact.caused_by() == Causation::Event(due.id()))
        .filter(|fact| {
            matches!(
                fact.event_type().as_str(),
                "money-transferred" | "deduction-unpaid"
            )
        })
        .count();
    assert_eq!(answers, 1, "economy answers {}", json(due));
}

#[test]
fn a_strict_boss_penalizes_the_bakerys_lateness_and_money_moves() {
    let facts = run(COMMUTE, 7, Some(STRICT_BOSS), "boss");
    let told: Vec<Value> = of(&facts, "late-for-shift")
        .iter()
        .map(|fact| json(fact))
        .filter(|late| late.get("manager").is_some())
        .collect();
    assert!(
        !told.is_empty(),
        "the bakery's lateness is told to its manager"
    );
    let penalties: Vec<&EventEnvelope> = of(&facts, "wage-deduction-due")
        .into_iter()
        .filter(|due| matches!(due.caused_by(), Causation::Action(_)))
        .collect();
    println!(
        "{} told to the manager, {} penalties",
        told.len(),
        penalties.len()
    );
    assert!(!penalties.is_empty(), "the boss penalized");
    assert!(penalties.len() <= told.len(), "at most once per lateness");
    for due in penalties {
        assert_answered(&facts, due);
    }
}
