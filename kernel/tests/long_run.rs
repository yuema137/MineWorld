//! Hundreds of simulated days, headless, twice: the first real `AC-11` and `AC-12` evidence.
//!
//! A world assembled the way a World Pack loader assembles one — install systems, create the
//! people, state at genesis what is true of them — and then left to run. Nothing in it is a
//! fixture for the scheduler: the clock, the queue, the processes, the deferrals and the
//! interruptions are the real ones, and the only driver is `World::advance_to`.
//!
//! ```text
//! routine   owns `daily-activity`. At genesis each person begins one; at each activity's end the
//!           owner ends it, says so, and starts the next, for a seeded duration of one to eight
//!           hours. Asked to interrupt, it refuses while the person sleeps and otherwise ends the
//!           activity early and starts a leisure one.
//! pager     hears every activity end; for a seeded quarter of them it defers a page to a seeded
//!           person, minutes later. When the page falls due it asks routine to interrupt that
//!           person's activity, and records what it was told.
//! ```
//!
//! The seed is configuration the two systems are built with, and every random choice is a pure
//! function of the seed and an identity the world allocated — never of a clock, an address or a
//! hash — so the same seed must give the same history byte for byte, and a different seed a
//! different one.
//!
//! Run with `--nocapture` to see the wall time and the counts this run produced.

use std::collections::BTreeSet;

use mineworld_contracts::{
    EntityId, EntityKey, EntityType, Event, EventEnvelope, EventSchemaVersion, EventTypeId,
    ProcessTypeId, SystemId, Visibility, WorldTime,
};
use mineworld_kernel::{
    Emission, InterruptOutcome, InterruptRequest, KernelError, Process, ProcessKind, ProcessStart,
    ScheduleSnapshot, System, SystemDeclaration, SystemIdentity, SystemVersion, World, WorldView,
};
use serde::{Deserialize, Serialize};

/// The length of the run, and the meaning of "a day" — the test's, not the kernel's (`INV-12`).
const DAYS: i64 = 300;
const DAY: i64 = 86_400;
const HOUR: i64 = 3_600;
const PEOPLE: usize = 6;
/// The bounds on an activity's length, stated by this test's requirement.
const SHORTEST: i64 = HOUR;
const LONGEST: i64 = 8 * HOUR;

fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("a payload encodes")
}

fn t(seconds: i64) -> WorldTime {
    WorldTime::from_seconds(seconds)
}

/// SplitMix64: a pure function from a seed and an input to a well-mixed number. No state, so a
/// choice depends on exactly what it is given.
fn mix(seed: u64, input: u64) -> u64 {
    let mut z = seed ^ input.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn between(seed: u64, input: u64, low: i64, high: i64) -> i64 {
    let span = u64::try_from(high - low + 1).expect("a positive span");
    low + i64::try_from(mix(seed, input) % span).expect("fits")
}

// ---------------------------------------------------------------------------------------------
// routine
// ---------------------------------------------------------------------------------------------

struct Routine {
    seed: u64,
}

impl SystemIdentity for Routine {
    const ID: SystemId = SystemId::from_static("routine");
}

struct DailyActivity;

impl ProcessKind for DailyActivity {
    const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("daily-activity");
    type Owner = Routine;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Doing {
    Sleeping,
    Working,
    Resting,
}

#[derive(Serialize, Deserialize)]
struct RoutineBegun;

impl Event for RoutineBegun {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("routine-begun");
    const OWNER: SystemId = Routine::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

#[derive(Debug, Serialize, Deserialize)]
struct ActivityEnded {
    doing: Doing,
    interrupted: bool,
}

impl Event for ActivityEnded {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("activity-ended");
    const OWNER: SystemId = Routine::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

fn doing(process: &Process) -> Doing {
    serde_json::from_slice(process.state_for::<DailyActivity>().expect("an activity"))
        .expect("decodes")
}

impl Routine {
    /// Starts `person`'s next activity. Its length is a function of the seed and the identity of
    /// the process it follows (or the person, at the start), so it is the same on every replay.
    fn begin(
        &self,
        world: &mut WorldView<'_, Self>,
        person: EntityId,
        next: Doing,
        choice: u64,
    ) -> Result<(), KernelError> {
        let length = between(self.seed, choice, SHORTEST, LONGEST);
        world.start_process(
            ProcessStart::<DailyActivity>::new(encode(&next))
                .with_participants(vec![person])
                .ending_at(t(world.at().seconds() + length)),
        )?;
        Ok(())
    }

    fn ended(person: EntityId, doing: Doing, interrupted: bool) -> Emission {
        Emission::new::<ActivityEnded>(
            encode(&ActivityEnded { doing, interrupted }),
            Visibility::Public,
        )
        .about(vec![person])
    }
}

impl System for Routine {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .emitting::<RoutineBegun>()
            .emitting::<ActivityEnded>()
            .subscribing_to::<RoutineBegun>()
    }

    /// Genesis: each person's routine begins, asleep.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let person = event.subjects()[0];
        self.begin(world, person, Doing::Sleeping, person.raw())?;
        Ok(Vec::new())
    }

    fn wake(
        &self,
        world: &mut WorldView<'_, Self>,
        process: &Process,
    ) -> Result<Vec<Emission>, KernelError> {
        let person = process.participants()[0];
        let was = doing(process);
        world.end_process::<DailyActivity>(process.id())?;
        let next = match was {
            Doing::Sleeping => Doing::Working,
            Doing::Working => Doing::Resting,
            Doing::Resting => Doing::Sleeping,
        };
        self.begin(world, person, next, process.id().raw())?;
        Ok(vec![Self::ended(person, was, false)])
    }

    fn interrupt(
        &self,
        world: &mut WorldView<'_, Self>,
        request: &InterruptRequest,
    ) -> Result<Vec<Emission>, KernelError> {
        let Some(process) = world.read().process(request.process()).cloned() else {
            return Ok(Vec::new());
        };
        let was = doing(&process);
        if was == Doing::Sleeping {
            return Ok(Vec::new());
        }
        let person = process.participants()[0];
        world.end_process::<DailyActivity>(process.id())?;
        self.begin(world, person, Doing::Resting, process.id().raw() ^ 0xFF)?;
        Ok(vec![Self::ended(person, was, true)])
    }
}

// ---------------------------------------------------------------------------------------------
// pager
// ---------------------------------------------------------------------------------------------

struct Pager {
    seed: u64,
    people: Vec<EntityId>,
}

impl SystemIdentity for Pager {
    const ID: SystemId = SystemId::from_static("pager");
}

#[derive(Serialize, Deserialize)]
struct Paged {
    target: EntityId,
}

impl Event for Paged {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("paged");
    const OWNER: SystemId = Pager::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

#[derive(Serialize, Deserialize)]
struct PageAnswered {
    outcome: InterruptOutcome,
}

impl Event for PageAnswered {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("page-answered");
    const OWNER: SystemId = Pager::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl System for Pager {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([Routine::ID])
            .emitting::<Paged>()
            .emitting::<PageAnswered>()
            .subscribing_to::<ActivityEnded>()
            .subscribing_to::<Paged>()
    }

    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let choice = event.id().raw();
        if event.event_type() == &ActivityEnded::EVENT_TYPE {
            if !mix(self.seed, choice).is_multiple_of(4) {
                return Ok(Vec::new());
            }
            let index = usize::try_from(mix(self.seed, choice ^ 0xA5) % self.people.len() as u64)
                .expect("fits");
            let target = self.people[index];
            let minutes = between(self.seed, choice ^ 0x5A, 1, 120);
            world.defer(
                t(world.at().seconds() + minutes * 60),
                Emission::new::<Paged>(encode(&Paged { target }), Visibility::Public)
                    .about(vec![target]),
            )?;
            return Ok(Vec::new());
        }

        // A page has fallen due: ask the owner of the target's activity to interrupt it.
        let paged: Paged = serde_json::from_slice(event.payload().payload()).expect("paged");
        let activity = world
            .read()
            .processes()
            .find(|process| process.participants().contains(&paged.target))
            .map(Process::id);
        let outcome = match activity {
            Some(activity) => world.request_interrupt(activity)?,
            None => InterruptOutcome::NotRunning,
        };
        Ok(vec![Emission::new::<PageAnswered>(
            encode(&PageAnswered { outcome }),
            Visibility::Public,
        )])
    }
}

// ---------------------------------------------------------------------------------------------
// Assembly, and the run.
// ---------------------------------------------------------------------------------------------

/// Assembles the world as a pack loader would: systems, people, and genesis facts.
fn assemble(seed: u64) -> (World, Vec<EventEnvelope>) {
    let mut world = World::new();
    let mut people = Vec::new();
    let mut entities = Vec::new();
    for index in 0..PEOPLE {
        let key = EntityKey::new(format!("person-{index}")).expect("a key");
        entities.push(key);
    }
    // Entities are created before the systems that need their identities are built.
    for key in entities {
        people.push(
            world
                .create_entity(key, EntityType::Person)
                .expect("a person"),
        );
    }
    world.install(Routine { seed }).expect("routine installs");
    world
        .install(Pager {
            seed,
            people: people.clone(),
        })
        .expect("pager installs");
    let facts = people
        .iter()
        .map(|person| {
            Emission::new::<RoutineBegun>(encode(&RoutineBegun), Visibility::SystemInternal)
                .about(vec![*person])
        })
        .collect();
    let genesis = world.genesis(WorldTime::EPOCH, facts).expect("genesis");
    (world, genesis)
}

struct Run {
    history: Vec<EventEnvelope>,
    instants: u64,
    wall: std::time::Duration,
}

/// Runs a seeded world from genesis to `DAYS`, a day at a time.
fn run(seed: u64) -> Run {
    let started = std::time::Instant::now();
    let (mut world, mut history) = assemble(seed);
    let mut instants = 0;
    for day in 1..=DAYS {
        let advanced = world.advance_to(t(day * DAY)).expect("a day passes");
        instants += advanced.instants();
        history.extend(advanced.into_events());
    }
    Run {
        history,
        instants,
        wall: started.elapsed(),
    }
}

fn bytes(history: &[EventEnvelope]) -> Vec<u8> {
    serde_json::to_vec(history).expect("serializes")
}

fn count(history: &[EventEnvelope], kind: &str) -> usize {
    history
        .iter()
        .filter(|event| event.event_type().as_str() == kind)
        .count()
}

fn outcomes(history: &[EventEnvelope]) -> Vec<InterruptOutcome> {
    history
        .iter()
        .filter(|event| event.event_type().as_str() == "page-answered")
        .map(|event| {
            serde_json::from_slice::<PageAnswered>(event.payload().payload())
                .expect("decodes")
                .outcome
        })
        .collect()
}

/// `AC-11` and `AC-12`: three hundred simulated days run headless, cheaply, and the same seed
/// replays to the same bytes.
#[test]
fn three_hundred_days_run_headless_and_replay_identically_from_the_same_seed() {
    let first = run(42);
    let second = run(42);
    let history = &first.history;

    let ended = count(history, "activity-ended");
    let pages = count(history, "paged");
    let answered = outcomes(history);
    let refused = answered
        .iter()
        .filter(|outcome| **outcome == InterruptOutcome::Refused)
        .count();
    let interrupted = answered
        .iter()
        .filter(|outcome| **outcome == InterruptOutcome::Ended)
        .count();
    println!(
        "seed 42: {} days, {} facts ({} activity-ended, {} paged, {} answered: {} refused, {} \
         ended), {} instants, wall {:?} (second run {:?})",
        DAYS,
        history.len(),
        ended,
        pages,
        answered.len(),
        refused,
        interrupted,
        first.instants,
        first.wall,
        second.wall
    );

    // Locate before counting (`ARC-23`). First: the run covered the whole span, for everyone —
    // every person has an ended activity on every one of the days. A run that stalled after a week
    // would still produce thousands of facts; it would not produce this.
    let mut covered = BTreeSet::new();
    for event in history
        .iter()
        .filter(|event| event.event_type().as_str() == "activity-ended")
    {
        covered.insert((event.at().seconds() / DAY, event.subjects()[0]));
    }
    let days_covered: BTreeSet<i64> = covered.iter().map(|(day, _)| *day).collect();
    assert_eq!(
        days_covered.len(),
        usize::try_from(DAYS).expect("fits"),
        "every simulated day has activity"
    );
    assert_eq!(
        covered.len(),
        usize::try_from(DAYS).expect("fits") * PEOPLE,
        "every person, every day"
    );
    let last = history.last().expect("a history").at();
    assert!(
        last.seconds() > (DAYS - 1) * DAY,
        "the run reached its final day: last fact at {last}"
    );

    // Then the count, with a bound from the requirement, not the implementation: no activity lasts
    // longer than LONGEST, and an interruption only shortens one, so each person ends at least
    // DAY / LONGEST activities a day.
    let at_least = usize::try_from(DAYS * (DAY / LONGEST)).expect("fits") * PEOPLE;
    assert!(
        ended >= at_least,
        "{ended} activity endings, fewer than the {at_least} the 8-hour bound guarantees"
    );

    // Both of the owner's answers occurred, so the run exercised interruption rather than a
    // degenerate path: it refused while people slept, and ended activities otherwise.
    assert!(answered.contains(&InterruptOutcome::Refused));
    assert!(answered.contains(&InterruptOutcome::Ended));
    assert_eq!(
        pages,
        answered.len(),
        "every page that fell due was answered"
    );

    // Idle time was skipped: the world visited only instants where something was due, a tiny
    // fraction of the seconds simulated.
    let seconds = u64::try_from(DAYS * DAY).expect("fits");
    assert!(
        first.instants * 100 < seconds,
        "{} instants for {seconds} simulated seconds",
        first.instants
    );

    // And the decisive claim: the same seed, the same history, byte for byte.
    assert_eq!(first.instants, second.instants);
    assert_eq!(bytes(&first.history), bytes(&second.history));
}

/// The seed is not decoration: a different seed makes a different world. Without this, two equal
/// histories could mean the seed was never read.
#[test]
fn a_different_seed_makes_a_different_history() {
    let forty_two = run(42);
    let forty_three = run(43);
    assert_ne!(bytes(&forty_two.history), bytes(&forty_three.history));
}

/// Saved mid-run and restored into a world assembled the same way, the world continues exactly as
/// the uninterrupted run does: the same facts, with the same identities, at the same instants, from
/// the same causes. Every piece of this world's state is in its processes and its schedule, which
/// is what S4 saves; component state is S5's.
#[test]
fn a_world_saved_mid_run_continues_exactly_as_the_uninterrupted_run() {
    const SAVE_ON_DAY: i64 = 137;
    let straight = run(42);

    let (mut world, _) = assemble(42);
    let _ = world
        .advance_to(t(SAVE_ON_DAY * DAY))
        .expect("advances to the save point");
    let saved: ScheduleSnapshot = serde_json::from_slice(
        &serde_json::to_vec(&world.schedule_snapshot()).expect("serializes"),
    )
    .expect("parses");
    assert!(
        !saved.processes.is_empty() && !saved.entries.is_empty(),
        "the save point has work in flight: {} processes, {} entries",
        saved.processes.len(),
        saved.entries.len()
    );

    let (mut restored, _) = assemble(42);
    restored.restore_schedule(saved).expect("restores");
    let mut continued = Vec::new();
    for day in SAVE_ON_DAY + 1..=DAYS {
        continued.extend(
            restored
                .advance_to(t(day * DAY))
                .expect("a day passes")
                .into_events(),
        );
    }

    let expected: Vec<EventEnvelope> = straight
        .history
        .iter()
        .filter(|event| event.at().seconds() > SAVE_ON_DAY * DAY)
        .cloned()
        .collect();
    assert!(
        expected.len() > 1_000,
        "the continued stretch is substantial: {} facts",
        expected.len()
    );
    assert_eq!(bytes(&continued), bytes(&expected));
}
