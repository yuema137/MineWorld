//! E-7, the economy half of `AC-2` at pack level.
//!
//! - Economy installed **without employment** loads and sells: it subscribes to `wage-due` and does
//!   not depend on employment (`ARC-28`).
//! - A world **without economy** offers no `buy` and answers one `Unavailable`; nobody's holdings
//!   change.
//! - Economy without inventory is refused by the registry, naming inventory.

mod support;

use mineworld_contracts::ActionResult;
use mineworld_economy::EconomySystem;
use mineworld_employment::EmploymentSystem;
use mineworld_inventory::InventorySystem;
use mineworld_item::ItemSystem;
use mineworld_kernel::{KernelError, System, SystemIdentity, World};
use mineworld_presence::PresenceSystem;
use support::{GENESIS, Packs, SHOP, Town, buys};

/// The only switch for the "without economy" world. Set to `true`, that world enables economy after
/// all — a negative control that shows the assertions can fail.
const WITHOUT_ECONOMY: bool = false;

#[test]
fn economy_without_employment_installs_and_sells() {
    assert!(
        !EconomySystem
            .declaration()
            .depends_on()
            .contains(&EmploymentSystem::ID),
        "economy hears wage-due without depending on employment"
    );
    let mut town = Town::begun(SHOP);
    let done = town.buy("alice", "apple", 1).expect("dispatch answers");
    assert!(
        matches!(done.result(), ActionResult::Accepted { .. }),
        "a world with economy and no employment sells: {:?}",
        done.result()
    );
}

#[test]
fn a_world_without_economy_offers_no_buy_answers_unavailable_and_holds_still() {
    let mut town = Town::begun(Packs {
        economy: WITHOUT_ECONOMY,
        employment: false,
    });
    for observer in ["alice", "bob", "carol", "erin"] {
        assert!(
            buys(&town.observation(observer, GENESIS)).is_empty(),
            "{observer} is offered a buy in a world without economy"
        );
    }
    let done = town.buy("alice", "apple", 1).expect("dispatch answers");
    assert_eq!(*done.result(), ActionResult::Unavailable);
    assert!(done.events().is_empty());
    assert!(town.holdings("alice").is_empty(), "holdings never change");
    assert_eq!(town.wallet("alice"), None, "and nobody has a wallet");
}

#[test]
fn a_world_enabling_economy_without_inventory_is_refused_naming_inventory() {
    let mut world = World::new();
    world.install(PresenceSystem).expect("presence");
    world.install(ItemSystem).expect("item");
    match world.install(EconomySystem) {
        Err(KernelError::SystemDependencyMissing { system, dependency }) => {
            assert_eq!(system, EconomySystem::ID);
            assert_eq!(dependency, InventorySystem::ID);
        }
        other => panic!("the registry must refuse, naming inventory: {other:?}"),
    }
}
