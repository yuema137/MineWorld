//! What a person is offered in a shop, and what a shop's place discloses.

use mineworld_contracts::{EntityId, EntityType, ItemId, LifecycleState, PlaceId};
use mineworld_inventory::Holdings;
use mineworld_kernel::WorldRead;
use mineworld_presence::{Offer, Presence};

use crate::action::{Buy, buy_requirement, purchasable};
use crate::component::{Asks, Listing, Shop};

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
    asks_of(world, shop.operator().entity_id())
        .asks()
        .iter()
        .map(|ask| {
            Offer::complete(&Buy::new(ask.item()), buy_requirement(place))
                .expect("a buy encodes")
                .with_target_available(purchasable(world, observer, &shop, ask.item()))
        })
        .collect()
}

/// A seller's asks; none when it has never listed anything.
pub(crate) fn asks_of(world: &WorldRead<'_>, seller: EntityId) -> Asks {
    world.component::<Asks>(seller).cloned().unwrap_or_default()
}

/// What `item` costs in `shop` now: its operator's ask, if it has one, scaled by the shop's adjustments
/// ([`Shop::adjust`]).
pub(crate) fn asking_price(world: &WorldRead<'_>, shop: &Shop, item: ItemId) -> Option<u64> {
    world
        .component::<Asks>(shop.operator().entity_id())
        .and_then(|asks| asks.price(item))
        .map(|ask| shop.adjust(ask))
}

/// A shop's listing as it stands: its operator's asks, and how many of each the operator holds now.
pub(crate) fn listing(world: &WorldRead<'_>, shop: &Shop) -> Listing {
    let operator = shop.operator().entity_id();
    let stock = world.component::<Holdings>(operator);
    Listing::of(shop, asks_of(world, operator).asks(), |item| {
        stock.map_or(0, |held| held.count(item))
    })
}
