//! Buying from any seller at the observed price, and the seller actions (`ARC-112`; S31-a AC-a-1,
//! AC-a-2, AC-a-3, AC-a-10), over the hand-built store through presence's real `observe` and the
//! kernel's real dispatch. Oracles are literal prices and wallets written here.
//!
//! The store asks bread 300; Grace, a person 2 m from Alice in the store, asks 260 for the two loaves
//! she carries; Felix keeps the store.

mod support;

use mineworld_contracts::{
    ActionRecord, ActionResult, Affordance, Causation, Component, EntityId, Rejection,
    RejectionCode,
};
use mineworld_economy::code::{
    ALREADY_LISTED, CANNOT_CARRY, INSUFFICIENT_FUNDS, NOT_FOR_SALE, NOT_HELD, OUT_OF_STOCK,
    PRICE_CHANGED, PRICE_OUT_OF_RANGE, PRICE_UNCHANGED,
};
use mineworld_economy::{
    Asks, AsksView, Buy, CurrencyRecord, ListForSale, MoneyTransferred, PRICE_MAX, SetPrice,
    ShopSide, Sold, Wallet, Wealth, Withdraw,
};
use mineworld_inventory::ItemsTransferred;
use serde_json::Value;
use support::{GENESIS, SHOP, Town, buys, of};

fn code(code: RejectionCode) -> Rejection {
    Rejection::System { code, detail: None }
}

/// The buy `observer` is offered for `item`, from `seller` (a person) or from the shop (`None`).
fn offer(
    seen: &mineworld_contracts::Observation<Value>,
    seller: Option<EntityId>,
    item: mineworld_contracts::ItemId,
) -> Option<&Affordance<Value>> {
    buys(seen).into_iter().find(|affordance| {
        affordance.target() == seller
            && affordance.payload().is_some_and(|payload| {
                serde_json::from_value::<Buy>(payload.clone()).is_ok_and(|buy| buy.item() == item)
            })
    })
}

/// The buy an offer carries.
fn carried(affordance: &Affordance<Value>) -> Buy {
    serde_json::from_value(affordance.payload().expect("complete").clone()).expect("a buy")
}

/// AC-a-1: two sellers, two prices; each sale's three facts share one cause.
#[test]
fn a_buyer_buys_from_a_shop_and_from_a_person_each_at_its_own_price() {
    let mut town = Town::begun(SHOP);
    let seen = town.observation("alice", GENESIS);
    let (bread, grace) = (town.item("bread"), town.id("grace"));
    let from_shop = carried(offer(&seen, None, bread).expect("the store's bread is offered"));
    let from_grace = carried(offer(&seen, Some(grace), bread).expect("grace's bread is offered"));
    assert_eq!((from_shop.price(), from_grace.price()), (300, 260));

    for (id, seller, price) in [(1, None, 300), (2, Some("grace"), 260)] {
        let done = town
            .buy_at("alice", seller, "bread", price, id)
            .expect("answers");
        assert!(
            matches!(done.result(), ActionResult::Accepted { .. }),
            "{seller:?}: {:?}",
            done.result()
        );
        let sold = of::<Sold>(done.events());
        let paid = of::<MoneyTransferred>(done.events());
        let goods = of::<ItemsTransferred>(done.events());
        assert_eq!((sold.len(), paid.len(), goods.len()), (1, 1, 1));
        assert_eq!(sold[0].0.price(), price);
        assert_eq!(paid[0].0.amount(), price, "I-a-4: the amount is the price");
        let causes: Vec<&Causation> = [&sold[0].1, &paid[0].1, &goods[0].1]
            .iter()
            .map(|fact| fact.caused_by())
            .collect();
        assert!(
            causes.iter().all(|cause| **cause == *causes[0])
                && matches!(causes[0], Causation::Action(_)),
            "one cause, the request: {causes:?}"
        );
    }
    assert_eq!(town.wallet("alice"), Some(1_000 - 560));
    assert_eq!(town.wallet("corner-store"), Some(1_300));
    assert_eq!(town.wallet("grace"), Some(1_260));
    assert_eq!(town.holdings("alice"), [("bread".to_owned(), 2)]);
    assert_eq!(town.holdings("grace"), [("bread".to_owned(), 1)]);
}

/// AC-a-2: an offer observed before a keeper re-prices is refused `price-changed`, and nothing moves.
#[test]
fn a_stale_price_is_refused_and_nothing_moves() {
    let mut town = Town::begun(SHOP);
    let seen = town.observation("alice", GENESIS);
    let observed = carried(offer(&seen, None, town.item("bread")).expect("offered"));
    assert_eq!(observed.price(), 300);
    let repriced = town
        .sell(
            "felix",
            &SetPrice::new(town.item("bread"), 320, Some(ShopSide::Shop)),
            1,
        )
        .expect("answers");
    assert!(
        matches!(repriced.result(), ActionResult::Accepted { .. }),
        "{:?}",
        repriced.result()
    );
    assert_eq!(town.ask("corner-store", "bread"), 320);
    let before = town.state();
    let done = town
        .buy_at("alice", None, "bread", observed.price(), 2)
        .expect("answers");
    assert_eq!(*done.result(), ActionResult::Rejected(code(PRICE_CHANGED)));
    assert!(done.events().is_empty());
    assert_eq!(town.state(), before, "state bytes before = after");
}

/// AC-a-3, the buys: each named refusal by dispatch and, where an offer exists, by the offer's own
/// refusal; nothing written.
#[test]
fn each_buy_refusal_is_named_by_the_offer_and_by_the_dispatch_and_moves_nothing() {
    let mut town = Town::begun(SHOP);
    let cases: [(&str, Option<&str>, &str, Rejection, &str); 8] = [
        (
            "alice",
            Some("grace"),
            "apple",
            code(NOT_FOR_SALE),
            "grace asks nothing for apples",
        ),
        (
            "bob",
            None,
            "bread",
            code(INSUFFICIENT_FUNDS),
            "bob's 150 < 300",
        ),
        (
            "alice",
            None,
            "juice",
            code(OUT_OF_STOCK),
            "the store holds no juice",
        ),
        (
            "erin",
            None,
            "apple",
            code(CANNOT_CARRY),
            "erin carries six",
        ),
        (
            "felix",
            Some("grace"),
            "bread",
            Rejection::TooFarAway,
            "felix is 3.6 m from grace",
        ),
        (
            "carol",
            Some("grace"),
            "bread",
            Rejection::TooFarAway,
            "carol is in another place",
        ),
        (
            "felix",
            None,
            "apple",
            Rejection::PreconditionFailed,
            "a keeper is the shop (SelfTrade)",
        ),
        (
            "carol",
            None,
            "apple",
            Rejection::NoSupportedInteraction,
            "in no shop, no target",
        ),
    ];
    for (id, (buyer, seller, item, expected, why)) in cases.into_iter().enumerate() {
        let seen = town.observation(buyer, GENESIS);
        let target = seller.map(|seller| town.id(seller));
        let price = match seller {
            Some(seller) => town.ask(seller, item),
            None => town.ask("corner-store", item),
        };
        if let Some(affordance) = offer(&seen, target, town.item(item)) {
            assert!(!affordance.is_available(), "{why}: offered unavailable");
            assert_eq!(
                affordance.unavailable_reason(),
                Some(&expected),
                "{why}: the offer"
            );
        }
        let before = town.state();
        let done = town
            .buy_at(
                buyer,
                seller,
                item,
                price,
                u64::try_from(id).expect("small") + 1,
            )
            .expect("answers");
        assert_eq!(*done.result(), ActionResult::Rejected(expected), "{why}");
        assert!(done.events().is_empty(), "{why}");
        assert_eq!(town.state(), before, "{why}: nothing written");
    }
}

/// A seller action's request.
fn record<A: mineworld_contracts::Action + serde::Serialize>(action: &A) -> ActionRecord {
    ActionRecord::new::<A>(serde_json::to_vec(action).expect("encodes"))
}

/// AC-a-3, the seller actions: accepted and refused by code, writing only the seller's asks.
#[test]
fn seller_actions_are_accepted_or_refused_by_name() {
    let mut town = Town::begun(SHOP);
    let (apple, bread) = (town.item("apple"), town.item("bread"));
    let shop = Some(ShopSide::Shop);
    let refusals = [
        (
            "alice",
            record(&SetPrice::new(bread, 1, shop)),
            Rejection::PermissionDenied,
            "alice keeps no shop",
        ),
        (
            "grace",
            record(&ListForSale::new(bread, 300, None)),
            code(ALREADY_LISTED),
            "grace asks for bread already",
        ),
        (
            "felix",
            record(&ListForSale::new(apple, 90, shop)),
            code(ALREADY_LISTED),
            "the store asks for apples already",
        ),
        (
            "grace",
            record(&SetPrice::new(bread, 0, None)),
            code(PRICE_OUT_OF_RANGE),
            "a price of zero",
        ),
        (
            "grace",
            record(&SetPrice::new(bread, PRICE_MAX + 1, None)),
            code(PRICE_OUT_OF_RANGE),
            "above PRICE_MAX",
        ),
        (
            "alice",
            record(&ListForSale::new(apple, 50, None)),
            code(NOT_HELD),
            "alice holds no apple",
        ),
        (
            "grace",
            record(&SetPrice::new(bread, 260, None)),
            code(PRICE_UNCHANGED),
            "the same price",
        ),
        (
            "grace",
            record(&Withdraw::new(apple, None)),
            code(NOT_FOR_SALE),
            "grace asks nothing for apples",
        ),
    ];
    for (id, (actor, request, expected, why)) in refusals.into_iter().enumerate() {
        let before = town.state();
        let done = town
            .act(actor, None, request, u64::try_from(id).expect("small") + 1)
            .expect("answers");
        assert_eq!(*done.result(), ActionResult::Rejected(expected), "{why}");
        assert!(done.events().is_empty(), "{why}");
        assert_eq!(town.state(), before, "{why}: nothing written");
    }

    for (id, (actor, request, seller, item, ask)) in [
        (
            "grace",
            record(&SetPrice::new(bread, 240, None)),
            "grace",
            "bread",
            240,
        ),
        (
            "grace",
            record(&Withdraw::new(bread, None)),
            "grace",
            "bread",
            0,
        ),
        (
            "grace",
            record(&ListForSale::new(bread, 250, None)),
            "grace",
            "bread",
            250,
        ),
        (
            "felix",
            record(&Withdraw::new(apple, shop)),
            "corner-store",
            "apple",
            0,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let done = town
            .act(actor, None, request, u64::try_from(id).expect("small") + 20)
            .expect("answers");
        assert!(
            matches!(done.result(), ActionResult::Accepted { .. }),
            "{actor}: {:?}",
            done.result()
        );
        assert_eq!(done.events().len(), 1, "one fact");
        assert_eq!(
            town.ask(seller, item),
            ask,
            "{actor} changed {seller}'s ask"
        );
    }
}

/// The seller actions are offered incomplete, once per type, never with a target (SD-S31a-6, Q-S31a-3).
#[test]
fn seller_actions_are_offered_incomplete_once_per_type() {
    let town = Town::begun(SHOP);
    let kinds = |observer: &str| -> Vec<(String, bool)> {
        let seen = town.observation(observer, GENESIS);
        seen.affordances()
            .iter()
            .filter(|affordance| {
                ["list-for-sale", "set-price", "withdraw"]
                    .contains(&affordance.action_type().as_str())
            })
            .map(|affordance| {
                assert!(affordance.target().is_none());
                (
                    affordance.action_type().as_str().to_owned(),
                    affordance.payload().is_some(),
                )
            })
            .collect()
    };
    let incomplete = |names: &[&str]| -> Vec<(String, bool)> {
        names
            .iter()
            .map(|name| ((*name).to_owned(), false))
            .collect()
    };
    assert_eq!(
        kinds("grace"),
        incomplete(&["set-price", "withdraw"]),
        "grace asks for all she holds"
    );
    assert_eq!(
        kinds("felix"),
        incomplete(&["set-price", "withdraw"]),
        "felix keeps the store, which asks for everything it holds"
    );
    assert_eq!(
        kinds("erin"),
        incomplete(&["list-for-sale"]),
        "erin holds pens nobody lists"
    );
    assert!(
        kinds("alice").is_empty(),
        "alice holds nothing and keeps nothing"
    );
}

/// AC-a-10: wallet and wealth to the holder only; the currency to every observer about itself; a
/// person seller's asks to whoever perceives her, availability only; the shop listing unchanged.
#[test]
fn money_is_disclosed_to_its_holder_and_asks_to_whoever_perceives_the_seller() {
    let town = Town::begun(SHOP);
    let seen = town.observation("alice", GENESIS);
    let types = |subject: &str| -> Vec<String> {
        seen.entity(town.id(subject))
            .map(|entity| {
                entity
                    .components()
                    .iter()
                    .map(|record| record.component_type().as_str().to_owned())
                    .collect()
            })
            .unwrap_or_default()
    };
    let mine = types("alice");
    for kind in [
        Wallet::COMPONENT_TYPE,
        Wealth::COMPONENT_TYPE,
        CurrencyRecord::COMPONENT_TYPE,
    ] {
        assert!(
            mine.contains(&kind.as_str().to_owned()),
            "alice is told her {kind}"
        );
    }
    for other in ["bob", "grace"] {
        let theirs = types(other);
        for kind in [Wallet::COMPONENT_TYPE, Wealth::COMPONENT_TYPE] {
            assert!(
                !theirs.contains(&kind.as_str().to_owned()),
                "{other}'s {kind}"
            );
        }
    }
    let asks = seen
        .entity(town.id("grace"))
        .and_then(|entity| {
            entity
                .components()
                .iter()
                .find(|record| *record.component_type() == Asks::COMPONENT_TYPE)
        })
        .expect("alice perceives grace's asks");
    let view: AsksView = serde_json::from_value(asks.payload().clone()).expect("an asks view");
    assert_eq!(
        view.asks()
            .iter()
            .map(|line| (line.item(), line.price(), line.available()))
            .collect::<Vec<_>>(),
        [(town.item("bread"), 260, true)]
    );
    let fields: Vec<&String> = asks.payload()["asks"][0]
        .as_object()
        .expect("a line")
        .keys()
        .collect();
    assert_eq!(fields, ["available", "item", "price"], "never a count");
}
