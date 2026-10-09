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

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("writable");
    for entry in std::fs::read_dir(from).expect("readable") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copied");
        }
    }
}

/// A scratch copy of social-cafe, named as the pack, with `configure:` listing `keys` and these files.
fn configured(
    name: &str,
    keys: &[&str],
    files: &[(&str, &str)],
) -> mineworld_test_support::Scratch {
    let copy = mineworld_test_support::scratch!(name).within("social-cafe");
    copy_dir(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../worlds/social-cafe"),
        &copy,
    );
    let manifest = copy.join("world.yaml");
    let text = std::fs::read_to_string(&manifest).expect("reads");
    let listed: String = keys.iter().map(|key| format!("  - {key}\n")).collect();
    std::fs::write(&manifest, format!("{text}\nconfigure:\n{listed}")).expect("writes");
    std::fs::create_dir_all(copy.join("configure")).expect("writable");
    for (file, text) in files {
        std::fs::write(copy.join(file), text).expect("writes");
    }
    copy
}

/// IB-11: a configured section, a region and a class, shown as resolved — the base, the café's region,
/// each person's class, and "default (compiled)" for the section left unconfigured; the JSON
/// byte-stable over two runs; `--place` shows what applies there; the World Pack unchanged.
#[test]
fn a_configured_world_shows_its_resolved_sections_regions_and_classes() {
    let world = configured(
        "cli-interactions-configured",
        &["classes", "conversation"],
        &[
            (
                "configure/classes.yaml",
                "- { class: regular, of: person, tag: regular }\n",
            ),
            (
                "configure/conversation.yaml",
                "parameters:\n  - { gap: 600 }\n  - { actor: regular, gap: 3600 }\n\
                 regions:\n  cafe:\n    parameters: [ { gap: 1800 } ]\n",
            ),
        ],
    );
    let before = listing(&world);
    let first = interactions(&world, &["--json"]);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let second = interactions(&world, &["--json"]);
    assert_eq!(first.stdout, second.stdout, "the JSON is byte-stable");
    let document: serde_json::Value = serde_json::from_slice(&first.stdout).expect("JSON");
    let conversation = &document["sections"]["conversation"];
    assert_eq!(conversation["base"]["parameters"]["gap"], 600);
    assert_eq!(conversation["regions"][0][0], "cafe");
    assert_eq!(conversation["regions"][0][1]["parameters"]["gap"], 1800);
    assert_eq!(conversation["base"]["scoped"][0]["fields"]["gap"], 3600);
    assert_eq!(document["sections"]["group-activity"], "default (compiled)");
    assert_eq!(document["classes"]["bob"], "regular");
    assert_eq!(document["classes"]["alice"], "person");

    let at_cafe = interactions(&world, &["--json", "--place", "cafe"]);
    let at_cafe: serde_json::Value = serde_json::from_slice(&at_cafe.stdout).expect("JSON");
    assert_eq!(
        at_cafe["sections"]["conversation"]["region"]["parameters"]["gap"],
        1800
    );
    let at_park = interactions(&world, &["--json", "--place", "park"]);
    let at_park: serde_json::Value = serde_json::from_slice(&at_park.stdout).expect("JSON");
    assert_eq!(
        at_park["sections"]["conversation"]["base"]["parameters"]["gap"],
        600
    );

    let text = interactions(&world, &[]);
    let text = String::from_utf8_lossy(&text.stdout);
    assert!(text.contains("section  conversation:"), "{text}");
    assert!(
        text.contains("section  group-activity: default (compiled)"),
        "{text}"
    );
    assert!(text.contains("class    bob              regular"), "{text}");
    assert_eq!(listing(&world), before, "the World Pack is read only");
}

/// IB-7 through the real `mineworld validate`: a rule in a section whose pack declares no action, a
/// parameter outside its bound (at its line and column), an undefined class and an ambiguous pair
/// (both named) are refused by name.
#[test]
fn validate_refuses_each_section_mistake_by_name() {
    for (name, keys, file, text, expected) in [
        (
            "cli-ib7-rules",
            &["conversation"][..],
            "configure/conversation.yaml",
            "rules:\n  - { action: whisper, effect: forbid }\n",
            &[
                "line 2",
                "'whisper' is not an action 'conversation' declares (it declares: none)",
            ][..],
        ),
        (
            "cli-ib7-bound",
            &["conversation"][..],
            "configure/conversation.yaml",
            "parameters:\n  - { gap: 0 }\n",
            &["line 2 column", "'gap' is 1 … 86400, not 0"][..],
        ),
        (
            "cli-ib7-undefined",
            &["conversation"][..],
            "configure/conversation.yaml",
            "parameters:\n  - { actor: knight, gap: 60 }\n",
            &["parameters[0]", "'knight'"][..],
        ),
        (
            "cli-ib7-ambiguous",
            &["conversation"][..],
            "configure/conversation.yaml",
            "parameters:\n  - { actor: person, gap: 60 }\n  - { target: person, gap: 90 }\n",
            &["parameters[0]", "parameters[1]", "'gap'"][..],
        ),
    ] {
        let world = configured(name, keys, &[(file, text)]);
        let output = Command::new(env!("CARGO_BIN_EXE_mineworld"))
            .arg("validate")
            .arg(world.path())
            .output()
            .expect("the binary runs");
        let complaint = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "{name}: refused");
        for needle in expected {
            assert!(
                complaint.contains(needle),
                "{name}: `{needle}` in {complaint}"
            );
        }
    }
}
