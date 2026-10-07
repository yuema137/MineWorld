//! What a person is offered: one complete give per kind they hold, to each other person present.

use mineworld_contracts::{EntityId, EntityType, LifecycleState};
use mineworld_inventory::Holdings;
use mineworld_kernel::WorldRead;
use mineworld_presence::Offer;

use crate::action::{Give, give_requirement};

/// Whether `entity` is a living Person: the only giver and the only taker of a give.
pub(crate) fn living_person(world: &WorldRead<'_>, entity: EntityId) -> bool {
    world.entity(entity).is_some_and(|record| {
        record.entity_type() == EntityType::Person
            && record.lifecycle() != LifecycleState::Destroyed
    })
}

/// The gives `observer` may attempt against `target`.
///
/// Nothing without a target, to oneself (perception asks about every person present, the observer
/// included — step-10 F-40), to anything but a living Person, or from somebody who holds nothing.
/// Otherwise one **complete** offer per kind held, count 1, in item order, so the order is fixed and
/// two runs of one world observe the same offers (`AC-12`). The target is available when inventory
/// says it can take one more; perception prices place and reach.
pub(crate) fn gives(
    world: &WorldRead<'_>,
    observer: EntityId,
    target: Option<EntityId>,
) -> Vec<Offer> {
    let Some(target) = target else {
        return Vec::new();
    };
    if target == observer || !living_person(world, observer) || !living_person(world, target) {
        return Vec::new();
    }
    let Some(holdings) = world.component::<Holdings>(observer) else {
        return Vec::new();
    };
    let takes = mineworld_inventory::can_take(world, target, 1);
    holdings
        .held()
        .iter()
        .map(|held| {
            Offer::complete(&Give::new(held.item(), 1), give_requirement())
                .expect("a give encodes")
                .with_target_available(takes)
        })
        .collect()
}
