//! (h) Integers only (`INV-TW-5`, step-19 §17.6 criterion 6): no floating-point type token appears in
//! the pack's source. The scan reads every `.rs` file under `src/`, comments included, so a float
//! cannot hide in a doc example either. (This file is outside `src/` and is not scanned.)

use std::path::{Path, PathBuf};

fn sources(directory: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).expect("readable") {
        let path = entry.expect("an entry").path();
        if path.is_dir() {
            sources(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
}

/// Whether `text` holds `token` as a word (not inside an identifier such as `xf64y`).
fn holds_word(text: &str, token: &str) -> bool {
    let word = |c: char| c.is_ascii_alphanumeric() || c == '_';
    text.match_indices(token).any(|(at, _)| {
        let before = text[..at].chars().next_back().is_none_or(|c| !word(c));
        let after = text[at + token.len()..]
            .chars()
            .next()
            .is_none_or(|c| !word(c));
        before && after
    })
}

#[test]
fn no_floating_point_type_appears_in_the_packs_source() {
    let mut found = Vec::new();
    sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut found,
    );
    assert!(found.len() >= 8, "the scan found the sources: {found:?}");
    let tokens = [concat!("f", "32"), concat!("f", "64")];
    let offending: Vec<String> = found
        .iter()
        .flat_map(|path| {
            let text = std::fs::read_to_string(path).expect("readable");
            tokens
                .iter()
                .filter(|token| holds_word(&text, token))
                .map(|token| format!("{}: {token}", path.display()))
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(
        offending.is_empty(),
        "floating point in the pack: {offending:?}"
    );
}
