//! This pack's payload encoding: JSON, its own (see conversation's `codec.rs`).

use mineworld_contracts::{ActionIntent, Rejection};

use crate::action::Give;

/// Reads a request's `give` payload; a payload that is not one is a precondition the request failed.
pub(crate) fn read_give(intent: &ActionIntent) -> Result<Give, Rejection> {
    let payload = intent
        .payload()
        .payload_for::<Give>()
        .map_err(|_| Rejection::PreconditionFailed)?;
    serde_json::from_slice(payload).map_err(|_| Rejection::PreconditionFailed)
}
