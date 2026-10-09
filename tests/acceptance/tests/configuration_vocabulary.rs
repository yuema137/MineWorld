//! IA-8 — the configuration seam names no physics and no interaction (step-18-interaction-list §11.4;
//! `docs/DECISIONS.md` `ARC-61`, `ARC-62`).
//!
//! PR IL-a is a framework precursor of the World Interaction List: the seam exists before any pack
//! configures anything, and must not be shaped by the first one in name. The absence is held
//! structurally:
//!
//! ```text
//! the seam's code   no word beginning with a word of the seam scan's physics list
//!                   (`seam_vocabulary.rs`), nor talk, spoke, convers, give, buy, sell, trade, eat,
//!                   drink, kick, throw, shove, permit, forbid, biograph or class — in authoring's
//!                   configuration contract, worldpack's configure module and its in-crate tests, and
//!                   every test file PR IL-a adds
//! ```
//!
//! Every line is read, comments included. This file is not scanned, because it has to name the
//! vocabulary it looks for. Words are split as the I-2 scan splits them (`precursor_vocabulary.rs`,
//! `ARC-35` item 7) — at every character that is not an ASCII letter or digit, and at every
//! lower-to-upper case boundary — written again here so that neither scan is edited. A listed file
//! that is missing fails the test.

use std::path::{Path, PathBuf};

/// The seam scan's physics list (`seam_vocabulary.rs` `PHYSICS`), and the interaction words of
/// §11.4 IA-8: a word matches when, lowercased, it begins with one of these.
const VOCABULARY: [&str; 26] = [
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
    "permit",
    "forbid",
    "biograph",
    "class",
];

/// The one word of the list the seam itself must say: `classes` is a reserved `configure:` key
/// (SD-IA-6, QIA-1), refused by name until IL-b gives it its meaning. (file, a substring of the line,
/// the word admitted, why). An entry admits one word on the lines of one file that contain its
/// substring, and an entry that admits nothing fails the test, so the list cannot go stale.
const RESERVED_KEY_LINES: [(&str, &str, &str, &str); 3] = [
    (
        "worldpack/src/configure.rs",
        "(\"classes\", \"the World's Interaction List's entity classes\")",
        "classes",
        "SD-IA-6: the reserved key itself, in RESERVED, so a world cannot give it a meaning first",
    ),
    (
        "worldpack/tests/configuration.rs",
        "(\"classes\", \"entity classes\")",
        "classes",
        "IA-3: the test that the reserved key is refused, naming what it is reserved for",
    ),
    (
        "tools/cli/tests/configure.rs",
        "\"configure:\\n  - classes\\n\"",
        "classes",
        "IA-3: the real `mineworld validate` refuses the reserved key",
    ),
];

/// The files PR IL-a adds that hold the seam, and every test file it adds but this one.
const SCANNED: [&str; 8] = [
    "authoring/src/configuration.rs",
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

/// Every listed word in `text`, as `path:line: word`, except those an entry of
/// [`RESERVED_KEY_LINES`] admits; each admitting entry's index is marked in `used`.
fn listed_words(path: &Path, text: &str, used: &mut [bool]) -> Vec<String> {
    let mut found = Vec::new();
    for (number, line) in text.lines().enumerate() {
        for word in words(line) {
            if !VOCABULARY.iter().any(|listed| word.starts_with(listed)) {
                continue;
            }
            let admitted = RESERVED_KEY_LINES
                .iter()
                .position(|(file, substring, admits, _)| {
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
fn the_configuration_seam_names_no_physics_and_no_interaction() {
    let root = root();
    let mut found = Vec::new();
    let mut used = [false; RESERVED_KEY_LINES.len()];
    for file in SCANNED {
        let path = root.join(file);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("a scanned file is missing: {file}: {error}"));
        found.extend(listed_words(&path, &text, &mut used));
    }
    assert!(
        found.is_empty(),
        "the configuration seam must name no physics and no interaction (ARC-61, ARC-62):\n{}",
        found.join("\n")
    );
    for ((file, substring, word, reason), used) in RESERVED_KEY_LINES.iter().zip(used) {
        assert!(!reason.is_empty(), "{file}: an exemption needs a reason");
        assert!(
            used,
            "{file}: the exemption for {word:?} on \"{substring}\" admits nothing; remove it"
        );
    }
}

/// The splitter sees the vocabulary inside identifiers and not inside other words.
#[test]
fn the_splitter_finds_the_vocabulary_inside_identifiers_and_not_inside_other_words() {
    let found = listed_words(
        Path::new("x.rs"),
        "let TalkTo = give_item; // Sellers trade; repeat, great, overthrow, unforbidden",
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
