//! The wire protocol: the only frames that exist, and what a client may therefore say.
//!
//! `docs/NETWORKING.md` §5 lists the message classes; this module is that list as closed Rust
//! enums, which is what makes the authority model structural rather than remembered. A client can
//! say exactly two things:
//!
//! ```text
//! join     name a seat, and be told which observer it is
//! submit   an ActionRequest, and the client's own correlation token
//! ```
//!
//! and there is no third — no frame that sets a value, names another observer, widens a scope or
//! asserts a fact. *"My money is now 5000"* is not a frame this protocol has, so it is refused as
//! a protocol violation (`NETWORKING.md` §2) rather than being checked and rejected somewhere
//! deeper.
//!
//! # What this module does not do
//!
//! **It does not encode identity.** `EntityId`, `ActionId` and `EventId` reach the wire as decimal
//! strings because `mineworld-contracts` encodes them that way whenever the format is
//! human-readable, and a JSON parser with only doubles corrupts anything above 2^53
//! (`spike/FINDINGS.md` F2, `DECISIONS.md` DEP-3). Every frame below therefore carries contract
//! types and lets the contract's own `serde` write them. A structural mirror of contract shapes at
//! the protocol boundary is what PR 04 deleted; writing one here would undo that fix.
//!
//! **It does not interpret a payload.** [`WirePayload`] carries the bytes of whatever JSON a client
//! wrote, and the only thing this module knows about them is that they are JSON.
//!
//! # Why correlation lives here and not in `ActionResult`
//!
//! `contracts/src/action.rs` records that `ActionResult` deliberately carries no `ActionId`, and
//! that the one genuinely open case — recognizing later facts caused by a request — "is a field in
//! a protocol frame, not in the kernel's answer". [`ServerFrame::Result`] is that frame: it echoes
//! the client's own [`CorrelationToken`] so an answer can be paired with its request, and it names
//! the [`ActionId`] the *server* allocated so a client can recognize a later event whose
//! `Causation` points at its own request.

//!
//! # Layout
//!
//! ```text
//! protocol           the frames, the refusal codes, and what can be wrong with a protocol value
//! protocol/request   a submitted request's payload and the client's correlation token
//! protocol/summary   what a world is: its instance identity and its composition
//! ```

mod request;
mod summary;
#[cfg(test)]
mod tests;

use std::fmt;

use mineworld_contracts::{
    ActionId, ActionRequest, ActionResult, ContractError, EntityId, EntityKey, Observation,
};
use mineworld_persistence::WorldRevision;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use request::{CorrelationToken, MAX_TOKEN_LENGTH, WirePayload, into_kernel_request};
pub use summary::{SystemSummary, WorldInstanceId, WorldSummary};

/// Which revision of this protocol a server speaks. A client that does not recognize the number
/// should refuse to connect rather than guess.
pub const PROTOCOL_VERSION: u32 = 1;

/// An `Observation` as this transport carries it.
///
/// `serde_json::Value` rather than the contract's `Vec<u8>` default, which
/// `spike/FINDINGS.md` F8.2 measured as unusable on a JSON wire: with the default a component's
/// payload reaches a client as an array of byte integers. The generic parameter exists precisely
/// so a transport can choose, and this is the choice F8.2 asked the server to state.
pub type WireObservation = Observation<Value>;

/// What a client may say. Two variants, and the closed set is the point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum ClientFrame {
    /// Ask to occupy a seat. The server answers with the observer that seat is, and a client
    /// never names an observer itself — which is why there is no request that widens perception
    /// (`INV-13`).
    Join {
        /// The seat, named by the authoring key of the entity it belongs to.
        seat: EntityKey,
    },
    /// Submit a request. No identity and no instant: the server allocates both (`INV-6`).
    Submit {
        /// The client's own token, echoed on the answer.
        token: CorrelationToken,
        /// What is being asked of the world.
        request: ActionRequest<WirePayload>,
    },
}

/// The tags a client frame may carry, which is also the whole of what a client may say.
const CLIENT_FRAME_TAGS: [&str; 2] = ["join", "submit"];

impl ClientFrame {
    /// Reads one text frame, or the refusal to send back.
    ///
    /// Two steps rather than one, so that the two failures a client cares about are distinct: a
    /// frame whose kind this protocol does not have is [`RefusalCode::UnknownFrame`] — which is
    /// where a message asserting state lands — while a frame of a known kind that does not decode
    /// is [`RefusalCode::MalformedFrame`], with the contract's own complaint as its detail. The
    /// token is recovered where possible even from a frame that failed, so that a client can still
    /// pair the refusal with what it sent.
    pub fn decode(text: &str) -> Result<Self, Refusal> {
        let value: Value = serde_json::from_str(text)
            .map_err(|error| Refusal::new(RefusalCode::MalformedFrame).detailed(error))?;
        let token = value
            .get("token")
            .and_then(Value::as_str)
            .and_then(|token| CorrelationToken::new(token).ok());
        let Some(tag) = value.get("t").and_then(Value::as_str) else {
            return Err(Refusal::new(RefusalCode::MalformedFrame)
                .about(token)
                .detail("a frame states its kind in a string field named \"t\""));
        };
        if !CLIENT_FRAME_TAGS.contains(&tag) {
            return Err(Refusal::new(RefusalCode::UnknownFrame)
                .about(token)
                .detail(format!(
                    "this protocol has no frame of kind \"{tag}\"; a client may only join or submit"
                )));
        }
        serde_json::from_value(value).map_err(|error| {
            Refusal::new(RefusalCode::MalformedFrame)
                .about(token)
                .detailed(error)
        })
    }
}

/// What the server says. Everything a client ever receives is one of these.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum ServerFrame {
    /// The seat is this client's, and this is the observer it sees the world as.
    Welcome {
        /// Which revision of this protocol the server speaks.
        protocol: u32,
        /// The seat that was granted.
        seat: EntityKey,
        /// The observer every observation on this connection belongs to.
        observer: EntityId,
        /// What the world is, at the moment of joining.
        world: WorldSummary,
    },
    /// What this connection's observer perceives, and nothing else.
    Observation {
        /// A per-connection counter, so a client can order two frames stamped with the same
        /// `WorldTime` — which a one-second clock and a 10 Hz stream otherwise cannot be
        /// (`FINDINGS.md` F7).
        seq: u64,
        /// The persisted revision of the state this observation was computed from, or `None` for a
        /// world that is not persisted (`PROTOCOL.md` §5). Committed before this frame was sent.
        revision: Option<WorldRevision>,
        /// The observation itself.
        observation: WireObservation,
    },
    /// The world's answer to one submitted request.
    Result {
        /// The client's own token, as it sent it.
        token: CorrelationToken,
        /// The identity the server allocated for the request.
        action_id: ActionId,
        /// What the world answered.
        result: ActionResult,
    },
    /// The frame was not something this protocol accepts, and nothing happened.
    Refused {
        /// The token, when one could be recovered from the offending frame.
        #[serde(skip_serializing_if = "Option::is_none")]
        token: Option<CorrelationToken>,
        /// Which refusal this is.
        code: RefusalCode,
        /// A note for a developer. Nothing branches on it; a client reacts to `code`.
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
    },
}

/// Why a frame was refused at the protocol layer.
///
/// Distinct from [`Rejection`](mineworld_contracts::Rejection), which is the world's judgement
/// about a well-formed request: these are the ways a *frame* fails to be a request at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefusalCode {
    /// The frame did not decode: not JSON, no kind, or a kind whose fields do not fit.
    MalformedFrame,
    /// A frame of a kind this protocol does not have. A message that asserts state rather than
    /// requesting an action arrives here (`NETWORKING.md` §2).
    UnknownFrame,
    /// A request arrived before the connection had a seat, so there is no observer to act as.
    NotJoined,
    /// The connection already has a seat. A seat is held for the life of the connection: a client
    /// that could change it could change what it perceives.
    AlreadyJoined,
    /// No such seat. The roster is the world host's, not the client's.
    UnknownSeat,
    /// The seat names an entity this world does not have, which is a fault in the world's
    /// composition rather than in the client's frame.
    SeatNotInWorld,
    /// The request asked the world to act as somebody other than this connection's observer.
    ActorNotObserver,
    /// A system broke its own contract while resolving the request. Not a rejected request: the
    /// world reports it and the request had no answer.
    DispatchFailed,
    /// The world is no longer running, so there is nothing to submit to.
    WorldStopped,
}

/// One refusal, before it becomes a frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    code: RefusalCode,
    token: Option<CorrelationToken>,
    detail: Option<String>,
}

impl Refusal {
    /// A refusal with this code and nothing else said about it.
    pub const fn new(code: RefusalCode) -> Self {
        Self {
            code,
            token: None,
            detail: None,
        }
    }

    /// Pairs the refusal with the token of the frame that caused it, when one was recoverable.
    #[must_use]
    pub fn about(mut self, token: Option<CorrelationToken>) -> Self {
        self.token = token;
        self
    }

    /// Adds a developer-facing note.
    #[must_use]
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Adds another error's own words as the note.
    #[must_use]
    pub fn detailed(self, error: impl fmt::Display) -> Self {
        self.detail(error.to_string())
    }

    /// Which refusal this is.
    pub const fn code(&self) -> RefusalCode {
        self.code
    }

    /// The frame to send back.
    pub fn into_frame(self) -> ServerFrame {
        ServerFrame::Refused {
            token: self.token,
            code: self.code,
            detail: self.detail,
        }
    }
}

/// What can be wrong with a protocol value itself, before any world is involved.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProtocolError {
    /// A correlation token of an unusable length.
    #[error("a correlation token is 1 to {limit} bytes, and this one is {length}")]
    TokenLength {
        /// The offending length.
        length: usize,
        /// The limit.
        limit: usize,
    },
    /// A correlation token containing a control character.
    #[error("a correlation token must not contain control characters")]
    TokenNotPrintable,
    /// A world instance identity that is not the hexadecimal string one is written as.
    #[error("a world instance is 128 bits of lowercase hexadecimal, and this is {0:?}")]
    InstanceId(String),
    /// A seat name that is not a legal entity key.
    #[error("a seat is named by an entity key: {0}")]
    Seat(#[from] ContractError),
}
