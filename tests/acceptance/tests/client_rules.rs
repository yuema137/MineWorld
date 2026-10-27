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
//! e  a moved prop (ARC-171, S33-e X-7): a Tween, an AnimationPlayer or a per-frame callback in a
//!    script that draws a depiction (`clients/3d-spike/scripts/depictions/`)
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

// The 3D client's directory, named once (16e D-e-3). Without the client (ARC-347's removal check), the
// scans skip its scripts and its admissions and say so.
#[macro_use]
#[path = "support/client_3d.rs"]
mod client_3d;

/// The GDScript string literals that name an action: (file, literal, why it may be written there).
const ACTION_LITERALS: [(&str, &str, &str); 30] = [
    (
        "clients/2d/scripts/intents.gd",
        "walk-to",
        "composed by the 2D client's only submitter: a click on the floor asks the server to plan the \
         walk (walk-to, ARC-75); the server routes and the client paces, as every client does",
    ),
    (
        "clients/2d/scripts/intents.gd",
        "walk-step",
        "composed by the 2D client's only submitter: a click on the floor asks the server to plan the \
         walk (walk-step, ARC-75); the server routes and the client paces, as every client does",
    ),
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
        "clients/2d/scripts/intents.gd",
        "sleep",
        "composed by the 2D client's only submitter from the typed minutes (S35 N-e §4.3): sleep { for }",
    ),
    (
        "clients/2d/scripts/intents.gd",
        "list-for-sale",
        "composed by the client's only submitter from the player's item and price (S31-e §5.6)",
    ),
    (
        "clients/2d/scripts/intents.gd",
        "set-price",
        "composed by the client's only submitter from the player's item and price (S31-e §5.6)",
    ),
    (
        "clients/2d/scripts/intents.gd",
        "withdraw",
        "composed by the client's only submitter from the player's item and price (S31-e §5.6)",
    ),
    (
        "clients/2d/scripts/intents.gd",
        "throw",
        "composed by the 2D client's only submitter as the offered complete throw with `toward` set to \
         the clicked floor point (S12 PR 13d, ARC-342); the server decides the throw",
    ),
    (
        "clients/2d/scripts/harness/interact.gd",
        "sleep",
        "the scripted check's step that waits a number of seconds (step-13 §15.3); not an action",
    ),
    (
        "clients/2d/scripts/app.gd",
        "invite",
        "the --invite command-line option, the join credential handed to MineWorldLink (PROTOCOL.md \
         §4.1; S11-A); it shares its spelling with group-activity's action type, and no request is built \
         from it",
    ),
    (
        client_3d!("scripts/slice/slice_link.gd"),
        "move",
        "the slice's connection reports the body's strides (PROTOCOL.md §6.2); the one action it builds",
    ),
    (
        client_3d!("scripts/slice/intents.gd"),
        "walk-to",
        "the 3D client's one request builder asks the server to plan a walk (ARC-75), as the 2D client's \
         intents.gd does; a probe walks through it, never building the request itself",
    ),
    (
        client_3d!("scripts/slice/intents.gd"),
        "talk",
        "intents.gd builds every interaction request of the 3D client (step-15 §4.4)",
    ),
    (
        client_3d!("scripts/slice/intents.gd"),
        "kick",
        "built by the 3D client's one request builder when F is pressed at an object, or submitted as the \
         world's matching complete offer, unchanged (step-11 §22 SD-E6, S15 PR 12e)",
    ),
    (
        client_3d!("scripts/slice/intents.gd"),
        "throw",
        "built by the 3D client's one request builder from G, G or G, aim, G, or submitted as the world's \
         matching complete offer, unchanged (step-11 §22 SD-E6, S15 PR 12e)",
    ),
    (
        client_3d!("scripts/slice/intents.gd"),
        "shove",
        "built by the 3D client's one request builder when R is pressed at a person, or submitted as the \
         world's matching complete offer, unchanged (step-11 §22 SD-E6, S15 PR 12e)",
    ),
    (
        client_3d!("scripts/slice/intents.gd"),
        "list-for-sale",
        "composed by the client's only submitter from the player's item and price (S31-e §5.6)",
    ),
    (
        client_3d!("scripts/slice/intents.gd"),
        "set-price",
        "composed by the client's only submitter from the player's item and price (S31-e §5.6)",
    ),
    (
        client_3d!("scripts/slice/intents.gd"),
        "withdraw",
        "composed by the client's only submitter from the player's item and price (S31-e §5.6)",
    ),
    (
        client_3d!("scripts/human.gd"),
        "move",
        "the name of an AnimationNodeBlendTree input of the character's locomotion graph, not a request \
         (step-15 Q-16a-5)",
    ),
    (
        client_3d!("scripts/human.gd"),
        "sit",
        "the name of the seated input of the character's AnimationNodeTransition, as `move` is of its \
         locomotion graph (step-15 Q-16a-5); it shares its spelling with posture's action type, and the \
         pose follows the server's posture state, never builds a request (S34-a)",
    ),
    (
        client_3d!("scripts/slice/slice_probe_market.gd"),
        "sleep",
        "a probe script's step key, a pause of that many seconds (16d C5); it shares its spelling with \
         sleep's action type (S35 N-a) and builds no request",
    ),
    (
        client_3d!("scripts/shots.gd"),
        "stand",
        "a capture phase's name in the offline body checks (standing still versus walking), written into \
         the shot's file name; it shares its spelling with posture's action type and builds no request",
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

    let three_d = client_3d::present(
        &root,
        "the 3D client's scripts and their admissions (I-S14-1, I-S14-6)",
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
            used || (!three_d && client_3d::holds(file)),
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

/// String literals in the 3D client that equal a place key and are not used as one, each reviewed:
/// (file, literal, why). An entry for a literal compared with a place's key is forbidden (S33-c §5.6).
const PLACE_KEY_LITERALS: [(&str, &str, &str); 10] = [
    (
        "clients/3d-spike/scripts/depictions/data/depiction_file.gd",
        "street",
        "the depiction file's section name (MODULE_SPEC.md §6.1.2: `street`, the binding of the street \
         every place opens onto), a field of the Presentation Pack's format, never compared with a place",
    ),
    (
        "clients/shared/checks/station_state_check.gd",
        "station",
        "the `station` role tag in the headless check's literal place tag lists and frames (S29-f §5.1, \
         ARC-108), a tag the fixture rule receives as data, never a place key the client binds",
    ),
    (
        "clients/shared/environment/far_field.gd",
        "station",
        "the far-field file's skyline slot name (MODULE_SPEC.md §6.1.1: slots station and landmark), a \
         field of the Presentation Pack's format, never compared with a place",
    ),
    (
        client_3d!("scripts/slice/far_field_view.gd"),
        "station",
        "the far-field skyline slot name, read from the loaded file (MODULE_SPEC.md §6.1.1)",
    ),
    (
        client_3d!("scripts/town.gd"),
        "cafe",
        "the promenade spike's own interior id in its building table, an offline scene that never \
         connects to a world (CLAUDE.md, mineworld-3d); not a place of any world",
    ),
    (
        client_3d!("scripts/slice/slice_world.gd"),
        "street",
        "a build-timing stage label printed by SliceWorld._tick",
    ),
    (
        client_3d!("scripts/slice/slice_world.gd"),
        "cafe",
        "a build-timing stage label printed by SliceWorld._tick",
    ),
    (
        client_3d!("scripts/slice/slice_probe.gd"),
        "street",
        "the name of a standalone review view (--measure's viewpoints), a file name, not a place",
    ),
    (
        client_3d!("scripts/slice/slice_probe_town.gd"),
        "street",
        "the --slice-street probe mode's name and its verdict label (16c), never compared with a place",
    ),
    (
        client_3d!("scripts/slice/slice_layout.gd"),
        "street",
        "SHOWN's drawn_as vocabulary (D-16c-7: room, door, street), how a place is drawn, not which",
    ),
];

/// Every place key of every World Pack in the checkout: the stems of `worlds/*/places/*.yaml`.
fn place_keys(worlds: &Path) -> Vec<String> {
    let mut keys = Vec::new();
    let packs = std::fs::read_dir(worlds)
        .unwrap_or_else(|error| panic!("the World Packs' root is missing: {worlds:?}: {error}"));
    for pack in packs {
        let places = pack.expect("a directory entry").path().join("places");
        let Ok(entries) = std::fs::read_dir(&places) else {
            continue;
        };
        for entry in entries {
            let path = entry.expect("a directory entry").path();
            if path
                .extension()
                .is_some_and(|extension| extension == "yaml")
            {
                let stem = path
                    .file_stem()
                    .expect("a stem")
                    .to_string_lossy()
                    .into_owned();
                if !keys.contains(&stem) {
                    keys.push(stem);
                }
            }
        }
    }
    keys.sort();
    keys
}

/// Check (d): the place-key literals of one file, except a `tag("…")` argument and the admitted ones.
/// A literal is matched to its blanked `""` in the lexed code by its order on the line it starts on.
fn place_key_findings(file: &str, text: &str, keys: &[String], used: &mut [bool]) -> Vec<String> {
    let lexed = lex(text);
    let mut found = Vec::new();
    let mut seen_on_line = std::collections::BTreeMap::<usize, usize>::new();
    for (line, literal) in &lexed.literals {
        let nth = {
            let count = seen_on_line.entry(*line).or_insert(0);
            *count += 1;
            *count - 1
        };
        if !keys.iter().any(|key| key == literal) {
            continue;
        }
        let code = lexed.code.get(line - 1).map(String::as_str).unwrap_or("");
        let in_tag = code
            .match_indices("\"\"")
            .nth(nth)
            .is_some_and(|(at, _)| code[..at].trim_end().ends_with("tag("));
        if in_tag {
            continue;
        }
        match PLACE_KEY_LITERALS
            .iter()
            .position(|(admitted_file, admitted, _)| *admitted_file == file && admitted == literal)
        {
            Some(entry) => used[entry] = true,
            None => found.push(format!(
                "{file}:{line}: \"{literal}\" is a place key; the 3D client keeps no list of a world's \
                 place keys (SC-4, I-S33-3): bind by tag with `SliceDepiction.tag()`"
            )),
        }
    }
    found
}

/// X-6 (S33-c §5.6): the 3D client and the shared client modules name no place key. They bind a
/// world's places by disclosed tags and doorway coincidence (`DECISIONS.md` ARC-144); a tag is written
/// only as `SliceDepiction.tag("…")`.
#[test]
fn the_3d_client_names_no_place_key() {
    let root = root();
    let keys = place_keys(&root.join("worlds"));
    assert!(
        keys.iter().any(|key| key == "cafe") && keys.iter().any(|key| key == "lane"),
        "the World Packs' place keys were not found: {keys:?}"
    );
    let three_d = client_3d::present(
        &root,
        "the 3D client's place-key scan and its admissions (X-6)",
    );
    let mut files = Vec::new();
    if three_d {
        client_scripts(&root.join(client_3d!("scripts")), &mut files);
    }
    client_scripts(&root.join("clients/shared"), &mut files);
    // With the 3D client, its scripts and the shared modules; without it, the shared modules alone.
    assert!(
        files.len() >= if three_d { 40 } else { 1 },
        "every 3D client script is scanned: {}",
        files.len()
    );
    let mut found = Vec::new();
    let mut used = [false; PLACE_KEY_LITERALS.len()];
    for path in &files {
        let text = std::fs::read_to_string(path).expect("a client script reads");
        found.extend(place_key_findings(&relative(path), &text, &keys, &mut used));
    }
    assert!(
        found.is_empty(),
        "a place key in the 3D client:\n{}",
        found.join("\n")
    );
    for ((file, literal, reason), used) in PLACE_KEY_LITERALS.iter().zip(used) {
        assert!(!reason.is_empty(), "{file}: an admission needs a reason");
        assert!(
            used || (!three_d && client_3d::holds(file)),
            "{file}: the admission of \"{literal}\" admits nothing; remove it"
        );
    }
}

// --- check e: no client compares money or reads a reference value (S31-e §5.11, X-8; step-31 §12) ----------

/// Functions admitted to hold a comparison beside a wallet word and a price word: (file, function, why).
/// None is needed (S31-e AC-e-10); an admission that admits nothing fails.
const MONEY_ADMITTED: [(&str, &str, &str); 0] = [];

/// The words that name what a client holds, and what something costs. Matched as an identifier's words
/// (`name_words`) or a string literal's.
const WALLET_WORDS: [&str; 3] = ["balance", "wallet", "wealth"];
const PRICE_WORDS: [&str; 4] = ["price", "amount", "ask", "cost"];

/// How many comparison tokens (`<`, `>`, `<=`, `>=`) a line of code (strings blanked, comments removed)
/// holds: the return arrow `->` and the shifts `<<`, `>>` are not comparisons.
fn comparisons(code: &str) -> usize {
    let chars: Vec<char> = code.chars().collect();
    let mut count = 0;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '<' || c == '>' {
            let next = chars.get(i + 1).copied();
            let previous = if i > 0 { Some(chars[i - 1]) } else { None };
            if next == Some(c) {
                i += 2;
                continue;
            }
            if !(c == '>' && previous == Some('-')) {
                count += 1;
            }
            i += if next == Some('=') { 2 } else { 1 };
            continue;
        }
        i += 1;
    }
    count
}

/// Whether a text's words (an identifier's, or a literal's split the same way) hold one of `words`.
fn says_any(text: &str, words: &[&str]) -> bool {
    name_words(text)
        .iter()
        .any(|word| words.contains(&word.as_str()))
}

/// The top-level functions of a lexed file: (name, first line index, one past the last).
fn functions(code: &[String]) -> Vec<(String, usize, usize)> {
    let mut found: Vec<(String, usize, usize)> = Vec::new();
    for (index, line) in code.iter().enumerate() {
        let head = line
            .strip_prefix("static func ")
            .or_else(|| line.strip_prefix("func "));
        let top = !line.starts_with(|c: char| c.is_whitespace()) && !line.trim().is_empty();
        if top
            && let Some(last) = found.last_mut()
            && last.2 == usize::MAX
        {
            last.2 = index;
        }
        if let Some(rest) = head {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            found.push((name, index, usize::MAX));
        }
    }
    if let Some(last) = found.last_mut()
        && last.2 == usize::MAX
    {
        last.2 = code.len();
    }
    found
}

/// Check e over one file: e1, a function that compares beside a wallet word and a price word; e2, a
/// reference value named as a string.
fn money_findings(file: &str, text: &str, used: &mut [bool]) -> Vec<String> {
    let lexed = lex(text);
    let mut found = Vec::new();
    // The currency record's field and economy's component (`ARC-113`). A bare "reference" is not matched:
    // the 3D client's cast names a reference body by it, which is no value (S31-e D-e-6).
    for (line, literal) in &lexed.literals {
        if literal == "references" || literal == "reference-value" {
            found.push(format!(
                "{file}:{line}: reads a reference value: never charged, never computed with in a client \
                 (step-31 §6.1, I-S31-6)"
            ));
        }
    }
    for (name, from, to) in functions(&lexed.code) {
        let body = &lexed.code[from..to];
        let compares = body.iter().any(|line| comparisons(line) > 0);
        if !compares {
            continue;
        }
        let in_body = |words: &[&str]| {
            body.iter().any(|line| says_any(line, words))
                || lexed
                    .literals
                    .iter()
                    .any(|(line, literal)| *line > from && *line <= to && says_any(literal, words))
        };
        if !(in_body(&WALLET_WORDS) && in_body(&PRICE_WORDS)) {
            continue;
        }
        match MONEY_ADMITTED
            .iter()
            .position(|(admitted_file, admitted, _)| *admitted_file == file && *admitted == name)
        {
            Some(entry) => used[entry] = true,
            None => found.push(format!(
                "{file}:{}: `{name}` compares money: a client decides nothing about prices (step-31 §12, \
                 X-8)",
                from + 1
            )),
        }
    }
    found
}

/// The shared sit decision (S33-f I-f-6; `ARC-145`): the one file that holds it.
const SIT_RULE: &str = "clients/shared/sitting/sit_rule.gd";
/// The rule's own constants (step-33 SD-S33-7): the radius and the wait. Its other constants (a folder
/// name, a rounding) are not the rule.
const SIT_RULE_CONSTANTS: [&str; 2] = ["SIT_NEAR_M", "SIT_STILL_S"];

/// I-f-6 (S33-f, step-34 §8.1): neither client holds a copy of the sit rule. Every constant the shared
/// sit decision declares is declared nowhere else in the clients, so a client that decides sitting by
/// itself — its own radius or its own wait — is a finding.
#[test]
fn no_client_holds_a_copy_of_the_sit_rule() {
    let root = root();
    let rule = std::fs::read_to_string(root.join(SIT_RULE)).expect("the shared sit rule reads");
    let names: Vec<String> = lex(&rule)
        .code
        .iter()
        .filter(|code| code.trim_start().starts_with("const "))
        .filter_map(|code| declared(code).map(str::to_owned))
        .filter(|name| SIT_RULE_CONSTANTS.contains(&name.as_str()))
        .collect();
    assert!(
        names.len() == SIT_RULE_CONSTANTS.len(),
        "the shared sit rule's constants were not found (has it moved?): {names:?}"
    );
    let mut files = Vec::new();
    client_scripts(&root.join("clients"), &mut files);
    let mut found = Vec::new();
    for path in &files {
        let file = relative(path);
        if file == SIT_RULE {
            continue;
        }
        let text = std::fs::read_to_string(path).expect("a client script reads");
        for (index, code) in lex(&text).code.iter().enumerate() {
            if let Some(name) = declared(code)
                && names.iter().any(|n| n == name)
            {
                found.push(format!(
                    "{file}:{}: `{name}` copies the shared sit rule ({SIT_RULE})",
                    index + 1
                ));
            }
        }
    }
    assert!(found.is_empty(), "I-f-6:\n{}", found.join("\n"));
}

/// The one 3D client script that may name a Presentation Pack's path: the class that resolves the pack
/// it is given (`docs/decisions/ARC-51.md`). Every other reader asks it.
const PACK_RESOLVER: &str = "clients/3d-spike/scripts/pack/pack.gd";

/// Whether a string literal names a path under `presentation/`.
fn names_a_presentation_path(literal: &str) -> bool {
    let components: Vec<&str> = literal.split('/').collect();
    components.len() > 1 && components[..components.len() - 1].contains(&"presentation")
}

/// What the pack scan refuses in one 3D client script: a literal naming a pack path, or a `PACK_DIR`.
fn pack_path_findings(file: &str, text: &str) -> Vec<String> {
    let lexed = lex(text);
    let mut found = Vec::new();
    for (line, literal) in &lexed.literals {
        if names_a_presentation_path(literal) {
            found.push(format!(
                "{file}:{line}: \"{literal}\" names a Presentation Pack's path; ask SlicePack (ARC-51)"
            ));
        }
    }
    for (index, code) in lexed.code.iter().enumerate() {
        if code.contains("PACK_DIR") {
            found.push(format!(
                "{file}:{}: `PACK_DIR`: the pack is the one the client is given; ask SlicePack (ARC-51)",
                index + 1
            ));
        }
    }
    found
}

/// X-8 (S31-e §5.11): no client script — 2D, 3D or a shared module — compares money or reads a reference
/// value. The server says whether a price is affordable, current or in range.
#[test]
fn no_client_compares_money_or_reads_a_reference_value() {
    let mut files = Vec::new();
    client_scripts(&root().join("clients"), &mut files);
    assert!(
        files
            .iter()
            .any(|f| f.ends_with("clients/shared/trade/money.gd")),
        "the shared trade module is scanned"
    );
    let mut found = Vec::new();
    let mut used = [false; MONEY_ADMITTED.len()];
    for path in &files {
        let text = std::fs::read_to_string(path).expect("a client script reads");
        found.extend(money_findings(&relative(path), &text, &mut used));
    }
    assert!(
        found.is_empty(),
        "a reference client decides nothing about money (S31-e check e):\n{}",
        found.join("\n")
    );
    for ((file, name, reason), used) in MONEY_ADMITTED.iter().zip(used) {
        assert!(!reason.is_empty(), "{file}: an admission needs a reason");
        assert!(
            used,
            "{file}: the admission of `{name}` admits nothing; remove it"
        );
    }
}

/// Check e finds X-8's plant and a planted reference read, and not a return arrow, a shift or a formatter
/// that compares a digit counter with no wallet word.
#[test]
fn check_e_finds_a_money_comparison_and_nothing_else() {
    let mut used = [false; MONEY_ADMITTED.len()];
    let plant = "func _rows(obs) -> Array:\n\
         \tvar rows := []\n\
         \tfor row in rows:\n\
         \t\tif int(row[\"amount\"]) > int(MineWorldTrade.money_of(obs).get(\"balance\", 0)):\n\
         \t\t\trow[\"buttons\"] = []\n\
         \treturn rows\n";
    let found = money_findings("x.gd", plant, &mut used);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].starts_with("x.gd:1: `_rows` compares money"),
        "{found:?}"
    );
    let reference =
        "func typical(obs) -> int:\n\treturn obs.own_component(\"currency\").get(\"references\")\n";
    let found = money_findings("x.gd", reference, &mut used);
    assert!(
        found.len() == 1 && found[0].contains("reference value"),
        "{found:?}"
    );
    let clean = "func f() -> int:\n\treturn 1 << 3\n\
         static func format(amount: int, wallet: Dictionary) -> String:\n\treturn str(amount)\n\
         func digits(amount: int) -> String:\n\tvar n := 0\n\twhile n < 3:\n\t\tn += 1\n\treturn str(amount)\n";
    assert!(money_findings("x.gd", clean, &mut used).is_empty());
    assert_eq!(
        comparisons("func f() -> int: return a >> 2 if b <= c else d"),
        1
    );
    assert_eq!(comparisons("var s := \"\"  # blanked string"), 0);
}

/// AC-16f-1 (ARC-51): the 3D client reads the Presentation Pack it is given. No 3D client script but
/// `PACK_RESOLVER` writes a path under `presentation/`, and none writes `PACK_DIR`; the resolver itself
/// does name the default pack, so the scan is seen to find what it looks for.
#[test]
fn the_3d_client_reads_its_pack_only_through_slice_pack() {
    let root = root();
    // The removability check (16e) runs this crate on a checkout with the 3D client removed.
    if !client_3d::present(&root, "its pack reads through SlicePack (AC-16f-1)") {
        return;
    }
    let mut files = Vec::new();
    client_scripts(&root.join(client_3d!("scripts")), &mut files);
    assert!(
        files.len() >= 40,
        "every 3D client script is scanned: {}",
        files.len()
    );
    let mut found = Vec::new();
    let mut resolver_names_the_default = false;
    for path in &files {
        let file = relative(path);
        let text = std::fs::read_to_string(path).expect("a client script reads");
        let findings = pack_path_findings(&file, &text);
        if file == PACK_RESOLVER {
            resolver_names_the_default = !findings.is_empty();
        } else {
            found.extend(findings);
        }
    }
    assert!(
        resolver_names_the_default,
        "{PACK_RESOLVER} names the default pack's path, and the scan finds it"
    );
    assert!(
        found.is_empty(),
        "a 3D client script reads a pack by path:\n{}",
        found.join("\n")
    );
    assert!(names_a_presentation_path(
        "../../presentation/mineworld-default/3D"
    ));
    assert!(!names_a_presentation_path("the 3D presentation file %s"));
}

/// Check (d)'s exemption is exactly a `tag(…)` argument, by position on the line.
#[test]
fn a_tag_argument_is_exempt_and_nothing_else() {
    let keys = vec!["bakery".to_owned(), "pier".to_owned()];
    let mut used = [false; PLACE_KEY_LITERALS.len()];
    let ok = place_key_findings(
        "x.gd",
        "var a := [tag(\"bakery\"), SliceDepiction.tag(\"pier\")]\n",
        &keys,
        &mut used,
    );
    assert!(ok.is_empty(), "{ok:?}");
    let bad = place_key_findings(
        "x.gd",
        "var a := tag(\"pier\") + \"bakery\"\nif p == 'pier':\n",
        &keys,
        &mut used,
    );
    assert_eq!(bad.len(), 2, "{bad:?}");
    assert!(bad[0].starts_with("x.gd:1: \"bakery\""), "{bad:?}");
    assert!(bad[1].starts_with("x.gd:2: \"pier\""), "{bad:?}");
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

/// What check (e) refuses in a depiction's script: a way of moving a prop the world does not move.
const MOVERS: [&str; 5] = [
    "create_tween",
    "Tween",
    "AnimationPlayer",
    "func _process(",
    "func _physics_process(",
];

/// Check (e) over one lexed file: each line of code that names a mover, by file and line.
fn mover_findings(file: &str, text: &str) -> Vec<String> {
    let lexed = lex(text);
    let mut found = Vec::new();
    for (i, line) in lexed.code.iter().enumerate() {
        for mover in MOVERS {
            let at = line.find(mover);
            // `Tween` as a word, not inside `TweenSomething` or `between`
            let word = at.is_some_and(|k| {
                let before = line[..k].chars().next_back();
                let after = line[k + mover.len()..].chars().next();
                mover.ends_with('(')
                    || (!before.is_some_and(|c| c.is_alphanumeric() || c == '_')
                        && !after.is_some_and(|c| c.is_alphanumeric() || c == '_'))
            });
            if word {
                found.push(format!(
                    "{file}:{}: `{mover}` moves a drawn prop; nothing moves unless a disclosed Process \
                     moves it (ARC-171): idle in a shader, or draw what the server discloses",
                    i + 1
                ));
            }
        }
    }
    found
}

/// Check (e), the honesty scan (S33-e §5.6, X-7; `DECISIONS.md` ARC-171): no script that draws a
/// depiction — the generic data builders and every world's depiction script — animates what it draws. A
/// landmark, a moored boat, a train or a ferry is static unless the world moves it; presentation idles in
/// shaders, which are not scanned.
#[test]
fn no_depiction_moves_a_prop_the_world_does_not_move() {
    let root = root();
    // The removability check (16e) runs this crate on a checkout with the 3D client removed.
    if !client_3d::present(&root, "no depiction moves a prop (S33-e X-7)") {
        return;
    }
    let mut files = Vec::new();
    client_scripts(&root.join(client_3d!("scripts/depictions")), &mut files);
    assert!(
        files
            .iter()
            .any(|f| f.ends_with("depictions/data/deck_builder.gd")),
        "the deck builder is scanned: {files:?}"
    );
    let mut found = Vec::new();
    for path in &files {
        let text = std::fs::read_to_string(path).expect("a depiction script reads");
        found.extend(mover_findings(&relative(path), &text));
    }
    assert!(
        found.is_empty(),
        "a depiction animates a prop:\n{}",
        found.join("\n")
    );
}

/// Check (e) names the file and line of a planted tween (X-7), and not a comment or a longer word.
#[test]
fn the_honesty_scan_names_a_tweened_boat() {
    let planted = "var boat := Node3D.new()\n\
                   # a Tween would be wrong here\n\
                   var t := boat.create_tween()\n\
                   var between := 1\n";
    let found = mover_findings("deck_builder.gd", planted);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].starts_with("deck_builder.gd:3: `create_tween`"),
        "{found:?}"
    );
}
