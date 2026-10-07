//! E-8, consumption eats food and drinks drinks only, over a hand-built café through presence's real
//! `observe` and the kernel's real dispatch: offered complete per edible or drinkable kind carried,
//! never for goods and never at another person; decided here; used up through inventory's
//! constructor; refused with the reason a client can act on.

mod support;

use mineworld_consumption::{Drink, Eat};
use mineworld_contracts::{ActionResult, Affordance, Causation, Event, Rejection, Visibility};
use mineworld_inventory::ItemsConsumed;
use serde_json::Value;
use support::{GENESIS, HOLDINGS, Meal, Town, meals};

fn owned(pairs: &[(&str, u32)]) -> Vec<(String, u32)> {
    pairs.iter().map(|(k, n)| ((*k).to_owned(), *n)).collect()
}

/// (action, item key) per offered meal.
fn offered(town: &Town, observer: &str) -> Vec<(String, String)> {
    let seen = town.observation(observer, GENESIS);
    meals(&seen)
        .into_iter()
        .map(|affordance: &Affordance<Value>| {
            assert!(affordance.target().is_none(), "a meal has no target");
            assert!(
                affordance.is_available(),
                "a meal of what one carries is available"
            );
            let payload = affordance
                .payload()
                .expect("a complete affordance carries its request")
                .clone();
            let item = match affordance.action_type().as_str() {
                "eat" => serde_json::from_value::<Eat>(payload)
                    .expect("an eat")
                    .item(),
                _ => serde_json::from_value::<Drink>(payload)
                    .expect("a drink")
                    .item(),
            };
            let name = ["coffee", "croissant", "mug", "tea"]
                .into_iter()
                .find(|name| town.item(name) == item)
                .expect("a kind of the town");
            (
                affordance.action_type().as_str().to_owned(),
                name.to_owned(),
            )
        })
        .collect()
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
        .collect()
}

#[test]
fn a_person_is_offered_one_complete_meal_per_edible_or_drinkable_kind_carried() {
    let town = Town::begun(true, &HOLDINGS);
    assert_eq!(
        offered(&town, "alice"),
        pairs(&[("drink", "coffee"), ("eat", "croissant")]),
        "in item order; the mug is goods and is never offered"
    );
    assert_eq!(offered(&town, "bob"), pairs(&[("drink", "tea")]));
    assert!(
        offered(&town, "carol").is_empty(),
        "somebody who carries nothing is offered no meal"
    );
}

#[test]
fn an_accepted_meal_uses_one_up_through_inventorys_fact_caused_by_the_request() {
    let mut town = Town::begun(true, &HOLDINGS);
    let eaten = town
        .meal("alice", Meal::Eat, "croissant", None, 1)
        .expect("dispatch answers");
    assert!(
        matches!(eaten.result(), ActionResult::Accepted { .. }),
        "{:?}",
        eaten.result()
    );
    let drunk = town
        .meal("alice", Meal::Drink, "coffee", None, 2)
        .expect("dispatch answers");
    assert!(matches!(drunk.result(), ActionResult::Accepted { .. }));
    for done in [&eaten, &drunk] {
        let facts: Vec<_> = done
            .events()
            .iter()
            .filter(|fact| *fact.event_type() == ItemsConsumed::EVENT_TYPE)
            .collect();
        assert_eq!(facts.len(), 1, "one items-consumed, and nothing else");
        assert_eq!(done.events().len(), 1);
        assert!(
            matches!(facts[0].caused_by(), Causation::Action(_)),
            "caused by the request (AC-9)"
        );
        assert_eq!(*facts[0].visibility(), Visibility::Participants);
        assert_eq!(facts[0].participants(), &[town.id("alice")]);
    }
    assert_eq!(
        town.holdings("alice"),
        owned(&[("mug", 1)]),
        "the croissant and the coffee are gone; the mug stays"
    );
}

#[test]
fn the_wrong_meal_for_a_kind_is_refused_at_dispatch_and_writes_nothing() {
    let mut town = Town::begun(true, &HOLDINGS);
    for (id, (who, meal, item, target, expected, why)) in [
        (
            "alice",
            Meal::Eat,
            "coffee",
            None,
            Rejection::NoSupportedInteraction,
            "a drink is not eaten",
        ),
        (
            "alice",
            Meal::Drink,
            "croissant",
            None,
            Rejection::NoSupportedInteraction,
            "food is not drunk",
        ),
        (
            "alice",
            Meal::Eat,
            "mug",
            None,
            Rejection::NoSupportedInteraction,
            "goods are never consumed",
        ),
        (
            "alice",
            Meal::Drink,
            "coffee",
            Some("bob"),
            Rejection::NoSupportedInteraction,
            "a meal is nobody else's",
        ),
        (
            "bob",
            Meal::Eat,
            "croissant",
            None,
            Rejection::PreconditionFailed,
            "bob carries no croissant",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let before = town.state();
        let done = town
            .meal(
                who,
                meal,
                item,
                target,
                u64::try_from(id).expect("small") + 1,
            )
            .expect("dispatch answers");
        assert_eq!(*done.result(), ActionResult::Rejected(expected), "{why}");
        assert!(done.events().is_empty(), "{why}");
        assert_eq!(town.state(), before, "{why}: nothing written");
    }
}
