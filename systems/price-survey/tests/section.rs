//! AC-c-1: the section decodes or is refused naming its key; genesis opens an empty book that only
//! its holder is shown; a category no declared kind has is refused at reduction by name.

mod support;

use mineworld_contracts::Visibility;
use mineworld_kernel::KernelError;
use mineworld_price_survey::SurveyBookOpened;
use support::{Setup, Town, of, section};

#[test]
fn genesis_opens_an_empty_book_with_the_four_settings() {
    let mut town = Town::begun();
    let opened = of::<SurveyBookOpened>(&town.log);
    assert_eq!(opened.len(), 1, "one survey-book-opened, for pia");
    let (fact, envelope) = &opened[0];
    assert_eq!(*envelope.visibility(), Visibility::SystemInternal);
    assert_eq!(fact.surveyor().entity_id(), town.id("pia"));
    let book = town.book("pia", 0).expect("pia sees her own book");
    let categories: Vec<&str> = book.categories().iter().map(|c| c.as_str()).collect();
    assert_eq!(categories, ["drink", "food"], "sorted");
    assert_eq!(book.depth(), 4);
    assert_eq!(book.window(), 86_400);
    assert_eq!(book.resurvey_after(), 3_600);
    assert!(book.kinds().is_empty() && book.surveyed().is_empty());
}

#[test]
fn defaults_are_the_steps_example() {
    let authored = section("{ kinds: { categories: [food] } }").expect("decodes");
    assert_eq!(authored.depth(), 32);
    assert_eq!(authored.window(), 7 * 86_400);
    assert_eq!(authored.resurvey_after(), 3_600);
}

#[test]
fn each_bad_value_is_refused_naming_its_key() {
    let seventeen: Vec<String> = (0..17).map(|n| format!("c{n}")).collect();
    let seventeen = format!("{{ kinds: {{ categories: [{}] }} }}", seventeen.join(", "));
    let cases: [(&str, &str); 12] = [
        (
            "{ kinds: { categories: [] } }",
            "kinds.categories: at least one category",
        ),
        (&seventeen, "kinds.categories: at most 16, not 17"),
        (
            "{ kinds: { categories: [food, food] } }",
            "kinds.categories: `food` is listed twice",
        ),
        ("{ kinds: { categories: [Food] } }", "a category must be"),
        (
            "{ kinds: { categories: [food] }, depth: 0 }",
            "depth: 1 to 64, not 0",
        ),
        (
            "{ kinds: { categories: [food] }, depth: 65 }",
            "depth: 1 to 64, not 65",
        ),
        (
            "{ kinds: { categories: [food] }, window: \"59m\" }",
            "window: 1h to 30d, not `59m`",
        ),
        (
            "{ kinds: { categories: [food] }, window: \"31d\" }",
            "window: 1h to 30d, not `31d`",
        ),
        (
            "{ kinds: { categories: [food] }, window: \"7x\" }",
            "window: a whole number followed by m, h or d, such as \"7d\", not `7x`",
        ),
        (
            "{ kinds: { categories: [food] }, resurvey_after: \"0m\" }",
            "resurvey_after: 1m to 1d, not `0m`",
        ),
        (
            "{ kinds: { categories: [food] }, resurvey_after: \"2d\" }",
            "resurvey_after: 1m to 1d, not `2d`",
        ),
        ("{ kinds: { categories: [food] }, colour: red }", "colour"),
    ];
    for (yaml, expected) in cases {
        let refused = section(yaml).expect_err(yaml).to_string();
        assert!(refused.contains(expected), "{yaml}: {refused}");
    }
}

#[test]
fn a_category_no_kind_has_is_refused_at_genesis_by_name() {
    let refused = Town::build(&Setup::default().pia("{ kinds: { categories: [fod] } }"))
        .err()
        .expect("genesis refuses `fod`");
    let KernelError::FactRefusedByOwner { system, reason, .. } = &refused else {
        panic!("got: {refused:?}");
    };
    assert_eq!(system.as_str(), "price-survey");
    let rendered = format!("{reason:?}");
    assert!(
        rendered.contains("survey-category-unknown") && rendered.contains("fod"),
        "{rendered}"
    );
}

#[test]
fn a_person_without_the_section_has_no_book() {
    let mut town = Town::begun();
    assert!(town.book("nat", 0).is_none());
}
