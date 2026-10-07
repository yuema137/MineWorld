//! This pack's payload encoding: JSON, its own, as every pack's is (see conversation's `codec.rs` for
//! why a codec is never shared infrastructure).

use mineworld_contracts::{Action, ActionRecord, ContractError, Event, EventRecord};
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Encodes a payload this pack declared. Infallible in practice: structs of integers, slugs and typed
/// ids, and no float anywhere in this workspace.
pub(crate) fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("this pack's payloads are JSON-representable by construction")
}

/// Encodes a disclosed component as the self-describing value an observation carries.
pub(crate) fn to_value<T: Serialize>(value: &T) -> serde_json::Value {
    serde_json::to_value(value)
        .expect("this pack's payloads are JSON-representable by construction")
}

/// Reads an action payload back as the action type this pack declared.
pub(crate) fn action_payload<A: Action>(record: &ActionRecord) -> Result<A, ContractError> {
    let payload = record.payload_for::<A>()?;
    serde_json::from_slice(payload).map_err(|_| ContractError::ActionTypeMismatch {
        expected: A::ACTION_TYPE,
        actual: record.action_type().clone(),
    })
}

/// Reads an event payload back as its declared type, through `payload_for`, so a record of another
/// type or schema version is refused rather than misread.
pub(crate) fn event_payload<E: Event>(record: &EventRecord) -> Result<E, ContractError> {
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
