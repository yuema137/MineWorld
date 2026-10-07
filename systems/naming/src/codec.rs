//! This pack's payload encoding: JSON, its own (see conversation's `codec.rs`).

use mineworld_contracts::{ContractError, Event, EventRecord};
use serde::Serialize;

/// Encodes a payload this pack declared; infallible for a struct of a typed id and a string.
pub(crate) fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("this pack's payloads are JSON-representable by construction")
}

/// Encodes a disclosed component as the self-describing value an observation carries.
pub(crate) fn to_value<T: Serialize>(value: &T) -> serde_json::Value {
    serde_json::to_value(value)
        .expect("this pack's payloads are JSON-representable by construction")
}

/// Reads one of this pack's event payloads as `E`, through `payload_for`.
pub(crate) fn event_payload<E: Event>(record: &EventRecord) -> Result<E, ContractError> {
    let payload = record.payload_for::<E>()?;
    serde_json::from_slice(payload).map_err(|_| ContractError::EventTypeMismatch {
        expected: E::EVENT_TYPE,
        actual: record.event_type().clone(),
    })
}
