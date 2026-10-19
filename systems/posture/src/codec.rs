//! This pack's payload encoding: JSON, its own, as every pack's is (see appearance's `codec.rs`).

use mineworld_contracts::{Action, ActionRecord, ContractError, Event, EventRecord};
use serde::Serialize;

/// Encodes a payload this pack declared. Infallible in practice: structs of typed ids, slugs, integers
/// and validated values, and no float anywhere in this workspace.
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

/// Reads one of this pack's event payloads as `E`, through `payload_for`, so a record of another type
/// or schema version is refused rather than misread.
pub(crate) fn event_payload<E: Event>(record: &EventRecord) -> Result<E, ContractError> {
    let payload = record.payload_for::<E>()?;
    serde_json::from_slice(payload).map_err(|_| ContractError::EventTypeMismatch {
        expected: E::EVENT_TYPE,
        actual: record.event_type().clone(),
    })
}
