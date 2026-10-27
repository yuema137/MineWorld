//! A 3D Presentation Pack's place assets (`assets/places/<id>/`; S14 16f-d1, `PACKAGE_FORMAT.md` §3):
//! every refusal by its code and the detail that names the field, over the corpus
//! `tests/fixtures/place_asset/`, and the default pack's exported café accepted (AC-16f-11).
//!
//! Each corpus case is a small pack folder holding one asset, `kiosk`. Its name is a code in lower case
//! with `-` for `_`, without the `ASSET_` prefix (`unlicensed` → `ASSET_UNLICENSED`), optionally followed
//! by `-<variant>`; `valid` must pass.

use std::path::{Path, PathBuf};

use mineworld_packages::PackageError;
use mineworld_packages::place_asset::{self, AssetCode};

fn corpus(case: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/place_asset")
        .join(case)
}

#[test]
fn each_planted_asset_fault_is_refused_by_its_code_and_names_the_field() {
    let rows: [(&str, AssetCode, &[&str]); 4] = [
        (
            "malformed",
            AssetCode::Malformed,
            &["type prop_visual", "place_visual"],
        ),
        // the adversarial criterion of 16f-d1: an asset with no licence is refused by name
        (
            "unlicensed",
            AssetCode::Unlicensed,
            &["license.spdx", "SPDX"],
        ),
        (
            "reference",
            AssetCode::Reference,
            &["semantic_bindings.entrances", "back_door"],
        ),
        (
            "reference-recipe",
            AssetCode::Reference,
            &["m_00112233aabb", "materials/material_rules.yaml"],
        ),
    ];
    for code in AssetCode::ALL {
        assert!(
            rows.iter().any(|(_, c, _)| *c == code),
            "no corpus case plants {code}"
        );
    }
    for (case, code, words) in rows {
        let Err(PackageError::PlaceAsset {
            code: got, detail, ..
        }) = place_asset::check_pack(&corpus(case))
        else {
            panic!("{case} is not refused as a place asset");
        };
        assert_eq!(got, code, "{case}: {detail}");
        for word in words {
            assert!(
                detail.contains(word),
                "{case}: the detail names {word}: {detail}"
            );
        }
    }
}

#[test]
fn a_valid_asset_is_read_with_its_entrances() {
    let assets = place_asset::check_pack(&corpus("valid")).expect("the valid case is read");
    assert_eq!(assets.ids().collect::<Vec<_>>(), ["kiosk"]);
    assert_eq!(
        assets.entrances("kiosk"),
        Some(["front_door".to_owned()].as_slice())
    );
}

/// AC-16f-11: the café the export tool wrote into the default pack is a valid set of place assets, each
/// with the entrance its depiction anchors on.
#[test]
fn the_default_pack_carries_social_cafes_place_art() {
    let pack = Path::new(env!("CARGO_MANIFEST_DIR")).join("../presentation/mineworld-default/3D");
    let assets = place_asset::check_pack(&pack).expect("the default pack's place assets are valid");
    let ids: Vec<&str> = assets.ids().collect();
    assert_eq!(
        ids,
        [
            "social-cafe-cafe",
            "social-cafe-florist",
            "social-cafe-north-frontage",
            "social-cafe-south-frontage",
            "social-cafe-street",
        ]
    );
    // the station is the pack's station kit, drawn at run time (S29-d, `station_kit`), not place art
    for enterable in ["social-cafe-cafe", "social-cafe-florist"] {
        let entrances = assets.entrances(enterable).expect("listed");
        assert!(
            entrances.iter().any(|e| e == "front_door"),
            "{enterable} has a front_door entrance: {entrances:?}"
        );
    }
}
