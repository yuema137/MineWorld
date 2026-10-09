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
    YARD, assert_active, assert_aggregate, assert_ao1_ao3, assert_day_one, assert_reachable,
    copy_of, keys_of, print_objects_activity, run, scan, the_world_file_still_says_the_geometry,
    without_bodies,
};
use headless::{Tables, fresh, mineworld, stderr, stdout};

const SEATS: [&str; 12] = [
    "ada", "ben", "cleo", "dina", "eli", "fay", "gus", "hal", "ivy", "jon", "kai", "lea",
];

/// `validate` of a copy of the yard with `edit` applied to `file` must fail, naming every word.
fn refused(case: &str, file: &str, from: &str, to: &str, names: &[&str]) {
    let copy = copy_of(fresh(&format!("bodies-yard-refused-{case}")));
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

/// `validate` of a copy of the yard after `edit` must fail, naming every word; returns what it said.
fn refused_after(case: &str, edit: impl FnOnce(&Path), names: &[&str]) -> String {
    // Its own directory names: the 12b test above refuses copies with some of the same case names,
    // and the two tests run at once.
    let copy = copy_of(fresh(&format!("bodies-yard-object-refused-{case}")));
    edit(&copy);
    let output = mineworld(&["validate", copy.to_str().expect("a path")]);
    let said = format!("{}{}", stdout(&output), stderr(&output));
    println!("{case}: {}", said.trim());
    assert!(!output.status.success(), "{case}: the copy must be refused");
    for name in names {
        assert!(said.contains(name), "{case}: names {name:?}: {said}");
    }
    said
}

/// Replaces `from` (said exactly once) with `to` in `file` of `copy`.
fn replace_once(copy: &Path, file: &str, from: &str, to: &str) {
    let path = copy.join(file);
    let text = std::fs::read_to_string(&path).expect("reads");
    assert_eq!(text.matches(from).count(), 1, "{file} says {from:?} once");
    std::fs::write(&path, text.replace(from, to)).expect("writes");
}

/// PO-2 (binary half) and PO-16 (step-11 §18.4): objects that do not fit, malformed object bodies, a
/// form on the wrong kind of file, and QB-3's two refusals, each through the real `validate` on a
/// copy of the yard. The yard itself validates (the first test).
#[test]
fn objects_that_do_not_fit_and_held_objects_are_refused_at_load() {
    let ball = "  at: { place: hall, x: 5000, y: 2500 }";
    refused_after(
        "unshaped",
        |copy| {
            let file = copy.join("places/court.yaml");
            let (kept, found) =
                bodies::strip_section(&std::fs::read_to_string(&file).expect("reads"), "body");
            assert!(found, "the court has a body");
            std::fs::write(&file, kept).expect("writes");
        },
        &["bodies-unshaped", "court"],
    );
    // 200 mm east of ben (4 000, 2 000).
    refused_after(
        "on-person",
        |copy| {
            replace_once(
                copy,
                "items/hall-ball.yaml",
                ball,
                "  at: { place: hall, x: 4200, y: 2000 }",
            )
        },
        &["bodies-object-on-person", "hall-ball", "ben"],
    );
    // A box half 200 at (4 900, 4 500): its footprint reaches x = 5 100, into the table (5 000).
    refused_after(
        "in-solid",
        |copy| {
            replace_once(
                copy,
                "items/hall-box-west.yaml",
                "x: 3000, y: 4500",
                "x: 4900, y: 4500",
            );
        },
        &["bodies-object-in-solid", "hall-box-west", "hall"],
    );
    // 50 mm from the west wall: the ball's footprint reaches x = −60.
    refused_after(
        "outside",
        |copy| {
            replace_once(
                copy,
                "items/hall-ball.yaml",
                ball,
                "  at: { place: hall, x: 50, y: 2500 }",
            )
        },
        &["bodies-object-outside", "hall-ball", "(50, 2500)"],
    );
    // 100 mm from hall-ball-2 at (3 500, 6 200).
    refused_after(
        "overlap",
        |copy| {
            replace_once(
                copy,
                "items/hall-ball.yaml",
                ball,
                "  at: { place: hall, x: 3600, y: 6200 }",
            )
        },
        &["bodies-object-overlap", "hall-ball", "hall-ball-2"],
    );
    // 24 more boxes half 400 in two rows along the hall's south and north walls: each can cover 9
    // capacity points, so 45 for the people + 8 × 4 for the hall's own objects + 24 × 9 = 293 points
    // would be needed, against the hall's 226.
    refused_after(
        "capacity",
        |copy| {
            let mut keys = Vec::new();
            for i in 0..24 {
                let (x, y) = if i < 14 {
                    (500 + 850 * i, 500)
                } else {
                    (500 + 850 * (i - 14), 8_500)
                };
                let key = format!("cap-{i:02}");
                std::fs::write(
                    copy.join("items").join(format!("{key}.yaml")),
                    format!(
                        "body:\n  shape: {{ box: {{ x: 400, y: 400, z: 400 }} }}\n  at: {{ place: hall, x: {x}, y: {y} }}\n"
                    ),
                )
                .expect("writes");
                keys.push(format!("  - {key}\n"));
            }
            replace_once(
                copy,
                "world.yaml",
                "items:\n",
                &format!("items:\n{}", keys.concat()),
            );
        },
        &["bodies-capacity", "hall", "226 points", "12 people"],
    );
    // The loader refuses malformed object bodies at their line, with bodies' own message.
    refused_after(
        "ball-zero",
        |copy| replace_once(copy, "items/hall-ball.yaml", "{ ball: 110 }", "{ ball: 0 }"),
        &["hall-ball.yaml", "line", "a ball's radius is 50 to 400 mm"],
    );
    refused_after(
        "box-wide",
        |copy| {
            replace_once(
                copy,
                "items/hall-box-west.yaml",
                "{ box: { x: 200, y: 200, z: 200 } }",
                "{ box: { x: 500, y: 100, z: 100 } }",
            );
        },
        &[
            "hall-box-west.yaml",
            "line",
            "a box's half-extents are 50 to 400 mm",
        ],
    );
    refused_after(
        "unknown-key",
        |copy| {
            replace_once(
                copy,
                "items/hall-ball.yaml",
                "  shape:",
                "  spin: 3\n  shape:",
            )
        },
        &["hall-ball.yaml", "line", "spin"],
    );
    refused_after(
        "both-forms",
        |copy| {
            replace_once(
                copy,
                "items/hall-ball.yaml",
                "  shape:",
                "  floor: { min: { x: 0, y: 0 }, max: { x: 1000, y: 1000 } }\n  shape:",
            );
        },
        &["hall-ball.yaml", "line", "never both"],
    );
    // A form on the other kind of file.
    refused_after(
        "object-form-in-a-place",
        |copy| {
            let file = copy.join("places/court.yaml");
            let (kept, _) =
                bodies::strip_section(&std::fs::read_to_string(&file).expect("reads"), "body");
            std::fs::write(
                &file,
                kept + "body:\n  shape: { ball: 110 }\n  at: { place: court, x: 1000, y: 1000 }\n",
            )
            .expect("writes");
        },
        &["bodies-section-kind", "court"],
    );
    refused_after(
        "place-form-in-an-item",
        |copy| {
            let file = copy.join("items/hall-ball.yaml");
            let (kept, _) =
                bodies::strip_section(&std::fs::read_to_string(&file).expect("reads"), "body");
            std::fs::write(
                &file,
                kept + "body:\n  floor: { min: { x: 0, y: 0 }, max: { x: 1000, y: 1000 } }\n",
            )
            .expect("writes");
        },
        &["bodies-section-kind", "hall-ball"],
    );
    // PO-16, QB-3 end to end: an object that is also a declared kind (the item pack installed) …
    refused_after(
        "held-kind",
        |copy| {
            replace_once(
                copy,
                "world.yaml",
                "  - movement\n",
                "  - movement\n  - item\n",
            );
            let file = copy.join("items/hall-ball.yaml");
            let text = std::fs::read_to_string(&file).expect("reads");
            std::fs::write(&file, text + "item: { category: toy, name: Ball }\n").expect("writes");
        },
        &["bodies-held-kind", "hall-ball"],
    );
    // … and an object somebody holds at genesis (item and inventory installed, no `item:` on the
    // ball): inventory's own declared-kind rule refuses it.
    refused_after(
        "held-at-genesis",
        |copy| {
            replace_once(
                copy,
                "world.yaml",
                "  - movement\n",
                "  - movement\n  - item\n  - inventory\n",
            );
            let file = copy.join("people/ada.yaml");
            let text = std::fs::read_to_string(&file).expect("reads");
            std::fs::write(&file, text + "holdings: { hall-ball: 1 }\n").expect("writes");
        },
        // Inventory's own refusal, as inventory words it (its message names no holder).
        &["inventory", "stocked", "PreconditionFailed"],
    );
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
    check(7, &scanned(7, "bodies-yard-30", None));
}

/// The AO-2 ruling (step-11 §18.11): seeds 7, 8 and 9, each checked as seed 7 is, and AO-2′ (a) —
/// every action in every bucket summed over the three — on the aggregate. Evidence, not part of the
/// gate: `cargo test -p mineworld-cli --test bodies_yard -- --ignored --nocapture`. With
/// `BODIES_YARD_SAVES=<directory>` it scans `<directory>/bodies-yard-30-seed-<n>` instead of running.
#[test]
#[ignore = "evidence for the AO-2 ruling (three seeds); run with --ignored"]
fn thirty_days_of_bodies_yard_at_seeds_7_8_and_9() {
    let saved = std::env::var_os("BODIES_YARD_SAVES").map(std::path::PathBuf::from);
    let reports: Vec<_> = [7, 8, 9]
        .into_iter()
        .map(|seed| {
            let name = format!("bodies-yard-30-seed-{seed}");
            (seed, scanned(seed, &name, saved.as_deref()))
        })
        .collect();
    // Every seed is checked, and every failure reported, before the test fails.
    let failed: Vec<u64> = reports
        .iter()
        .filter(|(seed, report)| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| check(*seed, report))).is_err()
        })
        .map(|(seed, _)| *seed)
        .collect();
    let reports: Vec<bodies::Report> = reports.into_iter().map(|(_, report)| report).collect();
    assert_aggregate(&reports);
    assert!(
        failed.is_empty(),
        "seeds failing their own checks: {failed:?}"
    );
}

/// PO-9's run at `seed`: 30 days, saved as `name` — or the save `name` already in `saved` — and its
/// scan. Each test saves under its own name: two tests running at once must never share a save.
fn scanned(seed: u64, name: &str, saved: Option<&Path>) -> bodies::Report {
    the_world_file_still_says_the_geometry(Path::new(YARD));
    // The operator's saves (`BODIES_YARD_SAVES`) are only read; a run's own save is scratch, held
    // until the scan has read it.
    let scratch;
    let save = match saved {
        Some(directory) => directory.join(name),
        None => {
            scratch = fresh(name);
            let printed = run(Path::new(YARD), seed, 30, Some(&scratch));
            assert!(printed.contains("faults     0"), "faults 0: {printed}");
            scratch.to_path_buf()
        }
    };
    scan(&Tables::read(&save), &keys_of(Path::new(YARD)))
}

/// PO-9's checks on one seed's report: activity first (AO-1, AO-3, AO-2′ (c) and (b′); ARC-23, step-11
/// PO-9 as the AO-2 ruling restates it), 12b's precondition, then the scan.
fn check(seed: u64, report: &bodies::Report) {
    println!("seed {seed}:");
    print_objects_activity(report);
    assert_ao1_ao3(report, &SEATS);
    assert_day_one(report);
    assert_reachable(report);
    assert_active(report, &SEATS);
    println!(
        "seed {seed}: {} requests scanned; {} violations; closest: {}; closest to an object: {}",
        report.requests,
        report.violations.len(),
        report.located(),
        report.located_object()
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
    let copy = without_bodies(fresh("bodies-yard-without-bodies"));
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
        // PO-10: nothing offers a kick, a throw or a shove, so none happens, and nothing is pushed.
        assert_eq!(
            (bucket.kicks, bucket.throws, bucket.shoves, bucket.pushes),
            (0, 0, 0, 0),
            "no kick, throw, shove or push without bodies, bucket {index}"
        );
    }
    for word in [
        "kick",
        "throw",
        "shove",
        "object-moved",
        "person-shoved",
        "object-placed",
    ] {
        assert!(
            !printed.contains(word),
            "no {word} without bodies: {printed}"
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
