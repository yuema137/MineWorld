//! Controllers and needs on two worlds (`pr-s35-nc-controllers.md` §5): `rule` people go to bed at night,
//! eat what they carry, and still arrive on time for work, in Market Town and in the test-only
//! `commute-town`, with no code change between the two (R-LE2-2). Since N-d both worlds carry the packs
//! as content (`pr-s35-nd-content.md` §§3–4); `fixture/needs.rs` now makes only the removal copy that
//! AC-Nc-1 compares.
//!
//! ```text
//! AC-Nc-4   3 days, seed 7, all rule: every seated person falls asleep at least once between 22:00 and
//!           02:00, and eats at least one nourishing item per day of the run (F-Nc-6)         default
//! AC-Nc-3   30 days × seeds 1, 7, 42: every shift present, nobody late                        milestone
//! AC-Nc3-4  the same, every seat `motivated` (`pr-s35-nc3-motivated-sleep.md` §5)                milestone
//! AC-Nc-1   neither pack: `run` output identical to the base for all-rule and all-motivated  milestone
//!           (the base is a binary built before N-c, named by MINEWORLD_BASE_BINARY; Lakeside and
//!           Bodies Yard by the milestone's baseline digests)
//! ```
//!
//! Facts are read as JSON by their slugs, so this test links neither the needs nor a market crate.

mod headless;
mod social;

#[path = "fixture/needs.rs"]
mod needs_fixture;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use mineworld_contracts::EventEnvelope;
use serde_json::Value;

use headless::{Tables, fresh, mineworld};

const MARKET: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/market-town");
const COMMUTE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worlds/commute-town");

const DAY: i64 = 86_400;

fn id_of(value: &Value) -> Option<u64> {
    let entity = value.get("entity").unwrap_or(value);
    entity.as_u64().or_else(|| entity.as_str()?.parse().ok())
}

fn payload(fact: &EventEnvelope) -> Value {
    serde_json::from_slice(fact.payload().payload()).unwrap_or(Value::Null)
}

/// What one run measured, per person entity id.
#[derive(Debug, Default)]
struct Measured {
    /// Who fell asleep between 22:00 and 02:00 at least once, and every fall-asleep's time of day.
    asleep_at_night: BTreeSet<u64>,
    fell_asleep: BTreeMap<u64, Vec<String>>,
    /// When each nourishing meal was eaten, per person.
    meals: BTreeMap<u64, Vec<String>>,
    /// Seated people with needs, by entity id: a `needs-assigned` person who walked at least once (a
    /// person no controller drives — Market Town's Otto — never walks).
    people: BTreeSet<u64>,
    /// Everybody with needs, and everybody who started a walk.
    with_needs: BTreeSet<u64>,
    walked: BTreeSet<u64>,
    /// How sleeps ended: `woke` facts by reason.
    woke: BTreeMap<String, usize>,
}

/// `d<day> HH:MM` of an instant, days counted from genesis: for a reader of the printed measurement.
fn clock(at: i64, genesis: i64) -> String {
    let of_day = at.rem_euclid(DAY);
    format!(
        "d{} {:02}:{:02}",
        (at - genesis) / DAY,
        of_day / 3_600,
        of_day % 3_600 / 60
    )
}

fn measure(facts: &[EventEnvelope], nourishing: &BTreeSet<u64>) -> Measured {
    let genesis = facts.first().map_or(0, |fact| fact.at().seconds());
    let mut measured = Measured::default();
    for fact in facts {
        let at = fact.at().seconds();
        let value = payload(fact);
        match fact.event_type().as_str() {
            "needs-assigned" => {
                if let Some(person) = id_of(&value["person"]) {
                    measured.with_needs.insert(person);
                }
            }
            "walk-started" => {
                if let Some(person) = id_of(&value["person"]) {
                    measured.walked.insert(person);
                }
            }
            "fell-asleep" => {
                let person = id_of(&value["person"]).expect("fell-asleep names a person");
                let of_day = at.rem_euclid(DAY);
                if !(2 * 3_600..22 * 3_600).contains(&of_day) {
                    measured.asleep_at_night.insert(person);
                }
                measured
                    .fell_asleep
                    .entry(person)
                    .or_default()
                    .push(clock(at, genesis));
            }
            "woke" => {
                let reason = value["reason"].as_str().unwrap_or("?").to_owned();
                *measured.woke.entry(reason).or_default() += 1;
            }
            "items-consumed" => {
                let holder = id_of(&value["holder"]).expect("a holder");
                let item = id_of(&value["item"]).expect("an item");
                if nourishing.contains(&item) {
                    let count = value["count"].as_u64().unwrap_or(1);
                    let meals = measured.meals.entry(holder).or_default();
                    for _ in 0..count {
                        meals.push(clock(at, genesis));
                    }
                }
            }
            _ => {}
        }
    }
    measured.people = measured
        .with_needs
        .intersection(&measured.walked)
        .copied()
        .collect();
    measured
}

/// The item kinds the copy gave a nourishment, from `nourishment-assigned`.
fn nourishing(facts: &[EventEnvelope]) -> BTreeSet<u64> {
    facts
        .iter()
        .filter(|fact| fact.event_type().as_str() == "nourishment-assigned")
        .filter_map(|fact| {
            let value = payload(fact);
            (value["hunger"].as_u64()? > 0).then(|| id_of(&value["item"]))?
        })
        .collect()
}

/// `run` of `world` (which carries both packs) for `days` with `seed`.
fn run(world: &str, days: u64, seed: u64) -> Measured {
    let save = fresh(&format!("needs-ctl-save-{days}-{seed}"));
    let (days, seed) = (days.to_string(), seed.to_string());
    let output = mineworld(&[
        "run",
        world,
        "--headless",
        "--seed",
        &seed,
        "--days",
        &days,
        "--save",
        save.to_str().expect("a printable path"),
    ]);
    assert!(
        output.status.success(),
        "run {world}: {}",
        headless::stderr(&output)
    );
    let facts = social::facts_of(&Tables::read(&save));
    let food = nourishing(&facts);
    assert!(!food.is_empty(), "{world}: the world nourishes something");
    measure(&facts, &food)
}

/// AC-Nc-4 on one world: every person with needs falls asleep between 22:00 and 02:00 at least once and
/// eats at least one nourishing item per day of the run. "One per day" is read as a count — at least
/// `days` meals each — not as a meal on each calendar day (F-Nc-6): a person carries at most six things
/// (inventory's `PERSON_CAPACITY`), and `rule` does not shop (step-35 §6.2), so in a world without a
/// shop the larder, not the band, decides when meals run out; the per-person rhythm over a month is
/// N-d's AC-S35-8.
fn assert_sleeps_and_eats(world: &str, days: i64, seats: usize) {
    let measured = run(world, u64::try_from(days).expect("days"), 7);
    println!("{world}: {measured:#?}");
    assert_eq!(
        measured.people.len(),
        seats,
        "{world}: people with needs (a run that measures nothing fails)"
    );
    for person in &measured.people {
        assert!(
            measured.asleep_at_night.contains(person),
            "{world}: #{person} never fell asleep between 22:00 and 02:00: {:?}",
            measured.fell_asleep.get(person)
        );
        let meals = measured.meals.get(person).map_or(0, Vec::len);
        assert!(
            meals >= usize::try_from(days).expect("days"),
            "{world}: #{person} ate {meals} times in {days} days"
        );
    }
    // `woke { reason: acted }` is printed, not asserted: a sleeper pushed by somebody else's walk is
    // `arrived` and so wakes as acting (sleep's rule, F-Nc-7), which no controller decides.
}

#[test]
fn rule_people_sleep_at_night_and_eat_in_two_worlds() {
    // Market Town's eleven seats (Otto is no seat); Commute Town's four.
    assert_sleeps_and_eats(MARKET, 3, 11);
    assert_sleeps_and_eats(COMMUTE, 3, 4);
}

/// Shifts in a run: `(shift-started, of them present, late-for-shift)`, read by slug.
fn shifts(facts: &[EventEnvelope]) -> (usize, usize, usize) {
    let mut counted = (0, 0, 0);
    for fact in facts {
        match fact.event_type().as_str() {
            "shift-started" => {
                counted.0 += 1;
                if payload(fact)["present"].as_bool() == Some(true) {
                    counted.1 += 1;
                }
            }
            "late-for-shift" => counted.2 += 1,
            _ => {}
        }
    }
    counted
}

#[test]
#[ignore = "acceptance at the milestone (SPEED FIRST): AC-Nc-3, 30 days × seeds 1, 7, 42 in two worlds"]
fn with_needs_and_sleep_every_shift_is_still_present() {
    every_shift_is_present(None);
}

#[test]
#[ignore = "acceptance at the milestone (SPEED FIRST): AC-Nc3-4, all motivated, 30 days × seeds 1, 7, 42 in two worlds"]
fn with_motivated_seats_every_shift_is_still_present() {
    every_shift_is_present(Some(ALL_MOTIVATED));
}

/// AC-Nc-3 (and, with every seat `motivated`, AC-Nc3-4): 30 days × seeds 1, 7, 42, every shift present.
fn every_shift_is_present(decisions: Option<&str>) {
    // AC-LE2-1's literal counts: Market Town three shifts a day (Alice's and Felix's, and since S29-b
    // Ivan's 07:00 morning shift at the station newsstand, `pr-s29b-newsstand.md` §2 item 4), Commute
    // Town four.
    for (world, per_day) in [(MARKET, 3), (COMMUTE, 4)] {
        for seed in [1, 7, 42] {
            let tag = if decisions.is_some() {
                "motivated"
            } else {
                "rule"
            };
            let save = fresh(&format!("needs-ac3-save-{tag}-{seed}"));
            let seed_text = seed.to_string();
            let mut arguments = vec![
                "run",
                world,
                "--headless",
                "--seed",
                &seed_text,
                "--days",
                "30",
                "--save",
                save.to_str().expect("a printable path"),
            ];
            if let Some(decisions) = decisions {
                arguments.extend(["--decisions", decisions]);
            }
            let output = mineworld(&arguments);
            assert!(output.status.success(), "{}", headless::stderr(&output));
            let (started, present, late) = shifts(&social::facts_of(&Tables::read(&save)));
            println!("{world} seed {seed}: {present} of {started} present, {late} late");
            assert_eq!(started, 30 * per_day, "{world} seed {seed}");
            assert_eq!(present, started, "{world} seed {seed}: AC-LE2-1 at 100 %");
            assert_eq!(late, 0, "{world} seed {seed}: nobody late");
        }
    }
}

const CAFE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/social-cafe");
const ALL_MOTIVATED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../cognition/motivated/examples/social-cafe-all-motivated.yaml"
);

/// The deterministic lines of `binary run world` for 30 days, seed 7, with an optional decision list.
fn printed(binary: &str, world: &str, decisions: Option<&str>) -> Vec<String> {
    let mut arguments = vec!["run", world, "--headless", "--seed", "7", "--days", "30"];
    if let Some(decisions) = decisions {
        arguments.extend(["--decisions", decisions]);
    }
    let output = std::process::Command::new(binary)
        .args(&arguments)
        .output()
        .expect("the binary runs");
    assert!(output.status.success(), "{}", headless::stderr(&output));
    headless::deterministic(&headless::stdout(&output))
        .into_iter()
        .map(str::to_owned)
        .collect()
}

#[test]
#[ignore = "acceptance at the milestone (SPEED FIRST): AC-Nc-1 against a binary built before N-c, named by \
            MINEWORLD_BASE_BINARY; Lakeside and Bodies Yard by the milestone's baseline digests"]
fn without_the_packs_every_run_is_the_base_run() {
    let base = std::env::var("MINEWORLD_BASE_BINARY")
        .expect("MINEWORLD_BASE_BINARY names a mineworld binary built before N-c");
    // Market Town carries both packs since N-d: its "neither pack" form is the removal copy.
    let market = needs_fixture::without_needs("needs-ac1-market", Path::new(MARKET));
    for world in [market.to_str().expect("a printable path"), CAFE] {
        for decisions in [None, Some(ALL_MOTIVATED)] {
            assert_eq!(
                printed(headless::BINARY, world, decisions),
                printed(&base, world, decisions),
                "{world} with {decisions:?}: AC-Nc-1"
            );
        }
    }
}
