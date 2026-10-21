//! A 3D Presentation Pack's place shells (`places/shells.yaml`; S14 16f-b, `ARC-52`): every refusal by
//! its code and the detail that names the field, over the corpus `tests/fixtures/shells/`, and the two
//! shipped packs' files accepted.
//!
//! A corpus file's stem is a code in lower case with `-` for `_`, without the `SHELLS_` prefix
//! (`out-of-range.yaml` → `SHELLS_OUT_OF_RANGE`), optionally followed by `-<variant>`; `valid*` files
//! must pass.

use std::path::{Path, PathBuf};

use mineworld_packages::shells::{self, ShellsCode};
use mineworld_packages::{PackageError, SHELLS_FILE};

fn corpus() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/shells")
}

fn code_of(stem: &str) -> Option<ShellsCode> {
    if stem.starts_with("valid") {
        return None;
    }
    ShellsCode::ALL
        .into_iter()
        .filter(|code| {
            let name = code
                .as_str()
                .trim_start_matches("SHELLS_")
                .to_lowercase()
                .replace('_', "-");
            stem == name || stem.starts_with(&format!("{name}-"))
        })
        .max_by_key(|code| code.as_str().len())
        .or_else(|| panic!("{stem}: no code matches this fixture's name"))
}

#[test]
fn every_planted_shells_file_is_refused_by_its_code_and_names_the_field() {
    let rows: [(&str, ShellsCode, &str); 5] = [
        ("malformed", ShellsCode::Malformed, "roof"),
        ("malformed-format", ShellsCode::Malformed, "format"),
        (
            "out-of-range",
            ShellsCode::OutOfRange,
            "default.wall.height_m",
        ),
        (
            "out-of-range-colour",
            ShellsCode::OutOfRange,
            "materials.plaster_grey.colour[1]",
        ),
        (
            "reference",
            ShellsCode::Reference,
            "tags.cafe.floor.material: 'oak_boards'",
        ),
    ];
    for (stem, code, names) in rows {
        let path = corpus().join(format!("{stem}.yaml"));
        let Err(PackageError::Shells {
            code: got, detail, ..
        }) = shells::read(&path)
        else {
            panic!("{stem}: not refused as a shells fault");
        };
        assert_eq!(got, code, "{stem}: {detail}");
        assert!(
            detail.contains(names),
            "{stem}: the detail must name {names}: {detail}"
        );
    }
}

#[test]
fn every_corpus_file_gives_the_code_its_name_states() {
    let mut read = 0;
    for entry in std::fs::read_dir(corpus()).expect("the corpus") {
        let path = entry.expect("an entry").path();
        let stem = path
            .file_stem()
            .and_then(std::ffi::OsStr::to_str)
            .expect("a stem")
            .to_owned();
        let got = match shells::read(&path) {
            Ok(_) => None,
            Err(PackageError::Shells { code, .. }) => Some(code),
            Err(other) => panic!("{stem}: {other}"),
        };
        assert_eq!(got, code_of(&stem), "{stem}");
        read += 1;
    }
    assert!(read >= 7, "the corpus was read ({read} files)");
}

/// The shipped 3D packs' shells files are valid, and a pack without one is valid too.
#[test]
fn the_shipped_packs_shells_are_accepted() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../presentation");
    for pack in ["mineworld-default/3D", "mineworld-greybox/3D"] {
        let dir = root.join(pack);
        assert!(
            dir.join(SHELLS_FILE).is_file(),
            "{pack} ships {SHELLS_FILE}"
        );
        shells::check_pack(&dir).unwrap_or_else(|error| panic!("{pack}: {error}"));
    }
    shells::check_pack(&corpus()).expect("a folder without places/shells.yaml is valid");
}
