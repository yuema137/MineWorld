use mineworld_authoring::DataSource;

use super::*;
use crate::catalogue::tests::file;
use crate::value::{LevelValue, ScaledInt};

const FIXTURE: &str = "format: 1
attributes:
  stature:
    group: appearance
    kind: { quantity: { unit: m, scale: 3, min: 0.500, max: 2.500 } }
    visibility: perceivable
    mutability: fixed
    label: Stature
    phrase: \"{value} m tall\"
    perceived_as: { bands: [ { below: 1.55, words: short }, { below: 1.85, words: of average height }, { words: tall } ] }
  chronotype:
    group: behaviour
    kind: { level: [early-bird, neutral, night-owl] }
    visibility: learnable
    mutability: evolving
    label: Chronotype
  languages:
    group: background
    kind: { set: { of: [english, spanish, tagalog], max: 3 } }
    visibility: perceivable
    mutability: fixed
    label: Languages
    phrase: \"speaks {value}\"
    words: { english: English, spanish: Spanish, tagalog: Tagalog }
  glasses:
    group: appearance
    kind: flag
    visibility: perceivable
    mutability: fixed
    label: Glasses
    phrase: \"wears {value}\"
  secret:
    group: psychology
    kind: { level: [low, high] }
    visibility: hidden
    mutability: fixed
    label: Secret
";

fn catalogue() -> Catalogue {
    Catalogue::build(&[file("town", DataSource::World, "t", FIXTURE)]).expect("builds")
}

fn values(id: &str) -> Option<AttributeValue> {
    let slug = |text: &str| ValueSlug::new(text).expect("a slug");
    Some(match id {
        "town.stature" => AttributeValue::Quantity(ScaledInt::new(1660, 3)),
        "town.chronotype" => AttributeValue::Level(LevelValue {
            slug: slug("early-bird"),
            rank: 0,
        }),
        "town.languages" => {
            AttributeValue::Set([slug("tagalog"), slug("english"), slug("spanish")].into())
        }
        "town.glasses" => AttributeValue::Flag(false),
        "town.secret" => AttributeValue::Level(LevelValue {
            slug: slug("high"),
            rank: 1,
        }),
        _ => return None,
    })
}

fn texts(view: View) -> Vec<(String, Option<String>, bool)> {
    let catalogue = catalogue();
    disclose(
        &catalogue,
        &crate::ATTRIBUTES,
        |declaration| values(declaration.id.as_str()),
        view,
        None,
    )
    .expect("something to disclose")
    .entries
    .into_iter()
    .map(|entry| {
        (
            entry.attribute.to_string(),
            entry.text,
            entry.value.is_some(),
        )
    })
    .collect()
}

/// M-c-4's test: the holder sees every value but the hidden one, exact; another observer the
/// perceivable ones, `stature` as its band's words with no value. Goldens written by hand.
#[test]
fn the_holder_and_another_observer_see_what_their_view_admits() {
    let words = |text: &str| Some(text.to_owned());
    assert_eq!(
        texts(View::Holder),
        [
            ("town.chronotype".to_owned(), words("early bird"), true),
            ("town.glasses".to_owned(), None, true),
            (
                "town.languages".to_owned(),
                words("speaks English, Spanish and Tagalog"),
                true
            ),
            ("town.stature".to_owned(), words("1.660 m tall"), true),
        ]
    );
    assert_eq!(
        texts(View::Other),
        [
            ("town.glasses".to_owned(), None, true),
            (
                "town.languages".to_owned(),
                words("speaks English, Spanish and Tagalog"),
                true
            ),
            ("town.stature".to_owned(), words("of average height"), false),
        ]
    );
}

#[test]
fn a_long_text_is_cut_at_a_character_boundary() {
    let long = "é".repeat(100);
    let cut = bounded(long);
    assert!(cut.len() <= TEXT_BOUND && cut.ends_with('…'), "{cut}");
}

const RANKED: &str = "format: 1
attributes:
  alpha:
    group: appearance
    facets: [prompt, observed]
    kind: { category: [one, two] }
    visibility: perceivable
    mutability: fixed
    label: Alpha
    phrase: \"alpha {value}\"
    prompt_rank: 30
  beta:
    group: appearance
    kind: { category: [one, two] }
    visibility: perceivable
    mutability: fixed
    label: Beta
    phrase: \"beta {value}\"
  gamma:
    group: appearance
    facets: [observed]
    kind: { category: [one, two] }
    visibility: perceivable
    mutability: fixed
    label: Gamma
    phrase: \"gamma {value}\"
    prompt_rank: 10
  born:
    group: background
    kind: instant
    visibility: learnable
    mutability: fixed
    label: Born
    phrase: \"born {value}\"
";

/// M-f-3's test: schema 2 (now 3) carries label, sorted facets and rank, and orders entries by
/// `(rank, attribute)` with an absent rank last; an instant renders no text (Q-S30f-4).
#[test]
fn schema_two_entries_carry_label_facets_and_rank_in_rank_order() {
    let catalogue =
        Catalogue::build(&[file("town", DataSource::World, "t", RANKED)]).expect("builds");
    let one = ValueSlug::new("one").expect("a slug");
    let block = disclose(
        &catalogue,
        &crate::ATTRIBUTES,
        |declaration| {
            Some(if declaration.id.as_str() == "town.born" {
                AttributeValue::Instant(mineworld_contracts::WorldTime::from_seconds(-5))
            } else {
                AttributeValue::Category(one.clone())
            })
        },
        View::Holder,
        None,
    )
    .expect("a block");
    assert_eq!(block.schema, 3);
    assert!(block.words.is_empty(), "no locale offered, no words");
    let order: Vec<&str> = block
        .entries
        .iter()
        .map(|entry| entry.attribute.as_str())
        .collect();
    assert_eq!(
        order,
        ["town.gamma", "town.alpha", "town.beta", "town.born"]
    );
    let alpha = &block.entries[1];
    assert_eq!(alpha.label, "Alpha");
    assert_eq!(
        alpha.facets.iter().map(String::as_str).collect::<Vec<_>>(),
        ["observed", "prompt"]
    );
    assert_eq!(alpha.rank, Some(30));
    let born = &block.entries[3];
    assert_eq!((born.text.as_deref(), born.value.is_some()), (None, true));
    let wire = serde_json::to_value(alpha).expect("encodes");
    assert_eq!(
        wire,
        serde_json::json!({ "attribute": "town.alpha", "kind": "category", "label": "Alpha",
            "facets": ["observed", "prompt"], "rank": 30, "value": { "category": "one" },
            "text": "alpha one" })
    );
}

const COUNTED: &str = "format: 1
attributes:
  children:
    group: background
    kind: { quantity: { unit: count, scale: 0, min: 0, max: 20 } }
    visibility: learnable
    mutability: evolving
    label: Children
    phrase: \"{value} children\"
    phrase_one: \"{value} child\"
";

/// A quantity of exactly one unit takes its `phrase_one`; every other amount takes `phrase`.
#[test]
fn a_quantity_of_one_unit_takes_its_singular_phrase() {
    let catalogue =
        Catalogue::build(&[file("town", DataSource::World, "t", COUNTED)]).expect("builds");
    let declaration = catalogue.resolve("town.children").expect("declared");
    let rendered = |units: i64| {
        phrase(
            declaration,
            &AttributeValue::Quantity(ScaledInt::new(units, 0)),
            &units.to_string(),
        )
    };
    assert_eq!(rendered(1).as_deref(), Some("1 child"));
    assert_eq!(rendered(0).as_deref(), Some("0 children"));
    assert_eq!(rendered(2).as_deref(), Some("2 children"));
}

/// A test-only `fr` label pack for [`FIXTURE`]: `languages` lacks the word for `tagalog`.
const FR: &str = "format: 1
locale: fr
words: { list: \", \", and: \" et \", join: \", \", more: \" …\", pair: \" : \", \"yes\": oui, \"no\": non }
labels:
  town.stature: { label: Taille, phrase: \"mesure {value} m\", perceived_as: [petit, de taille moyenne, grand] }
  town.chronotype: { label: Chronotype, phrase: \"{value}\", words: { early-bird: lève-tôt, neutral: neutre, night-owl: couche-tard } }
  town.languages: { label: Langues, phrase: \"parle {value}\", words: { english: anglais, spanish: espagnol } }
  town.glasses: { label: Lunettes, flag: porte des lunettes }
";

fn fr_labels(catalogue: &Catalogue) -> Labels {
    let origin = mineworld_authoring::DataOrigin {
        pack: "town".to_owned(),
        source: DataSource::World,
        directory: mineworld_authoring::DataDirectory::from_static("labels"),
        path: std::path::PathBuf::from("labels/fr.yaml"),
    };
    let file = serde_saphyr::from_str(FR).expect("decodes");
    Labels::build(vec![(origin, file)], catalogue).expect("builds")
}

/// `(attribute, English text, fr label, fr text)` of each entry `view` is disclosed.
type Localised4 = (String, Option<String>, Option<String>, Option<String>);

fn localised_texts(view: View, value_of: fn(&str) -> Option<AttributeValue>) -> Vec<Localised4> {
    let catalogue = catalogue();
    let labels = fr_labels(&catalogue);
    let fr = Locale::canonical("fr").expect("a locale");
    let block = disclose(
        &catalogue,
        &crate::ATTRIBUTES,
        |declaration| value_of(declaration.id.as_str()),
        view,
        Some(&labels),
    )
    .expect("a block");
    assert_eq!(
        block.words.keys().map(Locale::as_str).collect::<Vec<_>>(),
        ["fr"],
        "the block carries the offered locale's reader words"
    );
    block
        .entries
        .into_iter()
        .map(|entry| {
            (
                entry.attribute.to_string(),
                entry.text,
                entry.labels.get(&fr).cloned(),
                entry.texts.get(&fr).cloned(),
            )
        })
        .collect()
}

/// §5.4 per kind, M-k-2 and M-k-3's test (goldens by hand): a level and a band take the locale's
/// words; a quantity keeps its digits in the locale's phrase; a set missing one member's word is not
/// localised at all (never a slug, never half French); a label reaches the holder's view only; English
/// is unchanged.
#[test]
fn every_offered_locale_is_rendered_whole_or_not_at_all() {
    let some = |text: &str| Some(text.to_owned());
    assert_eq!(
        localised_texts(View::Holder, values),
        [
            (
                "town.chronotype".to_owned(),
                some("early bird"),
                some("Chronotype"),
                some("lève-tôt")
            ),
            ("town.glasses".to_owned(), None, some("Lunettes"), None),
            (
                "town.languages".to_owned(),
                some("speaks English, Spanish and Tagalog"),
                None,
                None
            ),
            (
                "town.stature".to_owned(),
                some("1.660 m tall"),
                some("Taille"),
                some("mesure 1.660 m")
            ),
        ]
    );
    assert_eq!(
        localised_texts(View::Other, values),
        [
            ("town.glasses".to_owned(), None, None, None),
            (
                "town.languages".to_owned(),
                some("speaks English, Spanish and Tagalog"),
                None,
                None
            ),
            (
                "town.stature".to_owned(),
                some("of average height"),
                None,
                some("de taille moyenne")
            ),
        ]
    );
    // a set whose every member has its word joins with the locale's own `list` and `and`; a true
    // flag takes the locale's `flag` words
    let others = |id: &str| -> Option<AttributeValue> {
        let slug = |text: &str| ValueSlug::new(text).expect("a slug");
        match id {
            "town.languages" => Some(AttributeValue::Set(
                [slug("english"), slug("spanish")].into(),
            )),
            "town.glasses" => Some(AttributeValue::Flag(true)),
            _ => None,
        }
    };
    assert_eq!(
        localised_texts(View::Other, others),
        [
            (
                "town.glasses".to_owned(),
                some("wears glasses"),
                None,
                some("porte des lunettes")
            ),
            (
                "town.languages".to_owned(),
                some("speaks English and Spanish"),
                None,
                some("parle anglais et espagnol")
            ),
        ]
    );
}

/// A one-unit phrase is for a quantity only: a flag declaring one is refused at the catalogue.
#[test]
fn a_one_unit_phrase_on_a_non_quantity_is_refused() {
    let text = COUNTED.replace(
        "kind: { quantity: { unit: count, scale: 0, min: 0, max: 20 } }",
        "kind: flag",
    );
    assert!(Catalogue::build(&[file("town", DataSource::World, "t", &text)]).is_err());
}
