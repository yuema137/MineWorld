//! The template determinism test (`pr-dl-a-decision-list.md` C6, AC-DLa-6, R-S24-8): **every** in-process
//! model this build installs is held to `AC-6`, with no edit here when a model is installed.
//!
//! ```text
//! for each in-process model M of cognition/installed, every seat of Social Café on M:
//!     run --days 10 --save A          uninterrupted
//!     run --days 10 --save B          again, a new process        A and B equal, byte for byte
//!     run --days 5  --save C          then --days 10 --save C     C and A equal, byte for byte
//! ```
//!
//! A model that keeps state across consults — a counter behind `&self` that clippy did not see — decides
//! differently after the resume, whose process starts it afresh, and the third comparison sees it.
//! Compared are the saves' facts, journal and snapshots as stored, never the printed fingerprint.

mod headless;

use std::path::Path;

use headless::{PACK, Tables, fresh, mineworld, seats, stderr, stdout};
use mineworld_installed_models::{AVAILABLE, Catalog};

const SEED: &str = "7";

/// `run` with every seat on `model`, to the age `days`, kept in `save`; its stdout.
fn run_on(model: &str, days: u64, save: &Path) -> String {
    let flags: Vec<String> = seats()
        .iter()
        .map(|seat| format!("{seat}={model}"))
        .collect();
    let days = days.to_string();
    let save = save.to_str().expect("a printable path");
    let mut arguments = vec![
        "run",
        PACK,
        "--headless",
        "--seed",
        SEED,
        "--days",
        &days,
        "--save",
        save,
    ];
    for flag in &flags {
        arguments.extend(["--seat-model", flag.as_str()]);
    }
    let output = mineworld(&arguments);
    assert!(output.status.success(), "{model}: {}", stderr(&output));
    stdout(&output)
}

#[test]
fn every_installed_model_reproduces_its_history_twice_and_across_a_resume() {
    let models: Vec<&str> = AVAILABLE
        .iter()
        .filter(|model| !model.is_external())
        .map(|model| model.id())
        .collect();
    assert!(models.contains(&"rule"), "rule is installed: {models:?}");
    for model in models {
        let started = std::time::Instant::now();
        let (a, b, c) = (
            fresh(&format!("models-{model}-a")),
            fresh(&format!("models-{model}-b")),
            fresh(&format!("models-{model}-c")),
        );
        let printed = run_on(model, 10, &a);
        let seat_count = seats().len();
        assert!(
            printed
                .lines()
                .next()
                .is_some_and(|header| header.contains(&format!("({seat_count} {model})"))),
            "{model}: every seat runs it: {printed}"
        );
        run_on(model, 10, &b);
        let halfway = run_on(model, 5, &c);
        assert!(
            halfway.contains("created "),
            "{model}: the first half creates the save"
        );
        let resumed = run_on(model, 10, &c);
        assert!(
            resumed.contains("resumed "),
            "{model}: the second half resumes it: {resumed}"
        );

        let control = Tables::read(&a);
        assert!(
            control.facts.len() > 1_000,
            "{model}: the run did something"
        );
        control.assert_same_history(&Tables::read(&b), &format!("{model}: a second run"));
        control.assert_same_history(
            &Tables::read(&c),
            &format!("{model}: killed at day 5 and resumed"),
        );
        println!(
            "{model}: {} facts, {:.1} s",
            control.facts.len(),
            started.elapsed().as_secs_f64()
        );
    }
}
