//! `mineworld inspect` and the `AC-9` causation check over a real run's save (step-08 C6).
//!
//! The check is shown to *see* what it checks (`ARC-23`): a save with one forged fact — a cause no
//! journaled request carried — must fail it, naming that fact. A save a server still commits to is
//! inspected as the prefix one journal pass read: a revision committed meanwhile is not half-seen.

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
    // The retained snapshots (ARC-81, D-SR-9): exactly the save's, each with its stored size.
    let kept: Vec<String> = before
        .snapshots
        .iter()
        .map(|(revision, bytes)| format!("r{revision} {} B", bytes.len()))
        .collect();
    assert_eq!(
        lines(&report, "snapshots  "),
        [format!(
            "snapshots  {} kept (zstd): {}",
            kept.len(),
            kept.join(", ")
        )],
        "{report}"
    );
    assert!(
        kept.len() >= 3,
        "genesis, an anchor and the newest two: {kept:?}"
    );
    let facts = count_after(lines(&report, "facts      ")[0], "facts ");
    assert_eq!(facts, u64::try_from(before.facts.len()).expect("fits"));
    assert!(facts > 10_000, "a month of a busy world: {facts}");
    assert!(report.contains(
        "systems    presence v3, movement v2, conversation v4, group-activity v3, relationships v3, \
         naming v1, schedule v1, bodies v5, persona v1, appearance v1\n"
    ));
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
            retire: Vec::new(),
            seal: None,
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

/// One more accepted request and the fact it caused, made from the save's newest request: its entry
/// and its first fact, renumbered to action `action`, revision `revision` and fact `fact_id`.
fn another_request(
    template: &mineworld_persistence::backend::StoredRevision,
    revision: u64,
    action: u64,
    fact_id: u64,
) -> RevisionRow {
    let mut entry: serde_json::Value =
        serde_json::from_slice(&template.entry).expect("an entry is JSON");
    let mut renumbered = false;
    renumber_action(&mut entry, action, &mut renumbered);
    assert!(renumbered, "the template carries its action id: {entry}");
    let mut fact: serde_json::Value =
        serde_json::from_slice(&template.facts[0].bytes).expect("a fact is JSON");
    fact["id"] = serde_json::Value::String(fact_id.to_string());
    fact["caused_by"] = serde_json::json!({ "action": action.to_string() });
    RevisionRow {
        revision: WorldRevision::from_raw(revision),
        at: template.at,
        action_id: Some(action),
        entry: serde_json::to_vec(&entry).expect("encodes"),
        facts: vec![FactRow {
            id: fact_id,
            bytes: serde_json::to_vec(&fact).expect("encodes"),
        }],
        snapshot: None,
        retire: Vec::new(),
        seal: None,
    }
}

fn renumber_action(value: &mut serde_json::Value, action: u64, done: &mut bool) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, inner) in map.iter_mut() {
                if key == "action_id" {
                    *inner = serde_json::Value::String(action.to_string());
                    *done = true;
                } else {
                    renumber_action(inner, action, done);
                }
            }
        }
        serde_json::Value::Array(items) => {
            for inner in items {
                renumber_action(inner, action, done);
            }
        }
        _ => {}
    }
}

#[test]
fn a_save_being_committed_to_is_inspected_as_one_consistent_prefix() {
    let save = fresh("inspect-live");
    run(7, 2, Some(&save));
    let backend = SqliteBackend::open(&save, Durability::ProcessCrash).expect("opens");
    let template = backend
        .revisions_after(WorldRevision::from_raw(0))
        .expect("a journal")
        .map(|stored| stored.expect("a revision"))
        .filter(|stored| stored.action_id.is_some() && !stored.facts.is_empty())
        .last()
        .expect("a run's save holds an accepted request with a fact");
    drop(backend);

    // A writer commits one request and its fact after another, as a server does, while inspections
    // run: each inspection can meet a revision committed between its two passes. Bounded, so the save
    // stays small: `COMMITS` revisions, one about every millisecond.
    const COMMITS: u64 = 3_000;
    let writer = {
        let save = save.to_path_buf();
        std::thread::spawn(move || {
            let mut backend = SqliteBackend::open(&save, Durability::ProcessCrash).expect("opens");
            let mut head = backend.head().expect("a head").raw();
            let mut action = backend
                .highest_action_id()
                .expect("reads")
                .expect("a request");
            let mut fact = backend.last_facts(1).expect("facts")[0].id;
            for _ in 0..COMMITS {
                (head, action, fact) = (head + 1, action + 1, fact + 1);
                backend
                    .commit(&another_request(&template, head, action, fact))
                    .expect("commits");
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        })
    };
    let (mut overlapped, mut failures) = (0, Vec::new());
    while !writer.is_finished() && overlapped < 200 {
        let output = mineworld(&["inspect", save.to_str().expect("path")]);
        overlapped += 1;
        if !output.status.success() {
            failures.push(stderr(&output));
        }
    }
    writer.join().expect("the writer ran");
    assert!(
        overlapped >= 5,
        "inspections ran while the save was committed to: {overlapped}"
    );
    assert!(
        failures.is_empty(),
        "{} of {overlapped} inspections of a live save failed, first: {}",
        failures.len(),
        failures[0]
    );
    let settled = mineworld(&["inspect", save.to_str().expect("path")]);
    assert!(
        settled.status.success(),
        "every committed request resolves: {}",
        stderr(&settled)
    );
}
