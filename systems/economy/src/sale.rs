//! Buying: the one judgement both the offer and the dispatch call, and the one settlement that states
//! a sale's three facts together (`ARC-112`; `pr-s31a-currency-asks.md` §5.7).
//!
//! The judgement is two halves (`pr-s31h-haggling.md` §5.6): [`resolve_seller`] finds who sells and at
//! what ask (steps 1–3), and [`judge_exchange`] asks whether this buyer may pay this seller this price
//! for one unit (steps 5–9). `buy` runs both; an accepted proposal (`ARC-118`) runs the second at the
//! proposed price, through the same code, and settles through the same [`settle`].
//!
//! No reference value is read here: a sale is at an agreed price, never at a kind's typical worth
//! (`ARC-113`, I-S31-6).

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
use crate::component::{Asks, Shop};
use crate::event::{MoneyTransferred, Sold};
use crate::money::{admit_payment, balance, pay};
use crate::offer::{asking_price, asks_of, living_person, shop_here};
use crate::system::refused;

/// Who sells, as the buyer meets them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Seller {
    /// The shop at this place; its operator is paid.
    Shop { place: PlaceId, operator: EntityId },
    /// This person, selling for itself.
    Person(EntityId),
}

impl Seller {
    /// Who is paid and gives the item.
    pub(crate) const fn entity(&self) -> EntityId {
        match *self {
            Self::Shop { operator, .. } => operator,
            Self::Person(person) => person,
        }
    }

    /// The shop's place, for a shop.
    pub(crate) const fn shop_place(&self) -> Option<PlaceId> {
        match *self {
            Self::Shop { place, .. } => Some(place),
            Self::Person(_) => None,
        }
    }
}

/// A sale: who sells, who buys, the kind and the price of one unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Sale {
    pub(crate) seller: Seller,
    pub(crate) buyer: EntityId,
    pub(crate) item: ItemId,
    pub(crate) price: u64,
}

impl Sale {
    /// What a buy of this sale requires of space.
    pub(crate) fn requirement(&self) -> SpatialRequirement {
        match self.seller {
            Seller::Shop { place, .. } => buy_requirement(place),
            Seller::Person(_) => person_buy_requirement(),
        }
    }
}

/// Steps 1–3 of a buy (§5.7): the buyer a living Person; the seller — the target, a living Person
/// other than the buyer with asks, or with no target the shop where the buyer stands; and the
/// seller's current ask for the kind (a shop's after its adjustments).
pub(crate) fn resolve_seller(
    world: &WorldRead<'_>,
    buyer: EntityId,
    target: Option<EntityId>,
    item: ItemId,
) -> Result<(Seller, u64), Rejection> {
    if !living_person(world, buyer) {
        return Err(Rejection::NoSupportedInteraction);
    }
    let (seller, asking) = match target {
        Some(person) => {
            if person == buyer
                || !living_person(world, person)
                || world.component::<Asks>(person).is_none()
            {
                return Err(Rejection::NoSupportedInteraction);
            }
            (Seller::Person(person), asks_of(world, person).price(item))
        }
        None => {
            let (place, shop) = shop_here(world, buyer).ok_or(Rejection::NoSupportedInteraction)?;
            let operator = shop.operator().entity_id();
            (
                Seller::Shop { place, operator },
                asking_price(world, &shop, item),
            )
        }
    };
    let price = asking.ok_or_else(|| code::refusal(NOT_FOR_SALE))?;
    Ok((seller, price))
}

/// Step 5: the buyer is not the seller, nor a keeper of the shop that sells.
pub(crate) fn judge_self_trade(world: &WorldRead<'_>, sale: &Sale) -> Result<(), Rejection> {
    let keeper = sale.seller.shop_place().is_some_and(|place| {
        world
            .component::<Shop>(place.entity_id())
            .is_some_and(|shop| shop.kept_by(sale.buyer))
    });
    if sale.seller.entity() == sale.buyer || keeper {
        return Err(Rejection::PreconditionFailed);
    }
    Ok(())
}

/// Steps 6–9 at the sale's price: the buyer's balance; the seller's stock; the buyer's capacity; and
/// the two owners' own rules, asked last so the codes cannot disagree with them (`ARC-26`).
pub(crate) fn judge_goods(world: &WorldRead<'_>, sale: &Sale) -> Result<(), Rejection> {
    let (from, buyer) = (sale.seller.entity(), sale.buyer);
    if balance(world, buyer) < sale.price {
        return Err(code::refusal(INSUFFICIENT_FUNDS));
    }
    if world
        .component::<Holdings>(from)
        .is_none_or(|held| held.count(sale.item) == 0)
    {
        return Err(code::refusal(OUT_OF_STOCK));
    }
    if !mineworld_inventory::can_take(world, buyer, 1) {
        return Err(code::refusal(CANNOT_CARRY));
    }
    admit_payment(world, buyer, from, sale.price)?;
    mineworld_inventory::admit_transfer(world, from, buyer, sale.item, 1)
}

/// Steps 5–9 (§5.6): everything a sale at `sale.price` needs, whatever price was agreed and however.
/// `buy` and an accepted proposal both run it.
pub(crate) fn judge_exchange(world: &WorldRead<'_>, sale: &Sale) -> Result<(), Rejection> {
    judge_self_trade(world, sale)?;
    judge_goods(world, sale)
}

/// The one judgement of a buy (§5.7): [`resolve_seller`]; the submitted price equal to the asking
/// price (when one is submitted); then [`judge_exchange`].
pub(crate) fn judge_sale(
    world: &WorldRead<'_>,
    buyer: EntityId,
    target: Option<EntityId>,
    item: ItemId,
    submitted: Option<u64>,
) -> Result<Sale, Rejection> {
    let (seller, price) = resolve_seller(world, buyer, target, item)?;
    if submitted.is_some_and(|submitted| submitted != price) {
        return Err(code::refusal(PRICE_CHANGED));
    }
    let sale = Sale {
        seller,
        buyer,
        item,
        price,
    };
    judge_exchange(world, &sale)?;
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

/// A buy's three facts: re-judged from the state at resolve, then [`settle`]. A refusal here is the
/// named owner's `FactRefusedByOwner`.
pub(crate) fn settle_buy(
    world: &WorldRead<'_>,
    buyer: EntityId,
    target: Option<EntityId>,
    buy: &Buy,
) -> Result<Vec<Emission>, KernelError> {
    let sale = judge_sale(world, buyer, target, buy.item(), Some(buy.price()))
        .map_err(|reason| refused(Sold::EVENT_TYPE, reason))?;
    settle(world, &sale)
}

/// A judged sale's three facts, in this order: `sold`, economy's `money-transferred` and inventory's
/// `items-transferred`, each built by its owner's checked constructor, all heard where the sale
/// happens — the shop, or the place the buyer stands in with a person seller.
pub(crate) fn settle(world: &WorldRead<'_>, sale: &Sale) -> Result<Vec<Emission>, KernelError> {
    let buyer = sale.buyer;
    let place = match sale.seller {
        Seller::Shop { place, .. } => place,
        Seller::Person(_) => located(world, buyer)
            .map(|location| location.place())
            .ok_or_else(|| refused(Sold::EVENT_TYPE, Rejection::PreconditionFailed))?,
    };
    let heard = Visibility::Place(place);
    let seller = sale.seller.entity();
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
