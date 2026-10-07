//! `mineworld inspect` and the `AC-9` causation check over a real run's save (step-08 C6).
//!
//! The check is shown to *see* what it checks (`ARC-23`): a save with one forged fact — a cause no
//! journaled request carried — must fail it, naming that fact.

mod headless;

use headless::{Tables, count_after, fresh, lines, mineworld, run, stderr, stdout};
use mineworld_contracts::WorldTime;
use mineworld_persistence::backend::{FactRow, RevisionRow};
use mineworld_persistence::{
    Durability, JournalEntry, Outcome, PersistenceBackend, SqliteBackend, WorldInput,
    WorldRevision, format,
};

#[test]
fn inspect_reports_a_run_s_save_and_every_cause_in_it_resolves() {
    let save = fresh("inspect-run");
    run(7, 30, Some(&save));
    let before = Tables::read(&save);

    let output = mineworld(&["inspect", save.to_str().expect("path"), "--last", "200"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let report = stdout(&output);

    // Located: the report read this save — its head and its whole fact log.
    let head = before.journal.len();
    assert!(
        report.contains(&format!("head       revision {head} ")),
        "the head on disk: {report}"
    );
    let facts = count_after(lines(&report, "facts      ")[0], "facts ");
    assert_eq!(facts, u64::try_from(before.facts.len()).expect("fits"));
    assert!(facts > 10_000, "a month of a busy world: {facts}");
    assert!(report.contains("systems    presence v2, movement v1, conversation v1"));
    for kind in [
        "causes     action ",
        "causes     event ",
        "causes     world genesis ",
    ] {
        assert_eq!(lines(&report, kind).len(), 1, "{kind}: {report}");
    }
    assert_eq!(
        lines(&report, "  #").len(),
        200,
        "the window of the newest 200 facts"
    );
    assert!(
        report.contains(&format!(
            "AC-9       every cause resolves: {facts} fact(s) checked"
        )),
        "{report}"
    );

    // Read-only: the save's tables are what they were.
    Tables::read(&save).assert_same_history(&before, "after inspect");
}

#[test]
fn a_fact_whose_cause_no_request_carried_fails_the_check_by_name() {
    let save = fresh("inspect-forged");
    run(7, 2, Some(&save));
    let clean = mineworld(&["inspect", save.to_str().expect("path")]);
    assert!(clean.status.success(), "the unforged save passes first");

    // Forge one revision through the backend's own commit: a copy of the newest fact, renumbered,
    // claiming to be caused by a request that was never made.
    let mut backend = SqliteBackend::open(&save, Durability::ProcessCrash).expect("opens");
    let head = backend.head().expect("a head");
    let newest = backend.last_facts(1).expect("facts").remove(0);
    let mut fact: serde_json::Value =
        serde_json::from_slice(&newest.bytes).expect("a fact is JSON");
    let forged_id = newest.id + 1;
    fact["id"] = serde_json::Value::String(forged_id.to_string());
    fact["caused_by"] = serde_json::json!({ "action": "999999" });
    let at = fact["at"].as_i64().expect("an instant") + 1;
    let entry = JournalEntry {
        input: WorldInput::Advance {
            until: WorldTime::from_seconds(at),
        },
        outcome: Outcome::Advanced {
            instants: 1,
            skipped: 0,
        },
    };
    backend
        .commit(&RevisionRow {
            revision: WorldRevision::from_raw(head.raw() + 1),
            at,
            action_id: None,
            entry: format::encode(&entry).expect("encodes"),
            facts: vec![FactRow {
                id: forged_id,
                bytes: serde_json::to_vec(&fact).expect("encodes"),
            }],
            snapshot: None,
        })
        .expect("the backend stores rows it is given");
    drop(backend);

    let output = mineworld(&["inspect", save.to_str().expect("path")]);
    assert!(
        !output.status.success(),
        "a forged cause must fail the check"
    );
    let complaint = stderr(&output);
    assert!(
        complaint.contains(&format!("fact #{forged_id} caused by action 999999")),
        "names the fact: {complaint}"
    );
    assert!(!complaint.contains("panicked"), "{complaint}");
}

#[test]
fn a_directory_with_no_save_is_refused_by_name() {
    let empty = fresh("inspect-nothing");
    std::fs::create_dir_all(&empty).expect("a directory");
    let output = mineworld(&["inspect", empty.to_str().expect("path")]);
    assert!(!output.status.success());
    let complaint = stderr(&output);
    assert!(complaint.contains("inspect-nothing") && complaint.contains("holds no save"));
    assert!(!complaint.contains("panicked"));
}
