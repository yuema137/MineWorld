//! What a frame says about the connection itself rather than about the world: which connection it
//! is, whether control of its Person changed hands when it joined, and why the server is closing it.

use std::fmt;

use serde::{Deserialize, Serialize};

use super::ProtocolError;

/// Which connection this is: allocated once per connection, never reused within a process.
///
/// It names a connection on the admin surface (S11-D) and nowhere else. It is not a credential:
/// knowing another connection's session lets a client do nothing. On the wire it is a decimal
/// string, as every identity is (`PROTOCOL.md` §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub struct SessionId(u64);

impl SessionId {
    /// The session with this number.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// The number.
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<SessionId> for String {
    fn from(value: SessionId) -> Self {
        value.to_string()
    }
}

impl TryFrom<String> for SessionId {
    type Error = ProtocolError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value
            .parse()
            .map(Self)
            .map_err(|_| ProtocolError::SessionId(value))
    }
}

/// Whether control of the Person changed hands when this connection joined (`PROTOCOL.md` §5.1).
///
/// It never says which controller kind or which player — only that the seat was free, was being
/// driven by the server, or was held for this connection's own earlier socket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TookOver {
    /// The seat was free. The only value a server sends before S11-B.
    None,
    /// An in-server controller was driving the Person (from S11-B).
    Hosted,
    /// This join resumed a seat held after its connection dropped (from S11-B).
    Held,
}

/// Why the server is about to close a connection (`PROTOCOL.md` §5.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClosingReason {
    /// The client sent `leave`.
    Left,
    /// The operator removed this connection (from S11-D).
    Kicked,
    /// A newer connection re-took this seat with its `resume` (from S11-B).
    Superseded,
    /// The invite was wrong or missing.
    Unauthorized,
    /// The client speaks another revision of the protocol.
    ProtocolMismatch,
    /// The world stopped, so there is nothing left to observe.
    WorldStopped,
    /// The server process is shutting down.
    ServerStopping,
}
