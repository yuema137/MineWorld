//! A seller's actions — list-for-sale, set-price, withdraw — judged once for both the offer and the
//! dispatch, and the facts they state (`ARC-112`; `pr-s31a-currency-asks.md` §5.6).

use mineworld_contracts::{EntityId, ItemId, PlaceId, Rejection, Visibility};
use mineworld_inventory::Holdings;
use mineworld_kernel::{Emission, WorldRead};
use mineworld_presence::{Offer, Presence};

use crate::action::{Change, ListForSale, SellerRequest, SetPrice, Withdraw, seller_requirement};
use crate::code::{
    self, ALREADY_LISTED, NOT_FOR_SALE, NOT_HELD, PRICE_OUT_OF_RANGE, PRICE_UNCHANGED,
};
use crate::codec;
use crate::event::{Listed, PriceSet, Withdrawn};
use crate::money::price_in_range;
use crate::offer::{asks_of, living_person, shop_here};

/// A seller action the rules admit: whose ask, at which shop's place (none for one's own), and the
/// change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Judged {
    seller: EntityId,
    shop: Option<PlaceId>,
    item: ItemId,
    change: Change,
    from: Option<u64>,
}

impl Judged {
    /// The shop's place, when the actor acts for a shop.
    pub(crate) const fn shop(&self) -> Option<PlaceId> {
        self.shop
    }
}

/// Whose ask `actor` would change, and at which shop, for `for_shop`: the operator of the shop the
/// actor keeps and stands in, or the actor itself.
fn seller_of(
    world: &WorldRead<'_>,
    actor: EntityId,
    for_shop: bool,
) -> Result<(EntityId, Option<PlaceId>), Rejection> {
    if !living_person(world, actor) {
        return Err(Rejection::NoSupportedInteraction);
    }
    if !for_shop {
        return Ok((actor, None));
    }
    let (place, shop) = shop_here(world, actor).ok_or(Rejection::NoSupportedInteraction)?;
    if !shop.kept_by(actor) {
        return Err(Rejection::PermissionDenied);
    }
    Ok((shop.operator().entity_id(), Some(place)))
}

/// Whether `seller` holds at least one `item`.
fn holds(world: &WorldRead<'_>, seller: EntityId, item: ItemId) -> bool {
    world
        .component::<Holdings>(seller)
        .is_some_and(|held| held.count(item) > 0)
}

/// The one judgement of a seller action, in the order §5.6 states: the actor a living Person; for a
/// shop, a shop here that the actor keeps; the price in range; the ask's existence and the seller's
/// holding. The requirement is evaluated after, by the caller (SD-S31a-2).
pub(crate) fn judge_listing(
    world: &WorldRead<'_>,
    actor: EntityId,
    request: &SellerRequest,
) -> Result<Judged, Rejection> {
    let (seller, shop) = seller_of(world, actor, request.for_shop)?;
    if let Change::List(price) | Change::Reprice(price) = request.change
        && !price_in_range(price)
    {
        return Err(code::refusal(PRICE_OUT_OF_RANGE));
    }
    let asked = asks_of(world, seller).price(request.item);
    match (request.change, asked) {
        (Change::List(_), Some(_)) => return Err(code::refusal(ALREADY_LISTED)),
        (Change::List(_), None) if !holds(world, seller, request.item) => {
            return Err(code::refusal(NOT_HELD));
        }
        (Change::Reprice(_) | Change::Withdraw, None) => return Err(code::refusal(NOT_FOR_SALE)),
        (Change::Reprice(price), Some(current)) if price == current => {
            return Err(code::refusal(PRICE_UNCHANGED));
        }
        _ => {}
    }
    Ok(Judged {
        seller,
        shop,
        item: request.item,
        change: request.change,
        from: asked,
    })
}

/// Where a seller action is heard: the shop's place for a shop, the actor's place for its own ask, or
/// the actor alone when it stands nowhere.
fn audience(
    world: &WorldRead<'_>,
    actor: EntityId,
    judged: &Judged,
) -> (Visibility, Option<PlaceId>) {
    let place = judged.shop.or_else(|| {
        world
            .component::<Presence>(actor)
            .map(|presence| presence.location().place())
    });
    match place {
        Some(place) => (Visibility::Place(place), Some(place)),
        None => (Visibility::Entities([actor].into_iter().collect()), None),
    }
}

/// The fact an admitted seller action states, about the seller and the actor.
pub(crate) fn fact(world: &WorldRead<'_>, actor: EntityId, judged: &Judged) -> Emission {
    let (heard, place) = audience(world, actor, judged);
    let (seller, item) = (judged.seller, judged.item);
    let emission = match judged.change {
        Change::List(price) => {
            Emission::new::<Listed>(codec::encode(&Listed::new(seller, item, price)), heard)
        }
        Change::Reprice(to) => Emission::new::<PriceSet>(
            codec::encode(&PriceSet::new(
                seller,
                item,
                judged.from.expect("a re-priced ask exists"),
                to,
            )),
            heard,
        ),
        Change::Withdraw => {
            Emission::new::<Withdrawn>(codec::encode(&Withdrawn::new(seller, item)), heard)
        }
    };
    let mut about = vec![seller];
    if actor != seller {
        about.push(actor);
    }
    let emission = emission.about(about);
    match place {
        Some(place) => emission.at_place(place),
        None => emission,
    }
}

/// The seller actions offered to `observer` (SD-S31a-6, Q-S31a-3): each **incomplete**, at most once
/// per action type — an incomplete offer carries no payload to tell "for me" from "for the shop". One
/// `list-for-sale` when the observer, or the shop it keeps and stands in, holds a kind with no ask; one
/// `set-price` and one `withdraw` when either has an ask. Never with a target.
pub(crate) fn seller_offers(
    world: &WorldRead<'_>,
    observer: EntityId,
    target: Option<EntityId>,
) -> Vec<Offer> {
    if target.is_some() || !living_person(world, observer) {
        return Vec::new();
    }
    let mut sellers = vec![(observer, None)];
    if let Some((place, shop)) = shop_here(world, observer)
        && shop.kept_by(observer)
    {
        sellers.push((shop.operator().entity_id(), Some(place)));
    }
    let unlisted = |seller: EntityId| {
        let asks = asks_of(world, seller);
        world.component::<Holdings>(seller).is_some_and(|held| {
            held.held()
                .iter()
                .any(|line| line.count() > 0 && asks.price(line.item()).is_none())
        })
    };
    let asking = |seller: EntityId| !asks_of(world, seller).asks().is_empty();
    let first = |pred: &dyn Fn(EntityId) -> bool| {
        sellers
            .iter()
            .find(|(seller, _)| pred(*seller))
            .map(|(_, shop)| seller_requirement(*shop))
    };
    let mut offers = Vec::new();
    if let Some(requirement) = first(&unlisted) {
        offers.push(Offer::new::<ListForSale>(requirement));
    }
    if let Some(requirement) = first(&asking) {
        offers.push(Offer::new::<SetPrice>(requirement));
        offers.push(Offer::new::<Withdraw>(requirement));
    }
    offers
}
