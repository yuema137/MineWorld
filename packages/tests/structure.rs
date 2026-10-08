//! `mineworld-packages` is a leaf: it names no MineWorld crate but itself (step-16 §14.4 EA-9,
//! `DECISIONS.md` `ARC-53`). The SDK, the World Pack loader, the CLI and the rule controller depend on
//! it; a MineWorld dependency here would put that crate under all four, and the kernel under the
//! controller.

use std::path::Path;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

#[test]
fn the_manifest_names_no_mineworld_dependency() {
    let manifest = std::fs::read_to_string(Path::new(ROOT).join("Cargo.toml")).expect("manifest");
    let mut table = String::new();
    let mut named = Vec::new();
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            line.clone_into(&mut table);
            continue;
        }
        let dependencies = table.contains("dependencies");
        if dependencies && line.starts_with("mineworld") {
            named.push(format!("{table} {line}"));
        }
    }
    assert!(manifest.contains("[dependencies]"), "the manifest was read");
    assert!(
        named.is_empty(),
        "a MineWorld dependency in a leaf: {named:?}"
    );
}

/// Code only: a comment may say who uses this crate (`mineworld_sdk::package!()` in an example), which
/// is documentation of the dependency pointing the other way.
#[test]
fn the_sources_name_no_other_mineworld_crate() {
    let mut named = Vec::new();
    let mut files = 0;
    for entry in std::fs::read_dir(Path::new(ROOT).join("src")).expect("src") {
        let path = entry.expect("an entry").path();
        let text = std::fs::read_to_string(&path).expect("a source file");
        files += 1;
        for line in text
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
        {
            for (at, _) in line.match_indices("mineworld_") {
                let name: String = line[at..]
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                if name != "mineworld_packages" {
                    named.push(format!("{}: {name}", path.display()));
                }
            }
        }
    }
    assert!(files > 0, "no source file was read");
    assert!(named.is_empty(), "{named:?}");
}
