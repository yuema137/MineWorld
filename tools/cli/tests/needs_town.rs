//! Step 35's acceptance on two towns (`pr-s35-nd-content.md` §5; step-35 §9): Market Town and the
//! test-only `commute-town`, both carrying `sleep` and `needs` as content, run by the same binary with no
//! code change between them (R-LE2-2). Thresholds are the step's, fixed before any run (I-S35-7); every
//! verdict is printed as PASS or FAIL with its numbers before an assertion fires (the `rhythm.rs` pattern).
//!
//! ```text
//! AC-Nd-2   3 days, seed 7: every seated person is asleep at 03:00 of each night after the first day,
//!           eats ≥ 2 nourishing items on each of days 2–3, no `buy` is refused and no
//!           `deduction-unpaid` is stated on days 2–3                                       default
//! AC-S35-5  7 days (default) and 30 days (milestone): `needs` states one `needs-assigned` per person
//!           with needs, its other genesis facts, and nothing after genesis
//! AC-S35-8  rule, 30 days × seeds 1, 7, 42, seated people: asleep at 03:00 ≥ 850 ‰; at 14:00 ≤ 100 ‰;
//!           mean sleep per person per day in [7.0 h, 9.5 h]; TV(food eaten by hour, ATUS Eating and
//!           drinking) ≤ 350 ‰; nobody at hunger ≥ 900 for more than 24 consecutive hours      milestone
//! AC-S35-1  30 days, seed 7: two runs and a SIGKILL-and-resume run have the same history     milestone
//! AC-S35-2b Market Town without the packs (fixture/needs.rs) prints what a binary built before step 35
//!           prints for the same copy (MINEWORLD_BASE_BINARY)                                  milestone
//! ```
//!
//! Elsewhere, unchanged and now run on the worlds with both packs: AC-S35-9 is `punctuality.rs`'s
//! 30-day × 3-seed test and `needs_controllers.rs`'s AC-Nc-3; AC-S35-12 is `market_town.rs`'s ASRB-2
//! growth bound (≤ 0.25 MiB per simulated day from day 30 to day 300, Market Town); AC-S35-2 (a) is the
//! milestone's `scripts/baselines.txt` comparison for Social Café, Lakeside and Bodies Yard.
//!
//! Hunger is replayed from the facts with the step's rates (both worlds run them: Market Town states
//! them in `configure/needs.yaml`, Commute Town leaves them unsaid): 120 an hour awake, 30 asleep, clamped
//! to 0 … 1 000, lowered at each nourishing `items-consumed` by the kind's `nourishment-assigned` value.
//! Facts are read as JSON by their slugs, so this test links neither the needs nor a market crate.

mod headless;
mod social;

#[path = "fixture/needs.rs"]
mod needs_fixture;

use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

use mineworld_contracts::EventEnvelope;
use mineworld_test_support::process;
use serde_json::Value;

use headless::{BINARY, Tables, fresh, mineworld};

const MARKET: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/market-town");
const COMMUTE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worlds/commute-town");
/// Each world and its seated people (Market Town's Otto has needs and no seat).
const WORLDS: [(&str, usize); 2] = [(MARKET, 11), (COMMUTE, 4)];

const DAY: i64 = 86_400;
const HOUR: i64 = 3_600;
/// The step's rates (step-35 §5.5), per hour.
const HUNGER_AWAKE: i64 = 120;
const HUNGER_ASLEEP: i64 = 30;
const FAMISHED: i64 = 900;
/// The ATUS table the oracle reads; the test does not depend on the model crate for it.
const ATUS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../cognition/motivated/data/atus-a3-2025.tsv"
));

fn id_of(value: &Value) -> Option<u64> {
    let entity = value.get("entity").unwrap_or(value);
    entity.as_u64().or_else(|| entity.as_str()?.parse().ok())
}

fn payload(fact: &EventEnvelope) -> Value {
    serde_json::from_slice(fact.payload().payload()).unwrap_or(Value::Null)
}

/// One finished run: its facts, its printed summary, and its span.
struct Run {
    facts: Vec<EventEnvelope>,
    printed: String,
    genesis: i64,
    days: i64,
}

fn run(world: &str, days: i64, seed: u64, name: &str) -> Run {
    let save = fresh(name);
    let output = mineworld(&[
        "run",
        world,
        "--headless",
        "--seed",
        &seed.to_string(),
        "--days",
        &days.to_string(),
        "--save",
        save.to_str().expect("a printable path"),
    ]);
    assert!(
        output.status.success(),
        "{world}: {}",
        headless::stderr(&output)
    );
    let facts = social::facts_of(&Tables::read(&save));
    let genesis = facts.first().map_or(0, |fact| fact.at().seconds());
    Run {
        facts,
        printed: headless::stdout(&output),
        genesis,
        days,
    }
}

/// What a run measured, per person entity id.
#[derive(Debug, Default)]
struct Measured {
    /// People with needs who walked at least once: the seated ones.
    seated: BTreeSet<u64>,
    with_needs: BTreeSet<u64>,
    /// Sleeps as `[from, to)`, an unfinished one closed at the run's end.
    sleeps: BTreeMap<u64, Vec<(i64, i64)>>,
    /// Nourishing meals as `(at, hunger lowered)`, one entry per item eaten.
    meals: BTreeMap<u64, Vec<(i64, i64)>>,
    /// The longest stretch at hunger ≥ 900, in seconds, per person.
    famished: BTreeMap<u64, i64>,
    /// Food eaten by hour of day, items.
    by_hour: [u64; 24],
}

fn measure(run: &Run) -> Measured {
    let end = run.genesis + run.days * DAY;
    let mut measured = Measured::default();
    let mut walked = BTreeSet::new();
    let mut nourishment: BTreeMap<u64, i64> = BTreeMap::new();
    let mut hunger: BTreeMap<u64, Hunger> = BTreeMap::new();
    let mut since: BTreeMap<u64, i64> = BTreeMap::new();
    for fact in &run.facts {
        let at = fact.at().seconds();
        let value = payload(fact);
        let person = || id_of(&value["person"]);
        match fact.event_type().as_str() {
            "needs-assigned" => {
                let person = person().expect("needs-assigned names a person");
                let level = value["levels"]["hunger"]
                    .as_i64()
                    .or_else(|| value["hunger"].as_i64())
                    .expect("needs-assigned states hunger");
                measured.with_needs.insert(person);
                hunger.insert(person, Hunger::new(at, level));
            }
            "nourishment-assigned" => {
                let item = id_of(&value["item"]).expect("an item");
                nourishment.insert(item, value["hunger"].as_i64().unwrap_or(0));
            }
            "walk-started" => {
                walked.extend(person());
            }
            "fell-asleep" => {
                let person = person().expect("fell-asleep names a person");
                since.insert(person, at);
                if let Some(level) = hunger.get_mut(&person) {
                    level.turn(at, true);
                }
            }
            "woke" => {
                let person = person().expect("woke names a person");
                if let Some(from) = since.remove(&person) {
                    measured.sleeps.entry(person).or_default().push((from, at));
                }
                if let Some(level) = hunger.get_mut(&person) {
                    level.turn(at, false);
                }
            }
            "items-consumed" => {
                let holder = id_of(&value["holder"]).expect("a holder");
                let item = id_of(&value["item"]).expect("an item");
                let fed = nourishment.get(&item).copied().unwrap_or(0);
                if fed > 0 {
                    let count = value["count"].as_i64().unwrap_or(1);
                    let hour = usize::try_from(at.rem_euclid(DAY) / HOUR).expect("an hour");
                    measured.by_hour[hour] += u64::try_from(count).expect("a count");
                    for _ in 0..count {
                        measured.meals.entry(holder).or_default().push((at, fed));
                        if let Some(level) = hunger.get_mut(&holder) {
                            level.eat(at, fed);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    for (person, from) in since {
        measured.sleeps.entry(person).or_default().push((from, end));
    }
    for (person, mut level) in hunger {
        level.advance(end);
        level.close(end);
        measured.famished.insert(person, level.longest);
    }
    measured.seated = measured.with_needs.intersection(&walked).copied().collect();
    measured
}

/// Hunger replayed from the facts, in 1/3 600 of a point so an hour's rise is exact.
struct Hunger {
    level: i64,
    at: i64,
    asleep: bool,
    famished_from: Option<i64>,
    longest: i64,
}

impl Hunger {
    const SCALE: i64 = HOUR;

    fn new(at: i64, level: i64) -> Self {
        let mut hunger = Self {
            level: level * Self::SCALE,
            at,
            asleep: false,
            famished_from: None,
            longest: 0,
        };
        if level >= FAMISHED {
            hunger.famished_from = Some(at);
        }
        hunger
    }

    fn rate(&self) -> i64 {
        if self.asleep {
            HUNGER_ASLEEP
        } else {
            HUNGER_AWAKE
        }
    }

    /// Rises to `to`, noting the instant it crosses into famished.
    fn advance(&mut self, to: i64) {
        let rate = self.rate();
        let threshold = FAMISHED * Self::SCALE;
        let next = (self.level + rate * (to - self.at)).min(1_000 * Self::SCALE);
        if self.famished_from.is_none() && next >= threshold {
            let crossing = self.at + (threshold - self.level).max(0) / rate;
            self.famished_from = Some(crossing);
        }
        self.level = next;
        self.at = to;
    }

    fn turn(&mut self, at: i64, asleep: bool) {
        self.advance(at);
        self.asleep = asleep;
    }

    fn eat(&mut self, at: i64, fed: i64) {
        self.advance(at);
        self.level = (self.level - fed * Self::SCALE).max(0);
        if self.level < FAMISHED * Self::SCALE {
            self.close(at);
        }
    }

    fn close(&mut self, at: i64) {
        if let Some(from) = self.famished_from.take() {
            self.longest = self.longest.max(at - from);
        }
    }
}

fn asleep_at(sleeps: &[(i64, i64)], at: i64) -> bool {
    sleeps.iter().any(|(from, to)| *from <= at && at < *to)
}

/// The ATUS *Eating and drinking* row, normalised to 1 000.
fn atus_eating() -> [i64; 24] {
    let row = ATUS
        .lines()
        .find(|line| line.starts_with("eating-and-drinking\t"))
        .expect("the ATUS table has the eating row");
    let values: Vec<i64> = row
        .split('\t')
        .skip(1)
        .map(|value| value.parse().expect("an integer"))
        .collect();
    assert_eq!(values.len(), 24, "24 hours");
    let total: i64 = values.iter().sum();
    std::array::from_fn(|hour| values[hour] * 1_000 / total)
}

/// The rhythm numbers of AC-S35-8 over the seated people of one run.
#[derive(Debug)]
struct Rhythm {
    asleep_03: i64,
    asleep_14: i64,
    sleep_hours: f64,
    tv: i64,
    longest_famished_h: f64,
    meals_per_day: f64,
}

fn rhythm(run: &Run, measured: &Measured) -> Rhythm {
    let none = Vec::new();
    let people = &measured.seated;
    let person_days = i64::try_from(people.len()).expect("fits") * run.days;
    let share_at = |hour: i64| -> i64 {
        let asleep: i64 = people
            .iter()
            .map(|person| {
                let sleeps = measured.sleeps.get(person).unwrap_or(&none);
                (0..run.days)
                    .filter(|day| asleep_at(sleeps, run.genesis + day * DAY + hour * HOUR))
                    .count() as i64
            })
            .sum();
        asleep * 1_000 / person_days.max(1)
    };
    let slept: i64 = people
        .iter()
        .flat_map(|person| measured.sleeps.get(person).unwrap_or(&none))
        .map(|(from, to)| to - from)
        .sum();
    let eaten: u64 = measured.by_hour.iter().sum();
    let atus = atus_eating();
    let tv: i64 = (0..24)
        .map(|hour| {
            let model = i64::try_from(measured.by_hour[hour] * 1_000 / eaten.max(1)).expect("fits");
            (model - atus[hour]).abs()
        })
        .sum::<i64>()
        / 2;
    let longest = people
        .iter()
        .filter_map(|person| measured.famished.get(person))
        .max()
        .copied()
        .unwrap_or(0);
    let meals: usize = people
        .iter()
        .map(|person| measured.meals.get(person).map_or(0, Vec::len))
        .sum();
    Rhythm {
        asleep_03: share_at(3),
        asleep_14: share_at(14),
        sleep_hours: slept as f64 / HOUR as f64 / person_days.max(1) as f64,
        tv,
        longest_famished_h: longest as f64 / HOUR as f64,
        meals_per_day: meals as f64 / person_days.max(1) as f64,
    }
}

/// Prints PASS or FAIL for a verdict and returns whether it held.
fn verdict(what: &str, holds: bool, numbers: &str) -> bool {
    println!("{} {what}: {numbers}", if holds { "PASS" } else { "FAIL" });
    holds
}

// ── AC-Nd-2 ──────────────────────────────────────────────────────────────────────────────────────

fn smoke(world: &str, seats: usize) -> bool {
    let run = run(world, 3, 7, "needs-town-smoke");
    let measured = measure(&run);
    let mut holds = verdict(
        &format!("{world}: seated people with needs"),
        measured.seated.len() == seats,
        &format!("{} of {seats}", measured.seated.len()),
    );
    let none = Vec::new();
    for person in &measured.seated {
        let sleeps = measured.sleeps.get(person).unwrap_or(&none);
        let nights: Vec<bool> = (1..run.days)
            .map(|day| asleep_at(sleeps, run.genesis + day * DAY + 3 * HOUR))
            .collect();
        let meals: Vec<usize> = (0..run.days)
            .map(|day| {
                let (from, to) = (run.genesis + day * DAY, run.genesis + (day + 1) * DAY);
                measured.meals.get(person).map_or(0, |meals| {
                    meals
                        .iter()
                        .filter(|(at, _)| (from..to).contains(at))
                        .count()
                })
            })
            .collect();
        holds &= verdict(
            &format!("{world}: #{person} asleep at 03:00 of nights 2–3, ≥ 2 meals on days 2–3"),
            nights.iter().all(|asleep| *asleep) && meals[1..].iter().all(|count| *count >= 2),
            &format!("asleep {nights:?}, meals per day {meals:?}"),
        );
    }
    let refused: Vec<&str> = headless::lines(&run.printed, "requests   buy rejected");
    let unpaid = run
        .facts
        .iter()
        .filter(|fact| fact.event_type().as_str() == "deduction-unpaid")
        .filter(|fact| fact.at().seconds() >= run.genesis + DAY)
        .count();
    holds &= verdict(
        &format!("{world}: no buy refused, no deduction unpaid on days 2–3"),
        refused.is_empty() && unpaid == 0,
        &format!("refused {refused:?}, unpaid {unpaid}"),
    );
    holds
}

#[test]
fn people_sleep_each_night_and_eat_twice_a_day_in_two_towns() {
    let mut holds = true;
    for (world, seats) in WORLDS {
        holds &= smoke(world, seats);
    }
    assert!(holds, "AC-Nd-2: see the FAIL lines above");
}

// ── AC-S35-5 ─────────────────────────────────────────────────────────────────────────────────────

/// `needs` states only genesis facts: one `needs-assigned` per person with needs, a
/// `nourishment-assigned` per nourished kind and its configuration — and nothing at any later instant,
/// so no decay Process and no threshold fact exist (a Process that stated nothing would cost no fact
/// either; N-b's own tests hold that none is scheduled, I-S35-3).
fn needs_states_only_genesis(world: &str, days: i64) {
    let run = run(world, days, 7, "needs-town-lazy");
    let mut assigned = BTreeMap::<u64, usize>::new();
    let mut later = Vec::new();
    for fact in &run.facts {
        if fact.provenance().emitted_by().as_str() != "needs" {
            continue;
        }
        if fact.at().seconds() != run.genesis {
            later.push(fact.event_type().as_str().to_owned());
        }
        if fact.event_type().as_str() == "needs-assigned" {
            let person = id_of(&payload(fact)["person"]).expect("a person");
            *assigned.entry(person).or_default() += 1;
        }
    }
    let once = !assigned.is_empty() && assigned.values().all(|count| *count == 1);
    let holds = verdict(
        &format!("{world} ({days} days): needs states only its genesis facts"),
        once && later.is_empty(),
        &format!("needs-assigned per person {assigned:?}; after genesis {later:?}"),
    );
    assert!(holds, "AC-S35-5 on {world}");
}

#[test]
fn needs_writes_nothing_while_hunger_rises_for_a_week() {
    for (world, _) in WORLDS {
        needs_states_only_genesis(world, 7);
    }
}

#[test]
#[ignore = "acceptance at the milestone (SPEED FIRST): AC-S35-5 over 30 days in two towns"]
fn needs_writes_nothing_while_hunger_rises_for_a_month() {
    for (world, _) in WORLDS {
        needs_states_only_genesis(world, 30);
    }
}

// ── AC-S35-8 ─────────────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "acceptance at the milestone (SPEED FIRST): AC-S35-8, 30 days × seeds 1, 7, 42 in two towns"]
fn the_towns_sleep_at_night_and_eat_at_meal_times() {
    let mut holds = true;
    for (world, seats) in WORLDS {
        for seed in [1, 7, 42] {
            let run = run(world, 30, seed, "needs-town-rhythm");
            let measured = measure(&run);
            let numbers = rhythm(&run, &measured);
            let what = format!("{world} seed {seed}");
            println!("{what}: {numbers:?}");
            for person in &measured.seated {
                println!(
                    "  #{person}: {} meals, {} sleeps, longest famished {:.1} h",
                    measured.meals.get(person).map_or(0, Vec::len),
                    measured.sleeps.get(person).map_or(0, Vec::len),
                    measured.famished.get(person).copied().unwrap_or(0) as f64 / HOUR as f64
                );
            }
            holds &= verdict(
                &format!("{what}: seated"),
                measured.seated.len() == seats,
                &measured.seated.len().to_string(),
            );
            holds &= verdict(
                &format!("{what}: asleep at 03:00 ≥ 850 ‰"),
                numbers.asleep_03 >= 850,
                &format!("{} ‰", numbers.asleep_03),
            );
            holds &= verdict(
                &format!("{what}: asleep at 14:00 ≤ 100 ‰"),
                numbers.asleep_14 <= 100,
                &format!("{} ‰", numbers.asleep_14),
            );
            holds &= verdict(
                &format!("{what}: sleep per person per day in [7.0 h, 9.5 h]"),
                (7.0..=9.5).contains(&numbers.sleep_hours),
                &format!("{:.2} h", numbers.sleep_hours),
            );
            holds &= verdict(
                &format!("{what}: TV(food by hour, ATUS eating) ≤ 350 ‰"),
                numbers.tv <= 350,
                &format!("{} ‰", numbers.tv),
            );
            holds &= verdict(
                &format!("{what}: nobody famished (≥ 900) for more than 24 h"),
                numbers.longest_famished_h <= 24.0,
                &format!(
                    "longest {:.1} h; {:.2} meals per person per day",
                    numbers.longest_famished_h, numbers.meals_per_day
                ),
            );
        }
    }
    assert!(holds, "AC-S35-8: see the FAIL lines above");
}

// ── AC-S35-1 ─────────────────────────────────────────────────────────────────────────────────────

/// Starts a 30-day run of `world` with `--save save` and SIGKILLs it once it has printed `day <day>`.
fn killed_at(world: &str, save: &Path, day: u64) {
    let mut child = Command::new(BINARY)
        .args([
            "run",
            world,
            "--headless",
            "--seed",
            "7",
            "--days",
            "30",
            "--save",
            save.to_str().expect("path"),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("the binary starts");
    let reader = BufReader::new(child.stdout.take().expect("piped"));
    let wanted = format!("day {day} ");
    for line in reader.lines() {
        if line.expect("utf-8").starts_with(&wanted) {
            break;
        }
    }
    let killed = process::kill(&mut child);
    assert!(
        killed.killed(),
        "killed by SIGKILL, not finished: {killed:?}"
    );
}

fn saved_run(world: &str, save: &Path) {
    let output = mineworld(&[
        "run",
        world,
        "--headless",
        "--seed",
        "7",
        "--days",
        "30",
        "--save",
        save.to_str().expect("path"),
    ]);
    assert!(
        output.status.success(),
        "{world}: {}",
        headless::stderr(&output)
    );
}

#[test]
#[ignore = "acceptance at the milestone (SPEED FIRST): AC-S35-1, 30 days, two runs and a kill-and-resume run"]
fn the_same_seed_is_the_same_town_after_a_kill() {
    for (world, _) in WORLDS {
        let (control, twin, killed) = (
            fresh("needs-town-control"),
            fresh("needs-town-twin"),
            fresh("needs-town-killed"),
        );
        saved_run(world, &control);
        saved_run(world, &twin);
        killed_at(world, &killed, 15);
        let head_at_kill = Tables::read(&killed).journal.len();
        saved_run(world, &killed);
        let control = Tables::read(&control);
        assert!(
            head_at_kill < control.journal.len(),
            "{world}: killed short of the end ({head_at_kill} of {})",
            control.journal.len()
        );
        Tables::read(&twin).assert_same_history(&control, &format!("{world}: a second run"));
        Tables::read(&killed)
            .assert_same_history(&control, &format!("{world}: killed and resumed"));
        println!("PASS {world}: AC-S35-1, head {}", control.journal.len());
    }
}

// ── AC-S35-2 (b) ─────────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "acceptance at the milestone (SPEED FIRST): AC-S35-2 (b) against a binary built before step 35, \
            named by MINEWORLD_BASE_BINARY"]
fn market_town_without_the_packs_is_the_base_town() {
    let base = std::env::var("MINEWORLD_BASE_BINARY")
        .expect("MINEWORLD_BASE_BINARY names a mineworld binary built before step 35");
    let copy = needs_fixture::without_needs("needs-town-removed", Path::new(MARKET));
    let world = copy.to_str().expect("a printable path");
    let printed = |binary: &str| -> Vec<String> {
        let output = Command::new(binary)
            .args(["run", world, "--headless", "--seed", "7", "--days", "30"])
            .output()
            .expect("the binary runs");
        assert!(output.status.success(), "{}", headless::stderr(&output));
        headless::deterministic(&headless::stdout(&output))
            .into_iter()
            .map(str::to_owned)
            .collect()
    };
    assert_eq!(printed(BINARY), printed(&base), "AC-S35-2 (b)");
}
