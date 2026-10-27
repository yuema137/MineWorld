//! The drift guard of `ARC-182` (`pr-s31d-reporter-policies.md` §5.7, AC-d-7): the wire mirrors a
//! controller reads the market through equal their owners' types.
//!
//! `rule-controller` may not link a market pack (`ARC-35` check 2), so it reads economy's, inventory's and
//! item's records and builds economy's requests through mirrors in `price::wire`. Here, where both sides
//! may be named, each real record is built by its owner's own code in a hand-built world, read through the
//! mirror and through the owner's type, and compared field by field; each mirror request is decoded by
//! economy's own type. A renamed field or a bumped schema turns this red.

mod support;

use mineworld_contracts::{
    Action, Component, ComponentRecord, EntityId, Observation, PerceivedEntity,
};
use mineworld_economy::{
    AsksView, CurrencyRecord, ListForSale, Listing, SetPrice, Shop, ShopSide, SymbolAt,
};
use mineworld_inventory::Holdings;
use mineworld_item::{CATEGORY_MAX_BYTES, Category, ItemCatalogue};
use mineworld_rule_controller::price::block::{CATEGORY_MAX_BYTES as MIRRORED_MAX, is_category};
use mineworld_rule_controller::price::wire;
use mineworld_rule_controller::price::wire::{
    AsksSeen, CatalogueSeen, CurrencySeen, HoldingsSeen, ListingSeen, ShopSideSeen, SymbolAtSeen,
};
use mineworld_rule_controller::read;
use serde::de::DeserializeOwned;
use serde_json::Value;
use support::{SHOP, Setup, Town, t};

/// The one record of `component_type` on `entity`, with its labels checked against the mirror `M` and
/// the owner's type `O`: type, owner and schema version all equal.
fn labelled<M: Component, O: Component>(
    entity: &PerceivedEntity<Value>,
) -> &ComponentRecord<Value> {
    assert_eq!(M::COMPONENT_TYPE, O::COMPONENT_TYPE);
    assert_eq!(M::OWNER, O::OWNER, "{}", M::COMPONENT_TYPE);
    assert_eq!(
        M::SCHEMA_VERSION,
        O::SCHEMA_VERSION,
        "{}",
        M::COMPONENT_TYPE
    );
    entity
        .components()
        .iter()
        .find(|record| *record.component_type() == M::COMPONENT_TYPE)
        .unwrap_or_else(|| panic!("{} is disclosed", M::COMPONENT_TYPE))
}

/// The record read through the mirror (labels checked by `payload_for`) and through the owner's type.
fn both<M: Component + DeserializeOwned, O: DeserializeOwned>(
    entity: &PerceivedEntity<Value>,
    record: &ComponentRecord<Value>,
) -> (M, O) {
    let mirror: M = read::disclosed(entity).expect("the mirror decodes the owner's record");
    let owner: O = serde_json::from_value(record.payload().clone()).expect("the owner decodes it");
    (mirror, owner)
}

fn entity(observation: &Observation<Value>, id: EntityId) -> &PerceivedEntity<Value> {
    observation.entity(id).expect("perceived")
}

/// The store town in US cents with references; erin (carrying pens) looks round the store, where grace
/// sells bread of her own.
fn store() -> Town {
    let setup = Setup::store(SHOP)
        .with_reference("bread", "{ reference: 300 }")
        .with_reference("apple", "{ reference: 90 }");
    let mut setup = setup;
    setup.configuration =
        Some("{ currency: { name: US dollar, symbol: $, exponent: 2, symbol_at: before } }".into());
    Town::build(&setup, |_| {}).expect("the store begins")
}

/// A crown town of exponent 0 with no shop at all: only grace's own asks.
fn crowns() -> Town {
    let mut setup = Setup::store(SHOP)
        .with_economy("corner-store", "{ wallet: 1000 }")
        .with_reference("bread", "{ reference: 3 }");
    setup.configuration =
        Some("{ currency: { name: crown, symbol: cr, exponent: 0, symbol_at: after } }".into());
    Town::build(&setup, |_| {}).expect("the crown town begins")
}

fn check_currency(observation: &Observation<Value>, me: EntityId) {
    let mine = entity(observation, me);
    let record = labelled::<CurrencySeen, CurrencyRecord>(mine);
    let (mirror, owner): (CurrencySeen, CurrencyRecord) = both(mine, record);
    assert_eq!(mirror.name, owner.name());
    assert_eq!(mirror.symbol, owner.symbol());
    assert_eq!(mirror.exponent, owner.exponent());
    let at = match owner.symbol_at() {
        SymbolAt::Before => SymbolAtSeen::Before,
        SymbolAt::After => SymbolAtSeen::After,
    };
    assert_eq!(mirror.symbol_at, at);
    assert_eq!(mirror.references.len(), owner.references().len());
    assert!(!owner.references().is_empty(), "references are compared");
    for (seen, line) in mirror.references.iter().zip(owner.references()) {
        assert_eq!((seen.item, seen.value), (line.item(), line.value()));
    }
}

fn check_asks(observation: &Observation<Value>, seller: EntityId) {
    let theirs = entity(observation, seller);
    let record = labelled::<AsksSeen, mineworld_economy::Asks>(theirs);
    let (mirror, owner): (AsksSeen, AsksView) = both(theirs, record);
    assert!(!owner.asks().is_empty(), "asks are compared");
    assert_eq!(mirror.asks.len(), owner.asks().len());
    for (seen, line) in mirror.asks.iter().zip(owner.asks()) {
        assert_eq!(
            (seen.item, seen.price, seen.available),
            (line.item(), line.price(), line.available())
        );
    }
}

/// AC-d-7, the shop world: the listing, the catalogue, the currency, holdings and a person's asks.
#[test]
fn the_mirrors_read_a_shop_world_exactly_as_its_owners_wrote_it() {
    let town = store();
    let observation = town.observation("erin", t(60));
    let store_id = town.id("store");
    let place = entity(&observation, store_id);

    let record = labelled::<ListingSeen, Shop>(place);
    let (mirror, owner): (ListingSeen, Listing) = both(place, record);
    assert_eq!(mirror.operator, owner.operator());
    assert!(!owner.listed().is_empty(), "the listing is compared");
    assert_eq!(mirror.listed.len(), owner.listed().len());
    for (seen, line) in mirror.listed.iter().zip(owner.listed()) {
        assert_eq!(
            (seen.item, seen.price, seen.in_stock),
            (line.item(), line.price(), line.in_stock())
        );
    }

    let record = labelled::<CatalogueSeen, ItemCatalogue>(place);
    let (mirror, owner): (CatalogueSeen, ItemCatalogue) = both(place, record);
    assert!(!owner.kinds().is_empty(), "the catalogue is compared");
    assert_eq!(mirror.kinds.len(), owner.kinds().len());
    for (seen, kind) in mirror.kinds.iter().zip(owner.kinds()) {
        assert_eq!(seen.item, kind.item());
        assert_eq!(seen.category, kind.category().as_str());
        assert_eq!(seen.name, kind.name().as_str());
    }

    let erin = town.id("erin");
    let mine = entity(&observation, erin);
    let record = labelled::<HoldingsSeen, Holdings>(mine);
    let (mirror, owner): (HoldingsSeen, Holdings) = both(mine, record);
    assert!(!owner.held().is_empty(), "holdings are compared");
    assert_eq!(mirror.held.len(), owner.held().len());
    for (seen, held) in mirror.held.iter().zip(owner.held()) {
        assert_eq!((seen.item, seen.count), (held.item(), held.count()));
    }

    check_currency(&observation, erin);
    check_asks(&observation, town.id("grace"));
}

/// AC-d-7, the second world: a crown of exponent 0, a person seller's asks, and no shop.
#[test]
fn the_mirrors_read_a_crown_world_without_a_shop() {
    let town = crowns();
    let observation = town.observation("alice", t(60));
    check_currency(&observation, town.id("alice"));
    check_asks(&observation, town.id("grace"));
    let place = entity(&observation, town.id("store"));
    assert!(
        read::disclosed::<ListingSeen>(place).is_none(),
        "no shop, no listing"
    );
}

/// The mirror requests are economy's requests: same labels, and economy's own types decode them to the
/// same values.
#[test]
fn the_mirror_requests_are_economys_requests() {
    let town = store();
    let bread = town.item("bread");
    assert_eq!(wire::SetPrice::ACTION_TYPE, SetPrice::ACTION_TYPE);
    assert_eq!(wire::SetPrice::OWNER, SetPrice::OWNER);
    assert_eq!(wire::SetPrice::CATEGORY, SetPrice::CATEGORY);
    assert_eq!(wire::ListForSale::ACTION_TYPE, ListForSale::ACTION_TYPE);
    assert_eq!(wire::ListForSale::OWNER, ListForSale::OWNER);
    assert_eq!(wire::BUY, mineworld_economy::Buy::ACTION_TYPE);
    for side in [None, Some(ShopSideSeen::Shop)] {
        let mirror = wire::SetPrice {
            item: bread,
            price: 303,
            side,
        };
        let encoded = serde_json::to_value(mirror).expect("encodes");
        let owner: SetPrice = serde_json::from_value(encoded.clone()).expect("economy decodes it");
        let expected = SetPrice::new(bread, 303, side.map(|_| ShopSide::Shop));
        assert_eq!(owner, expected);
        assert_eq!(serde_json::to_value(expected).expect("encodes"), encoded);

        let mirror = wire::ListForSale {
            item: bread,
            price: 250,
            side,
        };
        let encoded = serde_json::to_value(mirror).expect("encodes");
        let owner: ListForSale =
            serde_json::from_value(encoded.clone()).expect("economy decodes it");
        let expected = ListForSale::new(bread, 250, side.map(|_| ShopSide::Shop));
        assert_eq!(owner, expected);
        assert_eq!(serde_json::to_value(expected).expect("encodes"), encoded);
    }
    let buy = mineworld_economy::Buy::new(bread, 300);
    let seen: wire::BuySeen =
        serde_json::from_value(serde_json::to_value(buy).expect("encodes")).expect("decodes");
    assert_eq!((seen.item, seen.price), (buy.item(), buy.price()));
}

/// The category rule `rule-controller` restates is item's.
#[test]
fn the_restated_category_rule_is_items() {
    assert_eq!(MIRRORED_MAX, CATEGORY_MAX_BYTES);
    let long = "a".repeat(CATEGORY_MAX_BYTES);
    let longer = "a".repeat(CATEGORY_MAX_BYTES + 1);
    for candidate in [
        "food",
        "drink",
        "goods",
        "dry-goods",
        "a1",
        "",
        "-food",
        "food-",
        "Food",
        "fo od",
        "fööd",
        long.as_str(),
        longer.as_str(),
    ] {
        assert_eq!(
            is_category(candidate),
            Category::new(candidate).is_ok(),
            "{candidate:?}"
        );
    }
}
