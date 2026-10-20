//! `configure/notices.yaml` and the `notices:` section: every bound is refused while decoding, with a
//! message naming the key (AC-b-7's configuration arms).

use mineworld_notices::{AuthoredNotices, NoticesConfiguration, NoticesSystem};
use mineworld_sdk::SystemPack;

fn decode(text: &str) -> Result<NoticesConfiguration, String> {
    serde_saphyr::from_str::<NoticesConfiguration>(text).map_err(|error| error.to_string())
}

fn topic(body: &str) -> String {
    format!("topics:\n  news.daily:\n{body}")
}

const GOOD: &str = "    body: text\n    publishers: [pia]\n    valid_for: { min: \"1h\", max: \"7d\" }\n    per_day: 2\n";

#[test]
fn the_fixture_decodes() {
    let decoded = decode(&topic(GOOD)).expect("decodes");
    assert_eq!(decoded.topics().len(), 1);
    let topic = decoded.topics().values().next().expect("one");
    assert_eq!(topic.valid_for().min(), 3_600);
    assert_eq!(topic.valid_for().max(), 7 * 86_400);
    // The loader's own path decodes it too.
    serde_saphyr::with_deserializer_from_str(&self::topic(GOOD), |file| {
        NoticesSystem::decode_configuration(file)
    })
    .expect("through the pack's decoder");
}

#[test]
fn each_bound_is_refused_naming_its_key() {
    let cases: Vec<(String, &str)> = vec![
        ("topics: {}\n".to_owned(), "topics declares 1 to 32"),
        (
            "topics:\n  News:\n    body: text\n    publishers: [pia]\n    valid_for: { min: \"1h\", max: \"7d\" }\n    per_day: 2\n".to_owned(),
            "a topic is",
        ),
        (topic(&GOOD.replace("text", "poem")), "unknown variant"),
        (topic(&GOOD.replace("[pia]", "[]")), "publishers lists 1 to 16"),
        (topic(&GOOD.replace("[pia]", "[pia, pia]")), "listed twice"),
        (topic(&GOOD.replace("\"1h\"", "\"30s\"")), "a duration is"),
        (topic(&GOOD.replace("\"7d\"", "\"31d\"")), "a duration is"),
        (topic(&GOOD.replace("\"1h\"", "\"08h\"")), "a duration is"),
        (topic(&GOOD.replace("min: \"1h\", max: \"7d\"", "min: \"8d\", max: \"7d\"")), "valid_for: min"),
        (topic(&GOOD.replace("per_day: 2", "per_day: 0")), "per_day is 1 to 96"),
        (topic(&GOOD.replace("per_day: 2", "per_day: 97")), "per_day is 1 to 96"),
        (topic(&format!("{GOOD}    colour: red\n")), "unknown field"),
        (
            format!("{}defaults:\n  by_tag: {{ staff: [news.weekly] }}\n", topic(GOOD)),
            "defaults.by_tag.staff: unknown topic `news.weekly`",
        ),
        (
            format!("{}defaults:\n  by_tag: {{ staff: [news.daily, news.daily] }}\n", topic(GOOD)),
            "listed twice",
        ),
    ];
    for (text, expected) in cases {
        let refusal = decode(&text).expect_err(&format!("refused: {text}"));
        assert!(
            refusal.contains(expected),
            "{expected:?} not in {refusal:?}"
        );
    }
}

#[test]
fn thirty_three_topics_are_refused() {
    let mut text = String::from("topics:\n");
    for index in 0..33 {
        text.push_str(&format!("  t{index}:\n{GOOD}"));
    }
    let refusal = decode(&text).expect_err("33 topics");
    assert!(refusal.contains("topics declares 1 to 32"), "{refusal}");
}

#[test]
fn the_section_refuses_duplicates_and_unknown_keys() {
    let ok: AuthoredNotices =
        serde_saphyr::from_str("subscribe: [news.daily, prices.ingredients]").expect("decodes");
    assert_eq!(ok.subscribe().len(), 2);
    let empty: AuthoredNotices = serde_saphyr::from_str("subscribe: []").expect("none");
    assert!(empty.subscribe().is_empty());
    for (text, expected) in [
        ("subscribe: [news.daily, news.daily]", "listed twice"),
        ("subscribe: [news.daily]\ncolour: red", "unknown field"),
        ("subscribe: [Bad]", "a topic is"),
    ] {
        let refusal = serde_saphyr::from_str::<AuthoredNotices>(text)
            .expect_err(text)
            .to_string();
        assert!(
            refusal.contains(expected),
            "{expected:?} not in {refusal:?}"
        );
    }
}
