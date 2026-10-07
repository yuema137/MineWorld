//! This pack's payload encoding: JSON, its own (see conversation's `codec.rs`).

use mineworld_contracts::{ActionIntent, ContractError, Event, EventRecord, Rejection};

use crate::action::Buy;

/// Encodes a payload this pack declared; infallible for its structs of ids and integers.
pub(crate) fn encode<T: serde::Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("this pack's payloads are JSON-representable by construction")
}

/// Encodes a disclosed component as the self-describing value an observation carries.
pub(crate) fn to_value<T: serde::Serialize>(value: &T) -> serde_json::Value {
    serde_json::to_value(value)
        .expect("this pack's payloads are JSON-representable by construction")
}

/// Reads an event payload as `E` — this pack's own, or employment's `wage-due`, whose published type
/// it decodes through (`ARC-28`) — through `payload_for`.
pub(crate) fn event_payload<E: Event>(record: &EventRecord) -> Result<E, ContractError> {
    let payload = record.payload_for::<E>()?;
    serde_json::from_slice(payload).map_err(|_| ContractError::EventTypeMismatch {
        expected: E::EVENT_TYPE,
        actual: record.event_type().clone(),
    })
}

/// Reads a request's `buy` payload; a payload that is not one is a precondition the request failed.
pub(crate) fn read_buy(intent: &ActionIntent) -> Result<Buy, Rejection> {
    let payload = intent
        .payload()
        .payload_for::<Buy>()
        .map_err(|_| Rejection::PreconditionFailed)?;
    serde_json::from_slice(payload).map_err(|_| Rejection::PreconditionFailed)
}
