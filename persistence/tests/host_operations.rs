//! Host operations (`ARC-374`, pr-nightlife N2-C2): an advance's lifecycle in its own table beside a save,
//! created by the first append, never re-executed, and refused by name when it cannot have happened.
//!
//! Each test drives a real `PersistentWorld` against a real SQLite file, as `host_journal.rs` does.

mod support;

use std::num::NonZeroU32;

use mineworld_contracts::{EntityKey, WorldTime};
use mineworld_persistence::format::encode;
use mineworld_persistence::{
    AdvanceOutcome, Creation, Durability, EndReason, OperationEvent, OperationId, OperationRecord,
    PersistError, PersistentWorld, SqliteBackend, WorldRevision, verify,
};
use support::{Scratch, assembled, composed, genesis_facts, t};

fn create(scratch: &Scratch) -> PersistentWorld {
    let backend = SqliteBackend::create(scratch.path(), Durability::PowerLoss).expect("creates");
    let (world, people) = assembled();
    let (persisted, _) = PersistentWorld::create(
        Box::new(backend),
        world,
        Creation {
            instance: 0x5eed_0000_0000_0000_0000_0000_0000_0374,
            pack: "test".to_owned(),
            at: t(0),
            facts: genesis_facts(&people),
        },
    )
    .expect("a save is created");
    persisted
}

fn open(scratch: &Scratch) -> SqliteBackend {
    SqliteBackend::open(scratch.path(), Durability::PowerLoss).expect("opens")
}

fn tables(scratch: &Scratch) -> Vec<String> {
    let connection =
        rusqlite::Connection::open(SqliteBackend::file(scratch.path())).expect("opens the file");
    let mut statement = connection
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
        .expect("prepares");
    statement
        .query_map([], |row| row.get::<_, String>(0))
        .expect("queries")
        .map(|name| name.expect("reads"))
        .collect()
}

fn started(operation: u128) -> OperationEvent {
    OperationEvent::AdvanceStarted {
        operation: OperationId::from_raw(operation),
        seat: EntityKey::new("ada").expect("a key"),
        adapter: "sleep".to_owned(),
        prior_scale: NonZeroU32::new(12).expect("non-zero"),
    }
}

fn ended(operation: u128, outcome: AdvanceOutcome) -> OperationEvent {
    OperationEvent::AdvanceEnded {
        operation: OperationId::from_raw(operation),
        outcome,
        reason: None,
    }
}

fn refused_naming(scratch: &Scratch, record: usize, why: &str) {
    let refusal = verify(&open(scratch), composed()).expect_err("the history is incoherent");
    let PersistError::Damaged { detail } = refusal else {
        panic!("an incoherent history is refused as damaged, not as {refusal}");
    };
    assert!(
        detail.contains(&format!("host operation record {record}")) && detail.contains(why),
        "the refusal names record {record} and '{why}': {detail}"
    );
}

/// The table is created by the first append and by nothing else — not by hosting, not by the pacing
/// journal — so a save that never advanced keeps its tables and a pre-N2 binary reads past it.
#[test]
fn the_table_is_created_by_the_first_append_and_reads_back_in_order() {
    let scratch = Scratch::new("host-operations-order");
    let mut persisted = create(&scratch);
    assert!(persisted.operations().expect("reads").is_empty());
    assert!(!tables(&scratch).contains(&"host_operations".to_owned()));
    persisted
        .record_operation(started(1), t(100))
        .expect("appends");
    persisted
        .record_operation(ended(1, AdvanceOutcome::Woke), t(900))
        .expect("appends");
    assert!(tables(&scratch).contains(&"host_operations".to_owned()));
    let records = persisted.operations().expect("coherent");
    assert_eq!(
        records,
        [
            OperationRecord {
                at: t(100),
                revision: WorldRevision::GENESIS,
                event: started(1),
            },
            OperationRecord {
                at: t(900),
                revision: WorldRevision::GENESIS,
                event: ended(1, AdvanceOutcome::Woke),
            },
        ]
    );
    let verified = verify(&open(&scratch), composed()).expect("a coherent history verifies");
    assert_eq!(
        verified.revisions, 1,
        "genesis alone: no record is a revision"
    );
}

/// The bytes are pinned: the save's encoding of each record is the wire contract a later binary reads.
#[test]
fn the_bytes_of_each_record_are_pinned() {
    let record = OperationRecord {
        at: WorldTime::from_seconds(82_800),
        revision: WorldRevision::from_raw(41),
        event: started(0xabc),
    };
    assert_eq!(
        String::from_utf8(encode(&record).expect("encodes")).expect("utf-8"),
        r#"{"at":82800,"revision":41,"event":{"advance_started":{"operation":"00000000000000000000000000000abc","seat":"ada","adapter":"sleep","prior_scale":12}}}"#
    );
    let record = OperationRecord {
        at: WorldTime::from_seconds(111_600),
        revision: WorldRevision::from_raw(97),
        event: OperationEvent::AdvanceEnded {
            operation: OperationId::from_raw(0xabc),
            outcome: AdvanceOutcome::Stopped,
            reason: Some(EndReason::NotYetExecutable),
        },
    };
    assert_eq!(
        String::from_utf8(encode(&record).expect("encodes")).expect("utf-8"),
        r#"{"at":111600,"revision":97,"event":{"advance_ended":{"operation":"00000000000000000000000000000abc","outcome":"stopped","reason":"not_yet_executable"}}}"#
    );
}

/// An end with no start, a second start while one is open, a start of a known operation and a decreasing
/// instant cannot have happened: each is refused by name. An `interrupted` end may state an earlier instant
/// — the durable one the world resumed at.
#[test]
fn an_incoherent_lifecycle_is_refused_by_name() {
    let scratch = Scratch::new("host-operations-orphan");
    let mut persisted = create(&scratch);
    persisted
        .record_operation(ended(1, AdvanceOutcome::Woke), t(10))
        .expect("appends");
    refused_naming(&scratch, 1, "not the open one");

    let scratch = Scratch::new("host-operations-two-open");
    let mut persisted = create(&scratch);
    persisted
        .record_operation(started(1), t(10))
        .expect("appends");
    persisted
        .record_operation(started(2), t(10))
        .expect("appends");
    refused_naming(&scratch, 2, "while another is open");

    let scratch = Scratch::new("host-operations-restart");
    let mut persisted = create(&scratch);
    persisted
        .record_operation(started(1), t(10))
        .expect("appends");
    persisted
        .record_operation(ended(1, AdvanceOutcome::Cancelled), t(20))
        .expect("appends");
    persisted
        .record_operation(started(1), t(30))
        .expect("appends");
    refused_naming(&scratch, 3, "already started");

    let scratch = Scratch::new("host-operations-backwards");
    let mut persisted = create(&scratch);
    persisted
        .record_operation(started(1), t(500))
        .expect("appends");
    persisted
        .record_operation(ended(1, AdvanceOutcome::Woke), t(400))
        .expect("appends");
    refused_naming(&scratch, 2, "earlier");

    let scratch = Scratch::new("host-operations-interrupted");
    let mut persisted = create(&scratch);
    persisted
        .record_operation(started(1), t(500))
        .expect("appends");
    persisted
        .record_operation(ended(1, AdvanceOutcome::Interrupted), t(400))
        .expect("appends");
    verify(&open(&scratch), composed()).expect("an interrupted end states the durable instant");
}
