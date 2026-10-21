//! The 2D client's overhead names never cover one another (promo v2). The check
//! (`clients/2d/scripts/harness/label_check.gd`) seats five people within a metre of each other through
//! the client's own `people.gd`, settles their names through the shared `MineWorldLabelPlacement`, and
//! measures the drawn rectangles on screen. The test starts Godot, so it is `#[ignore]`d.
//!
//! ```text
//! cargo test -p mineworld-cli --test label_declutter_2d -- --ignored
//! ```

use std::path::Path;
use std::process::Command;

#[test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test label_declutter_2d -- --ignored"]
fn crowded_names_are_placed_apart_in_2d() {
    let godot = std::env::var("GODOT").unwrap_or_else(|_| "godot".to_owned());
    let client = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../clients/2d");
    let output = Command::new(&godot)
        .args(["--headless", "--path"])
        .arg(&client)
        .args(["--script", "res://scripts/harness/label_check.gd"])
        .output()
        .expect("Godot 4.7 runs (on PATH, or GODOT=...)");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && stdout.contains("label_check: PASS"),
        "the 2D label check failed:\n{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
