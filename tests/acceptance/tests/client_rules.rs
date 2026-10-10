//! I-S14-1.1 and I-S14-6 — no rule in a reference client, held structurally (step-15 §10, §18.3 S-1).
//!
//! A client acquires and reports; the server decides (`ENGINEERING_RULES.md` §§4, 7–9;
//! `clients/protocol/ADOPTION.md` §3.3). Three things a client could do to break that are visible in
//! its source, and this test refuses each, naming the file and line:
//!
//! ```text
//! a  an action's name, written as a string, outside the files that build requests: every action type
//!    a System Pack declares (ActionTypeId::from_static("…") under systems/*/src, so a new pack is
//!    covered without editing this file) may appear as a GDScript string literal only where an entry
//!    of ACTION_LITERALS admits it
//! b  a copied rule: a const or var whose name holds REACH, RANGE, CLEARANCE, NUDGE, CAPACITY or
//!    MAX_STRIDE, unless an entry of RULE_NAMES admits it
//! c  a pack read directly: a string literal naming a path under worlds/ or systems/ (I-S14-6)
//! ```
//!
//! Every `*.gd` under `clients/` is read, except `clients/*/tools/` (offline asset and capture tools),
//! `clients/protocol/demo/` (the protocol's throwaway demonstration, step-15 §10) and
//! `clients/protocol/checks/` (the shared module's own checks, PR 16b, which exercise named
//! affordances against a live server as a test does, and build no client's requests). Symlinked
//! directories are not followed: `clients/3d-spike/mineworld` is the shared module, read once where it
//! lives.
//!
//! **Comments are not read** (step-15 D-16a-6). The vocabulary scans (`precursor_vocabulary.rs`,
//! `seam_vocabulary.rs`) hold the absence of words and read comments too; this one holds where requests
//! are built, and a word in a comment builds nothing — the shared module documents `submit("talk", …)`
//! in a comment, rightly. A small GDScript lexer separates code, strings and comments.
//!
//! An entry that admits nothing fails, so neither list can go stale; a root that is missing, or a build
//! in which no action type is found, fails rather than passing an empty scan.

use std::path::{Path, PathBuf};

// The GDScript lexer, shared with `client_text.rs` (moved unchanged by step-20 SD-SET-a-13).
#[path = "support/gdscript.rs"]
mod gdscript;
use gdscript::lex;

/// The GDScript string literals that name an action: (file, literal, why it may be written there).
const ACTION_LITERALS: [(&str, &str, &str); 12] = [
    (
        "clients/2d/scripts/intents.gd",
        "move",
        "intents.gd builds every request of the 2D client and is its only submitter (step-13 ARC-47 R1, \
         R2); move is the one action it composes in PR 13a",
    ),
    (
        "clients/2d/scripts/intents.gd",
        "talk",
        "composed by the 2D client's only submitter from the typed utterance (step-13 §15.4 D-b-5, PR 13b)",
    ),
    (
        "clients/2d/scripts/intents.gd",
        "invite",
        "composed by the 2D client's only submitter from the typed activity kind (step-13 §15.4 D-b-5, \
         PR 13b)",
    ),
    (
        "clients/2d/scripts/intents.gd",
        "accept-invitation",
        "composed with an empty payload by the 2D client's only submitter (step-13 §15.4 D-b-5, PR 13b)",
    ),
    (
        "clients/2d/scripts/intents.gd",
        "decline-invitation",
        "composed with an empty payload by the 2D client's only submitter (step-13 §15.4 D-b-5, PR 13b)",
    ),
    (
        "clients/2d/scripts/intents.gd",
        "join-group-activity",
        "composed with an empty payload by the 2D client's only submitter (step-13 §15.4 D-b-5, PR 13b)",
    ),
    (
        "clients/2d/scripts/intents.gd",
        "leave-group-activity",
        "composed with an empty payload by the 2D client's only submitter (step-13 §15.4 D-b-5, PR 13b)",
    ),
    (
        "clients/2d/scripts/app.gd",
        "invite",
        "the --invite command-line option, the join credential handed to MineWorldLink (PROTOCOL.md \
         §4.1; S11-A); it shares its spelling with group-activity's action type, and no request is built \
         from it",
    ),
    (
        "clients/3d-spike/scripts/slice/slice_link.gd",
        "move",
        "the slice's connection reports the body's strides (PROTOCOL.md §6.2); the one action it builds",
    ),
    (
        "clients/3d-spike/scripts/slice/intents.gd",
        "talk",
        "intents.gd builds every interaction request of the 3D client (step-15 §4.4)",
    ),
    (
        "clients/3d-spike/scripts/human.gd",
        "move",
        "the name of an AnimationNodeBlendTree input of the character's locomotion graph, not a request \
         (step-15 Q-16a-5)",
    ),
    (
        "clients/protocol/mineworld/world_client.gd",
        "invite",
        "the join frame's credential field (PROTOCOL.md §§2, 4.1; S11-A), a wire key that shares its spelling \
         with group-activity's action type; no request is built from it",
    ),
];

/// Rule-named declarations admitted with a reason: (file, name, why). None in PR 16a (step-15 A16-10).
const RULE_NAMES: [(&str, &str, &str); 0] = [];

/// The words that name a rule a client might copy. `MAX_STRIDE` is matched as the pair.
const RULE_WORDS: [&str; 5] = ["reach", "range", "clearance", "nudge", "capacity"];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A path relative to the repository root, with `/` separators.
fn relative(path: &Path) -> String {
    let root = root().canonicalize().expect("the repository root resolves");
    let path = path.canonicalize().expect("a scanned file resolves");
    path.strip_prefix(&root)
        .expect("a scanned file is inside the repository")
        .to_string_lossy()
        .replace('\\', "/")
}

/// An identifier's words, lowercased: split at `_`, at digits, and at lower-to-upper case changes.
fn name_words(name: &str) -> Vec<String> {
    let mut words = Vec::new();
    for run in name.split(|c: char| !c.is_ascii_alphabetic()) {
        let mut word = String::new();
        let mut after_lower = false;
        for c in run.chars() {
            if c.is_ascii_uppercase() && after_lower {
                words.push(std::mem::take(&mut word));
            }
            after_lower = c.is_ascii_lowercase();
            word.push(c.to_ascii_lowercase());
        }
        if !word.is_empty() {
            words.push(word);
        }
    }
    words
}

/// Whether a declared name names a rule.
fn names_a_rule(name: &str) -> bool {
    let words = name_words(name);
    words.iter().any(|word| RULE_WORDS.contains(&word.as_str()))
        || words
            .windows(2)
            .any(|pair| pair[0] == "max" && pair[1] == "stride")
}

/// The name a line of code declares with `const` or `var`, if it declares one.
fn declared(code: &str) -> Option<&str> {
    let mut tokens = code.split_whitespace();
    while let Some(token) = tokens.next() {
        if token == "const" || token == "var" {
            let name = tokens.next()?;
            let end = name
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(name.len());
            return Some(&name[..end]);
        }
    }
    None
}

/// Whether a string literal names a path under `worlds/` or `systems/`.
fn names_a_pack_path(literal: &str) -> bool {
    let components: Vec<&str> = literal.split('/').collect();
    components.len() > 1
        && components[..components.len() - 1]
            .iter()
            .any(|component| *component == "worlds" || *component == "systems")
}

/// Every action type the build's System Packs declare, from their sources.
fn action_types(systems: &Path) -> Vec<String> {
    const MARK: &str = "ActionTypeId::from_static(\"";
    let mut files = Vec::new();
    let packs = std::fs::read_dir(systems)
        .unwrap_or_else(|error| panic!("the System Packs' root is missing: {systems:?}: {error}"));
    for pack in packs {
        let source = pack.expect("a directory entry").path().join("src");
        if source.is_dir() {
            rust_files(&source, &mut files);
        }
    }
    let mut types = Vec::new();
    for file in files {
        let text = std::fs::read_to_string(&file).expect("a source file reads");
        for (at, _) in text.match_indices(MARK) {
            let rest = &text[at + MARK.len()..];
            let name = &rest[..rest.find('"').expect("a closed action type literal")];
            if !types.iter().any(|known| known == name) {
                types.push(name.to_owned());
            }
        }
    }
    types.sort();
    types
}

fn rust_files(directory: &Path, into: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).expect("a source directory reads") {
        let path = entry.expect("a directory entry").path();
        if path.is_dir() {
            rust_files(&path, into);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            into.push(path);
        }
    }
}

/// Every client script the scan reads.
fn client_scripts(directory: &Path, into: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("a scanned directory is missing: {directory:?}: {error}"));
    for entry in entries {
        let entry = entry.expect("a directory entry");
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let kind = entry.file_type().expect("a file type");
        if kind.is_symlink() || name.starts_with('.') {
            continue;
        }
        if kind.is_dir() {
            let excluded = name == "tools"
                || path.ends_with("clients/protocol/demo")
                || path.ends_with("clients/protocol/checks");
            if !excluded {
                client_scripts(&path, into);
            }
        } else if name.ends_with(".gd") {
            into.push(path);
        }
    }
}

/// What the scan refuses in one file. `used_literals` and `used_names` mark the admitting entries.
fn findings(
    file: &str,
    text: &str,
    types: &[String],
    used_literals: &mut [bool],
    used_names: &mut [bool],
) -> Vec<String> {
    let lexed = lex(text);
    let mut found = Vec::new();
    for (line, literal) in &lexed.literals {
        if types.iter().any(|known| known == literal) {
            match ACTION_LITERALS
                .iter()
                .position(|(admitted_file, admitted, _)| {
                    *admitted_file == file && admitted == literal
                }) {
                Some(entry) => used_literals[entry] = true,
                None => found.push(format!(
                    "{file}:{line}: the action \"{literal}\" is named outside the files that build \
                     requests"
                )),
            }
        }
        if names_a_pack_path(literal) {
            found.push(format!(
                "{file}:{line}: \"{literal}\" names a pack's file; a client learns the world from frames \
                 (I-S14-6)"
            ));
        }
    }
    for (index, code) in lexed.code.iter().enumerate() {
        let Some(name) = declared(code) else { continue };
        if !names_a_rule(name) {
            continue;
        }
        match RULE_NAMES
            .iter()
            .position(|(admitted_file, admitted, _)| *admitted_file == file && *admitted == name)
        {
            Some(entry) => used_names[entry] = true,
            None => found.push(format!(
                "{file}:{}: `{name}` names a rule; the server decides it (I-S14-1)",
                index + 1
            )),
        }
    }
    found
}

#[test]
fn no_client_script_names_an_action_copies_a_rule_or_reads_a_pack() {
    let root = root();
    let types = action_types(&root.join("systems"));
    assert!(
        types.iter().any(|known| known == "move") && types.iter().any(|known| known == "talk"),
        "the build's action types were not found (has the declaration changed?): {types:?}"
    );

    let mut files = Vec::new();
    client_scripts(&root.join("clients"), &mut files);
    // The 3D client's 33 scripts after PR 16a (31 before it, plus targeting.gd and intents.gd) and
    // the shared module's 3: a walk that silently skipped a directory would read fewer.
    assert!(
        files.len() >= 36,
        "every client script is scanned: {}",
        files.len()
    );

    let mut found = Vec::new();
    let mut used_literals = [false; ACTION_LITERALS.len()];
    let mut used_names = [false; RULE_NAMES.len()];
    for path in &files {
        let text = std::fs::read_to_string(path).expect("a client script reads");
        found.extend(findings(
            &relative(path),
            &text,
            &types,
            &mut used_literals,
            &mut used_names,
        ));
    }
    assert!(
        found.is_empty(),
        "a reference client reports intent and never decides (step-15 I-S14-1, I-S14-6):\n{}",
        found.join("\n")
    );
    for ((file, literal, reason), used) in ACTION_LITERALS.iter().zip(used_literals) {
        assert!(!reason.is_empty(), "{file}: an admission needs a reason");
        assert!(
            used,
            "{file}: the admission of \"{literal}\" admits nothing; remove it"
        );
    }
    for ((file, name, reason), used) in RULE_NAMES.iter().zip(used_names) {
        assert!(!reason.is_empty(), "{file}: an admission needs a reason");
        assert!(
            used,
            "{file}: the admission of `{name}` admits nothing; remove it"
        );
    }
}

/// The lexer: a comment is not read, a `#` inside a string does not start one, escapes and triple
/// quotes are strings, and line numbers survive multi-line strings.
#[test]
fn the_lexer_reads_strings_and_not_comments() {
    let lexed = lex("var a := \"talk\"  # \"shove\" in a comment\n\
         var b := \"# not a comment\" + \"move\"\n\
         var c := &\"give\" + 'it\\'s'\n\
         var d := \"\"\"two\nlines\"\"\" + \"buy\"\n");
    assert_eq!(
        lexed.literals,
        [
            (1, "talk".to_owned()),
            (2, "# not a comment".to_owned()),
            (2, "move".to_owned()),
            (3, "give".to_owned()),
            (3, "it's".to_owned()),
            (4, "two\nlines".to_owned()),
            (5, "buy".to_owned()),
        ]
    );
    assert_eq!(lexed.code[0].trim_end(), "var a := \"\"");
}

/// Rule names are found inside identifiers and not inside other words; declarations are read past
/// annotations; pack paths are told from other paths.
#[test]
fn rule_names_and_pack_paths_are_recognised() {
    for name in [
        "TALK_REACH",
        "talkReach",
        "RANGE_M",
        "MAX_STRIDE",
        "nudge_mm",
        "SEAT_CAPACITY",
    ] {
        assert!(names_a_rule(name), "{name} names a rule");
    }
    for name in [
        "ORANGE",
        "arrange",
        "REPORT_DIST",
        "STRIDE",
        "MAX_SPEED",
        "CAPSULE_RADIUS",
    ] {
        assert!(!names_a_rule(name), "{name} names no rule");
    }
    assert_eq!(
        declared("@export var reach_m: float = 0.8"),
        Some("reach_m")
    );
    assert_eq!(declared("static func _range(a: int) -> int:"), None);
    assert!(names_a_pack_path(
        "res://../../worlds/social-cafe/people/alice.yaml"
    ));
    assert!(names_a_pack_path("systems/bodies/src/lib.rs"));
    assert!(!names_a_pack_path("res://scripts/slice/intents.gd"));
    assert!(!names_a_pack_path("worlds"));
}
