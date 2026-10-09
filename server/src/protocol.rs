//! The wire protocol: the only frames that exist, and what a client may therefore say.
//!
//! `docs/NETWORKING.md` §5 lists the message classes; this module is that list as closed Rust
//! enums, which is what makes the authority model structural rather than remembered. A client can
//! say exactly three things (`PROTOCOL.md` §2, revision 2):
//!
//! ```text
//! join     present the invite and a nickname, name a seat, and be told which observer it is
//! submit   an ActionRequest, and the client's own correlation token
//! leave    give the seat up and end the connection
//! ```
//!
//! and there is no fourth — no frame that sets a value, names another observer, widens a scope or
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
//! protocol              the frames, the refusal codes, and what can be wrong with a protocol value
//! protocol/request      a submitted request's payload and the client's correlation token
//! protocol/summary      what a world is: its instance identity and its composition
//! protocol/connection   what a frame says about the connection: its session, a takeover, a closing
//! protocol/fact         a recorded fact as a client receives it (PROTOCOL.md §5.2)
//! protocol/delta        one observation as the change from the previous one (PROTOCOL.md §5.3)
//! ```

mod connection;
pub mod delta;
mod fact;
mod request;
mod summary;
#[cfg(test)]
mod tests;

use std::fmt;

use mineworld_contracts::{
    ActionId, ActionRequest, ActionResult, ContractError, EntityId, EntityKey, EventId,
    Observation, PerceivedEvent, WorldTime,
};
use mineworld_persistence::WorldRevision;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::admission::{Nickname, OfferedInvite, OfferedResume, ResumeSecret};

pub use connection::{ClosingReason, SessionId, TookOver};
pub use fact::{PayloadForm, wire_fact};
pub(crate) use fact::{read_backfill, report_not_json};
pub use request::{CorrelationToken, MAX_TOKEN_LENGTH, WirePayload, into_kernel_request};
pub use summary::{ClockState, SystemSummary, WorldInstanceId, WorldSummary};

/// Which revision of this protocol a server speaks. A client that does not recognize the number
/// should refuse to connect rather than guess.
pub const PROTOCOL_VERSION: u32 = 2;

/// The revision a `join` without a `protocol` field speaks: revision 1's join had none, so such a
/// client is told `protocol_mismatch` rather than `malformed_frame`.
const fn revision_one() -> u32 {
    1
}

/// An `Observation` as this transport carries it.
///
/// `serde_json::Value` rather than the contract's `Vec<u8>` default, which
/// `spike/FINDINGS.md` F8.2 measured as unusable on a JSON wire: with the default a component's
/// payload reaches a client as an array of byte integers. The generic parameter exists precisely
/// so a transport can choose, and this is the choice F8.2 asked the server to state.
pub type WireObservation = Observation<Value>;

/// What a client may say. Three variants, and the closed set is the point.
///
/// Every variant denies unknown fields (`PROTOCOL.md` §2): a typo is loud, and a `join` that tries
/// to name an `observer` is refused rather than silently ignored. Decoding is structural only —
/// which of a join's fields are *acceptable* is the handshake's question, asked in the order
/// `PROTOCOL.md` §4.1 fixes, so a missing invite and a missing nickname decode as empty and are
/// answered there.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClientFrame {
    /// Ask to occupy a seat. The server answers with the observer that seat is, and a client
    /// never names an observer itself — which is why there is no request that widens perception
    /// (`INV-13`).
    Join {
        /// The revision the client speaks; `1` when absent.
        #[serde(default = "revision_one")]
        protocol: u32,
        /// The server's invite, as the client offers it. Empty when absent.
        #[serde(default)]
        invite: OfferedInvite,
        /// The player's nickname, unchecked. Empty when absent.
        #[serde(default)]
        nickname: String,
        /// The seat, named by the authoring key of the entity it belongs to.
        seat: EntityKey,
        /// A secret from an earlier `welcome`, to re-take a held seat (`PROTOCOL.md` §4.2).
        #[serde(default)]
        resume: Option<OfferedResume>,
        /// Take the seat from the connection that holds it, or from a dropped one's hold.
        #[serde(default)]
        take_over: bool,
        /// Ask for the reliable `perceived` stream, from a cursor (`PROTOCOL.md` §5.8).
        #[serde(default)]
        perceived: Option<PerceivedJoin>,
    },
    /// Submit a request. No identity and no instant: the server allocates both (`INV-6`).
    Submit {
        /// The client's own token, echoed on the answer.
        token: CorrelationToken,
        /// What is being asked of the world.
        request: ActionRequest<WirePayload>,
    },
    /// Give the seat up at once and end the connection.
    Leave {},
}

/// A `join`'s request for the `perceived` stream (`PROTOCOL.md` §5.8): the cursor to continue from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerceivedJoin {
    /// The `through` of the last `perceived` frame the client processed, or `null` for "from this
    /// world's first fact". Required: an absent `since` is malformed, not "from the beginning".
    #[serde(deserialize_with = "Option::deserialize")]
    pub since: Option<EventId>,
}

/// The tags a client frame may carry, which is also the whole of what a client may say.
const CLIENT_FRAME_TAGS: [&str; 3] = ["join", "submit", "leave"];

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
                    "this protocol has no frame of kind \"{tag}\"; a client may only join, submit \
                     or leave"
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
        /// The player's nickname, as accepted (trimmed). Shown to this connection only.
        nickname: Nickname,
        /// Which connection this is, for the admin surface. Not a credential.
        session: SessionId,
        /// The secret that re-takes this seat after a dropped socket. A fresh one on every welcome;
        /// `null` only from a server older than S11-B.
        resume: Option<ResumeSecret>,
        /// How long a dropped connection's seat is held, in wall seconds; `0` holds none.
        hold_seconds: u32,
        /// Whether control of the Person changed hands, and how (`PROTOCOL.md` §4.2).
        took_over: TookOver,
        /// What the world is, at the moment of joining.
        world: WorldSummary,
    },
    /// How the host is pacing the world's clock: right after `welcome`, then on every pause and
    /// resume (`PROTOCOL.md` §5.9).
    Clock {
        /// The world's instant when the clock last changed state (or at the welcome).
        at: WorldTime,
        /// World seconds per wall second.
        time_scale: u32,
        /// Whether the host has stopped the clock.
        paused: bool,
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
        /// The newest request submitted on this connection that had been dispatched when this
        /// observation was computed; `null` before the first (`PROTOCOL.md` §5.2).
        acted_through: Option<ActionId>,
        /// The observation itself.
        observation: WireObservation,
    },
    /// What changed since the previous frame on this connection (`PROTOCOL.md` §5.3, `DEP-15`).
    Delta {
        /// As on an observation.
        seq: u64,
        /// The `seq` of the frame this delta applies to: always the previous one.
        base: u64,
        /// As on an observation.
        revision: Option<WorldRevision>,
        /// As on an observation, stated whole.
        acted_through: Option<ActionId>,
        /// The change.
        delta: delta::ObservationDelta,
    },
    /// Facts this connection's observer learned, on the reliable stream it asked for
    /// (`PROTOCOL.md` §5.8).
    Perceived {
        /// The newest fact the server has considered for this connection: the client's cursor.
        through: EventId,
        /// The facts, oldest first.
        events: Vec<PerceivedEvent<Value>>,
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
    /// The server is about to close this connection, and says why. Never a refusal: being closed
    /// is not an answer to a frame.
    Closing {
        /// Why. A client branches on this.
        reason: ClosingReason,
        /// A note for a developer.
        #[serde(default, skip_serializing_if = "Option::is_none")]
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
    /// The `join` speaks a revision of this protocol the server does not. Followed by `closing`.
    ProtocolMismatch,
    /// The `join`'s invite was wrong or missing, answered after a fixed delay. Followed by
    /// `closing`: a connection gets one guess.
    Unauthorized,
    /// The `join`'s nickname is empty once trimmed, too long, or holds a control character.
    InvalidNickname,
    /// Another connection holds the seat, or it is held, and the `join` neither resumed nor took it
    /// over.
    SeatOccupied,
    /// The `join`'s `resume` matches neither the seat's hold nor its live connection.
    InvalidResume,
    /// The `join`'s `perceived.since` is a cursor this world cannot serve (`PROTOCOL.md` §5.8).
    CursorUnavailable,
    /// The connection's `perceived` stream fell further behind than the server holds. Followed by
    /// `closing`.
    Lagged,
    /// The host has paused the world's clock: the request was refused before it was given an
    /// identity, so nothing was half-done (`PROTOCOL.md` §5.9).
    Paused,
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
    /// A session identity that is not the decimal string one is written as.
    #[error("a session is a decimal integer written as a string, and this is {0:?}")]
    SessionId(String),
    /// A seat name that is not a legal entity key.
    #[error("a seat is named by an entity key: {0}")]
    Seat(#[from] ContractError),
}
