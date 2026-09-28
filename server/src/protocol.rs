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

#[cfg(test)]
mod tests;

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use mineworld_contracts::{
    ActionId, ActionRequest, ActionResult, ContractError, EntityId, EntityKey, Observation,
    SystemId, WorldTime,
};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

/// Which revision of this protocol a server speaks. A client that does not recognize the number
/// should refuse to connect rather than guess.
pub const PROTOCOL_VERSION: u32 = 1;

/// The longest correlation token a client may choose.
pub const MAX_TOKEN_LENGTH: usize = 64;

/// An `Observation` as this transport carries it.
///
/// `serde_json::Value` rather than the contract's `Vec<u8>` default, which
/// `spike/FINDINGS.md` F8.2 measured as unusable on a JSON wire: with the default a component's
/// payload reaches a client as an array of byte integers. The generic parameter exists precisely
/// so a transport can choose, and this is the choice F8.2 asked the server to state.
pub type WireObservation = Observation<Value>;

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

/// Which running world this is.
///
/// Not a name, not a secret and not a UUID: a value that distinguishes *this* running world from
/// another one. It exists because `docs/MVP.md` §9.1 requires `AC-15`'s evidence to name identity
/// rather than appearance — *same world instance*, first of three lines — and without it the only
/// argument that two clients are connected to one world is that somebody typed one address twice.
/// Two clients that were each talking to their own server would be told two different instances
/// here, which is exactly the false success §9.1 exists to exclude.
///
/// Allocated when a world's thread starts (`runtime::WorldRuntime::new`) from the wall clock, the
/// process and a per-process ordinal, so two worlds never share one — in the same process because
/// the ordinal differs, and across processes because the instant and the process do. It is
/// deliberately **not** derived from the world's content: two worlds loaded from the same World
/// Pack are two instances, and `AC-15` is about the instance.
///
/// On the wire it is a lowercase hexadecimal string, for the reason every identity is a string
/// (`DEP-3`): a JSON parser whose only number type is a double cannot carry 128 bits, and cannot
/// carry 64 either.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub struct WorldInstanceId(u128);

impl WorldInstanceId {
    /// The next instance identity this process will hand out.
    pub(crate) fn allocate() -> Self {
        // A world within this process, and this process at this instant. Not a cryptographic
        // identity: nothing authenticates with it, and the only property required is that two
        // worlds do not collide.
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let ordinal = u128::from(NEXT.fetch_add(1, Ordering::Relaxed));
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_nanos());
        let process = u128::from(std::process::id());
        Self((nanos << 32) ^ (process << 16) ^ ordinal)
    }

    /// The identity a caller already holds — for a test, or for reading one back off the wire.
    pub const fn from_raw(value: u128) -> Self {
        Self(value)
    }

    /// The identity as the number it is.
    pub const fn raw(self) -> u128 {
        self.0
    }
}

impl fmt::Display for WorldInstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

impl From<WorldInstanceId> for String {
    fn from(value: WorldInstanceId) -> Self {
        value.to_string()
    }
}

impl TryFrom<String> for WorldInstanceId {
    type Error = ProtocolError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        u128::from_str_radix(&value, 16)
            .map(Self)
            .map_err(|_| ProtocolError::InstanceId(value))
    }
}

/// What a world is, as a client or an operator is told: enough to see that it is running and what
/// it is composed of, and nothing that would make this an observation.
///
/// Deliberately not a view of state. It names the systems a world installed — which is public
/// information about its composition, the same information a World Pack states — and counts its
/// entities. It lists no entity, no component and no position, because a client's knowledge of
/// state arrives only through an `Observation` (`INV-13`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldSummary {
    /// Which revision of this protocol the server speaks.
    pub protocol: u32,
    /// Which running world this is — the same value for every client connected to it.
    pub instance: WorldInstanceId,
    /// The world's own clock, as of this answer.
    pub at: WorldTime,
    /// How many entities the world has allocated.
    pub entities: usize,
    /// The systems this world is composed of, in registration order.
    pub systems: Vec<SystemSummary>,
    /// The seats a client may ask for.
    pub seats: Vec<EntityKey>,
    /// How many clients are connected and seated.
    pub clients: usize,
    /// How many observations have been dropped because a client was not reading them.
    ///
    /// Reported rather than hidden: the world delivers observations without waiting for anybody, so
    /// a slow client loses frames, and a number that only ever appeared in a comment would make
    /// that policy invisible to whoever is running the server.
    pub observations_dropped: u64,
    /// How many deferrals dispatch handed back with no scheduler to queue them.
    ///
    /// Zero in a world whose systems defer nothing. Any other number is the size of what S4 will
    /// take over, and until then it is work this server was asked for and could not do.
    pub deferrals_unscheduled: u64,
    /// How many dispatches ended in a system breaking its own contract.
    ///
    /// `kernel/src/dispatch.rs`: an error out of dispatch is a bug in a system, not a rejected
    /// request. This server keeps serving and counts them here.
    pub faults: u64,
}

/// One installed system, as a status answer names it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemSummary {
    /// The system's name.
    pub system: SystemId,
    /// Whether it is currently in the pipeline. A disabled system's actions are `Unavailable`.
    pub enabled: bool,
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
