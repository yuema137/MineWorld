//! `worlds/bodies-yard` through the real binary (step-11 PB-4, PB-9, PB-10).
//!
//! ```text
//! PB-4   `mineworld validate` refuses, by name, copies whose authored people do not fit their room —
//!        two people 420 mm apart, a centre inside the table, a centre 200 mm from the floor's edge,
//!        a hall too small for twelve — and copies whose `body:` section is malformed; the yard
//!        itself validates
//! PB-9   30 days, seed 7, saved: activity first (every seat moves, someone stops short, someone is
//!        nudged, somebody crosses each way, in every 10-day bucket); then the scan of the save finds
//!        no pair closer than 595 mm, nobody in a wall, no nudge over 310 mm, ≤ 4 per request
//! PB-10  the same world without `bodies`: it runs, everybody moves, and the same scan sees overlaps,
//!        naming the closest pair — the instrument can see what it measures (ARC-23)
//! ```

mod bodies;
mod headless;

use std::path::Path;

use bodies::{
    YARD, assert_active, copy_of, keys_of, run, scan, the_world_file_still_says_the_geometry,
    without_bodies,
};
use headless::{Tables, fresh, mineworld, stderr, stdout};

const SEATS: [&str; 12] = [
    "ada", "ben", "cleo", "dina", "eli", "fay", "gus", "hal", "ivy", "jon", "kai", "lea",
];

/// `validate` of a copy of the yard with `edit` applied to `file` must fail, naming every word.
fn refused(case: &str, file: &str, from: &str, to: &str, names: &[&str]) {
    let copy = copy_of(&fresh(&format!("bodies-yard-refused-{case}")));
    let path = copy.join(file);
    let text = std::fs::read_to_string(&path).expect("reads");
    assert_eq!(
        text.matches(from).count(),
        1,
        "{case}: {file} says {from:?} once"
    );
    std::fs::write(&path, text.replace(from, to)).expect("writes");
    let output = mineworld(&["validate", copy.to_str().expect("a path")]);
    let said = format!("{}{}", stdout(&output), stderr(&output));
    println!("{case}: {}", said.trim());
    assert!(!output.status.success(), "{case}: the copy must be refused");
    for name in names {
        assert!(said.contains(name), "{case}: names {name:?}: {said}");
    }
}

#[test]
fn people_who_do_not_fit_and_malformed_bodies_are_refused_at_load() {
    let output = mineworld(&["validate", YARD]);
    assert!(
        output.status.success(),
        "the yard validates: {}",
        stderr(&output)
    );
    the_world_file_still_says_the_geometry(Path::new(YARD));

    let ben = "    x: 4000\n    y: 2000";
    let cleo = "    x: 8000\n    y: 2000";
    refused(
        "overlap",
        "people/ben.yaml",
        ben,
        "    x: 2420\n    y: 2000",
        &["bodies-overlap", "ada", "ben", "hall", "420 mm"],
    );
    refused(
        "in-solid",
        "people/cleo.yaml",
        cleo,
        "    x: 5100\n    y: 4500",
        &["bodies-in-solid", "cleo", "hall"],
    );
    refused(
        "outside",
        "people/cleo.yaml",
        cleo,
        "    x: 200\n    y: 4500",
        &["bodies-outside", "cleo", "hall", "(200, 4500)"],
    );
    refused(
        "capacity",
        "places/hall.yaml",
        "max: { x: 12000, y: 9000 }",
        "max: { x: 1300, y: 1300 }",
        &[
            "bodies-capacity",
            "hall",
            "4 points",
            "12 people",
            "needs 45",
        ],
    );
    refused(
        "floor",
        "places/hall.yaml",
        "min: { x: 0, y: 0 }",
        "min: { x: 12000, y: 0 }",
        &["hall.yaml", "line", "at least 620 mm"],
    );
    refused(
        "unknown-key",
        "places/hall.yaml",
        "  solids:\n",
        "  wall: { x: 1 }\n  solids:\n",
        &["hall.yaml", "line", "wall"],
    );
    refused(
        "height",
        "places/hall.yaml",
        "height: 750",
        "height: 0",
        &["hall.yaml", "line", "1 to 10000 mm high"],
    );
}

#[test]
fn thirty_days_of_bodies_yard_keep_every_body_apart() {
    the_world_file_still_says_the_geometry(Path::new(YARD));
    let save = fresh("bodies-yard-30");
    let printed = run(Path::new(YARD), 7, 30, Some(&save));
    assert!(printed.contains("faults     0"), "faults 0: {printed}");
    let report = scan(&Tables::read(&save), &keys_of(Path::new(YARD)));
    assert_active(&report, &SEATS);
    println!(
        "{} requests scanned; closest: {}",
        report.requests,
        report.located()
    );
    assert!(
        report.violations.is_empty(),
        "{} violations, the first: {:#?}",
        report.violations.len(),
        &report.violations[..report.violations.len().min(5)]
    );
    assert!(report.closest_mm() >= 595, "{}", report.located());
}

#[test]
fn without_bodies_the_same_world_runs_and_the_scan_sees_people_overlap() {
    let copy = without_bodies(&fresh("bodies-yard-without-bodies"));
    let save = fresh("bodies-yard-without-bodies-save");
    let printed = run(&copy, 7, 30, Some(&save));
    assert!(printed.contains("faults     0"), "faults 0: {printed}");
    let report = scan(&Tables::read(&save), &keys_of(&copy));
    for (index, bucket) in report.buckets.iter().enumerate() {
        for seat in SEATS {
            assert!(
                bucket.moves.get(seat).copied().unwrap_or(0) > 0,
                "{seat} moved in bucket {index}"
            );
        }
        assert_eq!(
            (bucket.stopped, bucket.displaced),
            (0, 0),
            "nobody resolves anybody"
        );
    }
    println!(
        "without bodies: {} violations; closest: {}",
        report.violations.len(),
        report.located()
    );
    assert!(
        report.closest_mm() < 595,
        "the scan sees an overlap: {}",
        report.located()
    );
}
