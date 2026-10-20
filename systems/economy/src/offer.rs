//! What this pack reads to offer and to disclose: who may act, the shop where a person stands, a
//! seller's asks, a shop's listing, a person seller's asks view, and a holder's wealth.

use mineworld_contracts::{EntityId, EntityType, ItemId, LifecycleState, PlaceId};
use mineworld_inventory::Holdings;
use mineworld_kernel::WorldRead;
use mineworld_presence::Presence;

use crate::component::{Asks, AsksView, Listing, ReferenceValue, Shop, Wealth};
use crate::money::balance;

/// Whether `entity` is a living Person: the only buyer, and the only actor of a seller action.
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

/// A living person seller's asks view, when it has any ask: per ask, whether it holds one now.
pub(crate) fn asks_view(world: &WorldRead<'_>, seller: EntityId) -> Option<AsksView> {
    if !living_person(world, seller) {
        return None;
    }
    let asks = world.component::<Asks>(seller)?;
    if asks.asks().is_empty() {
        return None;
    }
    let held = world.component::<Holdings>(seller);
    Some(AsksView::of(asks, |item| {
        held.is_some_and(|held| held.count(item) > 0)
    }))
}

/// A holder's wealth (`ARC-114`): its wallet plus count × reference value over the kinds it holds that
/// have one, saturating; and the units it holds of kinds without one. Computed, never stored.
pub(crate) fn wealth(world: &WorldRead<'_>, holder: EntityId) -> Wealth {
    let (mut amount, mut unvalued) = (balance(world, holder), 0_u64);
    if let Some(held) = world.component::<Holdings>(holder) {
        for line in held.held() {
            let count = u64::from(line.count());
            match world.component::<ReferenceValue>(line.item().entity_id()) {
                Some(reference) => {
                    amount = amount.saturating_add(count.saturating_mul(reference.value()));
                }
                None => unvalued = unvalued.saturating_add(count),
            }
        }
    }
    Wealth::new(amount, unvalued)
}
