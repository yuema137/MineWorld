//! The 3D slice's needs rows, for the recorded observations, equal the hand-written oracle line for line
//! (S35 N-e2 C4, AC-S35-10 / AC-Ne-7; `pr-s35-ne-presentation.md` §6.2).
//!
//! The probe (`clients/3d-spike/scripts/slice/needs_rows_probe.gd`) prints `ROWS <fixture> <locale> <json>`
//! from the same builder the HUD uses. The oracle `tests/fixtures/observations/needs/rows3d.<locale>.txt` was
//! written by hand from the fixtures. It holds the rows N-e owns (needs, asleep, the next part of the day); the
//! 2D panels also show the 13b agenda row, which the 3D HUD does not word (see the ledger's parity finding).
//!
//! The test starts Godot, so it is `#[ignore]`d: `cargo test` reports it as ignored, never as passed.
//!
//! ```text
//! cargo test -p mineworld-cli --test needs_rows_parity -- --ignored --test-threads=1
//! ```

use std::fs;
use std::path::Path;
use std::process::Command;

const FIXTURES: [&str; 3] = ["market-town-night", "commute-late", "commute-sleeper"];
const LOCALES: [&str; 2] = ["en", "zh_Hans"];

/// Godot on PATH, or `GODOT`; the test refuses to pass without it.
fn godot() -> String {
    let binary = std::env::var("GODOT").unwrap_or_else(|_| "godot".to_owned());
    let found = Command::new(&binary)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success());
    assert!(found, "Godot 4.7 is needed on PATH (or GODOT=...) to run the 3D probe");
    binary
}

#[test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test needs_rows_parity -- --ignored"]
fn the_3d_needs_rows_equal_the_oracle_for_each_fixture_and_locale() {
    let godot = godot();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let client = root.join("clients/3d-spike");
    let fixtures = root.join("tests/fixtures/observations/needs");
    let output = Command::new(&godot)
        .args(["--headless", "--path"])
        .arg(&client)
        .args(["--script", "res://scripts/slice/needs_rows_probe.gd", "--"])
        .arg(format!("--fixtures={}", fixtures.display()))
        .output()
        .expect("Godot runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "the rows probe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    for locale in LOCALES {
        let oracle = fs::read_to_string(fixtures.join(format!("rows3d.{locale}.txt")))
            .expect("the oracle is committed");
        let expected: Vec<&str> = oracle.lines().collect();
        let marker = format!(" {locale} ");
        let got: Vec<&str> = stdout
            .lines()
            .filter(|line| line.starts_with("ROWS ") && line.contains(&marker))
            .collect();
        assert_eq!(got.len(), FIXTURES.len(), "{locale}: one ROWS line per fixture");
        assert_eq!(got, expected, "{locale}: the 3D rows against the oracle");
    }
}
