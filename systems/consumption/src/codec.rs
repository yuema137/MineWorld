//! This pack's payload encoding: JSON, its own (see conversation's `codec.rs`).

use mineworld_contracts::{Action, ActionIntent, ItemId, Rejection};

use crate::action::{DRUNK, Drink, EATEN, Eat};

/// Which kind a request names, and the category its action requires: an `eat` wants [`EATEN`], a
/// `drink` [`DRUNK`]. A payload that is neither is a precondition the request failed.
pub(crate) fn read_meal(intent: &ActionIntent) -> Result<(ItemId, &'static str), Rejection> {
    let record = intent.payload();
    let malformed = |_| Rejection::PreconditionFailed;
    if *record.action_type() == Eat::ACTION_TYPE {
        let eat: Eat = serde_json::from_slice(
            record
                .payload_for::<Eat>()
                .map_err(|_| Rejection::PreconditionFailed)?,
        )
        .map_err(malformed)?;
        return Ok((eat.item(), EATEN));
    }
    if *record.action_type() == Drink::ACTION_TYPE {
        let drink: Drink = serde_json::from_slice(
            record
                .payload_for::<Drink>()
                .map_err(|_| Rejection::PreconditionFailed)?,
        )
        .map_err(malformed)?;
        return Ok((drink.item(), DRUNK));
    }
    Err(Rejection::PreconditionFailed)
}
