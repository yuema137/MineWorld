//! This pack's payload encoding: JSON, its own (see conversation's `codec.rs`).

use mineworld_contracts::{Action, ActionRecord, ContractError, Event, EventRecord};
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Encodes a payload this pack declared; infallible for its structs of ids, times and integers.
pub(crate) fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("this pack's payloads are JSON-representable by construction")
}

/// Encodes a disclosed record as the self-describing value an observation carries.
pub(crate) fn to_value<T: Serialize>(value: &T) -> serde_json::Value {
    serde_json::to_value(value)
        .expect("this pack's payloads are JSON-representable by construction")
}

/// Reads an action payload as `A`.
pub(crate) fn action_payload<A: Action + DeserializeOwned>(
    record: &ActionRecord,
) -> Result<A, ContractError> {
    let payload = record.payload_for::<A>()?;
    serde_json::from_slice(payload).map_err(|_| ContractError::ActionTypeMismatch {
        expected: A::ACTION_TYPE,
        actual: record.action_type().clone(),
    })
}

/// Reads an event payload as `E` — this pack's own, or presence's or conversation's, whose published
/// types it decodes through (`ARC-28`).
pub(crate) fn event_payload<E: Event + DeserializeOwned>(
    record: &EventRecord,
) -> Result<E, ContractError> {
    let payload = record.payload_for::<E>()?;
    serde_json::from_slice(payload).map_err(|_| ContractError::EventTypeMismatch {
        expected: E::EVENT_TYPE,
        actual: record.event_type().clone(),
    })
}

/// Reads this pack's own state for a process back.
pub(crate) fn state<T: DeserializeOwned>(bytes: &[u8]) -> Option<T> {
    serde_json::from_slice(bytes).ok()
}
