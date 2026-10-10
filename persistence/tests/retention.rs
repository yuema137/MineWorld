//! Snapshot retention on a real save (`docs/DECISIONS.md` `ARC-81`; design
//! `.structured-coding/plans/mvp0/pr-s6-save-retention.md` D-SR-3, ASR-10).
//!
//! ```text
//! the sweep        interval 8 (anchors every 512) through 3 · 4 096 + 5 revisions: after every commit
//!                  the stored snapshots are exactly the rule's set, and resume's snapshot is never
//!                  more than one interval behind the head
//! a checkpoint     a clean shutdown's off-lattice snapshot resumes with nothing re-executed, and is
//!                  retired by the next scheduled snapshot
//! a new interval   a save resumed under another interval keeps genesis and the anchors both
//!                  lattices share, and converges on the new lattice's set
//! ```
//!
//! The expected sets are computed here from the rule as `ARC-81` states it, not by the crate's own
//! function, and compared with what the save actually holds.

mod support;

use std::collections::BTreeSet;

use mineworld_contracts::EntityId;
use mineworld_persistence::{
    Creation, Durability, PersistenceBackend, PersistentWorld, SqliteBackend, WorldRevision, verify,
};
use support::{Scratch, Step, assembled, composed, genesis_facts, script, t};

const INSTANCE: u128 = 0x5eed_0000_0000_0000_0000_0000_0000_0081;

fn create(scratch: &Scratch, interval: u64) -> (PersistentWorld, Vec<EntityId>) {
    let backend = SqliteBackend::create(scratch.path(), Durability::ProcessCrash).expect("creates");
    let (world, people) = assembled();
    let (persisted, _) = PersistentWorld::create(
        Box::new(backend),
        world,
        Creation {
            instance: INSTANCE,
            pack: "retention".to_owned(),
            at: t(0),
            facts: genesis_facts(&people),
        },
    )
    .expect("a save is created");
    (persisted.snapshot_every(interval), people)
}

fn open(scratch: &Scratch) -> SqliteBackend {
    SqliteBackend::open(scratch.path(), Durability::ProcessCrash).expect("opens")
}

/// The revisions the save holds a snapshot at, read from the file.
fn stored(reader: &SqliteBackend) -> BTreeSet<u64> {
    reader
        .snapshot_revisions()
        .expect("reads")
        .into_iter()
        .map(WorldRevision::raw)
        .collect()
}

/// `ARC-81`'s K(n) for a run whose only snapshots were scheduled ones at `interval`: genesis, every
/// multiple of 64 · interval up to n, n and n − interval.
fn rule(n: u64, interval: u64) -> BTreeSet<u64> {
    let anchor = 64 * interval;
    let mut kept: BTreeSet<u64> = (1..=n / anchor).map(|k| k * anchor).collect();
    kept.insert(1);
    kept.insert(n);
    if n > interval {
        kept.insert(n - interval);
    }
    kept
}

/// Drives one step — an advance, then a request — calling `after` after each call that committed.
fn step(
    persisted: &mut PersistentWorld,
    people: &[EntityId],
    step: &Step,
    mut after: impl FnMut(&PersistentWorld),
) {
    let before = persisted.revision();
    let _ = persisted.advance_to(t(step.at)).expect("advances");
    if persisted.revision() != before {
        after(persisted);
    }
    let _ = persisted.dispatch(&step.intent(people), t(step.at));
    after(persisted);
}

#[test]
fn every_commit_leaves_exactly_the_rule_s_snapshots_and_resume_s_one_is_always_near() {
    const INTERVAL: u64 = 8;
    const END: u64 = 3 * 4_096 + 5;
    let scratch = Scratch::new("retention-sweep");
    let (mut persisted, people) = create(&scratch, INTERVAL);
    let reader = open(&scratch);
    let (mut commits, mut retiring) = (0_u64, 0_u64);
    let mut previous = stored(&reader);
    let steps = script(usize::try_from(END).expect("small"), 10, 1);
    for each in &steps {
        if persisted.revision().raw() >= END {
            break;
        }
        step(&mut persisted, &people, each, |persisted| {
            let head = persisted.revision().raw();
            let now = stored(&reader);
            let scheduled = head - head % INTERVAL;
            let expected = if scheduled == 0 {
                BTreeSet::from([1])
            } else {
                rule(scheduled, INTERVAL)
            };
            assert_eq!(
                now, expected,
                "at r{head}: the save holds exactly K({scheduled})"
            );
            let (resume_from, _) = reader
                .latest_snapshot(persisted.revision())
                .expect("reads")
                .expect("resume always has a snapshot");
            assert!(
                resume_from.raw() == 1 || head - resume_from.raw() < INTERVAL,
                "at r{head}: resume's snapshot r{} is within one interval",
                resume_from.raw()
            );
            if previous.difference(&now).next().is_some() {
                retiring += 1;
            }
            previous = now;
            commits += 1;
        });
    }
    let head = persisted.revision().raw();
    drop(persisted);
    // Located (ARC-23): the sweep crossed three anchors, and pruning actually happened.
    assert!(head >= END, "the sweep reached r{head}");
    assert_eq!(
        commits,
        head - 1,
        "every revision after genesis was checked"
    );
    let anchors: Vec<u64> = stored(&reader)
        .into_iter()
        .filter(|revision| *revision > 1 && revision.is_multiple_of(64 * INTERVAL))
        .collect();
    assert_eq!(anchors.len(), usize::try_from(head / 512).expect("small"));
    assert!(
        retiring > 1_000,
        "commits that retired a snapshot: {retiring}"
    );

    let verified = verify(&open(&scratch), composed()).expect("the pruned save verifies");
    assert_eq!(verified.revisions, head);
    assert_eq!(
        verified.snapshots,
        u64::try_from(stored(&reader).len()).expect("small"),
        "every snapshot the save still holds was compared"
    );
}

#[test]
fn a_clean_shutdown_s_checkpoint_resumes_with_nothing_replayed_then_is_retired() {
    const INTERVAL: u64 = 16;
    let scratch = Scratch::new("retention-checkpoint");
    let (mut persisted, people) = create(&scratch, INTERVAL);
    let steps = script(400, 10, 1);
    let mut index = 0;
    while persisted.revision().raw() < 100 || persisted.revision().raw() % INTERVAL == 0 {
        step(&mut persisted, &people, &steps[index], |_| {});
        index += 1;
    }
    let head = persisted.revision();
    assert!(
        persisted.checkpoint().expect("checkpoints"),
        "at the head's instant"
    );
    drop(persisted);
    assert!(
        stored(&open(&scratch)).contains(&head.raw()),
        "the off-lattice checkpoint at {head} is stored"
    );

    let (resumed, how) =
        PersistentWorld::resume(Box::new(open(&scratch)), composed()).expect("resumes");
    assert_eq!(
        (how.snapshot, how.replayed),
        (head, 0),
        "resumed from the checkpoint with nothing re-executed"
    );
    let mut resumed = resumed.snapshot_every(INTERVAL);
    let next = head.raw() - head.raw() % INTERVAL + INTERVAL;
    while resumed.revision().raw() < next {
        step(&mut resumed, &people, &steps[index], |_| {});
        index += 1;
    }
    let scheduled = resumed.revision().raw() - resumed.revision().raw() % INTERVAL;
    assert_eq!(scheduled, next, "the next scheduled snapshot was committed");
    drop(resumed);
    assert_eq!(
        stored(&open(&scratch)),
        rule(next, INTERVAL),
        "the next scheduled snapshot retired the checkpoint"
    );
    verify(&open(&scratch), composed()).expect("and the save verifies");
}

#[test]
fn a_save_resumed_under_another_interval_converges_on_the_new_lattice() {
    let scratch = Scratch::new("retention-interval");
    let (mut persisted, people) = create(&scratch, 16);
    let steps = script(2_000, 10, 1);
    let mut index = 0;
    while persisted.revision().raw() < 1_100 {
        step(&mut persisted, &people, &steps[index], |_| {});
        index += 1;
    }
    let head = persisted.revision().raw();
    drop(persisted);
    let last16 = head - head % 16;
    assert_eq!(
        stored(&open(&scratch)),
        BTreeSet::from([1, 1_024, last16 - 16, last16]),
        "under interval 16 the anchor is 1 024"
    );

    let (resumed, _) =
        PersistentWorld::resume(Box::new(open(&scratch)), composed()).expect("resumes");
    let mut resumed = resumed.snapshot_every(8);
    let target = head - head % 8 + 16;
    while resumed.revision().raw() < target {
        step(&mut resumed, &people, &steps[index], |_| {});
        index += 1;
    }
    let now = resumed.revision().raw();
    drop(resumed);
    let last8 = now - now % 8;
    assert_eq!(
        stored(&open(&scratch)),
        BTreeSet::from([1, 1_024, last8 - 8, last8]),
        "under interval 8: genesis, the anchor both lattices share, and the newest two; \
         512 was retired under 16 and is not invented"
    );
    verify(&open(&scratch), composed()).expect("the save verifies");
}
