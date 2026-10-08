//! A code pack's and a world's identity, checked the one way every carrier is (`ARC-53`; step-16
//! §14.2 PD-4, PD-5, PD-7).

use std::path::Path;

use mineworld_packages::{
    Compatibility, Identity, License, PackId, PackType, Package, Version, distinct,
};

/// What `package!()` records in this crate — the real expansion, from this crate's own Cargo fields.
const OURS: Package = mineworld_packages::package!();

#[test]
fn package_records_the_crate_it_is_written_in() {
    assert_eq!(OURS.name(), "mineworld-packages");
    let identity = Identity::of_code_pack(&OURS, PackType::SystemPack).expect("a valid identity");
    assert_eq!(
        identity.version.to_string(),
        mineworld_packages::FRAMEWORK_VERSION
    );
    assert_eq!(identity.license.as_str(), "MIT");
    assert_eq!(identity.authors, ["Yue Ma"]);
}

/// Cargo checks a code pack's name and version; nothing checks its licence or authors but this. Each
/// refusal names the pack and the text.
#[test]
fn a_code_pack_without_a_usable_licence_or_author_is_refused_by_name() {
    for (license, authors, needle) in [
        ("", "Yue Ma", "'' is not an SPDX licence expression"),
        (
            "MIT/Apache-2.0",
            "Yue Ma",
            "'MIT/Apache-2.0' is not an SPDX",
        ),
        ("MIT", "", "no author"),
    ] {
        let package = Package::declared("acme-thing", "0.1.0", license, authors, "");
        let refusal = Identity::of_code_pack(&package, PackType::SystemPack)
            .expect_err(needle)
            .to_string();
        assert!(
            refusal.contains("acme-thing") && refusal.contains(needle),
            "{refusal}"
        );
    }
    let two = Package::declared(
        "acme-thing",
        "0.1.0",
        "MIT OR Apache-2.0",
        "A:B",
        "https://x",
    );
    let identity = Identity::of_code_pack(&two, PackType::SystemPack).expect("an expression");
    assert_eq!(identity.authors, ["A", "B"]);
    assert_eq!(identity.repository.as_deref(), Some("https://x"));
}

/// Every Cargo package name and World Pack id MineWorld ships satisfies the id rule; what it refuses,
/// it refuses saying why.
#[test]
fn the_id_rule_admits_what_ships_and_refuses_the_rest() {
    for id in [
        "mineworld-group-activity",
        "mineworld-rule-controller",
        "social-cafe",
        "bodies-yard",
        "mineworld-default-3d",
        "a",
    ] {
        PackId::new(id).unwrap_or_else(|error| panic!("{id}: {error}"));
    }
    for (id, why) in [
        ("", "1–64"),
        (&"a".repeat(65), "1–64"),
        ("3d", "lowercase letter"),
        ("Cafe", "lowercase letter"),
        ("cafe_town", "only a–z"),
        ("cafe--town", "no '--'"),
        ("cafe-", "no '--'"),
    ] {
        let refusal = PackId::new(id).expect_err(id).to_string();
        assert!(refusal.contains(why), "{id}: {refusal}");
    }
}

/// A world's fields are optional to the loader and required here, the first absent one named.
#[test]
fn a_world_without_its_package_fields_is_refused_naming_the_field() {
    let manifest = Path::new("worlds/x/world.yaml");
    let version = || Some(Version::new("0.1.0").expect("semver"));
    let license = || Some(License::new("MIT").expect("spdx"));
    let range = || Some(Compatibility::new("^0.1").expect("range"));
    let whole = Identity::of_world(manifest, "x", version(), license(), range()).expect("whole");
    assert_eq!(whole.kind, PackType::WorldPack);
    for (identity, field) in [
        (
            Identity::of_world(manifest, "x", None, license(), range()),
            "world.version",
        ),
        (
            Identity::of_world(manifest, "x", version(), None, range()),
            "world.license",
        ),
        (
            Identity::of_world(manifest, "x", version(), license(), None),
            "mineworld",
        ),
    ] {
        let refusal = identity.expect_err(field).to_string();
        assert!(
            refusal.contains("worlds/x/world.yaml") && refusal.contains(field),
            "{refusal}"
        );
    }
}

/// `^0.1` means what it means to Cargo: 0.1.x, nothing else; the framework is refused out of range.
#[test]
fn a_framework_range_admits_what_cargo_admits() {
    let caret = Compatibility::new("^0.1").expect("range");
    for (version, admitted) in [
        ("0.1.0", true),
        ("0.1.9", true),
        ("0.2.0", false),
        ("1.0.0", false),
    ] {
        assert_eq!(
            caret.admits(&Version::new(version).expect("semver")),
            admitted,
            "{version}"
        );
    }
    let refusal = Compatibility::new("^9")
        .expect("range")
        .require_framework()
        .expect_err("out of range")
        .to_string();
    assert!(
        refusal.contains("^9") && refusal.contains(mineworld_packages::FRAMEWORK_VERSION),
        "{refusal}"
    );
}

#[test]
fn two_packs_with_one_id_are_refused_naming_both() {
    let pack = |id: &str| Identity {
        id: PackId::new(id).expect("id"),
        kind: PackType::PresentationPack,
        version: Version::new("0.1.0").expect("semver"),
        license: License::new("MIT").expect("spdx"),
        authors: vec!["A".to_owned()],
        repository: None,
        mineworld: None,
    };
    let (a, b, a_again) = (pack("a"), pack("b"), pack("a"));
    assert!(distinct([(&a, "here"), (&b, "there")]).is_ok());
    let refusal = distinct([(&a, "here"), (&b, "there"), (&a_again, "elsewhere")])
        .expect_err("a twice")
        .to_string();
    assert!(
        refusal.contains("the id a") && refusal.contains("here") && refusal.contains("elsewhere"),
        "{refusal}"
    );
}
