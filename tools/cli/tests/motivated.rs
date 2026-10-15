//! The `motivated` model in a real world (`pr-dl-b-motivated.md` §8, AC-DLb-2 and AC-DLb-7; C5).
//!
//! ```text
//! AC-DLb-2  alice and bob on motivated, the rest on rule: two in-memory 30-day runs print the same history;
//!           a 15-day save resumed to 30 equals an uninterrupted 30-day save, facts, journal and snapshots
//!           byte for byte
//! AC-DLb-7  every seat on motivated, 30 days: no request the world refused as unavailable, and no fault
//! ```

mod headless;

use std::path::Path;

use headless::{PACK, Tables, deterministic, fresh, lines, mineworld, stderr, stdout};

const SEED: &str = "7";

/// The opt-in decision list that puts every seat on `motivated`
/// (`cognition/motivated/examples/social-cafe-all-motivated.yaml`).
const ALL_MOTIVATED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../cognition/motivated/examples/social-cafe-all-motivated.yaml"
);

/// `run` for `days` with alice and bob on `motivated`, kept in `save` when given; its stdout.
fn run_mixed(days: u64, save: Option<&Path>) -> String {
    let days = days.to_string();
    let mut arguments = vec![
        "run",
        PACK,
        "--headless",
        "--seed",
        SEED,
        "--days",
        &days,
        "--seat-model",
        "alice=motivated",
        "--seat-model",
        "bob=motivated",
    ];
    let save = save.map(|save| save.to_str().expect("a printable path").to_owned());
    if let Some(save) = &save {
        arguments.extend(["--save", save.as_str()]);
    }
    let output = mineworld(&arguments);
    assert!(output.status.success(), "{}", stderr(&output));
    stdout(&output)
}

#[test]
fn mixed_seats_reproduce_their_history_in_memory_and_across_a_resume() {
    let first_text = run_mixed(30, None);
    let second_text = run_mixed(30, None);
    assert_eq!(
        deterministic(&first_text),
        deterministic(&second_text),
        "two in-memory 30-day runs print the same history"
    );
    assert!(
        first_text.contains("alice") && first_text.contains("motivated"),
        "the header names the mixed seats: {first_text}"
    );

    let uninterrupted = fresh("motivated-mixed-30");
    run_mixed(30, Some(&uninterrupted));
    let resumed = fresh("motivated-mixed-resumed");
    run_mixed(15, Some(&resumed));
    let second_half = run_mixed(30, Some(&resumed));
    assert!(
        second_half.contains("resumed "),
        "the second run resumes the save: {second_half}"
    );
    Tables::read(&uninterrupted).assert_same_history(
        &Tables::read(&resumed),
        "mixed motivated: a 15-day save resumed to 30 days",
    );
}

#[test]
fn an_all_motivated_run_submits_nothing_the_world_does_not_offer() {
    let output = mineworld(&[
        "run",
        PACK,
        "--headless",
        "--seed",
        SEED,
        "--days",
        "30",
        "--decisions",
        ALL_MOTIVATED,
    ]);
    assert!(output.status.success(), "{}", stderr(&output));
    let printed = stdout(&output);
    let refused_as_unavailable: u64 = lines(&printed, "requests ")
        .iter()
        .filter(|line| line.split_whitespace().any(|word| word == "unavailable"))
        .map(|line| {
            line.split_whitespace()
                .last()
                .and_then(|number| number.parse::<u64>().ok())
                .unwrap_or_else(|| panic!("no count at the end of: {line}"))
        })
        .sum();
    assert_eq!(
        refused_as_unavailable, 0,
        "no request was refused as unavailable: {printed}"
    );
    assert_eq!(
        lines(&printed, "faults     0").len(),
        1,
        "the run has no faults: {printed}"
    );
}
