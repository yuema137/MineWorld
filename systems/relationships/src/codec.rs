//! This pack's payload encoding: JSON, its own (see conversation's `codec.rs`).
//!
//! [`event_payload`] is also how this pack reads *other* packs' facts: generic over the owner's
//! published type, through `payload_for`, so the type check and the schema-version refusals apply.

use mineworld_contracts::{ContractError, Event, EventRecord};
use serde::Serialize;

/// Encodes a payload this pack declared; infallible for structs of integers and typed ids.
pub(crate) fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("this pack's payloads are JSON-representable by construction")
}

/// Encodes a disclosed component as the self-describing value an observation carries.
pub(crate) fn to_value<T: Serialize>(value: &T) -> serde_json::Value {
    serde_json::to_value(value)
        .expect("this pack's payloads are JSON-representable by construction")
}

/// Reads an event payload as `E` — this pack's or the owner's published type.
pub(crate) fn event_payload<E: Event>(record: &EventRecord) -> Result<E, ContractError> {
    let payload = record.payload_for::<E>()?;
    serde_json::from_slice(payload).map_err(|_| ContractError::EventTypeMismatch {
        expected: E::EVENT_TYPE,
        actual: record.event_type().clone(),
    })
}
