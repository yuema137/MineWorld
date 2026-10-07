//! E-7, the consumption half of `AC-2` at pack level: a world without consumption offers no `eat` or
//! `drink`, answers both `Unavailable`, and nobody's holdings change; consumption without inventory is
//! refused by the registry, naming inventory.

mod support;

use mineworld_consumption::ConsumptionSystem;
use mineworld_contracts::ActionResult;
use mineworld_inventory::InventorySystem;
use mineworld_item::ItemSystem;
use mineworld_kernel::{KernelError, SystemIdentity, World};
use mineworld_presence::PresenceSystem;
use support::{GENESIS, HOLDINGS, Meal, Town, meals};

/// The only switch for the "without" world. Set to `true`, that world enables consumption after all —
/// the negative control (M-E7) that shows the assertions can fail.
const WITHOUT: bool = false;

#[test]
fn a_world_without_consumption_offers_no_meal_answers_unavailable_and_holds_still() {
    let mut town = Town::begun(WITHOUT, &HOLDINGS);
    for observer in ["alice", "bob", "carol"] {
        assert!(
            meals(&town.observation(observer, GENESIS)).is_empty(),
            "{observer} is offered a meal in a world without consumption"
        );
    }
    let before = town.holdings("alice");
    for (id, meal, item) in [(1, Meal::Eat, "croissant"), (2, Meal::Drink, "coffee")] {
        let done = town
            .meal("alice", meal, item, None, id)
            .expect("dispatch answers");
        assert_eq!(*done.result(), ActionResult::Unavailable, "{item}");
        assert!(done.events().is_empty());
    }
    assert_eq!(town.holdings("alice"), before, "holdings never change");
}

#[test]
fn a_world_enabling_consumption_without_inventory_is_refused_naming_inventory() {
    let mut world = World::new();
    world.install(PresenceSystem).expect("presence");
    world.install(ItemSystem).expect("item");
    match world.install(ConsumptionSystem) {
        Err(KernelError::SystemDependencyMissing { system, dependency }) => {
            assert_eq!(system, ConsumptionSystem::ID);
            assert_eq!(dependency, InventorySystem::ID);
        }
        other => panic!("the registry must refuse, naming inventory: {other:?}"),
    }
}
