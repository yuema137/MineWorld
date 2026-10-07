//! What a person is offered: one complete eat or drink per edible or drinkable kind they carry.

use mineworld_contracts::{EntityId, EntityType, ItemId, LifecycleState, SpatialRequirement};
use mineworld_inventory::Holdings;
use mineworld_item::ItemKind;
use mineworld_kernel::WorldRead;
use mineworld_presence::Offer;

use crate::action::{DRUNK, Drink, EATEN, Eat};

/// Whether `entity` is a living Person: the only one who eats.
pub(crate) fn living_person(world: &WorldRead<'_>, entity: EntityId) -> bool {
    world.entity(entity).is_some_and(|record| {
        record.entity_type() == EntityType::Person
            && record.lifecycle() != LifecycleState::Destroyed
    })
}

/// The category of a declared kind, read from item's `ItemKind`.
pub(crate) fn category<'w>(world: &WorldRead<'w>, item: ItemId) -> Option<&'w str> {
    world
        .component::<ItemKind>(item.entity_id())
        .map(|kind| kind.category().as_str())
}

/// The meals `observer` may attempt: none with a target, none for anything but a living Person, none
/// for goods. Otherwise one **complete** offer per edible or drinkable kind held, in item order, with
/// no spatial requirement — a person eats what they carry, wherever they are (step-10 QS-40).
pub(crate) fn meals(
    world: &WorldRead<'_>,
    observer: EntityId,
    target: Option<EntityId>,
) -> Vec<Offer> {
    if target.is_some() || !living_person(world, observer) {
        return Vec::new();
    }
    let Some(holdings) = world.component::<Holdings>(observer) else {
        return Vec::new();
    };
    holdings
        .held()
        .iter()
        .filter_map(|held| {
            let item = held.item();
            match category(world, item) {
                Some(EATEN) => Some(Offer::complete(&Eat::new(item), SpatialRequirement::NONE)),
                Some(DRUNK) => Some(Offer::complete(&Drink::new(item), SpatialRequirement::NONE)),
                _ => None,
            }
        })
        .map(|offer| offer.expect("a meal encodes"))
        .collect()
}
