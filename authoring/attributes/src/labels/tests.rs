//! Label packs (§5.2): decoding with line and column, every refusal by name, and the merge.

use std::path::PathBuf;

use mineworld_authoring::{DataDirectory, DataOrigin, DataSource};

use super::*;
use crate::declaration::{AttributesFile, DataFile};

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
  glasses:
    group: appearance
    kind: flag
    visibility: perceivable
    mutability: fixed
    label: Glasses
    phrase: \"wears {value}\"
";

const WORDS: &str = "words: { list: \" ; \", and: \" et \", join: \" / \", more: \" ...\", pair: \" = \", \"yes\": oui, \"no\": non }\n";

fn catalogue() -> Catalogue {
    let decoded: AttributesFile = serde_saphyr::from_str(TOWN).expect("decodes");
    let origin = DataOrigin {
        pack: "town".to_owned(),
        source: DataSource::World,
        directory: DataDirectory::from_static("attributes"),
        path: PathBuf::from("attributes/town.yaml"),
    };
    Catalogue::build(&[(origin, DataFile::Attributes(decoded))]).expect("builds")
}

fn decode(yaml: &str) -> Result<LabelFile, String> {
    serde_saphyr::from_str::<LabelFile>(yaml).map_err(|error| error.to_string())
}

fn label_file(pack: &str, source: DataSource, stem: &str, yaml: &str) -> (DataOrigin, LabelFile) {
    (
        DataOrigin {
            pack: pack.to_owned(),
            source,
            directory: DataDirectory::from_static("labels"),
            path: PathBuf::from(format!("{pack}/labels/{stem}.yaml")),
        },
        decode(yaml).expect("decodes"),
    )
}

fn build(files: Vec<(DataOrigin, LabelFile)>) -> Result<Labels, String> {
    Labels::build(files, &catalogue()).map_err(|refusal| refusal.reason)
}

fn fr() -> Locale {
    Locale::canonical("fr").expect("valid")
}

#[test]
fn a_world_file_overrides_a_pack_field_by_field_and_offers_its_locale() {
    let pack = format!(
        "format: 1\nlocale: fr\n{WORDS}labels:\n  town.hair-colour: {{ label: Cheveux, phrase: \"cheveux {{value}}\", words: {{ brown: bruns, grey: gris }} }}\n"
    );
    let world = "format: 1\nlocale: fr\nlabels:\n  town.hair-colour: { words: { grey: argentés } }\n";
    let labels = build(vec![
        label_file("people", DataSource::EntityPack, "fr", &pack),
        label_file("town", DataSource::World, "fr", world),
    ])
    .expect("merges");
    assert_eq!(labels.locales().collect::<Vec<_>>(), [&fr()]);
    let hair = &labels.of(&fr()).expect("offered").attributes
        [&QualifiedId::new("town.hair-colour").expect("an id")];
    assert_eq!(hair.label.as_deref(), Some("Cheveux"));
    let grey = ValueSlug::new("grey").expect("a slug");
    assert_eq!(hair.words[&grey], "argentés", "the world's word wins");
    assert_eq!(hair.words.len(), 2);
    assert_eq!(
        labels.of(&fr()).expect("offered").words.for_readers().join,
        " / "
    );
}

#[test]
fn two_packs_giving_one_field_are_refused_naming_both_files() {
    let one = format!("format: 1\nlocale: fr\n{WORDS}labels:\n  town.glasses: {{ label: Lunettes }}\n");
    let two = "format: 1\nlocale: fr\nlabels:\n  town.glasses: { label: Verres }\n";
    let refused = build(vec![
        label_file("alpha", DataSource::EntityPack, "fr", &one),
        label_file("beta", DataSource::EntityPack, "fr", two),
    ])
    .expect_err("duplicate");
    assert!(
        refused.starts_with("DuplicateLabel: town.glasses label is given in alpha/labels/fr.yaml and again in beta/labels/fr.yaml"),
        "{refused}"
    );
}

#[test]
fn english_is_the_declaration_and_never_a_label_file() {
    let refused = build(vec![label_file(
        "town",
        DataSource::World,
        "en",
        &format!("format: 1\nlocale: en\n{WORDS}"),
    )])
    .expect_err("en");
    assert!(refused.contains("EnglishIsTheDeclaration"), "{refused}");
    assert!(refused.contains("overrides.yaml"), "{refused}");
}

#[test]
fn the_locale_must_be_canonical_and_the_file_name_s_stem() {
    let refused = build(vec![label_file(
        "town",
        DataSource::World,
        "de",
        &format!("format: 1\nlocale: fr\n{WORDS}"),
    )])
    .expect_err("stem");
    assert!(
        refused.contains("line 2") && refused.contains("BadLabelFile: the locale 'fr' is not the file name's stem 'de'"),
        "{refused}"
    );
    let refused = build(vec![label_file(
        "town",
        DataSource::World,
        "zh-Hans",
        &format!("format: 1\nlocale: zh-Hans\n{WORDS}"),
    )])
    .expect_err("not canonical");
    assert!(refused.contains("BadLabelFile: 'zh-Hans' is not a locale"), "{refused}");
}

#[test]
fn a_locale_without_its_joining_words_is_refused() {
    let refused = build(vec![label_file(
        "town",
        DataSource::World,
        "fr",
        "format: 1\nlocale: fr\nlabels:\n  town.glasses: { label: Lunettes }\n",
    )])
    .expect_err("no words");
    assert!(refused.starts_with("BadLabelFile: no labels/fr.yaml"), "{refused}");
    let partial = decode("format: 1\nlocale: fr\nwords: { list: a, and: b }\n").expect_err("partial");
    assert!(partial.contains("missing field"), "{partial}");
    let unknown = decode(&format!("format: 1\nlocale: fr\n{}", WORDS.replace("list:", "lisst:")))
        .expect_err("unknown key");
    assert!(unknown.contains("lisst"), "{unknown}");
}

#[test]
fn an_unknown_attribute_is_refused_at_its_line_with_the_nearest_ids() {
    let refused = build(vec![label_file(
        "town",
        DataSource::World,
        "fr",
        &format!("format: 1\nlocale: fr\n{WORDS}labels:\n  town.hair-color: {{ label: Cheveux }}\n"),
    )])
    .expect_err("unknown");
    assert!(refused.contains("line 5"), "{refused}");
    assert!(
        refused.contains("UnknownAttribute: 'town.hair-color'") && refused.contains("town.hair-colour"),
        "{refused}"
    );
}

#[test]
fn every_field_rule_is_refused_by_name() {
    let cases = [
        ("town.hair-colour: { words: { blue: bleus } }", "OutOfDomain: 'town.hair-colour' has no value 'blue'"),
        ("town.stature: { words: { tall: grand } }", "BadLabel: 'town.stature' field 'words' is only for a category, level or set"),
        ("town.hair-colour: { flag: oui }", "BadLabel: 'town.hair-colour' field 'flag' is only for a flag"),
        ("town.hair-colour: { perceived_as: [a, b] }", "BadLabel: 'town.hair-colour' field 'perceived_as' is only for an attribute declared with bands"),
        ("town.stature: { perceived_as: [petit, grand] }", "lists 2 band(s), and the declaration has 3"),
    ];
    for (entry, expected) in cases {
        let refused = build(vec![label_file(
            "town",
            DataSource::World,
            "fr",
            &format!("format: 1\nlocale: fr\n{WORDS}labels:\n  {entry}\n"),
        )])
        .expect_err(entry);
        assert!(refused.contains(expected), "{entry}: {refused}");
        assert!(refused.contains("line 5"), "{entry}: {refused}");
    }
    for (entry, expected) in [
        ("town.glasses: { phrase: \"porte des lunettes\" }", "BadLabel: the phrase"),
        ("town.glasses: { phrase: \"{value} {x}\" }", "BadLabel: the phrase"),
        ("town.glasses: { label: \"\" }", "BadLabel: a label string is empty"),
        ("town.glasses: { label: \"a\\tb\" }", "BadLabel"),
        ("town.glasses: { colour: rouge }", "colour"),
    ] {
        let refused = decode(&format!("format: 1\nlocale: fr\n{WORDS}labels:\n  {entry}\n"))
            .expect_err(entry);
        assert!(refused.contains(expected), "{entry}: {refused}");
        assert!(refused.contains("line 5") || refused.contains(":5:"), "{entry}: {refused}");
    }
    let long = "x".repeat(161);
    let refused = decode(&format!("format: 1\nlocale: fr\n{WORDS}labels:\n  town.glasses: {{ label: {long} }}\n"))
        .expect_err("long");
    assert!(refused.contains("over 160 bytes"), "{refused}");
}

#[test]
fn what_a_locale_lacks_is_listed_piece_by_piece() {
    let labels = build(vec![label_file(
        "town",
        DataSource::World,
        "fr",
        &format!("format: 1\nlocale: fr\n{WORDS}labels:\n  town.hair-colour: {{ label: Cheveux, phrase: \"cheveux {{value}}\", words: {{ brown: bruns, grey: gris }} }}\n  town.glasses: {{ label: Lunettes, flag: lunettes }}\n"),
    )])
    .expect("builds");
    let catalogue = catalogue();
    let lacking: Vec<String> = catalogue
        .attributes()
        .flat_map(|declaration| missing(declaration, labels.of(&fr()).and_then(|of| of.attributes.get(&declaration.id))))
        .collect();
    assert_eq!(
        lacking,
        [
            "town.hair-colour words.auburn",
            "town.stature label",
            "town.stature phrase",
            "town.stature perceived_as"
        ]
    );
}
