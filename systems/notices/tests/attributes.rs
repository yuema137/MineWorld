//! S31-b2 Part A (`pr-s31b2-notices-attributes.md` §§5.2-5.5): where a world enables `attributes`,
//! `notices` owns `notices.subscriptions`. Handed sets become the same `subscriptions-seeded` fact a
//! tag default is (AC-b2-2, I-b2-2); the vocabulary is the world's own topics (AC-b2-3, M-b2-3); a
//! second source is refused (AC-b2-4, M-b2-1); the holder's record carries the rendered attribute and
//! nobody else's does (AC-b2-5's pack half, M-b2-2); without `attributes` nothing changes (AC-b2-1).
//!
//! W1 is S31-b's scratch café with `defaults.by_tag`; W1-attr the same café with `attributes` enabled
//! and the same mapping handed as roles would resolve it. W2 / W2-attr is a generated variant: topics
//! renamed, the tag renamed, a third topic added.

mod support;

use mineworld_authoring::PackConfiguration;
use mineworld_contracts::{EntityId, EventEnvelope, Visibility};
use mineworld_kernel::KernelError;
use mineworld_notices::{NoticesConfiguration, NoticesSystem, SubscriptionsSeeded};
use serde_json::Value;
use support::{Cafe, Recipe, WithAttributes, catalogue, of, set, text};

/// The topics of W1, then the same topics with the tag default of W1.
const W1_TOPICS: &str = "topics:\n  prices.ingredients:\n    body: prices\n    publishers: [pia]\n    \
     valid_for: { min: \"1h\", max: \"48h\" }\n    per_day: 4\n  news.daily:\n    body: text\n    \
     publishers: [pia]\n    valid_for: { min: \"1h\", max: \"7d\" }\n    per_day: 2\n";
const W1_DEFAULTS: &str = "defaults:\n  by_tag: { barista: [prices.ingredients] }\n";

/// W2: `prices.ingredients` renamed `market.daily`, a third topic, the tag renamed `vendor`.
const W2_TOPICS: &str = "topics:\n  market.daily:\n    body: prices\n    publishers: [pia]\n    \
     valid_for: { min: \"1h\", max: \"48h\" }\n    per_day: 4\n  news.daily:\n    body: text\n    \
     publishers: [pia]\n    valid_for: { min: \"1h\", max: \"7d\" }\n    per_day: 2\n  \
     weather.warnings:\n    body: text\n    publishers: [nat]\n    valid_for: { min: \"1h\", max: \"1d\" }\n    \
     per_day: 3\n";
const W2_DEFAULTS: &str = "defaults:\n  by_tag: { vendor: [market.daily] }\n";

/// A world variant: its topics, its defaults, its tag and the topic its tag defaults to.
struct Variant {
    topics: &'static str,
    defaults: &'static str,
    tag: &'static str,
    topic: &'static str,
}

const W1: Variant = Variant {
    topics: W1_TOPICS,
    defaults: W1_DEFAULTS,
    tag: "barista",
    topic: "prices.ingredients",
};
const W2: Variant = Variant {
    topics: W2_TOPICS,
    defaults: W2_DEFAULTS,
    tag: "vendor",
    topic: "market.daily",
};

/// The two tagged subscribers.
const TAGGED: [&str; 2] = ["sam", "sue"];

/// The variant without `attributes`: tag defaults, no sections.
fn plain(variant: &Variant) -> Cafe {
    let configuration = format!("{}{}", variant.topics, variant.defaults);
    Cafe::try_begin(&Recipe {
        configuration: Some(&configuration),
        sections: Vec::new(),
        tags: TAGGED.iter().map(|person| (*person, variant.tag)).collect(),
    })
    .expect("genesis")
}

/// The variant with `attributes`: no tag defaults, the same mapping handed as roles resolve it.
fn with_attributes(
    variant: &Variant,
    defaults: bool,
    sections: Vec<(&str, &str)>,
) -> Result<Cafe, KernelError> {
    let configuration = if defaults {
        format!("{}{}", variant.topics, variant.defaults)
    } else {
        variant.topics.to_owned()
    };
    let handed = WithAttributes {
        handed: TAGGED
            .iter()
            .map(|person| (*person, set(&[variant.topic])))
            .collect(),
    };
    Cafe::try_begin_with(
        &Recipe {
            configuration: Some(&configuration),
            sections,
            tags: TAGGED.iter().map(|person| (*person, variant.tag)).collect(),
        },
        Some(&handed),
    )
}

/// `(person, topics)` of every `subscriptions-seeded`, in log order.
fn seeded(facts: &[EventEnvelope]) -> Vec<(EntityId, Vec<String>)> {
    of::<SubscriptionsSeeded>(facts)
        .into_iter()
        .map(|(fact, _)| {
            (
                fact.person().entity_id(),
                fact.topics()
                    .iter()
                    .map(|topic| topic.as_str().to_owned())
                    .collect(),
            )
        })
        .collect()
}

/// The named refusal genesis reported.
fn refusal_code(result: Result<Cafe, KernelError>) -> String {
    match result {
        Ok(_) => panic!("genesis should be refused"),
        Err(KernelError::FactRefusedByOwner { reason, .. }) => format!("{reason:?}"),
        Err(other) => panic!("another refusal: {other:?}"),
    }
}

/// The `subscriptions` record `observer` is shown about `subject` at second `at`, raw.
fn subscriptions_record(cafe: &mut Cafe, observer: &str, subject: &str, at: i64) -> Option<Value> {
    let about = cafe.id(subject);
    let observation = cafe.observation(observer, at);
    support::record::<Value>(&observation, about, "subscriptions")
}

/// AC-b2-2 (W1 vs W1-attr, W2 vs W2-attr) and I-b2-2: every person's `Subscriptions` at genesis is
/// equal, the `subscriptions-seeded` facts are equal in person and topics (same type, schema and
/// visibility; a handed one is `origin: default` and caused by the handed fact), and every notice's
/// audience is equal.
#[test]
fn roles_give_the_same_subscriptions_as_tag_defaults() {
    for variant in [W1, W2] {
        let mut by_tag = plain(&variant);
        let mut by_roles = with_attributes(&variant, false, Vec::new()).expect("genesis");
        for person in support::PEOPLE {
            assert_eq!(
                by_tag.subscriptions(person),
                by_roles.subscriptions(person),
                "{person}'s subscriptions in {}",
                variant.topic
            );
        }
        assert_eq!(by_roles.subscriptions("sam"), [variant.topic]);
        assert_eq!(seeded(&by_tag.genesis), seeded(&by_roles.genesis));
        for (fact, envelope) in of::<SubscriptionsSeeded>(&by_roles.genesis) {
            assert_eq!(fact.origin(), mineworld_notices::Origin::Default, "SD-b2-2");
            assert_eq!(*envelope.visibility(), Visibility::SystemInternal);
            assert_eq!(envelope.subjects(), [fact.person().entity_id()]);
            assert_eq!(envelope.provenance().emitted_by().as_str(), "notices");
            let caused = by_roles
                .genesis
                .iter()
                .find(|cause| {
                    matches!(envelope.caused_by(),
                        mineworld_contracts::Causation::Event(id) if *id == cause.id())
                })
                .expect("caused by a fact of the genesis");
            assert_eq!(caused.event_type().as_str(), "attributes-handed");
        }
        for cafe in [&mut by_tag, &mut by_roles] {
            let body = mineworld_notices::NoticeBody::Prices(vec![cafe.line("bread", 250, 1)]);
            support::accepted(
                cafe.publish("pia", variant.topic, body, 3_600, 60),
                "pia publishes prices",
            );
            support::accepted(
                cafe.publish("pia", "news.daily", text("flour is up"), 3_600, 61),
                "pia publishes news",
            );
        }
        let audience = |cafe: &Cafe| {
            cafe.log
                .iter()
                .filter(|fact| fact.event_type().as_str() == "notice-published")
                .map(|fact| fact.visibility().clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(audience(&by_tag), audience(&by_roles));
    }
}

/// AC-b2-3 and M-b2-3 (pack half): the vocabulary `notices.topics` is the world's own topic keys —
/// W2's are not W1's — and a handed value outside them is refused (`unknown-topic`).
#[test]
fn the_vocabulary_is_the_worlds_topics() {
    for (variant, keys) in [
        (W1, vec!["news.daily", "prices.ingredients"]),
        (W2, vec!["market.daily", "news.daily", "weather.warnings"]),
    ] {
        let decoded: NoticesConfiguration =
            serde_saphyr::from_str(variant.topics).expect("the configuration decodes");
        let built = catalogue(&NoticesSystem::yields(&decoded)).expect("the catalogue builds");
        let vocabulary: Vec<&str> = built
            .vocabularies()
            .iter()
            .find(|(id, _)| id.as_str() == "notices.topics")
            .map(|(_, values)| values.iter().map(|value| value.as_str()).collect())
            .expect("notices.topics");
        assert_eq!(vocabulary, keys);
        assert!(
            built
                .attributes()
                .any(|declaration| declaration.id.as_str() == "notices.subscriptions"),
            "the shipped declaration draws from it"
        );
    }

    let stale = WithAttributes {
        handed: vec![("sam", set(&["prices.ingredients"]))],
    };
    let refused = Cafe::try_begin_with(
        &Recipe {
            configuration: Some(W2_TOPICS),
            sections: Vec::new(),
            tags: Vec::new(),
        },
        Some(&stale),
    );
    assert!(refusal_code(refused).contains("unknown-topic"));
}

/// AC-b2-4 and M-b2-1: with `attributes`, `defaults.by_tag` and a `notices:` section are each refused
/// `subscriptions-are-attributes`; without `attributes`, both still load.
#[test]
fn one_source_per_world() {
    let by_tag = refusal_code(with_attributes(&W1, true, Vec::new()));
    assert!(by_tag.contains("subscriptions-are-attributes"), "{by_tag}");
    assert!(
        by_tag.contains("roles"),
        "the detail says what to do: {by_tag}"
    );

    let section = refusal_code(with_attributes(
        &W1,
        false,
        vec![("nat", "subscribe: [news.daily]")],
    ));
    assert!(
        section.contains("subscriptions-are-attributes"),
        "{section}"
    );
    assert!(section.contains("notices.subscriptions"), "{section}");

    let both = format!("{W1_TOPICS}{W1_DEFAULTS}");
    let without = Cafe::try_begin(&Recipe {
        configuration: Some(&both),
        sections: vec![("nat", "subscribe: [news.daily]")],
        tags: TAGGED.iter().map(|person| (*person, "barista")).collect(),
    })
    .expect("without attributes, both sources load");
    assert_eq!(without.subscriptions("nat"), ["news.daily"]);
    assert_eq!(without.subscriptions("sam"), ["prices.ingredients"]);
}

/// A handed value that is not a set is refused `PreconditionFailed`.
#[test]
fn a_handed_value_that_is_not_a_set_is_refused() {
    let wrong = WithAttributes {
        handed: vec![(
            "sam",
            mineworld_attributes_model::AttributeValue::Flag(true),
        )],
    };
    let refused = Cafe::try_begin_with(
        &Recipe {
            configuration: Some(W1_TOPICS),
            sections: Vec::new(),
            tags: Vec::new(),
        },
        Some(&wrong),
    );
    assert!(refusal_code(refused).contains("PreconditionFailed"));
}

/// AC-b2-5 (pack half) and M-b2-2: the holder's `subscriptions` record carries `attributes`, the sorted
/// set rendered with the declaration's phrase; another observer gets no record about them; a runtime
/// `subscribe` changes the projected value; and without `attributes` the record is exactly the
/// component, with no `attributes` field (AC-b2-1, I-b2-4).
#[test]
fn the_holder_sees_the_rendered_attribute_and_nobody_else_sees_anything() {
    let mut cafe = with_attributes(&W2, false, Vec::new()).expect("genesis");
    let own = subscriptions_record(&mut cafe, "sam", "sam", 0).expect("sam's own record");
    assert_eq!(own["topics"], serde_json::json!(["market.daily"]));
    let rendered = own["attributes"].to_string();
    assert!(rendered.contains("follows"), "the phrase: {rendered}");
    assert!(rendered.contains("market.daily"), "the value: {rendered}");
    assert!(
        subscriptions_record(&mut cafe, "nat", "sam", 0).is_none(),
        "another observer is told nothing about sam"
    );

    support::accepted(
        cafe.subscribe("sam", "weather.warnings", 30),
        "sam subscribes",
    );
    let after = subscriptions_record(&mut cafe, "sam", "sam", 31).expect("sam's own record");
    let rendered = after["attributes"].to_string();
    assert!(
        rendered.find("market.daily") < rendered.find("weather.warnings")
            && rendered.contains("weather.warnings"),
        "the projection follows the component, sorted: {rendered}"
    );

    let mut without = plain(&W2);
    let own = subscriptions_record(&mut without, "sam", "sam", 0).expect("sam's own record");
    assert_eq!(own, serde_json::json!({ "topics": ["market.daily"] }));
}
