//! The advance's admission, lifecycle and durable identity (pr-nightlife N2-C2): every refusal, the repeated
//! operation, the commands a running advance refuses or is ended by, and an interrupted one after a restart.
//!
//! No socket and no thread: the tests call the world thread's own entries. The stop adapter is written here
//! against the public seam (`AdvanceStop`), over a stub system the test world installs — the server names no
//! pack. Its Person is "in the state" when their key begins `sleeper`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use mineworld_contracts::{EntityId, EntityKey, EntityType, SystemId, WorldTime};
use mineworld_kernel::{
    System, SystemDeclaration, SystemIdentity, SystemVersion, World, WorldRead,
};
use mineworld_persistence::{
    AdvanceOutcome, Creation, Durability, EndReason, OperationEvent, OperationId, PersistentWorld,
    SqliteBackend,
};
use mineworld_sdk::advance::{AdvanceStop, StopIdentity};

use super::super::{ClockChange, ControlAnswer, ControlCommand, WorldRuntime};
use super::{AdvanceRefusal, AdvanceRequest, AdvanceStatus, Running};
use crate::host::{HostConfig, Hosted, HostedWorld, SeatRoster, Seated};
use crate::protocol::{RefusalCode, SessionId};
use crate::seats::{Departure, JoinRequest};

/// The stub system whose state the adapter reads: it owns nothing and does nothing.
#[derive(Default)]
struct Rest;

impl SystemIdentity for Rest {
    const ID: SystemId = SystemId::from_static("probe-rest");
}

impl System for Rest {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
    }
}

/// The stub adapter: a Person whose key begins `sleeper` rests, as occurrence 1, with no horizon.
pub(super) struct Keyed;

impl AdvanceStop for Keyed {
    fn name(&self) -> &'static str {
        "keyed"
    }

    fn system(&self) -> SystemId {
        Rest::ID
    }

    fn identify(&self, world: &WorldRead<'_>, person: EntityId) -> Option<StopIdentity> {
        let entity = world.entity(person)?;
        entity
            .key()
            .as_str()
            .starts_with("sleeper")
            .then_some(StopIdentity::from_raw(1))
    }

    fn horizon(&self, _: &WorldRead<'_>, _: EntityId, _: StopIdentity) -> Option<WorldTime> {
        None
    }

    fn holds(&self, world: &WorldRead<'_>, person: EntityId, _: StopIdentity) -> bool {
        self.identify(world, person).is_some()
    }
}

fn key(seat: &str) -> EntityKey {
    EntityKey::new(seat).expect("a legal key")
}

fn op(value: u128) -> OperationId {
    OperationId::from_raw(value)
}

fn request(operation: u128, seat: &str, until: &str) -> AdvanceRequest {
    AdvanceRequest {
        operation: op(operation),
        seat: key(seat),
        until: until.to_owned(),
    }
}

const SEATS: [&str; 3] = ["sleeper", "awake", "other"];

/// The composed world, before any entity: the stub system enabled when `rest`.
fn composed(rest: bool) -> World {
    let mut world = World::new();
    if rest {
        world.install(Rest).expect("the stub system installs");
    }
    world
}

/// A world with the three seats' people.
fn people(rest: bool) -> World {
    let mut world = composed(rest);
    for seat in SEATS {
        world
            .create_entity(key(seat), EntityType::Person)
            .expect("the person is created");
    }
    world
}

/// A runtime over a fresh world — persisted in `save` when given — started `--solo` or not.
fn host(save: Option<&Path>, solo: bool, rest: bool) -> WorldRuntime {
    super::town::register();
    let hosted = match save {
        None => {
            let mut world = people(rest);
            world
                .genesis(WorldTime::EPOCH, Vec::new())
                .expect("the world begins");
            HostedWorld::new(world)
        }
        Some(directory) => {
            let backend = SqliteBackend::create(directory, Durability::ProcessCrash)
                .expect("the save is created");
            let (persisted, _) = PersistentWorld::create(
                Box::new(backend),
                people(rest),
                Creation {
                    instance: 0x5eed_0000_0000_0000_0000_0000_0000_00a2,
                    pack: "probe".to_owned(),
                    at: WorldTime::EPOCH,
                    facts: Vec::new(),
                },
            )
            .expect("the save is begun");
            HostedWorld::persisted(persisted, 64).expect("a persisted world")
        }
    };
    reopen(hosted, solo)
}

fn reopen(hosted: HostedWorld, solo: bool) -> WorldRuntime {
    let hosted = hosted.seating(SeatRoster::new(SEATS.map(key)));
    let config = HostConfig {
        solo,
        ..HostConfig::default()
    };
    WorldRuntime::new(hosted, Arc::new(config)).expect("the world starts")
}

/// Resumes the save in `save` (the stub system enabled) and starts a solo host over it.
fn resume(save: &Path) -> WorldRuntime {
    let backend = SqliteBackend::open(save, Durability::ProcessCrash).expect("the save opens");
    let (persisted, _) =
        PersistentWorld::resume(Box::new(backend), composed(true)).expect("the save resumes");
    reopen(
        HostedWorld::persisted(persisted, 64).expect("a persisted world"),
        true,
    )
}

/// Seats a connection on `seat` as session `session`.
fn seat(runtime: &mut WorldRuntime, seat: &str, session: u64) -> Seated {
    runtime
        .join(
            &JoinRequest::plain(key(seat), SessionId::new(session)),
            None,
        )
        .expect("the seat is granted")
}

fn advance(
    runtime: &mut WorldRuntime,
    asked: AdvanceRequest,
) -> Result<AdvanceStatus, AdvanceRefusal> {
    match runtime.control(ControlCommand::Advance(asked)) {
        ControlAnswer::Advance(answer) => answer,
        other => panic!("an advance is answered as one: {other:?}"),
    }
}

fn ended(status: &AdvanceStatus) -> Option<(AdvanceOutcome, Option<EndReason>)> {
    status.ended
}

/// Starts an advance and leaves it running, as the N2-C3 loop will between slices.
fn start_running(runtime: &mut WorldRuntime, operation: u128, seated: &Seated) {
    let asked = request(operation, "sleeper", "keyed");
    let running: Running = runtime.admit(&asked).expect("admitted");
    assert_eq!(running.subscription(), seated.subscription());
    runtime
        .record_operation(OperationEvent::AdvanceStarted {
            operation: asked.operation,
            seat: asked.seat,
            adapter: asked.until,
            prior_scale: runtime.clock.scale(),
        })
        .expect("recorded");
    runtime.advancing = Some(running);
}

/// Each of N2-I3's refusals, in its order, and the request that passes them all. A refusal records nothing.
#[test]
fn every_refusal_is_named_and_records_nothing() {
    let mut shared = host(None, false, true);
    let _sleeper = seat(&mut shared, "sleeper", 1);
    assert_eq!(
        advance(&mut shared, request(1, "sleeper", "keyed")),
        Err(AdvanceRefusal::NotSolo),
        "a shared host never advances"
    );

    let mut solo = host(None, true, false);
    let _sleeper = seat(&mut solo, "sleeper", 1);
    assert_eq!(
        advance(&mut solo, request(1, "sleeper", "keyed")),
        Err(AdvanceRefusal::UnknownAdapter),
        "the adapter's system is not enabled in this world"
    );

    let mut solo = host(None, true, true);
    assert_eq!(
        advance(&mut solo, request(1, "sleeper", "nap")),
        Err(AdvanceRefusal::UnknownAdapter),
        "no adapter of that name"
    );
    assert_eq!(
        advance(&mut solo, request(1, "sleeper", "keyed")),
        Err(AdvanceRefusal::SeatMismatch),
        "nobody drives the seat"
    );
    let sleeper = seat(&mut solo, "sleeper", 1);
    let awake = seat(&mut solo, "awake", 2);
    assert_eq!(
        advance(&mut solo, request(1, "sleeper", "keyed")),
        Err(AdvanceRefusal::OtherControllerConnected),
        "a second decision session"
    );
    solo.depart(awake.subscription(), Departure::Left);
    let _ = solo.control(ControlCommand::Change(ClockChange {
        paused: Some(true),
        time_scale: None,
    }));
    assert_eq!(
        advance(&mut solo, request(1, "sleeper", "keyed")),
        Err(AdvanceRefusal::Paused)
    );
    let _ = solo.control(ControlCommand::Change(ClockChange {
        paused: Some(false),
        time_scale: None,
    }));
    solo.depart(sleeper.subscription(), Departure::Left);
    let _awake = seat(&mut solo, "awake", 3);
    assert_eq!(
        advance(&mut solo, request(1, "awake", "keyed")),
        Err(AdvanceRefusal::NotAsleep)
    );
    assert!(solo.operations.is_empty(), "no refusal is recorded");
    assert_eq!(
        solo.control(ControlCommand::AdvanceStatus(op(1))),
        ControlAnswer::Advance(Err(AdvanceRefusal::UnknownOperation))
    );
}

/// A seat held for a dropped connection is a decision session N2-I3 does not run past; once the hold ends it
/// is not. A hosted or free seat never counts.
#[test]
fn a_held_seat_refuses_until_its_hold_ends() {
    let mut solo = host(None, true, true);
    let _sleeper = seat(&mut solo, "sleeper", 1);
    let awake = seat(&mut solo, "awake", 2);
    solo.depart(awake.subscription(), Departure::Dropped);
    assert_eq!(
        advance(&mut solo, request(1, "sleeper", "keyed")),
        Err(AdvanceRefusal::OtherControllerConnected),
        "inside the hold window"
    );
    let now = solo.clock.now();
    solo.table.expire(
        std::time::Instant::now() + std::time::Duration::from_secs(31),
        now,
    );
    let admitted = advance(&mut solo, request(1, "sleeper", "keyed")).expect("admitted");
    assert_eq!(ended(&admitted), None, "running");
}

/// An admitted advance records `advance-started` and runs; a repeated operation with the same payload answers
/// its status without starting again, and one with another payload is refused. With nothing due and no
/// horizon nothing could ever end it, so its first slice ends it `stopped` — never a false `woke` — having
/// moved no time.
#[test]
fn an_admitted_advance_runs_once_and_a_repeat_is_idempotent() {
    let mut solo = host(None, true, true);
    let _sleeper = seat(&mut solo, "sleeper", 1);
    let before = solo.world.world().now();
    let running = advance(&mut solo, request(7, "sleeper", "keyed")).expect("admitted");
    assert_eq!(ended(&running), None, "running");
    assert_eq!(
        advance(&mut solo, request(7, "sleeper", "keyed")),
        Ok(running.clone())
    );
    assert_eq!(solo.operations.len(), 1, "a repeat records nothing");
    solo.advance_slice();
    let status = solo.advance_status(op(7)).expect("known");
    assert_eq!(ended(&status), Some((AdvanceOutcome::Stopped, None)));
    assert_eq!(solo.world.world().now(), before, "no time moved");
    assert_eq!(solo.operations.len(), 2, "started, then ended");
    assert_eq!(
        advance(&mut solo, request(7, "awake", "keyed")),
        Err(AdvanceRefusal::OperationConflict)
    );
    assert_eq!(
        advance(&mut solo, request(7, "sleeper", "nap")),
        Err(AdvanceRefusal::OperationConflict)
    );
    assert_eq!(
        solo.control(ControlCommand::AdvanceCancel(op(7))),
        ControlAnswer::Advance(Ok(status)),
        "cancelling an ended advance answers its status"
    );
}

/// N2-I12 for everything but the loop: while an advance runs, a join, another session's submit, a clock
/// change and a release of another seat are refused; a second advance is `busy`; cancel ends it
/// `cancelled` and submits nothing.
#[test]
fn a_running_advance_refuses_what_would_seat_or_repace_around_it() {
    let mut solo = host(None, true, true);
    let sleeper = seat(&mut solo, "sleeper", 1);
    start_running(&mut solo, 9, &sleeper);
    let revision_before = solo.world.world().now();

    let joined = solo.join(&JoinRequest::plain(key("awake"), SessionId::new(2)), None);
    assert_eq!(
        joined.map(|_| ()).map_err(|refusal| refusal.code()),
        Err(RefusalCode::AdvanceInProgress)
    );
    assert_eq!(
        solo.control(ControlCommand::Change(ClockChange {
            paused: Some(true),
            time_scale: None,
        })),
        ControlAnswer::AdvanceInProgress
    );
    assert_eq!(
        solo.control(ControlCommand::Release(key("awake"))),
        ControlAnswer::AdvanceInProgress
    );
    assert_eq!(
        solo.control(ControlCommand::Kick(SessionId::new(2))),
        ControlAnswer::AdvanceInProgress
    );
    assert_eq!(
        advance(&mut solo, request(10, "sleeper", "keyed")),
        Err(AdvanceRefusal::Busy)
    );
    assert!(
        solo.refused_by_advance(None),
        "a library caller's submit is refused"
    );
    assert!(!solo.refused_by_advance(Some(sleeper.subscription())));

    let cancelled = solo.control(ControlCommand::AdvanceCancel(op(9)));
    let ControlAnswer::Advance(Ok(status)) = cancelled else {
        panic!("cancel answers a status: {cancelled:?}");
    };
    assert_eq!(ended(&status), Some((AdvanceOutcome::Cancelled, None)));
    assert!(solo.advancing.is_none());
    assert_eq!(
        solo.world.world().now(),
        revision_before,
        "cancel submits nothing"
    );
}

/// The sleeper's departure, kick or release ends the advance `disconnected`; the night is never finished in
/// the background.
#[test]
fn losing_the_sleepers_connection_ends_it_disconnected() {
    for how in ["leave", "drop", "kick", "release"] {
        let mut solo = host(None, true, true);
        let sleeper = seat(&mut solo, "sleeper", 1);
        start_running(&mut solo, 3, &sleeper);
        match how {
            "leave" => solo.depart(sleeper.subscription(), Departure::Left),
            "drop" => solo.depart(sleeper.subscription(), Departure::Dropped),
            "kick" => {
                let answer = solo.control(ControlCommand::Kick(SessionId::new(1)));
                assert!(matches!(answer, ControlAnswer::Kicked(_)), "{answer:?}");
            }
            _ => {
                let answer = solo.control(ControlCommand::Release(key("sleeper")));
                assert!(
                    matches!(answer, ControlAnswer::Released { .. }),
                    "{answer:?}"
                );
            }
        }
        let status = solo.advance_status(op(3)).expect("known");
        assert_eq!(
            ended(&status),
            Some((AdvanceOutcome::Disconnected, None)),
            "{how}"
        );
    }
}

/// N2-I8: the lifecycle is in the save's own table. A process that dies between `advance-started` and
/// `advance-ended` leaves an open operation; the next start appends `interrupted` at the durable instant and
/// resumes nothing, `verify` still passes, and the repeated id answers the recorded result.
#[test]
fn a_crash_between_records_is_interrupted_at_the_next_start() {
    let directory = scratch("interrupted");
    {
        let mut solo = host(Some(&directory), true, true);
        let sleeper = seat(&mut solo, "sleeper", 1);
        start_running(&mut solo, 0xabc, &sleeper);
        // The process dies here: no end, no shutdown, no checkpoint.
        drop(solo);
    }
    let mut reopened = resume(&directory);
    let Hosted::Persisted(world) = &reopened.world else {
        panic!("persisted");
    };
    let records = world.operations().expect("coherent");
    assert_eq!(records.len(), 2);
    assert!(matches!(
        records[1].event,
        OperationEvent::AdvanceEnded {
            outcome: AdvanceOutcome::Interrupted,
            ..
        }
    ));
    assert!(reopened.advancing.is_none(), "nothing resumes");
    let _sleeper = seat(&mut reopened, "sleeper", 1);
    let status = advance(&mut reopened, request(0xabc, "sleeper", "keyed")).expect("known");
    assert_eq!(ended(&status), Some((AdvanceOutcome::Interrupted, None)));
    drop(reopened);
    let verified = mineworld_persistence::verify(
        &SqliteBackend::open(&directory, Durability::ProcessCrash).expect("opens"),
        composed(true),
    );
    assert!(verified.is_ok(), "{verified:?}");
    let _ = std::fs::remove_dir_all(&directory);
}

/// A save from before host operations has no table: it resumes with an empty history, and a host that never
/// advances never creates it.
#[test]
fn a_save_without_the_table_resumes_with_no_operations() {
    let directory = scratch("legacy");
    drop(host(Some(&directory), true, true));
    let reopened = resume(&directory);
    assert!(reopened.operations.is_empty());
    drop(reopened);
    let _ = std::fs::remove_dir_all(&directory);
}

/// A scratch save directory, fresh: removed if a previous run left it.
fn scratch(name: &str) -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("mineworld-n2-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    directory
}
