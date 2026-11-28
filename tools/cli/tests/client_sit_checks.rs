//! S34-e's headless sitting checks run for real: the shared walk-then-sit chain
//! (`clients/shared/checks/sit_chain_check.gd`, I-e-3 and the SD-c3-5 row), the shared pose and
//! depiction loader with its exact legacy corpus and format-2 validation (`sit_pose_check.gd`,
//! S34-B), and the 2D seated sprite (`clients/2d/scripts/harness/seated_sprite_check.gd`, SD-e-4).
//!
//! A check passes only when Godot exits cleanly AND prints its PASS line, and no script error was
//! printed: a parse error can end a script run with exit code 0.
//!
//! ```text
//! cargo test -p mineworld-cli --test client_sit_checks -- --ignored --test-threads=1
//! ```

mod godot2d;
// `godot2d::worlds` reads saves through it.
mod headless;
mod support;

use std::process::Command;

const SHARED: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../clients/shared");
const TWO_D: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../clients/2d");

/// Imports `project`, runs `script` in it headless and asserts the run printed `pass_line`.
fn run(project: &str, script: &str, pass_line: &str) {
    let binary = godot2d::godot();
    let imported = Command::new(&binary)
        .args(["--headless", "--path", project, "--import"])
        .output()
        .expect("godot runs");
    assert!(imported.status.success(), "{project} imports");
    let done = Command::new(&binary)
        .args(["--headless", "--path", project, "--script", script])
        .output()
        .expect("godot runs");
    let out = format!(
        "{}{}",
        String::from_utf8_lossy(&done.stdout),
        String::from_utf8_lossy(&done.stderr)
    );
    for line in out.lines() {
        eprintln!("[{script}] {line}");
    }
    assert!(done.status.success(), "{script} exits cleanly");
    assert!(
        !out.contains("SCRIPT ERROR"),
        "{script} printed a script error"
    );
    assert!(!out.contains("[FAIL]"), "{script} reported a failed case");
    assert!(
        out.lines().any(|l| l.starts_with(pass_line)),
        "{script} printed no `{pass_line}` line"
    );
}

#[test]
#[ignore = "starts Godot"]
fn the_shared_sit_chain_decides_nothing() {
    run(
        SHARED,
        "res://checks/sit_chain_check.gd",
        "sit_chain_check: PASS",
    );
}

#[test]
#[ignore = "starts Godot"]
fn the_shared_pose_keeps_legacy_outputs_and_validates_vectors() {
    run(
        TWO_D,
        "../shared/checks/sit_pose_check.gd",
        "sit_pose_check: PASS",
    );
}

#[test]
#[ignore = "starts Godot"]
fn the_2d_seated_sprite_draws_every_mode() {
    run(
        TWO_D,
        "res://scripts/harness/seated_sprite_check.gd",
        "seated_sprite_check: PASS",
    );
}
