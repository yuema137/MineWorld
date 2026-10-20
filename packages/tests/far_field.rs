//! A Presentation Pack's far field: every refusal of `MODULE_SPEC.md` §6.1.1 by its code and the
//! detail that names what is wrong, over the shared corpus `tests/fixtures/far_field/` that the
//! clients' GDScript validator also reads (`ARC-102`; RL-e0 AC-FF-1, AC-FF-3).
//!
//! The corpus is named so either validator can tell what each file must give: a file's stem is a
//! code in lower case with `-` for `_` (`scope-mismatch.yaml` → `FAR_FIELD_SCOPE_MISMATCH`),
//! optionally followed by `-<variant>`; `valid-*` files must pass. Stems ending `-2d` are read as a
//! 2D pack's, all others as a 3D pack's.

use std::path::{Path, PathBuf};

use mineworld_packages::far_field::{self, Sector};
use mineworld_packages::{Dimension, FarFieldCode, PackageError};

fn corpus() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/far_field")
}

fn dimensions_of(stem: &str) -> &'static [Dimension] {
    if stem.ends_with("-2d") {
        &[Dimension::TwoD]
    } else {
        &[Dimension::ThreeD]
    }
}

fn read(stem: &str) -> Result<far_field::FarField, PackageError> {
    far_field::read(&corpus().join(format!("{stem}.yaml")), dimensions_of(stem))
}

/// The code a corpus file must give, from its stem.
fn expected(stem: &str) -> Option<FarFieldCode> {
    if stem.starts_with("valid-") {
        return None;
    }
    FarFieldCode::ALL
        .into_iter()
        .filter(|code| {
            let name = code
                .as_str()
                .trim_start_matches("FAR_FIELD_")
                .to_lowercase()
                .replace('_', "-");
            stem == name || stem.starts_with(&format!("{name}-"))
        })
        .max_by_key(|code| code.as_str().len())
        .or_else(|| panic!("{stem}: no code matches this fixture's name"))
}

/// One row per refusal: the fixture, its code, and words its detail must carry. The detail is
/// what tells an author which field to fix, so it is asserted, not only the code.
#[test]
fn every_planted_far_field_is_refused_by_its_code_and_names_the_fault() {
    let rows: [(&str, FarFieldCode, &[&str]); 17] = [
        ("malformed", FarFieldCode::Malformed, &["colour_grade"]),
        ("malformed-enum", FarFieldCode::Malformed, &["paved_path"]),
        (
            "scope-mismatch",
            FarFieldCode::ScopeMismatch,
            &["social-cafe.yaml", "scope-mismatch.yaml"],
        ),
        (
            "section-missing",
            FarFieldCode::SectionMissing,
            &["skyline", "3d"],
        ),
        (
            "section-foreign",
            FarFieldCode::SectionForeign,
            &["surround", "2d"],
        ),
        ("gap", FarFieldCode::Gap, &["[175.0, 185.0)"]),
        ("gap-half-degree", FarFieldCode::Gap, &["[179.5, 180.0)"]),
        ("gap-2d", FarFieldCode::Gap, &["surround.extent_m 0"]),
        (
            "inside-boundary",
            FarFieldCode::InsideBoundary,
            &["water harbour shore_m 50", "60"],
        ),
        (
            "profile",
            FarFieldCode::Profile,
            &["silhouette north", "increase"],
        ),
        ("out-of-range", FarFieldCode::OutOfRange, &["datum_m 6000"]),
        ("uncited", FarFieldCode::Uncited, &["silhouette north"]),
        ("slot", FarFieldCode::Slot, &["'museum' is not a slot"]),
        // S29-c: the filled slot (ARC-107).
        (
            "slot-town",
            FarFieldCode::SlotTown,
            &["only station is filled"],
        ),
        (
            "slot-filled",
            FarFieldCode::Slot,
            &["a filled slot states place"],
        ),
        (
            "out-of-range-filled",
            FarFieldCode::OutOfRange,
            &["slot landmark footprint_m 0"],
        ),
        ("uncited-filled", FarFieldCode::Uncited, &["slot station"]),
    ];
    for (stem, code, needles) in rows {
        let refusal = read(stem).expect_err(stem);
        let PackageError::FarField {
            code: got, path, ..
        } = &refusal
        else {
            panic!("{stem}: not a far-field refusal: {refusal}");
        };
        assert_eq!(*got, code, "{stem}: {refusal}");
        assert_eq!(
            expected(stem),
            Some(code),
            "{stem}: the name and the row disagree"
        );
        let message = refusal.to_string();
        assert!(message.starts_with(code.as_str()), "{message}");
        assert!(path.ends_with(format!("{stem}.yaml")), "{message}");
        for needle in needles {
            assert!(
                message.contains(needle),
                "{stem}: '{needle}' not in: {message}"
            );
        }
    }
}

/// The corpus is complete for the cross-check: every file gives the code its name says (so the
/// GDScript side can be held to the same names), and every file-level code has a file.
#[test]
fn every_corpus_file_gives_the_code_its_name_says() {
    let mut seen = Vec::new();
    for entry in std::fs::read_dir(corpus()).expect("the corpus lists") {
        let path = entry.expect("an entry").path();
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("a stem")
            .to_owned();
        let got = match read(&stem) {
            Ok(_) => None,
            Err(PackageError::FarField { code, .. }) => Some(code),
            Err(other) => panic!("{stem}: {other}"),
        };
        assert_eq!(got, expected(&stem), "{stem}");
        seen.extend(got);
    }
    for code in FarFieldCode::ALL {
        // MISSING is the absence of a file and WORLD_UNKNOWN a client's launch: no file carries them.
        if matches!(code, FarFieldCode::Missing | FarFieldCode::WorldUnknown) {
            continue;
        }
        assert!(seen.contains(&code), "no corpus file gives {code}");
    }
}

#[test]
fn the_shipped_shape_of_a_3d_and_a_2d_far_field_is_accepted() {
    let three = read("valid-3d").expect("the 3D example validates");
    assert_eq!(three.terrain_silhouettes.map(|s| s.len()), Some(7));
    read("valid-2d").expect("the 2D example validates");
    // A pack presenting both dimensions requires both sets of sections: the 3D example lacks the
    // surround and the 2D one the sky.
    let both = [Dimension::TwoD, Dimension::ThreeD];
    for stem in ["valid-3d", "valid-2d"] {
        let refusal =
            far_field::read(&corpus().join(format!("{stem}.yaml")), &both).expect_err(stem);
        assert!(
            refusal.to_string().starts_with("FAR_FIELD_SECTION_MISSING"),
            "{refusal}"
        );
    }
}

fn sector(from_deg: f64, to_deg: f64) -> Sector {
    Sector { from_deg, to_deg }
}

/// The gap arithmetic names the first uncovered run to 0.1°, including one through north.
#[test]
fn the_first_uncovered_interval_is_named_to_a_tenth_of_a_degree() {
    assert_eq!(
        far_field::uncovered(&[sector(0.0, 175.0), sector(185.0, 0.0)]),
        Some((175.0, 185.0))
    );
    assert_eq!(
        far_field::uncovered(&[sector(300.0, 60.0), sector(60.0, 300.0)]),
        None
    );
    assert_eq!(
        far_field::uncovered(&[sector(10.0, 350.0)]),
        Some((350.0, 10.0)),
        "a gap through north is one run"
    );
    assert_eq!(far_field::uncovered(&[]), Some((0.0, 360.0)));
}

/// A pack is refused when it has no `far_field/`, and a world when its file is absent; both name
/// the pack and say what is missing.
#[test]
fn a_missing_far_field_names_the_pack_and_the_world() {
    let pack = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let missing = far_field::require_for_world(&pack, "a-style", "lakeside").expect_err("absent");
    assert_eq!(
        missing,
        PackageError::FarFieldMissing {
            pack: "a-style".to_owned(),
            scope: "lakeside".to_owned()
        }
    );
    let message = missing.to_string();
    assert!(message.starts_with("FAR_FIELD_MISSING"), "{message}");
    assert!(
        message.contains("a-style") && message.contains("far_field/lakeside.yaml"),
        "{message}"
    );
    far_field::require_for_world(&pack, "a-style", "valid-3d").expect("the file is there");

    let none = far_field::check_pack(&corpus(), &[Dimension::ThreeD]).expect_err("no far_field/");
    let message = none.to_string();
    assert!(
        message.starts_with("FAR_FIELD_MISSING") && message.contains("no far_field/"),
        "{message}"
    );
}

/// AC-FF-4: where a world has both a 2D and a 3D far field in the shipped packs, they describe the
/// same water — each body's sector within 1° and its shore within 1 m — so the two clients never
/// disagree about where the bay is. Every shipped file also validates for its own dimension.
#[test]
fn two_d_and_three_d_water_agree() {
    let family = Path::new(env!("CARGO_MANIFEST_DIR")).join("../presentation/mineworld-default");
    let two = far_field::check_pack(&family.join("2D"), &[Dimension::TwoD]).expect("2D validates");
    let three =
        far_field::check_pack(&family.join("3D"), &[Dimension::ThreeD]).expect("3D validates");
    let mut compared = 0;
    for world in three.iter().filter(|world| two.contains(world)) {
        let file = |dimension: &str| {
            family
                .join(dimension)
                .join(format!("far_field/{world}.yaml"))
        };
        let two = far_field::read(&file("2D"), &[Dimension::TwoD]).expect("2D");
        let three = far_field::read(&file("3D"), &[Dimension::ThreeD]).expect("3D");
        assert_eq!(two.water.len(), three.water.len(), "{world}: water bodies");
        for (a, b) in two.water.iter().zip(&three.water) {
            let name = a.name.as_str();
            assert_eq!(name, b.name.as_str(), "{world}");
            let angle = |x: f64, y: f64| (x - y).rem_euclid(360.0).min((y - x).rem_euclid(360.0));
            assert!(
                angle(a.sector.from_deg, b.sector.from_deg) <= 1.0,
                "{world} {name} from"
            );
            assert!(
                angle(a.sector.to_deg, b.sector.to_deg) <= 1.0,
                "{world} {name} to"
            );
            assert!((a.shore_m - b.shore_m).abs() <= 1.0, "{world} {name} shore");
            assert_eq!(a.horizon, b.horizon, "{world} {name} horizon");
        }
        compared += 1;
    }
    assert_eq!(compared, 2, "social-cafe and lakeside have both");
}
