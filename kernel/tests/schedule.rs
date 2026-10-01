//! The world clock and the schedule, from outside the kernel: what a request may defer, when it
//! happens, what it says about why, and what a world refuses so that time only runs forward.
//!
//! The claims, from step-04 §3 C1:
//!
//! ```text
//! a deferral is queued, not recorded, and fires at its instant as the deferring system's fact,
//!   caused by what that system was handling                                 — INV-15, F-2
//! advancing jumps between due instants; empty spans are never visited       — SD-2, AC-11
//! time does not run backwards, and a request may not overtake due work      — SD-1, AC-12
//! the schedule saves and restores with its pending work intact              — S5 readiness
//! a disabled system's deferred fact does not happen                         — AC-2
//! ```
//!
//! The systems here are stubs written the way a System Pack writes one. They know nothing: a timer
//! that rings, and a chime that answers a ring. The kernel never learns what either means.

use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionResult, ActionTypeId, Causation, EntityId,
    EntityKey, EntityType, Event, EventEnvelope, EventSchemaVersion, EventTypeId, SystemId,
    Visibility, WorldTime,
};
use mineworld_kernel::{
    Emission, KernelError, ScheduleSnapshot, System, SystemDeclaration, SystemIdentity,
    SystemVersion, World, WorldView,
};
use serde::{Deserialize, Serialize};

fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("a payload encodes")
}

fn t(seconds: i64) -> WorldTime {
    WorldTime::from_seconds(seconds)
}

// ---------------------------------------------------------------------------------------------
// Timer: answers `set`, says so now, and rings later.
// ---------------------------------------------------------------------------------------------

struct Timer;

impl SystemIdentity for Timer {
    const ID: SystemId = SystemId::from_static("timer");
}

#[derive(Serialize, Deserialize)]
struct Set {
    delay: i64,
    label: u32,
}

impl Action for Set {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("set");
    const OWNER: SystemId = Timer::ID;
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct TimerSet {
    label: u32,
}

impl Event for TimerSet {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("timer-set");
    const OWNER: SystemId = Timer::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Rang {
    label: u32,
}

impl Event for Rang {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("rang");
    const OWNER: SystemId = Timer::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl System for Timer {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .providing::<Set>()
            .emitting::<TimerSet>()
            .emitting::<Rang>()
    }

    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let set: Set = serde_json::from_slice(intent.payload().payload()).expect("set decodes");
        let at = t(world.at().seconds() + set.delay);
        world.defer(
            at,
            Emission::new::<Rang>(encode(&Rang { label: set.label }), Visibility::Public)
                .about(vec![intent.actor()]),
        )?;
        Ok(vec![Emission::new::<TimerSet>(
            encode(&TimerSet { label: set.label }),
            Visibility::Public,
        )])
    }
}

// ---------------------------------------------------------------------------------------------
// Chime: answers every ring in the same instant, and for an odd label asks to ring again later.
// ---------------------------------------------------------------------------------------------

struct Chime;

impl SystemIdentity for Chime {
    const ID: SystemId = SystemId::from_static("chime");
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Chimed {
    label: u32,
}

impl Event for Chimed {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("chimed");
    const OWNER: SystemId = Chime::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// How long after an odd-labelled ring the chime rings again.
const ECHO_DELAY: i64 = 60;

impl System for Chime {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([Timer::ID])
            .emitting::<Chimed>()
            .emitting::<Rang>()
            .subscribing_to::<Rang>()
    }

    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let rang: Rang = serde_json::from_slice(event.payload().payload()).expect("rang decodes");
        if rang.label % 2 == 1 {
            world.defer(
                t(world.at().seconds() + ECHO_DELAY),
                Emission::new::<Rang>(
                    encode(&Rang {
                        label: rang.label + 1,
                    }),
                    Visibility::Public,
                ),
            )?;
        }
        Ok(vec![Emission::new::<Chimed>(
            encode(&Chimed { label: rang.label }),
            Visibility::Public,
        )])
    }
}

// ---------------------------------------------------------------------------------------------
// The world they compose, and a way to make requests of it.
// ---------------------------------------------------------------------------------------------

struct Bench {
    world: World,
    actor: EntityId,
    next_action: u64,
}

impl Bench {
    fn new() -> Self {
        let mut world = World::new();
        world.install(Timer).expect("timer installs");
        world.install(Chime).expect("chime installs");
        let actor = world
            .create_entity(EntityKey::new("keeper").expect("a key"), EntityType::Person)
            .expect("an actor");
        Self {
            world,
            actor,
            next_action: 1,
        }
    }

    fn intent(&mut self, delay: i64, label: u32, at: WorldTime) -> ActionIntent {
        let id = ActionId::from_raw(self.next_action);
        self.next_action += 1;
        ActionIntent::new(
            id,
            self.actor,
            ActionRecord::new::<Set>(encode(&Set { delay, label })),
            at,
        )
    }

    fn set(&mut self, delay: i64, label: u32, at: WorldTime) -> Result<ActionResult, KernelError> {
        let intent = self.intent(delay, label, at);
        self.world
            .dispatch(&intent, at)
            .map(|dispatched| dispatched.result().clone())
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

/// A request that defers a fact is answered with what happened *now*; the deferred fact waits in
/// the schedule and happens at its own instant, as the deferring system's fact, caused by the
/// request — and its consequences are reduced in that same instant.
#[test]
fn a_deferred_fact_fires_at_its_instant_caused_by_the_request_that_deferred_it() {
    let mut bench = Bench::new();
    let intent = bench.intent(30, 2, t(100));
    let dispatched = bench.world.dispatch(&intent, t(100)).expect("dispatched");

    assert_eq!(types(dispatched.events()), vec!["timer-set"]);
    assert_eq!(dispatched.deferred().len(), 1, "reported as queued");
    assert_eq!(bench.world.scheduled(), 1, "and it is in the schedule");
    assert_eq!(bench.world.next_instant(), Some(t(130)));
    assert_eq!(bench.world.now(), t(100));

    let early = bench.world.advance_to(t(129)).expect("advances");
    assert!(early.events().is_empty(), "nothing is due before t130");
    assert_eq!(early.instants(), 0);
    assert_eq!(bench.world.now(), t(129));

    let advanced = bench.world.advance_to(t(130)).expect("advances");
    assert_eq!(advanced.instants(), 1);
    assert_eq!(types(advanced.events()), vec!["rang", "chimed"]);

    let rang = &advanced.events()[0];
    assert_eq!(rang.at(), t(130));
    assert_eq!(*rang.caused_by(), Causation::Action(intent.action_id()));
    assert_eq!(*rang.provenance().emitted_by(), Timer::ID);
    assert_eq!(
        rang.provenance().controller_decision(),
        Some(intent.action_id())
    );
    assert_eq!(rang.subjects(), [bench.actor]);
    // The identity continues the world's counter: the request's fact was 1, this is 2.
    assert_eq!(rang.id().raw(), 2);

    let chimed = &advanced.events()[1];
    assert_eq!(chimed.at(), t(130), "reduced in the same instant");
    assert_eq!(*chimed.caused_by(), Causation::Event(rang.id()));
    assert_eq!(*chimed.provenance().emitted_by(), Chime::ID);

    assert_eq!(bench.world.scheduled(), 0);
}

/// A fact deferred while *reacting* is caused by the fact that was being reacted to, and lands
/// strictly later — never in the instant it was asked for.
#[test]
fn a_fact_deferred_by_a_reaction_lands_strictly_later_and_names_its_parent() {
    let mut bench = Bench::new();
    bench.set(10, 1, t(0)).expect("dispatched");

    let first = bench.world.advance_to(t(10)).expect("advances");
    assert_eq!(types(first.events()), vec!["rang", "chimed"]);
    let odd_ring = first.events()[0].id();
    assert_eq!(bench.world.next_instant(), Some(t(10 + ECHO_DELAY)));

    let second = bench
        .world
        .advance_to(t(10 + ECHO_DELAY))
        .expect("advances");
    assert_eq!(types(second.events()), vec!["rang", "chimed"]);
    let echo = &second.events()[0];
    assert_eq!(
        *echo.provenance().emitted_by(),
        Chime::ID,
        "the deferring system"
    );
    assert_eq!(*echo.caused_by(), Causation::Event(odd_ring));
    let decoded: Rang = serde_json::from_slice(echo.payload().payload()).expect("decodes");
    assert_eq!(decoded, Rang { label: 2 });
}

/// The clock jumps from one due instant to the next: two rings ten million seconds apart cost two
/// instants, not ten million.
#[test]
fn advancing_visits_only_the_instants_something_is_due() {
    let mut bench = Bench::new();
    bench.set(10_000, 2, t(0)).expect("dispatched");
    bench.set(10_000_000, 4, t(0)).expect("dispatched");

    let advanced = bench.world.advance_to(t(1_000_000_000)).expect("advances");
    assert_eq!(advanced.instants(), 2);
    assert_eq!(
        advanced
            .events()
            .iter()
            .map(EventEnvelope::at)
            .collect::<Vec<_>>(),
        vec![t(10_000), t(10_000), t(10_000_000), t(10_000_000)]
    );
    assert_eq!(bench.world.now(), t(1_000_000_000));
    assert_eq!(bench.world.step().expect("steps"), None, "nothing left");
}

/// `step` fires exactly one instant: everything due then, and nothing after.
#[test]
fn a_step_fires_one_instant() {
    let mut bench = Bench::new();
    bench.set(5, 2, t(0)).expect("dispatched");
    bench.set(5, 4, t(0)).expect("dispatched");
    bench.set(9, 6, t(0)).expect("dispatched");

    let stepped = bench
        .world
        .step()
        .expect("steps")
        .expect("something was due");
    assert_eq!(stepped.instants(), 1);
    assert_eq!(
        types(stepped.events()),
        vec!["rang", "chimed", "rang", "chimed"]
    );
    assert_eq!(bench.world.now(), t(5));
    assert_eq!(bench.world.next_instant(), Some(t(9)));
}

/// Time does not run backwards: a request at an instant before the world's clock is refused, and so
/// is advancing to one. Neither refusal changes anything.
#[test]
fn time_does_not_run_backwards() {
    let mut bench = Bench::new();
    bench.set(1, 2, t(500)).expect("dispatched");
    let fired = bench.world.advance_to(t(600)).expect("advances");
    assert_eq!(fired.instants(), 1);

    let before = bench.world.schedule_snapshot();
    assert_eq!(
        bench.set(1, 4, t(599)),
        Err(KernelError::ClockWouldMoveBackwards {
            now: t(600),
            at: t(599)
        })
    );
    assert_eq!(
        bench.world.advance_to(t(10)),
        Err(KernelError::ClockWouldMoveBackwards {
            now: t(600),
            at: t(10)
        })
    );
    assert_eq!(bench.world.schedule_snapshot(), before, "nothing changed");
}

/// A request may not overtake work that was due first: dispatching at or after a due instant is
/// refused, unchanged, until the world has been advanced past it — including work due at the very
/// instant of the request, which was queued first and so fires first.
#[test]
fn a_request_may_not_overtake_work_that_is_due() {
    let mut bench = Bench::new();
    bench.set(20, 2, t(0)).expect("dispatched");

    assert_eq!(
        bench.set(1, 4, t(20)),
        Err(KernelError::ScheduledWorkDue {
            due: t(20),
            at: t(20)
        })
    );
    assert_eq!(
        bench.set(1, 4, t(25)),
        Err(KernelError::ScheduledWorkDue {
            due: t(20),
            at: t(25)
        })
    );
    assert_eq!(
        bench.world.scheduled(),
        1,
        "the refused requests queued nothing"
    );

    let advanced = bench.world.advance_to(t(25)).expect("advances");
    assert_eq!(types(advanced.events()), vec!["rang", "chimed"]);
    assert!(matches!(
        bench.set(1, 4, t(25)),
        Ok(ActionResult::Accepted { .. })
    ));
}

/// Genesis states when a world begins, and that instant is the clock's first.
#[test]
fn a_world_begins_at_its_genesis_instant() {
    let mut bench = Bench::new();
    let facts = vec![Emission::new::<TimerSet>(
        encode(&TimerSet { label: 0 }),
        Visibility::Public,
    )];
    bench.world.genesis(t(7_200), facts).expect("genesis");
    assert_eq!(bench.world.now(), t(7_200));
    assert_eq!(
        bench.set(1, 2, t(7_199)),
        Err(KernelError::ClockWouldMoveBackwards {
            now: t(7_200),
            at: t(7_199)
        })
    );
}

/// A disabled system does not act, and a fact it deferred is that system acting later: it is
/// skipped, counted, and recorded nowhere.
#[test]
fn a_disabled_system_does_not_fire_what_it_deferred() {
    let mut bench = Bench::new();
    bench.set(10, 2, t(0)).expect("dispatched");
    bench.world.disable(&Chime::ID).expect("chime disables");
    bench.world.disable(&Timer::ID).expect("timer disables");

    let advanced = bench.world.advance_to(t(10)).expect("advances");
    assert!(advanced.events().is_empty());
    assert_eq!(advanced.skipped(), 1);
    assert_eq!(advanced.instants(), 1);
    assert_eq!(bench.world.scheduled(), 0);
}

/// The schedule saves with its pending work and restores into a world assembled the same way; the
/// restored world then fires exactly the facts the original would have — same identities, same
/// instants, same causes.
#[test]
fn pending_work_survives_a_save_and_restore() {
    let mut original = Bench::new();
    original.set(30, 1, t(100)).expect("dispatched");
    original.set(45, 2, t(100)).expect("dispatched");
    let first = original.world.advance_to(t(130)).expect("advances");
    assert_eq!(types(first.events()), vec!["rang", "chimed"]);
    assert_eq!(
        original.world.scheduled(),
        2,
        "the echo and the second ring wait"
    );

    let bytes = serde_json::to_vec(&original.world.schedule_snapshot()).expect("serializes");
    let snapshot: ScheduleSnapshot = serde_json::from_slice(&bytes).expect("parses");

    let mut restored = Bench::new();
    restored.world.restore_schedule(snapshot).expect("restores");
    assert_eq!(restored.world.now(), t(130));
    assert_eq!(restored.world.next_instant(), Some(t(145)));

    let expected = original.world.advance_to(t(1_000)).expect("advances");
    let actual = restored.world.advance_to(t(1_000)).expect("advances");
    assert_eq!(
        types(actual.events()),
        vec!["rang", "chimed", "rang", "chimed"]
    );
    assert_eq!(actual.events(), expected.events());
}

/// Restoring is assembly. A world that has run keeps its own time, and a snapshot holding work for a
/// system this world does not have is refused, changing nothing.
#[test]
fn a_schedule_restores_only_into_a_world_that_could_fire_it() {
    let mut original = Bench::new();
    original.set(30, 2, t(0)).expect("dispatched");
    let snapshot = original.world.schedule_snapshot();

    let mut running = Bench::new();
    running.set(1, 2, t(0)).expect("dispatched");
    assert_eq!(
        running.world.restore_schedule(snapshot.clone()),
        Err(KernelError::RestoreAfterTheWorldHasRun)
    );

    let mut bare = World::new();
    assert_eq!(
        bare.restore_schedule(snapshot),
        Err(KernelError::PersistedEntryNamesUninstalledSystem { system: Timer::ID })
    );
    assert_eq!(bare.scheduled(), 0);
    assert_eq!(bare.now(), WorldTime::EPOCH);
}
