//! The queue's ordering rule, tested where the queue is: inside the crate, because the schedule is
//! deliberately not reachable from outside it (`DEP-6`'s isolating interface). Everything a world
//! does with the schedule is tested from `kernel/tests/schedule.rs`.
//!
//! The expected orders here never come from the schedule: they are computed by an independent
//! oracle — a stable sort of `(instant, insertion index)` — so a schedule that ordered wrongly could
//! not also produce the expectation it is checked against.

use mineworld_contracts::{
    Causation, Event, EventSchemaVersion, EventTypeId, SystemId, Visibility, WorldTime,
};
use serde::{Deserialize, Serialize};

use super::{Schedule, Scheduled, ScheduledEntry, Sequence};
use crate::error::KernelError;
use crate::system::{Cause, Deferral, Emission};

#[derive(Serialize, Deserialize)]
struct Marked;

impl Event for Marked {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("marked");
    const OWNER: SystemId = SystemId::from_static("marker");
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// An entry that carries its own insertion index in its payload, so the drained order can be read
/// back as the sequence of indices that went in.
fn item(index: usize, at: WorldTime) -> Scheduled {
    let emission = Emission::new::<Marked>(index.to_le_bytes().to_vec(), Visibility::Public);
    let cause = Cause {
        caused_by: Causation::SystemTick {
            system: SystemId::from_static("marker"),
        },
        decision: None,
    };
    Scheduled::Fact(Deferral::new(
        at,
        emission,
        SystemId::from_static("marker"),
        &cause,
    ))
}

fn index_of(item: &Scheduled) -> usize {
    let Scheduled::Fact(deferral) = item;
    let bytes: [u8; 8] = deferral
        .emission()
        .record()
        .payload()
        .as_slice()
        .try_into()
        .expect("an index payload is eight bytes");
    usize::from_le_bytes(bytes)
}

/// Drains the schedule instant by instant, the way a world does, returning the indices in firing
/// order and the instants visited.
fn drain(schedule: &mut Schedule) -> (Vec<usize>, Vec<WorldTime>) {
    let mut order = Vec::new();
    let mut instants = Vec::new();
    while let Some(next) = schedule.next_instant() {
        instants.push(next);
        while let Some((_, item)) = schedule.pop_at(next) {
            order.push(index_of(&item));
        }
    }
    (order, instants)
}

/// A small deterministic generator, so the 10,000-entry test is reproducible and needs no
/// dependency. Its quality does not matter; only that it scatters instants and repeats some.
fn scatter(state: &mut u64) -> i64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    i64::try_from((*state >> 33) % 500).expect("below 500 fits")
}

#[test]
fn entries_due_at_one_instant_fire_in_the_order_they_were_queued() {
    let mut schedule = Schedule::new();
    let t = WorldTime::from_seconds(10);
    for index in 0..5 {
        schedule.insert(t, item(index, t)).expect("queued");
    }
    let (order, instants) = drain(&mut schedule);
    assert_eq!(order, vec![0, 1, 2, 3, 4]);
    assert_eq!(instants, vec![t], "five entries, one instant");
}

#[test]
fn an_earlier_instant_fires_first_whatever_order_it_was_queued_in() {
    let mut schedule = Schedule::new();
    let late = WorldTime::from_seconds(900);
    let early = WorldTime::from_seconds(5);
    schedule.insert(late, item(0, late)).expect("queued");
    schedule.insert(early, item(1, early)).expect("queued");
    schedule.insert(late, item(2, late)).expect("queued");

    let (order, instants) = drain(&mut schedule);
    assert_eq!(order, vec![1, 0, 2]);
    assert_eq!(instants, vec![early, late]);
}

/// Idle time is not visited: the schedule's next instant after 10 is 1,000,000, with nothing in
/// between to step through.
#[test]
fn the_next_instant_skips_everything_in_between() {
    let mut schedule = Schedule::new();
    let first = WorldTime::from_seconds(10);
    let far = WorldTime::from_seconds(1_000_000);
    schedule.insert(far, item(0, far)).expect("queued");
    schedule.insert(first, item(1, first)).expect("queued");

    assert_eq!(schedule.next_instant(), Some(first));
    assert!(schedule.pop_at(first).is_some());
    assert_eq!(
        schedule.pop_at(first),
        None,
        "nothing else is due at the first instant"
    );
    assert_eq!(schedule.next_instant(), Some(far));
}

/// Ten thousand entries over five hundred instants, inserted in scattered order, drain exactly in
/// `(instant, insertion order)` — checked against a stable sort that knows nothing of the schedule.
#[test]
fn ten_thousand_scattered_entries_drain_in_instant_then_insertion_order() {
    const ENTRIES: usize = 10_000;
    let mut state = 0x5eed_u64;
    let mut schedule = Schedule::new();
    let mut oracle: Vec<(WorldTime, usize)> = Vec::with_capacity(ENTRIES);
    for index in 0..ENTRIES {
        let at = WorldTime::from_seconds(scatter(&mut state));
        schedule.insert(at, item(index, at)).expect("queued");
        oracle.push((at, index));
    }
    // Stable: equal instants keep insertion order, which is the rule under test.
    oracle.sort_by_key(|(at, _)| *at);
    let expected: Vec<usize> = oracle.iter().map(|(_, index)| *index).collect();

    let (order, instants) = drain(&mut schedule);
    assert_eq!(order.len(), ENTRIES, "every entry fired exactly once");
    assert_eq!(order, expected);

    // Locate before trusting the comparison (`ARC-23`): the instants really were scattered and
    // repeated, so the tie-break was exercised rather than vacuously satisfied.
    let distinct: std::collections::BTreeSet<WorldTime> =
        oracle.iter().map(|(at, _)| *at).collect();
    assert_eq!(instants.len(), distinct.len());
    assert!(
        distinct.len() > 400 && distinct.len() <= 500,
        "instants spread over the generator's range of 500, got {}",
        distinct.len()
    );
    assert!(
        ENTRIES / distinct.len() >= 10,
        "on average many entries share an instant, so insertion order decided their order"
    );
}

/// A schedule saved and restored is the same schedule: same entries, same order, and the counter
/// continues where it stopped, so an entry queued after restoring sorts after every saved one at
/// its instant.
#[test]
fn a_restored_schedule_continues_where_the_saved_one_stopped() {
    let mut saved = Schedule::new();
    let t = WorldTime::from_seconds(50);
    for index in 0..3 {
        saved.insert(t, item(index, t)).expect("queued");
    }
    let bytes = serde_json::to_vec(&saved.entries()).expect("entries serialize");
    let entries: Vec<ScheduledEntry> = serde_json::from_slice(&bytes).expect("entries parse");

    let mut restored =
        Schedule::restore(WorldTime::from_seconds(40), saved.next_sequence(), entries)
            .expect("a schedule's own snapshot restores");
    assert_eq!(restored, saved);

    restored
        .insert(t, item(3, t))
        .expect("queued after restoring");
    let (order, _) = drain(&mut restored);
    assert_eq!(order, vec![0, 1, 2, 3]);
}

#[test]
fn a_snapshot_that_no_schedule_could_have_produced_is_refused() {
    let mut saved = Schedule::new();
    let t = WorldTime::from_seconds(50);
    saved.insert(t, item(0, t)).expect("queued");
    saved.insert(t, item(1, t)).expect("queued");
    let entries = saved.entries();
    let next = saved.next_sequence();

    assert_eq!(
        Schedule::restore(WorldTime::from_seconds(60), next, entries.clone()),
        Err(KernelError::PersistedEntryBeforeNow {
            at: t,
            now: WorldTime::from_seconds(60)
        }),
        "an entry a world would already have fired"
    );

    assert_eq!(
        Schedule::restore(t, 2, entries.clone()),
        Err(KernelError::PersistedSequenceOutsideCounter {
            sequence: Sequence(2),
            next: 2
        }),
        "a counter that would hand out a sequence already used"
    );

    let mut repeated = entries.clone();
    repeated[1].sequence = repeated[0].sequence;
    repeated[1].at = WorldTime::from_seconds(70);
    assert_eq!(
        Schedule::restore(t, next, repeated),
        Err(KernelError::PersistedSequenceRepeated {
            sequence: Sequence(1)
        }),
        "one sequence on two entries, even at different instants"
    );

    assert_eq!(
        Schedule::restore(t, 0, Vec::new()),
        Err(KernelError::PersistedSequenceCounterTooLow { next: 0, first: 1 })
    );
}
