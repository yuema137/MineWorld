//! PB-11 — `worlds/bodies-yard` is deterministic with bodies resolving every arrival: two processes
//! give the same bytes, and a run SIGKILLed at day 5, 15 or 25 and run again finishes the same world
//! (`ARC-25`, `ARC-27`; the `run_restart.rs` pattern).
//!
//! ```text
//! control    run … --days 30 --save A
//! second     run … --days 30 --save B                         facts, journal, snapshots = A's
//! killed     run … --days 30 --save K   SIGKILL at day k      (k = 5, 15, 25)
//!            run … --days 30 --save K   a new process         = A's, resumed at the head on disk
//! replay     mineworld replay … --save A                      re-executed from genesis
//! ```
//!
//! What would make this pass without proving anything, and how each is excluded (`ARC-23`): the
//! victim was never killed (its status is a signal, it printed no summary, and its head is short of
//! the control's); the survivor ignored the save (it reports resuming at that head, with a
//! re-executed tail on at least one kill point); nothing was resolved (the control's save holds
//! stopped-shorts and displaced arrivals, counted before any comparison).

mod bodies;
mod headless;

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

use bodies::{YARD, keys_of, run, scan};
use headless::{BINARY, Tables, fresh, mineworld, stderr, stdout};
use mineworld_test_support::process;

const SEED: u64 = 7;
const DAYS: u64 = 30;

/// Starts `run --save save` and SIGKILLs it once it has printed the line for `day`.
fn killed_at(save: &Path, day: u64) {
    let mut child = Command::new(BINARY)
        .args([
            "run",
            YARD,
            "--headless",
            "--seed",
            &SEED.to_string(),
            "--days",
            &DAYS.to_string(),
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

/// "resumed … at revision R (snapshot S + N re-executed)" → (R, N).
fn resumed(printed: &str) -> (u64, u64) {
    let header = printed.lines().next().expect("a header");
    assert!(header.contains("resumed"), "a resume: {header}");
    let number_after = |label: &str| -> u64 {
        header
            .split(label)
            .nth(1)
            .and_then(|rest| {
                rest.trim_start()
                    .split(|c: char| !c.is_ascii_digit())
                    .next()
            })
            .and_then(|digits| digits.parse().ok())
            .unwrap_or_else(|| panic!("no '{label}' in: {header}"))
    };
    (number_after("at revision "), number_after(" + "))
}

#[test]
fn bodies_yard_is_the_same_world_in_two_processes_and_after_sigkill() {
    let a = fresh("bodies-yard-restart-a");
    run(Path::new(YARD), SEED, DAYS, Some(&a));
    let control = Tables::read(&a);
    let report = scan(&control, &keys_of(Path::new(YARD)));
    let total = |count: fn(&bodies::Bucket) -> u64| report.buckets.iter().map(count).sum::<u64>();
    let stopped = total(|bucket| bucket.stopped);
    let displaced = total(|bucket| bucket.displaced);
    let (kicks, throws, shoves, pushes) = (
        total(|bucket| bucket.kicks),
        total(|bucket| bucket.throws),
        total(|bucket| bucket.shoves),
        total(|bucket| bucket.pushes),
    );
    println!(
        "control: {} facts, {} journal rows, {} snapshots; {stopped} stopped-short, {displaced} \
         displaced arrivals; {kicks} kicks, {throws} throws, {shoves} shoves, {pushes} pushes",
        control.facts.len(),
        control.journal.len(),
        control.snapshots.len()
    );
    assert!(
        stopped > 0 && displaced > 0,
        "bodies resolved arrivals in the control"
    );
    // PO-11 (QO-16): the control holds every physical interaction, so determinism is shown with
    // flights, pushes and shoves in it, not only people walking.
    assert!(
        kicks > 0 && throws > 0 && shoves > 0 && pushes > 0,
        "the control kicks, throws, shoves and pushes"
    );

    let b = fresh("bodies-yard-restart-b");
    run(Path::new(YARD), SEED, DAYS, Some(&b));
    Tables::read(&b).assert_same_history(&control, "a second process");

    let head = u64::try_from(control.journal.len()).expect("fits");
    let mut tails = Vec::new();
    for day in [5, 15, 25] {
        let save = fresh(&format!("bodies-yard-restart-killed-{day}"));
        killed_at(&save, day);
        let at_kill = u64::try_from(Tables::read(&save).journal.len()).expect("fits");
        assert!(
            at_kill < head,
            "day {day}: killed short of the end ({at_kill} of {head})"
        );
        let survivor = run(Path::new(YARD), SEED, DAYS, Some(&save));
        let (resumed_at, tail) = resumed(&survivor);
        println!("day {day}: killed at revision {at_kill}; resumed there, {tail} re-executed");
        assert_eq!(
            resumed_at, at_kill,
            "day {day}: resumed at the head on disk"
        );
        tails.push(tail);
        Tables::read(&save).assert_same_history(&control, &format!("killed at day {day}"));
    }
    assert!(
        tails.iter().any(|tail| *tail > 0),
        "a tail re-executed: {tails:?}"
    );

    let replay = mineworld(&["replay", YARD, "--save", a.to_str().expect("a path")]);
    println!("replay: {}", stdout(&replay).trim());
    assert!(
        replay.status.success(),
        "verify from genesis: {}",
        stderr(&replay)
    );
}
