//! The world thread's clock and in-server seam, driven with injected wall instants (`ARC-69`; TW-c AC-TWc-2,
//! AC-TWc-3 and the mutations M-TWc-1, M-TWc-2, M-TWc-7).
//!
//! No socket, no thread and no real time: the test calls the runtime's tick and admin command with the wall
//! instant it means, so every consult instant and every wall instant is exact. The controllers are written
//! here against the public seam (`HostedController`), as a real one would be: the server names none.

use std::cell::{Cell, RefCell};
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use mineworld_contracts::{
    Action, ActionRecord, ActionRequest, ActionTypeId, EntityId, EntityKey, EntityType, SystemId,
    WorldTime,
};
use mineworld_kernel::{Emission, World};
use mineworld_persistence::{
    Creation, Durability, HostRecord, PacingChange, PersistentWorld, SqliteBackend,
};
use serde::{Deserialize, Serialize};

use super::{ClockChange, ControlAnswer, ControlCommand, WorldRuntime};
use crate::host::{HostConfig, Hosted, HostedWorld, SeatRoster, TimeScaleChoice};
use crate::hosted::{HostedAnswer, HostedBinding, HostedController};
use crate::protocol::WireObservation;

/// One consult a probe received: the world instant it was consulted at (its observation's `at`), the wall
/// instant in milliseconds from the test's base at which the world thread consulted it, and what became of
/// the request it made, if it made one.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Consult {
    at: i64,
    wall: u64,
    answer: Option<String>,
}

type Log = Rc<RefCell<Vec<Consult>>>;

/// The wall instant, in milliseconds from the base, the test is at: read by the probes at each consult.
type WallMillis = Rc<Cell<u64>>;

/// What the probes ask the world for: a request the kernel does not know, so it is refused, and every refusal
/// is journaled as the input's outcome (a persisted world keeps every input).
#[derive(Debug, Serialize, Deserialize)]
struct Ask {
    words: String,
}

impl Action for Ask {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("probe-ask");
    const OWNER: SystemId = SystemId::from_static("probe-system");
}

fn ask(actor: EntityId) -> ActionRequest {
    let payload = serde_json::to_vec(&Ask {
        words: "hello".to_owned(),
    })
    .expect("encodes");
    ActionRequest::new(actor, ActionRecord::<Vec<u8>>::new::<Ask>(payload))
}

/// Records one consult, and returns the request the probe makes at it (none unless it asks).
fn consulted(
    log: &Log,
    wall: &WallMillis,
    observation: &WireObservation,
    asking: bool,
) -> Option<ActionRequest> {
    log.borrow_mut().push(Consult {
        at: observation.at().seconds(),
        wall: wall.get(),
        answer: None,
    });
    asking.then(|| ask(observation.observer()))
}

fn answered(log: &Log, answer: &HostedAnswer) {
    if let Some(last) = log.borrow_mut().last_mut() {
        last.answer = Some(format!("{answer:?}"));
    }
}

/// A paced probe: the lattice of `cadence` wall seconds at the scale in force, from `origin`, exactly the
/// shape the CLI adapter gives a paced seat (`tools/cli/src/hosted.rs`), written here so that the server is
/// tested without the adapter. A rescale moves the origin to the pending instant and the period to the new
/// scale.
struct PacedProbe {
    cadence: i64,
    origin: i64,
    period: i64,
    asking: bool,
    log: Log,
    wall: WallMillis,
}

impl HostedController for PacedProbe {
    fn next_consult(&self, after: WorldTime) -> WorldTime {
        let after = after.seconds();
        if after < self.origin {
            return WorldTime::from_seconds(self.origin);
        }
        WorldTime::from_seconds(
            self.origin + ((after - self.origin) / self.period + 1) * self.period,
        )
    }

    fn decide(&mut self, observation: &WireObservation) -> Option<ActionRequest> {
        consulted(&self.log, &self.wall, observation, self.asking)
    }

    fn answered(&mut self, answer: &HostedAnswer) {
        answered(&self.log, answer);
    }

    fn rescaled(&mut self, pending: WorldTime, scale: NonZeroU32) {
        self.origin = pending.seconds();
        self.period = self.cadence * i64::from(scale.get());
    }
}

/// A reactive probe: consulted every world second of the scale in force, from the instant it was due.
struct ReactiveProbe {
    step: i64,
    asking: bool,
    log: Log,
    wall: WallMillis,
}

impl HostedController for ReactiveProbe {
    fn next_consult(&self, after: WorldTime) -> WorldTime {
        WorldTime::from_seconds(after.seconds() + self.step)
    }

    fn decide(&mut self, observation: &WireObservation) -> Option<ActionRequest> {
        consulted(&self.log, &self.wall, observation, self.asking)
    }

    fn answered(&mut self, answer: &HostedAnswer) {
        answered(&self.log, answer);
    }

    fn rescaled(&mut self, _pending: WorldTime, scale: NonZeroU32) {
        self.step = i64::from(scale.get());
    }
}

fn scale(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).expect("non-zero")
}

fn key(seat: &str) -> EntityKey {
    EntityKey::new(seat).expect("a legal key")
}

/// The town's world: a person for each seat, created but not yet begun (a persisted world begins itself).
fn people() -> World {
    let mut world = World::new();
    for seat in ["paced", "reactive"] {
        world
            .create_entity(key(seat), EntityType::Person)
            .expect("the person is created");
    }
    world
}

/// Two seats: `paced` (seat k = 1 of a cadence of 5 wall seconds) and `reactive`, at 12× to begin with. The
/// save directory, if any, is created fresh.
struct Town {
    runtime: WorldRuntime,
    paced: Log,
    reactive: Log,
    wall: WallMillis,
    base: Instant,
}

fn town(save: Option<&Path>, asking: bool) -> Town {
    let base = Instant::now();
    let wall: WallMillis = Rc::new(Cell::new(0));
    let paced: Log = Rc::new(RefCell::new(Vec::new()));
    let reactive: Log = Rc::new(RefCell::new(Vec::new()));

    let hosted = match save {
        None => {
            let mut world = people();
            world
                .genesis(WorldTime::EPOCH, Vec::<Emission>::new())
                .expect("the world begins");
            HostedWorld::new(world)
        }
        Some(directory) => {
            let backend = SqliteBackend::create(directory, Durability::ProcessCrash)
                .expect("the save is created");
            let (persisted, _) = PersistentWorld::create(
                Box::new(backend),
                people(),
                Creation {
                    instance: 0x5eed_0000_0000_0000_0000_0000_0000_0042,
                    pack: "probe".to_owned(),
                    at: WorldTime::EPOCH,
                    facts: Vec::<Emission>::new(),
                },
            )
            .expect("the save is begun");
            HostedWorld::persisted(persisted, 64).expect("a persisted world")
        }
    };

    let config = HostConfig {
        time_scale: TimeScaleChoice {
            flag: Some(scale(12)),
            pack: None,
        },
        ..HostConfig::default()
    };
    let (paced_log, paced_wall) = (Rc::clone(&paced), Rc::clone(&wall));
    let (reactive_log, reactive_wall) = (Rc::clone(&reactive), Rc::clone(&wall));
    let hosted = hosted
        .seating(SeatRoster::new([key("paced"), key("reactive")]))
        .hosting(key("paced"), move |binding: HostedBinding| {
            // Seat k = 1: its first lattice instant is one wall second after the binding, as the adapter has it.
            // A fresh world binds at its epoch less one second (`first_binding`).
            Box::new(PacedProbe {
                cadence: 5,
                origin: binding.at.seconds() + i64::from(binding.scale.get()),
                period: 5 * i64::from(binding.scale.get()),
                asking,
                log: Rc::clone(&paced_log),
                wall: Rc::clone(&paced_wall),
            })
        })
        .hosting(key("reactive"), move |binding: HostedBinding| {
            Box::new(ReactiveProbe {
                step: i64::from(binding.scale.get()),
                asking,
                log: Rc::clone(&reactive_log),
                wall: Rc::clone(&reactive_wall),
            })
        });
    let runtime = WorldRuntime::new_at(hosted, Arc::new(config), base).expect("the world starts");
    Town {
        runtime,
        paced,
        reactive,
        wall,
        base,
    }
}

/// One thing that happens to the town, at a wall instant in milliseconds from its base.
#[derive(Clone, Copy, Debug)]
enum Event {
    /// The world thread ticks.
    Tick,
    /// The host changes the scale.
    Rescale(u32),
    /// The host pauses the clock.
    Pause,
    /// The host resumes the clock.
    Resume,
}

/// Runs the town for `seconds` of wall time: its ticks at `tick_times` and the given events at their wall
/// instants. Events are applied in wall order; an event comes before a tick at the same instant.
fn run(town: &mut Town, tick_times: &[u64], events: &[(u64, Event)], seconds: u64) {
    let mut schedule: Vec<(u64, u8, Event)> = tick_times
        .iter()
        .copied()
        .take_while(|at| *at <= seconds * 1_000)
        .map(|at| (at, 1, Event::Tick))
        .chain(events.iter().map(|&(at, event)| (at, 0, event)))
        .collect();
    schedule.sort_by_key(|(at, order, _)| (*at, *order));
    for (at, _, event) in schedule {
        let wall = town.base + Duration::from_millis(at);
        town.wall.set(at);
        let change = match event {
            Event::Tick => {
                town.runtime.tick_at(wall);
                continue;
            }
            Event::Rescale(to) => ClockChange {
                paused: None,
                time_scale: Some(scale(to)),
            },
            Event::Pause => ClockChange {
                paused: Some(true),
                time_scale: None,
            },
            Event::Resume => ClockChange {
                paused: Some(false),
                time_scale: None,
            },
        };
        let answer = town
            .runtime
            .control_at(ControlCommand::Change(change), wall);
        assert!(
            matches!(answer, ControlAnswer::Clock(_)),
            "the change is answered: {answer:?}"
        );
    }
}

/// Ticks every 100 ms, exactly.
fn steady() -> Vec<u64> {
    (0..=60_000).step_by(100).collect()
}

/// The same ticks, each delayed by a seeded 0 to 39 ms: the same world, watched with jitter (INV-TWc-2).
fn jittered(seed: u64) -> Vec<u64> {
    let mut state = seed;
    (0..=600)
        .map(|k| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            100 * k + (state >> 33) % 40
        })
        .collect()
}

/// The world's instants of a log, in the order they were consulted.
fn instants(log: &Log) -> Vec<i64> {
    log.borrow().iter().map(|consult| consult.at).collect()
}

/// What a log says, without the wall instants that differ between runs: each consult's instant and answer.
fn outcomes(log: &Log) -> Vec<(i64, Option<String>)> {
    log.borrow()
        .iter()
        .map(|consult| (consult.at, consult.answer.clone()))
        .collect()
}

/// Consecutive differences of the wall instants of a log, in milliseconds.
fn wall_gaps(log: &Log) -> Vec<i64> {
    log.borrow()
        .windows(2)
        .map(|pair| {
            i64::try_from(pair[1].wall).expect("small")
                - i64::try_from(pair[0].wall).expect("small")
        })
        .collect()
}

/// The host journal of a persisted town's world.
fn journal(town: &Town) -> Vec<HostRecord> {
    match &town.runtime.world {
        Hosted::Persisted(world) => world.host_records().expect("the journal reads"),
        Hosted::Ephemeral(_) => panic!("an ephemeral town keeps no journal"),
    }
}

/// The rescales of AC-TWc-2 and their wall instants, in milliseconds.
const RESCALES: [(u64, Event); 2] = [(7_300, Event::Rescale(24)), (19_100, Event::Rescale(6))];

/// A scratch save directory, fresh: removed if a previous run left it.
fn scratch(name: &str) -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("mineworld-tw-c-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    directory
}

/// AC-TWc-2 (INV-TWc-1): a live rescale keeps every hosted cadence. At wall 7.3 s the scale becomes 24×; at
/// 19.1 s, 6×. The paced seat's consult instants are the server's rescheduled chain (the straddling gap is
/// `T − p_prev + ⌈(p − T)·s₁/s₀⌉`), its wall instants are five seconds apart to within a tick, and the reactive
/// seat is consulted once a wall second to within a tick.
#[test]
fn a_live_rescale_keeps_every_hosted_cadence() {
    let mut town = town(None, false);
    run(&mut town, &steady(), &RESCALES, 60);

    let paced = instants(&town.paced);
    assert_eq!(
        &paced[..6],
        &[11, 71, 175, 295, 382, 412],
        "12× then 24× then 6×: 60, then the straddling gap 104 (= 16 + ⌈44·24/12⌉), then 120, the \
         straddling gap 87 (= 75 + ⌈45·6/24⌉), then 30"
    );
    for gap in wall_gaps(&town.paced) {
        assert!(
            (4_900..=5_100).contains(&gap),
            "paced consults are five wall seconds apart to within one tick, not {gap} ms"
        );
    }

    let reactive = instants(&town.reactive);
    assert_eq!(
        &reactive[..8],
        &[11, 23, 35, 47, 59, 71, 83, 103],
        "one world second at 12×, then the reactive chain moved by the same wall wait"
    );
    for gap in wall_gaps(&town.reactive) {
        assert!(
            (900..=1_100).contains(&gap),
            "the reactive seat is consulted once a wall second to within one tick, not {gap} ms"
        );
    }
}

/// SD-TWc-4 (INV-TWc-2): the instants in-server controllers are consulted at are a function of the world, its
/// seeds and the host's pacing changes, not of when the ticks ran. Two runs with jittered ticks, and the same
/// rescales at the same wall instants, consult the same instants in the same order.
#[test]
fn in_server_consult_instants_do_not_follow_tick_jitter() {
    let mut steady_town = town(None, false);
    run(&mut steady_town, &steady(), &RESCALES, 60);
    let mut jittered_town = town(None, false);
    run(&mut jittered_town, &jittered(7), &RESCALES, 60);

    for (a, b) in [
        (
            instants(&steady_town.reactive),
            instants(&jittered_town.reactive),
        ),
        (instants(&steady_town.paced), instants(&jittered_town.paced)),
    ] {
        let common = a.len().min(b.len());
        assert!(common > 5, "a real comparison: {common} consults");
        assert_eq!(
            &a[..common],
            &b[..common],
            "a seat's consult instants follow the tick schedule"
        );
    }
}

/// The wall instant, in milliseconds, at which a segment that is at `anchor_world` at `anchor_wall` and runs at
/// `scale` reads `target`: the least whole millisecond at which its reading is `target`. Exact for scales of
/// at most 1 000.
fn wall_for(anchor_wall: u64, anchor_world: i64, scale: u32, target: i64) -> u64 {
    let distance = i128::from(target - anchor_world) * 1_000;
    let wait = (distance + i128::from(scale) - 1) / i128::from(scale);
    anchor_wall + u64::try_from(wait).expect("forward in time")
}

/// AC-TWc-3 (INV-TWc-2), persisted: a persisted scripted session — two paced probes and a reactive one that
/// ask the world something from every consult, two rescales and a pause — is run three ways. Two runs with
/// jittered ticks and the changes at the same wall instants give identical host journals and identical
/// consults and answers. A third run, with steady ticks, makes its changes at the **world** instants the first
/// run's journal recorded — each at the wall instant that reads that instant — and gives the same three results.
#[test]
fn a_persisted_session_is_a_function_of_its_journal_not_of_tick_timing() {
    let first_dir = scratch("first");
    let mut first = town(Some(&first_dir), true);
    let script: [(u64, Event); 4] = [
        (7_300, Event::Rescale(24)),
        (13_000, Event::Pause),
        (14_000, Event::Resume),
        (19_100, Event::Rescale(6)),
    ];
    run(&mut first, &steady(), &script, 60);
    let first_journal = journal(&first);
    assert!(
        first
            .paced
            .borrow()
            .iter()
            .any(|consult| consult.answer.is_some())
            && first
                .reactive
                .borrow()
                .iter()
                .any(|consult| consult.answer.is_some()),
        "the seats really asked the world something at their consults"
    );
    assert_eq!(
        first_journal
            .iter()
            .map(|record| record.change)
            .collect::<Vec<_>>(),
        [
            PacingChange::Started {
                scale: scale(12),
                paused: false
            },
            PacingChange::Scaled { to: scale(24) },
            PacingChange::Paused,
            PacingChange::Resumed,
            PacingChange::Scaled { to: scale(6) },
        ],
        "the first run journals its start and its four changes, in order"
    );

    let second_dir = scratch("second");
    let mut second = town(Some(&second_dir), true);
    run(&mut second, &jittered(11), &script, 60);
    assert_eq!(
        journal(&second),
        first_journal,
        "identical host journals under jitter"
    );

    // The third run: its changes at the world instants the first journal recorded.
    let mut events: Vec<(u64, Event)> = Vec::new();
    let (mut anchor_wall, mut anchor_world, mut speed) = (0_u64, 0_i64, 12_u32);
    let mut resume_wall = 0_u64;
    for record in &first_journal {
        let at = record.at.seconds();
        match record.change {
            PacingChange::Started { .. } => {}
            PacingChange::Scaled { to } => {
                let wall = wall_for(anchor_wall, anchor_world, speed, at);
                events.push((wall, Event::Rescale(to.get())));
                (anchor_wall, anchor_world, speed) = (wall, at, to.get());
            }
            PacingChange::Paused => {
                let wall = wall_for(anchor_wall, anchor_world, speed, at);
                events.push((wall, Event::Pause));
                (anchor_wall, anchor_world) = (wall, at);
                resume_wall = wall + 1_000;
            }
            PacingChange::Resumed => {
                events.push((resume_wall, Event::Resume));
                (anchor_wall, anchor_world) = (resume_wall, at);
            }
            PacingChange::Stopped => {}
        }
    }
    let third_dir = scratch("third");
    let mut third = town(Some(&third_dir), true);
    run(&mut third, &steady(), &events, 60);
    assert_eq!(
        journal(&third),
        first_journal,
        "the changes landed at the recorded world instants"
    );

    for (name, other) in [("jittered", &second), ("recorded-instants", &third)] {
        for (seat, ours, theirs) in [
            ("paced", &first.paced, &other.paced),
            ("reactive", &first.reactive, &other.reactive),
        ] {
            let (ours, theirs) = (outcomes(ours), outcomes(theirs));
            let common = ours.len().min(theirs.len());
            assert!(common > 5, "a real comparison: {common} consults");
            assert_eq!(
                ours[..common],
                theirs[..common],
                "{name}: the {seat} seat is consulted at the same instants and answered the same way"
            );
        }
    }
    // The towns hold their saves open; Windows cannot remove an open SQLite file.
    drop((first, second, third));
    for directory in [first_dir, second_dir, third_dir] {
        let _ = std::fs::remove_dir_all(directory);
    }
}
