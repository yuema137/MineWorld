//! This pack's payload encoding: JSON, its own (see conversation's `codec.rs`).

use mineworld_contracts::{ContractError, Event, EventRecord};
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Encodes a payload this pack declared; infallible for its structs of ids, times and integers.
pub(crate) fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("this pack's payloads are JSON-representable by construction")
}

/// Reads an event payload as `E` — this pack's own, or inventory's or sleep's, whose published types
/// it decodes through (`ARC-28`).
pub(crate) fn event_payload<E: Event + DeserializeOwned>(
    record: &EventRecord,
) -> Result<E, ContractError> {
    let payload = record.payload_for::<E>()?;
    serde_json::from_slice(payload).map_err(|_| ContractError::EventTypeMismatch {
        expected: E::EVENT_TYPE,
        actual: record.event_type().clone(),
    })
}
