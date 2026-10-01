//! Processes, from outside the kernel: `docs/CORE_CONCEPTS.md` §10's dinner and phone call, built
//! with the types as they are.
//!
//! ```text
//! dining   owns `having-dinner`; `sit-down` starts one for an hour, `serve` moves it to the next
//!          course; at its end it finishes it; asked to interrupt, it decides by course:
//!              starters → end it      main → suspend it      dessert → refuse
//! phone    `call` rings someone; reacting to its own ring, it asks for the callee's dinner to be
//!          interrupted, and records what it was told
//! ```
//!
//! The claims, from step-04 §3 C3:
//!
//! ```text
//! the owner ends its own process at its expected end; the fact names the process   — INV-3, AC-9
//! a refused interruption leaves the process running, and the requester is told      — SD-6
//! an ended or suspended process is not woken at its old end
//! an uninterruptible process is answered without consulting the owner
//! a non-owner's attempt to change a process is refused by name (and does not compile — see
//!   tests/compile_fail/a_system_cannot_end_or_start_another_systems_process.rs)    — INV-7, F-8
//! a process survives a save and restore and keeps its scheduled end                 — S5 readiness
//! ```
//!
//! Neither system knows what dinner is to the kernel: the kernel stores a process of type
//! `having-dinner` with bytes only `dining` reads (`INV-12`).

use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionResult, ActionTypeId, Causation, EntityId,
    EntityKey, EntityType, Event, EventEnvelope, EventSchemaVersion, EventTypeId, ProcessId,
    ProcessTypeId, SystemId, Visibility, WorldTime,
};
use mineworld_kernel::{
    Emission, InterruptOutcome, InterruptRequest, KernelError, Process, ProcessKind, ProcessPhase,
    ProcessStart, ScheduleSnapshot, System, SystemDeclaration, SystemIdentity, SystemVersion,
    World, WorldView,
};
use serde::{Deserialize, Serialize};

fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("a payload encodes")
}

fn t(seconds: i64) -> WorldTime {
    WorldTime::from_seconds(seconds)
}

/// How long a dinner is expected to last.
const DINNER: i64 = 3_600;

// ---------------------------------------------------------------------------------------------
// dining
// ---------------------------------------------------------------------------------------------

struct Dining;

impl SystemIdentity for Dining {
    const ID: SystemId = SystemId::from_static("dining");
}

struct Dinner;

impl ProcessKind for Dinner {
    const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("having-dinner");
    type Owner = Dining;
}

/// What `dining` keeps for a dinner: which course it is on. Only `dining` reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Course {
    Starters,
    Main,
    Dessert,
}

#[derive(Serialize, Deserialize)]
struct SitDown {
    with: EntityId,
    duration: i64,
    uninterruptible: bool,
}

impl Action for SitDown {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("sit-down");
    const OWNER: SystemId = Dining::ID;
}

#[derive(Serialize, Deserialize)]
struct Serve {
    dinner: ProcessId,
}

impl Action for Serve {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("serve");
    const OWNER: SystemId = Dining::ID;
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct DinnerStarted {
    dinner: ProcessId,
}

impl Event for DinnerStarted {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("dinner-started");
    const OWNER: SystemId = Dining::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct DinnerEnded {
    dinner: ProcessId,
    interrupted: bool,
}

impl Event for DinnerEnded {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("dinner-ended");
    const OWNER: SystemId = Dining::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

fn course_of(process: &Process) -> Course {
    serde_json::from_slice(process.state_for::<Dinner>().expect("a dinner")).expect("a course")
}

fn ended(dinner: ProcessId, interrupted: bool) -> Emission {
    Emission::new::<DinnerEnded>(
        encode(&DinnerEnded {
            dinner,
            interrupted,
        }),
        Visibility::Public,
    )
}

impl System for Dining {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .providing::<SitDown>()
            .providing::<Serve>()
            .emitting::<DinnerStarted>()
            .emitting::<DinnerEnded>()
    }

    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        if *intent.action_type() == Serve::ACTION_TYPE {
            let serve: Serve = serde_json::from_slice(intent.payload().payload()).expect("serve");
            let next = match world.read().process(serve.dinner).map(course_of) {
                Some(Course::Starters) => Course::Main,
                _ => Course::Dessert,
            };
            world.set_process_state::<Dinner>(serve.dinner, encode(&next))?;
            return Ok(Vec::new());
        }

        let sit: SitDown = serde_json::from_slice(intent.payload().payload()).expect("sit-down");
        let mut start = ProcessStart::<Dinner>::new(encode(&Course::Starters))
            .with_participants(vec![intent.actor(), sit.with])
            .ending_at(t(world.at().seconds() + sit.duration));
        if sit.uninterruptible {
            start = start.uninterruptible();
        }
        let dinner = world.start_process(start)?;
        Ok(vec![Emission::new::<DinnerStarted>(
            encode(&DinnerStarted { dinner }),
            Visibility::Public,
        )])
    }

    fn wake(
        &self,
        world: &mut WorldView<'_, Self>,
        process: &Process,
    ) -> Result<Vec<Emission>, KernelError> {
        world.end_process::<Dinner>(process.id())?;
        Ok(vec![ended(process.id(), false)])
    }

    fn interrupt(
        &self,
        world: &mut WorldView<'_, Self>,
        request: &InterruptRequest,
    ) -> Result<Vec<Emission>, KernelError> {
        let dinner = request.process();
        let course = world.read().process(dinner).map(course_of);
        match course {
            Some(Course::Starters) => {
                world.end_process::<Dinner>(dinner)?;
                Ok(vec![ended(dinner, true)])
            }
            Some(Course::Main) => {
                world.suspend_process::<Dinner>(dinner)?;
                Ok(Vec::new())
            }
            Some(Course::Dessert) | None => Ok(Vec::new()),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// phone
// ---------------------------------------------------------------------------------------------

struct Phone;

impl SystemIdentity for Phone {
    const ID: SystemId = SystemId::from_static("phone");
}

/// A process kind the phone owns. It exists so the phone can try to use its *own* kind against the
/// dining system's process — which compiles, and must be refused at run time (F-8).
struct Ringing;

impl ProcessKind for Ringing {
    const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("ringing");
    type Owner = Phone;
}

#[derive(Serialize, Deserialize)]
struct Call {
    callee: EntityId,
}

impl Action for Call {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("call");
    const OWNER: SystemId = Phone::ID;
}

/// Hangs up on someone's dinner directly — the attempt `INV-7` forbids, written with the phone's
/// own process kind so that it compiles.
#[derive(Serialize, Deserialize)]
struct Barge {
    dinner: ProcessId,
}

impl Action for Barge {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("barge");
    const OWNER: SystemId = Phone::ID;
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Rang {
    callee: EntityId,
}

impl Event for Rang {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("phone-rang");
    const OWNER: SystemId = Phone::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Answered {
    outcome: InterruptOutcome,
}

impl Event for Answered {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("call-answered");
    const OWNER: SystemId = Phone::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl System for Phone {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .providing::<Call>()
            .providing::<Barge>()
            .emitting::<Rang>()
            .emitting::<Answered>()
            .subscribing_to::<Rang>()
    }

    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        if *intent.action_type() == Barge::ACTION_TYPE {
            let barge: Barge = serde_json::from_slice(intent.payload().payload()).expect("barge");
            world.end_process::<Ringing>(barge.dinner)?;
            return Ok(Vec::new());
        }
        let call: Call = serde_json::from_slice(intent.payload().payload()).expect("call");
        Ok(vec![Emission::new::<Rang>(
            encode(&Rang {
                callee: call.callee,
            }),
            Visibility::Public,
        )])
    }

    /// Hearing its own ring, the phone looks for the callee's dinner — reads are open — and asks
    /// its owner to interrupt it. It changes nothing itself; it records what it was told.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let rang: Rang = serde_json::from_slice(event.payload().payload()).expect("rang");
        let dinner = world
            .read()
            .processes()
            .find(|process| process.participants().contains(&rang.callee))
            .map(Process::id);
        let outcome = match dinner {
            Some(dinner) => world.request_interrupt(dinner)?,
            None => InterruptOutcome::NotRunning,
        };
        Ok(vec![Emission::new::<Answered>(
            encode(&Answered { outcome }),
            Visibility::Public,
        )])
    }
}

// ---------------------------------------------------------------------------------------------
// The world they compose.
// ---------------------------------------------------------------------------------------------

struct Bench {
    world: World,
    alice: EntityId,
    bob: EntityId,
    next_action: u64,
}

impl Bench {
    fn new() -> Self {
        let mut world = World::new();
        world.install(Dining).expect("dining installs");
        world.install(Phone).expect("phone installs");
        let alice = world
            .create_entity(EntityKey::new("alice").expect("a key"), EntityType::Person)
            .expect("alice");
        let bob = world
            .create_entity(EntityKey::new("bob").expect("a key"), EntityType::Person)
            .expect("bob");
        Self {
            world,
            alice,
            bob,
            next_action: 1,
        }
    }

    fn act<A: Action + Serialize>(
        &mut self,
        actor: EntityId,
        payload: &A,
        at: WorldTime,
    ) -> Result<(ActionResult, Vec<EventEnvelope>), KernelError> {
        let id = ActionId::from_raw(self.next_action);
        self.next_action += 1;
        let intent = ActionIntent::new(id, actor, ActionRecord::new::<A>(encode(payload)), at);
        let (result, events, _) = self.world.dispatch(&intent, at)?.into_parts();
        Ok((result, events))
    }

    /// Alice and Bob sit down to dinner at `at`; returns the dinner.
    fn dinner(&mut self, at: WorldTime, uninterruptible: bool) -> ProcessId {
        let (result, events) = self
            .act(
                self.alice,
                &SitDown {
                    with: self.bob,
                    duration: DINNER,
                    uninterruptible,
                },
                at,
            )
            .expect("dispatched");
        assert!(matches!(result, ActionResult::Accepted { .. }));
        let started: DinnerStarted =
            serde_json::from_slice(events[0].payload().payload()).expect("started");
        started.dinner
    }

    fn serve(&mut self, dinner: ProcessId, at: WorldTime) {
        self.act(self.alice, &Serve { dinner }, at).expect("served");
    }

    /// Someone calls Bob at `at`; returns the facts, and what the phone was told.
    fn call_bob(&mut self, at: WorldTime) -> (Vec<EventEnvelope>, InterruptOutcome) {
        let (_, events) = self
            .act(self.alice, &Call { callee: self.bob }, at)
            .expect("dispatched");
        let answered = events
            .iter()
            .find(|event| event.event_type() == &Answered::EVENT_TYPE)
            .expect("the phone records what it was told");
        let answered: Answered =
            serde_json::from_slice(answered.payload().payload()).expect("answered");
        (events, answered.outcome)
    }
}

fn types(events: &[EventEnvelope]) -> Vec<&str> {
    events
        .iter()
        .map(|event| event.event_type().as_str())
        .collect()
}

// ---------------------------------------------------------------------------------------------
// The claims.
// ---------------------------------------------------------------------------------------------

/// The owner is woken at the expected end, ends its own process, and the fact it emits is caused by
/// the process — not by the request that started it an hour earlier.
#[test]
fn the_owner_ends_its_own_process_at_its_expected_end() {
    let mut bench = Bench::new();
    let dinner = bench.dinner(t(0), false);

    let record = bench.world.processes().get(dinner).expect("running");
    assert_eq!(record.owner(), &Dining::ID);
    assert_eq!(record.process_type(), &Dinner::PROCESS_TYPE);
    assert_eq!(record.participants(), [bench.alice, bench.bob]);
    assert_eq!(record.started(), t(0));
    assert_eq!(record.expected_end(), Some(t(DINNER)));
    assert_eq!(bench.world.next_instant(), Some(t(DINNER)));

    let advanced = bench.world.advance_to(t(DINNER)).expect("advances");
    assert_eq!(types(advanced.events()), vec!["dinner-ended"]);
    let fact = &advanced.events()[0];
    assert_eq!(*fact.caused_by(), Causation::Process(dinner));
    assert_eq!(fact.provenance().emitted_by(), &Dining::ID);
    assert_eq!(fact.provenance().controller_decision(), None);
    assert!(bench.world.processes().is_empty(), "ended and gone");
}

/// The phone asks during dessert; the dining system refuses. The dinner runs on unchanged, the phone
/// is told `Refused`, and the dinner still ends at its hour.
#[test]
fn a_refused_interruption_leaves_the_process_running_and_tells_the_requester() {
    let mut bench = Bench::new();
    let dinner = bench.dinner(t(0), false);
    bench.serve(dinner, t(600));
    bench.serve(dinner, t(1_200));

    let before = bench.world.processes().get(dinner).cloned();
    let (events, outcome) = bench.call_bob(t(1_800));
    assert_eq!(outcome, InterruptOutcome::Refused);
    assert_eq!(types(&events), vec!["phone-rang", "call-answered"]);
    assert_eq!(
        bench.world.processes().get(dinner).cloned(),
        before,
        "nothing about the dinner changed"
    );

    let advanced = bench.world.advance_to(t(DINNER)).expect("advances");
    assert_eq!(types(advanced.events()), vec!["dinner-ended"]);
}

/// The phone asks during the starters; the dining system ends the dinner. Its fact is the dining
/// system's, caused by the ring the phone was handling, and recorded before the phone's own fact —
/// it happened while the phone was still deciding. The old wake at the hour is ignored.
#[test]
fn an_owner_that_ends_its_process_on_request_records_why_and_is_not_woken_again() {
    let mut bench = Bench::new();
    let dinner = bench.dinner(t(0), false);

    let (events, outcome) = bench.call_bob(t(300));
    assert_eq!(outcome, InterruptOutcome::Ended);
    assert_eq!(
        types(&events),
        vec!["phone-rang", "dinner-ended", "call-answered"]
    );
    let rang = &events[0];
    let interrupted = &events[1];
    assert_eq!(interrupted.provenance().emitted_by(), &Dining::ID);
    assert_eq!(*interrupted.caused_by(), Causation::Event(rang.id()));
    let decoded: DinnerEnded =
        serde_json::from_slice(interrupted.payload().payload()).expect("ended");
    assert_eq!(
        decoded,
        DinnerEnded {
            dinner,
            interrupted: true
        }
    );
    assert!(bench.world.processes().get(dinner).is_none());

    let advanced = bench.world.advance_to(t(DINNER)).expect("advances");
    assert!(advanced.events().is_empty(), "the old wake is stale");
    assert_eq!(advanced.instants(), 1, "it was visited, and ignored");
    assert_eq!(advanced.skipped(), 0, "stale is not skipped");
}

/// During the main course the dining system suspends the dinner instead. A suspended process is not
/// woken at its old end.
#[test]
fn a_suspended_process_is_not_woken() {
    let mut bench = Bench::new();
    let dinner = bench.dinner(t(0), false);
    bench.serve(dinner, t(600));

    let (_, outcome) = bench.call_bob(t(900));
    assert_eq!(outcome, InterruptOutcome::Suspended);
    let record = bench.world.processes().get(dinner).expect("still there");
    assert_eq!(record.phase(), ProcessPhase::Suspended);

    let advanced = bench.world.advance_to(t(DINNER)).expect("advances");
    assert!(advanced.events().is_empty());
    assert!(bench.world.processes().get(dinner).is_some());

    // Asked again while suspended, the owner leaves it so: that is a refusal, not a second
    // suspension.
    let (_, again) = bench.call_bob(t(DINNER + 1));
    assert_eq!(again, InterruptOutcome::Refused);
}

/// An uninterruptible dinner is answered without asking the owner — who, during the starters, would
/// have ended it.
#[test]
fn an_uninterruptible_process_is_answered_without_consulting_its_owner() {
    let mut bench = Bench::new();
    let dinner = bench.dinner(t(0), true);
    let (events, outcome) = bench.call_bob(t(300));
    assert_eq!(outcome, InterruptOutcome::Uninterruptible);
    assert_eq!(types(&events), vec!["phone-rang", "call-answered"]);
    assert!(bench.world.processes().get(dinner).is_some());
}

/// Nothing to interrupt, and nobody to decide.
#[test]
fn a_request_with_no_running_process_or_no_enabled_owner_is_answered_without_one() {
    let mut bench = Bench::new();
    let (_, outcome) = bench.call_bob(t(10));
    assert_eq!(outcome, InterruptOutcome::NotRunning);

    let dinner = bench.dinner(t(20), false);
    bench.world.disable(&Dining::ID).expect("dining disables");
    let (_, outcome) = bench.call_bob(t(30));
    assert_eq!(outcome, InterruptOutcome::OwnerDisabled);
    assert!(bench.world.processes().get(dinner).is_some());

    // And a disabled owner is not woken at the end either: the entry is skipped, and counted.
    let advanced = bench.world.advance_to(t(20 + DINNER)).expect("advances");
    assert!(advanced.events().is_empty());
    assert_eq!(advanced.skipped(), 1);
}

/// The phone uses its *own* process kind against the dining system's dinner — the one form of the
/// attempt that compiles. It is refused by name, and the dinner is untouched.
#[test]
fn a_non_owner_cannot_end_a_process_it_does_not_own() {
    let mut bench = Bench::new();
    let dinner = bench.dinner(t(0), false);
    let before = bench.world.processes().get(dinner).cloned();

    assert_eq!(
        bench.act(bench.alice, &Barge { dinner }, t(10)).err(),
        Some(KernelError::ProcessNotOwned {
            process: dinner,
            owner: Dining::ID,
            writing_system: Phone::ID,
        })
    );
    assert_eq!(bench.world.processes().get(dinner).cloned(), before);
}

/// A process cannot start and end in one instant: that would be an event.
#[test]
fn a_process_must_be_expected_to_end_later_than_it_starts() {
    let mut bench = Bench::new();
    let refused = bench.act(
        bench.alice,
        &SitDown {
            with: bench.bob,
            duration: 0,
            uninterruptible: false,
        },
        t(50),
    );
    assert_eq!(
        refused.err(),
        Some(KernelError::ProcessEndNotInTheFuture {
            end: t(50),
            now: t(50)
        })
    );
    assert!(bench.world.processes().is_empty());
}

/// A process saved mid-dinner and restored into a world assembled the same way keeps its identity,
/// its state and its scheduled end, and ends exactly as the original does.
#[test]
fn a_process_survives_a_save_and_restore_with_its_scheduled_end() {
    let mut original = Bench::new();
    let dinner = original.dinner(t(0), false);
    original.serve(dinner, t(600));
    let checkpoint = original.world.advance_to(t(1_000)).expect("advances");
    assert!(checkpoint.events().is_empty());

    let bytes = serde_json::to_vec(&original.world.schedule_snapshot()).expect("serializes");
    let snapshot: ScheduleSnapshot = serde_json::from_slice(&bytes).expect("parses");

    let mut restored = Bench::new();
    restored.world.restore_schedule(snapshot).expect("restores");
    let record = restored.world.processes().get(dinner).expect("restored");
    assert_eq!(record.expected_end(), Some(t(DINNER)));
    assert_eq!(course_of(record), Course::Main);
    assert_eq!(
        restored.world.processes().get(dinner),
        original.world.processes().get(dinner)
    );

    let expected = original.world.advance_to(t(10_000)).expect("advances");
    let actual = restored.world.advance_to(t(10_000)).expect("advances");
    assert_eq!(types(actual.events()), vec!["dinner-ended"]);
    assert_eq!(actual.events()[0].at(), t(DINNER));
    assert_eq!(actual.events(), expected.events());
}

/// A system that starts a process with an end and has no answer when it comes is a bug, said by
/// name rather than a process left running past its end in silence.
#[test]
fn a_system_that_does_not_wake_its_process_is_named() {
    struct Forgetful;
    impl SystemIdentity for Forgetful {
        const ID: SystemId = SystemId::from_static("forgetful");
    }
    struct Nap;
    impl ProcessKind for Nap {
        const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("napping");
        type Owner = Forgetful;
    }
    #[derive(Serialize, Deserialize)]
    struct Doze;
    impl Action for Doze {
        const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("doze");
        const OWNER: SystemId = Forgetful::ID;
    }
    impl System for Forgetful {
        const VERSION: SystemVersion = SystemVersion::new(1);
        fn declaration(&self) -> SystemDeclaration {
            SystemDeclaration::of::<Self>().providing::<Doze>()
        }
        fn resolve(
            &self,
            world: &mut WorldView<'_, Self>,
            _intent: &ActionIntent,
        ) -> Result<Vec<Emission>, KernelError> {
            world.start_process(ProcessStart::<Nap>::new(Vec::new()).ending_at(t(60)))?;
            Ok(Vec::new())
        }
    }

    let mut world = World::new();
    world.install(Forgetful).expect("installs");
    let sleeper = world
        .create_entity(
            EntityKey::new("sleeper").expect("a key"),
            EntityType::Person,
        )
        .expect("an entity");
    let intent = ActionIntent::new(
        ActionId::from_raw(1),
        sleeper,
        ActionRecord::new::<Doze>(encode(&Doze)),
        t(0),
    );
    let _ = world.dispatch(&intent, t(0)).expect("dispatched");
    assert_eq!(
        world.advance_to(t(60)),
        Err(KernelError::ProcessNotWokenBySystem {
            system: Forgetful::ID,
            process: ProcessId::from_raw(1),
        })
    );
}

/// Two owners that answer every interruption by requesting one of the other's would recurse without
/// end inside one call. The nesting is bounded like reduction is, and the error says so.
#[test]
fn interruptions_that_request_interruptions_forever_hit_the_limit() {
    struct Left;
    impl SystemIdentity for Left {
        const ID: SystemId = SystemId::from_static("left");
    }
    struct Right;
    impl SystemIdentity for Right {
        const ID: SystemId = SystemId::from_static("right");
    }
    struct Leaning;
    impl ProcessKind for Leaning {
        const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("leaning-left");
        type Owner = Left;
    }
    struct Tilting;
    impl ProcessKind for Tilting {
        const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("tilting-right");
        type Owner = Right;
    }
    #[derive(Serialize, Deserialize)]
    struct Begin;
    impl Action for Begin {
        const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("begin");
        const OWNER: SystemId = Left::ID;
    }
    #[derive(Serialize, Deserialize)]
    struct Lean;
    impl Action for Lean {
        const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("lean");
        const OWNER: SystemId = Right::ID;
    }

    /// Asks for an interruption of the first process the *other* system owns.
    fn the_other<S: SystemIdentity>(world: &mut WorldView<'_, S>) -> Result<(), KernelError> {
        let other = world
            .read()
            .processes()
            .find(|process| *process.owner() != S::ID)
            .map(Process::id)
            .expect("the other system's process");
        world.request_interrupt(other).map(|_| ())
    }

    impl System for Left {
        const VERSION: SystemVersion = SystemVersion::new(1);
        fn declaration(&self) -> SystemDeclaration {
            SystemDeclaration::of::<Self>().providing::<Begin>()
        }
        fn resolve(
            &self,
            world: &mut WorldView<'_, Self>,
            _intent: &ActionIntent,
        ) -> Result<Vec<Emission>, KernelError> {
            if world.read().processes().next().is_none() {
                world.start_process(ProcessStart::<Leaning>::new(Vec::new()))?;
                return Ok(Vec::new());
            }
            the_other(world)?;
            Ok(Vec::new())
        }
        fn interrupt(
            &self,
            world: &mut WorldView<'_, Self>,
            _request: &InterruptRequest,
        ) -> Result<Vec<Emission>, KernelError> {
            the_other(world)?;
            Ok(Vec::new())
        }
    }
    impl System for Right {
        const VERSION: SystemVersion = SystemVersion::new(1);
        fn declaration(&self) -> SystemDeclaration {
            SystemDeclaration::of::<Self>().providing::<Lean>()
        }
        fn resolve(
            &self,
            world: &mut WorldView<'_, Self>,
            _intent: &ActionIntent,
        ) -> Result<Vec<Emission>, KernelError> {
            world.start_process(ProcessStart::<Tilting>::new(Vec::new()))?;
            Ok(Vec::new())
        }
        fn interrupt(
            &self,
            world: &mut WorldView<'_, Self>,
            _request: &InterruptRequest,
        ) -> Result<Vec<Emission>, KernelError> {
            the_other(world)?;
            Ok(Vec::new())
        }
    }

    let mut world = World::new();
    world.install(Left).expect("installs");
    world.install(Right).expect("installs");
    let actor = world
        .create_entity(EntityKey::new("actor").expect("a key"), EntityType::Person)
        .expect("an entity");
    let request = |id: u64, record: ActionRecord| {
        ActionIntent::new(ActionId::from_raw(id), actor, record, t(0))
    };
    let _ = world
        .dispatch(
            &request(1, ActionRecord::new::<Begin>(encode(&Begin))),
            t(0),
        )
        .expect("left starts leaning");
    let _ = world
        .dispatch(&request(2, ActionRecord::new::<Lean>(encode(&Lean))), t(0))
        .expect("right starts tilting");

    let error = world
        .dispatch(
            &request(3, ActionRecord::new::<Begin>(encode(&Begin))),
            t(0),
        )
        .expect_err("the requests recurse");
    assert!(
        matches!(error, KernelError::InterruptionsTooDeep { limit: 16, .. }),
        "the nesting bound, stated as a literal: {error:?}"
    );
}
