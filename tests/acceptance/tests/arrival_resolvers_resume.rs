//! SC-6 — a world with a resolver installed survives `SIGKILL`, and continues byte for byte as if it
//! never died (step-11 §16 RS-11; `docs/DECISIONS.md` `ARC-25`, `ARC-39`).
//!
//! persistence's `kill_and_resume` pattern, with test-fences in the world:
//!
//! ```text
//! parent      control   a child runs the whole script into its own save, uninterrupted
//!             for each kill point K (early, middle, late):
//!               victim    a child runs the script into a fresh save; once it has printed
//!                         "revision K" the parent SIGKILLs it
//!               survivor  a NEW child process opens the same file, resumes it — re-executing the
//!                         journal tail, which asks the resolver again — and finishes the script
//!               compare   journal, facts and snapshots of the survivor's save and the control's,
//!                         byte for byte; and verify() the survivor's save from genesis
//! ```
//!
//! Every child, and the parent before `verify`, registers `[fences]` before composing: the catalog is
//! per process, and a process that installs test-fences without registering it panics (`ARC-39`).
//!
//! What would make this pass without proving anything, and how each is excluded (`ARC-23`):
//!
//! ```text
//! the child was never killed           its exit status must be SIGKILL, it must not have printed
//!                                      "done", and the file's head must be short of the control's
//! the survivor did not read the file   it reports the head it resumed from, which must be the one on
//!                                      disk, and the tail it re-executed
//! the tail was empty every time        at least one kill point resumes with a tail > 0
//! the resolver never changed anything  the control's stopped-short facts and displaced arrivals are
//!                                      counted first, and each must be > 0
//! ```
//!
//! This file is a program (`harness = false`), because the same binary is both the parent that kills
//! and the child that is killed.

mod resolvers;

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use mineworld_contracts::{ActionId, ActionIntent, ActionRecord, ActionResult, WorldTime};
use mineworld_movement::Move;
use mineworld_persistence::{
    Creation, Durability, PersistenceBackend, PersistentWorld, SqliteBackend, WorldRevision, verify,
};
use mineworld_presence::{ArrivalResolver, register_resolvers};
use mineworld_test_support::{Scratch, process};
use resolvers::{Fences, Pack, Plan, Where, at};

const ROLE: &str = "MINEWORLD_RESOLVER_KILL_ROLE";
const DIRECTORY: &str = "MINEWORLD_RESOLVER_KILL_DIR";

const INSTANCE: u128 = 0x0000_0000_0000_0000_0000_0000_f3ce_0012;
const SNAPSHOT_INTERVAL: u64 = 32;
const STEPS: u64 = 400;
const DURABILITY: Durability = Durability::PowerLoss;

/// The four people and where genesis places them: two west of the fence, two east of it.
const PEOPLE: [&str; 4] = ["alice", "bob", "carol", "dan"];

fn plan() -> Plan {
    let mut plan = Plan::new(
        &[Pack::Movement, Pack::Fences],
        &[
            ("alice", Some((Where::Yard, 3_800, 1_500))),
            ("bob", Some((Where::Yard, 4_200, 2_500))),
            ("carol", Some((Where::Yard, 5_300, 1_800))),
            ("dan", Some((Where::Yard, 6_000, 2_200))),
        ],
    );
    plan.fence = Some(5_000);
    plan
}

/// The one resolver this program's processes register, before anything composes.
fn register() {
    register_resolvers(vec![Box::new(Fences) as Box<dyn ArrivalResolver>]);
}

fn t(seconds: i64) -> WorldTime {
    WorldTime::from_seconds(seconds)
}

/// A deterministic mix of two numbers: the script's only source of variety, the same in every
/// process.
fn mix(a: u64, b: u64) -> u64 {
    let mut x = a
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(b.wrapping_mul(0xC2B2_AE3D_27D4_EB4F));
    x ^= x >> 29;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^ (x >> 32)
}

/// Step `index`: at `20 * (index + 1)` seconds, request `index + 1` — seat `index % 4` asks to move
/// to a point in the 3 m × 2 m box straddling the fence. Movement refuses one more than a stride from
/// where the person is; the fence stops one that crosses it from the west and makes room around the
/// stop.
fn intent(index: u64, cast: &resolvers::Cast) -> (i64, ActionIntent) {
    let actor = cast.person(PEOPLE[usize::try_from(index % 4).expect("small")]);
    let roll = mix(index, 13);
    let x = 3_500 + i32::try_from(roll % 3_000).expect("small");
    let y = 1_000 + i32::try_from((roll >> 16) % 2_000).expect("small");
    let at_seconds = 20 * i64::try_from(index + 1).expect("small");
    let payload = serde_json::to_vec(&Move::new(at(cast.yard, x, y))).expect("encodes");
    (
        at_seconds,
        ActionIntent::new(
            ActionId::from_raw(index + 1),
            actor.entity_id(),
            ActionRecord::new::<Move>(payload),
            t(at_seconds),
        ),
    )
}

// =============================================================================================
// the children
// =============================================================================================

fn create(directory: &Path) -> PersistentWorld {
    let backend = SqliteBackend::create(directory, DURABILITY).expect("a new save is created");
    let (world, _, facts) = plan().assembled();
    let (persisted, _) = PersistentWorld::create(
        Box::new(backend),
        world,
        Creation {
            instance: INSTANCE,
            pack: "resolver-yard".to_owned(),
            at: WorldTime::EPOCH,
            facts,
        },
    )
    .expect("the world begins");
    persisted.snapshot_every(SNAPSHOT_INTERVAL)
}

/// Drives the world through every step the save does not hold yet, printing each committed revision
/// and what each accepted move recorded.
fn finish(world: &mut PersistentWorld) {
    let report = |world: &PersistentWorld| println!("revision {}", world.revision().raw());
    let (_, cast, _) = plan().assembled();
    let done = world.highest_action_id().expect("reads").unwrap_or(0);
    for index in done..STEPS {
        let (at_seconds, intent) = intent(index, &cast);
        let _ = world.advance_to(t(at_seconds)).expect("advances");
        report(world);
        let answered = world.dispatch(&intent, t(at_seconds)).expect("answered");
        if matches!(answered.result(), ActionResult::Accepted { .. }) {
            let kinds: Vec<&str> = answered
                .events()
                .iter()
                .map(|event| event.event_type().as_str())
                .collect();
            let arrived = kinds.iter().filter(|kind| **kind == "arrived").count();
            let stopped = kinds
                .iter()
                .filter(|kind| **kind == "stopped-short")
                .count();
            println!("move accepted stopped {stopped} displaced {}", arrived - 1);
        } else {
            println!("move refused");
        }
        report(world);
    }
}

fn child(role: &str) {
    register();
    let directory = PathBuf::from(std::env::var(DIRECTORY).expect("a directory"));
    let mut world = match role {
        "create" => create(&directory),
        "resume" => {
            let backend = SqliteBackend::open(&directory, DURABILITY).expect("the save opens");
            let (world, how) =
                PersistentWorld::resume(Box::new(backend), plan().composed()).expect("resumes");
            println!(
                "resumed head {} snapshot {} replayed {} facts {}",
                how.head.raw(),
                how.snapshot.raw(),
                how.replayed,
                how.facts
            );
            world.snapshot_every(SNAPSHOT_INTERVAL)
        }
        other => panic!("no role {other}"),
    };
    finish(&mut world);
    world.checkpoint().expect("checkpoints");
    println!("done {}", world.revision().raw());
}

// =============================================================================================
// the parent
// =============================================================================================

struct Ran {
    lines: Vec<String>,
    killed: bool,
}

/// Runs a child to completion, or kills it with SIGKILL as soon as it reports revision `kill_at`.
fn run(role: &str, directory: &Path, kill_at: Option<u64>) -> Ran {
    let mut process = Command::new(std::env::current_exe().expect("this program"))
        .env(ROLE, role)
        .env(DIRECTORY, directory)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("a child starts");
    let stdout = process.stdout.take().expect("piped");
    let mut lines = Vec::new();
    let mut sent_kill = None;
    for line in BufReader::new(stdout).lines() {
        let line = line.expect("a line");
        let reached = line
            .strip_prefix("revision ")
            .and_then(|number| number.parse::<u64>().ok());
        lines.push(line);
        if let (Some(kill_at), Some(reached)) = (kill_at, reached)
            && reached >= kill_at
            && sent_kill.is_none()
        {
            sent_kill = Some(process::kill(&mut process));
        }
    }
    let status = process.wait().expect("the child is reaped");
    if kill_at.is_none() {
        assert!(status.success(), "{role} failed: {status}");
    }
    Ran {
        lines,
        killed: sent_kill.is_some_and(|sent| sent.killed()),
    }
}

/// Every journal row, fact and snapshot of a save, as stored, for a byte-for-byte comparison.
#[derive(Debug, PartialEq, Eq)]
struct Rows {
    journal: Vec<(u64, Vec<u8>)>,
    facts: Vec<(u64, u64, Vec<u8>)>,
    snapshots: Vec<(u64, Vec<u8>)>,
}

fn rows(directory: &Path) -> Rows {
    let backend = SqliteBackend::open(directory, DURABILITY).expect("the save opens");
    let journal: Vec<(u64, Vec<u8>)> = backend
        .journal_after(WorldRevision::from_raw(0))
        .expect("reads")
        .into_iter()
        .map(|(revision, entry)| (revision.raw(), entry))
        .collect();
    let mut facts = Vec::new();
    for (revision, _) in &journal {
        for fact in backend
            .facts_of(WorldRevision::from_raw(*revision))
            .expect("reads")
        {
            facts.push((*revision, fact.id, fact.bytes));
        }
    }
    let snapshots = backend
        .snapshot_revisions()
        .expect("reads")
        .into_iter()
        .map(|revision| {
            let bytes = backend
                .snapshot_at(revision)
                .expect("reads")
                .expect("a listed snapshot exists");
            (revision.raw(), bytes)
        })
        .collect();
    Rows {
        journal,
        facts,
        snapshots,
    }
}

fn head(directory: &Path) -> u64 {
    SqliteBackend::open(directory, DURABILITY)
        .expect("opens")
        .head()
        .expect("reads")
        .raw()
}

/// The parent's scratch: only the parent owns it; a child is handed its path (DEP-29).
fn scratch(name: &str) -> Scratch {
    mineworld_test_support::scratch!(empty format!("resolver-kill-{name}"))
}

/// The resume line: (head, snapshot, replayed, facts).
fn resumed_line(lines: &[String]) -> (u64, u64, u64, u64) {
    let line = lines
        .iter()
        .find(|line| line.starts_with("resumed "))
        .expect("the survivor reports how it resumed");
    let numbers: Vec<u64> = line
        .split_whitespace()
        .filter_map(|word| word.parse().ok())
        .collect();
    (numbers[0], numbers[1], numbers[2], numbers[3])
}

/// The control's activity: (moves accepted, moves refused, stopped-short facts, displaced arrivals).
fn activity(lines: &[String]) -> (u64, u64, u64, u64) {
    let (mut accepted, mut refused, mut stopped, mut displaced) = (0, 0, 0, 0);
    for line in lines {
        if line == "move refused" {
            refused += 1;
        } else if let Some(rest) = line.strip_prefix("move accepted stopped ") {
            let numbers: Vec<u64> = rest
                .split_whitespace()
                .filter_map(|word| word.parse().ok())
                .collect();
            accepted += 1;
            stopped += numbers[0];
            displaced += numbers[1];
        }
    }
    (accepted, refused, stopped, displaced)
}

fn main() {
    if let Ok(role) = std::env::var(ROLE) {
        child(&role);
        return;
    }
    // A filter naming something else skips this program, so `cargo test -p mineworld-acceptance
    // some_other_test` stays fast.
    let filter = std::env::args()
        .skip(1)
        .find(|argument| !argument.starts_with('-'));
    if filter
        .as_deref()
        .is_some_and(|filter| !"arrival_resolvers_resume".contains(filter))
    {
        return;
    }
    register();
    let started = std::time::Instant::now();

    let control_dir = scratch("control");
    let control = run("create", &control_dir, None);
    assert!(
        control
            .lines
            .last()
            .is_some_and(|line| line.starts_with("done "))
    );
    let total = head(&control_dir);
    let control_rows = rows(&control_dir);
    // Located first: the resolver changed arrivals in this run, both ways, so equality below is about
    // resolved facts and not about a world in which the fence never mattered.
    let (accepted, refused, stopped, displaced) = activity(&control.lines);
    println!(
        "[resolver-yard] control: {total} revisions, {} facts, {} snapshots; {accepted} moves \
         accepted, {refused} refused; {stopped} stopped-short, {displaced} displaced arrivals",
        control_rows.facts.len(),
        control_rows.snapshots.len()
    );
    assert_eq!(accepted + refused, STEPS, "every move is answered");
    assert!(stopped > 0, "the fence stopped somebody: {stopped}");
    assert!(
        displaced > 0,
        "the fence made somebody make room: {displaced}"
    );
    assert!(total > STEPS, "the control run holds {total} revisions");

    let mut tails = Vec::new();
    for (label, kill_at) in [
        ("early", total / 5),
        ("middle", total / 2 + 3),
        ("late", total * 4 / 5 + 7),
    ] {
        let dir = scratch(label);
        let victim = run("create", &dir, Some(kill_at));
        let on_disk = head(&dir);
        assert!(victim.killed, "{label}: the victim died of SIGKILL");
        assert!(
            !victim.lines.iter().any(|line| line.starts_with("done")),
            "{label}: the victim never finished"
        );
        assert!(
            on_disk >= kill_at && on_disk < total,
            "{label}: the save stops mid-run ({on_disk}, killed at {kill_at}, full run {total})"
        );

        let survivor = run("resume", &dir, None);
        let (resumed_head, snapshot, replayed, facts) = resumed_line(&survivor.lines);
        assert_eq!(
            resumed_head, on_disk,
            "{label}: the survivor resumed from the file's head"
        );
        assert_eq!(
            replayed,
            on_disk - snapshot,
            "{label}: and re-executed the tail after its snapshot"
        );
        println!(
            "[resolver-yard] {label}: kill sent at revision {kill_at}, {on_disk} committed; survivor \
             restored snapshot {snapshot} and re-executed {replayed} revisions ({facts} facts)"
        );
        tails.push(replayed);

        assert_eq!(
            rows(&dir),
            control_rows,
            "{label}: the survivor's save is the control's, byte for byte"
        );
        let verified = verify(
            &SqliteBackend::open(&dir, DURABILITY).expect("opens"),
            plan().composed(),
        )
        .expect("the survivor's whole history reproduces");
        assert_eq!(verified.revisions, total);
    }
    assert!(
        tails.iter().any(|tail| *tail > 0),
        "at least one resume re-executed a tail: {tails:?}"
    );
    println!(
        "[resolver-yard] PASS in {:.1} s",
        started.elapsed().as_secs_f64()
    );
}
