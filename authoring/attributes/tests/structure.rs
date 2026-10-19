//! Structural guards (`pr-s30c-attributes-pack.md` I-c-2, I-c-7; step-30 AC-S30-12, X-S30-10).

use std::path::{Path, PathBuf};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// Every `.rs` file under `dir`, skipping build output.
fn sources(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(next) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&next) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            if path.is_dir() {
                if name != "target" && !name.to_string_lossy().starts_with('.') {
                    pending.push(path);
                }
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// I-c-2 (AC-S30-12): the kernel and the contracts know nothing of attributes.
#[test]
fn the_kernel_and_the_contracts_name_no_attribute_concept() {
    let words = [
        "AttributeId",
        "AttributeValue",
        "Catalogue",
        "Handle<",
        "AttributeOwner",
        "traits-stated",
        "trait-changed",
        "attribute-change-proposed",
        "attributes-composed",
        "attributes-handed",
        "attributes-catalogue",
        "\"traits\"",
    ];
    for crate_dir in ["kernel", "contracts"] {
        for file in sources(&Path::new(ROOT).join(crate_dir)) {
            let text = std::fs::read_to_string(&file).expect("readable");
            for word in words {
                assert!(!text.contains(word), "{} names {word}", file.display());
            }
        }
    }
}

/// X-S30-10: every `Handle::declared(` sits in a `const` item — a handle is never built from a slug at
/// run time.
#[test]
fn every_handle_is_a_constant() {
    let model = Path::new(ROOT).join("authoring/attributes");
    for file in sources(Path::new(ROOT)) {
        if file.starts_with(&model) {
            continue;
        }
        let text = std::fs::read_to_string(&file).expect("readable");
        for (number, line) in text.lines().enumerate() {
            if line.contains("Handle::declared(") && !line.trim_start().starts_with("//") {
                assert!(
                    line.contains("const "),
                    "{}:{}: Handle::declared outside a const item",
                    file.display(),
                    number + 1
                );
            }
        }
    }
}
