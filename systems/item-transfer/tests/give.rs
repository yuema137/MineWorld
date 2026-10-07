//! Give over a hand-built café through presence's real `observe` and the kernel's real dispatch:
//! offered complete, per kind held, to each other person present and never to oneself; decided here;
//! stated through inventory's constructor; refused with the reason a client can act on (D-5, D-6).

mod support;

use mineworld_contracts::{ActionId, ActionResult, Causation, Event, Rejection, Visibility};
use mineworld_inventory::ItemsTransferred;
use mineworld_item_transfer::Give;
use support::{GENESIS, HOLDINGS, Town, gives_to};

fn owned(pairs: &[(&str, u32)]) -> Vec<(String, u32)> {
    pairs.iter().map(|(k, n)| ((*k).to_owned(), *n)).collect()
}

fn payload(affordance: &mineworld_contracts::Affordance<serde_json::Value>) -> Give {
    serde_json::from_value(
        affordance
            .payload()
            .expect("a complete affordance carries its request")
            .clone(),
    )
    .expect("the payload is a give")
}

#[test]
fn a_person_is_offered_one_complete_give_per_kind_held_to_each_other_person_present() {
    let town = Town::begun(true, &HOLDINGS);
    let (alice, bob, carol) = (town.id("alice"), town.id("bob"), town.id("carol"));
    let seen = town.observation("alice", GENESIS);

    let to_bob = gives_to(&seen, bob);
    assert_eq!(to_bob.len(), 2, "Alice holds two kinds: {to_bob:?}");
    assert_eq!(
        to_bob.iter().map(|a| payload(a)).collect::<Vec<_>>(),
        [
            Give::new(town.item("coffee"), 1),
            Give::new(town.item("tea"), 1)
        ],
        "one per kind, count 1, in item order"
    );
    assert!(
        to_bob.iter().all(|a| a.is_available()),
        "Bob is within 3 m and can take one"
    );

    let to_carol = gives_to(&seen, carol);
    assert_eq!(to_carol.len(), 2);
    assert!(
        to_carol
            .iter()
            .all(|a| a.unavailable_reason() == Some(&Rejection::TooFarAway)),
        "Carol is 8 m away: offered, unavailable, with the reason: {to_carol:?}"
    );
    assert!(
        gives_to(&seen, alice).is_empty(),
        "nobody is offered a give to themselves"
    );

    let empty_handed = town.observation("carol", GENESIS);
    assert!(
        empty_handed
            .affordances()
            .iter()
            .all(|a| a.action_type().as_str() != "give"),
        "somebody who holds nothing is offered no give"
    );
}

#[test]
fn an_accepted_give_moves_one_item_through_inventorys_fact_caused_by_the_request() {
    let mut town = Town::begun(true, &HOLDINGS);
    let (alice, bob) = (town.id("alice"), town.id("bob"));
    let done = town
        .give("alice", Some("bob"), "coffee", 1, 1)
        .expect("dispatch answers");
    assert!(
        matches!(done.result(), ActionResult::Accepted { .. }),
        "{:?}",
        done.result()
    );
    assert_eq!(done.events().len(), 1);
    let fact = &done.events()[0];
    assert_eq!(*fact.event_type(), ItemsTransferred::EVENT_TYPE);
    assert_eq!(
        *fact.caused_by(),
        Causation::Action(ActionId::from_raw(1)),
        "caused by the request (AC-9)"
    );
    assert_eq!(*fact.visibility(), Visibility::Participants);
    assert_eq!(fact.participants(), &[alice, bob]);
    assert_eq!(fact.provenance().emitted_by().as_str(), "item-transfer");

    assert_eq!(town.holdings("alice"), owned(&[("tea", 1)]));
    assert_eq!(town.holdings("bob"), owned(&[("apple", 1), ("coffee", 1)]));
    let after = town.observation("alice", support::t(1));
    assert_eq!(
        gives_to(&after, bob)
            .iter()
            .map(|a| payload(a))
            .collect::<Vec<_>>(),
        [Give::new(town.item("tea"), 1)],
        "what is offered follows what is held"
    );
}

#[test]
fn a_give_is_refused_with_a_reason_a_client_can_act_on_and_writes_nothing() {
    let mut town = Town::begun(true, &HOLDINGS);
    for (id, (what, taker, item, count, reason)) in [
        (
            "to oneself",
            Some("alice"),
            "coffee",
            1,
            Rejection::NoSupportedInteraction,
        ),
        (
            "to nobody",
            None,
            "coffee",
            1,
            Rejection::NoSupportedInteraction,
        ),
        (
            "to a place",
            Some("cafe"),
            "coffee",
            1,
            Rejection::NoSupportedInteraction,
        ),
        (
            "to somebody 8 m away",
            Some("carol"),
            "coffee",
            1,
            Rejection::TooFarAway,
        ),
        (
            "a kind not held",
            Some("bob"),
            "apple",
            1,
            Rejection::PreconditionFailed,
        ),
        (
            "more than held",
            Some("bob"),
            "coffee",
            2,
            Rejection::PreconditionFailed,
        ),
        (
            "none at all",
            Some("bob"),
            "coffee",
            0,
            Rejection::PreconditionFailed,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let done = town
            .give(
                "alice",
                taker,
                item,
                count,
                u64::try_from(id).expect("small") + 1,
            )
            .expect("dispatch answers");
        assert_eq!(*done.result(), ActionResult::Rejected(reason), "{what}");
        assert!(done.events().is_empty(), "{what}: nothing recorded");
    }
    assert_eq!(town.holdings("alice"), owned(&[("coffee", 1), ("tea", 1)]));
    assert_eq!(town.holdings("bob"), owned(&[("apple", 1)]));
}

/// D-5's offer and dispatch halves: a give to a person who already carries six is offered unavailable,
/// `TargetUnavailable`, and refused the same at dispatch.
#[test]
fn a_give_to_a_full_person_is_offered_unavailable_and_refused_the_same() {
    let mut town = Town::begun(
        true,
        &[("alice", "{ coffee: 1, tea: 1 }"), ("bob", "{ apple: 6 }")],
    );
    let bob = town.id("bob");
    let seen = town.observation("alice", GENESIS);
    let to_bob = gives_to(&seen, bob);
    assert_eq!(to_bob.len(), 2);
    assert!(
        to_bob
            .iter()
            .all(|a| a.unavailable_reason() == Some(&Rejection::TargetUnavailable)),
        "Bob carries six: {to_bob:?}"
    );
    let done = town
        .give("alice", Some("bob"), "coffee", 1, 1)
        .expect("dispatch answers");
    assert_eq!(
        *done.result(),
        ActionResult::Rejected(Rejection::TargetUnavailable)
    );
    assert_eq!(town.holdings("bob"), owned(&[("apple", 6)]));
}
