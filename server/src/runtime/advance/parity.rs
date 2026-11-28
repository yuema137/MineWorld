//! N2-I6 oracle parity (pr-nightlife N2-C3): the ordinary host, driven by an injected wall clock that ticks at
//! every due instant, is frozen as a fixture **before the advance loop existed**; the advance must reproduce it
//! — every consult, every answer, every fact byte in the save, and the instant reached.
//!
//! `MINEWORLD_WRITE_FIXTURES=1` rewrites a fixture from the ordinary host; never from the loop.

use std::path::PathBuf;

use mineworld_persistence::{AdvanceOutcome, EndReason};

use super::town::{DAWN, DUSK, Setup, night, ordinary, scratch};

/// Where a frozen oracle lives.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/runtime/advance/fixtures")
        .join(format!("{name}.txt"))
}

/// The frozen trace, rewritten from the ordinary host when asked.
fn frozen(name: &str, ordinary_trace: &[String]) -> Vec<String> {
    let path = fixture(name);
    if !ordinary_trace.is_empty() && std::env::var_os("MINEWORLD_WRITE_FIXTURES").is_some() {
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("writable");
        std::fs::write(&path, ordinary_trace.join("\n") + "\n").expect("written");
    }
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} reads: {error}", path.display()))
        .lines()
        .map(str::to_owned)
        .collect()
}

/// The ordinary host still produces the frozen oracle: what the loop is compared with has not moved.
#[test]
fn the_ordinary_host_reproduces_the_frozen_oracle_night() {
    let mut whole = night(&scratch("oracle-whole"), Setup::default());
    let visited = ordinary(&mut whole);
    assert_eq!(visited.last().copied(), Some(DAWN), "it rests until dawn");
    let trace = whole.trace();
    assert_eq!(trace, frozen("night-whole", &trace));

    let mut nudged = night(
        &scratch("oracle-nudged"),
        Setup {
            nudge_at: Some(23 * 3_600 + 50 * 60),
            ..Setup::default()
        },
    );
    let visited = ordinary(&mut nudged);
    let woke = *visited.last().expect("visited");
    assert!(
        woke < DAWN,
        "the nudge wakes the sleeper before dawn: {woke}"
    );
    let trace = nudged.trace();
    assert_eq!(trace, frozen("night-nudged", &trace));
}

/// The advance reproduces the frozen oracle exactly: every consult and answer, every fact's bytes, the instant
/// reached — the whole night to the alarm, ending `woke` there, at the scale it began with.
#[test]
fn the_advanced_night_is_the_oracle_night() {
    let mut whole = night(&scratch("advance-whole"), Setup::default());
    let scale = whole.runtime.clock.scale();
    let status = whole.advanced(1, true, 100_000);
    assert_eq!(status.ended, Some((AdvanceOutcome::Woke, None)));
    assert_eq!(
        whole.runtime.world.world().now().seconds(),
        DAWN,
        "the alarm, not later"
    );
    assert_eq!(whole.trace(), frozen("night-whole", &[]));
    assert_eq!(whole.runtime.clock.scale(), scale, "no rescale");
    assert!(!whole.runtime.clock.paused());
    assert_eq!(
        whole.runtime.clock.now().seconds(),
        DAWN,
        "the clock runs on from the instant reached, with no wall time folded in"
    );
}

/// An earlier wake stops the night at the first completed instant at which the rest no longer holds — the
/// oracle's — and never past it.
#[test]
fn an_early_wake_stops_the_advance_where_the_oracle_stops() {
    let mut nudged = night(
        &scratch("advance-nudged"),
        Setup {
            nudge_at: Some(DUSK + 50 * 60),
            ..Setup::default()
        },
    );
    let status = nudged.advanced(2, true, 100_000);
    assert_eq!(status.ended, Some((AdvanceOutcome::Woke, None)));
    assert_eq!(nudged.trace(), frozen("night-nudged", &[]));
}

/// N2-I10, N2-I12: cancelled between two slices, the advance stops where it is: the sleeper still rests, the
/// server submitted nothing for them, the clock runs on from the instant reached at the same scale. A sweep
/// between slices delivers and neither consults nor moves the world.
#[test]
fn between_slices_a_sweep_only_sweeps_and_a_cancel_stops_and_submits_nothing() {
    let mut night = night(&scratch("advance-cancel"), Setup::default());
    let partial = night.advanced(3, true, 3);
    assert_eq!(partial.ended, None, "three slices do not finish the night");
    let reached = night.runtime.world.world().now();
    let consults = night.log.borrow().len();
    let wall = night.wall_for(DAWN + 3_600);
    night.runtime.tick_at(wall);
    assert_eq!(
        night.runtime.world.world().now(),
        reached,
        "a sweep moves nothing"
    );
    assert_eq!(
        night.log.borrow().len(),
        consults,
        "a sweep consults nobody"
    );
    let answer = night
        .runtime
        .control(super::super::ControlCommand::AdvanceCancel(op(3)));
    assert!(
        matches!(&answer, super::super::ControlAnswer::Advance(Ok(status)) if status.ended == Some((AdvanceOutcome::Cancelled, None))),
        "{answer:?}"
    );
    assert!(night.resting(), "cancel leaves the sleeper asleep");
    assert_eq!(night.runtime.world.world().now(), reached);
    assert_eq!(
        night.runtime.clock.now(),
        reached,
        "re-anchored at the instant reached"
    );
    assert_eq!(night.runtime.clock.scale().get(), super::town::SCALE);
    let actions_by_sleeper = night
        .trace()
        .iter()
        .filter(|line| line.contains("\"rest\"") || line.contains("\"rise\""))
        .count();
    assert_eq!(
        actions_by_sleeper, 0,
        "no fact of the sleeper's own actions beyond the rest"
    );
}

/// N2-I12: the sleeper's own `rise` (its `wake`), submitted between slices, is dispatched at the boundary the
/// advance reached and ends it `woke` there.
#[test]
fn the_sleepers_own_wake_between_slices_ends_it_woke_at_the_boundary() {
    let mut night = night(&scratch("advance-rise"), Setup::default());
    let _ = night.advanced(4, true, 2);
    let boundary = night.runtime.world.world().now();
    let sleeper = night.sleeper.observer();
    let subscription = night.sleeper.subscription();
    night
        .runtime
        .submit(
            Some(subscription),
            sleeper,
            super::town::request(sleeper, None, &super::town::Rise {}),
        )
        .expect("dispatched");
    night.pump(true, 1);
    let status = night.runtime.advance_status(op(4)).expect("known");
    assert_eq!(status.ended, Some((AdvanceOutcome::Woke, None)));
    assert_eq!(
        night.runtime.world.world().now(),
        boundary,
        "not one instant further"
    );
}

/// N2-I13 (D-N2-6): a sleeper whose client does not read is never released `lagged` by the advance's speed:
/// slices yield to delivery and the night waits; once the client reads, it finishes — still the oracle's night.
#[test]
fn a_client_that_does_not_read_holds_the_night_back_and_loses_nothing() {
    let mut night = night(
        &scratch("advance-unread"),
        Setup {
            perceived_backlog: 64,
            ..Setup::default()
        },
    );
    let stalled = night.advanced(5, false, 100_000);
    assert_eq!(stalled.ended, None, "waiting for delivery, not released");
    let waiting = night
        .runtime
        .advancing
        .as_ref()
        .map(|running| (running.awaiting_sweep, running.progress));
    let (awaiting, progress) = waiting.expect("still running");
    assert!(awaiting && progress.yielded > 0, "{progress}");
    assert!(progress.high_water < 64, "never past the bound: {progress}");
    // The client reads; each sweep lets the next slice go.
    night.pump(true, 100_000);
    let status = night.runtime.advance_status(op(5)).expect("known");
    assert_eq!(status.ended, Some((AdvanceOutcome::Woke, None)));
    assert_eq!(night.trace(), frozen("night-whole", &[]));
}

/// N2-I13: a stream too small for one instant's facts is still released `lagged` by delivery's own rule, and the
/// advance ends `disconnected`, its reason named.
#[test]
fn a_lagged_release_ends_the_advance_disconnected_and_named() {
    let mut night = night(
        &scratch("advance-lagged"),
        Setup {
            perceived_backlog: 1,
            ..Setup::default()
        },
    );
    let status = night.advanced(6, true, 100_000);
    assert_eq!(
        status.ended,
        Some((AdvanceOutcome::Disconnected, Some(EndReason::Lagged)))
    );
}

/// A save that can no longer be written stops the world: the advance ends `failed` and is not retried.
#[test]
fn a_world_that_stopped_mid_advance_fails_it_and_never_retries() {
    let mut night = night(&scratch("advance-failed"), Setup::default());
    let _ = night.advanced(7, true, 1);
    night.runtime.stopped = Some("the save is gone".to_owned());
    let reached = night.runtime.world.world().now();
    night.pump(true, 1);
    assert!(night.runtime.advancing.is_none());
    night.runtime.advance_slice();
    assert_eq!(
        night.runtime.world.world().now(),
        reached,
        "nothing further"
    );
    let last = night
        .runtime
        .operations
        .last()
        .map(|record| record.event.clone());
    assert!(
        !matches!(
            last,
            Some(mineworld_persistence::OperationEvent::AdvanceEnded {
                outcome: AdvanceOutcome::Woke,
                ..
            })
        ),
        "never reported as morning: {last:?}"
    );
}

fn op(value: u128) -> mineworld_persistence::OperationId {
    mineworld_persistence::OperationId::from_raw(value)
}
