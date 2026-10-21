//! The 3D client's overhead names never cover one another (promo v2: Bob's and Alice's names overlapped
//! behind the counter). The check (`clients/3d-spike/scripts/slice/label_declutter_check.gd`) runs the
//! greedy placement on rectangles, then settles real names built by `SliceFigures.overhead_label` through
//! a real camera and measures their drawn rectangles on screen. It prints `PASS <case>` / `FAIL <case>`.
//!
//! The test starts Godot, so it is `#[ignore]`d: `cargo test` reports it as ignored, never as passed.
//!
//! ```text
//! cargo test -p mineworld-cli --test label_declutter -- --ignored
//! ```

use std::path::Path;
use std::process::Command;

#[test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test label_declutter -- --ignored"]
fn overhead_names_are_placed_apart() {
    let godot = std::env::var("GODOT").unwrap_or_else(|_| "godot".to_owned());
    let client = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../clients/3d-spike");
    let output = Command::new(&godot)
        .args(["--headless", "--path"])
        .arg(&client)
        .args(["--script", "res://scripts/slice/label_declutter_check.gd"])
        .output()
        .expect("Godot 4.7 runs (on PATH, or GODOT=...)");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let passed = stdout
        .lines()
        .filter(|line| line.starts_with("PASS "))
        .count();
    let failed: Vec<&str> = stdout
        .lines()
        .filter(|line| line.starts_with("FAIL "))
        .collect();
    assert!(
        output.status.success() && failed.is_empty(),
        "the declutter check failed: {failed:?}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(passed, 8, "every case ran:\n{stdout}");
}
