//! Snapshot retention and log sealing on a real save (`docs/DECISIONS.md` `ARC-81`, `ARC-89`; designs
//! `.structured-coding/plans/mvp0/pr-s6-save-retention.md` D-SR-3, ASR-10 and
//! `pr-sr-b-log-compression.md` D-SRB-2, ASRB-10).
//!
//! ```text
//! the sweep        interval 8 (anchors every 512) through 3 · 4 096 + 5 revisions: after every commit
//!                  the stored snapshots are exactly the rule's set, and resume's snapshot is never
//!                  more than one interval behind the head
//! sealing sweep    interval 2 (anchors every 128) through 3 000 revisions: after every commit the
//!                  sealed blocks are exactly the sealing rule's, the live tail is every later revision
//!                  once, and a resume at that head decodes no block; around every anchor, every
//!                  revision and every fact appears exactly once across blocks and tail
//! a checkpoint     a clean shutdown's off-lattice snapshot resumes with nothing re-executed, and is
//!                  retired by the next scheduled snapshot
//! a new interval   a save resumed under another interval keeps genesis and the anchors both
//!                  lattices share, and converges on the new lattice's set
//! verify_from      from genesis and from every retained anchor to the head (ASR-4 at test scale);
//!                  a revision not kept is refused by name; a bad later snapshot is seen
//! ```
//!
//! The expected sets are computed here from the rules as `ARC-81` and `ARC-89` state them, not by the
//! crate's own functions, and compared with what the save actually holds.

mod support;

use std::collections::BTreeSet;
use std::rc::Rc;

use mineworld_contracts::EntityId;
use mineworld_persistence::backend::{
    FactRow, Facts, ManifestRow, RevisionRow, Revisions, SealedSpan,
};
use mineworld_persistence::{
    Creation, Durability, PersistError, PersistenceBackend, PersistentWorld, SqliteBackend,
    WorldRevision, verify, verify_from,
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

/// A read-only backend sharing one connection with the test, so that after a resume the test can ask
/// that connection how many block frames it decompressed (`ARC-89` I-SRB-4). Resume only reads.
struct Shared(Rc<SqliteBackend>);

impl PersistenceBackend for Shared {
    fn initialize(&mut self, _: &ManifestRow, _: &RevisionRow) -> Result<(), PersistError> {
        unreachable!("resume never initializes")
    }
    fn manifest(&self) -> Result<ManifestRow, PersistError> {
        self.0.manifest()
    }
    fn head(&self) -> Result<WorldRevision, PersistError> {
        self.0.head()
    }
    fn commit(&mut self, _: &RevisionRow) -> Result<(), PersistError> {
        unreachable!("resume never commits")
    }
    fn revisions_after(&self, after: WorldRevision) -> Result<Revisions<'_>, PersistError> {
        self.0.revisions_after(after)
    }
    fn facts_after(&self, after: Option<u64>) -> Result<Facts<'_>, PersistError> {
        self.0.facts_after(after)
    }
    fn facts_of(&self, revision: WorldRevision) -> Result<Vec<FactRow>, PersistError> {
        self.0.facts_of(revision)
    }
    fn last_facts(&self, count: usize) -> Result<Vec<FactRow>, PersistError> {
        self.0.last_facts(count)
    }
    fn latest_snapshot(
        &self,
        at_most: WorldRevision,
    ) -> Result<Option<(WorldRevision, Vec<u8>)>, PersistError> {
        self.0.latest_snapshot(at_most)
    }
    fn snapshot_revisions(&self) -> Result<Vec<WorldRevision>, PersistError> {
        self.0.snapshot_revisions()
    }
    fn snapshot_at(&self, revision: WorldRevision) -> Result<Option<Vec<u8>>, PersistError> {
        self.0.snapshot_at(revision)
    }
    fn highest_action_id(&self) -> Result<Option<u64>, PersistError> {
        self.0.highest_action_id()
    }
    fn sealed(&self) -> Result<Vec<SealedSpan>, PersistError> {
        self.0.sealed()
    }
    fn checkpoint(&mut self, _: WorldRevision, _: &[u8]) -> Result<(), PersistError> {
        unreachable!("resume never checkpoints")
    }
}

/// `ARC-89`'s blocks for head `n` at anchor interval `anchor`, from the rule as stated:
/// (k·A, (k + 1)·A] for every k with (k + 2)·A ≤ n.
fn sealing_rule(n: u64, anchor: u64) -> Vec<(u64, u64)> {
    (0..)
        .take_while(|k| (k + 2) * anchor <= n)
        .map(|k| (k * anchor + 1, (k + 1) * anchor))
        .collect()
}

/// Where every row of the log is, read by raw SQL: the live journal's first and last revision and its
/// row count.
fn live_journal(scratch: &Scratch) -> (Option<u64>, Option<u64>, u64) {
    let connection =
        rusqlite::Connection::open(SqliteBackend::file(scratch.path())).expect("opens");
    let (first, last, count): (Option<i64>, Option<i64>, i64) = connection
        .query_row(
            "SELECT MIN(revision), MAX(revision), COUNT(*) FROM journal",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("reads");
    let raw = |value: i64| u64::try_from(value).expect("a revision");
    (first.map(raw), last.map(raw), raw(count))
}

/// The whole log through both cursors: every revision 1 … `head` once and in order, and every fact once
/// — the facts of the revision stream, in order, are exactly the fact stream. Returns the fact count.
fn every_row_once(reader: &SqliteBackend, head: u64) -> u64 {
    let mut facts_by_revision = Vec::new();
    let mut revisions = 0;
    for stored in reader
        .revisions_after(WorldRevision::from_raw(0))
        .expect("a cursor")
    {
        let stored = stored.expect("reads");
        revisions += 1;
        assert_eq!(stored.revision.raw(), revisions, "revision order, no gap");
        facts_by_revision.extend(stored.facts.into_iter().map(|fact| fact.id));
    }
    assert_eq!(revisions, head, "every revision to the head");
    let facts: Vec<u64> = reader
        .facts_after(None)
        .expect("a cursor")
        .map(|fact| fact.expect("reads").id)
        .collect();
    assert_eq!(
        facts, facts_by_revision,
        "every fact once, in EventId order"
    );
    assert!(
        facts.windows(2).all(|pair| pair[0] < pair[1]),
        "strictly increasing ids"
    );
    facts.len() as u64
}

#[test]
fn every_commit_leaves_exactly_the_sealing_rule_s_blocks_and_resume_reads_none() {
    // ASRB-10: interval 2, so anchors every 128 revisions and the first seal at 256.
    const INTERVAL: u64 = 2;
    const ANCHOR: u64 = 64 * INTERVAL;
    const END: u64 = 3_000;
    let scratch = Scratch::new("sealing-sweep");
    let (mut persisted, people) = create(&scratch, INTERVAL);
    let reader = Rc::new(open(&scratch));
    let steps = script(usize::try_from(END).expect("small"), 10, 1);
    let (mut commits, mut resumes, mut full_checks) = (0_u64, 0_u64, 0_u64);
    for each in &steps {
        if persisted.revision().raw() >= END {
            break;
        }
        step(&mut persisted, &people, each, |persisted| {
            let head = persisted.revision().raw();
            let spans: Vec<(u64, u64)> = reader
                .sealed()
                .expect("reads")
                .iter()
                .map(|block| (block.span.first.raw(), block.span.last.raw()))
                .collect();
            assert_eq!(
                spans,
                sealing_rule(head, ANCHOR),
                "at r{head}: the rule's blocks"
            );
            let sealed_through = spans.last().map_or(0, |(_, last)| *last);
            assert_eq!(
                live_journal(&scratch),
                (Some(sealed_through + 1), Some(head), head - sealed_through),
                "at r{head}: the live tail is every revision after the blocks, once"
            );
            if head.is_multiple_of(ANCHOR) || head % ANCHOR == 1 {
                every_row_once(&reader, head);
                full_checks += 1;
            }
            let before = reader.blocks_decoded();
            let (_, how) =
                PersistentWorld::resume(Box::new(Shared(Rc::clone(&reader))), composed())
                    .expect("resumes");
            assert_eq!(how.head.raw(), head);
            assert_eq!(
                reader.blocks_decoded(),
                before,
                "at r{head}: resume decoded no block (snapshot {})",
                how.snapshot
            );
            resumes += 1;
            commits += 1;
        });
    }
    let head = persisted.revision().raw();
    drop(persisted);
    // Located: the sweep sealed many blocks and checked every commit.
    assert!(head >= END, "the sweep reached r{head}");
    assert_eq!(commits, head - 1);
    assert_eq!(resumes, commits);
    let blocks = reader.sealed().expect("reads").len() as u64;
    assert_eq!(blocks, head / ANCHOR - 1, "{blocks} blocks at r{head}");
    assert!(
        blocks >= 20 && full_checks >= 40,
        "{blocks} blocks, {full_checks} full checks"
    );
    let facts = every_row_once(&reader, head);
    let verified = verify(&open(&scratch), composed()).expect("the sealed save verifies");
    assert_eq!((verified.revisions, verified.facts), (head, facts));
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
fn verification_from_every_retained_anchor_reaches_the_head_and_sees_a_later_bad_snapshot() {
    // Interval 2: anchors every 128 revisions, so a short run holds several (ASR-4 at test scale).
    let scratch = Scratch::new("retention-verify-from");
    let (mut persisted, people) = create(&scratch, 2);
    let steps = script(1_000, 10, 1);
    let mut index = 0;
    while persisted.revision().raw() < 700 {
        step(&mut persisted, &people, &steps[index], |_| {});
        index += 1;
    }
    let head = persisted.revision().raw();
    drop(persisted);

    let reader = open(&scratch);
    let held = stored(&reader);
    let anchors: Vec<u64> = held
        .iter()
        .copied()
        .filter(|revision| *revision > 1 && revision.is_multiple_of(128))
        .collect();
    assert_eq!(anchors, [128, 256, 384, 512, 640], "located: five anchors");
    let facts_after = |from: u64| -> u64 {
        (from + 1..=head)
            .map(|revision| {
                reader
                    .facts_of(WorldRevision::from_raw(revision))
                    .expect("reads")
                    .len() as u64
            })
            .sum()
    };
    for from in std::iter::once(1).chain(anchors.iter().copied()) {
        let verified = verify_from(&reader, composed(), WorldRevision::from_raw(from))
            .unwrap_or_else(|error| panic!("from r{from}: {error}"));
        assert_eq!(verified.head.raw(), head);
        assert_eq!(
            verified.revisions,
            head - from,
            "from r{from}: every later revision"
        );
        assert_eq!(
            verified.facts,
            facts_after(from),
            "from r{from}: every later fact"
        );
        assert_eq!(
            verified.snapshots,
            held.iter().filter(|revision| **revision > from).count() as u64,
            "from r{from}: every later snapshot the save holds"
        );
    }

    // A revision the rule did not keep is refused by name, with what the save does hold.
    let refused = verify_from(&reader, composed(), WorldRevision::from_raw(130));
    assert!(
        matches!(&refused, Err(PersistError::NoSnapshotAt { revision, retained })
            if revision.raw() == 130
                && retained.iter().map(|kept| kept.raw()).collect::<BTreeSet<_>>() == held),
        "{refused:?}"
    );
    drop(reader);

    // Genesis' frame written over the anchor at 512: from 384 the re-execution reaches 512 and the
    // stored state there disagrees with history.
    let connection =
        rusqlite::Connection::open(SqliteBackend::file(scratch.path())).expect("opens");
    let changed = connection
        .execute(
            "UPDATE snapshots SET snapshot = (SELECT snapshot FROM snapshots WHERE revision = 1)
             WHERE revision = 512",
            [],
        )
        .expect("executes");
    assert_eq!(changed, 1);
    drop(connection);
    let verified = verify_from(&open(&scratch), composed(), WorldRevision::from_raw(384));
    assert!(
        matches!(&verified, Err(PersistError::SnapshotDisagreesWithHistory { revision })
            if revision.raw() == 512),
        "{verified:?}"
    );
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
