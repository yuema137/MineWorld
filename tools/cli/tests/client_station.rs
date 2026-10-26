//! The shared station fixture rules (`clients/shared/station`, `ARC-108`; S29-e §5.2, AC-e-1) run for
//! real: the window, kiosk and board rule (written by S29-f, adopted here) and the clock rule.
//!
//! Every test here starts Godot, so every test is `#[ignore]`d: `cargo test` reports them as ignored,
//! never as passed, on a machine without Godot. Run them with
//!
//! ```text
//! cargo test -p mineworld-cli --test client_station -- --ignored --test-threads=1
//! ```

mod godot2d;
// `godot2d::worlds` reads saves through it.
mod headless;
mod support;

use std::path::Path;
use std::process::Command;

const SHARED: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../clients/shared");

/// Runs a script of `clients/shared/checks/` headless in the shared module's project and returns
/// whether it exited cleanly and what it printed.
fn check(script: &str) -> (bool, Vec<String>) {
    let binary = godot2d::godot();
    let imported = Command::new(&binary)
        .args(["--headless", "--path", SHARED, "--import"])
        .output()
        .expect("godot runs");
    assert!(imported.status.success(), "clients/shared imports");
    let done = Command::new(&binary)
        .args(["--headless", "--path", SHARED, "--script"])
        .arg(Path::new("res://checks").join(script))
        .output()
        .expect("godot runs");
    let lines: Vec<String> = String::from_utf8_lossy(&done.stdout)
        .lines()
        .map(str::to_owned)
        .collect();
    for line in &lines {
        eprintln!("[{script}] {line}");
    }
    (done.status.success(), lines)
}

#[test]
#[ignore = "starts Godot"]
fn station_state_table() {
    let (ok, lines) = check("station_state_check.gd");
    assert!(ok, "the station state check exits cleanly");
    assert!(lines.iter().any(|l| l == "all station state checks pass"));
    assert!(!lines.iter().any(|l| l.contains("[FAIL]")));
    // X-7: a planted departures record shows its lines; nothing disclosed shows none.
    assert!(
        lines
            .iter()
            .any(|l| l == "STATION of_observation lines2 open shut")
    );
    assert!(
        lines
            .iter()
            .any(|l| l == "STATION unentered none shut shut")
    );
}

#[test]
#[ignore = "starts Godot"]
fn station_clock_table() {
    let (ok, lines) = check("station_clock_check.gd");
    assert!(ok, "the station clock check exits cleanly");
    assert!(lines.iter().any(|l| l == "all station clock checks pass"));
    assert!(!lines.iter().any(|l| l.contains("[FAIL]")));
    // M-e-1: no calendar disclosed means no hands, never the host's time.
    assert!(
        lines
            .iter()
            .any(|l| l == "CLOCK no calendar -> no hands -1")
    );
}
