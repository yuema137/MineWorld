//! The host journal (`ARC-69`, TW-c C2): a table of pacing records beside a save, created by the first
//! append, never re-executed, and refused by name when it cannot have happened.
//!
//! Each test drives a real `PersistentWorld` against a real SQLite file, as `save.rs` does.

mod support;

use std::num::NonZeroU32;

use mineworld_contracts::WorldTime;
use mineworld_persistence::backend::HostRow;
use mineworld_persistence::format::encode;
use mineworld_persistence::{
    Creation, Durability, HostRecord, PacingChange, PersistError, PersistenceBackend,
    PersistentWorld, SqliteBackend, WorldRevision, verify,
};
use support::{Scratch, assembled, composed, genesis_facts, t};

const INSTANCE: u128 = 0x5eed_0000_0000_0000_0000_0000_0000_0011;

fn non_zero(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).expect("non-zero")
}

fn create(scratch: &Scratch) -> PersistentWorld {
    let backend = SqliteBackend::create(scratch.path(), Durability::PowerLoss).expect("creates");
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
    persisted
}

fn raw(scratch: &Scratch) -> rusqlite::Connection {
    rusqlite::Connection::open(SqliteBackend::file(scratch.path())).expect("opens the file")
}

fn tables(scratch: &Scratch) -> Vec<String> {
    let connection = raw(scratch);
    let mut statement = connection
        .prepare("SELECT name FROM sqlite_master WHERE type IN ('table', 'index') ORDER BY name")
        .expect("prepares");
    statement
        .query_map([], |row| row.get::<_, String>(0))
        .expect("queries")
        .map(|name| name.expect("reads"))
        .collect()
}

fn open(scratch: &Scratch) -> SqliteBackend {
    SqliteBackend::open(scratch.path(), Durability::PowerLoss).expect("opens")
}

/// A save that is never hosted has exactly the tables it was created with (INV-TW-1): the journal's
/// table is created by the first append, not by the schema.
#[test]
fn a_save_that_is_never_hosted_has_no_host_journal() {
    let scratch = Scratch::new("host-journal-absent");
    let _ = create(&scratch);
    assert_eq!(
        tables(&scratch),
        [
            "facts",
            "facts_by_revision",
            "journal",
            "manifest",
            "sealed",
            "snapshots"
        ],
        "a save as created names exactly its five tables (`sealed`: ARC-89) and its index"
    );
}

/// The first append creates the table; a save without it reads an empty journal; the records come back
/// in the order they were written, at the revision the save was at.
#[test]
fn the_first_append_creates_the_table_and_the_journal_reads_back_in_order() {
    let scratch = Scratch::new("host-journal-order");
    let mut persisted = create(&scratch);
    assert!(
        persisted.host_records().expect("reads").is_empty(),
        "no journal, so an empty one"
    );
    persisted
        .record_host(
            PacingChange::Started {
                scale: non_zero(12),
                paused: false,
            },
            t(1_000),
        )
        .expect("appends");
    persisted
        .record_host(PacingChange::Paused, t(1_030))
        .expect("appends");
    persisted
        .record_host(PacingChange::Scaled { to: non_zero(24) }, t(1_030))
        .expect("appends");
    persisted
        .record_host(PacingChange::Resumed, t(1_060))
        .expect("appends");
    persisted
        .record_host(PacingChange::Stopped, t(1_090))
        .expect("appends");
    assert!(
        tables(&scratch).contains(&"host_journal".to_owned()),
        "the first append created the table"
    );
    let changes: Vec<PacingChange> = persisted
        .host_records()
        .expect("reads")
        .iter()
        .map(|record| record.change)
        .collect();
    assert_eq!(
        changes,
        [
            PacingChange::Started {
                scale: non_zero(12),
                paused: false
            },
            PacingChange::Paused,
            PacingChange::Scaled { to: non_zero(24) },
            PacingChange::Resumed,
            PacingChange::Stopped,
        ]
    );
    let revisions: Vec<WorldRevision> = persisted
        .host_records()
        .expect("reads")
        .iter()
        .map(|record| record.revision)
        .collect();
    assert!(
        revisions
            .iter()
            .all(|revision| *revision == WorldRevision::GENESIS),
        "no input was journaled, so every record is at the save's head, genesis"
    );
    let rows = open(&scratch).host_records().expect("reads");
    assert_eq!(
        rows.len(),
        5,
        "the reopened save reads the same five records"
    );
}

/// A record's bytes are pinned: a change to a variant's encoding is a change to every save that holds one.
#[test]
fn the_bytes_of_each_pacing_change_are_pinned() {
    let at = WorldTime::from_seconds(4_112);
    let revision = WorldRevision::from_raw(37);
    let encoded = |change| {
        String::from_utf8(
            encode(&HostRecord {
                at,
                revision,
                change,
            })
            .expect("encodes"),
        )
        .expect("json is utf-8")
    };
    assert_eq!(
        encoded(PacingChange::Scaled { to: non_zero(24) }),
        r#"{"at":4112,"revision":37,"change":{"Scaled":{"to":24}}}"#
    );
    assert_eq!(
        encoded(PacingChange::Started {
            scale: non_zero(12),
            paused: false
        }),
        r#"{"at":4112,"revision":37,"change":{"Started":{"scale":12,"paused":false}}}"#
    );
    assert_eq!(
        encoded(PacingChange::Paused),
        r#"{"at":4112,"revision":37,"change":"Paused"}"#
    );
    assert_eq!(
        encoded(PacingChange::Resumed),
        r#"{"at":4112,"revision":37,"change":"Resumed"}"#
    );
    assert_eq!(
        encoded(PacingChange::Stopped),
        r#"{"at":4112,"revision":37,"change":"Stopped"}"#
    );
}

/// A journal whose instant goes backwards cannot have happened (INV-TW-8 over the save's lifetime): verify
/// refuses it and names the record.
#[test]
fn verify_refuses_a_journal_whose_instant_decreases_naming_the_record() {
    let scratch = Scratch::new("host-journal-decreasing");
    let mut persisted = create(&scratch);
    persisted
        .record_host(PacingChange::Paused, t(100))
        .expect("appends");
    persisted
        .record_host(PacingChange::Resumed, t(50))
        .expect("appends");
    let refusal = verify(&open(&scratch), composed()).expect_err("the journal is incoherent");
    let PersistError::Damaged { detail } = refusal else {
        panic!("a damaged journal is refused as damaged, not as {refusal}");
    };
    assert!(
        detail.contains("record 2") && detail.contains("earlier"),
        "the refusal names record 2: {detail}"
    );
}

/// A journal row whose stored bytes do not decode is refused by the row, not guessed at (`ARC-69`).
#[test]
fn a_damaged_host_record_is_refused_by_name() {
    let scratch = Scratch::new("host-journal-damaged");
    let mut persisted = create(&scratch);
    persisted
        .record_host(PacingChange::Paused, t(10))
        .expect("appends");
    let mut backend = open(&scratch);
    backend
        .append_host_record(&HostRow {
            revision: WorldRevision::GENESIS,
            at: 20,
            record: b"not a record".to_vec(),
        })
        .expect("the storage accepts any bytes");
    let refusal = persisted
        .host_records()
        .expect_err("the second record is damaged");
    let PersistError::Damaged { detail } = refusal else {
        panic!("a damaged record is refused as damaged, not as {refusal}");
    };
    assert!(
        detail.contains("record 2"),
        "the refusal names it: {detail}"
    );
}

/// A coherent journal does not change what verify checks: the save verifies, and its revisions are the
/// same as with no journal at all.
#[test]
fn a_coherent_journal_leaves_verification_as_it_was() {
    let scratch = Scratch::new("host-journal-coherent");
    let mut persisted = create(&scratch);
    persisted
        .record_host(PacingChange::Paused, t(10))
        .expect("appends");
    persisted
        .record_host(PacingChange::Resumed, t(10))
        .expect("appends");
    let verified = verify(&open(&scratch), composed()).expect("the save reproduces");
    assert_eq!(verified.revisions, 1, "genesis alone, as with no journal");
}
