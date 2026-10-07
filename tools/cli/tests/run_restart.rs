//! `AC-6` end to end with controllers in the loop, and `F-13` closed for `run` (step-08 C5, `ARC-27`).
//!
//! ```text
//! control     mineworld run … --days 30 --save A                       uninterrupted
//! killed      mineworld run … --days 30 --save K   SIGKILL at day k    (k = 5, 15, 25)
//!             mineworld run … --days 30 --save K   the same command, a new process
//! continued   mineworld run … --days 10 --save C   then  --days 30 --save C
//! ```
//!
//! Every survivor's facts, journal and snapshots must equal the control's, byte for byte.
//!
//! What would make this pass without proving anything, and how each is excluded (`ARC-23`):
//!
//! ```text
//! the victim was never killed       its status is a signal, it printed no summary, and the save's
//!                                   head is short of the control's
//! the survivor ignored the save     it reports resuming at that head, with a re-executed tail on at
//!                                   least one kill point
//! F-13 never arose                  a line heard before the day-10 restart and answered after it is
//!                                   located in the log, and it is answered exactly once
//! ```

mod headless;

use std::io::{BufRead, BufReader};
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{Command, Stdio};

use headless::{BINARY, PACK, Tables, every_seat_active_in_every_bucket, fresh, run, seats};
use mineworld_contracts::{EventEnvelope, EventRecord};
use mineworld_conversation::Spoke;
use mineworld_persistence::format;

const SEED: u64 = 7;
const DAYS: u64 = 30;
/// `mineworld run`'s pace (`tools/cli/src/run.rs` `PACE`): the reply window is one pace long.
const PACE: i64 = 900;
const DAY: i64 = 86_400;

/// Starts `run --save save` and SIGKILLs it once it has printed the line for `day`.
fn killed_at(save: &Path, day: u64) {
    let mut child = Command::new(BINARY)
        .args([
            "run",
            PACK,
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
    child.kill().expect("SIGKILL is delivered");
    let status = child.wait().expect("reaped");
    assert_eq!(status.signal(), Some(9), "killed by SIGKILL, not finished");
    assert!(
        !printed.iter().any(|line| line.starts_with("history ")),
        "the victim did not finish: {printed:?}"
    );
}

/// "resumed … at revision R (snapshot S + N re-executed)" → (R, N).
fn resumed(printed: &str) -> (u64, u64) {
    let header = printed.lines().next().expect("a header");
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
    assert!(
        header.contains("resumed"),
        "a resume, not a creation: {header}"
    );
    (number_after("at revision "), number_after(" + "))
}

#[test]
fn a_killed_run_re_run_finishes_the_same_world_byte_for_byte() {
    let control = fresh("run-restart-control");
    let printed = run(SEED, DAYS, Some(&control));
    let seats = seats();
    let seats: Vec<&str> = seats.iter().map(String::as_str).collect();
    every_seat_active_in_every_bucket(&printed, &seats);
    let control = Tables::read(&control);
    let control_head = u64::try_from(control.journal.len()).expect("fits");

    let mut tails = Vec::new();
    for day in [5, 15, 25] {
        let save = fresh(&format!("run-restart-killed-{day}"));
        killed_at(&save, day);
        let head_at_kill = u64::try_from(Tables::read(&save).journal.len()).expect("fits");
        assert!(
            head_at_kill < control_head,
            "day {day}: killed short of the end ({head_at_kill} of {control_head})"
        );

        let survivor = run(SEED, DAYS, Some(&save));
        let (head, tail) = resumed(&survivor);
        assert_eq!(head, head_at_kill, "day {day}: resumed at the head on disk");
        tails.push(tail);
        Tables::read(&save).assert_same_history(&control, &format!("killed at day {day}"));
    }
    assert!(
        tails.iter().any(|tail| *tail > 0),
        "at least one resume re-executed a tail after its snapshot: {tails:?}"
    );
}

fn spoken(fact: &EventEnvelope) -> Option<Spoke> {
    let record: &EventRecord = fact.payload();
    serde_json::from_slice(record.payload_for::<Spoke>().ok()?).ok()
}

/// Lines said to somebody before `restart` and answered by them after it, within one pace:
/// `(said at, speaker, listener)`.
fn straddling(facts: &[(i64, Spoke)], restart: i64) -> Vec<(i64, Spoke)> {
    facts
        .iter()
        .filter(|(said_at, line)| {
            *said_at < restart
                && facts.iter().any(|(at, reply)| {
                    *at >= restart
                        && *at - *said_at < PACE
                        && reply.speaker() == line.listener()
                        && reply.listener() == line.speaker()
                })
        })
        .cloned()
        .collect()
}

#[test]
fn stopping_and_continuing_answers_a_line_that_straddles_the_restart_exactly_once() {
    let control = fresh("run-continue-control");
    run(SEED, DAYS, Some(&control));
    let control = Tables::read(&control);
    let facts: Vec<(i64, Spoke)> = control
        .facts
        .iter()
        .map(|(_, bytes)| format::decode::<EventEnvelope>(bytes, "fact").expect("a fact"))
        .filter_map(|fact| spoken(&fact).map(|spoke| (fact.at().seconds(), spoke)))
        .collect();
    assert!(
        facts.len() > 1_000,
        "the spoke facts were read: {}",
        facts.len()
    );

    // F-13's case, located rather than hoped for: the first day boundary at which a line said
    // before it is answered after it. A run stopped at that day must answer it once, not twice.
    let (day, lines) = (1..i64::try_from(DAYS).expect("fits"))
        .map(|day| (day, straddling(&facts, day * DAY)))
        .find(|(_, lines)| !lines.is_empty())
        .expect("some day boundary in 30 days has a straddling line");
    eprintln!("stopping at day {day}: {} straddling line(s)", lines.len());

    let continued = fresh("run-continue");
    run(
        SEED,
        u64::try_from(day).expect("positive"),
        Some(&continued),
    );
    let second = run(SEED, DAYS, Some(&continued));
    resumed(&second);
    Tables::read(&continued)
        .assert_same_history(&control, &format!("stopped at day {day} and continued"));

    // In the history both runs share, a line is answered — quoted back — at most once before its
    // speaker says anything new to the same listener, and each straddling line exactly once. A
    // controller that forgot it had answered would quote it again at its next consult.
    let answered = |said_at: i64, line: &Spoke| -> usize {
        let quoted = quoted(line.utterance().as_str());
        let opening = format!("I remember you. You said \"{quoted}\"");
        let next_line = facts
            .iter()
            .filter(|(at, later)| {
                *at > said_at
                    && later.speaker() == line.speaker()
                    && later.listener() == line.listener()
            })
            .map(|(at, _)| *at)
            .min()
            .unwrap_or(i64::MAX);
        facts
            .iter()
            .filter(|(at, reply)| {
                *at > said_at
                    && *at < next_line
                    && reply.speaker() == line.listener()
                    && reply.listener() == line.speaker()
                    && reply.utterance().as_str().starts_with(&opening)
            })
            .count()
    };
    let twice: Vec<i64> = facts
        .iter()
        .filter(|(said_at, line)| answered(*said_at, line) > 1)
        .map(|(said_at, _)| *said_at)
        .collect();
    assert!(
        twice.is_empty(),
        "lines answered more than once, said at: {twice:?}"
    );
    for (said_at, line) in &lines {
        assert_eq!(
            answered(*said_at, line),
            1,
            "the line said at {said_at} is answered exactly once"
        );
    }
}

/// How the rule controllers quote somebody: at most 80 characters, then an ellipsis.
fn quoted(said: &str) -> String {
    match said.char_indices().nth(80) {
        None => said.to_owned(),
        Some((boundary, _)) => format!("{}…", &said[..boundary]),
    }
}
