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
