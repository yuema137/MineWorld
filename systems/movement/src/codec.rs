//! This pack's payload encoding — its own, as every pack's is.
//!
//! "A payload's format is a contract between the system that declares the event type and whoever
//! reads it back" (`kernel/src/system.rs`), so this is a per-pack decision and not shared
//! infrastructure: `systems/presence/src/codec.rs` explains the choice of JSON (`DEP-5`). The one
//! payload this pack states in another pack's vocabulary — presence's `arrived` — is not encoded
//! here at all: it is built by presence's own `arrival`, so there is no second codec for it.

use mineworld_contracts::{Action, ActionRecord, ContractError, Event, EventRecord};
use serde::Serialize;

/// Encodes a payload this pack declared. Infallible for the reason presence's is: integers, typed
/// ids and options of them.
pub(crate) fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("this pack's payloads are JSON-representable by construction")
}

/// Encodes this pack's state as the already-encoded value an observation carries (the presence
/// perception seam takes `Value`, `systems/presence/src/observe.rs`). Infallible for `encode`'s reason.
pub(crate) fn to_value<T: Serialize>(value: &T) -> serde_json::Value {
    serde_json::to_value(value).expect("this pack's state is JSON-representable by construction")
}

/// Reads an action payload back as the action type this pack declared.
pub(crate) fn action_payload<A: Action>(record: &ActionRecord) -> Result<A, ContractError> {
    let payload = record.payload_for::<A>()?;
    serde_json::from_slice(payload).map_err(|_| ContractError::ActionTypeMismatch {
        expected: A::ACTION_TYPE,
        actual: record.action_type().clone(),
    })
}

/// Reads an event payload back as the event type this pack declared, on the same terms.
pub(crate) fn event_payload<E: Event>(record: &EventRecord) -> Result<E, ContractError> {
    let payload = record.payload_for::<E>()?;
    serde_json::from_slice(payload).map_err(|_| ContractError::EventTypeMismatch {
        expected: E::EVENT_TYPE,
        actual: record.event_type().clone(),
    })
}
