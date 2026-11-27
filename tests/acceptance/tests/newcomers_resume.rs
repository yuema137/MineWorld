//! Real process death before, between and after admissions; no state repair on resume.
mod newcomers;
use mineworld_contracts::{ActionId, ActionIntent, ActionResult, WorldTime};
use mineworld_persistence::{
    Creation, Durability, PersistenceBackend, PersistentWorld, SqliteBackend, WorldRevision,
    format, verify,
};
use mineworld_test_support::process;
use newcomers::*;
use std::{
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{Command, Stdio},
};

const ROLE: &str = "MINEWORLD_NEWCOMER_ROLE";
const DIRECTORY: &str = "MINEWORLD_NEWCOMER_DIRECTORY";
const FENCE: &str = "MINEWORLD_NEWCOMER_FENCE";

fn create(path: &Path) -> PersistentWorld {
    let assembled = assembled(&pack());
    PersistentWorld::create(
        Box::new(SqliteBackend::create(path, Durability::PowerLoss).unwrap()),
        assembled.world,
        Creation {
            instance: 0x530c,
            pack: "attributes-newcomers".into(),
            at: WorldTime::EPOCH,
            facts: assembled.facts,
        },
    )
    .unwrap()
    .0
    .snapshot_every(3)
}

fn fence(world: &PersistentWorld) {
    println!("revision {}", world.revision().raw());
    std::io::stdout().flush().unwrap();
    // Parent acknowledgment prevents racing past the intended kill point.
    if std::env::var_os(FENCE).is_some() {
        let mut ack = String::new();
        assert!(std::io::stdin().read_line(&mut ack).unwrap() > 0);
    }
}

fn child(role: &str) {
    let directory = std::env::var(DIRECTORY).unwrap();
    let path = Path::new(&directory);
    let mut world = if role == "create" {
        create(path)
    } else {
        let backend = SqliteBackend::open(path, Durability::PowerLoss).unwrap();
        let genesis = backend
            .facts_of(WorldRevision::GENESIS)
            .unwrap()
            .iter()
            .map(|row| format::decode(&row.bytes, "fact").unwrap())
            .collect::<Vec<_>>();
        check_configuration(&pack(), &genesis).expect("disk genesis checked before resume");
        let (world, report) = PersistentWorld::resume(Box::new(backend), composed()).unwrap();
        println!(
            "resumed {} {} {}",
            report.head.raw(),
            report.snapshot.raw(),
            report.replayed
        );
        world.snapshot_every(3)
    };
    fence(&world);
    let done = world.highest_action_id().unwrap().unwrap_or(0) as usize;
    for index in done..4 {
        let at = WorldTime::from_seconds((index as i64 + 1) * 60);
        let _ = world.advance_to(at).unwrap();
        let result = world.dispatch(&intent(index), at).unwrap();
        let kinds = result
            .events()
            .iter()
            .map(|event| event.event_type().as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            ["test-arrived", "newcomer-admitted", "traits-stated"]
        );
        fence(&world);
    }
    // A final rejected request gives the after-admissions kill point a real remaining revision.
    // Idle clock advances intentionally do not journal (ARC-25).
    let original = intent(0);
    let duplicate = ActionIntent::new(
        ActionId::from_raw(5),
        original.actor(),
        original.payload().clone(),
        WorldTime::from_seconds(300),
    );
    let answered = world
        .dispatch(&duplicate, WorldTime::from_seconds(300))
        .unwrap();
    assert!(matches!(answered.result(), ActionResult::Rejected { .. }));
    world.checkpoint().unwrap();
    println!("done {}", world.revision().raw());
}

fn run(role: &str, path: &Path, kill_at: Option<u64>) -> Vec<String> {
    let mut child = Command::new(std::env::current_exe().unwrap())
        .env(ROLE, role)
        .env(DIRECTORY, path)
        .env(FENCE, "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let output = child.stdout.take().unwrap();
    let mut lines = Vec::new();
    let mut killed = false;
    for line in BufReader::new(output).lines() {
        let line = line.unwrap();
        if let Some(revision) = line.strip_prefix("revision ") {
            let revision: u64 = revision.parse().unwrap();
            if kill_at == Some(revision) {
                killed = process::kill(&mut child).killed();
            } else {
                writeln!(input, "continue").unwrap();
            }
        }
        lines.push(line);
    }
    let status = child.wait().unwrap();
    if kill_at.is_some() {
        assert!(killed);
        assert!(!status.success());
        assert!(!lines.iter().any(|l| l.starts_with("done")));
    } else {
        assert!(status.success());
        assert!(lines.last().unwrap().starts_with("done"));
    }
    lines
}

#[derive(Debug, PartialEq, Eq)]
struct Rows {
    journal: Vec<(u64, Vec<u8>)>,
    facts: Vec<(u64, u64, Vec<u8>)>,
    snapshots: Vec<(u64, Vec<u8>)>,
}
fn rows(path: &Path) -> Rows {
    let backend = SqliteBackend::open(path, Durability::PowerLoss).unwrap();
    let mut journal = Vec::new();
    let mut facts = Vec::new();
    for row in backend.revisions_after(WorldRevision::from_raw(0)).unwrap() {
        let row = row.unwrap();
        let revision = row.revision.raw();
        for fact in row.facts {
            facts.push((revision, fact.id, fact.bytes));
        }
        journal.push((revision, row.entry));
    }
    let snapshots = backend
        .snapshot_revisions()
        .unwrap()
        .into_iter()
        .map(|r| (r.raw(), backend.snapshot_at(r).unwrap().unwrap()))
        .collect();
    Rows {
        journal,
        facts,
        snapshots,
    }
}
fn head(path: &Path) -> u64 {
    SqliteBackend::open(path, Durability::PowerLoss)
        .unwrap()
        .head()
        .unwrap()
        .raw()
}

fn main() {
    if let Ok(role) = std::env::var(ROLE) {
        child(&role);
        return;
    }
    if std::env::args()
        .skip(1)
        .find(|arg| !arg.starts_with('-'))
        .is_some_and(|arg| !"newcomers_resume".contains(&arg))
    {
        return;
    }
    let started = std::time::Instant::now();
    let control = mineworld_test_support::scratch!(empty "newcomer-control");
    run("create", control.path(), None);
    let expected = rows(control.path());
    let total = head(control.path());
    let mut replayed = 0;
    for (label, kill_at) in [("before", 1), ("between", 3), ("after", 5)] {
        let victim = mineworld_test_support::scratch!(empty format!("newcomer-{label}"));
        run("create", victim.path(), Some(kill_at));
        assert_eq!(head(victim.path()), kill_at);
        assert!(kill_at < total);
        let lines = run("resume", victim.path(), None);
        let report: Vec<u64> = lines
            .iter()
            .find(|l| l.starts_with("resumed "))
            .unwrap()
            .split_whitespace()
            .skip(1)
            .map(|n| n.parse().unwrap())
            .collect();
        assert_eq!(report[0], kill_at);
        assert_eq!(report[2], report[0] - report[1]);
        replayed += report[2];
        assert_eq!(
            rows(victim.path()),
            expected,
            "{label}: entire save equals control"
        );
        let verified = verify(
            &SqliteBackend::open(victim.path(), Durability::PowerLoss).unwrap(),
            composed(),
        )
        .unwrap();
        assert_eq!(verified.revisions, total);
        println!(
            "{label}: killed at {kill_at}, resumed snapshot {}, replayed {}",
            report[1], report[2]
        );
    }
    assert!(replayed > 0);
    assert!(started.elapsed().as_secs() < 120);
    println!("newcomers_resume PASS in {:?}", started.elapsed());
}
