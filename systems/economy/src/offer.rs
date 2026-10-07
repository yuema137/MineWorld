//! What a person is offered in a shop, and what a shop's place discloses.

use mineworld_contracts::{EntityId, EntityType, LifecycleState, PlaceId};
use mineworld_inventory::Holdings;
use mineworld_kernel::WorldRead;
use mineworld_presence::{Offer, Presence};

use crate::action::{Buy, buy_requirement, purchasable};
use crate::component::{Listing, Shop};

/// Whether `entity` is a living Person: the only buyer.
pub(crate) fn living_person(world: &WorldRead<'_>, entity: EntityId) -> bool {
    world.entity(entity).is_some_and(|record| {
        record.entity_type() == EntityType::Person
            && record.lifecycle() != LifecycleState::Destroyed
    })
}

/// The shop where `person` stands, if any, and its place.
pub(crate) fn shop_here(world: &WorldRead<'_>, person: EntityId) -> Option<(PlaceId, Shop)> {
    let place = world.component::<Presence>(person)?.location().place();
    let shop = world.component::<Shop>(place.entity_id())?.clone();
    Some((place, shop))
}

/// The buys `observer` may attempt: none with a target, none for anything but a living Person, none
/// outside a shop. In a shop, one **complete** offer per priced kind, in item order, so two runs
/// observe the same offers (`AC-12`); available when the purchase could happen now, otherwise offered
/// unavailable so a client can still show what is for sale (step-10 QS-44).
pub(crate) fn buys(
    world: &WorldRead<'_>,
    observer: EntityId,
    target: Option<EntityId>,
) -> Vec<Offer> {
    if target.is_some() || !living_person(world, observer) {
        return Vec::new();
    }
    let Some((place, shop)) = shop_here(world, observer) else {
        return Vec::new();
    };
    shop.prices()
        .iter()
        .map(|price| {
            Offer::complete(&Buy::new(price.item()), buy_requirement(place))
                .expect("a buy encodes")
                .with_target_available(purchasable(world, observer, &shop, price.item()))
        })
        .collect()
}

/// A shop's listing as it stands: its prices, and how many of each the operator holds now.
pub(crate) fn listing(world: &WorldRead<'_>, shop: &Shop) -> Listing {
    let stock = world.component::<Holdings>(shop.operator().entity_id());
    Listing::of(shop, |item| stock.map_or(0, |held| held.count(item)))
}
