//! Buying: the one judgement both the offer and the dispatch call, and the one settlement that states
//! a sale's three facts together (`ARC-112`; `pr-s31a-currency-asks.md` §5.7).
//!
//! No reference value is read here: a sale is at the seller's asking price, never at a kind's typical
//! worth (`ARC-113`, I-S31-6).

use mineworld_contracts::{
    EntityId, Event, ItemId, Location, PlaceId, Rejection, SpatialRequirement, Visibility,
};
use mineworld_inventory::{Holdings, InventorySystem, ItemsTransferred};
use mineworld_kernel::{Emission, KernelError, SystemIdentity, WorldRead};
use mineworld_presence::{Offer, Presence};

use crate::action::{Buy, buy_requirement, person_buy_requirement};
use crate::code::{
    self, CANNOT_CARRY, INSUFFICIENT_FUNDS, NOT_FOR_SALE, OUT_OF_STOCK, PRICE_CHANGED,
};
use crate::codec;
use crate::component::Asks;
use crate::event::{MoneyTransferred, Sold};
use crate::money::{admit_payment, balance, pay};
use crate::offer::{asking_price, asks_of, living_person, shop_here};
use crate::system::refused;

/// Who sells, as the buyer meets them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Seller {
    /// The shop at this place; its operator is paid.
    Shop { place: PlaceId, operator: EntityId },
    /// This person, selling for itself.
    Person(EntityId),
}

/// A sale the rules admit: who sells, the kind, the price, and what it requires of space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Sale {
    seller: Seller,
    item: ItemId,
    price: u64,
}

impl Sale {
    /// Who is paid and gives the item.
    const fn seller(&self) -> EntityId {
        match self.seller {
            Seller::Shop { operator, .. } => operator,
            Seller::Person(person) => person,
        }
    }

    /// What the sale requires of space.
    pub(crate) fn requirement(&self) -> SpatialRequirement {
        match self.seller {
            Seller::Shop { place, .. } => buy_requirement(place),
            Seller::Person(_) => person_buy_requirement(),
        }
    }
}

/// The one judgement of a buy (§5.7), in this order: the buyer a living Person; the seller — the
/// target, a living Person other than the buyer with asks, or with no target the shop where the buyer
/// stands; an ask for the kind; the submitted price equal to the asking price (when one is
/// submitted); no self-trade; the buyer's balance; the seller's stock; the buyer's capacity; and the
/// two owners' own rules, asked last so the codes cannot disagree with them (`ARC-26`).
pub(crate) fn judge_sale(
    world: &WorldRead<'_>,
    buyer: EntityId,
    target: Option<EntityId>,
    item: ItemId,
    submitted: Option<u64>,
) -> Result<Sale, Rejection> {
    if !living_person(world, buyer) {
        return Err(Rejection::NoSupportedInteraction);
    }
    let (seller, asking, keeper) = match target {
        Some(person) => {
            if person == buyer
                || !living_person(world, person)
                || world.component::<Asks>(person).is_none()
            {
                return Err(Rejection::NoSupportedInteraction);
            }
            (
                Seller::Person(person),
                asks_of(world, person).price(item),
                false,
            )
        }
        None => {
            let (place, shop) = shop_here(world, buyer).ok_or(Rejection::NoSupportedInteraction)?;
            let operator = shop.operator().entity_id();
            (
                Seller::Shop { place, operator },
                asking_price(world, &shop, item),
                shop.kept_by(buyer),
            )
        }
    };
    let price = asking.ok_or_else(|| code::refusal(NOT_FOR_SALE))?;
    if submitted.is_some_and(|submitted| submitted != price) {
        return Err(code::refusal(PRICE_CHANGED));
    }
    let sale = Sale {
        seller,
        item,
        price,
    };
    let from = sale.seller();
    if from == buyer || keeper {
        return Err(Rejection::PreconditionFailed);
    }
    if balance(world, buyer) < price {
        return Err(code::refusal(INSUFFICIENT_FUNDS));
    }
    if world
        .component::<Holdings>(from)
        .is_none_or(|held| held.count(item) == 0)
    {
        return Err(code::refusal(OUT_OF_STOCK));
    }
    if !mineworld_inventory::can_take(world, buyer, 1) {
        return Err(code::refusal(CANNOT_CARRY));
    }
    admit_payment(world, buyer, from, price)?;
    mineworld_inventory::admit_transfer(world, from, buyer, item, 1)?;
    Ok(sale)
}

/// Where a person stands, if anywhere.
pub(crate) fn located(world: &WorldRead<'_>, entity: EntityId) -> Option<Location> {
    world.component::<Presence>(entity).map(Presence::location)
}

/// A buy's whole decision at dispatch: [`judge_sale`] with the submitted price, then the requirement
/// (SD-S31a-2: the offer reports its refusal before its requirement, so the dispatch does the same).
pub(crate) fn validate(
    world: &WorldRead<'_>,
    buyer: EntityId,
    target: Option<EntityId>,
    buy: &Buy,
) -> Result<(), Rejection> {
    let sale = judge_sale(world, buyer, target, buy.item(), Some(buy.price()))?;
    let here = located(world, buyer).ok_or(Rejection::PreconditionFailed)?;
    let there = target.and_then(|target| located(world, target));
    sale.requirement().evaluate(&here, there.as_ref(), true)
}

/// The sale's three facts, in this order: `sold`, economy's `money-transferred` and inventory's
/// `items-transferred`, each built by its owner's checked constructor, all heard where the sale
/// happens. Re-judged from the state at resolve; a refusal here is the named owner's
/// `FactRefusedByOwner`.
pub(crate) fn settle(
    world: &WorldRead<'_>,
    buyer: EntityId,
    target: Option<EntityId>,
    buy: &Buy,
) -> Result<Vec<Emission>, KernelError> {
    let sale = judge_sale(world, buyer, target, buy.item(), Some(buy.price()))
        .map_err(|reason| refused(Sold::EVENT_TYPE, reason))?;
    let place = match sale.seller {
        Seller::Shop { place, .. } => place,
        Seller::Person(_) => located(world, buyer)
            .map(|location| location.place())
            .ok_or_else(|| refused(Sold::EVENT_TYPE, Rejection::PreconditionFailed))?,
    };
    let heard = Visibility::Place(place);
    let seller = sale.seller();
    let sold = Emission::new::<Sold>(
        codec::encode(&Sold::new(seller, buyer, sale.item, sale.price, place)),
        heard.clone(),
    )
    .about(vec![buyer, seller])
    .with_participants(vec![buyer, seller])
    .at_place(place);
    let paid = pay(world, buyer, seller, sale.price, heard)
        .map_err(|reason| refused(MoneyTransferred::EVENT_TYPE, reason))?;
    let goods =
        mineworld_inventory::transfer(world, seller, buyer, sale.item, 1).map_err(|reason| {
            KernelError::FactRefusedByOwner {
                system: InventorySystem::ID,
                event_type: ItemsTransferred::EVENT_TYPE,
                reason,
            }
        })?;
    Ok(vec![sold, paid, goods])
}

/// The buys `observer` is offered: with no target, standing in a shop, one **complete** `buy { item,
/// price }` per ask of the operator; with a target, a living person seller other than the observer,
/// one per ask of that seller. Each is `refused(code)` when [`judge_sale`] (without a submitted price)
/// refuses, so the offer and the dispatch answer the same (I-a-10).
pub(crate) fn buys(
    world: &WorldRead<'_>,
    observer: EntityId,
    target: Option<EntityId>,
) -> Vec<Offer> {
    if !living_person(world, observer) {
        return Vec::new();
    }
    let (asks, requirement, shop) = match target {
        Some(person) => {
            if person == observer || !living_person(world, person) {
                return Vec::new();
            }
            let Some(asks) = world.component::<Asks>(person) else {
                return Vec::new();
            };
            (asks.clone(), person_buy_requirement(), None)
        }
        None => {
            let Some((place, shop)) = shop_here(world, observer) else {
                return Vec::new();
            };
            (
                asks_of(world, shop.operator().entity_id()),
                buy_requirement(place),
                Some(shop),
            )
        }
    };
    asks.asks()
        .iter()
        .map(|ask| {
            let price = shop
                .as_ref()
                .map_or(ask.price(), |shop| shop.adjust(ask.price()));
            let offer =
                Offer::complete(&Buy::new(ask.item(), price), requirement).expect("a buy encodes");
            match judge_sale(world, observer, target, ask.item(), None) {
                Ok(_) => offer,
                Err(reason) => offer.refused(reason),
            }
        })
        .collect()
}
