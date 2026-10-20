//! I-S31-11 (S31-a I-a-9, `ARC-111`): no reference world's currency name is written into code. Every
//! name is read from the reference worlds' own `configure/economy.yaml`, so a world added later is
//! covered without editing this test; the scan covers every tracked file under `systems/`, `clients/`,
//! `cognition/` and `server/`, outside tests, fixtures and recorded evidence. A symbol such as `$` is
//! not scannable (it is ordinary text in code and prose) and is left out, as the design records.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The repository root.
fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The tracked files under `roots`, by path relative to the repository.
fn tracked(roots: &[&str]) -> Vec<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository())
        .args(["-c", "core.quotepath=off", "ls-files", "-z", "--"])
        .args(roots)
        .output()
        .expect("git runs");
    assert!(output.status.success(), "git ls-files");
    String::from_utf8(output.stdout)
        .expect("utf-8 paths")
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Every currency name a reference world declares (`worlds/<world>/configure/economy.yaml`).
fn reference_names() -> Vec<String> {
    let mut names = Vec::new();
    for path in tracked(&["worlds"]) {
        if !path.ends_with("configure/economy.yaml") {
            continue;
        }
        let text = std::fs::read_to_string(repository().join(&path)).expect("readable");
        let value: serde_json::Value = serde_saphyr::from_str(&text).expect("YAML");
        let name = value["currency"]["name"]
            .as_str()
            .unwrap_or_else(|| panic!("{path}: currency.name"));
        names.push(name.to_owned());
    }
    names
}

/// Whether `path` is a test, a fixture or recorded evidence, where a world's name may appear.
fn exempt(path: &str) -> bool {
    path.split('/')
        .any(|part| matches!(part, "tests" | "fixtures" | "evidence"))
}

#[test]
fn no_reference_world_currency_name_is_written_into_code() {
    let names = reference_names();
    assert!(
        !names.is_empty(),
        "at least Market Town declares its currency"
    );
    let mut found = Vec::new();
    for path in tracked(&["systems", "clients", "cognition", "server"]) {
        if exempt(&path) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(repository().join(&path)) else {
            continue; // not text
        };
        let lower = text.to_lowercase();
        for name in &names {
            if lower.contains(&name.to_lowercase()) {
                found.push(format!("{path}: names the currency '{name}'"));
            }
        }
    }
    assert!(
        found.is_empty(),
        "a world's currency is data (ARC-111):\n{}",
        found.join("\n")
    );
}

/// The scan sees a name where one is written, and nothing in an exempt path.
#[test]
fn the_scan_sees_a_name_and_exempts_tests_and_fixtures() {
    assert!(exempt(
        "clients/agent/tests/fixtures/market-town-start.json"
    ));
    assert!(exempt("systems/economy/tests/currency.rs"));
    assert!(!exempt("systems/economy/src/configuration.rs"));
    assert!(
        reference_names().iter().any(|name| name == "US dollar"),
        "Market Town's currency is read from its file"
    );
}
