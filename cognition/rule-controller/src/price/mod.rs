//! Pricing policies, shared by `rule` and `motivated` (`ARC-117`; `pr-s31d-reporter-policies.md` §§5.4 …
//! 5.7).
//!
//! ```text
//! block.rs   the pricing: block, decoded by hand, default hold / any
//! slots.rs   slot arithmetic on world seconds (also the price clerk's)
//! wire.rs    the market packs' records and requests, by slug (ARC-182)
//! sell.rs    the seller band: list at index 0, then one priceable ask per index
//! buy.rs     the buyer hook: cheapest instead of the chosen buy
//! ```
//!
//! Every function here reads the observation and takes **no draw**, so a model with the default block
//! decides exactly as before, consult for consult. Suggestions come only from the observer's own
//! `notice-board` record, which lists current notices only (I-S31-8): an expired suggestion is never used.

use mineworld_contracts::{
    ActionRequest, Affordance, ItemId, Observation, PersonId, SimDuration, Tag, WorldTime,
};
use mineworld_notices::{NoticeBoardRecord, NoticeBody, TopicId};
use serde_json::Value;

use crate::read;

pub mod block;
mod buy;
mod sell;
pub mod slots;
pub mod wire;

pub use block::{Buy, BuyChoice, ListRule, Pricing, Sell, SellPolicy};

/// The seller band: at most one `set-price` or `list-for-sale` this consult, or nothing.
///
/// `None` at once with the default block. Otherwise only for a seat whose own tags meet `sell.tags`;
/// index 0 of each `sell.every_minutes` period lists a held kind of a listed category, and index `j ≥ 1`
/// prices the `j − 1`th priceable ask by `sell.policy`, bounded by the kind's reference value.
// S31-h: `answer_proposals(observation, pricing)` runs first here, when it lands (§5.6).
pub fn seller_band(
    observation: &Observation<Value>,
    pace: SimDuration,
    pricing: &Pricing,
) -> Option<ActionRequest> {
    sell::band(observation, pace, &pricing.sell)
}

/// The buyer hook: the request for `chosen`, or — when `chosen` is a `buy`, `buy.choose` is `cheapest`
/// and my tags meet `buy.tags` — the request of the cheapest available complete `buy` of the same kind.
// S31-h: `haggle_instead(observation, pricing, request)` applies to the result, when it lands (§5.6).
pub fn on_chosen_buy(
    observation: &Observation<Value>,
    pricing: &Pricing,
    chosen: &Affordance<Value>,
) -> Option<ActionRequest> {
    buy::chosen(observation, &pricing.buy, chosen)
}

/// The current suggestion for one kind, as the board shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggested {
    /// The suggested price, in minor units.
    pub price: u64,
    /// How many notes the publisher said it rests on.
    pub basis: u16,
    /// When the notice was posted.
    pub issued_at: WorldTime,
    /// Who posted it.
    pub publisher: PersonId,
    /// On which topic.
    pub topic: TopicId,
}

/// The current suggestion for `item`: among the `prices` lines for it in my own `notice-board` record,
/// the line of the notice with the newest `issued_at`; ties go to the lower publisher id, then the lower
/// topic. Reads only the board, never remembered or perceived notices (X-6).
pub fn suggestion(observation: &Observation<Value>, item: ItemId) -> Option<Suggested> {
    let me = observation.entity(observation.observer())?;
    let board: NoticeBoardRecord = read::disclosed(me)?;
    let mut best: Option<Suggested> = None;
    for notice in &board.notices {
        let NoticeBody::Prices(lines) = notice.body() else {
            continue;
        };
        let Some(line) = lines.iter().find(|line| line.item() == item.entity_id()) else {
            continue;
        };
        let candidate = Suggested {
            price: line.price(),
            basis: line.basis(),
            issued_at: notice.issued_at(),
            publisher: notice.publisher(),
            topic: notice.topic().clone(),
        };
        let better = best.as_ref().is_none_or(|current| {
            let newer = candidate
                .issued_at
                .seconds()
                .cmp(&current.issued_at.seconds());
            newer
                .then_with(|| {
                    current
                        .publisher
                        .entity_id()
                        .raw()
                        .cmp(&candidate.publisher.entity_id().raw())
                })
                .then_with(|| current.topic.as_str().cmp(candidate.topic.as_str()))
                .is_gt()
        });
        if better {
            best = Some(candidate);
        }
    }
    best
}

/// Whether my own disclosed tags include one of `tags`; an empty list meets every seat.
pub(crate) fn meets(observation: &Observation<Value>, tags: &[Tag]) -> bool {
    if tags.is_empty() {
        return true;
    }
    observation
        .entity(observation.observer())
        .is_some_and(|me| tags.iter().any(|tag| me.tags().contains(tag)))
}

/// A record of my own, decoded through its mirror.
pub(crate) fn mine<C>(observation: &Observation<Value>) -> Option<C>
where
    C: mineworld_contracts::Component + serde::de::DeserializeOwned,
{
    read::disclosed(observation.entity(observation.observer())?)
}

/// A record of the place I stand in, decoded through its mirror.
pub(crate) fn here<C>(observation: &Observation<Value>) -> Option<C>
where
    C: mineworld_contracts::Component + serde::de::DeserializeOwned,
{
    let place = observation.self_location()?.place();
    read::disclosed(observation.entity(place.entity_id())?)
}

/// This crate's payload encoding for an affordance's own request: JSON bytes.
pub(crate) fn encoded(payload: &Value) -> Vec<u8> {
    serde_json::to_vec(payload).expect("a JSON value is JSON-representable")
}
