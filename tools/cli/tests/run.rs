//! `AC-11` and `AC-12` through the command a person types (step-08 C4, `ARC-27`).
//!
//! ```text
//! mineworld run worlds/social-cafe --headless --seed 7 --days 300 --save A     ┐ in parallel
//! mineworld run worlds/social-cafe --headless --seed 7 --days 300 --save B     │
//! mineworld run worlds/social-cafe --headless --seed 7 --days 300              ┘ in memory
//! ```
//!
//! What would make this pass without proving anything, and how each is excluded (`ARC-23`):
//!
//! ```text
//! two idle histories agreeing      before anything is compared, every seat must have an accepted
//!                                  move AND talk in every one of the ten 30-day buckets (I-9), the
//!                                  street must have been entered, and the fact count is located
//! a seed that is never read        seed 8 makes a different fact log
//! a fingerprint standing in for    equality is the bytes of facts, journal and snapshots, row by
//!   equality                       row; the printed fingerprint is never compared (§10.1 Q9)
//! the save diverging from memory   the in-memory run prints the same requests, facts, activity and
//!                                  history lines as the saved one
//! ```

mod headless;

use headless::{
    PACK, Tables, count_after, deterministic, every_seat_active_in_every_bucket, fresh, lines,
    mineworld, run, stderr,
};

const SEATS: [&str; 3] = ["alice", "visitor", "wanderer"];

#[test]
fn three_hundred_days_with_the_real_systems_and_the_same_seed_is_the_same_world() {
    let (first, second) = (fresh("run-300-a"), fresh("run-300-b"));
    let started = std::time::Instant::now();
    let (a, b, memory) = std::thread::scope(|scope| {
        let a = scope.spawn(|| run(7, 300, Some(&first)));
        let b = scope.spawn(|| run(7, 300, Some(&second)));
        let memory = scope.spawn(|| run(7, 300, None));
        (
            a.join().expect("run a"),
            b.join().expect("run b"),
            memory.join().expect("run in memory"),
        )
    });
    eprintln!(
        "three 300-day runs in parallel: {:.1} s wall",
        started.elapsed().as_secs_f64()
    );

    // ── AC-11, located before anything is compared (I-9). ──────────────────────────────────────
    assert_eq!(
        every_seat_active_in_every_bucket(&a, &SEATS),
        10,
        "ten 30-day buckets were read"
    );
    assert_eq!(lines(&a, "day ").len(), 300, "a line per simulated day");
    assert_eq!(lines(&a, "faults     0").len(), 1, "no system fault: {a}");
    let entered = lines(&a, "facts      person-entered-place ");
    assert!(
        entered.len() == 1 && count_after(entered[0], "person-entered-place ") > 0,
        "the street is used, so S6's PersonEnteredPlace occurs in a real run: {a}"
    );
    let history = lines(&a, "history ");
    let facts = count_after(history[0], "history ");
    assert!(
        facts > 100_000,
        "a busy world, not five genesis facts: {facts}"
    );

    // ── AC-12: the same seed is the same world, byte for byte. ────────────────────────────────
    let (first_tables, second_tables) = (Tables::read(&first), Tables::read(&second));
    assert_eq!(
        u64::try_from(first_tables.facts.len()).expect("fits"),
        facts,
        "the save holds every fact the run counted"
    );
    first_tables.assert_same_history(&second_tables, "two runs of seed 7");
    assert_ne!(
        first_tables.manifest.instance, second_tables.manifest.instance,
        "two runs are two worlds: the instance is the one thing excluded from AC-12 (ARC-27)"
    );
    assert_eq!(deterministic(&a), deterministic(&b), "and print the same");

    // Persistence does not change history: memory prints what the saved run printed, revisions apart.
    for prefix in [
        "requests ",
        "facts ",
        "activity ",
        "history ",
        "consults ",
        "faults ",
    ] {
        assert_eq!(lines(&a, prefix), lines(&memory, prefix), "{prefix}lines");
    }

    // And the save is a real one: replay re-executes it from genesis.
    let replayed = mineworld(&["replay", PACK, "--save", first.to_str().expect("path")]);
    assert!(replayed.status.success(), "{}", stderr(&replayed));
}

#[test]
fn a_different_seed_makes_a_different_world() {
    let (seven, eight) = (fresh("run-seed-7"), fresh("run-seed-8"));
    let printed_seven = run(7, 30, Some(&seven));
    let printed_eight = run(8, 30, Some(&eight));
    every_seat_active_in_every_bucket(&printed_seven, &SEATS);
    every_seat_active_in_every_bucket(&printed_eight, &SEATS);

    let (seven, eight) = (Tables::read(&seven), Tables::read(&eight));
    // Genesis is the pack's and is the same; what the controllers did is not.
    assert_eq!(seven.facts.first(), eight.facts.first());
    assert_ne!(
        seven.facts, eight.facts,
        "seed 8's history differs from seed 7's"
    );
}

#[test]
fn run_refuses_what_it_cannot_run_by_name_and_never_panics() {
    for (arguments, names) in [
        (
            vec!["run", PACK, "--seed", "1", "--days", "1"],
            "--headless",
        ),
        (vec!["run", PACK, "--headless", "--days", "1"], "--seed"),
        (vec!["run", PACK, "--headless", "--seed", "1"], "--days"),
        (
            vec!["run", PACK, "--headless", "--seed", "1", "--days", "0"],
            "--days",
        ),
        (
            vec!["run", PACK, "--headless", "--seed", "one", "--days", "1"],
            "--seed",
        ),
        (
            vec![
                "run",
                PACK,
                "--headless",
                "--seed",
                "1",
                "--days",
                "1",
                "--fast",
            ],
            "--fast",
        ),
        (
            vec![
                "run",
                "worlds/no-such-world",
                "--headless",
                "--seed",
                "1",
                "--days",
                "1",
            ],
            "worlds/no-such-world",
        ),
    ] {
        let output = mineworld(&arguments);
        let complaint = stderr(&output);
        assert!(!output.status.success(), "{arguments:?} must fail");
        assert!(complaint.contains(names), "{arguments:?}: {complaint}");
        assert!(
            !complaint.contains("panicked"),
            "{arguments:?}: {complaint}"
        );
    }
}
