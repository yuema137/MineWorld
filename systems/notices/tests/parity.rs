//! AC-b2-1: complete event envelopes from independent base 2b4d36fa, before owner adoption.
//! Captured with this scenario and the baseline support module; never regenerated from this branch.
mod support;

use support::{Cafe, Recipe, accepted, text};

#[test]
fn no_attributes_logs_match_pre_adoption_worlds() {
    for (name, topic, tag) in [
        ("w1", "prices.ingredients", "barista"),
        ("w2", "market.daily", "vendor"),
    ] {
        let mut configuration = support::CONFIGURATION.replace("prices.ingredients", topic);
        if name == "w2" {
            configuration.push_str("  weather.warnings:\n    body: text\n    publishers: [nat]\n    valid_for: { min: \"1h\", max: \"1d\" }\n    per_day: 3\n");
        }
        configuration.push_str(&format!("defaults:\n  by_tag: {{ {tag}: [{topic}] }}\n"));
        let mut cafe = Cafe::try_begin(&Recipe {
            configuration: Some(&configuration),
            sections: vec![("sue", "subscribe: []")],
            tags: vec![("sam", tag), ("sue", tag)],
        })
        .expect("genesis");
        accepted(cafe.subscribe("nat", "news.daily", 10), "subscribe");
        accepted(
            cafe.publish("pia", "news.daily", text("open"), 3600, 20),
            "publish news",
        );
        let prices = mineworld_notices::NoticeBody::Prices(vec![cafe.line("bread", 250, 1)]);
        accepted(
            cafe.publish("pia", topic, prices, 3600, 30),
            "publish prices",
        );
        accepted(cafe.unsubscribe("sam", topic, 40), "unsubscribe");
        let expected = match name {
            "w1" => include_str!("fixtures/pre-attributes-w1.json"),
            "w2" => include_str!("fixtures/pre-attributes-w2.json"),
            _ => unreachable!(),
        };
        assert_eq!(
            serde_json::to_string(&cafe.log).expect("log encodes"),
            expected.trim_end(),
            "{name}: every event byte is preserved"
        );
    }
}
