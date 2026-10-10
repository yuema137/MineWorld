//! What a Lakeside run printed and saved, read as step-16 PD-47 measures it: activity, the catch's
//! cause chain, and who fished and ate in which 30-day bucket.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

use mineworld_contracts::{Causation, EventEnvelope};
use mineworld_test_support::process;
use serde_json::Value;

use super::{BINARY, Lake, SEED, entity, lakeside, roots, text};
use crate::headless::{Tables, every_seat_active_in_every_bucket, lines};

const DAY: i64 = 86_400;
/// The width of the run summary's activity buckets, in days.
const BUCKET_DAYS: i64 = 30;

/// Every fact of a save, decoded.
pub fn facts_of(save: &Path) -> Vec<EventEnvelope> {
    Tables::read(save)
        .facts
        .iter()
        .map(|(_, bytes)| {
            mineworld_persistence::format::decode(bytes, "fact").expect("a stored fact decodes")
        })
        .collect()
}

fn payload(fact: &EventEnvelope) -> Value {
    serde_json::from_slice(fact.payload().payload()).expect("a JSON payload")
}

/// Whether `fact` is of the type `slug`, read as a string (EC-5).
pub fn of(fact: &EventEnvelope, slug: &str) -> bool {
    fact.event_type().as_str() == slug
}

/// The fact types a run printed (`facts      <type> <count>`).
pub fn fact_types(printed: &str) -> BTreeSet<String> {
    lines(printed, "facts      ")
        .iter()
        .filter_map(|line| line.split_whitespace().nth(1))
        .map(str::to_owned)
        .collect()
}

/// The accepted `fish` requests whose catch came into the angler's holdings — E-c's EC-7 chain: a
/// `fishing-started` caused by an action, the `catch` process's `fishing-ended { caught: true }`, and
/// inventory's `items-produced` for the same angler at the started catch's due instant.
fn landed_catches(facts: &[EventEnvelope]) -> usize {
    let started: BTreeSet<(String, String)> = facts
        .iter()
        .filter(|fact| of(fact, "fishing-started"))
        .filter(|fact| matches!(fact.caused_by(), Causation::Action(_)))
        .map(|fact| {
            let p = payload(fact);
            (entity(&p["angler"]), p["due"].to_string())
        })
        .collect();
    let landed: BTreeSet<(String, String)> = facts
        .iter()
        .filter(|fact| of(fact, "fishing-ended"))
        .filter(|fact| payload(fact)["caught"] == Value::Bool(true))
        .filter_map(|fact| match fact.caused_by() {
            Causation::Process(process) => {
                Some((format!("{process:?}"), entity(&payload(fact)["angler"])))
            }
            _ => None,
        })
        .collect();
    facts
        .iter()
        .filter(|fact| of(fact, "items-produced"))
        .filter(|fact| {
            let Causation::Process(process) = fact.caused_by() else {
                return false;
            };
            let holder = entity(&payload(fact)["holder"]);
            let at = serde_json::to_value(fact.at()).expect("an instant");
            landed.contains(&(format!("{process:?}"), holder.clone()))
                && started.contains(&(holder, at.to_string()))
        })
        .count()
}

/// Who fished, who ate, and whether the catch was eaten, by 30-day bucket.
#[derive(Default)]
struct Tally {
    fished: BTreeMap<(i64, String), u64>,
    consumed: BTreeMap<(i64, String), u64>,
    catch_eaten: BTreeMap<i64, u64>,
    /// The first kind of the Entity Pack eaten, located.
    pack_kind_eaten: Option<String>,
}

impl Tally {
    fn count(lake: &Lake, facts: &[EventEnvelope], entity_pack: &str, genesis: i64) -> Self {
        let bucket = |fact: &EventEnvelope| (fact.at().seconds() - genesis) / (BUCKET_DAYS * DAY);
        let mut tally = Self::default();
        for fact in facts {
            if of(fact, "fishing-started") && matches!(fact.caused_by(), Causation::Action(_)) {
                if let Some(seat) = lake.seat_of(&entity(&payload(fact)["angler"])) {
                    *tally
                        .fished
                        .entry((bucket(fact), seat.to_owned()))
                        .or_default() += 1;
                }
            } else if of(fact, "items-consumed") {
                let p = payload(fact);
                if let Some(seat) = lake.seat_of(&entity(&p["holder"])) {
                    *tally
                        .consumed
                        .entry((bucket(fact), seat.to_owned()))
                        .or_default() += 1;
                }
                let kind = entity(&p["item"]);
                if lake.catch.contains(&kind) {
                    *tally.catch_eaten.entry(bucket(fact)).or_default() += 1;
                }
                if let Some((key, source)) = lake.kinds.get(&kind)
                    && source == entity_pack
                {
                    tally.pack_kind_eaten.get_or_insert_with(|| key.clone());
                }
            }
        }
        tally
    }

    /// Every (seat, bucket) that never fished or never ate, and every bucket nobody ate the catch in.
    fn gaps(&self, lake: &Lake, buckets: i64) -> Vec<String> {
        let mut gaps = Vec::new();
        for index in 0..buckets {
            for seat in lake.seats.keys() {
                for (what, table) in [("fished", &self.fished), ("ate", &self.consumed)] {
                    if table.get(&(index, seat.clone())).copied().unwrap_or(0) == 0 {
                        gaps.push(format!("{seat} never {what} in bucket {}", index + 1));
                    }
                }
            }
            if self.catch_eaten.get(&index).copied().unwrap_or(0) == 0 {
                gaps.push(format!("nobody ate the catch in bucket {}", index + 1));
            }
        }
        gaps
    }
}

fn per_bucket(table: &BTreeMap<(i64, String), u64>) -> Vec<u64> {
    let mut sums: BTreeMap<i64, u64> = BTreeMap::new();
    for ((bucket, _), count) in table {
        *sums.entry(*bucket).or_default() += count;
    }
    sums.into_values().collect()
}

/// PD-47 (3)–(5) over a save's facts, bucket by bucket: the catch located, then every seat fishing and
/// eating in every bucket, the catch eaten in every bucket, and a kind of the Entity Pack eaten.
/// Returns the number of buckets read.
pub fn fishes_and_eats_in_every_bucket(
    lake: &Lake,
    facts: &[EventEnvelope],
    entity_pack: &str,
) -> usize {
    let genesis = facts.first().expect("a genesis").at().seconds();
    let last = facts.last().expect("facts").at().seconds();
    let buckets = (last - genesis) / (BUCKET_DAYS * DAY) + 1;

    let caught = landed_catches(facts);
    assert!(caught > 0, "no catch whose cause reaches an accepted fish");

    let tally = Tally::count(lake, facts, entity_pack, genesis);
    eprintln!(
        "{caught} catches landed; fish requests by bucket {:?}; eaten by bucket {:?}; the catch \
         eaten by bucket {:?}",
        per_bucket(&tally.fished),
        per_bucket(&tally.consumed),
        tally.catch_eaten.values().collect::<Vec<_>>(),
    );
    let gaps = tally.gaps(lake, buckets);
    assert!(gaps.is_empty(), "{}", gaps.join("\n"));
    let eaten = tally
        .pack_kind_eaten
        .unwrap_or_else(|| panic!("no kind of {entity_pack} was eaten"));
    eprintln!("a kind of {entity_pack} eaten: {eaten}");
    usize::try_from(buckets).expect("a count")
}

/// PD-47 (1)–(2): no fault, and every seat moved and talked in every bucket. Returns the buckets.
pub fn alive(lake: &Lake, printed: &str, what: &str) -> usize {
    eprintln!("── {what} ──");
    assert_eq!(
        lines(printed, "faults     0").len(),
        1,
        "{what}: no system fault: {printed}"
    );
    let seats: Vec<&str> = lake.seats.keys().map(String::as_str).collect();
    every_seat_active_in_every_bucket(printed, &seats)
}

/// Starts a 30-day run of Lakeside saved to `save` and SIGKILLs it once it has printed `day`'s line.
pub fn killed_at(save: &Path, day: u64) {
    let world = text(&lakeside());
    let save = text(save);
    let roots = roots();
    let seed = SEED.to_string();
    let mut arguments = vec![
        "run",
        &world,
        "--headless",
        "--seed",
        &seed,
        "--days",
        "30",
        "--save",
        &save,
    ];
    arguments.extend(roots.iter().map(String::as_str));
    let mut child = Command::new(BINARY)
        .args(&arguments)
        .env_remove("MINEWORLD_PACKS")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("the binary starts");
    let reader = BufReader::new(child.stdout.take().expect("piped"));
    let wanted = format!("day {day} ");
    let mut printed = Vec::new();
    for line in reader.lines() {
        let line = line.expect("utf-8");
        let reached = line.starts_with(&wanted);
        printed.push(line);
        if reached {
            break;
        }
    }
    let killed = process::kill(&mut child);
    assert!(
        killed.killed(),
        "killed by SIGKILL, not finished: {killed:?}"
    );
    assert!(
        !printed.iter().any(|line| line.starts_with("history ")),
        "the victim did not finish: {printed:?}"
    );
}
