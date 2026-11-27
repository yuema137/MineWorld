//! Real SQLite lifecycle through a test-pack adapter. Capability is closed: this adapter owns
//! the same SDK calls and production comparison guard, not public custom-pack CLI loading.
use super::*;
use mineworld_contracts::{Event, EventEnvelope};
use mineworld_persistence::{
    Creation, Durability, PersistenceBackend, PersistentWorld, SqliteBackend, WorldRevision,
    format, verify,
};
use mineworld_worldpack::{PackError, configure::compare};
use schema::{SchemaComposed, SummaryProcess};
use std::path::Path;

const DURABILITY: Durability = Durability::PowerLoss;
fn composed() -> World {
    let mut world = World::new();
    world.install(SchemaPack).expect("install schema pack");
    world
}
fn assembled(authored: i64) -> (World, Vec<mineworld_kernel::Emission>) {
    let initial = context();
    let section = person(&*initial, &format!("schema:\n  mood: {authored}\n")).expect("decode");
    let (ada, tags) = (key("ada"), Tags::default());
    let subjects = [ErasedSubject {
        key: &ada,
        kind: ContentKind::Person,
        tags: &tags,
        authored: section.as_deref(),
    }];
    let resolved =
        SchemaPack::resolve_subjects(Some(&*initial), Some(&*configuration()), &subjects)
            .expect("resolve")
            .expect("resolution");
    let mut world = composed();
    let id = world
        .create_entity(ada.clone(), EntityType::Person)
        .expect("person");
    let ids = BTreeMap::from([(ada.clone(), id)]);
    let facts = {
        let read = world.read();
        let seeding = Seeding::new(&read, &ids);
        let mut facts = resolved
            .context()
            .seed_context(&seeding)
            .expect("final context fact");
        facts.extend(
            resolved
                .seed(&seeding, &ada, id)
                .expect("seed")
                .expect("subject fact"),
        );
        facts
    };
    (world, facts)
}
fn summary(world: &World) -> i64 {
    let read = world.read();
    let process = read
        .processes()
        .find(|p| p.owner() == &SchemaPack::ID)
        .expect("summary process");
    let bytes = process
        .state_for::<SummaryProcess>()
        .expect("summary state");
    serde_json::from_slice::<SchemaComposed>(bytes)
        .expect("summary decodes")
        .total
}

/// Host guard adapter: read genesis from the reopened backend, compare BEFORE resume/append.
fn guarded_resume(
    path: &Path,
    authored: i64,
) -> Result<(PersistentWorld, mineworld_persistence::Resumed), PackError> {
    let backend = SqliteBackend::open(path, DURABILITY).expect("reopen save");
    let saved: Vec<EventEnvelope> = backend
        .facts_of(WorldRevision::GENESIS)
        .expect("disk genesis")
        .iter()
        .map(|row| format::decode(&row.bytes, "fact").expect("decode disk fact"))
        .collect();
    let (_, facts) = assembled(authored);
    compare(
        &saved,
        &facts,
        &BTreeMap::from([(SchemaComposed::EVENT_TYPE, SchemaPack::ID)]),
    )
    .map_err(|drift| PackError::ConfigurationDrift {
        system: drift.system,
        saved: drift.saved,
        here: drift.here,
    })?;
    Ok(PersistentWorld::resume(Box::new(backend), composed()).expect("resume stored journal"))
}

/// Saved rows in canonical order; equality covers journal, facts, snapshots and head.
fn saved_rows(path: &Path) -> Vec<Vec<u8>> {
    let backend = SqliteBackend::open(path, DURABILITY).expect("open saved state");
    let mut rows = vec![backend.head().expect("head").raw().to_le_bytes().to_vec()];
    for stored in backend
        .revisions_after(WorldRevision::from_raw(0))
        .expect("journal cursor")
    {
        let stored = stored.expect("stored revision");
        rows.push(stored.revision.raw().to_le_bytes().to_vec());
        rows.push(stored.entry);
        rows.extend(stored.facts.into_iter().map(|fact| fact.bytes));
    }
    for revision in backend.snapshot_revisions().expect("snapshots") {
        rows.push(revision.raw().to_le_bytes().to_vec());
        rows.push(
            backend
                .snapshot_at(revision)
                .expect("snapshot")
                .expect("listed"),
        );
    }
    rows
}
fn digest(rows: &[Vec<u8>]) -> u64 {
    rows.iter()
        .flat_map(|r| {
            (r.len() as u64)
                .to_le_bytes()
                .into_iter()
                .chain(r.iter().copied())
        })
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
        })
}

#[test]
fn derived_context_survives_real_reopen_and_drift_refuses_without_saved_mutation() {
    let scratch = mineworld_test_support::scratch!(empty "final-context-restart");
    let (world, facts) = assembled(2);
    let backend = SqliteBackend::create(scratch.path(), DURABILITY).expect("create SQLite");
    let (mut saved, _) = PersistentWorld::create(
        Box::new(backend),
        world,
        Creation {
            instance: 0x5300,
            pack: "schema-context".to_owned(),
            at: WorldTime::EPOCH,
            facts,
        },
    )
    .expect("create persistent world");
    assert_eq!(
        summary(saved.world()),
        6,
        "authored mood 2 plus default patience 4"
    );
    let advanced = saved
        .advance_to(WorldTime::from_seconds(10))
        .expect("real scheduled advance");
    assert_eq!(advanced.instants(), 1);
    assert_eq!(saved.revision().raw(), 2);
    drop(saved);

    let (mut resumed, report) = guarded_resume(scratch.path(), 2).expect("unchanged accepted");
    assert_eq!(report.head.raw(), 2);
    assert_eq!(
        report.replayed, 1,
        "reexecute the advance after genesis snapshot"
    );
    assert_eq!(summary(resumed.world()), 6);
    let advanced = resumed
        .advance_to(WorldTime::from_seconds(20))
        .expect("continue after reopen");
    assert_eq!(advanced.instants(), 1);
    assert_eq!(resumed.revision().raw(), 3);
    drop(resumed);
    let verified = verify(
        &SqliteBackend::open(scratch.path(), DURABILITY).expect("open for verification"),
        composed(),
    )
    .expect("replay verifies");
    assert_eq!(verified.revisions, 3);
    assert_eq!(verified.facts, 2);

    let before = saved_rows(scratch.path());
    let before_digest = digest(&before);
    let error = match guarded_resume(scratch.path(), 3) {
        Err(error) => error,
        Ok(_) => panic!("changed authored-derived context must refuse before append"),
    };
    assert!(
        matches!(error, PackError::ConfigurationDrift { system, .. } if system == SchemaPack::ID)
    );
    let after = saved_rows(scratch.path());
    assert_eq!(before, after, "saved rows unchanged by refused resume");
    assert_eq!(before_digest, digest(&after));
    println!(
        "saved state unchanged: {} rows, digest {before_digest:016x}; verified 3 revisions, 2 facts",
        before.len()
    );
}
