//! E-5 and the money half of E-4, over a hand-built store through presence's real `observe` and the
//! kernel's real dispatch: buy offered complete per priced kind to a person in the shop, available
//! iff stocked, affordable and carriable; decided here; paid through economy's own fact and handed
//! over through inventory's constructor; refused with the reason a client can act on. Money stated
//! past the rule is refused by its owner.

mod support;

use mineworld_authoring::AuthoredSection;
use mineworld_contracts::{
    ActionResult, Affordance, Causation, Component, EntityType, Event, PlaceId, Rejection,
    Visibility,
};
use mineworld_economy::{Buy, EconomySystem, Listing, MoneyTransferred, Shop, Wallet};
use mineworld_inventory::ItemsTransferred;
use mineworld_kernel::{KernelError, SystemIdentity};
use serde_json::Value;
use support::{GENESIS, Ledger, Post, SHOP, Town, buys, of, post, section};

fn owned(pairs: &[(&str, u32)]) -> Vec<(String, u32)> {
    pairs.iter().map(|(k, n)| ((*k).to_owned(), *n)).collect()
}

fn payload(affordance: &Affordance<Value>) -> Buy {
    serde_json::from_value(
        affordance
            .payload()
            .expect("a complete affordance carries its request")
            .clone(),
    )
    .expect("the payload is a buy")
}

/// Per offered buy: (item key, available).
fn offered(town: &Town, observer: &str) -> Vec<(String, bool)> {
    let seen = town.observation(observer, GENESIS);
    buys(&seen)
        .into_iter()
        .map(|affordance| {
            let item = payload(affordance).item();
            let name = ["apple", "bread", "juice", "pen"]
                .into_iter()
                .find(|name| town.item(name) == item)
                .expect("a kind of the town");
            assert!(affordance.target().is_none(), "a buy has no target");
            if !affordance.is_available() {
                assert_eq!(
                    affordance.unavailable_reason(),
                    Some(&Rejection::TargetUnavailable),
                    "the one reason an offer carries (QS-44)"
                );
            }
            (name.to_owned(), affordance.is_available())
        })
        .collect()
}

fn pairs(list: &[(&str, bool)]) -> Vec<(String, bool)> {
    list.iter().map(|(k, b)| ((*k).to_owned(), *b)).collect()
}

#[test]
fn a_person_in_a_shop_is_offered_one_complete_buy_per_priced_kind() {
    let town = Town::begun(SHOP);
    assert_eq!(
        offered(&town, "alice"),
        pairs(&[("apple", true), ("bread", true), ("juice", false)]),
        "per priced kind, in item order; juice is out of stock; the pen is not sold here"
    );
    assert_eq!(
        offered(&town, "bob"),
        pairs(&[("apple", true), ("bread", false), ("juice", false)]),
        "bob's 150 pays for an apple (100), not for bread (300)"
    );
    assert_eq!(
        offered(&town, "erin"),
        pairs(&[("apple", false), ("bread", false), ("juice", false)]),
        "erin carries six and can take nothing more"
    );
    assert!(
        offered(&town, "carol").is_empty(),
        "nobody outside a shop is offered a buy"
    );
}

#[test]
fn an_accepted_buy_moves_one_item_and_the_price_both_caused_by_the_request() {
    let mut town = Town::begun(SHOP);
    let done = town.buy("alice", "bread", 1).expect("dispatch answers");
    assert!(
        matches!(done.result(), ActionResult::Accepted { .. }),
        "{:?}",
        done.result()
    );
    let money = of::<MoneyTransferred>(done.events());
    let goods = of::<ItemsTransferred>(done.events());
    assert_eq!(
        (money.len(), goods.len()),
        (1, 1),
        "one payment, one hand-over"
    );
    let (paid, paid_fact) = &money[0];
    assert_eq!(
        (paid.from(), paid.to(), paid.amount()),
        (town.id("alice"), town.id("corner-store"), 300)
    );
    let store = PlaceId::new(town.id("store"), EntityType::Place).expect("a place");
    assert_eq!(
        *paid_fact.visibility(),
        Visibility::Place(store),
        "a purchase is heard in the shop (QS-45)"
    );
    let (handed, handed_fact) = &goods[0];
    assert_eq!(
        (handed.from(), handed.to(), handed.item(), handed.count()),
        (
            town.id("corner-store"),
            town.id("alice"),
            town.item("bread"),
            1
        )
    );
    assert_eq!(*handed_fact.visibility(), Visibility::Participants);
    for fact in [paid_fact, handed_fact] {
        assert!(
            matches!(fact.caused_by(), Causation::Action(_)),
            "caused by the request (AC-9)"
        );
    }
    assert_eq!(
        (town.wallet("alice"), town.wallet("corner-store")),
        (Some(700), Some(1_300))
    );
    assert_eq!(town.holdings("alice"), owned(&[("bread", 1)]));
    assert_eq!(
        town.holdings("corner-store"),
        owned(&[("apple", 3), ("bread", 1)])
    );
}

#[test]
fn a_buy_is_refused_at_dispatch_for_the_same_reasons_and_writes_nothing() {
    let mut town = Town::begun(SHOP);
    for (id, (buyer, item, expected, why)) in [
        ("bob", "bread", Rejection::TargetUnavailable, "cannot pay"),
        (
            "alice",
            "juice",
            Rejection::TargetUnavailable,
            "out of stock",
        ),
        (
            "erin",
            "apple",
            Rejection::TargetUnavailable,
            "cannot carry",
        ),
        (
            "carol",
            "apple",
            Rejection::NoSupportedInteraction,
            "in no shop",
        ),
        (
            "alice",
            "pen",
            Rejection::NoSupportedInteraction,
            "not sold here",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let before = town.state();
        let done = town
            .buy(buyer, item, u64::try_from(id).expect("small") + 1)
            .expect("dispatch answers");
        assert_eq!(*done.result(), ActionResult::Rejected(expected), "{why}");
        assert!(done.events().is_empty(), "{why}");
        assert_eq!(town.state(), before, "{why}: nothing written");
    }
}

#[test]
fn the_shops_listing_is_disclosed_to_whoever_is_there_and_a_wallet_to_its_holder() {
    let mut town = Town::begun(SHOP);
    let listing_seen_by = |town: &Town, observer: &str| -> Option<Listing> {
        let seen = town.observation(observer, GENESIS);
        let place = seen.entity(town.id("store"))?;
        place
            .components()
            .iter()
            .find(|record| *record.component_type() == Shop::COMPONENT_TYPE)
            .map(|record| serde_json::from_value(record.payload().clone()).expect("a listing"))
    };
    let stock = |listing: &Listing| -> Vec<(u64, u32)> {
        listing
            .listed()
            .iter()
            .map(|line| (line.price(), line.in_stock()))
            .collect()
    };
    let before = listing_seen_by(&town, "bob").expect("bob, in the store, sees its listing");
    assert_eq!(
        before.operator().entity_id(),
        town.id("corner-store"),
        "who runs it"
    );
    assert_eq!(stock(&before), [(100, 3), (300, 2), (200, 0)]);
    assert!(
        listing_seen_by(&town, "carol").is_none(),
        "carol, on the street, does not perceive the store"
    );

    let bought = town.buy("alice", "apple", 1).expect("answers");
    assert!(matches!(bought.result(), ActionResult::Accepted { .. }));
    let after = listing_seen_by(&town, "bob").expect("still there");
    assert_eq!(
        stock(&after),
        [(100, 2), (300, 2), (200, 0)],
        "a second person in the shop perceives the purchase as the shelf changing (CP-7)"
    );

    let seen = town.observation("bob", GENESIS);
    let wallets = |subject| {
        seen.entity(subject).map(|entity| {
            entity
                .components()
                .iter()
                .filter(|record| *record.component_type() == Wallet::COMPONENT_TYPE)
                .count()
        })
    };
    assert_eq!(
        wallets(town.id("bob")),
        Some(1),
        "bob is told his own wallet"
    );
    assert_eq!(
        wallets(town.id("alice")),
        Some(0),
        "bob perceives alice and is not told what she has"
    );
}

/// E-4: `money-transferred` stated past every rule — more than the payer holds, a zero amount, to
/// oneself, to a place — is refused by economy and writes nothing; a valid one through the same forged
/// path is reduced, the positive control.
#[test]
fn money_stated_past_the_rule_is_refused_by_its_owner_and_writes_nothing() {
    let mut town = Town::begun_plus(SHOP, support::STOCK, |world| {
        world
            .install(Ledger)
            .expect("the ledger installs after economy");
    });
    let (alice, bob, store) = (town.id("alice"), town.id("bob"), town.id("store"));
    for (id, (what, request)) in [
        (
            "more than the payer holds",
            Post {
                from: bob,
                to: alice,
                amount: 151,
            },
        ),
        (
            "a zero amount",
            Post {
                from: alice,
                to: bob,
                amount: 0,
            },
        ),
        (
            "to oneself",
            Post {
                from: alice,
                to: alice,
                amount: 1,
            },
        ),
        (
            "to a place",
            Post {
                from: alice,
                to: store,
                amount: 1,
            },
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let before = town.state();
        match post(
            &mut town,
            "alice",
            &request,
            u64::try_from(id).expect("small") + 1,
        ) {
            Err(KernelError::FactRefusedByOwner {
                system,
                event_type,
                reason,
            }) => {
                assert_eq!(system, EconomySystem::ID, "{what}");
                assert_eq!(event_type, MoneyTransferred::EVENT_TYPE, "{what}");
                assert_eq!(reason, Rejection::PreconditionFailed, "{what}");
            }
            other => panic!("{what}: economy must refuse as the owner, but got {other:?}"),
        }
        assert_eq!(town.state(), before, "{what}: nothing written");
    }
    let valid = post(
        &mut town,
        "alice",
        &Post {
            from: bob,
            to: alice,
            amount: 150,
        },
        9,
    )
    .expect("a valid forged payment reduces");
    assert!(matches!(valid.result(), ActionResult::Accepted { .. }));
    assert_eq!(
        (town.wallet("bob"), town.wallet("alice")),
        (Some(0), Some(1_150)),
        "positive control: the owner reduced a payment another system stated, to exactly zero"
    );
}

#[test]
fn the_economy_section_is_checked_as_it_is_decoded_and_as_it_is_seeded() {
    assert!(section("{ wallet: -1 }").is_err(), "no negative money");
    assert!(
        section("{ wallet: 1.5 }").is_err(),
        "integer minor units only"
    );
    assert!(
        section("{ wallet: 1, shop: { at: store, prices: { apple: 0 } } }").is_err(),
        "a price of zero"
    );
    assert!(section("{ wallet: 1, bank: 3 }").is_err(), "an unknown key");
    let authored = section("{ wallet: 1, shop: { at: store, prices: { bread: 3, apple: 2 } } }")
        .expect("valid");
    assert_eq!(
        EconomySystem::references(&authored)
            .iter()
            .map(|reference| (reference.key.as_str(), reference.entity_type))
            .collect::<Vec<_>>(),
        [
            ("store", EntityType::Place),
            ("apple", EntityType::Item),
            ("bread", EntityType::Item),
        ],
        "the shop's place and every priced kind must resolve, by kind"
    );

    let town = Town::begun(SHOP);
    let read = town.world.read();
    let seeding = mineworld_authoring::Seeding::new(&read, &town.keys);
    assert_eq!(
        EconomySystem::seed(&seeding, town.id("alice"), &authored)
            .expect_err("a shop on a person's file"),
        Rejection::PreconditionFailed,
        "only an organization runs a shop"
    );
    assert_eq!(
        EconomySystem::seed(
            &seeding,
            town.id("store"),
            &section("{ wallet: 1 }").expect("valid")
        )
        .expect_err("a wallet on a place"),
        Rejection::PreconditionFailed
    );
}
