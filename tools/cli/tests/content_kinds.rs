//! Item kinds and organizations through the real binary (`DECISIONS.md` `ARC-36`): `validate` names
//! them, and a world that declares them but whose packs never read them runs exactly as it would
//! without them, saved and resumed (`AC-6`, `AC-12`).
//!
//! The world is `worlds/social-cafe` copied at runtime with two item kinds and one organization added.
//! Nothing is committed under `worlds/`: the copy is written under the test target directory.

mod headless;

use std::path::Path;

use headless::{PACK, Scratch, Tables, fresh, mineworld, stderr, stdout};

/// Copies `from` into `to`, recursively.
fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a writable directory");
    for entry in std::fs::read_dir(from).expect("a readable directory") {
        let entry = entry.expect("a directory entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a file type").is_dir() {
            copy(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("a copyable file");
        }
    }
}

/// `worlds/social-cafe` as `with-things`, plus item kinds `lantern` and `pebble` and the organization
/// `chess-club`, each a tags-only file.
fn with_things(name: &str) -> Scratch {
    let root = fresh(name).within("with-things");
    copy(Path::new(PACK), &root);
    let manifest = root.join("world.yaml");
    let text = std::fs::read_to_string(&manifest).expect("social-cafe's manifest");
    assert!(
        text.contains("  id: social-cafe\n"),
        "the copied manifest names its id"
    );
    let text = text.replace("  id: social-cafe\n", "  id: with-things\n")
        + "\nitems:\n  - lantern\n  - pebble\n\norganizations:\n  - chess-club\n";
    std::fs::write(&manifest, text).expect("a writable manifest");
    for (relative, contents) in [
        ("items/lantern.yaml", "tags: [light]\n"),
        ("items/pebble.yaml", "tags: [stone]\n"),
        ("organizations/chess-club.yaml", "tags: [club]\n"),
    ] {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("a writable directory");
        std::fs::write(path, contents).expect("a writable file");
    }
    root
}

fn validate(world: &Path) -> String {
    let output = mineworld(&["validate", world.to_str().expect("a printable path")]);
    assert!(
        output.status.success(),
        "validate failed: {}\n{}",
        stderr(&output),
        stdout(&output)
    );
    stdout(&output)
}

/// `run` of `world` with these settings and a save, which must succeed.
fn run(world: &Path, seed: u64, days: u64, save: &Path) {
    let output = mineworld(&[
        "run",
        world.to_str().expect("a printable path"),
        "--headless",
        "--seed",
        &seed.to_string(),
        "--days",
        &days.to_string(),
        "--save",
        save.to_str().expect("a printable path"),
    ]);
    assert!(
        output.status.success(),
        "run failed: {}\n{}",
        stderr(&output),
        stdout(&output)
    );
}

/// The `  <id>  <key>` lines of a `validate` report.
fn id_lines(report: &str) -> Vec<&str> {
    report
        .lines()
        .filter(|line| {
            line.trim_start()
                .split_once("  ")
                .is_some_and(|(id, _)| id.parse::<u64>().is_ok())
        })
        .collect()
}

#[test]
fn validate_lists_items_and_organizations_after_every_existing_id() {
    let world = with_things("content-kinds-validate");

    let report = validate(&world);
    let original = validate(Path::new(PACK));

    assert!(
        report.contains("\n  items      lantern, pebble\n"),
        "the items line: {report}"
    );
    assert!(
        report.contains("\n  organizations chess-club\n"),
        "the organizations line: {report}"
    );
    assert!(
        !original.contains("  items ") && !original.contains("  organizations "),
        "a pack that declares neither prints neither line: {original}"
    );

    let ids = id_lines(&report);
    let original_ids = id_lines(&original);
    assert_eq!(
        original_ids.len(),
        18,
        "social-cafe's own report: {original}"
    );
    assert_eq!(
        ids[..18],
        original_ids[..],
        "every existing id stays where it was"
    );
    assert_eq!(
        ids[18..],
        ["  19  lantern", "  20  pebble", "  21  chess-club"],
        "and the new kinds come after them: {report}"
    );
    assert!(
        report.contains("\n53 genesis fact(s)"),
        "tags-only files add no genesis fact: {report}"
    );
}

#[test]
fn inert_items_and_organizations_change_no_fact_of_a_run() {
    let world = with_things("content-kinds-inert");
    let directory = fresh("content-kinds-inert-saves");
    std::fs::create_dir_all(&directory).expect("a writable directory");
    let (theirs, ours) = (directory.join("social-cafe"), directory.join("with-things"));

    run(Path::new(PACK), 7, 10, &theirs);
    run(&world, 7, 10, &ours);

    let (theirs, ours) = (Tables::read(&theirs), Tables::read(&ours));
    assert!(
        theirs.facts.len() > 1_000,
        "a busy town, not its genesis: {} facts",
        theirs.facts.len()
    );
    assert_eq!(ours.facts.len(), theirs.facts.len(), "fact counts");
    if let Some(index) = ours
        .facts
        .iter()
        .zip(&theirs.facts)
        .position(|(a, b)| a != b)
    {
        panic!(
            "the fact tables differ first at row {index} (key {} vs {})",
            ours.facts[index].0, theirs.facts[index].0
        );
    }
}

#[test]
fn a_world_with_items_and_organizations_resumes_byte_for_byte() {
    let world = with_things("content-kinds-resume");
    let directory = fresh("content-kinds-resume-saves");
    std::fs::create_dir_all(&directory).expect("a writable directory");
    let (control, continued) = (directory.join("control"), directory.join("continued"));

    run(&world, 7, 10, &control);
    run(&world, 7, 5, &continued);
    run(&world, 7, 10, &continued);

    Tables::read(&continued).assert_same_history(
        &Tables::read(&control),
        "resumed at day 5 against uninterrupted",
    );
}
