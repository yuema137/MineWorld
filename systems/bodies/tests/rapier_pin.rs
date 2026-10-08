//! PB-2 — Rapier is pinned exactly, with exactly the features `DEP-13` chose, and an upgrade must move
//! this pack's version with it (step-11 DC-1, DC-5).
//!
//! ```text
//! Cargo.lock           exactly one rapier3d, at 0.36.0, and one parry3d, at 0.31.1
//! resolved features    rapier3d's include enhanced-determinism, and exclude simd8, parallel and
//!                      serde-serialize — as the whole workspace resolves them, so no other crate can
//!                      switch one on behind this pack's back
//! the pair             (BodiesSystem::VERSION, the locked rapier3d version) = (1, "0.36.0")
//! ```
//!
//! The features are read from `cargo tree`, run through the `cargo` that built this test, offline:
//! the resolver's answer for the whole workspace, not a reading of one manifest.

use std::path::{Path, PathBuf};
use std::process::Command;

use mineworld_bodies::BodiesSystem;
use mineworld_kernel::{System, SystemVersion};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every `version` of the packages named `name` in the lock file.
fn locked(lock: &str, name: &str) -> Vec<String> {
    lock.split("[[package]]")
        .filter(|block| {
            block
                .lines()
                .any(|line| line.trim() == format!("name = \"{name}\""))
        })
        .filter_map(|block| {
            block.lines().find_map(|line| {
                line.trim()
                    .strip_prefix("version = \"")
                    .and_then(|rest| rest.strip_suffix('"'))
                    .map(str::to_owned)
            })
        })
        .collect()
}

/// rapier3d's features as the workspace resolves them for this host: `cargo tree`'s `{f}`.
///
/// Not `cargo metadata`: its resolve covers every platform, and offline it cannot fetch the
/// dependencies of platforms this machine never builds for.
fn resolved_features() -> Vec<String> {
    let output = Command::new(env!("CARGO"))
        .args([
            "tree",
            "--offline",
            "--package",
            "rapier3d",
            "--edges",
            "normal",
            "--depth",
            "0",
            "--format",
            "{p}|{f}",
        ])
        .current_dir(root())
        .output()
        .expect("`cargo tree` runs");
    assert!(
        output.status.success(),
        "`cargo tree` failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let printed = String::from_utf8(output.stdout).expect("utf-8");
    let line = printed.lines().next().expect("one line");
    let (package, features) = line.split_once('|').expect("{p}|{f}");
    assert_eq!(package, "rapier3d v0.36.0", "the package resolved");
    features.split(',').map(str::to_owned).collect()
}

#[test]
fn rapier_is_pinned_with_exactly_the_chosen_features_and_moves_with_this_packs_version() {
    let lock = std::fs::read_to_string(root().join("Cargo.lock")).expect("Cargo.lock reads");
    let rapier = locked(&lock, "rapier3d");
    let parry = locked(&lock, "parry3d");
    println!("locked: rapier3d {rapier:?}, parry3d {parry:?}");
    assert_eq!(rapier, ["0.36.0"], "exactly one rapier3d, at the pin");
    assert_eq!(parry, ["0.31.1"], "exactly one parry3d");

    let features = resolved_features();
    println!("rapier3d's resolved features: {features:?}");
    assert!(
        features
            .iter()
            .any(|feature| feature == "enhanced-determinism"),
        "enhanced-determinism is on: {features:?}"
    );
    for forbidden in ["simd8", "parallel", "serde-serialize"] {
        assert!(
            !features.iter().any(|feature| feature == forbidden),
            "{forbidden} must be off (DEP-13): {features:?}"
        );
    }

    // A Rapier upgrade is a change of results: it must arrive with a new version of this pack, so an
    // old save is refused by name rather than diverging on replay (ARC-25, step-11 DC-5). Change both
    // literals together, never one.
    assert_eq!(
        (BodiesSystem::VERSION, rapier[0].as_str()),
        (SystemVersion::new(1), "0.36.0"),
        "bodies' version and the locked Rapier move together"
    );
}
