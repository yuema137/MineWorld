//! IC-1 — a world survives `SIGKILL`, and continues byte for byte as if it never died (step-06 §5).
//!
//! ```text
//! parent      for each scenario:
//!               control   a child runs the whole script into its own save, uninterrupted
//!               for each kill point K (early, middle, late):
//!                 victim    a child runs the script into a fresh save; once it has printed
//!                           "revision K" the parent SIGKILLs it, while it is still writing
//!                 survivor  a NEW child process opens the same file, resumes it, and finishes the
//!                           script from the first input the save does not hold
//!                 compare   facts, journal and snapshots of the survivor's file and the control's,
//!                           byte for byte; and verify() the survivor's file from genesis
//! ```
//!
//! Two scenarios. **cafe**: the repository's own `worlds/social-cafe`, assembled by the pack loader,
//! driven by `move` and `talk` requests (some refused for distance by the movement and conversation
//! systems) at
//! `Durability::PowerLoss`, the hosted setting. **clock**: a test-local world whose state lives in
//! processes, deferrals and interruptions, driven only by the clock for thirty simulated days at
//! `Durability::ProcessCrash`.
//!
//! What would make this pass without proving anything, and how each is excluded (`ARC-23`):
//!
//! ```text
//! the child was never killed           its exit status must be SIGKILL, it must not have printed
//!                                      "done", and the file's head must be short of the control's
//! the survivor did not read the file   it reports the head, snapshot and tail it resumed from; the
//!                                      head must be the one on disk, and a survivor that ignored the
//!                                      file could not create a save over an existing one
//! the tail was empty every time        at least one kill point per scenario resumes with a tail > 0
//! two empty logs agreeing              the control's fact and revision counts are located first
//! ```
//!
//! This file is a program (`harness = false`), because the same binary is both the parent that kills
//! and the child that is killed.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use mineworld_contracts::{
    ActionId, ActionIntent, ActionRequest, ActionResult, EntityId, EntityKey, EntityType, Event,
    EventEnvelope, EventSchemaVersion, EventTypeId, ProcessId, ProcessTypeId, Rejection, SystemId,
    Visibility, WorldTime,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, Process, ProcessKind, ProcessPhase, ProcessStart, System,
    SystemDeclaration, SystemIdentity, SystemVersion, World, WorldView, owned_component,
};
use mineworld_persistence::{
    Creation, Durability, PersistenceBackend, PersistentWorld, SqliteBackend, verify,
};
use mineworld_worldpack::WorldPack;
use serde::{Deserialize, Serialize};
use serde_json::json;

const ROLE: &str = "MINEWORLD_KILL_TEST_ROLE";
const SCENARIO: &str = "MINEWORLD_KILL_TEST_SCENARIO";
const DIRECTORY: &str = "MINEWORLD_KILL_TEST_DIR";

const INSTANCE: u128 = 0x0000_0000_0000_0000_0000_0000_c0ff_ee01;
const SNAPSHOT_INTERVAL: u64 = 32;

fn t(seconds: i64) -> WorldTime {
    WorldTime::from_seconds(seconds)
}

fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("encodes")
}

/// A deterministic mix of two numbers: the scenario's only source of variety, the same in every
/// process.
fn mix(a: u64, b: u64) -> u64 {
    let mut x = a
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(b.wrapping_mul(0xC2B2_AE3D_27D4_EB4F));
    x ^= x >> 29;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^ (x >> 32)
}

// =============================================================================================
// the clock scenario's world
// =============================================================================================

struct Routine;

impl SystemIdentity for Routine {
    const ID: SystemId = SystemId::from_static("routine");
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct Days {
    activities: u32,
    interrupted: u32,
}

owned_component! {
    component = Days,
    owner = Routine,
    component_type = "days",
    schema_version = 1,
}

struct Activity;

impl ProcessKind for Activity {
    const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("activity");
    type Owner = Routine;
}

#[derive(Serialize, Deserialize)]
struct Woke;

impl Event for Woke {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("woke");
    const OWNER: SystemId = Routine::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

#[derive(Serialize, Deserialize)]
struct Ended {
    interrupted: bool,
}

impl Event for Ended {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("activity-ended");
    const OWNER: SystemId = Routine::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

fn ended(person: EntityId, interrupted: bool) -> Emission {
    Emission::new::<Ended>(encode(&Ended { interrupted }), Visibility::Public).about(vec![person])
}

/// Whether an activity is sleep, which refuses interruption: the owner's own bytes.
fn asleep(process: &Process) -> bool {
    serde_json::from_slice(process.state_for::<Activity>().expect("an activity")).expect("a bool")
}

impl System for Routine {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<Days>()
            .emitting::<Woke>()
            .emitting::<Ended>()
            .subscribing_to::<Woke>()
            .subscribing_to::<Ended>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Days>()
    }

    /// Records an ended activity and starts the next one, one to eight hours long, sometimes sleep.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let person = event.subjects()[0];
        if *event.event_type() == Ended::EVENT_TYPE {
            let interrupted = serde_json::from_slice::<Ended>(event.payload().payload())
                .expect("ended")
                .interrupted;
            let mut days = world
                .read()
                .component::<Days>(person)
                .cloned()
                .unwrap_or_default();
            days.activities += 1;
            days.interrupted += u32::from(interrupted);
            world.insert(person, days)?;
        }
        let roll = mix(event.id().raw(), person.raw());
        let hours = 1 + i64::try_from(roll % 8).expect("small");
        world.start_process(
            ProcessStart::<Activity>::new(encode(&roll.is_multiple_of(5)))
                .with_participants(vec![person])
                .ending_at(t(world.at().seconds() + hours * 3_600)),
        )?;
        Ok(Vec::new())
    }

    fn wake(
        &self,
        world: &mut WorldView<'_, Self>,
        process: &Process,
    ) -> Result<Vec<Emission>, KernelError> {
        world.end_process::<Activity>(process.id())?;
        Ok(vec![ended(process.participants()[0], false)])
    }

    /// Sleep refuses; anything else ends early.
    fn interrupt(
        &self,
        world: &mut WorldView<'_, Self>,
        request: &mineworld_kernel::InterruptRequest,
    ) -> Result<Vec<Emission>, KernelError> {
        let Some(process) = world.read().process(request.process()).cloned() else {
            return Ok(Vec::new());
        };
        if asleep(&process) {
            return Ok(Vec::new());
        }
        world.end_process::<Activity>(process.id())?;
        Ok(vec![ended(process.participants()[0], true)])
    }
}

struct Pager;

impl SystemIdentity for Pager {
    const ID: SystemId = SystemId::from_static("pager");
}

#[derive(Serialize, Deserialize)]
struct Paged;

impl Event for Paged {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("paged");
    const OWNER: SystemId = Pager::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// The running activity of a person, if any.
fn activity_of(world: &WorldView<'_, Pager>, person: EntityId) -> Option<ProcessId> {
    world
        .read()
        .processes()
        .find(|process| {
            process.participants().first() == Some(&person)
                && process.phase() == ProcessPhase::Running
        })
        .map(Process::id)
}

impl System for Pager {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .emitting::<Paged>()
            .subscribing_to::<Ended>()
            .subscribing_to::<Paged>()
    }

    /// After every third ended activity, pages that person ten to thirty minutes later; a page asks
    /// routine to interrupt whatever they are doing then.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let person = event.subjects()[0];
        if *event.event_type() == Paged::EVENT_TYPE {
            if let Some(activity) = activity_of(world, person) {
                world.request_interrupt(activity)?;
            }
            return Ok(Vec::new());
        }
        let roll = mix(event.id().raw(), 7);
        if roll.is_multiple_of(3) {
            let delay = 600 + i64::try_from(roll % 1_200).expect("small");
            world.defer(
                t(world.at().seconds() + delay),
                Emission::new::<Paged>(encode(&Paged), Visibility::Public).about(vec![person]),
            )?;
        }
        Ok(Vec::new())
    }
}

const CLOCK_PEOPLE: usize = 6;
const CLOCK_DAYS: i64 = 30;

fn clock_composed() -> World {
    let mut world = World::new();
    world.install(Routine).expect("routine installs");
    world.install(Pager).expect("pager installs");
    world
}

/// The clock scenario's inputs: an advance every simulated hour for thirty days.
fn clock_script() -> Vec<i64> {
    (1..=CLOCK_DAYS * 24).map(|hour| hour * 3_600).collect()
}

// =============================================================================================
// the cafe scenario
// =============================================================================================

fn social_cafe() -> WorldPack {
    WorldPack::read(Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../worlds/social-cafe"
    )))
    .expect("the repository's own pack reads")
}

const CAFE_STEPS: u64 = 300;
const CAFE_PEOPLE: [&str; 4] = ["alice", "bob", "visitor", "wanderer"];
/// Where `worlds/social-cafe` seats each of [`CAFE_PEOPLE`], in millimetres (its `people/*.yaml`).
const CAFE_SEATS: [(i32, i32); 4] = [(6_000, 8_000), (4_500, 6_100), (1_610, 600), (7_110, 3_900)];

/// The cafe scenario's step `index`: at `20 * (index + 1)` seconds, request `index + 1` — a `move`
/// near where the actor was seated, which the movement system refuses when it is more than a stride
/// from where they are, or a `talk` to the next person, which the conversation system refuses when
/// they are out of reach.
fn cafe_intent(index: u64, ids: &BTreeMap<EntityKey, EntityId>) -> (i64, ActionIntent) {
    let id = |key: &str| ids[&EntityKey::new(key).expect("a key")];
    let actor = id(CAFE_PEOPLE[usize::try_from(index % 4).expect("small")]);
    let other = id(CAFE_PEOPLE[usize::try_from((index + 1) % 4).expect("small")]);
    let roll = mix(index, 11);
    // The action payload is the owning system's JSON, carried as its bytes — exactly what the server
    // makes of a client's frame (`server/src/protocol.rs`, `WirePayload`).
    let request = if index.is_multiple_of(3) {
        // A point in the 2.4 m square centred on where the pack seats this person, so a move lands
        // within the server's 2 m stride of the person's current position often and outside it
        // often: both answers are journaled and replayed (counted by the parent, `ARC-23`).
        let (home_x, home_y) = CAFE_SEATS[usize::try_from(index % 4).expect("small")];
        let x = home_x + i32::try_from(roll % 2_400).expect("small") - 1_200;
        let y = home_y + i32::try_from((roll >> 16) % 2_400).expect("small") - 1_200;
        let payload = encode(&json!({ "to": {
            "place": { "entity": id("cafe").to_string(), "entity_type": "place" },
            "local": { "x": x, "y": y, "z": 0 }, "facing": null } }));
        json!({
            "actor": actor, "action_type": "move", "target": null, "actor_location": null,
            "payload": { "action_type": "move", "payload": payload },
        })
    } else {
        let payload = encode(&json!({ "utterance": format!("line {index}") }));
        json!({
            "actor": actor, "action_type": "talk", "target": other, "actor_location": null,
            "payload": { "action_type": "talk", "payload": payload },
        })
    };
    let request: ActionRequest = serde_json::from_value(request).expect("a well-formed request");
    let at = 20 * i64::try_from(index + 1).expect("small");
    (
        at,
        ActionIntent::allocate(request, ActionId::from_raw(index + 1), t(at)),
    )
}

// =============================================================================================
// the children
// =============================================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scenario {
    Cafe,
    Clock,
}

impl Scenario {
    const fn name(self) -> &'static str {
        match self {
            Self::Cafe => "cafe",
            Self::Clock => "clock",
        }
    }

    fn named(name: &str) -> Self {
        match name {
            "cafe" => Self::Cafe,
            "clock" => Self::Clock,
            other => panic!("no scenario {other}"),
        }
    }

    const fn durability(self) -> Durability {
        match self {
            Self::Cafe => Durability::PowerLoss,
            Self::Clock => Durability::ProcessCrash,
        }
    }

    fn composed(self) -> World {
        match self {
            Self::Cafe => social_cafe().compose().expect("composes").world,
            Self::Clock => clock_composed(),
        }
    }

    /// A new save, begun.
    fn create(self, directory: &Path) -> PersistentWorld {
        let backend =
            SqliteBackend::create(directory, self.durability()).expect("a new save is created");
        let (world, facts) = match self {
            Self::Cafe => {
                let assembled = social_cafe().assemble().expect("assembles");
                (assembled.world, assembled.facts)
            }
            Self::Clock => {
                let mut world = clock_composed();
                let facts = (0..CLOCK_PEOPLE)
                    .map(|index| {
                        let person = world
                            .create_entity(
                                EntityKey::new(format!("person-{index}")).expect("a key"),
                                EntityType::Person,
                            )
                            .expect("created");
                        Emission::new::<Woke>(encode(&Woke), Visibility::Public).about(vec![person])
                    })
                    .collect();
                (world, facts)
            }
        };
        let (persisted, _) = PersistentWorld::create(
            Box::new(backend),
            world,
            Creation {
                instance: INSTANCE,
                pack: self.name().to_owned(),
                at: WorldTime::EPOCH,
                facts,
            },
        )
        .expect("the world begins");
        persisted.snapshot_every(SNAPSHOT_INTERVAL)
    }

    /// Drives `world` through the rest of the script — every input the save does not hold yet —
    /// printing each committed revision.
    fn finish(self, world: &mut PersistentWorld) {
        let report = |world: &PersistentWorld| println!("revision {}", world.revision().raw());
        match self {
            Self::Cafe => {
                let ids = social_cafe().assemble().expect("assembles").ids;
                let done = world.highest_action_id().expect("reads").unwrap_or(0);
                for index in done..CAFE_STEPS {
                    let (at, intent) = cafe_intent(index, &ids);
                    let _ = world.advance_to(t(at)).expect("advances");
                    report(world);
                    let answered = world.dispatch(&intent, t(at)).expect("answered");
                    if intent.action_type().as_str() == "move" {
                        // Counted by the parent, so the claim that both answers occur is located.
                        match answered.result() {
                            ActionResult::Accepted { .. } => println!("move accepted"),
                            ActionResult::Rejected(Rejection::TooFarAway) => {
                                println!("move too-far-away");
                            }
                            other => println!("move {other:?}"),
                        }
                    }
                    report(world);
                }
            }
            Self::Clock => {
                let now = world.world().now().seconds();
                for until in clock_script().into_iter().filter(|until| *until > now) {
                    let _ = world.advance_to(t(until)).expect("advances");
                    report(world);
                }
            }
        }
    }
}

/// What a child does, selected by the environment.
fn child(role: &str) {
    let scenario = Scenario::named(&std::env::var(SCENARIO).expect("a scenario"));
    let directory = PathBuf::from(std::env::var(DIRECTORY).expect("a directory"));
    let mut world = match role {
        "create" => scenario.create(&directory),
        "resume" => {
            let backend =
                SqliteBackend::open(&directory, scenario.durability()).expect("the save opens");
            let (world, how) =
                PersistentWorld::resume(Box::new(backend), scenario.composed()).expect("resumes");
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
    scenario.finish(&mut world);
    world.checkpoint().expect("checkpoints");
    println!("done {}", world.revision().raw());
}

// =============================================================================================
// the parent
// =============================================================================================

/// How a child ended, and everything it printed.
struct Ran {
    lines: Vec<String>,
    killed: bool,
}

/// Runs a child to completion, or kills it with SIGKILL as soon as it reports revision `kill_at`.
fn run(role: &str, scenario: Scenario, directory: &Path, kill_at: Option<u64>) -> Ran {
    let mut process = Command::new(std::env::current_exe().expect("this program"))
        .env(ROLE, role)
        .env(SCENARIO, scenario.name())
        .env(DIRECTORY, directory)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("a child starts");
    let stdout = process.stdout.take().expect("piped");
    let mut lines = Vec::new();
    let mut sent_kill = false;
    for line in BufReader::new(stdout).lines() {
        let line = line.expect("a line");
        let reached = line
            .strip_prefix("revision ")
            .and_then(|number| number.parse::<u64>().ok());
        lines.push(line);
        if let (Some(kill_at), Some(reached)) = (kill_at, reached)
            && reached >= kill_at
            && !sent_kill
        {
            process.kill().expect("SIGKILL is delivered");
            sent_kill = true;
        }
    }
    let status = process.wait().expect("the child is reaped");
    let killed = status.signal() == Some(9);
    if kill_at.is_none() {
        assert!(
            status.success(),
            "{role} {} failed: {status}",
            scenario.name()
        );
    }
    Ran { lines, killed }
}

/// Every row of a save, as stored, for a byte-for-byte comparison.
#[derive(Debug, PartialEq, Eq)]
struct Rows {
    journal: Vec<(i64, i64, Option<i64>, Vec<u8>)>,
    facts: Vec<(i64, i64, Vec<u8>)>,
    snapshots: Vec<(i64, Vec<u8>)>,
}

fn rows(directory: &Path) -> Rows {
    let connection =
        rusqlite::Connection::open(SqliteBackend::file(directory)).expect("the save opens");
    let journal = connection
        .prepare("SELECT revision, at, action_id, entry FROM journal ORDER BY revision")
        .expect("prepares")
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .expect("queries")
        .collect::<Result<_, _>>()
        .expect("reads");
    let facts = connection
        .prepare("SELECT event_id, revision, fact FROM facts ORDER BY event_id")
        .expect("prepares")
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .expect("queries")
        .collect::<Result<_, _>>()
        .expect("reads");
    let snapshots = connection
        .prepare("SELECT revision, snapshot FROM snapshots ORDER BY revision")
        .expect("prepares")
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .expect("queries")
        .collect::<Result<_, _>>()
        .expect("reads");
    Rows {
        journal,
        facts,
        snapshots,
    }
}

fn head(directory: &Path, scenario: Scenario) -> u64 {
    SqliteBackend::open(directory, scenario.durability())
        .expect("opens")
        .head()
        .expect("reads")
        .raw()
}

fn scratch(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("mineworld-kill-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("a scratch directory");
    path
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

fn scenario(scenario: Scenario) {
    let started = std::time::Instant::now();
    let control_dir = scratch(&format!("{}-control", scenario.name()));
    let control = run("create", scenario, &control_dir, None);
    assert!(
        control
            .lines
            .last()
            .is_some_and(|line| line.starts_with("done "))
    );
    let total = head(&control_dir, scenario);
    let control_rows = rows(&control_dir);
    // Located first: a long history with many facts, so equality below is not two empty logs.
    assert!(
        total > 300,
        "{}: the control run holds {total} revisions",
        scenario.name()
    );
    // Floors from what the run shows, not from the code: in the cafe every third request is a `move`,
    // and each one accepted states one fact — counted from the control child's own report, which also
    // shows that refusals for distance occur and are therefore journaled and replayed too; in the
    // clock world an activity lasts at most eight hours, so every person ends at least three a day.
    let floor = match scenario {
        Scenario::Cafe => {
            let count = |what: &str| {
                control
                    .lines
                    .iter()
                    .filter(|line| line.as_str() == what)
                    .count() as u64
            };
            let (accepted, refused) = (count("move accepted"), count("move too-far-away"));
            println!(
                "[cafe] control: {} moves, {accepted} accepted, {refused} refused too-far-away",
                CAFE_STEPS / 3
            );
            assert_eq!(
                accepted + refused,
                CAFE_STEPS / 3,
                "every move is answered one of the two ways"
            );
            assert!(
                accepted > 0 && refused > 0,
                "both answers occur: {accepted} accepted, {refused} refused"
            );
            accepted
        }
        Scenario::Clock => CLOCK_DAYS.unsigned_abs() * 3 * CLOCK_PEOPLE as u64,
    };
    assert!(
        control_rows.facts.len() as u64 >= floor,
        "{}: the control run logged {} facts, below the floor {floor}",
        scenario.name(),
        control_rows.facts.len()
    );
    println!(
        "[{}] control: {total} revisions, {} facts, {} snapshots",
        scenario.name(),
        control_rows.facts.len(),
        control_rows.snapshots.len()
    );

    let mut tails = Vec::new();
    for (label, kill_at) in [
        ("early", total / 5),
        ("middle", total / 2 + 3),
        ("late", total * 4 / 5 + 7),
    ] {
        let dir = scratch(&format!("{}-{label}", scenario.name()));
        let victim = run("create", scenario, &dir, Some(kill_at));
        let on_disk = head(&dir, scenario);
        assert!(victim.killed, "{label}: the victim died of SIGKILL");
        assert!(
            !victim.lines.iter().any(|line| line.starts_with("done")),
            "{label}: the victim never finished"
        );
        assert!(
            on_disk >= kill_at && on_disk < total,
            "{label}: the save stops mid-run ({on_disk}, killed at {kill_at}, full run {total})"
        );
        let last_printed = victim
            .lines
            .iter()
            .rev()
            .find_map(|line| line.strip_prefix("revision ")?.parse::<u64>().ok())
            .expect("it printed revisions");

        let survivor = run("resume", scenario, &dir, None);
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
        assert!(
            snapshot <= on_disk && (snapshot == 1 || snapshot.is_multiple_of(SNAPSHOT_INTERVAL)),
            "{label}: the snapshot is genesis or a scheduled one ({snapshot})"
        );
        println!(
            "[{}] {label}: kill sent at revision {kill_at}, the victim last printed {last_printed}, \
             {on_disk} committed; survivor restored snapshot {snapshot} and re-executed {replayed} \
             revisions ({facts} facts)",
            scenario.name()
        );
        tails.push(replayed);

        assert_eq!(
            rows(&dir),
            control_rows,
            "{label}: the survivor's save is the control's, byte for byte"
        );
        let verified = verify(
            &SqliteBackend::open(&dir, scenario.durability()).expect("opens"),
            scenario.composed(),
        )
        .expect("the survivor's whole history reproduces");
        assert_eq!(verified.revisions, total);
        let _ = std::fs::remove_dir_all(&dir);
    }
    assert!(
        tails.iter().any(|tail| *tail > 0),
        "{}: at least one resume re-executed a tail: {tails:?}",
        scenario.name()
    );
    let _ = std::fs::remove_dir_all(&control_dir);
    println!(
        "[{}] PASS in {:.1} s",
        scenario.name(),
        started.elapsed().as_secs_f64()
    );
}

fn main() {
    if let Ok(role) = std::env::var(ROLE) {
        child(&role);
        return;
    }
    // `cargo test` passes its own arguments (a filter, `--nocapture`, …); a filter naming neither
    // scenario skips this program, so `cargo test -p mineworld-persistence some_other_test` stays fast.
    let filter = std::env::args()
        .skip(1)
        .find(|argument| !argument.starts_with('-'));
    for each in [Scenario::Cafe, Scenario::Clock] {
        if filter
            .as_deref()
            .is_none_or(|filter| "kill_and_resume".contains(filter) || each.name().contains(filter))
        {
            scenario(each);
        }
    }
}
