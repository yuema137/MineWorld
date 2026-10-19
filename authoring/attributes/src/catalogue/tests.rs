use std::path::PathBuf;

use mineworld_authoring::{DataDirectory, DataOrigin, DataSource};

use super::*;
use crate::declaration::AttributesFile;

pub(crate) fn file(
    pack: &str,
    source: DataSource,
    name: &str,
    yaml: &str,
) -> (DataOrigin, DataFile) {
    let decoded: AttributesFile = serde_saphyr::from_str(yaml).expect("decodes");
    (
        DataOrigin {
            pack: pack.to_owned(),
            source,
            directory: DataDirectory::from_static("attributes"),
            path: PathBuf::from(format!("attributes/{name}.yaml")),
        },
        DataFile::Attributes(decoded),
    )
}

const TOWN: &str = "format: 1
attributes:
  stature:
    group: appearance
    kind: { quantity: { unit: m, scale: 3, min: 0.500, max: 2.500 } }
    visibility: perceivable
    mutability: fixed
    label: Stature
    phrase: \"{value} m tall\"
    perceived_as: { bands: [ { below: 1.55, words: short }, { below: 1.85, words: of average height }, { words: tall } ] }
  hair-colour:
    group: appearance
    kind: { category: [brown, auburn, grey] }
    visibility: perceivable
    mutability: evolving
    label: Hair colour
    phrase: \"{value} hair\"
";

fn town() -> Catalogue {
    Catalogue::build(&[file("town", DataSource::World, "town", TOWN)]).expect("builds")
}

#[test]
fn declarations_take_their_files_namespace_and_resolve_by_short_slug() {
    let catalogue = town();
    assert_eq!(catalogue.len(), 2);
    assert_eq!(
        catalogue.resolve("stature").expect("short").id.as_str(),
        "town.stature"
    );
    assert_eq!(
        catalogue
            .resolve("town.hair-colour")
            .expect("qualified")
            .label,
        "Hair colour"
    );
    let unknown = catalogue.resolve("statur").expect_err("unknown");
    assert!(
        matches!(&unknown, AttributeError::UnknownAttribute { nearest, .. } if nearest[0] == "town.stature"),
        "{unknown}"
    );
}

#[test]
fn a_duplicate_a_foreign_namespace_and_an_ambiguous_slug_are_refused_by_name() {
    let twice = Catalogue::build(&[
        file("town", DataSource::World, "a", TOWN),
        file("town", DataSource::World, "b", TOWN),
    ])
    .expect_err("declared twice");
    assert!(
        matches!(twice, AttributeError::DuplicateAttribute { .. }),
        "{twice}"
    );

    let foreign = "format: 1\nattributes:\n  core.stature: { group: appearance, kind: flag, visibility: perceivable, mutability: fixed, label: X }\n";
    let refused = Catalogue::build(&[file("town", DataSource::World, "f", foreign)])
        .expect_err("core is not town's");
    assert!(
        matches!(refused, AttributeError::ForeignNamespace { .. }),
        "{refused}"
    );

    let other = TOWN.replace("stature:", "stature-x:");
    let catalogue = Catalogue::build(&[
        file("town", DataSource::World, "a", TOWN),
        file("harbour", DataSource::EntityPack, "b", &other),
    ])
    .expect("builds");
    let ambiguous = catalogue
        .resolve("hair-colour")
        .expect_err("two catalogues");
    assert!(
        matches!(ambiguous, AttributeError::AmbiguousSlug { .. }),
        "{ambiguous}"
    );
}

#[test]
fn overrides_follow_the_table_both_ways() {
    let narrowed = "format: 1\noverrides:\n  harbour.hair-colour: { words: { auburn: copper }, visibility: private }\n";
    let catalogue = Catalogue::build(&[
        file("harbour", DataSource::EntityPack, "a", &TOWN),
        file("town", DataSource::World, "o", narrowed),
    ])
    .expect("builds");
    let hair = catalogue.resolve("harbour.hair-colour").expect("declared");
    assert_eq!(hair.visibility, Visibility::Private);
    assert_eq!(
        hair.words_of(&ValueSlug::new("auburn").expect("a slug")),
        "copper"
    );

    for (field, value) in [
        ("kind", "flag"),
        ("mutability", "fixed"),
        ("owner", "economy"),
    ] {
        let refused =
            format!("format: 1\noverrides:\n  harbour.hair-colour: {{ {field}: {value} }}\n");
        let error = Catalogue::build(&[
            file("harbour", DataSource::EntityPack, "a", TOWN),
            file("town", DataSource::World, "o", &refused),
        ])
        .expect_err("not allowed");
        assert!(
            matches!(&error, AttributeError::OverrideNotAllowed { field: named, .. } if named == field),
            "{error}"
        );
    }
    let widened = "format: 1\noverrides:\n  harbour.stature: { visibility: perceivable }\n";
    let private = TOWN.replace(
        "visibility: perceivable\n    mutability: fixed",
        "visibility: private\n    mutability: fixed",
    );
    let error = Catalogue::build(&[
        file("harbour", DataSource::EntityPack, "a", &private),
        file("town", DataSource::World, "o", widened),
    ])
    .expect_err("widened");
    assert!(
        matches!(error, AttributeError::OverrideNotAllowed { .. }),
        "{error}"
    );
}

#[test]
fn an_owner_other_than_attributes_is_the_shipping_packs_own() {
    let owned = "format: 1\nattributes:\n  badge: { group: background, kind: { category: [gold] }, visibility: perceivable, mutability: fixed, owner: badges, label: Badge }\n";
    let catalogue =
        Catalogue::build(&[file("badges", DataSource::SystemPack, "b", owned)]).expect("its own");
    assert_eq!(
        catalogue.resolve("badge").expect("declared").owner.as_str(),
        "badges"
    );

    let in_world = owned.replace("badge:", "medal:");
    let error = Catalogue::build(&[file("town", DataSource::World, "w", &in_world)])
        .expect_err("not installed");
    assert!(
        matches!(error, AttributeError::OwnerNotInstalled { .. }),
        "{error}"
    );
    let error = Catalogue::build(&[
        file("badges", DataSource::SystemPack, "b", owned),
        file("town", DataSource::World, "w", &in_world),
    ])
    .expect_err("only badges declares its own");
    assert!(
        matches!(&error, AttributeError::BadDeclaration { field, .. } if field == "owner"),
        "{error}"
    );
}

#[test]
fn uses_are_checked_against_the_catalogue_naming_the_pack() {
    let reads = |uses: &str| {
        let shipped = format!("format: 1\nuses:\n{uses}");
        Catalogue::build(&[
            file("town", DataSource::World, "t", TOWN),
            file("reader", DataSource::SystemPack, "r", &shipped),
        ])
    };
    reads("  reads: { town.stature: quantity }\n  proposes: [town.hair-colour]\n")
        .expect("held and of its kind");
    let missing = reads("  reads: { town.weight: quantity }\n").expect_err("missing");
    assert!(
        matches!(missing, AttributeError::Missing { .. }),
        "{missing}"
    );
    let wrong = reads("  reads: { town.stature: level }\n").expect_err("wrong kind");
    assert!(matches!(wrong, AttributeError::WrongRead { .. }), "{wrong}");
    let fixed = reads("  proposes: [town.stature]\n").expect_err("fixed");
    assert!(
        matches!(&fixed, AttributeError::BadDeclaration { field, .. } if field == "proposes"),
        "{fixed}"
    );

    let in_world = Catalogue::build(&[file(
        "town",
        DataSource::World,
        "t",
        &format!("{TOWN}uses:\n  proposes: [town.hair-colour]\n"),
    )])
    .expect_err("a world declares no uses");
    assert!(
        matches!(&in_world, AttributeError::BadDeclaration { field, .. } if field == "uses"),
        "{in_world}"
    );
}

#[test]
fn a_bad_field_is_refused_at_its_line() {
    let phrase = TOWN.replace("\"{value} hair\"", "\"{value} {colour}\"");
    let refused = serde_saphyr::from_str::<AttributesFile>(&phrase).expect_err("two placeholders");
    let message = refused.to_string();
    assert!(
        message.contains("BadDeclaration") && message.contains("line 17"),
        "{message}"
    );
}
