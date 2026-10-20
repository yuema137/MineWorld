//! AC-FAD-1..3 — **a group activity lasts about an hour again** (`pr-fix-activity-double-draw.md` §4;
//! `DECISIONS.md` `ARC-151`).
//!
//! ```text
//! mineworld run <world> --headless --seed s --days 10 --save S, for the three reference worlds that
//! compose group-activity (social-cafe, market-town, lakeside) and seeds 7 and 11; S read back:
//!
//! length     each activity's group-activity-ended instant minus its group-activity-started instant,
//!            paired by the activity's process; an activity still running at the end is not counted
//! AC-FAD-1   the median over every ended activity is at least 55 minutes (ACTIVITY_LENGTH is 60)
//! AC-FAD-2   at most 5 % end under one minute
//! AC-FAD-3   every run starts at least one activity per simulated day
//! ```
//!
//! The cause these guard against is the paced rule drawing leave and join twice — once by name in its
//! social band, once again through the generic offered band (F-S27b-5). Measured on these runs
//! (pr-fix-activity-double-draw §10): with the double draw, median 44 minutes (30 in social-cafe seed 7),
//! mean 37, 7.5 % under a minute; with the fix — and identically on the commit before S27-b — median 60,
//! mean 43, 3.1 % under a minute. The mean stays below an hour because members still leave by name on 10
//! of 100 consults, as they did before S27-b, so the median is the measure (D-FAD-1). Mutation M-FAD-1
//! (pass no by-name list to `offered::attempt`) turns this red.

mod headless;
mod social;

use std::collections::BTreeMap;

use headless::{Tables, fresh, mineworld, stderr, stdout};
use mineworld_contracts::ProcessId;
use mineworld_group_activity::{GroupActivityEnded, GroupActivityStarted};
use social::decoded;

const WORLDS: [&str; 3] = ["social-cafe", "market-town", "lakeside"];
const SEEDS: [u64; 2] = [7, 11];
const DAYS: u64 = 10;
/// AC-FAD-1, in seconds: most activities run their hour (`ACTIVITY_LENGTH`), as before S27-b.
const MEDIAN_AT_LEAST: i64 = 55 * 60;
/// AC-FAD-2: out of 100, at most this share ends within a minute (10 % was the defect).
const UNDER_A_MINUTE_AT_MOST: usize = 5;

/// The lengths in seconds of every activity that started and ended in this run, and how many started.
fn lengths(world: &str, seed: u64) -> (Vec<i64>, usize) {
    let save = fresh(&format!("activity-length-{world}-{seed}"));
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let pack = format!("{root}/worlds/{world}");
    // The pack roots the long profile passes (`scripts/ci_parity.py` PACK_ROOTS): lakeside requires them.
    let (entities, presentation) = (
        format!("{root}/entities"),
        format!("{root}/presentation/mineworld-default"),
    );
    let (seed, days) = (seed.to_string(), DAYS.to_string());
    let path = save.to_str().expect("a printable path");
    let output = mineworld(&[
        "run",
        &pack,
        "--headless",
        "--seed",
        &seed,
        "--days",
        &days,
        "--save",
        path,
        "--packs",
        &entities,
        "--packs",
        &presentation,
    ]);
    assert!(
        output.status.success(),
        "run {world} failed: {}\n{}",
        stderr(&output),
        stdout(&output)
    );
    let mut started: BTreeMap<ProcessId, i64> = BTreeMap::new();
    let mut ended = Vec::new();
    for fact in social::facts_of(&Tables::read(&save)) {
        if let Some(start) = decoded::<GroupActivityStarted>(&fact) {
            started.insert(start.activity(), fact.at().seconds());
        } else if let Some(end) = decoded::<GroupActivityEnded>(&fact) {
            let began = started[&end.activity()];
            ended.push(fact.at().seconds() - began);
        }
    }
    (ended, started.len())
}

#[test]
fn group_activities_last_about_an_hour() {
    let mut all = Vec::new();
    for world in WORLDS {
        for seed in SEEDS {
            let (mut lengths, started) = lengths(world, seed);
            lengths.sort_unstable();
            let n = lengths.len().max(1);
            let mean = lengths.iter().sum::<i64>() / i64::try_from(n).expect("fits");
            let short = lengths.iter().filter(|length| **length < 60).count();
            eprintln!(
                "{world} seed {seed}: {started} started, {} ended; mean {} min, median {} min, \
                 under a minute {short} ({} %)",
                lengths.len(),
                mean / 60,
                lengths.get(lengths.len() / 2).copied().unwrap_or(0) / 60,
                short * 100 / n
            );
            assert!(
                started >= usize::try_from(DAYS).expect("fits"),
                "AC-FAD-3: {world} seed {seed} started {started} activities in {DAYS} days"
            );
            all.extend(lengths);
        }
    }
    all.sort_unstable();
    let n = all.len();
    let mean = all.iter().sum::<i64>() / i64::try_from(n).expect("fits");
    let short = all.iter().filter(|length| **length < 60).count();
    eprintln!(
        "all: {n} ended; mean {} min {} s, median {} min, under a minute {short} ({}.{} %)",
        mean / 60,
        mean % 60,
        all[n / 2] / 60,
        short * 100 / n,
        short * 1000 / n % 10
    );
    assert!(
        all[n / 2] >= MEDIAN_AT_LEAST,
        "AC-FAD-1: the median activity lasts {} s, under {MEDIAN_AT_LEAST} s",
        all[n / 2]
    );
    assert!(
        short * 100 <= UNDER_A_MINUTE_AT_MOST * n,
        "AC-FAD-2: {short} of {n} activities end within a minute"
    );
}
