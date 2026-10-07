//! `AC-2` for the two new packs: a world without one of them still runs, and every other system is
//! unchanged (`step-09-social.md` C7, `DECISIONS.md` `ARC-28`).
//!
//! The route a World Pack has to leave a system out is its `systems:` list — there is no pack-level
//! disable flag (`worldpack/src/read.rs`) — so each configuration is a test-time copy of
//! `worlds/social-cafe` whose `world.yaml` omits one system, run by the real binary.
//!
//! ```text
//! without relationships    every fact of every other system equals the full run's, in order, by
//!                          (type, instant, subjects, participants, place, payload bytes); the
//!                          requests, activity and consults printed are equal. Relationships only
//!                          ever listened, so taking it away takes nothing else away.
//! without group-activity   relationships stays installed and enabled; the world runs, every seat
//!                          moves and talks; nobody invites or joins (the controller reads the
//!                          affordances it is offered); people still become acquainted, by talking
//! ```
//!
//! The comparison is shown able to see a difference (`ARC-23`): the same projection of seed 7 and
//! seed 8 differs, at a located row.

mod headless;
mod social;

use std::path::{Path, PathBuf};

use headless::{
    PACK, Tables, every_seat_active_in_every_bucket, fresh, lines, mineworld, run, seats, stderr,
    stdout,
};
use mineworld_contracts::{Causation, EntityId, EventEnvelope};

const DAYS: u64 = 30;

/// A copy of the pack, named as the pack (a pack's id is its directory), with `system` left out.
fn without(system: &str) -> PathBuf {
    let root = fresh(&format!("composition-without-{system}"));
    let copy = root.join("social-cafe");
    copy_dir(Path::new(PACK), &copy);
    let manifest = copy.join("world.yaml");
    let text = std::fs::read_to_string(&manifest).expect("world.yaml reads");
    let line = format!("  - {system}\n");
    assert!(text.contains(&line), "the pack enables {system}");
    std::fs::write(&manifest, text.replace(&line, "")).expect("world.yaml writes");
    copy
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a directory");
    for entry in std::fs::read_dir(from).expect("the pack lists") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("a copy");
        }
    }
}

/// `run` of a given pack directory, saved; its stdout.
fn run_pack(pack: &Path, seed: u64, save: &Path) -> String {
    let output = mineworld(&[
        "run",
        pack.to_str().expect("path"),
        "--headless",
        "--seed",
        &seed.to_string(),
        "--days",
        &DAYS.to_string(),
        "--save",
        save.to_str().expect("path"),
    ]);
    assert!(output.status.success(), "{}", stderr(&output));
    stdout(&output)
}

/// What a fact says, without the identities that depend on how many facts came before it.
type Said = (
    String,
    i64,
    Vec<EntityId>,
    Vec<EntityId>,
    Option<EntityId>,
    Vec<u8>,
);

fn said(fact: &EventEnvelope) -> Said {
    (
        fact.event_type().as_str().to_owned(),
        fact.at().seconds(),
        fact.subjects().to_vec(),
        fact.participants().to_vec(),
        fact.place().map(|place| place.entity_id()),
        fact.payload().payload().clone(),
    )
}

fn relationships_fact(fact: &EventEnvelope) -> bool {
    matches!(
        fact.event_type().as_str(),
        "became-acquainted" | "relationship-changed"
    )
}

/// The first row at which two projections differ, or the shorter length if one is a prefix.
fn first_difference(a: &[Said], b: &[Said]) -> Option<usize> {
    a.iter()
        .zip(b)
        .position(|(x, y)| x != y)
        .or_else(|| (a.len() != b.len()).then(|| a.len().min(b.len())))
}

#[test]
fn without_relationships_every_other_system_is_unchanged() {
    let full_save = fresh("composition-full");
    let full = run(7, DAYS, Some(&full_save));
    let full_facts = social::facts_of(&Tables::read(&full_save));
    social::precondition(&full_facts, 30, true);

    let pack = without("relationships");
    let save = fresh("composition-no-relationships-save");
    let printed = run_pack(&pack, 7, &save);
    assert_eq!(
        lines(&printed, "faults     0").len(),
        1,
        "no fault: {printed}"
    );
    let facts = social::facts_of(&Tables::read(&save));
    assert!(
        !facts.iter().any(relationships_fact),
        "no relationship fact in a world without relationships"
    );

    let others: Vec<Said> = full_facts
        .iter()
        .filter(|fact| !relationships_fact(fact))
        .map(said)
        .collect();
    let without_them: Vec<Said> = facts.iter().map(said).collect();
    eprintln!(
        "full run: {} facts, {} of them not relationships'; without relationships: {} facts",
        full_facts.len(),
        others.len(),
        without_them.len()
    );
    assert!(
        others.len() > 10_000,
        "a busy month was compared: {}",
        others.len()
    );
    assert_eq!(
        first_difference(&others, &without_them),
        None,
        "every other system's facts are the same, in the same order"
    );
    for prefix in ["requests ", "activity ", "consults "] {
        assert_eq!(
            lines(&full, prefix),
            lines(&printed, prefix),
            "{prefix}lines"
        );
    }

    // The comparison can see a difference: seed 8's world differs from seed 7's, at a located row.
    let eight_save = fresh("composition-full-seed-8");
    run(8, DAYS, Some(&eight_save));
    let eight: Vec<Said> = social::facts_of(&Tables::read(&eight_save))
        .iter()
        .filter(|fact| !relationships_fact(fact))
        .map(said)
        .collect();
    let differs_at = first_difference(&others, &eight).expect("seed 8 differs from seed 7");
    eprintln!(
        "seed 7 and seed 8 first differ at row {differs_at}: {} vs {}",
        others[differs_at].0, eight[differs_at].0
    );
}

#[test]
fn without_group_activity_relationships_stays_and_the_world_runs() {
    let pack = without("group-activity");
    let save = fresh("composition-no-group-activity-save");
    let printed = run_pack(&pack, 7, &save);
    assert_eq!(
        lines(&printed, "faults     0").len(),
        1,
        "no fault: {printed}"
    );
    let seats = seats();
    let seats: Vec<&str> = seats.iter().map(String::as_str).collect();
    assert_eq!(every_seat_active_in_every_bucket(&printed, &seats), 1);
    for line in lines(&printed, "requests ") {
        for action in [
            "invite",
            "accept-invitation",
            "decline-invitation",
            "join-group-activity",
        ] {
            assert!(
                !line.starts_with(&format!("requests   {action} ")),
                "nobody is asked to do something together in a world without it: {line}"
            );
        }
    }

    let inspected = mineworld(&["inspect", save.to_str().expect("path"), "--last", "0"]);
    assert!(inspected.status.success(), "{}", stderr(&inspected));
    let report = stdout(&inspected);
    let systems = lines(&report, "systems ");
    assert_eq!(
        systems,
        ["systems    presence v2, movement v1, conversation v1, relationships v1"],
        "relationships is installed and enabled, group-activity is not there"
    );

    let facts = social::facts_of(&Tables::read(&save));
    let types: std::collections::BTreeMap<u64, &str> = facts
        .iter()
        .map(|fact| (fact.id().raw(), fact.event_type().as_str()))
        .collect();
    let acquainted = facts
        .iter()
        .filter(|fact| fact.event_type().as_str() == "became-acquainted")
        .count();
    assert!(
        acquainted > 0,
        "people still come to know each other, by talking"
    );
    for fact in facts.iter().filter(|fact| relationships_fact(fact)) {
        let Causation::Event(cause) = fact.caused_by() else {
            panic!("a relationship fact not caused by a fact");
        };
        assert_eq!(
            types.get(&cause.raw()).copied(),
            Some("spoke"),
            "with no activities, speech is all relationships hears"
        );
    }
    assert!(
        !facts
            .iter()
            .any(|fact| fact.event_type().as_str().contains("group-activity")
                || fact.event_type().as_str().starts_with("invit")),
        "no group-activity fact"
    );
    eprintln!("without group-activity: {acquainted} became-acquainted, all caused by spoke");
}
