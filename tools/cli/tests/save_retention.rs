//! SR — save retention, through the commands a person types (`docs/DECISIONS.md` `ARC-81`;
//! `.structured-coding/plans/mvp0/pr-s6-save-retention.md`).
//!
//! ```text
//! ASR-8   a format-2 save is refused by run, server, replay and inspect, naming formats 2 and 3
//! ```
//!
//! The format-2 save itself — rows written by the build before SR — is refused in
//! `persistence/tests/save.rs`; here a social-cafe save written by this build is marked format 2, so
//! that each command reaches the refusal past its own pack and configuration checks (design §14.1, D-1).

mod headless;

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use headless::{BINARY, PACK, fresh, mineworld, run, stderr};
use mineworld_persistence::SqliteBackend;

const REFUSAL: &str = "the save is format 2; this code reads format 3 and does not migrate";

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
