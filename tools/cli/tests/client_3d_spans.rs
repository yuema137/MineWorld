//! The 3D client's one span function — a wall split at its doorways — against hand-computed cases
//! (S14 16f-b C1, `docs/decisions/ARC-52.md`; `pr-16f-presentation-pack-3d.md` §7). The disclosed
//! colliders, the plain shells and the stub boxes all cut their walls with it.
//!
//! The check (`clients/3d-spike/scripts/places/span_check.gd`) prints one row per case and
//! `all span checks pass`. The test starts Godot, so it is `#[ignore]`d: `cargo test` reports it as
//! ignored, never as passed.
//!
//! ```text
//! cargo test -p mineworld-cli --test client_3d_spans -- --ignored --test-threads=1
//! ```

use std::path::Path;
use std::process::Command;

/// Godot on PATH, or `GODOT`; the test refuses to pass without it.
fn godot() -> String {
    let binary = std::env::var("GODOT").unwrap_or_else(|_| "godot".to_owned());
    let found = Command::new(&binary)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success());
    assert!(
        found,
        "Godot 4.7 is needed on PATH (or GODOT=...) to run the span check"
    );
    binary
}

#[test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_3d_spans -- --ignored"]
fn a_wall_is_split_at_its_doorways_as_hand_computed() {
    let godot = godot();
    let client = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../clients/3d-spike");
    let output = Command::new(&godot)
        .args(["--headless", "--path"])
        .arg(&client)
        .args(["--script", "res://scripts/places/span_check.gd"])
        .output()
        .expect("Godot runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let failed: Vec<&str> = stdout
        .lines()
        .filter(|line| line.starts_with("FAIL"))
        .collect();
    assert!(failed.is_empty(), "span cases failed: {failed:#?}");
    assert!(
        output.status.success() && stdout.lines().any(|line| line == "all span checks pass"),
        "the span check did not pass:\n{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
