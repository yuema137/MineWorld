//! A small hosted night for the advance's parity tests (pr-nightlife N2-C3, N2-I6): a stub system whose rest is a
//! Process with a horizon, hosted probes on two cadences that schedule kernel work, one that wakes the sleeper
//! early, and the ordinary host driven by an injected wall clock that ticks at every due instant — the oracle.
//!
//! ```text
//! rest  { until }   the actor rests: a `resting` Process ending at `until`; its end states `rose`
//! chime { }         defers a public `chimed` 30 s ahead (kernel entries between consults, and ties)
//! nudge { target }  ends the target's rest at once, stating `rose`
//! ```
//!
//! Everything the world does goes through the kernel's own pipeline; the save is the record compared.

use std::cell::RefCell;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use mineworld_contracts::{
    Action, ActionIntent, ActionRecord, ActionRequest, ActionTypeId, EntityId, EntityKey,
    EntityType, Event, EventEnvelope, EventId, EventSchemaVersion, EventTypeId, ProcessTypeId,
    Rejection, SystemId, Visibility, WorldTime,
};
use mineworld_kernel::{
    Emission, KernelError, Process, ProcessKind, ProcessStart, System, SystemDeclaration,
    SystemIdentity, SystemVersion, World, WorldRead, WorldView,
};
use mineworld_persistence::{Creation, Durability, PersistenceBackend, PersistentWorld, SqliteBackend};
use mineworld_sdk::advance::{AdvanceStop, StopIdentity, register_stops};
use serde::{Deserialize, Serialize};

use super::super::WorldRuntime;
use crate::host::{HostConfig, HostedWorld, SeatRoster, Seated, TimeScaleChoice};
use crate::hosted::{HostedAnswer, HostedBinding, HostedController};
use crate::perception::EventPerception;
use crate::protocol::{PerceivedJoin, SessionId, WireObservation};
use crate::seats::JoinRequest;

// ---------------------------------------------------------------------------------------------
// The stub system and its adapter
// ---------------------------------------------------------------------------------------------

#[derive(Default)]
pub(super) struct Rest;

impl SystemIdentity for Rest {
    const ID: SystemId = SystemId::from_static("probe-rest");
}

#[derive(Serialize, Deserialize)]
struct RestUntil {
    until: i64,
}

impl Action for RestUntil {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("rest");
    const OWNER: SystemId = Rest::ID;
}

#[derive(Serialize, Deserialize)]
struct Chime {}

impl Action for Chime {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("chime");
    const OWNER: SystemId = Rest::ID;
}

#[derive(Serialize, Deserialize)]
struct Nudge {}

impl Action for Nudge {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("nudge");
    const OWNER: SystemId = Rest::ID;
}

#[derive(Serialize, Deserialize)]
struct Chimed {}

impl Event for Chimed {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("chimed");
    const OWNER: SystemId = Rest::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

#[derive(Serialize, Deserialize)]
struct Rose {
    nudged: bool,
}

impl Event for Rose {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("rose");
    const OWNER: SystemId = Rest::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

struct Resting;

impl ProcessKind for Resting {
    const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("resting");
    type Owner = Rest;
}

/// How far ahead a chime rings.
const CHIME: i64 = 30;

fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("encodes")
}

fn resting_of<'a>(world: &WorldRead<'a>, person: EntityId) -> Option<&'a Process> {
    world.processes().find(|process| {
        *process.process_type() == Resting::PROCESS_TYPE && process.participants() == [person]
    })
}

impl System for Rest {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .providing::<RestUntil>()
            .providing::<Chime>()
            .providing::<Nudge>()
            .emitting::<Chimed>()
            .emitting::<Rose>()
    }

    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        let kind = intent.action_type();
        if *kind == RestUntil::ACTION_TYPE && resting_of(world, intent.actor()).is_some() {
            return Err(Rejection::Busy);
        }
        if *kind == Nudge::ACTION_TYPE
            && intent
                .target()
                .is_none_or(|target| resting_of(world, target).is_none())
        {
            return Err(Rejection::PreconditionFailed);
        }
        Ok(())
    }

    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let kind = intent.action_type();
        if *kind == RestUntil::ACTION_TYPE {
            let rest: RestUntil =
                serde_json::from_slice(intent.payload().payload()).expect("a rest");
            world.start_process(
                ProcessStart::<Resting>::new(Vec::new())
                    .with_participants(vec![intent.actor()])
                    .ending_at(WorldTime::from_seconds(rest.until)),
            )?;
            return Ok(Vec::new());
        }
        if *kind == Chime::ACTION_TYPE {
            world.defer(
                WorldTime::from_seconds(world.at().seconds() + CHIME),
                Emission::new::<Chimed>(encode(&Chimed {}), Visibility::Public)
                    .about(vec![intent.actor()]),
            )?;
            return Ok(Vec::new());
        }
        let target = intent.target().expect("validated");
        let process = resting_of(&world.read(), target)
            .map(Process::id)
            .expect("validated");
        world.end_process::<Resting>(process)?;
        Ok(vec![
            Emission::new::<Rose>(encode(&Rose { nudged: true }), Visibility::Public)
                .about(vec![target]),
        ])
    }

    fn wake(
        &self,
        world: &mut WorldView<'_, Self>,
        process: &Process,
    ) -> Result<Vec<Emission>, KernelError> {
        world.end_process::<Resting>(process.id())?;
        Ok(vec![
            Emission::new::<Rose>(encode(&Rose { nudged: false }), Visibility::Public)
                .about(process.participants().to_vec()),
        ])
    }
}

/// The adapter over the stub: an occurrence is one `resting` Process, its horizon the Process's end.
struct RestStop;

impl AdvanceStop for RestStop {
    fn name(&self) -> &'static str {
        "rest"
    }

    fn system(&self) -> SystemId {
        Rest::ID
    }

    fn identify(&self, world: &WorldRead<'_>, person: EntityId) -> Option<StopIdentity> {
        resting_of(world, person).map(|process| StopIdentity::from_raw(process.id().raw()))
    }

    fn horizon(
        &self,
        world: &WorldRead<'_>,
        person: EntityId,
        occurrence: StopIdentity,
    ) -> Option<WorldTime> {
        resting_of(world, person)
            .filter(|process| process.id().raw() == occurrence.raw())
            .and_then(Process::expected_end)
    }

    fn holds(&self, world: &WorldRead<'_>, person: EntityId, occurrence: StopIdentity) -> bool {
        resting_of(world, person).is_some_and(|process| process.id().raw() == occurrence.raw())
    }
}

/// Registers the build's adapters as every test binary of this crate does: one list, the same everywhere.
pub(super) fn register() {
    register_stops(vec![Box::new(RestStop), Box::new(super::tests::Keyed)]);
}

// ---------------------------------------------------------------------------------------------
// The probes
// ---------------------------------------------------------------------------------------------

/// What every probe of one town consulted: the seat, the instant, and what became of its request.
pub(super) type Log = Rc<RefCell<Vec<String>>>;

fn request<A: Action + Serialize>(actor: EntityId, target: Option<EntityId>, action: &A) -> ActionRequest {
    let record = ActionRecord::<Vec<u8>>::new::<A>(encode(action));
    match target {
        Some(target) => ActionRequest::new(actor, record).with_target(target),
        None => ActionRequest::new(actor, record),
    }
}

/// A hosted probe: consulted every `period` world seconds from `origin`; chimes when `chiming`, and nudges the
/// sleeper at its first consult at or after `nudge_at`.
struct Probe {
    seat: &'static str,
    origin: i64,
    period: i64,
    chiming: bool,
    nudge: Option<(i64, EntityId)>,
    log: Log,
}

impl HostedController for Probe {
    fn next_consult(&self, after: WorldTime) -> WorldTime {
        let after = after.seconds();
        if after < self.origin {
            return WorldTime::from_seconds(self.origin);
        }
        WorldTime::from_seconds(self.origin + ((after - self.origin) / self.period + 1) * self.period)
    }

    fn decide(&mut self, observation: &WireObservation) -> Option<ActionRequest> {
        let at = observation.at().seconds();
        self.log
            .borrow_mut()
            .push(format!("consult {} {at}", self.seat));
        if let Some((when, sleeper)) = self.nudge
            && at >= when
        {
            self.nudge = None;
            return Some(request(observation.observer(), Some(sleeper), &Nudge {}));
        }
        self.chiming
            .then(|| request(observation.observer(), None, &Chime {}))
    }

    fn answered(&mut self, answer: &HostedAnswer) {
        let line = match answer {
            HostedAnswer::Answered(submitted) => format!(
                "  answered {} {:?}",
                submitted.action_id().raw(),
                submitted.result()
            ),
            HostedAnswer::Refused(code) => format!("  refused {code:?}"),
        };
        self.log.borrow_mut().push(line);
    }

    fn rescaled(&mut self, _pending: WorldTime, _scale: NonZeroU32) {}
}

/// Every fact is learned by every observer: the sleeper's perceived stream carries the whole night.
struct LearnsEverything;

impl EventPerception for LearnsEverything {
    fn record(&mut self, _fact: &EventEnvelope) {}

    fn admits(&self, _fact: &EventEnvelope, _observer: EntityId) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------------------------
// The town
// ---------------------------------------------------------------------------------------------

pub(super) const SCALE: u32 = 12;
/// The town begins at 23:00 on day 0.
pub(super) const DUSK: i64 = 23 * 3_600;
/// The sleeper rests until 01:00 on day 1: two world hours, ten wall minutes at 12×.
pub(super) const DAWN: i64 = 25 * 3_600;

/// A night: its runtime, its probes' log, the sleeper's connection, and the save directory.
pub(super) struct Night {
    pub(super) runtime: WorldRuntime,
    pub(super) log: Log,
    pub(super) sleeper: Seated,
    pub(super) save: PathBuf,
    pub(super) base: Instant,
}

/// How the night is set up.
#[derive(Clone, Copy)]
pub(super) struct Setup {
    /// The world instant at which the nudger wakes the sleeper, if it does.
    pub(super) nudge_at: Option<i64>,
    /// The perceived stream's bound, which N2-I13 slices against.
    pub(super) perceived_backlog: usize,
}

impl Default for Setup {
    fn default() -> Self {
        Self {
            nudge_at: None,
            perceived_backlog: HostConfig::default().perceived_backlog,
        }
    }
}

const SEATS: [&str; 4] = ["sleeper", "bells", "watch", "nudger"];

/// A fresh night in `save`: the sleeper seated (perceiving) and resting until [`DAWN`], at the instant the town
/// began. Nothing has ticked.
pub(super) fn night(save: &Path, setup: Setup) -> Night {
    register();
    let _ = std::fs::remove_dir_all(save);
    let mut world = World::new();
    world.install(Rest).expect("the stub installs");
    for seat in SEATS {
        world
            .create_entity(EntityKey::new(seat).expect("a key"), EntityType::Person)
            .expect("a person");
    }
    let backend = SqliteBackend::create(save, Durability::ProcessCrash).expect("a save");
    let (persisted, _) = PersistentWorld::create(
        Box::new(backend),
        world,
        Creation {
            instance: 0x5eed_0000_0000_0000_0000_0000_0000_00c3,
            pack: "night".to_owned(),
            at: WorldTime::from_seconds(DUSK),
            facts: Vec::new(),
        },
    )
    .expect("the save is begun");
    let sleeper_id = persisted
        .world()
        .read()
        .resolve_key(&EntityKey::new("sleeper").expect("a key"))
        .expect("resolves");
    let log: Log = Rc::new(RefCell::new(Vec::new()));
    let mut hosted = HostedWorld::persisted(persisted, 64)
        .expect("persisted")
        .seating(SeatRoster::new(
            SEATS.map(|seat| EntityKey::new(seat).expect("a key")),
        ));
    hosted.events = Box::new(LearnsEverything);
    // Bells chime every wall 5 s (60 world s), watch every wall second (12 world s) chimes too, and the nudger
    // looks every wall 10 s (120 world s); their lattices start one world second apart, as the CLI's do.
    for (seat, offset, period, chiming, nudges) in [
        ("bells", 2, 60, true, false),
        ("watch", 2, 12, true, false),
        ("nudger", 3, 120, false, true),
    ] {
        let log = Rc::clone(&log);
        let nudge = setup.nudge_at.filter(|_| nudges).map(|at| (at, sleeper_id));
        hosted = hosted.hosting(
            EntityKey::new(seat).expect("a key"),
            move |binding: HostedBinding| {
                Box::new(Probe {
                    seat,
                    origin: binding.at.seconds() + offset,
                    period,
                    chiming,
                    nudge,
                    log: Rc::clone(&log),
                })
            },
        );
    }
    let config = HostConfig {
        solo: true,
        time_scale: TimeScaleChoice {
            flag: Some(NonZeroU32::new(SCALE).expect("non-zero")),
            pack: None,
        },
        perceived_backlog: setup.perceived_backlog,
        ..HostConfig::default()
    };
    let base = Instant::now();
    let mut runtime = WorldRuntime::new_at(hosted, Arc::new(config), base).expect("starts");
    let since = runtime.head;
    let sleeper = runtime
        .join(
            &JoinRequest::plain(EntityKey::new("sleeper").expect("a key"), SessionId::new(1)),
            Some(PerceivedJoin { since }),
        )
        .expect("seated");
    let rest = request(sleeper_id, None, &RestUntil { until: DAWN });
    runtime
        .submit_at(sleeper_id, rest, WorldTime::from_seconds(DUSK))
        .expect("the rest is dispatched");
    Night {
        runtime,
        log,
        sleeper,
        save: save.to_path_buf(),
        base,
    }
}

impl Night {
    /// The wall instant at which the host clock reads `at` exactly: the oracle's injected tick.
    pub(super) fn wall_for(&self, at: i64) -> Instant {
        let world = at - DUSK;
        let millis = (world * 1_000 + i64::from(SCALE) - 1) / i64::from(SCALE);
        self.base + Duration::from_millis(u64::try_from(millis).expect("after the start"))
    }

    /// Whether the sleeper still rests (the oracle's stop, read as the adapter reads it).
    pub(super) fn resting(&self) -> bool {
        let read = self.runtime.world.world().read();
        let sleeper = read
            .resolve_key(&EntityKey::new("sleeper").expect("a key"))
            .expect("resolves");
        resting_of(&read, sleeper).is_some()
    }

    /// The next instant anything is due: the kernel's queue or a hosted consult, whichever is first.
    pub(super) fn next_due(&self) -> Option<i64> {
        let kernel = self.runtime.world.world().next_instant();
        let hosted = self.runtime.table.earliest_due();
        [kernel, hosted].into_iter().flatten().min().map(WorldTime::seconds)
    }

    /// The trace a comparison reads: every consult and answer, every fact the save holds, in order, and the
    /// instant the world reached.
    pub(super) fn trace(&self) -> Vec<String> {
        let mut lines = self.log.borrow().clone();
        let backend = SqliteBackend::open(&self.save, Durability::ProcessCrash).expect("opens");
        for fact in backend.facts_after(None).expect("reads") {
            let fact = fact.expect("a fact");
            lines.push(format!(
                "fact {} {}",
                fact.id,
                String::from_utf8(fact.bytes).expect("json")
            ));
        }
        lines.push(format!("reached {}", self.runtime.world.world().now().seconds()));
        lines
    }

    /// The newest fact this world recorded.
    pub(super) fn head(&self) -> Option<EventId> {
        self.runtime.head
    }
}

impl Drop for Night {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.save);
    }
}

/// The ordinary host as the oracle (N2-I6): driven by an injected wall clock that ticks at exactly every due
/// instant, until the first instant after which the sleeper no longer rests.
pub(super) fn ordinary(night: &mut Night) -> Vec<i64> {
    let mut visited = Vec::new();
    while night.resting() {
        let Some(at) = night.next_due() else {
            break;
        };
        let at = at.max(night.runtime.world.world().now().seconds());
        night.runtime.tick_at(night.wall_for(at));
        visited.push(at);
    }
    visited
}

/// A scratch save directory for one test.
pub(super) fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("mineworld-n2c3-{}-{name}", std::process::id()))
}
