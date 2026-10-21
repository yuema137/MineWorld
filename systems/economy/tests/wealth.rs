//! Wealth (`ARC-114`; S31-a AC-a-11): wallet plus count × reference value of the kinds held that carry
//! one, and the units of kinds that carry none — a holder-only view, computed when disclosed. The
//! oracles are worked by hand here.

mod support;

use mineworld_contracts::{ActionResult, Component};
use mineworld_economy::Wealth;
use support::{GENESIS, SHOP, Setup, Town};

/// What `holder` is told of its own wealth, if anything: (amount, unvalued).
fn wealth(town: &Town, holder: &str) -> Option<(u64, u64)> {
    let seen = town.observation(holder, GENESIS);
    let record = seen
        .entity(town.id(holder))?
        .components()
        .iter()
        .find(|record| *record.component_type() == Wealth::COMPONENT_TYPE)?;
    let wealth: Wealth = serde_json::from_value(record.payload().clone()).expect("a wealth record");
    Some((wealth.amount(), wealth.unvalued()))
}

#[test]
fn wealth_is_the_wallet_plus_reference_values_of_holdings_and_counts_what_has_none() {
    let setup = Setup::store(SHOP)
        .with_reference("bread", "{ reference: 300 }")
        .with_reference("pen", "{ reference: 150 }")
        .carrying("grace", "{ bread: 2, pen: 1, juice: 1 }");
    let mut town = Town::build(&setup, |_| {}).expect("begins");
    // Grace: 1 000 + 2 × 300 + 1 × 150 = 1 750; the juice has no reference value.
    assert_eq!(wealth(&town, "grace"), Some((1_750, 1)));
    // Erin: 1 000 + 6 × 150 = 1 900.
    assert_eq!(wealth(&town, "erin"), Some((1_900, 0)));
    // Alice holds nothing: her wealth is her wallet.
    assert_eq!(wealth(&town, "alice"), Some((1_000, 0)));

    let done = town
        .buy_at("alice", Some("grace"), "bread", 260, 1)
        .expect("answers");
    assert!(
        matches!(done.result(), ActionResult::Accepted { .. }),
        "{:?}",
        done.result()
    );
    // Grace: 1 260 + 1 × 300 + 1 × 150 = 1 710; Alice: 740 + 300 = 1 040.
    assert_eq!(wealth(&town, "grace"), Some((1_710, 1)));
    assert_eq!(wealth(&town, "alice"), Some((1_040, 0)));
}

#[test]
fn wealth_is_told_to_its_holder_only_and_never_stored() {
    let town = Town::begun(SHOP);
    let seen = town.observation("alice", GENESIS);
    for other in ["bob", "grace", "felix"] {
        let told = seen.entity(town.id(other)).is_some_and(|entity| {
            entity
                .components()
                .iter()
                .any(|record| *record.component_type() == Wealth::COMPONENT_TYPE)
        });
        assert!(!told, "alice is not told {other}'s wealth");
    }
    let stored = town.world.read().components::<Wealth>().count();
    assert_eq!(stored, 0, "no entity carries a wealth component");
}
