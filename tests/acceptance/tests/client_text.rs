//! S20 SET-a — client UI text and the settings module, held structurally (step-20 §7, §12.3
//! SD-SET-a-13; `clients/shared/SETTINGS.md` §§2, 6, 8, 9).
//!
//! ```text
//! AC-SET-3   no hardcoded UI string in the 3D client or the shared settings module: no string literal
//!            with a letter is an argument of a text sink, unless it is a catalog key or an entry of
//!            ALLOWED admits it with its reason (the 2D client is 13b's R6, scripts/check_client_rules.py)
//! AC-SET-4   the catalogs are complete and consistent across layers: every en.po key is in its layer's
//!            zh_Hans.po with the same placeholders; every key a script names exists in a layer that
//!            script's client loads; no en.po key is unused; the data-built families cover every action
//!            type, refusal code and rejection reason; a key in two layers is marked `#. override`;
//!            messages.pot is the template of the shared en.po
//! SD-SET-a-17 the move of 13b's wording into the shared layer is verbatim: the effective wording (pack
//!            over shared) of every entry of 13b's merged en.po is unchanged, apart from SD-SET-a-12's
//!            named changes
//! INV-SET-1  no network identifier in the settings module
//! AC-SET-16  no platform branch in the settings module, apart from the declared Wayland case
//! ```
//!
//! Comments are not read (the shared lexer, `support/gdscript.rs`). An admission that admits nothing
//! fails, so no list goes stale. Every root that is scanned must exist and hold files.
//!
//! The `.po` reader below is about sixty lines of std-only code: a gettext crate for one test would be a
//! heavier dependency than the parser it saves (SD-SET-a-13).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "support/gdscript.rs"]
mod gdscript;
use gdscript::lex;

/// The first segment of every catalog key (step-20 SD-SET-a-6, 13b's families).
const KEY_FAMILIES: [&str; 15] = [
    "action", "reason", "state", "ui", "panel", "suggest", "format", "hud", "hint", "camera",
    "link", "note", "door", "language", "clock",
];

/// Where UI text is set: a literal argument of one of these is UI text.
const SINKS: [&str; 11] = [
    ".text =",
    ".tooltip_text =",
    "add_item(",
    "set_item_text(",
    "add_tab(",
    "note(",
    "toast(",
    "caption(",
    "add_line(",
    "set_tab_title(",
    "_say(",
];

/// Literals at a text sink that are not UI text: (file, literal, why).
const ALLOWED: [(&str, &str, &str); 0] = [];

/// INV-SET-1: identifiers the settings module never names.
const NETWORK: [&str; 8] = [
    "MineWorldClient",
    "HTTPRequest",
    "WebSocketPeer",
    "StreamPeer",
    "PacketPeer",
    "submit",
    "submit_affordance",
    "connect_to_world",
];

/// AC-SET-16's one admitted platform question: (file, code line with strings blanked, why).
const PLATFORM_ADMITTED: [(&str, &str, &str); 1] = [(
    "clients/shared/settings/display.gd",
    "return DisplayServer.get_name() == \"\"",
    "the menu's Wayland tooltip: on Wayland exclusive full screen is the same as borderless (Godot's \
     DisplayServer reference; step-20 §3.5.1, AC-SET-16)",
)];

/// 13b's merged wording, the oracle of the move (S12 PR 13b, #103).
const BEFORE_MOVE: &str = "aee8290:presentation/mineworld-default/2D/i18n/en.po";
/// SD-SET-a-12's named en changes to 13b's entries: (key, why).
const NAMED_CHANGES: [(&str, &str); 1] = [(
    "ui.hint",
    "\"Esc: close, quit\" becomes \"Esc: close, menu\": Esc opens the settings menu, which holds Quit \
     (QSET-2)",
)];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn relative(path: &Path) -> String {
    let root = root().canonicalize().expect("the repository root resolves");
    let path = path.canonicalize().expect("a scanned file resolves");
    path.strip_prefix(&root)
        .expect("inside the repository")
        .to_string_lossy()
        .replace('\\', "/")
}

// --- the .po reader ----------------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Entry {
    line: usize,
    key: String,
    text: String,
    marked_override: bool,
}

#[derive(Debug)]
struct Catalog {
    name: String,
    language: String,
    entries: Vec<Entry>,
}

impl Catalog {
    fn get(&self, key: &str) -> Option<&Entry> {
        self.entries.iter().find(|entry| entry.key == key)
    }

    fn keys(&self) -> BTreeSet<String> {
        self.entries.iter().map(|entry| entry.key.clone()).collect()
    }
}

fn unquote(quoted: &str) -> String {
    let inner = quoted
        .trim()
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or_else(|| panic!("not a quoted gettext string: {quoted}"));
    let mut out = String::new();
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some(other) => out.push(other),
            None => {}
        }
    }
    out
}

/// msgid/msgstr pairs with continuation lines; `#.` comments before an entry are read for `override`.
fn parse_po(name: &str, text: &str) -> Catalog {
    let mut entries = Vec::new();
    let mut language = String::new();
    let mut pending_override = false;
    let mut current: Option<(usize, String, Option<String>, bool)> = None;
    let mut in_str = false;
    let flush = |current: &mut Option<(usize, String, Option<String>, bool)>,
                 entries: &mut Vec<Entry>,
                 language: &mut String| {
        if let Some((line, key, text, marked)) = current.take() {
            let text =
                text.unwrap_or_else(|| panic!("{name}:{line}: msgid \"{key}\" has no msgstr"));
            if key.is_empty() {
                for header in text.lines() {
                    if let Some(value) = header.strip_prefix("Language:") {
                        *language = value.trim().to_owned();
                    }
                }
            } else {
                entries.push(Entry {
                    line,
                    key,
                    text,
                    marked_override: marked,
                });
            }
        }
    };
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.starts_with("#.") && line[2..].trim() == "override" {
            pending_override = true;
        } else if line.is_empty() || line.starts_with('#') {
            continue;
        } else if let Some(rest) = line.strip_prefix("msgid ") {
            flush(&mut current, &mut entries, &mut language);
            current = Some((index + 1, unquote(rest), None, pending_override));
            pending_override = false;
            in_str = false;
        } else if let Some(rest) = line.strip_prefix("msgstr ") {
            let entry = current
                .as_mut()
                .unwrap_or_else(|| panic!("{name}:{}: msgstr without msgid", index + 1));
            entry.2 = Some(unquote(rest));
            in_str = true;
        } else if line.starts_with('"') {
            let entry = current
                .as_mut()
                .unwrap_or_else(|| panic!("{name}:{}: stray string", index + 1));
            if in_str {
                entry.2.as_mut().expect("a msgstr").push_str(&unquote(line));
            } else {
                entry.1.push_str(&unquote(line));
            }
        } else {
            panic!("{name}:{}: not a gettext line: {line}", index + 1);
        }
    }
    flush(&mut current, &mut entries, &mut language);
    Catalog {
        name: name.to_owned(),
        language,
        entries,
    }
}

fn read_po(path: &Path) -> Catalog {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
    parse_po(&relative(path), &text)
}

// --- keys, placeholders, scripts ---------------------------------------------------------------------

/// Whether a literal is a catalog key: dotted, lowercase segments `[a-z0-9_-]+`, a known family first.
fn is_key(literal: &str) -> bool {
    let segments: Vec<&str> = literal.split('.').collect();
    segments.len() >= 2
        && KEY_FAMILIES.contains(&segments[0])
        && segments.iter().all(|segment| {
            !segment.is_empty()
                && segment
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
        })
}

/// A literal that starts a key built from data: `"action."`, `"ui.state."`.
fn key_prefix(literal: &str) -> Option<&str> {
    let stem = literal.strip_suffix('.')?;
    (is_key(stem) || KEY_FAMILIES.contains(&stem)).then_some(literal)
}

/// The `{name}` placeholders of a text.
fn placeholders(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        match after.find('}') {
            Some(close)
                if after[..close]
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_') =>
            {
                out.insert(after[..close].to_owned());
                rest = &after[close + 1..];
            }
            _ => rest = after,
        }
    }
    out
}

/// Every `*.gd` under `directory`, not following symlinks, skipping dot directories and `tools/`.
fn scripts(directory: &Path, into: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("a scanned directory is missing: {directory:?}: {error}"));
    for entry in entries {
        let entry = entry.expect("a directory entry");
        let name = entry.file_name().to_string_lossy().into_owned();
        let kind = entry.file_type().expect("a file type");
        if kind.is_symlink() || name.starts_with('.') {
            continue;
        }
        if kind.is_dir() && name != "tools" {
            scripts(&entry.path(), into);
        } else if kind.is_file() && name.ends_with(".gd") {
            into.push(entry.path());
        }
    }
}

fn scripts_of(directories: &[&str]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for directory in directories {
        scripts(&root().join(directory), &mut files);
    }
    files.sort();
    files
}

/// Every string literal of a set of scripts.
fn literals_of(files: &[PathBuf]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for file in files {
        let text = std::fs::read_to_string(file).expect("a script reads");
        out.extend(lex(&text).literals.into_iter().map(|(_, literal)| literal));
    }
    out
}

/// The shared layer and every Presentation Pack layer, by folder.
fn layers() -> (PathBuf, Vec<PathBuf>) {
    let shared = root().join("clients/shared/settings/locale");
    let mut packs = Vec::new();
    for pack in std::fs::read_dir(root().join("presentation")).expect("presentation/ reads") {
        let pack = pack.expect("an entry").path();
        if !pack.is_dir() {
            continue;
        }
        for view in std::fs::read_dir(&pack).expect("a pack reads") {
            let i18n = view.expect("an entry").path().join("i18n");
            if i18n.is_dir() {
                packs.push(i18n);
            }
        }
    }
    packs.sort();
    assert!(
        shared.join("en.po").is_file(),
        "the shared layer's en.po exists"
    );
    (shared, packs)
}

// --- the code families -------------------------------------------------------------------------------

/// Every action type the build's System Packs declare (as `client_rules.rs` reads them).
fn action_types() -> Vec<String> {
    const MARK: &str = "ActionTypeId::from_static(\"";
    let mut types = BTreeSet::new();
    for pack in std::fs::read_dir(root().join("systems")).expect("systems/ reads") {
        let source = pack.expect("an entry").path().join("src");
        let mut files = Vec::new();
        rust_files(&source, &mut files);
        for file in files {
            let text = std::fs::read_to_string(&file).expect("a source reads");
            for (at, _) in text.match_indices(MARK) {
                let rest = &text[at + MARK.len()..];
                types.insert(rest[..rest.find('"').expect("closed")].to_owned());
            }
        }
    }
    types.into_iter().collect()
}

fn rust_files(directory: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries {
        let path = entry.expect("an entry").path();
        if path.is_dir() {
            rust_files(&path, into);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            into.push(path);
        }
    }
}

/// The refusal codes of `server/PROTOCOL.md` §5.5's table.
fn refusal_codes() -> Vec<String> {
    let text =
        std::fs::read_to_string(root().join("server/PROTOCOL.md")).expect("PROTOCOL.md reads");
    let section = text
        .split("### 5.5")
        .nth(1)
        .expect("PROTOCOL.md has §5.5")
        .split("\n### ")
        .next()
        .expect("a section");
    section
        .lines()
        .filter_map(|line| line.strip_prefix("| `"))
        .filter_map(|rest| rest.split('`').next())
        .filter(|code| *code != "code")
        .map(str::to_owned)
        .collect()
}

/// The unit variants of the contracts' `Rejection`, in their wire form (snake_case).
fn rejection_reasons() -> Vec<String> {
    let text =
        std::fs::read_to_string(root().join("contracts/src/action.rs")).expect("action.rs reads");
    let body = text
        .split("pub enum Rejection {")
        .nth(1)
        .expect("contracts declare Rejection")
        .split("\n}")
        .next()
        .expect("its body");
    let mut out = Vec::new();
    for line in body.lines() {
        let line = line.trim();
        if let Some(name) = line.strip_suffix(',')
            && !name.is_empty()
            && name.chars().all(|c| c.is_ascii_alphanumeric())
        {
            let mut snake = String::new();
            for (i, c) in name.chars().enumerate() {
                if c.is_ascii_uppercase() && i > 0 {
                    snake.push('_');
                }
                snake.push(c.to_ascii_lowercase());
            }
            out.push(snake);
        }
    }
    out
}

// --- AC-SET-4 ----------------------------------------------------------------------------------------

/// One layer's completeness: every en key in zh_Hans, non-empty, the same placeholders, nothing extra.
fn layer_findings(dir: &Path) -> Vec<String> {
    let mut found = Vec::new();
    let en = read_po(&dir.join("en.po"));
    let zh_path = dir.join("zh_Hans.po");
    if !zh_path.is_file() {
        return vec![format!("{}: no zh_Hans.po beside en.po", relative(dir))];
    }
    let zh = read_po(&zh_path);
    for (catalog, language) in [(&en, "en"), (&zh, "zh_Hans")] {
        if catalog.language != language {
            found.push(format!(
                "{}: its header says Language: {}",
                catalog.name, catalog.language
            ));
        }
        let mut seen = BTreeSet::new();
        for entry in &catalog.entries {
            if !seen.insert(&entry.key) {
                found.push(format!(
                    "{}:{}: \"{}\" again",
                    catalog.name, entry.line, entry.key
                ));
            }
            if !is_key(&entry.key) {
                found.push(format!(
                    "{}:{}: \"{}\" is not a key",
                    catalog.name, entry.line, entry.key
                ));
            }
        }
    }
    for entry in &en.entries {
        match zh.get(&entry.key) {
            None => found.push(format!(
                "{}: \"{}\" has no zh_Hans entry",
                zh.name, entry.key
            )),
            Some(other) if other.text.trim().is_empty() => {
                found.push(format!(
                    "{}:{}: \"{}\" is empty",
                    zh.name, other.line, entry.key
                ));
            }
            Some(other) if placeholders(&other.text) != placeholders(&entry.text) => {
                found.push(format!(
                    "{}:{}: \"{}\" has placeholders {:?}, en has {:?}",
                    zh.name,
                    other.line,
                    entry.key,
                    placeholders(&other.text),
                    placeholders(&entry.text)
                ))
            }
            Some(other) if other.marked_override != entry.marked_override => found.push(format!(
                "{}:{}: \"{}\" is marked override in one language only",
                zh.name, other.line, entry.key
            )),
            Some(_) => {}
        }
    }
    for entry in &zh.entries {
        if en.get(&entry.key).is_none() {
            found.push(format!(
                "{}:{}: \"{}\" is not in en.po",
                zh.name, entry.line, entry.key
            ));
        }
    }
    found
}

/// The template of the shared en.po, as `messages.pot` must read.
fn template(en: &Catalog) -> String {
    let mut out = String::from(
        "# The translation template of the shared catalog layer, generated from en.po by\n\
         # tests/acceptance/tests/client_text.rs (MINEWORLD_WRITE_POT=1 cargo test -p mineworld-acceptance\n\
         # --test client_text). Do not edit by hand. Each entry's English text is its extracted comment.\n\
         msgid \"\"\n\
         msgstr \"\"\n\
         \"Project-Id-Version: mineworld-shared-settings 0.1.0\\n\"\n\
         \"MIME-Version: 1.0\\n\"\n\
         \"Content-Type: text/plain; charset=UTF-8\\n\"\n\
         \"Content-Transfer-Encoding: 8bit\\n\"\n",
    );
    for entry in &en.entries {
        out.push('\n');
        for line in entry.text.lines() {
            out.push_str(&format!("#. {line}\n"));
        }
        out.push_str(&format!("msgid \"{}\"\nmsgstr \"\"\n", entry.key));
    }
    out
}

/// Keys that a client's scripts name must exist in the en.po of a layer that client loads.
fn missing_keys(client: &str, files: &[PathBuf], loaded: &BTreeSet<String>) -> Vec<String> {
    let mut found = Vec::new();
    for file in files {
        let text = std::fs::read_to_string(file).expect("a script reads");
        for (line, literal) in lex(&text).literals {
            if is_key(&literal) && !loaded.contains(&literal) {
                found.push(format!(
                    "{}:{line}: \"{literal}\" is in no catalog the {client} client loads",
                    relative(file)
                ));
            }
        }
    }
    found
}

#[test]
fn the_catalogs_are_complete_and_consistent() {
    let (shared_dir, packs) = layers();
    let mut found = layer_findings(&shared_dir);
    for pack in &packs {
        found.extend(layer_findings(pack));
    }
    let shared = read_po(&shared_dir.join("en.po"));
    let shared_zh = read_po(&shared_dir.join("zh_Hans.po"));
    for key in ["language.self_name", "clock.default"] {
        for catalog in [&shared, &shared_zh] {
            if catalog.get(key).is_none() {
                found.push(format!("{}: no \"{key}\"", catalog.name));
            }
        }
    }
    // A key in two layers is marked `#. override` in the later one, and a mark must override something.
    let mut pack_keys = BTreeSet::new();
    for pack in &packs {
        let en = read_po(&pack.join("en.po"));
        for entry in &en.entries {
            pack_keys.insert(entry.key.clone());
            let in_shared = shared.get(&entry.key).is_some();
            if in_shared && !entry.marked_override {
                found.push(format!(
                    "{}:{}: \"{}\" is also in the shared layer; mark it `#. override`",
                    en.name, entry.line, entry.key
                ));
            }
            if entry.marked_override && !in_shared {
                found.push(format!(
                    "{}:{}: \"{}\" is marked override but overrides nothing",
                    en.name, entry.line, entry.key
                ));
            }
        }
    }
    // The data-built families cover every action type, refusal code and rejection reason.
    let mut codes = Vec::new();
    for action in action_types() {
        codes.push(format!("action.{action}"));
    }
    let refusals = refusal_codes();
    let reasons = rejection_reasons();
    assert!(
        refusals.len() >= 15 && refusals.contains(&"paused".to_owned()),
        "§5.5 read: {refusals:?}"
    );
    assert!(
        reasons.contains(&"too_far_away".to_owned()) && reasons.len() >= 7,
        "Rejection read: {reasons:?}"
    );
    codes.extend(
        refusals
            .iter()
            .chain(&reasons)
            .map(|code| format!("reason.{code}")),
    );
    for key in &codes {
        for catalog in [&shared, &shared_zh] {
            if catalog.get(key).is_none() {
                found.push(format!(
                    "{}: no \"{key}\" (a code the server or a System Pack sends)",
                    catalog.name
                ));
            }
        }
    }
    // Every key named in code exists where that client loads it.
    let shared_keys = shared.keys();
    let two_d = scripts_of(&["clients/2d/scripts"]);
    let three_d = scripts_of(&["clients/3d-spike/scripts"]);
    let module = scripts_of(&["clients/shared/settings"]);
    // The checks name keys no catalog has on purpose (the fallback); they count as asking, not as a client.
    let checks = scripts_of(&["clients/shared/checks"]);
    let two_d_pack = read_po(&root().join("presentation/mineworld-default/2D/i18n/en.po")).keys();
    let two_d_keys: BTreeSet<String> = shared_keys.union(&two_d_pack).cloned().collect();
    found.extend(missing_keys("2D", &two_d, &two_d_keys));
    found.extend(missing_keys("3D", &three_d, &shared_keys));
    found.extend(missing_keys("shared module's", &module, &shared_keys));
    // No en.po key is unused: named by a literal, under a data-built prefix, or of a data-built family.
    let all: Vec<PathBuf> = two_d
        .iter()
        .chain(&three_d)
        .chain(&module)
        .chain(&checks)
        .cloned()
        .collect();
    let used = literals_of(&all);
    let prefixes: Vec<&str> = used
        .iter()
        .filter_map(|literal| key_prefix(literal))
        .collect();
    for key in shared_keys.iter().chain(&pack_keys) {
        let family = key.split('.').next().unwrap_or("");
        let data_built = ["action", "reason", "state"].contains(&family);
        if !used.contains(key)
            && !prefixes.iter().any(|prefix| key.starts_with(prefix))
            && !data_built
        {
            found.push(format!(
                "\"{key}\" is in a catalog and no script asks for it"
            ));
        }
    }
    // messages.pot is the template of the shared en.po.
    let pot_path = shared_dir.join("messages.pot");
    let expected = template(&shared);
    if std::env::var_os("MINEWORLD_WRITE_POT").is_some() {
        std::fs::write(&pot_path, &expected).expect("messages.pot writes");
    }
    // Line endings as a Windows checkout may write them (core.autocrlf) are not a difference.
    let on_disk = std::fs::read_to_string(&pot_path)
        .ok()
        .map(|text| text.replace("\r\n", "\n"));
    if on_disk.as_deref() != Some(expected.as_str()) {
        found.push(format!(
            "{}: not the template of en.po; regenerate with MINEWORLD_WRITE_POT=1",
            relative(&shared_dir)
        ));
    }
    assert!(
        found.is_empty(),
        "the catalogs (AC-SET-4):\n{}",
        found.join("\n")
    );
}

// --- SD-SET-a-17 -------------------------------------------------------------------------------------

#[test]
fn the_move_of_13b_wording_is_verbatim() {
    let shown = Command::new("git")
        .args(["show", BEFORE_MOVE])
        .current_dir(root())
        .output()
        .expect("git runs");
    assert!(
        shown.status.success(),
        "13b's merged wording is in the history ({BEFORE_MOVE}): {}",
        String::from_utf8_lossy(&shown.stderr)
    );
    let before = parse_po(
        BEFORE_MOVE,
        &String::from_utf8(shown.stdout).expect("UTF-8"),
    );
    let (shared_dir, _) = layers();
    let shared = read_po(&shared_dir.join("en.po"));
    let pack = read_po(&root().join("presentation/mineworld-default/2D/i18n/en.po"));
    let mut found = Vec::new();
    let mut named_used = [false; NAMED_CHANGES.len()];
    for entry in &before.entries {
        let effective = pack.get(&entry.key).or_else(|| shared.get(&entry.key));
        let Some(effective) = effective else {
            found.push(format!(
                "\"{}\" was 13b's and is in neither layer",
                entry.key
            ));
            continue;
        };
        let moved = entry.key.starts_with("action.") || entry.key.starts_with("reason.");
        let lives_in_pack = pack.get(&entry.key).is_some();
        if moved && entry.key != "action.move" && lives_in_pack {
            found.push(format!(
                "\"{}\" was to move to the shared layer and is still in the pack",
                entry.key
            ));
        }
        if !moved && !lives_in_pack {
            found.push(format!(
                "\"{}\" is the pack's own wording and left the pack",
                entry.key
            ));
        }
        if effective.text != entry.text {
            match NAMED_CHANGES.iter().position(|(key, _)| *key == entry.key) {
                Some(at) => named_used[at] = true,
                None => found.push(format!(
                    "\"{}\" reads \"{}\", 13b's reads \"{}\"",
                    entry.key, effective.text, entry.text
                )),
            }
        }
    }
    for ((key, why), used) in NAMED_CHANGES.iter().zip(named_used) {
        assert!(!why.is_empty(), "{key}: a named change needs its reason");
        assert!(used, "{key}: the named change changes nothing; remove it");
    }
    assert!(
        found.is_empty(),
        "the move of 13b's wording (SD-SET-a-17):\n{}",
        found.join("\n")
    );
}

// --- AC-SET-3 ----------------------------------------------------------------------------------------

/// Whether a literal says something in words: letters outside `{placeholders}` and `%` specifiers.
fn has_words(literal: &str) -> bool {
    let mut out = String::new();
    let mut chars = literal.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            for d in chars.by_ref() {
                if d == '}' {
                    break;
                }
            }
        } else if c == '%' {
            while chars
                .peek()
                .is_some_and(|d| d.is_ascii_digit() || "-+.#0 ".contains(*d))
            {
                chars.next();
            }
            chars.next();
        } else {
            out.push(c);
        }
    }
    // Two letters at least: the lexer keeps an escape's character, so "\n" reads as "n".
    out.chars().filter(|c| c.is_alphabetic()).count() >= 2
}

/// What one script shows that is not a key: (line, literal).
fn shown_literals(text: &str) -> Vec<(usize, String)> {
    let lexed = lex(text);
    let mut by_line: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    for (line, literal) in lexed.literals {
        by_line.entry(line).or_default().push(literal);
    }
    let mut out = Vec::new();
    for (index, code) in lexed.code.iter().enumerate() {
        let line = index + 1;
        let Some(sink) = SINKS.iter().filter_map(|sink| code.find(sink)).min() else {
            continue;
        };
        if code[sink..].starts_with(".text ==") {
            continue;
        }
        let literals = by_line.get(&line).cloned().unwrap_or_default();
        let starts: Vec<usize> = code.match_indices("\"\"").map(|(at, _)| at).collect();
        let say_log = code
            .find("_say(")
            .and_then(|at| starts.iter().position(|start| *start > at));
        for (k, literal) in literals.into_iter().enumerate() {
            let Some(&start) = starts.get(k) else {
                continue;
            };
            let before = code[..start].trim_end();
            // A dictionary key — `data["name"]`, `get("name")`, `{"fps": n}` — names data; it shows nothing.
            let subscript = before.ends_with('[')
                || before.ends_with("get(")
                || before.ends_with("has(")
                || code[start + 2..].trim_start().starts_with(':');
            if start < sink
                || subscript
                || Some(k) == say_log
                || is_key(&literal)
                || !has_words(&literal)
            {
                continue;
            }
            out.push((line, literal));
        }
    }
    out
}

#[test]
fn no_hardcoded_ui_text_in_the_3d_client_or_the_settings_module() {
    let files = scripts_of(&["clients/3d-spike/scripts", "clients/shared/settings"]);
    // The 3D client's 34 scripts and the settings module's: a walk that skipped a folder reads fewer.
    assert!(
        files.len() >= 39,
        "every script is scanned: {}",
        files.len()
    );
    let mut found = Vec::new();
    let mut used = [false; ALLOWED.len()];
    for file in &files {
        let name = relative(file);
        let text = std::fs::read_to_string(file).expect("a script reads");
        for (line, literal) in shown_literals(&text) {
            match ALLOWED
                .iter()
                .position(|(f, l, _)| *f == name && *l == literal)
            {
                Some(at) => used[at] = true,
                None => found.push(format!(
                    "{name}:{line}: shows \"{literal}\"; UI text is a catalog key"
                )),
            }
        }
    }
    for ((file, literal, why), used) in ALLOWED.iter().zip(used) {
        assert!(!why.is_empty(), "{file}: an admission needs its reason");
        assert!(
            used,
            "{file}: the admission of \"{literal}\" admits nothing; remove it"
        );
    }
    assert!(
        found.is_empty(),
        "no hardcoded UI string (AC-SET-3, INV-SET-4):\n{}",
        found.join("\n")
    );
}

// --- INV-SET-1 and AC-SET-16 -------------------------------------------------------------------------

fn names_identifier(code: &str, name: &str) -> bool {
    code.match_indices(name).any(|(at, _)| {
        let before = code[..at].chars().next_back();
        let after = code[at + name.len()..].chars().next();
        !before.is_some_and(|c| c.is_alphanumeric() || c == '_')
            && !after.is_some_and(|c| c.is_alphanumeric() || c == '_')
    })
}

#[test]
fn the_settings_module_has_no_network_code_and_no_platform_branch() {
    let files = scripts_of(&["clients/shared/settings"]);
    assert!(
        files.len() >= 5,
        "the module's scripts are read: {}",
        files.len()
    );
    let mut found = Vec::new();
    let mut admitted = [false; PLATFORM_ADMITTED.len()];
    for file in &files {
        let name = relative(file);
        let text = std::fs::read_to_string(file).expect("a script reads");
        for (index, code) in lex(&text).code.iter().enumerate() {
            for identifier in NETWORK {
                if names_identifier(code, identifier) {
                    found.push(format!(
                        "{name}:{}: names {identifier} (INV-SET-1)",
                        index + 1
                    ));
                }
            }
            let asks = code.contains("OS.get_name()") || code.contains("DisplayServer.get_name()");
            let compares = code.contains("==")
                || code.contains("!=")
                || code.trim_start().starts_with("match ")
                || code.contains(" in ");
            if (asks && compares) || code.contains("OS.has_feature(") {
                match PLATFORM_ADMITTED
                    .iter()
                    .position(|(f, line, _)| *f == name && *line == code.trim())
                {
                    Some(at) => admitted[at] = true,
                    None => found.push(format!(
                        "{name}:{}: a platform branch (AC-SET-16): {}",
                        index + 1,
                        code.trim()
                    )),
                }
            }
        }
    }
    for ((file, line, why), used) in PLATFORM_ADMITTED.iter().zip(admitted) {
        assert!(!why.is_empty(), "{file}: an admission needs its reason");
        assert!(
            used,
            "{file}: the admission of `{line}` admits nothing; remove it"
        );
    }
    assert!(
        found.is_empty(),
        "the settings module (INV-SET-1, AC-SET-16):\n{}",
        found.join("\n")
    );
}

/// The reader and the scan's helpers on small inputs whose answers are known by hand.
#[test]
fn the_reader_and_the_scan_read_what_they_should() {
    let catalog = parse_po(
        "t.po",
        "msgid \"\"\nmsgstr \"\"\n\"Language: zh_Hans\\n\"\n\n#. override\nmsgid \"ui.a\"\nmsgstr \"x {b} \"\n\"y\"\n",
    );
    assert_eq!(catalog.language, "zh_Hans");
    assert_eq!(catalog.entries.len(), 1);
    assert_eq!(catalog.entries[0].text, "x {b} y");
    assert!(catalog.entries[0].marked_override);
    assert_eq!(
        placeholders("{a} and {b_2}, not {c d}"),
        BTreeSet::from(["a".to_owned(), "b_2".to_owned()])
    );
    assert!(is_key("action.join-group-activity.done") && is_key("reason.too_far_away"));
    assert!(!is_key("text.gd") && !is_key("res://x") && !is_key("ui"));
    assert!(!has_words("%s: %d  ·  {target}") && has_words("place: %s") && has_words("简体中文"));
    let shown = shown_literals(
        "label.text = \"Hello\"\nlabel.text = tr(\"ui.hello\")\nlabel.text = data[\"name\"]\n\
         _say(\"log line\", \"link.seated\")\nprint(\"not shown\")\nl.text = \"%s: %s\" % [a, b]\n\
         l.text = tr(\"ui.n\").format({\"count\": n})\n",
    );
    assert_eq!(shown, [(1, "Hello".to_owned())]);
}
