//! `rule`'s fetch band: a hungry person with nothing nourishing in hand goes for food, from what the
//! world discloses (`ARC-286`, Q-S35-OP1 (b); amended by `pr-s35-nh-hands-and-errands.md` R1, R4, R5).
//!
//! The world discloses facts; this band reads them and names no place, item or person:
//!
//! ```text
//! `needs`        how much every food kind of the world would nourish me (`kinds`, R3)
//! `holdings`     what I hold, how many of each, and `capacity` (R2)
//! `shop-stock`   every shop's place and the kinds it has in stock (economy, R5)
//! affordances    `pick-up` of what lies here, `buy` of what is sold here, each with its refusal code
//! ```
//!
//! The band, in order:
//!
//! ```text
//! 1  food lying here: an available pick-up of a kind that nourishes me     → pick it up (most nourishing)
//! 2  an available buy of a kind that nourishes me                          → buy it (most nourishing;
//!                                                                            ties: lowest ItemId)
//! 3  a pick-up or buy of a nourishing kind refused `cannot-carry`          → free one hand, by the
//!                                                                            policy's `free_hand` order
//! —  waiting at a punctual next part's place (`commute::holds`)            → nothing more (D-Nc2-5)
//! 4  a buy of a nourishing kind refused `insufficient-funds`               → nothing more (F-Nh-3)
//! 5  here sells, and no shop-stock is disclosed                            → nothing more (as before)
//! 6  shop-stock disclosed and no shop anywhere stocks food for me          → nothing: stay on my day
//! 7  a doorway to a shop-tagged place that stocks food for me (or that
//!    shop-stock does not list)                                             → walk there (lowest PlaceId)
//! 8  shop-tagged doorways, none stocking food for me                       → nothing (F-Ng-3)
//! 9  a place with one doorway                                              → walk out through it
//! 10 (7 or 9) the walk would make me miss setting out for a punctual next
//!    part (`commute::errand_fits`, F-Nh-7)                                 → nothing: stay on my day
//! ```
//!
//! "Nourishes me" reads `needs`' `kinds`; a world whose `needs` record does not carry it decides as
//! before this amendment (any kind I hold none of). "Free one hand" tries the policy's
//! [`FreeHand`] steps in order, each from what the world offers available now:
//!
//! ```text
//! drink / eat   the held kind that nourishes least (ties: lowest ItemId) — a cup of tea makes room
//! put-down      the held kind I hold most of, then lowest ItemId
//! give          that kind, to the person present the server lets me give it to, lowest id
//! ```
//!
//! No draw, no state: a pure function of the observation and the policy (`ARC-27`).

use mineworld_contracts::{ActionRequest, Affordance, ItemId, Observation, PlaceId, Rejection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::act::head_for;
use crate::needs::{Felt, NeedsPolicy, own};
use crate::read::{destination_tags, doorways};

/// One way to free a hand, as a world writes it in `free_hand`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum FreeHand {
    /// Drink a held drink that does not feed me.
    Drink,
    /// Eat a held kind that feeds me least.
    Eat,
    /// Put a held thing down where I stand.
    PutDown,
    /// Give a held thing to somebody present.
    Give,
}

impl FreeHand {
    /// The action type this step submits, by its slug (`ARC-35` check 2: no market crate is linked).
    const fn action(self) -> &'static str {
        match self {
            Self::Drink => "drink",
            Self::Eat => "eat",
            Self::PutDown => PUT_DOWN,
            Self::Give => "give",
        }
    }
}

/// The default `free_hand` order (Q-Nh-2): drink, eat, put down, give.
pub(crate) fn default_free_hand() -> Vec<FreeHand> {
    vec![
        FreeHand::Drink,
        FreeHand::Eat,
        FreeHand::PutDown,
        FreeHand::Give,
    ]
}

/// The most `free_hand` steps a policy lists.
pub(crate) const MAX_FREE_HAND: usize = 4;

/// The action types the hands of `item-transfer` are offered as, by their slugs. The rule decides them
/// by name in this band only, so the generic offered band leaves them out (`ARC-151`): nobody puts a
/// thing down or picks one up by chance.
pub const PUT_DOWN: &str = "put-down";
/// See [`PUT_DOWN`].
pub const PICK_UP: &str = "pick-up";

/// The action type economy offers for a priced kind, by its slug.
const BUY: &str = "buy";

/// The refusal codes this band reads, by their slugs.
const CANNOT_CARRY: &str = "cannot-carry";
const INSUFFICIENT_FUNDS: &str = "insufficient-funds";

/// The component types read by slug.
const HOLDINGS: &str = "holdings";
const SHOP_STOCK: &str = "shop-stock";

/// The part of inventory's disclosed `holdings` this band reads.
#[derive(Deserialize, Default)]
struct DisclosedHoldings {
    #[serde(default)]
    held: Vec<DisclosedHeld>,
}

#[derive(Deserialize)]
struct DisclosedHeld {
    item: ItemId,
    count: u32,
}

/// The part of economy's disclosed `shop-stock` this band reads.
#[derive(Deserialize)]
struct DisclosedStock {
    shops: Vec<DisclosedShop>,
}

#[derive(Deserialize)]
struct DisclosedShop {
    place: PlaceId,
    stocked: Vec<ItemId>,
}

/// A record of my own, by its component slug, decoded.
fn mine<T: serde::de::DeserializeOwned>(observation: &Observation<Value>, slug: &str) -> Option<T> {
    let me = observation.entity(observation.observer())?;
    let record = me
        .components()
        .iter()
        .find(|record| record.component_type().as_str() == slug)?;
    serde_json::from_value(record.payload().clone()).ok()
}

/// Every kind I hold and how many, in item order; none when inventory discloses nothing.
fn held(observation: &Observation<Value>) -> Vec<(ItemId, u32)> {
    mine::<DisclosedHoldings>(observation, HOLDINGS)
        .unwrap_or_default()
        .held
        .into_iter()
        .map(|line| (line.item, line.count))
        .collect()
}

/// What the band knows of food: how much a kind nourishes me, when `needs` disclosed `kinds`.
struct Food<'a> {
    felt: &'a Felt,
}

impl Food<'_> {
    /// Whether `needs` told me about kinds I do not carry.
    fn known(&self) -> bool {
        !self.felt.kinds.is_empty()
    }

    /// How much one of `item` nourishes me: from `kinds`, else from what I carry.
    fn of(&self, item: ItemId) -> u16 {
        self.felt
            .kinds
            .iter()
            .find(|entry| entry.item == item)
            .map_or_else(|| self.felt.nourishment_of(item), |entry| entry.hunger)
    }

    /// Whether `item` is worth having for hunger: it nourishes me — or, when `needs` tells me nothing of
    /// kinds I do not carry, it is a kind I hold none of (the band before R3).
    fn worth(&self, item: ItemId, held: &[(ItemId, u32)]) -> bool {
        if self.known() {
            self.of(item) > 0
        } else {
            !held.iter().any(|(kind, _)| *kind == item)
        }
    }
}

/// The item a complete affordance names in its payload.
fn item_of(affordance: &Affordance<Value>) -> Option<ItemId> {
    serde_json::from_value(affordance.payload()?.get("item")?.clone()).ok()
}

/// The refusal code of an unavailable affordance, when the refusal is a system's own.
fn code_of(affordance: &Affordance<Value>) -> Option<&str> {
    match affordance.unavailable_reason()? {
        Rejection::System { code, .. } => Some(code.as_str()),
        _ => None,
    }
}

/// Every affordance of `slug` naming a kind, with the kind.
fn offers_of<'a>(
    observation: &'a Observation<Value>,
    slug: &str,
) -> Vec<(ItemId, &'a Affordance<Value>)> {
    observation
        .affordances()
        .iter()
        .filter(|affordance| affordance.action_type().as_str() == slug)
        .filter_map(|affordance| Some((item_of(affordance)?, affordance)))
        .collect()
}

/// The available offer of the kind that nourishes most, ties to the lowest `ItemId`; without `kinds`,
/// the lowest `ItemId` worth having.
fn best<'a>(
    offers: &[(ItemId, &'a Affordance<Value>)],
    food: &Food<'_>,
    held: &[(ItemId, u32)],
) -> Option<&'a Affordance<Value>> {
    offers
        .iter()
        .filter(|(item, offer)| offer.is_available() && food.worth(*item, held))
        .max_by(|(a, _), (b, _)| food.of(*a).cmp(&food.of(*b)).then_with(|| b.cmp(a)))
        .map(|(_, offer)| *offer)
}

/// Whether one of `offers` of a kind worth having is refused with `code`.
fn refused_with(
    offers: &[(ItemId, &Affordance<Value>)],
    food: &Food<'_>,
    held: &[(ItemId, u32)],
    code: &str,
) -> bool {
    offers
        .iter()
        .any(|(item, offer)| food.worth(*item, held) && code_of(offer) == Some(code))
}

impl NeedsPolicy {
    /// The fetch band (the module's table). [`None`] when the band does not apply or decides nothing is
    /// to be done, so the consult goes on; `Some(None)` when my own walk already goes where the band
    /// would send me, so nothing redirects it. No draw.
    pub fn fetch(
        &self,
        observation: &Observation<Value>,
        waiting: bool,
    ) -> Option<Option<ActionRequest>> {
        self.fetch_within(observation, waiting, |_| true)
    }

    /// [`Self::fetch`], walking away (rows 7 and 9) only to a place for which `errand_fits` holds:
    /// the caller's test that the errand still leaves time to set out for a punctual next part
    /// (`commute::errand_fits`). A purchase or pick-up here is never held back by it.
    pub fn fetch_within(
        &self,
        observation: &Observation<Value>,
        waiting: bool,
        errand_fits: impl Fn(PlaceId) -> bool,
    ) -> Option<Option<ActionRequest>> {
        if !self.fetch_food {
            return None;
        }
        let felt = own(observation)?;
        if felt.hunger < self.fetch_from || felt.nourishment.iter().any(|food| food.hunger > 0) {
            return None;
        }
        let food = Food { felt: &felt };
        let held = held(observation);
        let me = observation.observer();
        // Only food is picked up, and only when `needs` says what feeds me: goods put down stay put.
        let pick_ups: Vec<(ItemId, &Affordance<Value>)> = offers_of(observation, PICK_UP)
            .into_iter()
            .filter(|(item, _)| food.known() && food.of(*item) > 0)
            .collect();
        let buys = offers_of(observation, BUY);
        // 1, 2: food lying here, then food sold here.
        if let Some(offer) = best(&pick_ups, &food, &held).or_else(|| best(&buys, &food, &held)) {
            return Some(offer.request(me, encoded));
        }
        // 3: hands full in front of food: free one hand, this consult.
        if (refused_with(&pick_ups, &food, &held, CANNOT_CARRY)
            || refused_with(&buys, &food, &held, CANNOT_CARRY))
            && let Some(freed) = self.free_one_hand(observation, &food, &held)
        {
            return Some(Some(freed));
        }
        if waiting {
            return None;
        }
        // 4: too poor for the food here: nothing more (F-Nh-3; no policy for being broke yet).
        if refused_with(&buys, &food, &held, INSUFFICIENT_FUNDS) {
            return None;
        }
        let here = *observation.self_location()?;
        let stock = mine::<DisclosedStock>(observation, SHOP_STOCK);
        // 5: here sells, and nothing tells me where else has food: nothing more, as before R5.
        let sells_here = !buys.is_empty()
            || observation
                .entity(here.place().entity_id())
                .is_some_and(|place| self.shop_tags.iter().any(|tag| place.tags().contains(tag)));
        if sells_here && stock.is_none() {
            return None;
        }
        let stocks_food =
            |shop: &DisclosedShop| shop.stocked.iter().any(|item| food.worth(*item, &held));
        // 6: shops are disclosed, and none anywhere has anything for me: a walk would find nothing.
        if let Some(stock) = &stock
            && !stock.shops.iter().any(stocks_food)
        {
            return None;
        }
        // 7, 8: a doorway to a shop that has food for me (or one shop-stock does not list).
        let shops: Vec<PlaceId> = destination_tags(observation, here)
            .into_iter()
            .filter(|(_, tags)| tags.iter().any(|tag| self.shop_tags.contains(tag)))
            .map(|(place, _)| place)
            .collect();
        let worth_walking = |place: &PlaceId| {
            stock.as_ref().is_none_or(|stock| {
                stock
                    .shops
                    .iter()
                    .find(|shop| shop.place == *place)
                    .is_none_or(stocks_food)
            })
        };
        let to = match shops.iter().copied().find(worth_walking) {
            Some(place) => place,
            None if !shops.is_empty() => return None,
            // 9: a dead end leads to the rest of the world.
            None => {
                let doors = doorways(observation, here)?;
                let mut doors = doors.iter();
                let only = doors.next()?;
                if doors.next().is_some() {
                    return None;
                }
                only.to()
            }
        };
        // 10: the walk away must still let me set out in time for a punctual next part (F-Nh-7).
        if !errand_fits(to) {
            return None;
        }
        Some(head_for(observation, to)?.request())
    }

    /// One request that frees a hand, by the `free_hand` order; nothing when no step is offered.
    fn free_one_hand(
        &self,
        observation: &Observation<Value>,
        food: &Food<'_>,
        held: &[(ItemId, u32)],
    ) -> Option<ActionRequest> {
        let me = observation.observer();
        // The kinds in the order a hand is freed of them: most held, then lowest ItemId.
        let mut by_count: Vec<(ItemId, u32)> = held.to_vec();
        by_count.sort_by(|(a, count_a), (b, count_b)| count_b.cmp(count_a).then_with(|| a.cmp(b)));
        for step in &self.free_hand {
            let offers: Vec<(ItemId, &Affordance<Value>)> = offers_of(observation, step.action())
                .into_iter()
                .filter(|(_, offer)| offer.is_available())
                .collect();
            let chosen = match step {
                FreeHand::Drink | FreeHand::Eat => offers
                    .iter()
                    .min_by(|(a, _), (b, _)| food.of(*a).cmp(&food.of(*b)).then_with(|| a.cmp(b)))
                    .map(|(_, offer)| *offer),
                FreeHand::PutDown | FreeHand::Give => by_count.iter().find_map(|(kind, _)| {
                    offers
                        .iter()
                        .filter(|(item, _)| item == kind)
                        .min_by_key(|(_, offer)| offer.target())
                        .map(|(_, offer)| *offer)
                }),
            };
            if let Some(request) = chosen.and_then(|offer| offer.request(me, encoded)) {
                return Some(request);
            }
        }
        None
    }
}

/// This band's payload encoding for an offered affordance: JSON bytes.
fn encoded(payload: &Value) -> Vec<u8> {
    serde_json::to_vec(payload).expect("a JSON value is JSON-representable")
}

/// The component type economy discloses a holder's money on, by its slug.
const WALLET: &str = "wallet";

/// The part of economy's disclosed `wallet` this band reads.
#[derive(Deserialize)]
struct DisclosedWallet {
    balance: u64,
}

impl NeedsPolicy {
    /// Whether `affordance` is a `buy` the chance band leaves alone (S35 N-h round 3, `ARC-360` item 8):
    /// in a world where I eat (`needs` disclosed to me), a purchase made by chance costs at most
    /// `chance_spend_permille` of my wallet, so money a person lives on goes on food first. Food is the
    /// fetch band's, by name; a world without `needs`, a buy without a price, or a wallet not disclosed
    /// is never held back.
    pub fn beyond_chance_budget(
        &self,
        observation: &Observation<Value>,
        affordance: &Affordance<Value>,
    ) -> bool {
        if affordance.action_type().as_str() != BUY || own(observation).is_none() {
            return false;
        }
        let Some(price) = affordance
            .payload()
            .and_then(|payload| payload.get("price"))
            .and_then(Value::as_u64)
        else {
            return false;
        };
        let Some(wallet) = mine::<DisclosedWallet>(observation, WALLET) else {
            return false;
        };
        u128::from(price) * 1_000
            > u128::from(wallet.balance) * u128::from(self.chance_spend_permille)
    }
}
