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

/// M-f-3's test: schema 2 carries label, sorted facets and rank, and orders entries by
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
    )
    .expect("a block");
    assert_eq!(block.schema, 2);
    let order: Vec<&str> = block
        .entries
        .iter()
        .map(|entry| entry.attribute.as_str())
        .collect();
    assert_eq!(order, ["town.gamma", "town.alpha", "town.beta", "town.born"]);
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
