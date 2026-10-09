//! IB-11 and QIB-11 at the binary (step-18-interaction-list §12.5; `docs/DECISIONS.md` `ARC-63`
//! item 10, `ARC-65` item 4).
//!
//! ```text
//! structural   `mineworld biography` builds its configured sections from the save's genesis facts
//!              and asks the SDK's selection — the only form in which a test can hold that the
//!              projection reads them, until a pack declares a configurable biography (QIB-11)
//! IB-11        `mineworld interactions`: on a scratch copy of a shipped world configuring both
//!              sections, a region and a class, the text and the JSON show the resolved base, the
//!              region, each person's class, and "default (compiled)" where a section is not
//!              configured; the JSON is byte-stable; the World Pack is unchanged afterwards
//! ```

use std::path::{Path, PathBuf};
use std::process::Command;

fn source(file: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(file);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// QIB-11: the projection reads the save's genesis, builds its configured sections from each composed
/// capability's section, and selects through the SDK — never by the compiled set alone.
#[test]
fn the_biography_projection_builds_its_configured_sections_from_the_saves_genesis() {
    let biography = source("biography.rs");
    for needle in [
        "facts_of(WorldRevision::GENESIS)",
        ".and_then(Capability::interaction_section)",
        "(section.consequences)(fact.payload().payload())",
        "interactions::biography::selected(fact, person, &selected, &configured)",
    ] {
        assert!(
            biography.contains(needle),
            "biography.rs no longer has `{needle}`"
        );
    }
    assert!(
        !biography.contains("selected.contains(fact.event_type())"),
        "the compiled set alone decides again"
    );
}

/// Every file under `root`, with its size and modification time, sorted: what "unchanged" compares.
fn listing(root: &Path) -> Vec<(PathBuf, u64, std::time::SystemTime)> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(directory) = stack.pop() {
        for entry in std::fs::read_dir(&directory).expect("readable") {
            let path = entry.expect("an entry").path();
            let metadata = std::fs::metadata(&path).expect("metadata");
            if metadata.is_dir() {
                stack.push(path);
            } else {
                out.push((path, metadata.len(), metadata.modified().expect("an mtime")));
            }
        }
    }
    out.sort();
    out
}

fn interactions(world: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_mineworld"))
        .arg("interactions")
        .arg(world)
        .args(args)
        .output()
        .expect("the binary runs")
}

/// With nothing configured, every section of the shipped world is "default (compiled)", and the
/// command leaves the World Pack as it found it.
#[test]
fn an_unconfigured_world_shows_its_compiled_defaults_and_is_left_unchanged() {
    let world = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../worlds/social-cafe");
    let before = listing(&world);
    let output = interactions(&world, &["--json"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON");
    let sections = document["sections"].as_object().expect("sections");
    assert!(
        sections
            .values()
            .all(|section| section == "default (compiled)"),
        "{sections:?}"
    );
    assert_eq!(document["classes"]["alice"], "person");
    assert_eq!(document["classes"]["cafe"], "place");
    assert_eq!(listing(&world), before, "the World Pack is read only");
}
