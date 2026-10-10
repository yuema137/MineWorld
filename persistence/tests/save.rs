//! A world and its save, in one process: create, drive, drop, resume, verify — and every way a save
//! can fail to be trusted (step-06 C3).
//!
//! Every test drives a real `PersistentWorld` against a real SQLite file. An in-memory twin — the bare
//! kernel, driven with the same calls — is the independent oracle for "the same world": what the save
//! rebuilds is compared with what a world that never touched a save became.

mod support;

use std::cell::Cell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use mineworld_contracts::{
    ActionResult, EntityId, Event, EventEnvelope, EventSchemaVersion, EventTypeId, Rejection,
    SystemId, Visibility,
};
use mineworld_kernel::{
    Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion, World,
    WorldView,
};
use mineworld_persistence::backend::{FactRow, ManifestRow, RevisionRow};
use mineworld_persistence::{
    Creation, Durability, PersistError, PersistenceBackend, PersistentWorld, SqliteBackend,
    WorldRevision, verify,
};
use support::{
    Echo, Ledger, Noted, Scratch, Step, assembled, composed, encode, genesis_facts, script, t,
};

const INSTANCE: u128 = 0x5eed_0000_0000_0000_0000_0000_0000_0007;
const INTERVAL: u64 = 16;
const STEPS: usize = 151;

fn create(scratch: &Scratch) -> (PersistentWorld, Vec<EntityId>) {
    let backend = SqliteBackend::create(scratch.path(), Durability::ProcessCrash).expect("creates");
    let (world, people) = assembled();
    let (persisted, _) = PersistentWorld::create(
        Box::new(backend),
        world,
        Creation {
            instance: INSTANCE,
            pack: "test".to_owned(),
            at: t(0),
            facts: genesis_facts(&people),
        },
    )
    .expect("a save is created");
    (persisted.snapshot_every(INTERVAL), people)
}

fn open(scratch: &Scratch) -> Box<dyn PersistenceBackend> {
    Box::new(SqliteBackend::open(scratch.path(), Durability::ProcessCrash).expect("opens"))
}

/// The twin: the bare kernel, assembled and begun exactly as `create` does.
fn twin() -> (World, Vec<EventEnvelope>) {
    let (mut world, people) = assembled();
    let facts = world.genesis(t(0), genesis_facts(&people)).expect("begins");
    (world, facts)
}

/// Drives a persisted world: advance to each step's instant, then dispatch it. Returns the facts.
fn drive(
    persisted: &mut PersistentWorld,
    people: &[EntityId],
    steps: &[Step],
) -> Vec<EventEnvelope> {
    let mut facts = Vec::new();
    for step in steps {
        facts.extend(
            persisted
                .advance_to(t(step.at))
                .expect("advances")
                .into_events(),
        );
        match persisted.dispatch(&step.intent(people), t(step.at)) {
            Ok(dispatched) => facts.extend(dispatched.events().to_vec()),
            Err(error) => panic!("dispatch failed: {error}"),
        }
    }
    facts
}

/// Drives the twin with the same calls. Returns the facts and how many advances fired something.
fn drive_twin(world: &mut World, people: &[EntityId], steps: &[Step]) -> (Vec<EventEnvelope>, u64) {
    let (mut facts, mut fired) = (Vec::new(), 0);
    for step in steps {
        let advanced = world.advance_to(t(step.at)).expect("advances");
        if advanced.instants() > 0 {
            fired += 1;
        }
        facts.extend(advanced.into_events());
        facts.extend(
            world
                .dispatch(&step.intent(people), t(step.at))
                .expect("dispatches")
                .events()
                .to_vec(),
        );
    }
    (facts, fired)
}

fn snapshot_bytes(world: &World) -> Vec<u8> {
    encode(&world.snapshot().expect("snapshots"))
}

// ---------------------------------------------------------------------------------------------
// a save rebuilds the world it saved
// ---------------------------------------------------------------------------------------------

#[test]
fn a_resumed_world_is_the_world_that_was_saved_and_continues_like_its_twin() {
    let scratch = Scratch::new("resume");
    // 151 steps: the head then falls between two snapshots (asserted below), so resuming has to
    // re-execute a tail rather than merely load.
    let steps = script(STEPS, 10, 1);
    let (mut persisted, people) = create(&scratch);
    let saved_facts = drive(&mut persisted, &people, &steps);
    let head = persisted.revision();
    drop(persisted);

    let (mut world, mut twin_facts) = twin();
    let (driven, fired) = drive_twin(&mut world, &people, &steps);
    twin_facts.extend(driven);

    // Located first (ARC-23): genesis, one revision per request whatever its answer, one per advance
    // that fired — counted on the twin, which knows nothing about saves.
    assert_eq!(
        head.raw(),
        1 + STEPS as u64 + fired,
        "the journal holds exactly the inputs that moved it"
    );
    assert!(fired > 20, "the script exercises advances: {fired}");
    assert_eq!(
        encode(&saved_facts),
        encode(&twin_facts[4..]),
        "persisting changed no fact"
    );

    let (mut resumed, how) = PersistentWorld::resume(open(&scratch), composed()).expect("resumes");
    let expected_snapshot = head.raw() - head.raw() % INTERVAL;
    assert_eq!(how.head, head);
    assert_eq!(
        how.snapshot.raw(),
        expected_snapshot,
        "the newest snapshot was read"
    );
    assert_eq!(
        how.replayed,
        head.raw() % INTERVAL,
        "and the tail after it re-executed"
    );
    assert!(
        how.replayed > 0,
        "the tail is not empty, so re-execution was exercised"
    );
    assert_eq!(resumed.instance(), INSTANCE, "the same world, by identity");
    assert_eq!(
        snapshot_bytes(resumed.world()),
        snapshot_bytes(&world),
        "the rebuilt state is the twin's, byte for byte"
    );

    let more = script(40, 10 + 37 * STEPS as i64, 1_000);
    let after_resume = drive(&mut resumed, &people, &more);
    let (after_twin, _) = drive_twin(&mut world, &people, &more);
    assert!(
        after_twin.len() > 40,
        "the continuation records facts: {}",
        after_twin.len()
    );
    assert_eq!(
        after_resume[0].id().raw(),
        u64::try_from(twin_facts.len()).expect("small") + 1,
        "identity continues at the next EventId"
    );
    assert_eq!(
        encode(&after_resume),
        encode(&after_twin),
        "and the facts continue identically"
    );
    assert_eq!(snapshot_bytes(resumed.world()), snapshot_bytes(&world));
}

#[test]
fn verification_re_executes_every_revision_and_every_snapshot_from_genesis() {
    let scratch = Scratch::new("verify");
    let steps = script(120, 10, 1);
    let (mut persisted, people) = create(&scratch);
    drive(&mut persisted, &people, &steps);
    let head = persisted.revision();
    drop(persisted);

    let (mut world, mut twin_facts) = twin();
    twin_facts.extend(drive_twin(&mut world, &people, &steps).0);

    let verified = verify(open(&scratch).as_ref(), composed()).expect("the history reproduces");
    assert_eq!(verified.head, head);
    assert_eq!(
        verified.revisions,
        head.raw(),
        "every revision, genesis included"
    );
    assert_eq!(
        verified.facts,
        u64::try_from(twin_facts.len()).expect("small"),
        "every fact the twin recorded was compared"
    );
    // Retention (ARC-81): below the first anchor (64 · INTERVAL) the save keeps genesis and the
    // newest two scheduled snapshots, and each of them equals the history's state.
    assert!(
        head.raw() / INTERVAL > 2,
        "pruning has happened: head {head}"
    );
    assert!(
        head.raw() < 64 * INTERVAL,
        "and no anchor exists yet: head {head}"
    );
    assert_eq!(
        verified.snapshots, 3,
        "genesis and the newest two scheduled snapshots, each equal to the history's state"
    );
}

// ---------------------------------------------------------------------------------------------
// what is journaled
// ---------------------------------------------------------------------------------------------

#[test]
fn every_request_is_a_revision_and_an_idle_advance_is_not() {
    let scratch = Scratch::new("journaling");
    let (mut persisted, people) = create(&scratch);
    let begun = persisted.revision();
    assert_eq!(begun, WorldRevision::GENESIS);

    let _ = persisted.advance_to(t(50)).expect("advances");
    assert_eq!(
        persisted.revision(),
        begun,
        "nothing was due: the clock moved, nothing else"
    );

    let at = |seconds, action, text: &str| Step {
        at: seconds,
        action: mineworld_contracts::ActionId::from_raw(action),
        actor: 0,
        target: Some(1),
        text: Some(text.to_owned()),
    };
    let rejected = persisted
        .dispatch(&at(60, 1, "").intent(&people), t(60))
        .expect("answered");
    assert_eq!(
        *rejected.result(),
        ActionResult::Rejected(Rejection::PreconditionFailed)
    );
    assert_eq!(
        persisted.revision().raw(),
        2,
        "a rejected request is journaled"
    );

    let fault = persisted
        .dispatch(&at(70, 2, "fault").intent(&people), t(70))
        .expect_err("the system breaks its contract");
    assert!(
        matches!(
            fault,
            PersistError::Kernel(KernelError::ActionNotResolvedBySystem { .. })
        ),
        "{fault:?}"
    );
    assert_eq!(
        persisted.revision().raw(),
        3,
        "a fault is journaled as the request's outcome"
    );

    let _ = persisted
        .dispatch(&at(80, 3, "kept").intent(&people), t(80))
        .expect("accepted");
    let _ = persisted.advance_to(t(500)).expect("the reminder fires");
    assert_eq!(
        persisted.revision().raw(),
        5,
        "and an advance that fired something is a revision"
    );
    assert_eq!(persisted.highest_action_id().expect("reads"), Some(3));
    let world_now = persisted.world().now();
    drop(persisted);

    let (mut resumed, how) = PersistentWorld::resume(open(&scratch), composed()).expect("resumes");
    assert_eq!(
        (how.snapshot.raw(), how.replayed),
        (1, 4),
        "the fault re-executed to the same error"
    );
    assert_eq!(
        resumed.world().now(),
        t(500),
        "the clock resumes at the last revision's instant"
    );
    assert_eq!(world_now, t(500));
    verify(open(&scratch).as_ref(), composed()).expect("and the history verifies");

    assert!(
        resumed.checkpoint().expect("checkpoints"),
        "at the head's instant a checkpoint is written"
    );
    let _ = resumed.advance_to(t(600)).expect("an idle advance");
    assert!(
        !resumed.checkpoint().expect("answers"),
        "after an idle advance it is not: the clock moved past every state the history produces (F-12)"
    );
    verify(open(&scratch).as_ref(), composed()).expect("and the checkpoint agrees with history");
}

#[test]
fn a_committed_revision_is_on_disk_when_dispatch_returns() {
    let scratch = Scratch::new("durable");
    let (mut persisted, people) = create(&scratch);
    for step in script(5, 10, 1) {
        let _ = persisted.advance_to(t(step.at)).expect("advances");
        let _ = persisted
            .dispatch(&step.intent(&people), t(step.at))
            .expect("answered");
        let other = SqliteBackend::open(scratch.path(), Durability::ProcessCrash).expect("opens");
        assert_eq!(
            other.head().expect("reads"),
            persisted.revision(),
            "a second connection sees the revision: committed, not buffered"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// state that existed only in memory is detected
// ---------------------------------------------------------------------------------------------

/// A system that breaks INV-7: it keeps a count in a process-wide variable rather than in a
/// component, and states it in every fact it emits. Exactly the state a save cannot carry.
struct Forgetful;

static REMEMBERED_ONLY_IN_MEMORY: AtomicU32 = AtomicU32::new(0);

impl SystemIdentity for Forgetful {
    const ID: SystemId = SystemId::from_static("forgetful");
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Counted {
    count: u32,
}

impl Event for Counted {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("counted");
    const OWNER: SystemId = Forgetful::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl System for Forgetful {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .emitting::<Counted>()
            .subscribing_to::<Noted>()
    }

    fn react(
        &self,
        _: &mut WorldView<'_, Self>,
        _: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let count = REMEMBERED_ONLY_IN_MEMORY.fetch_add(1, Ordering::SeqCst) + 1;
        Ok(vec![Emission::new::<Counted>(
            encode(&Counted { count }),
            Visibility::SystemInternal,
        )])
    }
}

fn forgetful_world() -> World {
    let mut world = composed();
    world.install(Forgetful).expect("installs");
    world
}

#[test]
fn state_kept_outside_the_stores_is_detected_as_a_divergence_at_the_revision_it_shows() {
    let scratch = Scratch::new("forgetful");
    let backend = SqliteBackend::create(scratch.path(), Durability::ProcessCrash).expect("creates");
    let mut world = forgetful_world();
    let people: Vec<EntityId> = support::PEOPLE
        .iter()
        .map(|key| {
            world
                .create_entity(
                    mineworld_contracts::EntityKey::new(*key).expect("a key"),
                    mineworld_contracts::EntityType::Person,
                )
                .expect("created")
        })
        .collect();
    let (persisted, _) = PersistentWorld::create(
        Box::new(backend),
        world,
        Creation {
            instance: INSTANCE,
            pack: "test".to_owned(),
            at: t(0),
            facts: genesis_facts(&people),
        },
    )
    .expect("creates");
    let mut persisted = persisted.snapshot_every(8);

    // Record, per revision, whether it recorded a `noted` fact — which is when the count shows.
    let mut shows = Vec::new();
    for step in script(30, 10, 1) {
        let _ = persisted.advance_to(t(step.at)).expect("advances");
        let before = persisted.revision();
        let dispatched = persisted
            .dispatch(&step.intent(&people), t(step.at))
            .expect("answered");
        if dispatched
            .events()
            .iter()
            .any(|fact| *fact.event_type() == Noted::EVENT_TYPE)
        {
            shows.push(WorldRevision::from_raw(before.raw() + 1));
        }
    }
    let head = persisted.revision();
    drop(persisted);

    let snapshot = head.raw() - head.raw() % 8;
    let first_after = *shows
        .iter()
        .find(|revision| revision.raw() > snapshot)
        .expect("the tail holds a revision that shows the count");
    let resumed = PersistentWorld::resume(open(&scratch), forgetful_world());
    assert!(
        matches!(&resumed, Err(PersistError::ReplayDiverged { revision, .. }) if *revision == first_after),
        "resume refuses at the first tail revision whose facts carry the count ({first_after}): {:?}",
        resumed.err()
    );

    let verified = verify(open(&scratch).as_ref(), forgetful_world());
    assert!(
        matches!(&verified, Err(PersistError::ReplayDiverged { revision, .. }) if *revision == WorldRevision::GENESIS),
        "verification refuses at genesis, whose four `born` facts the count already followed: {verified:?}"
    );
}

// ---------------------------------------------------------------------------------------------
// a damaged save is refused
// ---------------------------------------------------------------------------------------------

fn sql(scratch: &Scratch, statement: &str) {
    let connection =
        rusqlite::Connection::open(SqliteBackend::file(scratch.path())).expect("opens");
    let changed = connection.execute(statement, []).expect("executes");
    assert!(changed > 0, "the tampering touched a row: {statement}");
}

fn saved(scratch: &Scratch, steps: usize) -> WorldRevision {
    let (mut persisted, people) = create(scratch);
    drive(&mut persisted, &people, &script(steps, 10, 1));
    persisted.revision()
}

#[test]
fn an_altered_fact_or_snapshot_is_refused_where_it_was_altered() {
    let scratch = Scratch::new("tamper-fact");
    let head = saved(&scratch, 40);
    let altered: i64 = rusqlite::Connection::open(SqliteBackend::file(scratch.path()))
        .expect("opens")
        .query_row(
            "SELECT revision FROM facts WHERE event_id = (SELECT MAX(event_id) FROM facts)",
            [],
            |row| row.get(0),
        )
        .expect("the newest fact's revision");
    let altered = u64::try_from(altered).expect("a revision");
    assert!(
        altered > head.raw() - head.raw() % INTERVAL,
        "the altered fact ({altered}) lies after the newest snapshot, in the re-executed tail"
    );
    sql(
        &scratch,
        "UPDATE facts SET fact = CAST(REPLACE(CAST(fact AS TEXT), 'public', 'system_internal') AS BLOB)
         WHERE event_id = (SELECT MAX(event_id) FROM facts)",
    );
    let resumed = PersistentWorld::resume(open(&scratch), composed());
    assert!(
        matches!(&resumed, Err(PersistError::ReplayDiverged { revision, .. }) if revision.raw() == altered),
        "refused at exactly the revision whose fact was altered ({altered}): {:?}",
        resumed.err()
    );

    // A stored snapshot is a zstd frame of its JSON (ARC-81). (a) Bytes that are no frame at all are
    // damage, named with the revision — by resume when it is the snapshot resume reads, and by verify.
    let scratch = Scratch::new("tamper-snapshot-frame");
    let head = saved(&scratch, 40);
    let newest = open(&scratch)
        .latest_snapshot(head)
        .expect("reads")
        .expect("a snapshot")
        .0;
    assert!(
        newest.raw() > 1,
        "the newest snapshot is a scheduled one: {newest}"
    );
    sql(
        &scratch,
        &format!(
            "UPDATE snapshots SET snapshot = CAST('{{}}' AS BLOB) WHERE revision = {}",
            newest.raw()
        ),
    );
    let not_a_frame = format!("the snapshot at {newest} does not decompress");
    let resumed = PersistentWorld::resume(open(&scratch), composed());
    assert!(
        matches!(&resumed, Err(PersistError::Damaged { detail }) if detail.starts_with(&not_a_frame)),
        "{:?}",
        resumed.err()
    );
    let verified = verify(open(&scratch).as_ref(), composed());
    assert!(
        matches!(&verified, Err(PersistError::Damaged { detail }) if detail.starts_with(&not_a_frame)),
        "{verified:?}"
    );

    // (b) A valid frame of other JSON — genesis' snapshot copied over a later one — decompresses and
    // decodes, and disagrees with history at exactly that revision.
    let scratch = Scratch::new("tamper-snapshot-state");
    let head = saved(&scratch, 40);
    let newest = open(&scratch)
        .latest_snapshot(head)
        .expect("reads")
        .expect("a snapshot")
        .0;
    sql(
        &scratch,
        &format!(
            "UPDATE snapshots SET snapshot = (SELECT snapshot FROM snapshots WHERE revision = 1)
             WHERE revision = {}",
            newest.raw()
        ),
    );
    let verified = verify(open(&scratch).as_ref(), composed());
    assert!(
        matches!(&verified, Err(PersistError::SnapshotDisagreesWithHistory { revision }) if *revision == newest),
        "{verified:?}"
    );
}

#[test]
fn a_save_written_by_the_format_2_build_is_refused_by_name() {
    // Written by the build before SR (pr-s6-save-retention.md §14.1): uncompressed snapshot rows and
    // `format 2` in its manifest. Copied first, because opening a save in WAL mode writes beside it.
    let scratch = Scratch::new("format-2");
    let fixture = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/format-2"
    ));
    std::fs::copy(
        SqliteBackend::file(fixture),
        SqliteBackend::file(scratch.path()),
    )
    .expect("the fixture copies");
    let outdated = |result: Result<(), PersistError>| {
        assert!(
            matches!(
                result,
                Err(PersistError::SaveFormatOutdated {
                    saved: 2,
                    supported: 3
                })
            ),
            "{result:?}"
        );
    };
    outdated(PersistentWorld::resume(open(&scratch), composed()).map(|_| ()));
    outdated(verify(open(&scratch).as_ref(), composed()).map(|_| ()));
    let stored = open(&scratch).manifest().expect("the manifest row reads");
    assert_eq!(
        stored.format, 2,
        "the refusal read the fixture's own format"
    );
}

#[test]
fn a_save_of_another_format_or_another_composition_is_refused_by_name() {
    let scratch = Scratch::new("format");
    saved(&scratch, 3);
    sql(&scratch, "UPDATE manifest SET format = 4");
    assert!(matches!(
        PersistentWorld::resume(open(&scratch), composed()),
        Err(PersistError::SaveFormatTooNew {
            saved: 4,
            supported: 3
        })
    ));
    // Format 1 is S5's, written before a declaration recorded the owners of borrowed vocabularies
    // (ARC-26); format 2 stored snapshots uncompressed (ARC-81). Both are refused by name rather than
    // decoded on a guess.
    for older in [1, 2] {
        sql(&scratch, &format!("UPDATE manifest SET format = {older}"));
        assert!(matches!(
            PersistentWorld::resume(open(&scratch), composed()),
            Err(PersistError::SaveFormatOutdated { saved, supported: 3 }) if saved == older
        ));
    }

    let scratch = Scratch::new("composition");
    saved(&scratch, 3);
    let mut missing = World::new();
    missing.install(Ledger).expect("installs");
    assert!(matches!(
        PersistentWorld::resume(open(&scratch), missing),
        Err(PersistError::CompositionDiffers { position: 1, saved: Some(saved), running: None })
            if saved == Echo::ID
    ));
    let mut swapped = World::new();
    swapped.install(Echo).expect("installs");
    swapped.install(Ledger).expect("installs");
    assert!(matches!(
        PersistentWorld::resume(open(&scratch), swapped),
        Err(PersistError::CompositionDiffers { position: 0, .. })
    ));
}

struct Plain;
struct PlainNext;

impl SystemIdentity for Plain {
    const ID: SystemId = SystemId::from_static("plain");
}
impl SystemIdentity for PlainNext {
    const ID: SystemId = SystemId::from_static("plain");
}
impl System for Plain {
    const VERSION: SystemVersion = SystemVersion::new(1);
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
    }
}
impl System for PlainNext {
    const VERSION: SystemVersion = SystemVersion::new(2);
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
    }
}

fn plain_save(scratch: &Scratch, world: World) {
    let backend = SqliteBackend::create(scratch.path(), Durability::ProcessCrash).expect("creates");
    PersistentWorld::create(
        Box::new(backend),
        world,
        Creation {
            instance: INSTANCE,
            pack: "plain".to_owned(),
            at: t(0),
            facts: Vec::new(),
        },
    )
    .expect("creates");
}

fn with<S: System>(system: S) -> World {
    let mut world = World::new();
    world.install(system).expect("installs");
    world
}

#[test]
fn a_save_written_by_another_version_of_a_system_is_refused_whichever_way_it_differs() {
    let scratch = Scratch::new("newer");
    plain_save(&scratch, with(PlainNext));
    assert!(matches!(
        PersistentWorld::resume(open(&scratch), with(Plain)),
        Err(PersistError::PersistedSystemTooNew { saved, running, .. })
            if saved == SystemVersion::new(2) && running == SystemVersion::new(1)
    ));

    let scratch = Scratch::new("older");
    plain_save(&scratch, with(Plain));
    assert!(matches!(
        PersistentWorld::resume(open(&scratch), with(PlainNext)),
        Err(PersistError::PersistedSystemOutdated { saved, running, .. })
            if saved == SystemVersion::new(1) && running == SystemVersion::new(2)
    ));
}

#[test]
fn a_save_is_never_created_over_another_nor_opened_where_there_is_none() {
    let scratch = Scratch::new("exists");
    saved(&scratch, 1);
    assert!(matches!(
        SqliteBackend::create(scratch.path(), Durability::ProcessCrash),
        Err(PersistError::SaveExists { .. })
    ));
    let empty = Scratch::new("empty");
    assert!(matches!(
        SqliteBackend::open(empty.path(), Durability::ProcessCrash),
        Err(PersistError::NoSave { .. })
    ));
}

// ---------------------------------------------------------------------------------------------
// a world that cannot save stops
// ---------------------------------------------------------------------------------------------

/// A real SQLite backend whose commits can be made to fail on demand.
struct Failing {
    inner: SqliteBackend,
    fail: Rc<Cell<bool>>,
}

impl PersistenceBackend for Failing {
    fn initialize(
        &mut self,
        manifest: &ManifestRow,
        genesis: &RevisionRow,
    ) -> Result<(), PersistError> {
        self.inner.initialize(manifest, genesis)
    }
    fn manifest(&self) -> Result<ManifestRow, PersistError> {
        self.inner.manifest()
    }
    fn head(&self) -> Result<WorldRevision, PersistError> {
        self.inner.head()
    }
    fn commit(&mut self, revision: &RevisionRow) -> Result<(), PersistError> {
        if self.fail.get() {
            return Err(PersistError::Storage {
                detail: "disk full (injected)".to_owned(),
            });
        }
        self.inner.commit(revision)
    }
    fn journal_after(
        &self,
        after: WorldRevision,
    ) -> Result<Vec<(WorldRevision, Vec<u8>)>, PersistError> {
        self.inner.journal_after(after)
    }
    fn facts_of(&self, revision: WorldRevision) -> Result<Vec<FactRow>, PersistError> {
        self.inner.facts_of(revision)
    }
    fn last_facts(&self, count: usize) -> Result<Vec<FactRow>, PersistError> {
        self.inner.last_facts(count)
    }
    fn latest_snapshot(
        &self,
        at_most: WorldRevision,
    ) -> Result<Option<(WorldRevision, Vec<u8>)>, PersistError> {
        self.inner.latest_snapshot(at_most)
    }
    fn snapshot_revisions(&self) -> Result<Vec<WorldRevision>, PersistError> {
        self.inner.snapshot_revisions()
    }
    fn snapshot_at(&self, revision: WorldRevision) -> Result<Option<Vec<u8>>, PersistError> {
        self.inner.snapshot_at(revision)
    }
    fn highest_action_id(&self) -> Result<Option<u64>, PersistError> {
        self.inner.highest_action_id()
    }
    fn checkpoint(&mut self, revision: WorldRevision, snapshot: &[u8]) -> Result<(), PersistError> {
        self.inner.checkpoint(revision, snapshot)
    }
}

#[test]
fn a_world_whose_commit_failed_refuses_every_later_input() {
    let scratch = Scratch::new("failing");
    let fail = Rc::new(Cell::new(false));
    let backend = Failing {
        inner: SqliteBackend::create(scratch.path(), Durability::ProcessCrash).expect("creates"),
        fail: Rc::clone(&fail),
    };
    let (world, people) = assembled();
    let (mut persisted, _) = PersistentWorld::create(
        Box::new(backend),
        world,
        Creation {
            instance: INSTANCE,
            pack: "test".to_owned(),
            at: t(0),
            facts: genesis_facts(&people),
        },
    )
    .expect("creates");
    let steps = script(3, 10, 1);
    let _ = persisted
        .dispatch(&steps[0].intent(&people), t(steps[0].at))
        .expect("committed");

    fail.set(true);
    let refused = persisted.dispatch(&steps[1].intent(&people), t(steps[1].at));
    assert!(
        matches!(&refused, Err(PersistError::WorldAheadOfSave { revision, .. }) if revision.raw() == 3),
        "{:?}",
        refused.err()
    );
    fail.set(false);
    let still = persisted.dispatch(&steps[2].intent(&people), t(steps[2].at));
    assert!(
        matches!(still, Err(PersistError::WorldAheadOfSave { .. })),
        "the storage recovered, the world did not: it is ahead of its save"
    );
    assert_eq!(persisted.revision().raw(), 2);
    assert_eq!(
        open(&scratch).head().expect("reads").raw(),
        2,
        "nothing past revision 2 on disk"
    );
}
