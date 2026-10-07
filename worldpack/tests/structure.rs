//! The World Pack loader names no System Pack except the two its format is bound to
//! (`DECISIONS.md` `ARC-33`).
//!
//! It reaches every pack through the build's installed set (`systems/installed`). A pack it named
//! directly — a dependency line, an import — would be a pack whose installation edits this crate,
//! which is finding F-1 coming back. `presence` and `movement` are the exceptions, because a person's
//! `location` and a place's `passages` are fields of the format whose state those two packs own
//! (`ARC-31`, point 5).
//!
//! The allow-list names infrastructure and those two, never another pack, so installing a pack never
//! edits it.

use std::collections::BTreeSet;
use std::path::Path;

/// Every `mineworld_*` crate this crate may name, and why.
const ALLOWED: &[(&str, &str)] = &[
    ("mineworld_contracts", "the contract layer"),
    ("mineworld_kernel", "the kernel a world is assembled in"),
    ("mineworld_authoring", "the section seam (ARC-31)"),
    ("mineworld_sdk", "SectionOwner, re-exported"),
    (
        "mineworld_installed_systems",
        "the installed set: every pack, reached without naming it",
    ),
    (
        "mineworld_presence",
        "owns the state the format's `location` field becomes",
    ),
    (
        "mineworld_movement",
        "owns the state the format's `passages` field becomes",
    ),
    (
        "mineworld_worldpack",
        "this crate, in its own documentation's examples",
    ),
];

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

/// The `mineworld-*` keys of the manifest's `[dependencies]` table, as Rust crate names.
fn manifest_dependencies() -> BTreeSet<String> {
    let manifest = std::fs::read_to_string(Path::new(ROOT).join("Cargo.toml")).expect("manifest");
    let mut in_dependencies = false;
    let mut names = BTreeSet::new();
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            in_dependencies = line == "[dependencies]";
            continue;
        }
        if !in_dependencies || line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, _) = line
            .split_once('=')
            .unwrap_or_else(|| panic!("an unreadable [dependencies] line: {line}"));
        let name = name.trim();
        if name.starts_with("mineworld-") {
            names.insert(name.replace('-', "_"));
        }
    }
    assert!(!names.is_empty(), "no mineworld dependency was read");
    names
}

/// Every `mineworld_*` crate named anywhere in `src/`, with the file it was first seen in.
fn named_in_sources() -> Vec<(String, String)> {
    let mut files = Vec::new();
    collect_rust_files(&Path::new(ROOT).join("src"), &mut files);
    assert!(!files.is_empty(), "no source file was read");
    let mut named = Vec::new();
    for file in files {
        let text = std::fs::read_to_string(&file).expect("a source file");
        for (at, _) in text.match_indices("mineworld_") {
            let name: String = text[at..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            named.push((name, file.display().to_string()));
        }
    }
    named
}

fn collect_rust_files(directory: &Path, files: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(directory).expect("a readable directory") {
        let path = entry.expect("a directory entry").path();
        if path.is_dir() {
            collect_rust_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

fn allowed(name: &str) -> bool {
    ALLOWED.iter().any(|(allowed, _)| *allowed == name)
}

#[test]
fn the_manifest_depends_on_no_system_pack_but_presence_and_movement() {
    let refused: Vec<String> = manifest_dependencies()
        .into_iter()
        .filter(|name| !allowed(name))
        .collect();
    assert!(
        refused.is_empty(),
        "worldpack depends on {refused:?}; a System Pack is reached through \
         mineworld-installed-systems, never named here (ARC-33)"
    );
}

#[test]
fn the_sources_name_no_system_pack_but_presence_and_movement() {
    let refused: Vec<(String, String)> = named_in_sources()
        .into_iter()
        .filter(|(name, _)| !allowed(name))
        .collect();
    assert!(
        refused.is_empty(),
        "worldpack's sources name {refused:?}; a System Pack is reached through \
         mineworld-installed-systems, never named here (ARC-33)"
    );
}
