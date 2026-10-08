//! PB-15 — Rapier stays behind one module, no float reaches anything persisted, and this pack depends
//! on no System Pack but presence (`DECISIONS.md` `DEP-13`; step-11 I-3, I-5, SD-B1).
//!
//! ```text
//! the crate        in systems/bodies/src, only rapier.rs names `rapier3d`; outside systems/bodies,
//!                  no code file (*.rs, Cargo.toml) names it at all — the pin lives in this pack's own
//!                  manifest (step-11 §17.0, QP-3 overruled)
//! floats           no source file but rapier.rs names f32 or f64: positions, shapes and facts are
//!                  integers, and only the sweep is float
//! dependencies     the [dependencies] of this pack name no System Pack but presence
//! ```
//!
//! The module that wraps the crate is itself called `rapier`, so `mod rapier;` and `crate::rapier`
//! appear elsewhere: what is held is the crate's name, `rapier3d`, which no wrapper can avoid naming.

use std::path::{Path, PathBuf};

fn pack() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn root() -> PathBuf {
    pack().join("../..")
}

/// Every file under `directory`, recursively, skipping build output, VCS and hidden directories.
fn files_under(directory: &Path, into: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("a scanned directory is missing: {directory:?}: {error}"));
    for entry in entries {
        let path = entry.expect("a directory entry").path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if path.is_dir() {
            if name != "target" && !name.starts_with('.') && name != "node_modules" {
                files_under(&path, into);
            }
        } else {
            into.push(path);
        }
    }
}

/// `path:line` for every line of `path` that contains `needle`.
fn naming(path: &Path, needle: &str) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .enumerate()
        .filter(|(_, line)| line.contains(needle))
        .map(|(number, _)| format!("{}:{}", path.display(), number + 1))
        .collect()
}

fn sources() -> Vec<PathBuf> {
    let mut files = Vec::new();
    files_under(&pack().join("src"), &mut files);
    assert!(
        files.iter().any(|file| file.ends_with("src/rapier.rs")),
        "the adapter exists: {files:?}"
    );
    files
}

#[test]
fn only_the_adapter_names_rapier() {
    let found: Vec<String> = sources()
        .iter()
        .filter(|file| !file.ends_with("src/rapier.rs"))
        .flat_map(|file| naming(file, "rapier3d"))
        .collect();
    assert!(
        found.is_empty(),
        "only systems/bodies/src/rapier.rs may name rapier3d (DEP-13):\n{}",
        found.join("\n")
    );
}

#[test]
fn no_code_outside_this_pack_names_rapier() {
    let mut files = Vec::new();
    files_under(&root(), &mut files);
    let bodies = pack().canonicalize().expect("the pack's directory");
    let code: Vec<&PathBuf> = files
        .iter()
        .filter(|file| {
            file.extension().is_some_and(|extension| extension == "rs")
                || file.file_name().is_some_and(|name| name == "Cargo.toml")
        })
        .collect();
    assert!(
        code.len() > 200,
        "the workspace's code is scanned: {}",
        code.len()
    );
    let found: Vec<String> = code
        .iter()
        .filter(|file| {
            !file
                .canonicalize()
                .is_ok_and(|canonical| canonical.starts_with(&bodies))
        })
        .flat_map(|file| naming(file, "rapier3d"))
        .collect();
    assert!(
        found.is_empty(),
        "no code outside systems/bodies names rapier3d (DEP-13, step-11 §17.0):\n{}",
        found.join("\n")
    );
}

#[test]
fn no_float_outside_the_adapter() {
    let found: Vec<String> = sources()
        .iter()
        .filter(|file| !file.ends_with("src/rapier.rs"))
        .flat_map(|file| {
            let mut lines = naming(file, "f32");
            lines.extend(naming(file, "f64"));
            lines
        })
        .collect();
    assert!(
        found.is_empty(),
        "positions, shapes and facts are integers; only the sweep is float (step-11 I-3):\n{}",
        found.join("\n")
    );
}

#[test]
fn this_pack_depends_on_no_system_pack_but_presence() {
    let manifest = std::fs::read_to_string(pack().join("Cargo.toml")).expect("the manifest reads");
    let dependencies: Vec<&str> = manifest
        .split("[dependencies]")
        .nth(1)
        .expect("a [dependencies] table")
        .split("\n[")
        .next()
        .expect("its body")
        .lines()
        .filter_map(|line| line.split_once('=').map(|(name, _)| name.trim()))
        .filter(|name| !name.starts_with('#') && !name.is_empty())
        .collect();
    let mut packs = Vec::new();
    files_under_packs(&root().join("systems"), &mut packs);
    assert!(packs.len() >= 14, "the System Packs are listed: {packs:?}");
    let named: Vec<&str> = dependencies
        .iter()
        .copied()
        .filter(|name| {
            packs
                .iter()
                .any(|pack| *name == format!("mineworld-{pack}") && pack != "bodies")
        })
        .collect();
    println!("dependencies {dependencies:?}; packs named {named:?}");
    assert_eq!(named, ["mineworld-presence"], "presence, and no other pack");
}

/// The directory names under systems/ that hold a crate.
fn files_under_packs(systems: &Path, into: &mut Vec<String>) {
    for entry in std::fs::read_dir(systems).expect("systems/ lists") {
        let path = entry.expect("an entry").path();
        if path.join("Cargo.toml").is_file() {
            into.push(
                path.file_name()
                    .and_then(|name| name.to_str())
                    .expect("a name")
                    .to_owned(),
            );
        }
    }
}
