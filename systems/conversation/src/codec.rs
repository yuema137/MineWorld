//! This pack's payload encoding, and why every pack owns its own.
//!
//! The kernel never interprets a payload and never chooses its encoding: "a payload's format is a
//! contract between the system that declares the event type and whoever reads it back"
//! (`kernel/src/system.rs`). So the codec is not shared infrastructure to be lifted into a common
//! crate — it is a per-pack decision, and this file is fifteen lines of it. A later pack that
//! wants CBOR for a large payload changes nothing but its own copy.
//!
//! JSON here, because `DEP-5` already chose `serde_json` for the wire and because a payload a
//! human can read in a log is worth more than a few bytes in a world of this size.

use mineworld_contracts::{Action, ActionRecord, ContractError, Event, EventRecord};
use serde::Serialize;

/// Encodes a payload this pack declared.
///
/// Infallible in practice, and the `expect` is the honest way to say so: `serde_json` fails to
/// serialize only what cannot be represented as JSON — a map with non-string keys, a non-finite
/// float — and this pack's payloads are structs of integers, strings and typed ids. There is no
/// float anywhere in this workspace. The alternative would be a [`KernelError`] variant for "my
/// own encoder failed", which the kernel does not have and which a System Pack cannot add.
///
/// [`KernelError`]: mineworld_kernel::KernelError
pub(crate) fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("this pack's payloads are JSON-representable by construction")
}

/// Reads an action payload back as the action type this pack declared.
///
/// Two failures, one error type, because a System Pack has only the contract layer's vocabulary to
/// report in: a record written for another action type is
/// [`ContractError::ActionTypeMismatch`] from [`ActionRecord::payload_for`] itself, and a record
/// whose bytes are not a readable `A` is reported as the same refusal — literally true (*this
/// payload cannot be read as `A`*), and the one thing a caller can act on.
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
