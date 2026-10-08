//! SC-7 — the arrival-resolver seam names no physics, and movement names no resolver (step-11 §16
//! RS-13; `docs/DECISIONS.md` `ARC-39`).
//!
//! PR 12a is a framework precursor: the seam exists before the first pack that resolves arrivals, and
//! must not be shaped by it in name. Two absences are held here, structurally, because only a
//! structural test can hold an absence:
//!
//! ```text
//! the seam's code       no word beginning body, bodies, bodily, physic, rapier, nudg, collision,
//!                       collid, capsule or jolt — in presence's, movement's, the sdk's, the installed
//!                       set's and worldpack's sources, and in every test file PR 12a added
//! movement's sources    none of ArrivalResolver, Resolution, register_resolvers, require_registered,
//!                       test-fences, test-grid, test-rogue: movement states arrivals and never learns
//!                       who resolves them
//! ```
//!
//! Every non-Markdown file is read, comments included. Markdown is not: documentation must be able to
//! name the S15 step the seam serves. This file is not scanned either, because it has to name the
//! vocabulary it looks for.
//!
//! Words are split as the I-2 scan splits them (`precursor_vocabulary.rs`, `ARC-35` item 7) — at every
//! character that is not an ASCII letter or digit, and at every lower-to-upper case boundary — so
//! `RigidBody`, `body_shape` and `Bodies` are all seen. The splitter is written again here rather than
//! shared, so that file stays unedited. A listed directory or file that is missing fails the test.

use std::path::{Path, PathBuf};

/// The physics vocabulary: a word matches when, lowercased, it begins with one of these.
const PHYSICS: [&str; 10] = [
    "body",
    "bodies",
    "bodily",
    "physic",
    "rapier",
    "nudg",
    "collision",
    "collid",
    "capsule",
    "jolt",
];

/// Text that predates PR 12a, is not the seam, and may not be edited by it: (file, a substring of the
/// line, the word it admits, why). One entry admits one word on the lines of one file that contain its
/// substring, and an entry that admits nothing fails the test, so the list cannot go stale (step-11
/// §16.11 DR-4).
const PRE_EXISTING: [(&str, &str, &str, &str); 3] = [
    (
        "systems/movement/src/action.rs",
        "A body walking or jogging",
        "body",
        "MAX_STRIDE's documentation since S6 (41bb073): a person's body reporting its position, not \
         the seam; PR 12a may change movement's system.rs only (SD-R9)",
    ),
    (
        "systems/movement/src/action.rs",
        "this pack's policy rather than a physical",
        "physical",
        "MAX_STRIDE's documentation since S6 (41bb073): \"policy rather than a physical constant\", \
         not the seam; PR 12a may change movement's system.rs only (SD-R9)",
    ),
    (
        "worldpack/src/read.rs",
        "rather than a collision",
        "collision",
        "the World Pack reader since ff49368: a seat key repeating a population key is not a key \
         collision; a file PR 12a does not touch",
    ),
];

/// The installed set names its packs (`ARC-33`), and since PR 12b one of them is the first resolver's
/// pack, whose name is a word of the vocabulary above. These entries admit that name on exactly the
/// lines that list the pack — the installed set's two lines, and the one line of the registration test
/// that pins the list it registers — with the same rules as [`PRE_EXISTING`]: one word, lines holding
/// the substring, and an entry that admits nothing fails (step-11 §17.0 QP-2, §17.11 DB-8). Every other
/// line of every scanned file is still refused the word.
const INSTALLED_PACK_LINES: [(&str, &str, &str, &str); 2] = [
    (
        "systems/installed/src/lib.rs",
        "mineworld_bodies::BodiesSystem",
        "bodies",
        "ARC-33: the installed set lists each pack it links, here on its pack line and on the \
         resolution: line; the seam itself still names no pack",
    ),
    (
        "worldpack/tests/registration.rs",
        "const LISTED: &str = \"bodies\";",
        "bodies",
        "QP-2: the registration test pins the installed set's resolution: list, which names the pack \
         it registers",
    ),
];

/// Every admission, in one order: what predates the seam, then the installed set's pack lines.
fn admissions()
-> impl Iterator<Item = &'static (&'static str, &'static str, &'static str, &'static str)> {
    PRE_EXISTING.iter().chain(INSTALLED_PACK_LINES.iter())
}

/// How many admissions there are.
const ADMISSIONS: usize = PRE_EXISTING.len() + INSTALLED_PACK_LINES.len();

/// What movement must never name: the seam's resolver side and the synthetic resolvers.
const RESOLVER_NAMES: [&str; 7] = [
    "ArrivalResolver",
    "Resolution",
    "register_resolvers",
    "require_registered",
    "test-fences",
    "test-grid",
    "test-rogue",
];

/// The source directories the seam lives in, relative to the repository root.
const SEAM_DIRECTORIES: [&str; 5] = [
    "systems/presence/src",
    "systems/movement/src",
    "sdk/rust/src",
    "systems/installed/src",
    "worldpack/src",
];

/// Every test file PR 12a adds, but this one.
const ADDED_TEST_FILES: [&str; 7] = [
    "systems/presence/tests/resolver_catalog.rs",
    "systems/installed/tests/resolution.rs",
    "worldpack/tests/registration.rs",
    "tests/acceptance/tests/resolvers/mod.rs",
    "tests/acceptance/tests/arrival_resolvers.rs",
    "tests/acceptance/tests/arrival_resolvers_unregistered.rs",
    "tests/acceptance/tests/arrival_resolvers_resume.rs",
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A line's words, split as the I-2 scan splits them, lowercased.
fn words(line: &str) -> Vec<String> {
    let mut words = Vec::new();
    for run in line.split(|c: char| !c.is_ascii_alphanumeric()) {
        let mut word = String::new();
        let mut after_lower = false;
        for c in run.chars() {
            if c.is_ascii_uppercase() && after_lower {
                words.push(std::mem::take(&mut word));
            }
            after_lower = c.is_ascii_lowercase() || c.is_ascii_digit();
            word.push(c.to_ascii_lowercase());
        }
        if !word.is_empty() {
            words.push(word);
        }
    }
    words
}

/// Every non-Markdown file under `directory`, recursively; fails if the directory is missing.
fn files_under(directory: &Path, into: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("a scanned directory is missing: {directory:?}: {error}"));
    for entry in entries {
        let path = entry.expect("a directory entry").path();
        if path.is_dir() {
            files_under(&path, into);
        } else if path.extension().is_none_or(|extension| extension != "md") {
            into.push(path);
        }
    }
}

/// Every physics word in `text`, as `path:line: word`, except those an entry of [`admissions`]
/// admits; each admitting entry's index is marked in `used`.
fn physics_words(path: &Path, text: &str, used: &mut [bool]) -> Vec<String> {
    let mut found = Vec::new();
    for (number, line) in text.lines().enumerate() {
        for word in words(line) {
            if !PHYSICS.iter().any(|physics| word.starts_with(physics)) {
                continue;
            }
            let admitted = admissions().position(|(file, substring, admits, _)| {
                path.ends_with(file) && line.contains(substring) && word == *admits
            });
            match admitted {
                Some(entry) => used[entry] = true,
                None => found.push(format!("{}:{}: {word}", path.display(), number + 1)),
            }
        }
    }
    found
}

#[test]
fn the_seam_names_no_physics() {
    let root = root();
    let mut files = Vec::new();
    for directory in SEAM_DIRECTORIES {
        files_under(&root.join(directory), &mut files);
    }
    for file in ADDED_TEST_FILES {
        let path = root.join(file);
        assert!(path.is_file(), "a scanned file is missing: {file}");
        files.push(path);
    }
    // presence 8, movement 6, sdk 4, installed 1, worldpack 7 source files, and the 7 added tests.
    assert!(files.len() >= 33, "every file is scanned: {}", files.len());

    let mut found = Vec::new();
    let mut used = [false; ADMISSIONS];
    for file in &files {
        let text = std::fs::read_to_string(file).expect("a scanned file reads");
        found.extend(physics_words(file, &text, &mut used));
    }
    assert!(
        found.is_empty(),
        "the arrival-resolver seam must name no physics (ARC-39):\n{}",
        found.join("\n")
    );
    for ((file, substring, word, reason), used) in admissions().zip(used) {
        assert!(!reason.is_empty(), "{file}: an exemption needs a reason");
        assert!(
            used,
            "{file}: the exemption for {word:?} on \"{substring}\" admits nothing; remove it"
        );
    }
}

#[test]
fn movement_names_no_resolver() {
    let mut files = Vec::new();
    files_under(&root().join("systems/movement/src"), &mut files);
    assert!(files.len() >= 6, "every module of movement is scanned");

    let mut found = Vec::new();
    for file in &files {
        let text = std::fs::read_to_string(file).expect("a source file reads");
        for (number, line) in text.lines().enumerate() {
            for name in RESOLVER_NAMES {
                if line.contains(name) {
                    found.push(format!("{}:{}: {name}", file.display(), number + 1));
                }
            }
        }
    }
    assert!(
        found.is_empty(),
        "movement states arrivals and never learns who resolves them (ARC-39):\n{}",
        found.join("\n")
    );
}

/// The splitter sees the vocabulary inside identifiers and not inside other words.
#[test]
fn the_splitter_finds_physics_inside_identifiers_and_not_inside_other_words() {
    let found = physics_words(
        Path::new("x.rs"),
        "let RigidBody = body_shape; // Bodies collide; nobody, somebody, embody",
        &mut [false; ADMISSIONS],
    );
    assert_eq!(
        found,
        [
            "x.rs:1: body",
            "x.rs:1: body",
            "x.rs:1: bodies",
            "x.rs:1: collide",
        ]
    );
}
