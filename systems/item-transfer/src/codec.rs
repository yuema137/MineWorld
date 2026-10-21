//! This pack's payload encoding: JSON, its own (see conversation's `codec.rs`).

use mineworld_contracts::{Action, ActionIntent, Rejection};
use serde::de::DeserializeOwned;

use crate::action::Give;

/// Reads a request's payload as the action `A`; a payload that is not one is a precondition the request
/// failed.
pub(crate) fn read<A: Action + DeserializeOwned>(intent: &ActionIntent) -> Result<A, Rejection> {
    let payload = intent
        .payload()
        .payload_for::<A>()
        .map_err(|_| Rejection::PreconditionFailed)?;
    serde_json::from_slice(payload).map_err(|_| Rejection::PreconditionFailed)
}

/// Reads a request's `give` payload.
pub(crate) fn read_give(intent: &ActionIntent) -> Result<Give, Rejection> {
    read::<Give>(intent)
}
