//! Asks on the seller (`ARC-112`; S31-a C3, AC-a-3's section codes): the `economy:` section's keys per
//! kind of file, each refusal by its code; the genesis facts of a shop and of a person seller, in
//! order and with their audiences; the shop's listing, read from the operator's asks, unchanged in
//! shape; reference values on item kinds.

mod support;

use mineworld_authoring::{AuthoredSection, Seeding};
use mineworld_contracts::{
    Component, EntityType, Event, EventEnvelope, PlaceId, Rejection, RejectionCode, Visibility,
};
use mineworld_economy::code::{
    ASKS_ON_ORGANIZATION, ITEM_REFERENCE_ONLY, KEEPER_DUPLICATE, KEEPERS_TOO_MANY,
    PRICE_OUT_OF_RANGE, REFERENCE_ON_HOLDER, SHOP_ON_PERSON, WALLET_UNRESOLVED,
};
use mineworld_economy::event::Listed;
use mineworld_economy::{
    Asks, CurrencyDeclared, CurrencyRecord, EconomySystem, Funded, ReferenceValue, Shop, ShopOpened,
};
use serde_json::{Value, json};
use support::{GENESIS, SHOP, Setup, Town, section};

fn code(code: RejectionCode) -> Rejection {
    Rejection::System { code, detail: None }
}

/// The refusal `subject`'s file carrying `yaml` gets at genesis.
fn seeded(town: &Town, subject: &str, yaml: &str) -> Result<usize, Rejection> {
    let read = town.world.read();
    let seeding = Seeding::new(&read, &town.keys);
    <EconomySystem as AuthoredSection>::seed(
        &seeding,
        town.id(subject),
        &section(yaml).expect("decodes"),
    )
    .map(|facts| facts.len())
}

#[test]
fn each_kind_of_file_may_carry_only_its_keys_and_each_refusal_names_its_code() {
    let town = Town::begun(SHOP);
    let nine: Vec<String> = (0..9).map(|n| format!("k{n}")).collect();
    let cases = [
        (
            "alice",
            "{ asks: { apple: 1 } }",
            WALLET_UNRESOLVED,
            "a person with no wallet",
        ),
        (
            "corner-store",
            "{ shop: { at: store, prices: { apple: 1 } } }",
            WALLET_UNRESOLVED,
            "an organization with no wallet",
        ),
        (
            "alice",
            "{ wallet: 1, shop: { at: store, prices: { apple: 1 } } }",
            SHOP_ON_PERSON,
            "a shop on a person",
        ),
        (
            "alice",
            "{ wallet: 1, reference: 5 }",
            REFERENCE_ON_HOLDER,
            "a reference on a person",
        ),
        (
            "corner-store",
            "{ wallet: 1, reference: 5 }",
            REFERENCE_ON_HOLDER,
            "a reference on an organization",
        ),
        (
            "corner-store",
            "{ wallet: 1, asks: { apple: 1 } }",
            ASKS_ON_ORGANIZATION,
            "asks on an organization",
        ),
        (
            "corner-store",
            "{ wallet: 1, shop: { at: store, prices: { apple: 1 }, keepers: [felix, felix] } }",
            KEEPER_DUPLICATE,
            "a keeper twice",
        ),
        (
            "apple",
            "{ wallet: 1, reference: 5 }",
            ITEM_REFERENCE_ONLY,
            "a wallet on an item",
        ),
        (
            "apple",
            "{ asks: { bread: 2 }, reference: 5 }",
            ITEM_REFERENCE_ONLY,
            "asks on an item",
        ),
        (
            "apple",
            "{}",
            ITEM_REFERENCE_ONLY,
            "an item's section without a reference",
        ),
        (
            "apple",
            "{ reference: 1000000000001 }",
            PRICE_OUT_OF_RANGE,
            "a reference above 10^12",
        ),
        (
            "alice",
            "{ wallet: 1, asks: { apple: 1000000000001 } }",
            PRICE_OUT_OF_RANGE,
            "an ask above 10^12",
        ),
        (
            "corner-store",
            "{ wallet: 1, shop: { at: store, prices: { apple: 1000000000001 } } }",
            PRICE_OUT_OF_RANGE,
            "a shop price above 10^12",
        ),
    ];
    for (subject, yaml, expected, what) in cases {
        assert_eq!(seeded(&town, subject, yaml), Err(code(expected)), "{what}");
    }
    let too_many = format!(
        "{{ wallet: 1, shop: {{ at: store, prices: {{ apple: 1 }}, keepers: [{}] }} }}",
        nine.join(", ")
    );
    assert_eq!(
        seeded(&town, "corner-store", &too_many),
        Err(code(KEEPERS_TOO_MANY)),
        "a ninth keeper"
    );
    assert_eq!(
        seeded(&town, "store", "{ wallet: 1 }"),
        Err(Rejection::PreconditionFailed),
        "a place carries no economy"
    );
    // Positive controls: the bounds themselves are taken.
    assert_eq!(
        seeded(&town, "apple", "{ reference: 1000000000000 }"),
        Ok(1)
    );
    assert_eq!(
        seeded(
            &town,
            "alice",
            "{ wallet: 0, asks: { apple: 1, bread: 1000000000000 } }"
        ),
        Ok(3),
        "funded and two listed"
    );
    assert!(
        section("{ wallet: 1, asks: { apple: 0 } }").is_err(),
        "an ask of zero is refused as it is decoded"
    );
}

/// The facts of one type in genesis, with their envelopes.
fn genesis_of<'a>(town: &'a Town, event_type: &str) -> Vec<&'a EventEnvelope> {
    town.genesis
        .iter()
        .filter(|fact| fact.event_type().as_str() == event_type)
        .collect()
}

#[test]
fn a_shop_and_a_person_seller_list_at_genesis_in_order_and_heard_where_they_are() {
    let town = Town::begun(SHOP);
    let store = PlaceId::new(town.id("store"), EntityType::Place).expect("a place");
    let order: Vec<&str> = town
        .genesis
        .iter()
        .map(|fact| fact.event_type().as_str())
        .filter(|kind| {
            [
                CurrencyDeclared::EVENT_TYPE.as_str(),
                Funded::EVENT_TYPE.as_str(),
                ShopOpened::EVENT_TYPE.as_str(),
                Listed::EVENT_TYPE.as_str(),
            ]
            .contains(kind)
        })
        .collect();
    assert_eq!(
        order[..6],
        [
            "currency-declared",
            "funded",
            "shop-opened",
            "listed",
            "listed",
            "listed"
        ],
        "the configuration first, then the store's wallet, its shop and its three prices"
    );
    assert_eq!(order[order.len() - 2..], ["funded", "listed"], "grace last");

    let shop_listed: Vec<_> = genesis_of(&town, "listed")
        .into_iter()
        .filter(|fact| fact.subjects().contains(&town.id("corner-store")))
        .collect();
    assert_eq!(shop_listed.len(), 3);
    for fact in &shop_listed {
        assert_eq!(
            *fact.visibility(),
            Visibility::Place(store),
            "a shop's prices are heard in the shop, not by everybody (M-a-3)"
        );
    }
    let grace_listed: Vec<_> = genesis_of(&town, "listed")
        .into_iter()
        .filter(|fact| fact.subjects().contains(&town.id("grace")))
        .collect();
    assert_eq!(grace_listed.len(), 1);
    assert_eq!(
        *grace_listed[0].visibility(),
        Visibility::Entities([town.id("grace")].into_iter().collect()),
        "a person's genesis asks are heard by the person (SD-S31a-13)"
    );
    assert_eq!(
        *genesis_of(&town, "shop-opened")[0].visibility(),
        Visibility::Public
    );

    let read = town.world.read();
    let asks = |holder: &str| -> Vec<(u64, u64)> {
        read.component::<Asks>(town.id(holder))
            .map(|asks| {
                asks.asks()
                    .iter()
                    .map(|ask| (ask.item().entity_id().raw(), ask.price()))
                    .collect()
            })
            .unwrap_or_default()
    };
    let raw = |name: &str| town.id(name).raw();
    assert_eq!(
        asks("corner-store"),
        [
            (raw("apple"), 100),
            (raw("bread"), 300),
            (raw("juice"), 200)
        ]
    );
    assert_eq!(asks("grace"), [(raw("bread"), 260)]);
    let shop = read
        .component::<Shop>(town.id("store"))
        .expect("the store's shop");
    assert_eq!(shop.operator().entity_id(), town.id("corner-store"));
    assert!(shop.kept_by(town.id("felix")) && !shop.kept_by(town.id("alice")));
}

#[test]
fn the_listing_reads_the_operators_asks_and_keeps_its_shape() {
    let town = Town::begun(SHOP);
    let seen = town.observation("bob", GENESIS);
    let payload = seen
        .entity(town.id("store"))
        .expect("the store")
        .components()
        .iter()
        .find(|record| *record.component_type() == Shop::COMPONENT_TYPE)
        .expect("the listing")
        .payload()
        .clone();
    let item = |name: &str| serde_json::to_value(town.item(name)).expect("an id");
    let expected = json!({
        "operator": serde_json::to_value(
            mineworld_contracts::OrganizationId::new(town.id("corner-store"), EntityType::Organization)
                .expect("an organization"),
        ).expect("an id"),
        "listed": [
            { "item": item("apple"), "price": 100, "in_stock": 3 },
            { "item": item("bread"), "price": 300, "in_stock": 2 },
            { "item": item("juice"), "price": 200, "in_stock": 0 },
        ],
    });
    assert_eq!(
        serde_json::to_string(&payload).expect("encodes"),
        serde_json::to_string(&expected).expect("encodes"),
        "the shop listing is byte for byte the shape it had before asks (SD-S31a-12)"
    );
}

#[test]
fn an_item_kinds_reference_value_is_reduced_and_listed_in_the_currency_record() {
    let setup = Setup::store(SHOP)
        .with_reference("bread", "{ reference: 280 }")
        .with_reference("apple", "{ reference: 90 }");
    let town = Town::build(&setup, |_| {}).expect("begins");
    let read = town.world.read();
    assert_eq!(
        read.component::<ReferenceValue>(town.id("bread"))
            .map(ReferenceValue::value),
        Some(280)
    );
    let declared = genesis_of(&town, "reference-declared");
    assert_eq!(declared.len(), 2);
    assert!(
        declared
            .iter()
            .all(|fact| *fact.visibility() == Visibility::SystemInternal),
        "reference values are configuration-like, nobody's perception (Q-S31a-2)"
    );
    let seen = town.observation("carol", GENESIS);
    let record: Value = seen
        .entity(town.id("carol"))
        .expect("carol herself")
        .components()
        .iter()
        .find(|record| *record.component_type() == CurrencyRecord::COMPONENT_TYPE)
        .expect("the currency record")
        .payload()
        .clone();
    let record: CurrencyRecord = serde_json::from_value(record).expect("decodes");
    assert_eq!(
        record
            .references()
            .iter()
            .map(|line| (line.item(), line.value()))
            .collect::<Vec<_>>(),
        [(town.item("apple"), 90), (town.item("bread"), 280)],
        "every kind's reference value, in item order"
    );
}

#[test]
fn a_listing_or_reference_without_a_currency_is_refused_naming_the_code() {
    let setup = Setup::store(SHOP)
        .with_reference("bread", "{ reference: 280 }")
        .with_economy("corner-store", "{ wallet: 1 }");
    let mut setup = setup;
    setup.configuration = None;
    match Town::build(&setup, |_| {}) {
        Err(mineworld_kernel::KernelError::FactRefusedByOwner {
            event_type, reason, ..
        }) => {
            assert_eq!(
                event_type.as_str(),
                "reference-declared",
                "items are seeded first"
            );
            assert_eq!(reason, code(mineworld_economy::code::CURRENCY_UNDECLARED));
        }
        Ok(_) => panic!("began with no currency"),
        Err(other) => panic!("{other:?}"),
    }
}
