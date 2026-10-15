//! The world thread's clock and in-server seam, driven with injected wall instants (`ARC-69`; TW-c
//! AC-TWc-2 and the mutations M-TWc-1, M-TWc-2, M-TWc-7).
//!
//! No socket, no thread and no real time: the test calls the runtime's tick and admin command with the wall
//! instant it means, so every consult instant and every wall instant is exact. The controllers are written
//! here against the public seam (`HostedController`), as a real one would be: the server names none.

use std::cell::{Cell, RefCell};
use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use mineworld_contracts::{ActionRequest, EntityKey, EntityType, WorldTime};
use mineworld_kernel::{Emission, World};

use super::{ClockChange, ControlCommand, WorldRuntime};
use crate::host::{HostConfig, HostedWorld, SeatRoster};
use crate::hosted::{HostedBinding, HostedController};
use crate::protocol::WireObservation;

/// What a probe saw: the world instant of each consult (its observation's `at`), and the wall instant, in
/// milliseconds from the test's base, at which the world thread consulted it.
type Log = Rc<RefCell<Vec<(i64, u64)>>>;

/// The wall instant, in milliseconds from the base, the test is at: read by the probes at each consult.
type WallMillis = Rc<Cell<u64>>;

/// A paced probe: the lattice of `cadence` wall seconds at the scale in force, from `origin`, exactly the
/// shape the CLI adapter gives a paced seat (`tools/cli/src/hosted.rs`), written here so that the server is
/// tested without the adapter. A rescale moves the origin to the pending instant and the period to the new
/// scale.
struct PacedProbe {
    cadence: i64,
    origin: i64,
    period: i64,
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
        self.log
            .borrow_mut()
            .push((observation.at().seconds(), self.wall.get()));
        None
    }

    fn rescaled(&mut self, pending: WorldTime, scale: NonZeroU32) {
        self.origin = pending.seconds();
        self.period = self.cadence * i64::from(scale.get());
    }
}

/// A reactive probe: consulted every world second of the scale in force, from the instant it was due.
struct ReactiveProbe {
    step: i64,
    log: Log,
    wall: WallMillis,
}

impl HostedController for ReactiveProbe {
    fn next_consult(&self, after: WorldTime) -> WorldTime {
        WorldTime::from_seconds(after.seconds() + self.step)
    }

    fn decide(&mut self, observation: &WireObservation) -> Option<ActionRequest> {
        self.log
            .borrow_mut()
            .push((observation.at().seconds(), self.wall.get()));
        None
    }

    fn rescaled(&mut self, _pending: WorldTime, scale: NonZeroU32) {
        self.step = i64::from(scale.get());
    }
}

fn scale(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).expect("non-zero")
}

/// Two seats, `paced` (seat k = 1 of a cadence of 5 wall seconds) and `reactive`, at 12× to begin with.
/// Returns the runtime, the two logs, and the wall cell the probes read.
struct Town {
    runtime: WorldRuntime,
    paced: Log,
    reactive: Log,
    wall: WallMillis,
    base: Instant,
}

fn town() -> Town {
    let base = Instant::now();
    let wall: WallMillis = Rc::new(Cell::new(0));
    let paced: Log = Rc::new(RefCell::new(Vec::new()));
    let reactive: Log = Rc::new(RefCell::new(Vec::new()));

    let mut world = World::new();
    for seat in ["paced", "reactive"] {
        world
            .create_entity(
                EntityKey::new(seat).expect("a legal key"),
                EntityType::Person,
            )
            .expect("the person is created");
    }
    world
        .genesis(WorldTime::EPOCH, Vec::<Emission>::new())
        .expect("the world begins");

    let config = HostConfig {
        time_scale: scale(12),
        ..HostConfig::default()
    };
    let paced_log = Rc::clone(&paced);
    let paced_wall = Rc::clone(&wall);
    let reactive_log = Rc::clone(&reactive);
    let reactive_wall = Rc::clone(&wall);
    let hosted = HostedWorld::new(world)
        .seating(SeatRoster::new([
            EntityKey::new("paced").expect("a legal key"),
            EntityKey::new("reactive").expect("a legal key"),
        ]))
        .hosting(
            EntityKey::new("paced").expect("a legal key"),
            move |binding: HostedBinding| {
                // Seat k = 1: its first lattice instant is one wall second after the binding, as the
                // adapter has it. A fresh world binds at its epoch less one second (`first_binding`).
                Box::new(PacedProbe {
                    cadence: 5,
                    origin: binding.at.seconds() + i64::from(binding.scale.get()),
                    period: 5 * i64::from(binding.scale.get()),
                    log: Rc::clone(&paced_log),
                    wall: Rc::clone(&paced_wall),
                })
            },
        )
        .hosting(
            EntityKey::new("reactive").expect("a legal key"),
            move |binding: HostedBinding| {
                Box::new(ReactiveProbe {
                    step: i64::from(binding.scale.get()),
                    log: Rc::clone(&reactive_log),
                    wall: Rc::clone(&reactive_wall),
                })
            },
        );
    let runtime = WorldRuntime::new_at(hosted, Arc::new(config), base);
    Town {
        runtime,
        paced,
        reactive,
        wall,
        base,
    }
}

/// One thing that happens to the town, at a wall instant in milliseconds from its base.
#[derive(Clone, Copy)]
enum Event {
    /// The world thread ticks.
    Tick,
    /// The host changes the scale.
    Rescale(u32),
}

/// Runs the town for `seconds` of wall time with its ticks at `tick_times` and the given rescales. Ticks
/// and rescales are applied in wall order; a rescale comes before a tick at the same instant.
fn run(town: &mut Town, tick_times: &[u64], rescales: &[(u64, u32)], seconds: u64) {
    let mut events: Vec<(u64, u8, Event)> = tick_times
        .iter()
        .copied()
        .take_while(|at| *at <= seconds * 1_000)
        .map(|at| (at, 1, Event::Tick))
        .chain(rescales.iter().map(|&(at, to)| (at, 0, Event::Rescale(to))))
        .collect();
    events.sort_by_key(|(at, order, _)| (*at, *order));
    for (at, _, event) in events {
        let wall = town.base + Duration::from_millis(at);
        town.wall.set(at);
        match event {
            Event::Tick => town.runtime.tick_at(wall),
            Event::Rescale(to) => {
                let answer = town.runtime.control_at(
                    ControlCommand::Change(ClockChange {
                        paused: None,
                        time_scale: Some(scale(to)),
                    }),
                    wall,
                );
                assert!(
                    matches!(answer, super::ControlAnswer::Clock(state) if state.time_scale == to),
                    "the rescale is applied: {answer:?}"
                );
            }
        }
    }
}

/// Ticks every 100 ms, exactly.
fn steady() -> Vec<u64> {
    (0..=30_000).step_by(100).collect()
}

/// The same ticks, each delayed by a seeded 0 to 39 ms: the same world, watched with jitter (INV-TWc-2).
fn jittered(seed: u64) -> Vec<u64> {
    let mut state = seed;
    (0..=300)
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
    log.borrow().iter().map(|(at, _)| *at).collect()
}

/// Consecutive differences of the wall instants of a log, in milliseconds.
fn wall_gaps(log: &Log) -> Vec<i64> {
    log.borrow()
        .windows(2)
        .map(|pair| {
            i64::try_from(pair[1].1).expect("small") - i64::try_from(pair[0].1).expect("small")
        })
        .collect()
}

/// AC-TWc-2 (INV-TWc-1): a live rescale keeps every hosted cadence. At wall 7.3 s the scale becomes 24×; at
/// 19.1 s, 6×. The paced seat's consult instants are the server's rescheduled chain (the straddling gap is
/// `T − p_prev + ⌈(p − T)·s₁/s₀⌉`), its wall instants are five seconds apart to within a tick, and the reactive
/// seat is consulted once a wall second to within a tick.
#[test]
fn a_live_rescale_keeps_every_hosted_cadence() {
    let mut town = town();
    run(&mut town, &steady(), &[(7_300, 24), (19_100, 6)], 30);

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

/// SD-TWc-4 and AC-TWc-3 (in memory): the instants in-server controllers are consulted at are a function of
/// the world, its seeds and the host's pacing changes, not of when the ticks ran. Two runs with jittered
/// ticks, and the same rescales at the same wall instants, consult the same instants in the same order.
#[test]
fn in_server_consult_instants_do_not_follow_tick_jitter() {
    let mut steady_town = town();
    run(&mut steady_town, &steady(), &[(7_300, 24), (19_100, 6)], 30);

    let mut jittered_town = town();
    run(
        &mut jittered_town,
        &jittered(7),
        &[(7_300, 24), (19_100, 6)],
        30,
    );

    let (a, b) = (
        instants(&steady_town.reactive),
        instants(&jittered_town.reactive),
    );
    let common = a.len().min(b.len());
    assert!(common > 20, "a real comparison: {common} consults");
    assert_eq!(
        &a[..common],
        &b[..common],
        "the reactive seat's consult instants follow the tick schedule"
    );
    let (a, b) = (instants(&steady_town.paced), instants(&jittered_town.paced));
    let common = a.len().min(b.len());
    assert_eq!(
        &a[..common],
        &b[..common],
        "the paced seat's consult instants follow the tick schedule"
    );
}
