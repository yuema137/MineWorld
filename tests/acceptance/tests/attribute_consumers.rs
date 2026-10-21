//! S30-f AC-f-9 (`pr-s30f-attribute-consumers.md` §8; I-f-5, I-f-6, F-S30f-6): no attribute consumer
//! names an attribute id, a backend or `matraix`; the Python and GDScript consumers render no value; and the
//! shared protocol module both clients load is one module.
//!
//! ```text
//! a  over every consumer's source (cognition, motivated, the Python SDK, the agent, both clients and the
//!    protocol module): no literal `core.<slug>` and no `matraix` in any case — selection is by facet,
//!    and the built-in catalogue and MatrAIx are data a world chooses (P-1, P-2). Test files are not
//!    consumers and are not read.
//! b  over the Python and GDScript attribute readers (every file whose name holds `attribute`): no
//!    `{value}` and no `perceived_as` — the server renders every phrase once; these select, join and cut
//!    (F-S30f-8)
//! c  `clients/2d/mineworld` and `clients/3d-spike/mineworld` are `clients/protocol/mineworld`: a symlink
//!    to it or a byte-identical copy of every file in it
//! ```
//!
//! Without the 3D client (ARC-347's removal check), (a), (b) and (c) skip its directories and say so.

use std::path::{Path, PathBuf};

// The 3D client's directory, named once (16e D-e-3).
#[macro_use]
#[path = "support/client_3d.rs"]
mod client_3d;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The consumer directories of (a), and whether each is a Python or GDScript consumer for (b).
const CONSUMERS: [(&str, bool); 9] = [
    ("cognition/lm-controller/src", true),
    ("cognition/motivated/src", false),
    ("sdk/python/src", true),
    ("clients/agent/src", true),
    ("clients/2d/scripts", true),
    (client_3d!("scripts"), true),
    ("clients/protocol/mineworld", true),
    ("clients/2d/mineworld", true),
    (client_3d!("mineworld"), true),
];

/// Every source file under `directory`, test files left out.
fn sources(directory: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            sources(&path, into);
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        let source = [".rs", ".py", ".gd"].iter().any(|ext| name.ends_with(ext));
        let test = name.ends_with("_tests.rs") || name == "tests.rs" || name.starts_with("test_");
        if source && !test {
            into.push(path);
        }
    }
}

/// Whether `text` holds `core.` followed by a lower-case letter, not inside a longer word.
fn names_a_core_id(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    text.match_indices("core.").map(|(at, _)| at).find(|&at| {
        let before = at.checked_sub(1).map(|i| bytes[i]);
        let after = bytes.get(at + 5).copied();
        !before.is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_')
            && after.is_some_and(|b| b.is_ascii_lowercase())
    })
}

fn line_of(text: &str, at: usize) -> usize {
    text[..at].matches('\n').count() + 1
}

#[test]
fn no_consumer_names_an_id_a_backend_or_renders_a_value() {
    let mut findings = Vec::new();
    let mut read = 0;
    let three_d = client_3d::present(&root(), "the 3D client's consumers (AC-f-9 a, b)");
    for (directory, foreign) in CONSUMERS {
        if !three_d && client_3d::holds(directory) {
            continue;
        }
        let mut files = Vec::new();
        sources(&root().join(directory), &mut files);
        for file in files {
            let text = std::fs::read_to_string(&file).unwrap_or_default();
            read += 1;
            let shown = file
                .strip_prefix(root())
                .unwrap_or(&file)
                .display()
                .to_string();
            if let Some(at) = names_a_core_id(&text) {
                findings.push(format!(
                    "{shown}:{}: a core attribute id",
                    line_of(&text, at)
                ));
            }
            if let Some(at) = text.to_lowercase().find("matraix") {
                findings.push(format!("{shown}:{}: matraix", line_of(&text, at)));
            }
            if foreign && shown.contains("attribute") {
                for word in ["{value}", "perceived_as"] {
                    if let Some(at) = text.find(word) {
                        findings.push(format!("{shown}:{}: {word}", line_of(&text, at)));
                    }
                }
            }
        }
    }
    assert!(read > 50, "the scan read {read} files: a root is missing");
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn both_clients_load_the_one_protocol_module() {
    let shared = root().join("clients/protocol/mineworld");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&shared)
        .expect("the protocol module")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect();
    files.sort();
    assert!(
        files.iter().any(|file| file.ends_with("attributes.gd")),
        "the attribute reader is in the module"
    );
    let three_d = client_3d::present(&root(), "the 3D client's copy of the module (AC-f-9 c)");
    for client in ["clients/2d/mineworld", client_3d!("mineworld")] {
        if !three_d && client_3d::holds(client) {
            continue;
        }
        let copy = root().join(client);
        let link = std::fs::symlink_metadata(&copy).expect("the client's module");
        if link.file_type().is_symlink() {
            let target = std::fs::canonicalize(&copy).expect("resolves");
            assert_eq!(
                target,
                std::fs::canonicalize(&shared).expect("resolves"),
                "{client}"
            );
            continue;
        }
        for file in &files {
            let name = file.file_name().expect("a name");
            assert_eq!(
                std::fs::read(copy.join(name)).ok(),
                std::fs::read(file).ok(),
                "{client}/{} differs from the protocol module's",
                name.to_string_lossy()
            );
        }
    }
}
