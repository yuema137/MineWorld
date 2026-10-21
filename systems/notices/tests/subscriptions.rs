//! Subscriptions at genesis and after: tag defaults and authored sections (AC-b-7, F-S31b-3, M-b-5),
//! the holder-only record (I-b-5, M-b-4), every offer incomplete (I-b-8, M-b-1), and the two refusals
//! of a change that changes nothing.

mod support;

use mineworld_contracts::{ActionResult, Rejection};
use mineworld_kernel::KernelError;
use support::{Cafe, Recipe, record, records_of};

fn named(result: &ActionResult) -> String {
    match result {
        ActionResult::Rejected(Rejection::System { code, .. }) => code.as_str().to_owned(),
        other => format!("{other:?}"),
    }
}

fn baristas<'a>() -> Recipe<'a> {
    Recipe {
        configuration: Some(
            "topics:\n  prices.ingredients:\n    body: prices\n    publishers: [pia]\n    \
             valid_for: { min: \"1h\", max: \"48h\" }\n    per_day: 4\n  news.daily:\n    \
             body: text\n    publishers: [pia]\n    valid_for: { min: \"1h\", max: \"7d\" }\n    \
             per_day: 2\ndefaults:\n  by_tag: { barista: [prices.ingredients] }\n",
        ),
        sections: vec![("sue", "subscribe: []"), ("nat", "subscribe: [news.daily]")],
        tags: vec![("sam", "barista"), ("sue", "barista"), ("nat", "barista")],
    }
}

#[test]
fn defaults_apply_by_tag_and_an_authored_section_replaces_them() {
    let cafe = Cafe::try_begin(&baristas()).expect("genesis");
    assert_eq!(
        cafe.subscriptions("sam"),
        ["prices.ingredients"],
        "a barista without a section gets the default"
    );
    assert!(
        cafe.subscriptions("sue").is_empty(),
        "subscribe: [] means none, and replaces the default"
    );
    assert_eq!(
        cafe.subscriptions("nat"),
        ["news.daily"],
        "an authored section replaces, never merges"
    );
    assert!(
        cafe.subscriptions("pia").is_empty(),
        "pia has no tag and no section"
    );
    assert!(
        cafe.genesis
            .iter()
            .filter(|fact| fact.event_type().as_str() == "subscriptions-seeded")
            .all(|fact| *fact.visibility() == mineworld_contracts::Visibility::SystemInternal),
        "genesis subscriptions are SystemInternal (Q-S31b-2)"
    );
}

#[test]
fn a_section_naming_an_undeclared_topic_is_refused_at_genesis() {
    let recipe = Recipe {
        sections: vec![("sam", "subscribe: [news.weekly]")],
        ..Recipe::default()
    };
    let refusal = Cafe::try_begin(&recipe).err().expect("refused");
    let text = format!("{refusal:?}");
    assert!(
        matches!(refusal, KernelError::FactRefusedByOwner { .. }) && text.contains("unknown-topic"),
        "{text}"
    );
}

#[test]
fn a_section_without_configuration_is_refused_as_unconfigured() {
    let recipe = Recipe {
        configuration: None,
        ..Recipe::default()
    };
    let refusal = format!("{:?}", Cafe::try_begin(&recipe).err().expect("refused"));
    assert!(refusal.contains("notices-unconfigured"), "{refusal}");
}

#[test]
fn only_the_holder_sees_its_subscriptions() {
    let mut cafe = Cafe::begun();
    let sam = cafe.id("sam");
    let own = cafe.observation("sam", 10);
    let held: mineworld_notices::Subscriptions =
        record(&own, sam, "subscriptions").expect("sam sees his own");
    assert_eq!(held.topics().len(), 2);
    assert_eq!(records_of(&own, "subscriptions"), 1, "and nobody else's");
    let other = cafe.observation("nat", 20);
    assert!(
        record::<mineworld_notices::Subscriptions>(&other, sam, "subscriptions").is_none(),
        "nat, standing beside sam, does not see sam's subscriptions"
    );
}

#[test]
fn every_offer_is_incomplete() {
    let mut cafe = Cafe::begun();
    for name in ["pia", "sam", "sue", "nat"] {
        let observation = cafe.observation(name, 10);
        for affordance in observation.affordances() {
            let kind = affordance.action_type().as_str();
            if ["publish-notice", "subscribe", "unsubscribe"].contains(&kind) {
                assert!(
                    affordance.payload().is_none(),
                    "{name}'s {kind} offer is incomplete (Q-S31a-3)"
                );
            }
        }
        let counted = |kind: &str| {
            observation
                .affordances()
                .iter()
                .filter(|affordance| affordance.action_type().as_str() == kind)
                .count()
        };
        assert!(counted("subscribe") <= 1 && counted("unsubscribe") <= 1);
    }
    let nat = cafe.observation("nat", 20);
    let kinds: Vec<&str> = nat
        .affordances()
        .iter()
        .map(|affordance| affordance.action_type().as_str())
        .collect();
    assert!(kinds.contains(&"subscribe"), "nat may subscribe");
    assert!(!kinds.contains(&"unsubscribe"), "nat holds nothing to drop");
}

#[test]
fn a_change_that_changes_nothing_is_refused_by_name() {
    let mut cafe = Cafe::begun();
    let before = cafe.state();
    let again = cafe.subscribe("sam", "news.daily", 10).expect("dispatches");
    assert_eq!(named(again.result()), "already-subscribed");
    let none = cafe
        .unsubscribe("nat", "news.daily", 11)
        .expect("dispatches");
    assert_eq!(named(none.result()), "not-subscribed");
    let unknown = cafe
        .subscribe("nat", "news.weekly", 12)
        .expect("dispatches");
    assert_eq!(named(unknown.result()), "unknown-topic");
    assert_eq!(cafe.state(), before, "nothing moved");
}
