//! SR — save retention, through the commands a person types (`docs/DECISIONS.md` `ARC-81`;
//! `.structured-coding/plans/mvp0/pr-s6-save-retention.md`).
//!
//! ```text
//! ASR-8   a format-2 save is refused by run, server, replay and inspect, naming formats 2 and 3
//! ASR-4   verify_from on a real save's anchors (ignored; run by hand against a long run's save)
//! ```
//!
//! The format-2 save itself — rows written by the build before SR — is refused in
//! `persistence/tests/save.rs`; here a social-cafe save written by this build is marked format 2, so
//! that each command reaches the refusal past its own pack and configuration checks (design §14.1, D-1).

mod headless;

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use headless::{BINARY, PACK, fresh, mineworld, run, stderr};
use mineworld_persistence::{
    Durability, PersistenceBackend, SqliteBackend, WorldRevision, verify_from,
};

const REFUSAL: &str = "the save is format 2; this code reads format 3 and does not migrate";

/// ASR-4 on a real save, run by hand: `verify_from` from the first, middle and last retained anchor
/// (or from every one) of the save in `MINEWORLD_SR_SAVE`, composed from the pack in
/// `MINEWORLD_SR_PACK`. Not in the default suite: the saves it is meant for are the 30- and 300-day
/// release runs the design measures (§14), which the suite does not keep.
///
/// ```text
/// MINEWORLD_SR_PACK=worlds/market-town MINEWORLD_SR_SAVE=DIR [MINEWORLD_SR_ANCHORS=all] \
///   cargo test --release -p mineworld-cli --test save_retention -- --ignored --nocapture
/// ```
#[test]
#[ignore = "run by hand against a long run's save (ASR-4)"]
fn verify_from_retained_anchors_of_a_real_save() {
    let pack = std::env::var("MINEWORLD_SR_PACK").expect("MINEWORLD_SR_PACK names the pack");
    let save = std::env::var("MINEWORLD_SR_SAVE").expect("MINEWORLD_SR_SAVE names the save");
    let every = std::env::var("MINEWORLD_SR_ANCHORS").is_ok_and(|value| value == "all");
    let pack = mineworld_worldpack::WorldPack::read(&pack).expect("the pack reads");
    let backend = SqliteBackend::open(std::path::Path::new(&save), Durability::ProcessCrash)
        .expect("the save opens");
    let anchors: Vec<WorldRevision> = backend
        .snapshot_revisions()
        .expect("reads")
        .into_iter()
        .filter(|revision| revision.raw() > 1 && revision.raw().is_multiple_of(4_096))
        .collect();
    assert!(!anchors.is_empty(), "the save holds an anchor");
    let chosen: Vec<WorldRevision> = if every {
        anchors.clone()
    } else {
        let mut chosen = vec![
            anchors[0],
            anchors[anchors.len() / 2],
            anchors[anchors.len() - 1],
        ];
        chosen.dedup();
        chosen
    };
    println!(
        "anchors held: {}",
        anchors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    );
    for anchor in chosen {
        let started = Instant::now();
        let world = pack.compose().expect("the pack composes").world;
        let verified = verify_from(&backend, world, anchor)
            .unwrap_or_else(|error| panic!("from {anchor}: {error}"));
        assert_eq!(verified.revisions, verified.head.raw() - anchor.raw());
        println!(
            "verify_from {anchor}: {} revisions, {} facts and {} later snapshots reproduced to head \
             {} in {:.1} s",
            verified.revisions,
            verified.facts,
            verified.snapshots,
            verified.head,
            started.elapsed().as_secs_f64()
        );
    }
}

#[test]
fn every_command_that_opens_a_save_refuses_format_2_by_name() {
    let save = fresh("sr-format-2");
    run(7, 1, Some(&save));
    let connection = rusqlite::Connection::open(SqliteBackend::file(&save)).expect("opens");
    let changed = connection
        .execute("UPDATE manifest SET format = 2", [])
        .expect("executes");
    assert_eq!(changed, 1, "the manifest row was marked");
    drop(connection);
    let path = save.to_str().expect("a printable path");

    for arguments in [
        vec![
            "run",
            PACK,
            "--headless",
            "--seed",
            "7",
            "--days",
            "2",
            "--save",
            path,
        ],
        vec!["replay", PACK, "--save", path],
        vec!["inspect", path],
    ] {
        let output = mineworld(&arguments);
        let complaint = stderr(&output);
        assert!(!output.status.success(), "{arguments:?} must refuse");
        assert!(
            complaint.contains(REFUSAL),
            "{arguments:?} names both formats: {complaint}"
        );
        assert!(!complaint.contains("panicked"), "{complaint}");
    }

    // `server` builds its world before it binds, so a refused save ends the process; it is bounded
    // here all the same, so that a server which wrongly accepted the save fails the test, not hangs it.
    let mut server = Command::new(BINARY)
        .args([
            "server",
            PACK,
            "--listen",
            "127.0.0.1:0",
            "--invite",
            "sr-format-test",
            "--save",
            path,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the server starts");
    let started = Instant::now();
    let status = loop {
        if let Some(status) = server.try_wait().expect("polls") {
            break status;
        }
        if started.elapsed() > Duration::from_secs(60) {
            let _ = server.kill();
            let _ = server.wait();
            panic!("the server accepted a format-2 save and kept running");
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    let output = server.wait_with_output().expect("collects");
    let complaint = String::from_utf8_lossy(&output.stderr);
    assert!(!status.success(), "the server refuses");
    assert!(
        complaint.contains(REFUSAL),
        "the server names both formats: {complaint}"
    );
    assert!(!complaint.contains("panicked"), "{complaint}");
}
