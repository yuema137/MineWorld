//! **CP-4 — the market lives**, then `AC-11`, `AC-12` and `AC-6` for Market Town, through the command
//! a person types (step-10 §4.6 SD-34, P-5, P-6; `docs/DECISIONS.md` `ARC-38`, `ARC-35`'s 11f note).
//!
//! ```text
//! mineworld run worlds/market-town --headless --seed 7 --days 300 --save A   ┐
//! mineworld run worlds/market-town --headless --seed 7 --days 30  --save C   │ in parallel
//! mineworld run worlds/market-town --headless --seed 7 --days 30  --save D   │
//! mineworld run … --seed 7 --days 30 --save K, SIGKILLed after day 15, then  │
//!   the same command again                                                   │
//! mineworld run worlds/market-town --headless --seed 8 --days 30  --save E   ┘
//! ```
//!
//! **Activity before determinism (I-7).** Nothing is compared until every run that is compared has
//! been shown to be a living market: the 300-day save over its ten buckets, each 30-day save over its
//! one. Two dead markets agree as easily as two living ones (`ARC-23`).
//!
//! What would pass without proving anything, and how each is excluded:
//!
//! ```text
//! a market that drains after a short horizon   the full 300 days, in the default suite (QS-59)
//! money conserved by construction              the wallets economy wrote, read from the newest
//!                                              snapshot, against the replay of the facts up to it
//! a victim that was never killed               signal 9, no summary printed, its head short of C's
//! a restart that read nothing back             the re-run reports resuming at the head on disk
//! a comparison that cannot see a difference    seed 8's save differs from seed 7's, at a located row
//! ```

mod headless;
mod market;

use std::io::{BufRead, BufReader};
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{Command, Stdio};

use headless::{
    BINARY, Scratch, Tables, deterministic, every_seat_active_in_every_bucket, fresh, lines,
};
use market::{MARKET_TOWN, Town, market_lives, run_market, state_matches_replay};
use mineworld_contracts::EventEnvelope;

const SEED: u64 = 7;

/// Starts the 30-day run with `--save save` and SIGKILLs it once it has printed the line for `day`.
fn killed_at(save: &Path, day: u64) {
    let mut child = Command::new(BINARY)
        .args([
            "run",
            MARKET_TOWN,
            "--headless",
            "--seed",
            &SEED.to_string(),
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
    let mut printed = Vec::new();
    for line in reader.lines() {
        let line = line.expect("utf-8");
        let reached = line.starts_with(&wanted);
        printed.push(line);
        if reached {
            break;
        }
    }
    child.kill().expect("SIGKILL is delivered");
    let status = child.wait().expect("reaped");
    assert_eq!(status.signal(), Some(9), "killed by SIGKILL, not finished");
    assert!(
        !printed.iter().any(|line| line.starts_with("history ")),
        "the victim did not finish: {printed:?}"
    );
}

/// Every fact of a save, decoded.
fn facts_of(tables: &Tables) -> Vec<EventEnvelope> {
    tables
        .facts
        .iter()
        .map(|(_, bytes)| {
            mineworld_persistence::format::decode(bytes, "fact").expect("a stored fact decodes")
        })
        .collect()
}

/// A run's printed precondition (no fault, every seat moved and talked in every bucket) and P-5's
/// market conditions, then the stored state against the replay. Returns the facts.
fn living(town: &Town, printed: &str, save: &Path, days: i64, what: &str) -> Vec<EventEnvelope> {
    eprintln!("── {what} ──");
    let seats: Vec<&str> = town.seats.iter().map(String::as_str).collect();
    let buckets = every_seat_active_in_every_bucket(printed, &seats);
    assert_eq!(
        lines(printed, "faults     0").len(),
        1,
        "{what}: no system fault: {printed}"
    );
    let facts = facts_of(&Tables::read(save));
    let (_, read) = market_lives(town, &facts, days);
    assert_eq!(
        read, buckets,
        "{what}: the market and the activity table read the same buckets"
    );
    state_matches_replay(town, save, &facts);
    facts
}

#[test]
fn market_town_lives_three_hundred_days_then_the_same_seed_is_the_same_market() {
    let town = Town::read(Path::new(MARKET_TOWN));
    let long = fresh("market-300");
    let (control, twin, killed, other): (Scratch, Scratch, Scratch, Scratch) = (
        fresh("market-30-control"),
        fresh("market-30-twin"),
        fresh("market-30-killed"),
        fresh("market-30-seed-8"),
    );
    let started = std::time::Instant::now();
    let (long_printed, control_printed, twin_printed, (head_at_kill, resumed), other_printed) =
        std::thread::scope(|scope| {
            let long = scope.spawn(|| run_market(SEED, 300, Some(&long)));
            let control = scope.spawn(|| run_market(SEED, 30, Some(&control)));
            let twin = scope.spawn(|| run_market(SEED, 30, Some(&twin)));
            let killed = scope.spawn(|| {
                killed_at(&killed, 15);
                let head = Tables::read(&killed).journal.len();
                (head, run_market(SEED, 30, Some(&killed)))
            });
            let other = scope.spawn(|| run_market(8, 30, Some(&other)));
            (
                long.join().expect("the 300-day run"),
                control.join().expect("the control"),
                twin.join().expect("the twin"),
                killed.join().expect("the killed run"),
                other.join().expect("seed 8"),
            )
        });
    eprintln!(
        "a 300-day and four 30-day Market Town runs in parallel: {:.1} s wall",
        started.elapsed().as_secs_f64()
    );

    // ── CP-4: activity first (I-7). Nothing is compared before every compared run lives. ─────────
    let long_facts = living(&town, &long_printed, &long, 300, "300 days, seed 7");
    assert_eq!(lines(&long_printed, "day ").len(), 300, "a line per day");
    let history = lines(&long_printed, "history ");
    assert!(
        history.len() == 1
            && history[0].starts_with(&format!("history    {} facts", long_facts.len())),
        "the save holds every fact the run counted: {history:?}"
    );
    let control_facts = living(
        &town,
        &control_printed,
        &control,
        30,
        "30 days, the control",
    );
    living(&town, &twin_printed, &twin, 30, "30 days, the twin");
    living(&town, &resumed, &killed, 30, "30 days, killed and re-run");
    let other_facts = living(&town, &other_printed, &other, 30, "30 days, seed 8");

    // ── Only now: AC-12 and AC-6, byte for byte. ─────────────────────────────────────────────────
    let control_tables = Tables::read(&control);
    control_tables.assert_same_history(&Tables::read(&twin), "two 30-day runs of seed 7");
    assert_eq!(
        deterministic(&control_printed),
        deterministic(&twin_printed),
        "and they print the same"
    );

    let header = resumed.lines().next().expect("a header");
    assert!(
        header.contains("resumed") && header.contains(&format!("at revision {head_at_kill}")),
        "the re-run resumed the save on disk at the head the kill left: {header}"
    );
    assert!(
        head_at_kill < control_tables.journal.len(),
        "killed short of the end ({head_at_kill} of {})",
        control_tables.journal.len()
    );
    Tables::read(&killed).assert_same_history(&control_tables, "killed after day 15 and re-run");

    // The comparison can see a difference: seed 8 differs from seed 7, at a located row.
    let differs_at = control_facts
        .iter()
        .zip(&other_facts)
        .position(|(seven, eight)| seven != eight)
        .expect("seed 8's market differs from seed 7's");
    eprintln!(
        "seed 7 and seed 8 first differ at fact #{}: {} vs {}",
        control_facts[differs_at].id().raw(),
        control_facts[differs_at].event_type().as_str(),
        other_facts[differs_at].event_type().as_str()
    );
}
