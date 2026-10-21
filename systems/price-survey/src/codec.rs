//! This pack's payload encoding: JSON, its own (see conversation's `codec.rs`).

use mineworld_contracts::{Action, ActionIntent, ContractError, Event, EventRecord, Rejection};
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Encodes a payload this pack declared; infallible for its structs of ids, strings and integers.
pub(crate) fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("this pack's payloads are JSON-representable by construction")
}

/// Encodes a disclosed record as the self-describing value an observation carries.
pub(crate) fn to_value<T: Serialize>(value: &T) -> serde_json::Value {
    serde_json::to_value(value)
        .expect("this pack's payloads are JSON-representable by construction")
}

/// Reads an event payload as `E` — this pack's own, or economy's `sold`, whose published type it
/// decodes through (`ARC-28`) — through `payload_for`.
pub(crate) fn event_payload<E: Event + DeserializeOwned>(
    record: &EventRecord,
) -> Result<E, ContractError> {
    let payload = record.payload_for::<E>()?;
    serde_json::from_slice(payload).map_err(|_| ContractError::EventTypeMismatch {
        expected: E::EVENT_TYPE,
        actual: record.event_type().clone(),
    })
}

/// Reads a request's payload as `A`; a payload that is not one is a precondition the request failed.
pub(crate) fn read_action<A: Action + DeserializeOwned>(
    intent: &ActionIntent,
) -> Result<A, Rejection> {
    let payload = intent
        .payload()
        .payload_for::<A>()
        .map_err(|_| Rejection::PreconditionFailed)?;
    serde_json::from_slice(payload).map_err(|_| Rejection::PreconditionFailed)
}
