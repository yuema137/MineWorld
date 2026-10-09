//! IA-8 and IB-12 — the configuration seam and the Interaction List's framework name no pack's
//! vocabulary (step-18-interaction-list §11.4, §12.5; `docs/DECISIONS.md` `ARC-61` … `ARC-64`).
//!
//! The seam (PR IL-a) and the schema (PR IL-b) exist for every pack and must not be shaped by any one
//! in name. The absence is held structurally:
//!
//! ```text
//! the framework's code   no word beginning with a word of the seam scan's physics list
//!                        (`seam_vocabulary.rs`), nor talk, spoke, convers, give, buy, sell, trade, eat,
//!                        drink, kick, throw, shove or invit — in authoring's configuration contract,
//!                        classes and attachments, worldpack's configure module and its in-crate tests,
//!                        the SDK's interactions module, the CLI's interactions and biography commands,
//!                        and every framework test file IL-a and IL-b add
//! ```
//!
//! The schema's own words (permit, forbid, class, biograph, audience), which IL-a's scan also refused,
//! are allowed since IL-b (QIB-2): they are what the schema is. The packs that gain sections in IL-b
//! (`conversation`, `group-activity`) are not scanned; their sections are theirs to name.
//!
//! Every line is read, comments included. This file is not scanned, because it has to name the
//! vocabulary it looks for. Words are split as the I-2 scan splits them (`precursor_vocabulary.rs`,
//! `ARC-35` item 7) — at every character that is not an ASCII letter or digit, and at every
//! lower-to-upper case boundary — written again here so that neither scan is edited. A listed file
//! that is missing fails the test.

use std::path::{Path, PathBuf};

/// The seam scan's physics list (`seam_vocabulary.rs` `PHYSICS`), and the interaction words of
/// §12.5 IB-12: a word matches when, lowercased, it begins with one of these.
const VOCABULARY: [&str; 23] = [
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
    "talk",
    "spoke",
    "convers",
    "give",
    "buy",
    "sell",
    "trade",
    "eat",
    "drink",
    "kick",
    "throw",
    "shove",
    "invit",
];

/// The files IL-a and IL-b add that hold the seam and the schema, and every framework test file they
/// add but this one.
const SCANNED: &[&str] = &[
    "authoring/src/configuration.rs",
    "authoring/src/classes.rs",
    "authoring/src/attachment.rs",
    "sdk/rust/src/interactions",
    "sdk/rust/tests/interactions.rs",
    "tests/acceptance/tests/interaction_schema.rs",
    "tools/cli/src/interactions.rs",
    "tools/cli/src/biography.rs",
    "tools/cli/tests/interactions.rs",
    "worldpack/src/configure.rs",
    "worldpack/src/configure/tests.rs",
    "sdk/rust/tests/extensions.rs",
    "worldpack/tests/configuration.rs",
    "tools/cli/tests/configure.rs",
    "tests/acceptance/tests/configuration/mod.rs",
    "tests/acceptance/tests/configuration_seam.rs",
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

/// Words a scanned file must say that are not a pack's vocabulary: (file, a substring of the line, the
/// word admitted, why). An entry admits one word on the lines of one file that contain its substring,
/// and an entry that admits nothing fails the test, so the list cannot go stale.
const ADMITTED: [(&str, &str, &str, &str); 1] = [(
    "tools/cli/src/biography.rs",
    "format::decode(&manifest_row.body, \"manifest\")",
    "body",
    "persistence's ManifestRow field (the stored bytes), read since S8; not the physics word",
)];

/// Every listed word in `text`, as `path:line: word`, except those an entry of [`ADMITTED`] admits;
/// each admitting entry's index is marked in `used`.
fn listed_words(path: &Path, text: &str, used: &mut [bool]) -> Vec<String> {
    let mut found = Vec::new();
    for (number, line) in text.lines().enumerate() {
        for word in words(line) {
            if !VOCABULARY.iter().any(|listed| word.starts_with(listed)) {
                continue;
            }
            let admitted = ADMITTED.iter().position(|(file, substring, admits, _)| {
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

/// Every scanned file, a directory read whole (in a fixed order); a missing one fails.
fn scanned(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for file in SCANNED {
        let path = root.join(file);
        if path.is_dir() {
            let mut entries: Vec<PathBuf> = std::fs::read_dir(&path)
                .unwrap_or_else(|error| panic!("{file}: {error}"))
                .map(|entry| entry.expect("a directory entry").path())
                .collect();
            entries.sort();
            files.extend(entries);
        } else {
            assert!(path.is_file(), "a scanned file is missing: {file}");
            files.push(path);
        }
    }
    files
}

#[test]
fn the_configuration_seam_and_the_schema_name_no_packs_vocabulary() {
    let mut found = Vec::new();
    let mut used = [false; ADMITTED.len()];
    for path in scanned(&root()) {
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        found.extend(listed_words(&path, &text, &mut used));
    }
    assert!(
        found.is_empty(),
        "the configuration seam and the Interaction List's framework must name no pack's \
         vocabulary (ARC-61 … ARC-64):\n{}",
        found.join("\n")
    );
    for ((file, substring, word, reason), used) in ADMITTED.iter().zip(used) {
        assert!(!reason.is_empty(), "{file}: an admission needs a reason");
        assert!(
            used,
            "{file}: the admission of {word:?} on \"{substring}\" admits nothing; remove it"
        );
    }
}

/// The splitter sees the vocabulary inside identifiers and not inside other words.
#[test]
fn the_splitter_finds_the_vocabulary_inside_identifiers_and_not_inside_other_words() {
    let found = listed_words(
        Path::new("x.rs"),
        "let TalkTo = give_item; // Sellers trade; repeat, great, overthrow, uninvited",
        &mut [],
    );
    assert_eq!(
        found,
        [
            "x.rs:1: talk",
            "x.rs:1: give",
            "x.rs:1: sellers",
            "x.rs:1: trade",
        ]
    );
}
