//! S30-k1 AC-k-8 (`pr-s30k-localisation.md` §§6, 8; I-k-4, I-k-8): attribute text is localised by data
//! only. Over every attribute module and attribute reader — the model crate, the `attributes` system, the
//! Python SDK and cognition readers, the agent's disclosed reader, the shared GDScript module and both
//! clients' attribute panels — no code holds a CJK character or a locale literal (`"zh_Hans"`,
//! `"zh-Hans"`, `"fr"`, quoted in either style), and the GDScript readers put no attribute text through
//! `tr(`. Test files are not consumers and are not read; comments may name a locale as an example.
//!
//! The fact half of AC-k-6 (labels never reach a fact) is `tools/cli/tests/attributes_labels.rs`, which
//! runs the real binary on a world with and without its label files.

use std::path::{Path, PathBuf};

#[macro_use]
#[path = "support/client_3d.rs"]
mod client_3d;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Whole directories every source file of which is attribute code.
const MODULES: [&str; 2] = ["authoring/attributes/src", "systems/attributes/src"];

/// Directories whose attribute readers (a file name holding `attribute` or `disclosed`) are scanned, and
/// the clients' attribute panels named one by one.
const READERS: [&str; 5] = [
    "sdk/python/src",
    "cognition/lm-controller/src",
    "clients/agent/src",
    "clients/protocol/mineworld",
    "clients/2d/mineworld",
];
const PANELS: [&str; 2] = [
    "clients/2d/scripts/hud/readers.gd",
    client_3d!("scripts/slice/inspect_panel.gd"),
];

fn is_test(name: &str) -> bool {
    name.ends_with("_tests.rs") || name == "tests.rs" || name.starts_with("test_")
}

/// Every non-test `.rs`, `.py` and `.gd` file under `directory` that `keep` accepts by name.
fn sources(directory: &Path, keep: &dyn Fn(&str) -> bool, into: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            if name != "tests" {
                sources(&path, keep, into);
            }
            continue;
        }
        let source = [".rs", ".py", ".gd"].iter().any(|ext| name.ends_with(ext));
        if source && !is_test(&name) && keep(&name) {
            into.push(path);
        }
    }
}

fn scanned() -> Vec<PathBuf> {
    let root = root();
    let mut files = Vec::new();
    for module in MODULES {
        sources(&root.join(module), &|_| true, &mut files);
    }
    for directory in READERS {
        sources(
            &root.join(directory),
            &|name| name.contains("attribute") || name.contains("disclosed"),
            &mut files,
        );
    }
    files.extend(PANELS.iter().map(|panel| root.join(panel)));
    files.retain(|file| file.exists());
    files
}

/// The code of one line: the part before a comment (`//`, `#`, `##`).
fn code(line: &str, rust: bool) -> &str {
    let marker = if rust { "//" } else { "#" };
    line.find(marker).map_or(line, |at| &line[..at])
}

fn is_cjk(character: char) -> bool {
    matches!(character,
        '\u{3000}'..='\u{303F}' | '\u{3400}'..='\u{4DBF}' | '\u{4E00}'..='\u{9FFF}' | '\u{FF00}'..='\u{FFEF}')
}

#[test]
fn no_attribute_module_holds_a_locale_literal_or_a_translated_word() {
    let files = scanned();
    assert!(
        files.len() >= 10,
        "the scan reads the attribute modules: {files:?}"
    );
    let literals = [
        "\"zh_Hans\"",
        "'zh_Hans'",
        "\"zh-Hans\"",
        "'zh-Hans'",
        "\"fr\"",
        "'fr'",
    ];
    let mut found = Vec::new();
    for file in &files {
        let text = std::fs::read_to_string(file).expect("a source file reads");
        let rust = file.extension().is_some_and(|ext| ext == "rs");
        let gdscript = file.extension().is_some_and(|ext| ext == "gd");
        let python = file.extension().is_some_and(|ext| ext == "py");
        let mut docstring = false;
        for (number, line) in text.lines().enumerate() {
            let code = code(line, rust);
            let at = format!("{}:{}", file.display(), number + 1);
            // A Python docstring is documentation, like a comment: it may quote prose.
            let quotes = line.matches("\"\"\"").count();
            let in_docstring = python && (docstring || quotes > 0);
            if python && quotes % 2 == 1 {
                docstring = !docstring;
            }
            if !in_docstring && code.chars().any(is_cjk) {
                found.push(format!("{at}: a CJK character"));
            }
            if let Some(literal) = literals.iter().find(|literal| code.contains(*literal)) {
                found.push(format!("{at}: the locale literal {literal}"));
            }
            if gdscript
                && (code.contains(" tr(") || code.contains("(tr(") || code.contains("\ttr("))
            {
                found.push(format!("{at}: attribute text through tr()"));
            }
        }
    }
    assert!(found.is_empty(), "{}", found.join("\n"));
}
