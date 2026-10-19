//! PB-15 — Rapier stays behind one module, no float reaches anything persisted, and this pack depends
//! on no System Pack but presence (`DECISIONS.md` `DEP-13`; step-11 I-3, I-5, SD-B1).
//!
//! ```text
//! the crate        in systems/bodies/src, only rapier.rs names `rapier3d`; outside systems/bodies,
//!                  no code file (*.rs, Cargo.toml) names it at all — the pin lives in this pack's own
//!                  manifest (step-11 §17.0, QP-3 overruled)
//! floats           no source file but rapier.rs names f32 or f64: positions, shapes and facts are
//!                  integers, and only the sweep is float
//! dependencies     the pack's system dependency is presence alone; the [dependencies] of its crate
//!                  name no System Pack crate but presence, item and movement (step-11 QO-4, QO-16,
//!                  QD-5)
//! item             the item crate is used for one read only: every `mineworld_item` in this pack's
//!                  sources is `mineworld_item::is_declared(` (step-11 §18.0's bound on the crate
//!                  dependency; DECISIONS.md ARC-39 note 2, point 4)
//! movement         the movement crate is used for the Wayfinder seam this pack answers through
//!                  (DECISIONS.md ARC-39 note 5) and for `Passages`, read once by the genesis check
//!                  (step-11 §19, SD-D5, QD-5; ARC-39 note 7), and nothing else
//! ```
//!
//! The module that wraps the crate is itself called `rapier`, so `mod rapier;` and `crate::rapier`
//! appear elsewhere: what is held is the crate's name, `rapier3d`, which no wrapper can avoid naming.

use std::path::{Path, PathBuf};

use mineworld_bodies::BodiesSystem;
use mineworld_kernel::{System, SystemIdentity};
use mineworld_presence::PresenceSystem;

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
fn this_packs_system_dependency_is_presence_and_its_pack_crates_are_presence_and_item() {
    assert_eq!(
        BodiesSystem.declaration().depends_on(),
        [PresenceSystem::ID],
        "the system dependency: presence alone — a world installs bodies without item"
    );
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
    assert_eq!(
        named,
        ["mineworld-item", "mineworld-movement", "mineworld-presence"],
        "presence, item, and movement for its Wayfinder trait and its Passages (ARC-39 notes 5 and \
         7) — no other pack crate"
    );
}

/// NV-6 (step-11 §21.8): `pathfinding` is named only in `src/route.rs` and in this pack's manifest —
/// no other code file of the repository names it — and movement, which owns the catalog this pack
/// answers through, names no `bodies` anywhere in its sources (`ARC-62`: an owner never names its
/// implementers).
#[test]
fn the_search_crate_stays_in_route_rs_and_movement_names_no_bodies() {
    let mut files = Vec::new();
    files_under(&root(), &mut files);
    let found: Vec<String> = files
        .iter()
        .filter(|file| {
            file.extension()
                .is_some_and(|extension| extension == "rs" || extension == "toml")
                && file.file_name().is_some_and(|name| name != "Cargo.lock")
        })
        .filter(|file| !file.ends_with("systems/bodies/src/route.rs"))
        .filter(|file| !file.ends_with("systems/bodies/Cargo.toml"))
        .filter(|file| !file.ends_with("systems/bodies/tests/isolation.rs"))
        .flat_map(|file| naming(file, "pathfinding"))
        .collect();
    assert!(
        found.is_empty(),
        "pathfinding is named only in route.rs (DEP-34):\n{}",
        found.join("\n")
    );
    let mut movement = Vec::new();
    files_under(&root().join("systems/movement/src"), &mut movement);
    assert!(movement.len() >= 7, "movement's sources are scanned");
    let named: Vec<String> = movement
        .iter()
        .flat_map(|file| {
            let mut lines = naming(file, "bodies");
            lines.extend(naming(file, "Bodies"));
            lines
        })
        .collect();
    assert!(
        named.is_empty(),
        "movement never names the pack that answers its catalog:\n{}",
        named.join("\n")
    );
}

/// What this pack uses of the movement crate (step-11 §19, SD-D5, QD-5; ARC-39 notes 5 and 7): the
/// Wayfinder seam it answers through, and `Passages` — the doorway points the genesis check reads —
/// named exactly once, in `genesis.rs`. Nothing else of movement: not its stride rule, its constants or
/// its walk.
#[test]
fn this_pack_uses_nothing_of_the_movement_crate_but_the_wayfinder_seam_and_passages() {
    const CRATE: &str = "mineworld_movement::";
    const SEAM: [&str; 6] = [
        "RouteAnswer",
        "RouteAsk",
        "Wayfinder",
        "Waypoints",
        "register_wayfinders",
        "require_wayfinder",
    ];
    let mut passages = Vec::new();
    let mut other = Vec::new();
    for file in sources() {
        let text = std::fs::read_to_string(&file).expect("a source reads");
        for (number, line) in text.lines().enumerate() {
            let place = format!("{}:{}", file.display(), number + 1);
            let mut rest = line;
            while let Some(at) = rest.find(CRATE) {
                let tail = &rest[at + CRATE.len()..];
                // One name, or a `{ … }` list of them.
                let names: Vec<&str> = if let Some(list) = tail.strip_prefix('{') {
                    list.split('}')
                        .next()
                        .unwrap_or("")
                        .split(',')
                        .map(str::trim)
                        .filter(|name| !name.is_empty())
                        .collect()
                } else {
                    let end = tail
                        .find(|c: char| !c.is_alphanumeric() && c != '_')
                        .unwrap_or(tail.len());
                    vec![&tail[..end]]
                };
                for name in names {
                    if name == "Passages" {
                        passages.push(place.clone());
                    } else if !SEAM.contains(&name) {
                        other.push(format!("{place}: {name}"));
                    }
                }
                rest = tail;
            }
        }
    }
    println!("reads of Passages: {passages:?}");
    assert!(
        other.is_empty(),
        "bodies names nothing of mineworld_movement but the Wayfinder seam and Passages (step-11 \
         QD-5):\n{}",
        other.join("\n")
    );
    assert_eq!(
        passages.len(),
        1,
        "exactly one, in the genesis check: {passages:?}"
    );
    assert!(
        passages[0].contains("genesis.rs"),
        "Passages is read by the genesis check: {passages:?}"
    );
}

/// The one read: every use of the item crate in this pack's sources is a call of
/// `mineworld_item::is_declared` — no `use` of it, no other item, type or function.
#[test]
fn this_pack_uses_nothing_of_the_item_crate_but_is_declared() {
    const READ: &str = "mineworld_item::is_declared(";
    let mut reads = Vec::new();
    let mut other = Vec::new();
    for file in sources() {
        let text = std::fs::read_to_string(&file).expect("a source reads");
        for (number, line) in text.lines().enumerate() {
            let place = format!("{}:{}", file.display(), number + 1);
            let mut rest = line;
            while let Some(at) = rest.find("mineworld_item") {
                if rest[at..].starts_with(READ) {
                    reads.push(place.clone());
                } else {
                    other.push(format!("{place}: {}", line.trim()));
                }
                rest = &rest[at + "mineworld_item".len()..];
            }
        }
    }
    println!("reads of is_declared: {reads:?}");
    assert!(
        other.is_empty(),
        "bodies names nothing of mineworld_item but is_declared (step-11 §18.0):\n{}",
        other.join("\n")
    );
    assert_eq!(
        reads.len(),
        1,
        "exactly one call, in the genesis check: {reads:?}"
    );
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
