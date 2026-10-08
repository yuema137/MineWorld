//! What a submitted request carries across the wire: its payload, the conversion onto the kernel's
//! payload type, and the client's own correlation token.

use std::fmt;

use mineworld_contracts::ActionRequest;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use super::ProtocolError;

/// The longest correlation token a client may choose.
pub const MAX_TOKEN_LENGTH: usize = 64;

/// A payload as it arrives from a client: the bytes of the JSON the client wrote.
///
/// Two encodings, deliberately, because the two ends want different things. Reading, it accepts
/// any JSON value and keeps the canonical bytes of that value; writing, it *is* those bytes and
/// serializes exactly as `Vec<u8>` does. That pair is what lets
/// [`into_kernel_request`] re-parameterize a submitted request onto the payload type
/// `World::dispatch` takes, using the contract's own `serde` in both directions rather than a
/// hand-written mirror of its shapes.
///
/// The bytes are JSON text, which is already this repository's convention for an opaque payload:
/// `kernel/tests/dispatch.rs` encodes every emission payload with `serde_json::to_vec`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WirePayload(Vec<u8>);

impl WirePayload {
    /// The canonical JSON bytes of a value.
    pub fn from_value(value: &Value) -> Result<Self, serde_json::Error> {
        Ok(Self(serde_json::to_vec(value)?))
    }

    /// The payload as the kernel will see it: opaque bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl Serialize for WirePayload {
    /// Exactly as `Vec<u8>` serializes, which is what the kernel's payload type is.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for WirePayload {
    /// Any JSON value, kept as the bytes of its canonical text.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        Self::from_value(&value).map_err(D::Error::custom)
    }
}

/// Re-parameterizes a submitted request onto the payload type the kernel dispatches.
///
/// `ActionRecord` cannot be built from an `ActionTypeId` and a payload without naming the action's
/// Rust type — deliberately, so that a record's label cannot lie — and a transport holds the label
/// without knowing the type. So the conversion goes through the contract's own serialization and
/// deserialization, which also re-applies every check the contract makes, including the
/// envelope/payload agreement check that a hand-built client frame can fail (`FINDINGS.md` F8.1).
///
/// One JSON round trip per submitted intent. Intents are rare frames; observations, the frequent
/// ones, are serialized once and never converted.
pub fn into_kernel_request(
    request: &ActionRequest<WirePayload>,
) -> Result<ActionRequest, serde_json::Error> {
    serde_json::from_value(serde_json::to_value(request)?)
}

/// A client's own token for pairing an answer with the request that caused it.
///
/// Opaque to the server: it is echoed and never interpreted, because correlation is the
/// submitter's concern (`contracts/src/action.rs`). Bounded in length and free of control
/// characters, so that a client cannot make the server hold an unbounded string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CorrelationToken(String);

impl CorrelationToken {
    /// Validates a token: 1 to [`MAX_TOKEN_LENGTH`] bytes, and no control characters.
    pub fn new(value: impl Into<String>) -> Result<Self, ProtocolError> {
        let value = value.into();
        if value.is_empty() || value.len() > MAX_TOKEN_LENGTH {
            return Err(ProtocolError::TokenLength {
                length: value.len(),
                limit: MAX_TOKEN_LENGTH,
            });
        }
        if value.chars().any(char::is_control) {
            return Err(ProtocolError::TokenNotPrintable);
        }
        Ok(Self(value))
    }

    /// The token as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for CorrelationToken {
    type Error = ProtocolError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<CorrelationToken> for String {
    fn from(value: CorrelationToken) -> Self {
        value.0
    }
}

impl fmt::Display for CorrelationToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
