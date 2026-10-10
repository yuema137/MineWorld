//! Criterion 5 (step-19 §17.6, C4 (d)): weather moves nobody. Given two saves of the same world, one
//! without weather (TW-a's baseline) and one with it, every fact that weather does not own is the same
//! in both — instant, type, visibility, subjects and payload bytes, in order. Ids are excluded: they are
//! one world counter, and weather's facts take some of them.
//!
//! The saves are 300-day town runs, about 2.7 GB each (F-TWbd-1), far too large for the default
//! suite. So this test is opt-in: it is ignored unless run by name with both saves given:
//!
//! ```text
//! TWB_SAVE_WITHOUT=<dir> TWB_SAVE_WITH=<dir> \
//!   cargo test -p mineworld-weather --test moves_nobody -- --ignored --nocapture
//! ```
//!
//! It reads the saves' fact logs through the persistence backend and writes nothing.

use std::collections::BTreeMap;
use std::path::PathBuf;

use mineworld_contracts::EventEnvelope;
use mineworld_persistence::{Durability, PersistenceBackend, SqliteBackend};

/// The event types the weather pack owns.
const WEATHERS: [&str; 3] = ["weather-configured", "weather-day", "weather-changed"];

/// Every fact of the save in `variable`, oldest first.
fn facts(variable: &str) -> Vec<EventEnvelope> {
    let directory = PathBuf::from(std::env::var(variable).unwrap_or_else(|_| panic!("{variable}")));
    let backend = SqliteBackend::open(&directory, Durability::ProcessCrash).expect("opens");
    backend
        .last_facts(usize::MAX >> 2)
        .expect("reads")
        .into_iter()
        .map(|row| serde_json::from_slice(&row.bytes).expect("a fact"))
        .collect()
}

/// A fact without its id or cause (which names ids): instant, type, visibility, subjects, payload.
fn projection(fact: &EventEnvelope) -> (i64, String, String, String, Vec<u8>) {
    (
        fact.at().seconds(),
        fact.event_type().to_string(),
        format!("{:?}", fact.visibility()),
        format!("{:?}", fact.subjects()),
        fact.payload().payload().clone(),
    )
}

fn counts(facts: &[EventEnvelope]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for fact in facts {
        *counts.entry(fact.event_type().to_string()).or_insert(0) += 1;
    }
    counts
}

/// Every leaf where `a` and `b` differ, with its path.
fn leaves(
    path: &str,
    a: &serde_json::Value,
    b: &serde_json::Value,
    out: &mut Vec<(String, String, String)>,
) {
    use serde_json::Value;
    match (a, b) {
        (Value::Object(x), Value::Object(y)) if x.len() == y.len() => {
            for (key, value) in x {
                match y.get(key) {
                    Some(other) => leaves(&format!("{path}.{key}"), value, other, out),
                    None => out.push((format!("{path}.{key}"), value.to_string(), "∅".to_owned())),
                }
            }
        }
        (Value::Array(x), Value::Array(y)) if x.len() == y.len() => {
            for (index, (value, other)) in x.iter().zip(y).enumerate() {
                leaves(&format!("{path}[{index}]"), value, other, out);
            }
        }
        _ if a != b => out.push((path.to_owned(), a.to_string(), b.to_string())),
        _ => {}
    }
}

/// Diagnostic for a failure of the test below: classifies every differing non-weather payload, leaf
/// by leaf, by the key it differs at and by whether the difference is a numeric id shifted by one.
#[test]
#[ignore = "opt-in diagnostic: needs two 300-day saves (TWB_SAVE_WITHOUT, TWB_SAVE_WITH)"]
fn where_the_non_weather_facts_differ() {
    let without = facts("TWB_SAVE_WITHOUT");
    let with: Vec<EventEnvelope> = facts("TWB_SAVE_WITH")
        .into_iter()
        .filter(|fact| !WEATHERS.contains(&fact.event_type().as_str()))
        .collect();
    assert_eq!(without.len(), with.len());
    let mut by_key: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut differing_facts = 0;
    for (a, b) in without.iter().zip(&with) {
        let (pa, pb) = (projection(a), projection(b));
        assert_eq!(
            (pa.0, &pa.1, &pa.2, &pa.3),
            (pb.0, &pb.1, &pb.2, &pb.3),
            "instant, type, visibility, subjects"
        );
        if pa.4 == pb.4 {
            continue;
        }
        differing_facts += 1;
        let va: serde_json::Value = serde_json::from_slice(&pa.4).expect("json");
        let vb: serde_json::Value = serde_json::from_slice(&pb.4).expect("json");
        let mut out = Vec::new();
        leaves(&pa.1, &va, &vb, &mut out);
        for (path, x, y) in out {
            let number = |s: &str| s.trim_matches('"').parse::<u64>().ok();
            let shifted = matches!((number(&x), number(&y)), (Some(x), Some(y)) if y == x + 1);
            let entry = by_key.entry(path).or_insert((0, 0));
            entry.0 += 1;
            if shifted {
                entry.1 += 1;
            }
        }
    }
    println!(
        "{differing_facts} of {} non-weather facts differ in payload",
        without.len()
    );
    for (path, (all, shifted)) in &by_key {
        println!("  {path}: {all} differences, {shifted} of them an id shifted by +1");
    }
}

#[test]
#[ignore = "opt-in: needs two 300-day saves (TWB_SAVE_WITHOUT, TWB_SAVE_WITH)"]
fn weather_moves_nobody() {
    let without = facts("TWB_SAVE_WITHOUT");
    let with = facts("TWB_SAVE_WITH");
    let (theirs, ours): (Vec<&EventEnvelope>, Vec<&EventEnvelope>) = with
        .iter()
        .partition(|fact| !WEATHERS.contains(&fact.event_type().as_str()));
    println!(
        "without weather: {} facts; with weather: {} facts, of which {} are weather's {:?}",
        without.len(),
        with.len(),
        ours.len(),
        counts(&ours.iter().map(|fact| (*fact).clone()).collect::<Vec<_>>())
    );
    assert!(
        without
            .iter()
            .all(|fact| !WEATHERS.contains(&fact.event_type().as_str())),
        "the baseline has no weather"
    );
    let first_divergence = without
        .iter()
        .zip(&theirs)
        .position(|(a, b)| projection(a) != projection(b));
    if let Some(at) = first_divergence {
        panic!(
            "the first diverging non-weather fact is #{at}: without {:?} / with {:?}",
            projection(&without[at]),
            projection(theirs[at])
        );
    }
    assert_eq!(
        without.len(),
        theirs.len(),
        "the same number of non-weather facts"
    );
    let theirs_owned: Vec<EventEnvelope> = theirs.into_iter().cloned().collect();
    assert_eq!(
        counts(&without),
        counts(&theirs_owned),
        "and every type's count"
    );
    println!(
        "PASS: {} non-weather facts equal in instant, type, visibility, subjects and payload",
        without.len()
    );
}
