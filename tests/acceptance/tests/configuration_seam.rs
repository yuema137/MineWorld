//! The configuration seam, end to end with a test-only pack (`docs/DECISIONS.md` `ARC-61`;
//! step-18-interaction-list §11.4 IA-2, IA-4 (a), IA-7).
//!
//! `test-tuning` (`configuration/mod.rs`) is configured through the seam exactly as the loader does
//! it — its configuration decoded by `SystemPack::decode_configuration` (which `configures!()`
//! defines), its section by authoring's `Decode`, each seeded through `Seeding` — and then driven
//! through `World::dispatch` inside a `PersistentWorld`:
//!
//! ```text
//! IA-2    the section's reduction checks its value against the configured step: a multiple is
//!         taken, another refused; reduced before the configuration it is refused as unconfigured,
//!         which is why the loader seeds configuration first (the loader's order itself is held by
//!         worldpack's in-crate test)
//! IA-4 a  a save's genesis read back from disk, compared with this configuration: unchanged is
//!         accepted; a changed step, a removed configuration and an added one are each drift naming
//!         test-tuning
//! IA-7    a configured world killed with SIGKILL mid-run resumes in a new process and finishes byte
//!         for byte as an uninterrupted run, and its whole history verifies from genesis
//! ```
//!
//! IA-5 (two extension catalogs, in order, once) is `sdk/rust/tests/extensions.rs` (QIA-6).
//!
//! The SIGKILL victim and survivor are this test binary re-run on one test, selected by an environment
//! variable; without it, that test returns at once. Scratch saves are `mineworld_test_support`'s
//! scratch, removed when the test ends (`DEP-29`).

mod configuration;

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{Command, Stdio};

use configuration::{
    Advance, Advanced, Count, Stride, Tuning, TuningConfigured, composed, genesis_facts, world,
};
use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, ActionResult, EntityKey, Event, EventEnvelope, SystemId,
    WorldTime,
};
use mineworld_kernel::{KernelError, SystemIdentity, World};
use mineworld_persistence::{
    Creation, Durability, PersistenceBackend, PersistentWorld, SqliteBackend, WorldRevision,
    format, verify,
};
use mineworld_test_support::Scratch;
use mineworld_worldpack::configure::compare;

const PEOPLE: [&str; 2] = ["ada", "bo"];
const CONFIGURED: &str = "# configure/test-tuning.yaml\nstep: 5\n";
const STARTS: [(&str, u32); 2] = [("ada", 10), ("bo", 15)];
const DURABILITY: Durability = Durability::PowerLoss;
const STEPS: u64 = 240;
const SNAPSHOT_INTERVAL: u64 = 32;

const ROLE: &str = "MINEWORLD_CONFIGURATION_KILL_ROLE";
const DIRECTORY: &str = "MINEWORLD_CONFIGURATION_KILL_DIR";

fn key(value: &str) -> EntityKey {
    EntityKey::new(value).expect("a key")
}

/// The world, assembled and seeded through the seam with `configuration`.
fn assembled(
    configuration: Option<&str>,
    starts: &[(&str, u32)],
) -> (
    World,
    BTreeMap<EntityKey, mineworld_contracts::EntityId>,
    Vec<mineworld_kernel::Emission>,
) {
    let (world, ids) = world(&PEOPLE);
    let facts = genesis_facts(&world, &ids, configuration, starts, false).expect("seeds");
    (world, ids, facts)
}

/// The parent's scratch, an empty directory removed when the test ends; a SIGKILL child is handed its
/// path and never owns it (`DEP-29`).
fn scratch(name: &str) -> Scratch {
    mineworld_test_support::scratch!(empty format!("configuration-{name}"))
}

// ---- IA-2 ------------------------------------------------------------------------------------

#[test]
fn a_section_is_checked_against_the_configuration_seeded_before_it() {
    let (mut tuned, ids, facts) = assembled(Some(CONFIGURED), &STARTS);
    tuned
        .genesis(WorldTime::EPOCH, facts)
        .expect("multiples of 5 are taken");
    let read = tuned.read();
    assert_eq!(
        read.component::<Stride>(ids[&key("square")]),
        Some(&Stride { step: 5 }),
        "the configured step is on the place"
    );
    assert_eq!(
        read.component::<Count>(ids[&key("bo")]),
        Some(&Count { value: 15 })
    );

    let refusal = |configuration, starts: &[(&str, u32)], sections_first| {
        let (mut world, ids) = world(&PEOPLE);
        let facts =
            genesis_facts(&world, &ids, configuration, starts, sections_first).expect("seeds");
        match world.genesis(WorldTime::EPOCH, facts) {
            Err(KernelError::FactRefusedByOwner { system, reason, .. }) => {
                assert_eq!(system, Tuning::ID);
                format!("{reason:?}")
            }
            other => panic!("expected a refusal by test-tuning, got {other:?}"),
        }
    };
    assert!(refusal(Some(CONFIGURED), &[("ada", 12)], false).contains("tuning-start-off-step"));
    assert!(
        refusal(Some(CONFIGURED), &STARTS, true).contains("tuning-unconfigured"),
        "a section reduced before its configuration has nothing to check against"
    );

    let out_of_bound = genesis_facts(&tuned, &ids, Some("step: 0\n"), &[], false)
        .expect_err("the pack's own bound");
    assert!(
        out_of_bound.contains("line 1 column 7") && out_of_bound.contains("1 to 100, not 0"),
        "{out_of_bound}"
    );
}

// ---- IA-4 (a) --------------------------------------------------------------------------------

/// A save's genesis facts, read back from disk as a host reads them.
fn saved_genesis(directory: &Path) -> Vec<EventEnvelope> {
    SqliteBackend::open(directory, DURABILITY)
        .expect("the save opens")
        .facts_of(WorldRevision::GENESIS)
        .expect("reads")
        .iter()
        .map(|row| format::decode(&row.bytes, "fact").expect("decodes"))
        .collect()
}

fn create_save(directory: &Path, configuration: Option<&str>) -> PersistentWorld {
    let (world, _, facts) = assembled(
        configuration,
        if configuration.is_some() {
            &STARTS
        } else {
            &[]
        },
    );
    let backend = SqliteBackend::create(directory, DURABILITY).expect("a new save");
    PersistentWorld::create(
        Box::new(backend),
        world,
        Creation {
            instance: 0x0000_0000_0000_0000_0000_0000_c0f1_0001,
            pack: "tuning-yard".to_owned(),
            at: WorldTime::EPOCH,
            facts,
        },
    )
    .expect("the world begins")
    .0
}

#[test]
fn a_saves_configuration_is_compared_and_every_difference_is_drift_naming_the_pack() {
    let owners = BTreeMap::from([(TuningConfigured::EVENT_TYPE, Tuning::ID)]);
    let here = |configuration: Option<&str>| {
        assembled(
            configuration,
            if configuration.is_some() {
                &STARTS
            } else {
                &[]
            },
        )
        .2
    };

    let configured = scratch("drift-configured");
    create_save(configured.path(), Some(CONFIGURED));
    let saved = saved_genesis(configured.path());
    assert!(
        saved
            .iter()
            .any(|fact| *fact.event_type() == TuningConfigured::EVENT_TYPE),
        "the save holds its configuration"
    );
    assert_eq!(compare(&saved, &here(Some(CONFIGURED)), &owners), Ok(()));
    assert_eq!(
        compare(&saved, &here(Some("step: 5  # reformatted\n")), &owners),
        Ok(()),
        "the value is compared, not the text"
    );

    let changed = compare(&saved, &here(Some("step: 25\n")), &owners).expect_err("changed");
    assert_eq!(changed.system, SystemId::from_static("test-tuning"));
    assert!(changed.saved.contains(r#""step":5"#) && changed.here.contains(r#""step":25"#));

    let removed = compare(&saved, &here(None), &owners).expect_err("removed");
    assert_eq!(
        (removed.system, removed.here.as_str()),
        (Tuning::ID, "nothing")
    );

    let unconfigured = scratch("drift-unconfigured");
    create_save(unconfigured.path(), None);
    let added = compare(
        &saved_genesis(unconfigured.path()),
        &here(Some(CONFIGURED)),
        &owners,
    )
    .expect_err("added");
    assert_eq!(
        (added.system, added.saved.as_str()),
        (Tuning::ID, "nothing")
    );
}

// ---- IA-7 ------------------------------------------------------------------------------------

fn t(seconds: i64) -> WorldTime {
    WorldTime::from_seconds(seconds)
}

/// Request `index + 1`, at `10 * (index + 1)` seconds: ada and bo advance in turn.
fn intent(
    index: u64,
    ids: &BTreeMap<EntityKey, mineworld_contracts::EntityId>,
) -> (i64, ActionIntent) {
    let actor = ids[&key(PEOPLE[usize::try_from(index % 2).expect("small")])];
    let at = 10 * i64::try_from(index + 1).expect("small");
    let payload = serde_json::to_vec(&Advance {}).expect("encodes");
    (
        at,
        ActionIntent::new(
            ActionId::from_raw(index + 1),
            actor,
            ActionRecord::new::<Advance>(payload),
            t(at),
        ),
    )
}

/// The child's script: create or resume, then every step the save does not hold, each revision
/// reported.
fn child(role: &str, directory: &Path) {
    // The harness has printed "test sigkill_child_entry_point ... " with no newline: every report
    // below must start its own line.
    println!();
    let (_, ids, _) = assembled(Some(CONFIGURED), &STARTS);
    let mut world = match role {
        "create" => create_save(directory, Some(CONFIGURED)),
        "resume" => {
            let backend = SqliteBackend::open(directory, DURABILITY).expect("opens");
            let (world, how) =
                PersistentWorld::resume(Box::new(backend), composed()).expect("resumes");
            println!(
                "resumed head {} snapshot {}",
                how.head.raw(),
                how.snapshot.raw()
            );
            world
        }
        other => panic!("no role {other}"),
    }
    .snapshot_every(SNAPSHOT_INTERVAL);
    let done = world.highest_action_id().expect("reads").unwrap_or(0);
    for index in done..STEPS {
        let (at, intent) = intent(index, &ids);
        let _ = world.advance_to(t(at)).expect("advances");
        let answered = world.dispatch(&intent, t(at)).expect("answered");
        assert!(matches!(answered.result(), ActionResult::Accepted { .. }));
        println!("revision {}", world.revision().raw());
    }
    world.checkpoint().expect("checkpoints");
    println!("done {}", world.revision().raw());
}

/// The SIGKILL children's entry point: does nothing unless this binary was re-run as a child.
#[test]
fn sigkill_child_entry_point() {
    if let (Ok(role), Ok(directory)) = (std::env::var(ROLE), std::env::var(DIRECTORY)) {
        child(&role, Path::new(&directory));
    }
}

struct Ran {
    lines: Vec<String>,
    killed: bool,
}

fn run(role: &str, directory: &Path, kill_at: Option<u64>) -> Ran {
    let mut process = Command::new(std::env::current_exe().expect("this test binary"))
        .args([
            "sigkill_child_entry_point",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(ROLE, role)
        .env(DIRECTORY, directory)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("a child starts");
    let stdout = process.stdout.take().expect("piped");
    let mut lines = Vec::new();
    let mut sent = false;
    for line in BufReader::new(stdout).lines() {
        let line = line.expect("a line");
        let reached = line
            .strip_prefix("revision ")
            .and_then(|number| number.parse::<u64>().ok());
        lines.push(line);
        if let (Some(kill_at), Some(reached)) = (kill_at, reached)
            && reached >= kill_at
            && !sent
        {
            process.kill().expect("SIGKILL is delivered");
            sent = true;
        }
    }
    let status = process.wait().expect("reaped");
    if kill_at.is_none() {
        assert!(status.success(), "{role} failed: {status}");
    }
    Ran {
        lines,
        killed: status.signal() == Some(9),
    }
}

/// Stored rows, each with its revision or id.
type Stored = Vec<(u64, Vec<u8>)>;

/// Every journal row, fact and snapshot of a save, as stored.
fn rows(directory: &Path) -> (Stored, Stored, Stored) {
    let backend = SqliteBackend::open(directory, DURABILITY).expect("opens");
    let journal: Stored = backend
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
            facts.push((fact.id, fact.bytes));
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
                .expect("listed");
            (revision.raw(), bytes)
        })
        .collect();
    (journal, facts, snapshots)
}

fn head(directory: &Path) -> u64 {
    SqliteBackend::open(directory, DURABILITY)
        .expect("opens")
        .head()
        .expect("reads")
        .raw()
}

#[test]
fn a_configured_world_killed_mid_run_resumes_and_finishes_byte_for_byte() {
    let control = scratch("control");
    let ran = run("create", control.path(), None);
    assert!(
        ran.lines.iter().any(|line| line.starts_with("done ")),
        "the control finished"
    );
    let total = head(control.path());
    assert!(total > STEPS, "{total} revisions");
    let control_rows = rows(control.path());
    let advanced = control_rows
        .1
        .iter()
        .filter_map(|(_, bytes)| {
            let fact: EventEnvelope = format::decode(bytes, "fact").expect("decodes");
            let payload = fact.payload().payload_for::<Advanced>().ok()?;
            serde_json::from_slice::<Advanced>(payload).ok()
        })
        .filter(|advanced| advanced.by == 5)
        .count();
    assert_eq!(
        u64::try_from(advanced).expect("small"),
        STEPS,
        "every advance moved by the configured step"
    );

    let victim_dir = scratch("victim");
    let kill_at = total / 2 + 3;
    let victim = run("create", victim_dir.path(), Some(kill_at));
    let on_disk = head(victim_dir.path());
    assert!(victim.killed, "the victim died of SIGKILL");
    assert!(!victim.lines.iter().any(|line| line.starts_with("done")));
    assert!(
        on_disk >= kill_at && on_disk < total,
        "{on_disk} of {total}"
    );

    let survivor = run("resume", victim_dir.path(), None);
    let resumed = survivor
        .lines
        .iter()
        .find(|line| line.starts_with("resumed head "))
        .expect("the survivor reports its resume");
    assert!(
        resumed.starts_with(&format!("resumed head {on_disk} ")),
        "{resumed}"
    );
    assert_eq!(rows(victim_dir.path()), control_rows, "byte for byte");

    let verified = verify(
        &SqliteBackend::open(victim_dir.path(), DURABILITY).expect("opens"),
        composed(),
    )
    .expect("the whole history reproduces");
    assert_eq!(verified.revisions, total);
    println!(
        "[tuning-yard] control {total} revisions; killed at {kill_at} ({on_disk} on disk); resumed \
         and verified"
    );
}
