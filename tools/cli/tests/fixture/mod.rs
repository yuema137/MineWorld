//! The fixture assumption a hosted-world test relies on, checked rather than trusted
//! (`step-09-social.md` §4.3.7, `DECISIONS.md` `ARC-23`, `ARC-32`).
//!
//! A hosted world's clock runs one world second per wall second. Since 10c, a routine boundary that
//! fell while a test was running would wake a routine process and commit a revision — so every test
//! that claims "the revision is unchanged", or that a world's head is settled, depends on no boundary
//! falling in the stretch of the day it runs through. That is a property of `worlds/social-cafe`'s
//! **content**, not of any code, and a test must not pass because of content it never checks.
//!
//! So such a test calls [`assert_quiet`] first. It reads every `from: "HH:MM"` of every person's
//! `routine:` with a literal reader of its own — never schedule's code — and fails naming the
//! assumption if a boundary falls inside the window.

#![allow(dead_code)]

use std::path::Path;

/// The pack's people, whose files carry the routines.
const PEOPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../worlds/social-cafe/people"
);

/// Seconds in a day.
pub const DAY: i64 = 86_400;

/// Every routine boundary in the pack, as (person, seconds after midnight).
pub fn routine_boundaries() -> Vec<(String, i64)> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(Path::new(PEOPLE)).expect("people/ lists") {
        let path = entry.expect("an entry").path();
        let person = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        let text = std::fs::read_to_string(&path).expect("a person's file reads");
        for line in text.lines() {
            let Some(at) = line.find("from: \"") else {
                continue;
            };
            let clock = &line[at + 7..at + 12];
            let (hours, minutes) = clock.split_once(':').expect("HH:MM");
            let hours: i64 = hours.parse().expect("hours");
            let minutes: i64 = minutes.parse().expect("minutes");
            found.push((person.clone(), hours * 3_600 + minutes * 60));
        }
    }
    found
}

/// Fails, naming the fixture assumption, if any routine boundary falls in the `length` seconds of the
/// day that begin at `from` (seconds after midnight; the window may wrap past midnight).
///
/// Also fails if the pack has no routine boundary at all: a check that found nothing to check would
/// pass vacuously, which is the instrument failure `ARC-23` names.
pub fn assert_quiet(from: i64, length: i64, test: &str) {
    let boundaries = routine_boundaries();
    assert!(
        !boundaries.is_empty(),
        "fixture assumption unchecked: no routine boundary was found in {PEOPLE}, so the window \
         check would prove nothing"
    );
    let from = from.rem_euclid(DAY);
    let inside: Vec<String> = boundaries
        .iter()
        .filter(|(_, at)| (at - from).rem_euclid(DAY) < length)
        .map(|(person, at)| format!("{person} at {:02}:{:02}", at / 3_600, at % 3_600 / 60))
        .collect();
    assert!(
        inside.is_empty(),
        "fixture assumption violated: a routine boundary falls in the restart window that {test} \
         relies on ({:02}:{:02} for {} s). A boundary there wakes a routine process and commits a \
         revision while the test runs. Move it out of the window (worlds/social-cafe/people/alice.yaml \
         explains the window) or change the test. Inside the window: {}",
        from / 3_600,
        from % 3_600 / 60,
        length,
        inside.join(", ")
    );
}
