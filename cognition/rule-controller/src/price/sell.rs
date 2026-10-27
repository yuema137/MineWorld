//! The seller band (`pr-s31d-reporter-policies.md` §5.5).
//!
//! ```text
//! priceable   my own asks (my `asks` view), by item; then — when set-price is offered, I have no asks
//!             of my own and the place I stand in shows a listing — the shop's, by item (F-S31d-6)
//! index       j = floor((t mod every) / pace)                                     (slots.rs)
//! j = 0       list: the first held kind, by item, of a listed category, with no ask of mine and a
//!             reference value → list-for-sale { item, price: reference }
//! j ≥ 1       the (j − 1)th priceable ask, moved by the policy, bounded by the reference value
//! ```

use mineworld_contracts::{ActionRequest, ItemId, Observation, SimDuration};
use serde_json::Value;

use super::block::{Sell, SellPolicy};
use super::slots::period_index;
use super::wire::{
    AsksSeen, CatalogueSeen, CurrencySeen, HoldingsSeen, ListForSale, ListingSeen, SetPrice,
    ShopSideSeen,
};
use super::{here, meets, mine, suggestion};
use crate::act;
use crate::read;

/// The most a notice or an ask may say, in minor units (notices' `NOTICE_PRICE_MAX`).
const PRICE_MAX: u64 = mineworld_notices::NOTICE_PRICE_MAX;

/// One ask a seller may move: whose, which kind, its price, and the stock it reads for `scarcity`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Priceable {
    item: ItemId,
    price: u64,
    side: Option<ShopSideSeen>,
    stock: u32,
}

/// The seller band; see the module.
pub(super) fn band(
    observation: &Observation<Value>,
    pace: SimDuration,
    sell: &Sell,
) -> Option<ActionRequest> {
    if !sell.is_active() || !meets(observation, &sell.tags) {
        return None;
    }
    let index = period_index(
        observation.at().seconds(),
        pace.seconds(),
        sell.every_minutes,
    )?;
    let currency: CurrencySeen = mine(observation)?;
    if index == 0 {
        return list(observation, sell, &currency);
    }
    if sell.policy == SellPolicy::Hold {
        return None;
    }
    let ask = *priceable(observation).get(usize::try_from(index - 1).ok()?)?;
    let reference = currency.reference(ask.item)?;
    let proposed = match sell.policy {
        SellPolicy::Hold => return None,
        SellPolicy::TowardSuggestion => toward(observation, sell, ask)?,
        SellPolicy::Scarcity => scarcity(sell, ask)?,
    };
    let price = bounded(proposed, reference, sell);
    if price == ask.price {
        return None;
    }
    Some(act::request(
        observation.observer(),
        &SetPrice {
            item: ask.item,
            price,
            side: ask.side,
        },
    ))
}

/// My own asks by item, then — when I keep the shop I stand in — the shop's, by item.
fn priceable(observation: &Observation<Value>) -> Vec<Priceable> {
    let own: Vec<Priceable> = mine::<AsksSeen>(observation)
        .map(|asks| {
            let holdings = mine::<HoldingsSeen>(observation);
            asks.asks
                .iter()
                .map(|ask| Priceable {
                    item: ask.item,
                    price: ask.price,
                    side: None,
                    stock: holdings.as_ref().map_or(0, |held| held.count(ask.item)),
                })
                .collect()
        })
        .unwrap_or_default();
    if !own.is_empty() {
        return own;
    }
    // Economy offers `set-price` when I, or a shop I keep and stand in, have an ask. With no asks of my
    // own, the offer means I keep this shop (F-S31d-6).
    if !read::available::<SetPrice>(observation, None) {
        return Vec::new();
    }
    let Some(listing) = here::<ListingSeen>(observation) else {
        return Vec::new();
    };
    listing
        .listed
        .iter()
        .map(|line| Priceable {
            item: line.item,
            price: line.price,
            side: Some(ShopSideSeen::Shop),
            stock: line.in_stock,
        })
        .collect()
}

/// Index 0: list the first held kind of a listed category that I do not ask for yet, at its reference.
fn list(
    observation: &Observation<Value>,
    sell: &Sell,
    currency: &CurrencySeen,
) -> Option<ActionRequest> {
    if sell.list.categories.is_empty() || !read::available::<ListForSale>(observation, None) {
        return None;
    }
    let held: HoldingsSeen = mine(observation)?;
    let catalogue: CatalogueSeen = here(observation)?;
    let asked = mine::<AsksSeen>(observation)
        .map(|asks| asks.asks)
        .unwrap_or_default();
    held.held
        .iter()
        .filter(|line| line.count >= 1)
        .filter(|line| !asked.iter().any(|ask| ask.item == line.item))
        .filter(|line| {
            catalogue
                .category(line.item)
                .is_some_and(|category| sell.list.categories.iter().any(|c| c == category))
        })
        .find_map(|line| {
            let price = currency.reference(line.item)?.clamp(1, PRICE_MAX);
            Some(act::request(
                observation.observer(),
                &ListForSale {
                    item: line.item,
                    price,
                    side: None,
                },
            ))
        })
}

/// `toward-suggestion`: a step of `step_permille` of the gap toward the current suggestion, at least one
/// minor unit, unless the gap is within the deadband. `None`: no suggestion, or inside the deadband.
fn toward(observation: &Observation<Value>, sell: &Sell, ask: Priceable) -> Option<u64> {
    let target = i128::from(suggestion(observation, ask.item)?.price);
    let price = i128::from(ask.price);
    let gap = target - price;
    if gap.abs() * 1_000 <= i128::from(sell.deadband_permille) * price {
        return None;
    }
    let mut step = gap * i128::from(sell.step_permille) / 1_000;
    if step == 0 {
        step = gap.signum();
    }
    u64::try_from((price + step).max(1)).ok()
}

/// `scarcity`: up by `step_permille` of the ask (at least one) below `low_below` in stock, down above
/// `full_above`; otherwise nothing.
fn scarcity(sell: &Sell, ask: Priceable) -> Option<u64> {
    let step = (u128::from(ask.price) * u128::from(sell.step_permille) / 1_000).max(1);
    let step = u64::try_from(step).ok()?;
    if ask.stock < sell.low_below {
        Some(ask.price.saturating_add(step))
    } else if ask.stock > sell.full_above {
        Some(ask.price.saturating_sub(step).max(1))
    } else {
        None
    }
}

/// `price` bounded by the reference value: `[max(1, ref × floor ÷ 1 000), max(lo, ref × ceiling ÷ 1 000)]`,
/// and never above [`PRICE_MAX`].
pub(super) fn bounded(price: u64, reference: u64, sell: &Sell) -> u64 {
    let scaled = |permille: u32| {
        let value = u128::from(reference) * u128::from(permille) / 1_000;
        u64::try_from(value).unwrap_or(u64::MAX)
    };
    let low = scaled(sell.floor_permille).max(1);
    let high = scaled(sell.ceiling_permille).max(low);
    price.clamp(low, high).min(PRICE_MAX)
}
