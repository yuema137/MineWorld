//! A Presentation Pack's depiction files: every refusal of `MODULE_SPEC.md` §6.1.2 by its code and
//! the detail that names what is wrong, over the corpus `tests/fixtures/depiction/` that the 3D
//! client's reader also reads (`ARC-177`; S33-d AC-d-10, AC-d-12).
//!
//! The corpus is named so either reader can tell what each file must give: a file's stem is a code
//! in lower case with `-` for `_` (`on-floor.yaml` → `DEPICTION_ON_FLOOR`), optionally followed by
//! `-<variant>`; `valid-*` files must pass. A stem with `-client-` holds a fault only the client can
//! see (a texture slug, a tree form): this validator accepts it.

use std::path::{Path, PathBuf};

use mineworld_packages::depiction::{self, Depiction};
use mineworld_packages::{DepictionCode, PackageError};

fn corpus() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/depiction")
}

fn read(stem: &str) -> Result<Depiction, PackageError> {
    depiction::read(&corpus().join(format!("{stem}.yaml")), None)
}

/// The code a corpus file must give this validator, from its stem.
fn expected(stem: &str) -> Option<DepictionCode> {
    if stem.starts_with("valid-") || stem.contains("-client-") {
        return None;
    }
    DepictionCode::ALL
        .into_iter()
        .filter(|code| {
            let name = code
                .as_str()
                .trim_start_matches("DEPICTION_")
                .to_lowercase()
                .replace('_', "-");
            stem == name || stem.starts_with(&format!("{name}-"))
        })
        .max_by_key(|code| code.as_str().len())
        .or_else(|| panic!("{stem}: no code matches this fixture's name"))
}

fn stems() -> Vec<String> {
    let mut stems: Vec<String> = std::fs::read_dir(corpus())
        .expect("the corpus lists")
        .map(|entry| entry.expect("an entry").path())
        .filter(|path| path.extension().is_some_and(|e| e == "yaml"))
        .map(|path| {
            path.file_stem()
                .expect("a stem")
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    stems.sort();
    stems
}

/// Every corpus file gives the code its name says, and the fault-free ones are read.
#[test]
fn every_corpus_file_gives_the_code_its_name_says() {
    let stems = stems();
    let mut seen = Vec::new();
    for stem in &stems {
        match (expected(stem), read(stem)) {
            (None, Ok(_)) => {}
            (None, Err(error)) => panic!("{stem} must be read: {error}"),
            (Some(code), Ok(_)) => panic!("{stem} must be refused {code}, and was read"),
            (
                Some(code),
                Err(PackageError::Depiction {
                    code: got, detail, ..
                }),
            ) => {
                assert_eq!(got, code, "{stem}: {detail}");
                seen.push(code);
            }
            (Some(code), Err(other)) => panic!("{stem} must be refused {code}: {other}"),
        }
    }
    for code in DepictionCode::ALL {
        assert!(seen.contains(&code), "no corpus file plants {code}");
    }
}

/// One row per refusal: the fixture, its code, and words its detail must carry. The detail is what
/// tells an author which field to fix, so it is asserted, not only the code.
#[test]
fn each_refusal_names_the_field_at_fault() {
    let rows: [(&str, DepictionCode, &[&str]); 10] = [
        // AC-d-12: the file has no field for furniture (ARC-307).
        ("malformed", DepictionCode::Malformed, &["furnishings"]),
        (
            "malformed-height",
            DepictionCode::Malformed,
            &["terrain.regions[0].height", "flat_m"],
        ),
        (
            "scope-mismatch",
            DepictionCode::ScopeMismatch,
            &["lakeside.yaml", "scope-mismatch.yaml"],
        ),
        (
            "uncited",
            DepictionCode::Uncited,
            &["buildings[1]", "cottage", "source"],
        ),
        (
            "out-of-range",
            DepictionCode::OutOfRange,
            &["buildings[0]", "bakery", "5"],
        ),
        (
            "polygon",
            DepictionCode::Polygon,
            &["buildings[1].footprint_mm", "simple"],
        ),
        (
            "reference",
            DepictionCode::Reference,
            &["terrain.regions[1].layer", "sand"],
        ),
        (
            "door",
            DepictionCode::Door,
            &["buildings[1]", "openings[0]", "open"],
        ),
        // AC-d-12, M-d-3: a scatter area on the shore's floor.
        (
            "on-floor",
            DepictionCode::OnFloor,
            &["scatter[0]", "rocks", "shore", "50 mm"],
        ),
        (
            "on-floor-shell",
            DepictionCode::OnFloor,
            &["buildings[0]", "bakery", "500 mm"],
        ),
    ];
    for (stem, code, words) in rows {
        let Err(PackageError::Depiction {
            code: got, detail, ..
        }) = read(stem)
        else {
            panic!("{stem} is not refused as a depiction");
        };
        assert_eq!(got, code, "{stem}: {detail}");
        for word in words {
            assert!(
                detail.contains(word),
                "{stem}: the detail names {word}: {detail}"
            );
        }
    }
}

/// The near water's colours are the far field's: a name the world's far field lacks is refused.
#[test]
fn the_near_water_names_a_far_field_water() {
    let path = corpus().join("valid-full.yaml");
    let waters = ["lake".to_owned()];
    depiction::read(&path, Some(&waters)).expect("the far field has the lake");
    let other = ["sea".to_owned()];
    let Err(PackageError::Depiction { code, detail, .. }) = depiction::read(&path, Some(&other))
    else {
        panic!("a water the far field lacks is refused");
    };
    assert_eq!(code, DepictionCode::Reference, "{detail}");
    assert!(detail.contains("far_field_water"), "{detail}");
}
