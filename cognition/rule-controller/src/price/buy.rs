//! The buyer hook (`pr-s31d-reporter-policies.md` §5.5): once a model has chosen to buy a kind, `cheapest`
//! buys it from the cheapest available complete offer instead. No draw.

use mineworld_contracts::{ActionRequest, Affordance, Observation};
use serde_json::Value;

use super::block::{Buy, BuyChoice};
use super::wire::{BUY, BuySeen};
use super::{encoded, meets};

/// The payload of a `buy` affordance, decoded.
fn bought(affordance: &Affordance<Value>) -> Option<BuySeen> {
    if *affordance.action_type() != BUY {
        return None;
    }
    serde_json::from_value(affordance.payload()?.clone()).ok()
}

/// The request for `chosen`, or the cheapest available complete `buy` of the same kind; see the module.
pub(super) fn chosen(
    observation: &Observation<Value>,
    buy: &Buy,
    chosen: &Affordance<Value>,
) -> Option<ActionRequest> {
    let me = observation.observer();
    let unchanged = || chosen.request(me, encoded);
    if buy.choose == BuyChoice::Any || !meets(observation, &buy.tags) {
        return unchanged();
    }
    let Some(wanted) = bought(chosen) else {
        return unchanged();
    };
    let mut best: Option<(&Affordance<Value>, u64)> = None;
    for affordance in observation.affordances() {
        if !affordance.is_available() {
            continue;
        }
        let Some(offer) = bought(affordance) else {
            continue;
        };
        if offer.item != wanted.item {
            continue;
        }
        if best.is_none_or(|(_, price)| offer.price < price) {
            best = Some((affordance, offer.price));
        }
    }
    match best {
        Some((cheapest, _)) => cheapest.request(me, encoded),
        None => unchanged(),
    }
}
