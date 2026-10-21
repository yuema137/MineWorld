//! The world's currency is configuration (`ARC-111`; S31-a AC-a-7, the record of §5.3): decoding
//! `configure/economy.yaml` is validating, with each bound refused naming its key; the declared currency
//! is held by one never-woken process; every living Person observer is told it about itself, wherever it
//! stands; a second declaration is refused; and a world that authors an amount without a currency is
//! refused at genesis, naming `economy-currency-undeclared`.

mod support;

use std::collections::BTreeMap;

use mineworld_authoring::{
    Attached, ConfigurationContext, EntityClasses, PackConfiguration, Seeding,
};
use mineworld_contracts::{Component, EntityType, Event, Rejection};
use mineworld_economy::code::CURRENCY_UNDECLARED;
use mineworld_economy::{
    CurrencyDeclared, CurrencyProcess, CurrencyRecord, EconomySystem, Funded, SymbolAt,
};
use mineworld_inventory::InventorySystem;
use mineworld_item::ItemSystem;
use mineworld_kernel::{Interruptibility, KernelError, ProcessKind, SystemIdentity, World};
use mineworld_presence::PresenceSystem;
use support::{CURRENCY, GENESIS, SHOP, STOCK, Town, configured, key};

/// The `currency` record `observer` is told about `subject`, if any.
fn record_about(town: &Town, observer: &str, subject: &str) -> Option<CurrencyRecord> {
    let seen = town.observation(observer, GENESIS);
    seen.entity(town.id(subject))?
        .components()
        .iter()
        .find(|record| *record.component_type() == CurrencyRecord::COMPONENT_TYPE)
        .map(|record| serde_json::from_value(record.payload().clone()).expect("a currency record"))
}

#[test]
fn every_bound_of_the_configuration_is_refused_at_its_line_naming_the_key() {
    let valid = configured(CURRENCY).expect("the store's currency decodes");
    let currency = valid.currency();
    assert_eq!(
        (
            currency.name(),
            currency.symbol(),
            currency.exponent(),
            currency.symbol_at()
        ),
        ("test coin", "tc", 2, SymbolAt::After)
    );
    for exponent in [0, 4] {
        let text = format!(
            "currency: {{ name: crown, symbol: cr, exponent: {exponent}, symbol_at: before }}"
        );
        assert!(
            configured(&text).is_ok(),
            "exponent {exponent} is in bounds"
        );
    }
    let name = "n".repeat(65);
    let refused = [
        ("exponent: 5", "exponent", "an exponent of 5"),
        (&*format!("name: {name}"), "name", "a 65-byte name"),
        ("symbol: \"123456789\"", "symbol", "a 9-byte symbol"),
        (
            "symbol: \" $\"",
            "symbol",
            "a symbol with leading whitespace",
        ),
        ("symbol_at: middle", "symbol_at", "symbol_at: middle"),
        ("colour: gold", "colour", "an unknown key"),
    ];
    for (line, named, what) in refused {
        let mut fields: BTreeMap<&str, String> = [
            ("name", "name: crown".to_owned()),
            ("symbol", "symbol: cr".to_owned()),
            ("exponent", "exponent: 0".to_owned()),
            ("symbol_at", "symbol_at: after".to_owned()),
        ]
        .into();
        let key = line.split(':').next().expect("a key");
        fields.insert(key, line.to_owned());
        // The changed line is the third line of the file, under `currency:`.
        let mut text = String::from("# configure/economy.yaml\ncurrency:\n");
        text.push_str(&format!("  {}\n", fields[key]));
        for (other, value) in &fields {
            if *other != key {
                text.push_str(&format!("  {value}\n"));
            }
        }
        let error = configured(&text).expect_err(what).to_string();
        assert!(error.contains(named), "{what}: names `{named}`: {error}");
        assert!(error.contains('3'), "{what}: at line 3: {error}");
    }
    assert!(
        configured("{}")
            .expect_err("no currency")
            .to_string()
            .contains("currency"),
        "a configuration without `currency` is refused naming it"
    );
}

#[test]
fn the_currency_is_held_by_one_process_that_is_never_woken() {
    let town = Town::begun(SHOP);
    let read = town.world.read();
    let processes: Vec<_> = read
        .processes()
        .filter(|process| *process.process_type() == CurrencyProcess::PROCESS_TYPE)
        .collect();
    assert_eq!(processes.len(), 1, "one currency process");
    let process = processes[0];
    assert_eq!(*process.owner(), EconomySystem::ID);
    assert_eq!(process.expected_end(), None, "never woken");
    assert_eq!(
        process.interruptibility(),
        Interruptibility::Uninterruptible
    );
    let declared = town
        .genesis
        .iter()
        .filter(|fact| *fact.event_type() == CurrencyDeclared::EVENT_TYPE)
        .collect::<Vec<_>>();
    assert_eq!(declared.len(), 1, "one currency-declared at genesis");
    assert_eq!(
        *declared[0].visibility(),
        mineworld_contracts::Visibility::SystemInternal,
        "configuration is nobody's perception (Q-S31a-2)"
    );
    assert!(declared[0].subjects().is_empty(), "about nobody");
}

#[test]
fn every_living_person_is_told_the_currency_about_itself_wherever_it_stands() {
    let town = Town::begun(SHOP);
    let in_the_store = record_about(&town, "alice", "alice").expect("alice, in the store");
    let on_the_street = record_about(&town, "carol", "carol").expect("carol, on the street");
    assert_eq!(
        in_the_store, on_the_street,
        "the record is the same for every observer"
    );
    assert_eq!(
        (
            in_the_store.name(),
            in_the_store.symbol(),
            in_the_store.exponent(),
            in_the_store.symbol_at()
        ),
        ("test coin", "tc", 2, SymbolAt::After)
    );
    assert!(
        record_about(&town, "bob", "alice").is_none(),
        "bob perceives alice and is told the currency about himself, not about her"
    );
    // Economy's own judgement does not depend on the observer's place: asked directly about an
    // observer, it answers the same for one with no shop and one in a shop.
    let read = town.world.read();
    let direct = |name: &str| {
        mineworld_presence::PerceptionProvider::discloses(
            &EconomySystem,
            &read,
            town.id(name),
            town.id(name),
        )
        .into_iter()
        .filter(|record| *record.component_type() == CurrencyRecord::COMPONENT_TYPE)
        .count()
    };
    assert_eq!((direct("alice"), direct("carol")), (1, 1));
    assert_eq!(
        mineworld_presence::PerceptionProvider::discloses(
            &EconomySystem,
            &read,
            town.id("alice"),
            town.id("store"),
        )
        .iter()
        .filter(|record| *record.component_type() == CurrencyRecord::COMPONENT_TYPE)
        .count(),
        0,
        "never about a place"
    );
}

#[test]
fn a_world_that_authors_an_amount_without_a_currency_is_refused_naming_the_code() {
    match Town::begun_configured(SHOP, STOCK, None, |_| {}) {
        Err(KernelError::FactRefusedByOwner {
            system,
            event_type,
            reason,
        }) => {
            assert_eq!(system, EconomySystem::ID);
            assert_eq!(event_type, Funded::EVENT_TYPE, "the first amount reduced");
            assert_eq!(
                reason,
                Rejection::System {
                    code: CURRENCY_UNDECLARED,
                    detail: None
                }
            );
        }
        Ok(_) => panic!("a world with wallets and no currency began"),
        Err(other) => panic!("refused for another reason: {other:?}"),
    }
}

#[test]
fn a_second_currency_is_refused() {
    let mut world = World::new();
    world.install(PresenceSystem).expect("presence");
    world.install(ItemSystem).expect("item");
    world.install(InventorySystem).expect("inventory");
    world.install(EconomySystem).expect("economy");
    world
        .create_entity(key("someone"), EntityType::Person)
        .expect("created");
    let (classes, attached) = (EntityClasses::default(), Attached::none());
    let facts: Vec<_> = {
        let read = world.read();
        let keys = BTreeMap::new();
        let seeding = Seeding::new(&read, &keys);
        let once = || {
            <EconomySystem as PackConfiguration>::seed(
                &seeding,
                &configured(CURRENCY).expect("decodes"),
                &ConfigurationContext::new(&classes, &attached),
            )
            .expect("seeds")
        };
        once().into_iter().chain(once()).collect()
    };
    match world.genesis(GENESIS, facts) {
        Err(KernelError::FactRefusedByOwner {
            system, event_type, ..
        }) => {
            assert_eq!(system, EconomySystem::ID);
            assert_eq!(event_type, CurrencyDeclared::EVENT_TYPE);
        }
        other => panic!("a second currency must be refused by its owner: {other:?}"),
    }
}
