//! The request half of the central pipeline: what is asked of the world, and what the world
//! answers.
//!
//! `docs/CORE_CONCEPTS.md` §12 states the pipeline this module opens:
//!
//! ```text
//! ActionIntent → Validate → Resolve → Event(s) → Reducer(s) → New World State
//! ```
//!
//! Everything here belongs to the first arrow only. An [`ActionIntent`] is a *request*
//! (`INV-2`): a controller or a client says what it would like to happen, and nothing about
//! holding one makes it true. Validation, resolution and dispatch are the kernel's and the
//! systems' work and arrive with them; this module contains no rule, no check about whether an
//! action is possible, and no way to turn an intent into state.
//!
//! # A client submits a request; the world makes it an intent
//!
//! The first arrow has two halves, because identity is the world's to give (`INV-6`):
//!
//! ```text
//! ActionRequest    what a controller or client submits: who, what, of whom, from where
//!      │           no ActionId, no WorldTime — a client has neither an allocator nor the clock
//!      ▼           ActionIntent::allocate
//! ActionIntent     what the world dispatches, carrying the identity and instant it assigned
//! ```
//!
//! That split is `INV-6` in the types rather than in prose. It exists because a spike measured what
//! its absence costs: two Godot clients each invented an [`ActionId`] from a local counter, so they
//! collide on their first action, and the event log's causal chain would be anchored on identity a
//! client chose (`spike/FINDINGS.md` F4).
//!
//! # Why the answer is a value and not an error
//!
//! [`ActionResult::Unavailable`] is a first-class answer (`INV-10`). An action that no enabled
//! system provides does not exist in that world, and saying so is an ordinary outcome rather
//! than a failure of the machinery — a controller that asks to shoot in a world without a
//! combat system is not malfunctioning, and neither is the world. [`Rejection`] is likewise a
//! value a client can render, not a `ContractError`: `ContractError` means *this value is
//! malformed*, while a rejection means *the world considered your well-formed request and said
//! no*.
//!
//! # Kernel vocabulary, and the systems' own reasons
//!
//! `docs/ENGINEERING_RULES.md` §8 names the answers every client must be able to show, so those
//! are kernel vocabulary and a closed enum. A system must nonetheless be able to add a reason of
//! its own without editing the kernel, or installing a System Pack would amplify into a change
//! here (`docs/ENGINEERING_STANDARDS.md` change-amplification test). That is what
//! [`Rejection::System`] and [`RejectionCode`] are for.
//!
//! # No action is named here
//!
//! `talk`, `give_item` and `move` are System Pack vocabulary (`INV-12`). This module provides
//! the machinery for declaring an action type and carrying its payload; it does not know a
//! single one of them, and the only action names in this crate are in its tests.

use core::fmt;
use std::borrow::Cow;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::{ContractError, IdentifierKind};
use crate::ids::{ActionId, EntityId, EventId, SystemId, check_identifier, validate_identifier};
use crate::spatial::Location;
use crate::time::WorldTime;

/// The declared name of a kind of action: the slug a system provides and a client asks for.
///
/// Distinct from [`ActionId`], and deliberately so (`DD-1`): this names a *kind* of request that
/// exists for as long as its system is installed, while an [`ActionId`] identifies one submitted
/// request and is what correlates that request with its answer and with the events it caused.
///
/// Obeys the identifier rule documented in [`crate::ids`], checked by the same implementation, so
/// an action type written as a literal in code and one read from an authored file cannot be
/// judged differently.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ActionTypeId(Cow<'static, str>);

impl ActionTypeId {
    /// Validates a declared action type name against the identifier rule stated in
    /// [`crate::ids`].
    pub fn new(value: impl Into<String>) -> Result<Self, ContractError> {
        let value = value.into();
        validate_identifier(IdentifierKind::ActionTypeId, &value)?;
        Ok(Self(Cow::Owned(value)))
    }

    /// Declares the name as a literal in code, checked while the declaring crate compiles.
    ///
    /// This is what lets an action state its type and its owning system as part of its type
    /// rather than as data: an associated constant cannot hold a validated `String`, but it can
    /// hold this. An illegal literal is a compile error, so a declaration that would be rejected
    /// at load time never reaches a running world.
    pub const fn from_static(value: &'static str) -> Self {
        match check_identifier(value) {
            Ok(()) => Self(Cow::Borrowed(value)),
            Err(_) => panic!(
                "an action type id literal must be 1 to 64 bytes of lowercase ASCII letters, \
                 digits, '-' and '_', and must not begin or end with a separator"
            ),
        }
    }

    /// The name as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ActionTypeId {
    type Error = ContractError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for ActionTypeId {
    type Error = ContractError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<ActionTypeId> for String {
    fn from(value: ActionTypeId) -> Self {
        value.0.into_owned()
    }
}

impl fmt::Display for ActionTypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A reason of a system's own, beyond the reasons the kernel names.
///
/// Owned by the system that rejects: `closed-for-the-night` is a reason a hypothetical opening
/// hours system has, and the kernel must never learn it. A client that does not recognize a code
/// falls back to showing the request as refused, which is why [`Rejection::System`] is not a hole
/// in the client contract.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RejectionCode(Cow<'static, str>);

impl RejectionCode {
    /// Validates a rejection code against the identifier rule stated in [`crate::ids`].
    pub fn new(value: impl Into<String>) -> Result<Self, ContractError> {
        let value = value.into();
        validate_identifier(IdentifierKind::RejectionCode, &value)?;
        Ok(Self(Cow::Owned(value)))
    }

    /// Declares the code as a literal in code, checked while the declaring crate compiles: a
    /// system's rejection reasons are part of its own source, not data it loads.
    pub const fn from_static(value: &'static str) -> Self {
        match check_identifier(value) {
            Ok(()) => Self(Cow::Borrowed(value)),
            Err(_) => panic!(
                "a rejection code literal must be 1 to 64 bytes of lowercase ASCII letters, \
                 digits, '-' and '_', and must not begin or end with a separator"
            ),
        }
    }

    /// The code as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for RejectionCode {
    type Error = ContractError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for RejectionCode {
    type Error = ContractError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<RejectionCode> for String {
    fn from(value: RejectionCode) -> Self {
        value.0.into_owned()
    }
}

impl fmt::Display for RejectionCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Why a well-formed request was refused.
///
/// The first five variants are the answers `docs/ENGINEERING_RULES.md` §8 requires every client
/// to be able to show, which is what makes them kernel vocabulary rather than any one system's:
/// a 2D client greying out a menu entry and a 3D client refusing an interaction prompt are
/// reacting to the same closed set.
///
/// There is deliberately no `Display`: the English text a player reads is presentation, and a
/// contract that carried it would put localizable strings in the kernel (`DD-13`). A client maps
/// a variant to its own wording.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rejection {
    /// The actor or the target is occupied with something that cannot be interrupted this way.
    Busy,
    /// A spatial requirement of the action is not met: the wrong place, or too great a distance.
    TooFarAway,
    /// The actor is not permitted to do this, whether or not it is physically possible.
    PermissionDenied,
    /// The target exists and is reachable but offers no interaction of this kind.
    NoSupportedInteraction,
    /// The target exists but is not currently available to be acted upon.
    TargetUnavailable,
    /// The action exists, the owning system considered the request, and refused it without
    /// classifying the reason further.
    ///
    /// Distinct from [`ActionResult::Unavailable`], which is a different statement: that one
    /// means no enabled system provides this action type at all, so the action does not exist in
    /// this world and the answer would be the same for any actor at any moment (`INV-10`). This
    /// one means the action does exist and this attempt was refused.
    Unavailable,
    /// A declared precondition of the action could not be satisfied or could not be evaluated.
    PreconditionFailed,
    /// A reason belonging to the rejecting system rather than to the kernel.
    System {
        /// The rejecting system's own code.
        code: RejectionCode,
        /// A note for a log or a developer. Nothing branches on it; a client that needs to react
        /// differently reacts to `code`.
        detail: Option<String>,
    },
}

/// What the world answers a submitted [`ActionIntent`].
///
/// Three outcomes, and the difference between the last two is `INV-10`: a rejection is a
/// judgement about this request, while [`ActionResult::Unavailable`] is a statement that the
/// action itself does not exist in this world — there is no `shoot()` in a universe with no
/// combat system, however convincingly a controller asks.
///
/// The answer does not carry the [`ActionId`] it answers, and that survives the arrival of
/// [`ActionRequest`] — which is worth stating, because a client no longer allocates the id and the
/// obvious worry is that it can therefore no longer correlate.
///
/// Three facts, together, are why the field is still absent:
///
/// ```text
/// the producer already holds it   an ActionResult comes from dispatching an ActionIntent, and
///                                 ActionIntent::allocate is the only way one exists — so whoever
///                                 has the answer has the id, and a field here would restate it
/// request <-> answer              a protocol concern: the submitter's own correlation token in its
///                                 own frame. A token in this type would be a protocol field in the
///                                 kernel's vocabulary, and it would still not prevent a mismatched
///                                 pair — it would only look as though it had
/// answer <-> facts                already carried: Accepted { events } names the very facts the
///                                 request caused, so a client finds its own events without ever
///                                 learning the ActionId
/// ```
///
/// One case is genuinely open and belongs to the protocol layer rather than here: an action that
/// starts a process produces later facts, whose [`Causation`](crate::event::Causation) names the
/// action, and `Accepted { events }` cannot list facts that do not exist yet. A transport whose
/// client needs to recognize those must tell it the allocated id — the server holds it, having
/// allocated it — and that is a field in a protocol frame, not in the kernel's answer.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionResult {
    /// The request was resolved, and these are the events it caused, in the order they were
    /// recorded.
    ///
    /// An empty list says the request was accepted and produced no recorded fact. The kernel does
    /// not forbid that; whether a particular action may resolve to nothing belongs to the
    /// contract of the system that provides it.
    Accepted {
        /// The events the request caused.
        events: Vec<EventId>,
    },
    /// The request was well formed and was refused.
    Rejected(Rejection),
    /// No enabled system provides this action type, so the action does not exist here
    /// (`INV-10`).
    Unavailable,
}

/// A kind of action one system provides.
///
/// Implementing this trait *is* the declaration: both constants are part of the type, so an
/// action cannot exist without naming the system that owns it, and two intents of the same action
/// type cannot disagree about who resolves them. That is what lets dispatch route a request to
/// exactly one system and answer [`ActionResult::Unavailable`] when no enabled system provides
/// the type at all.
///
/// ```
/// use mineworld_contracts::{Action, ActionTypeId, SystemId};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct ExampleRequest {
///     repetitions: u32,
/// }
///
/// impl Action for ExampleRequest {
///     const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("example-action");
///     const OWNER: SystemId = SystemId::from_static("example-system");
/// }
/// ```
///
/// There is no schema version here, unlike [`Component`](crate::component::Component). A
/// component is stored state that outlives the code that wrote it, so a reader must be able to
/// tell an older row from a newer one. An action payload lives only between submission and
/// resolution, and never enters the event log — what the log records is the [`Event`] the action
/// caused, which carries its own payload.
///
/// [`Event`]: crate::event::Event
pub trait Action: Serialize + DeserializeOwned + Sized {
    /// The name of this kind of request.
    const ACTION_TYPE: ActionTypeId;
    /// The one system that resolves it.
    const OWNER: SystemId;
}

/// An action's payload as a transport carries it: labelled and opaque.
///
/// One of this crate's payload-erasure boundaries, and the only one for the action family: a
/// network frame carries bytes, not a Rust type, so something has to hold a payload this crate
/// cannot interpret. `P` is the encoded form the transport chose — bytes by default — and the
/// encoding is deliberately not decided here.
///
/// What the type does guarantee is that the label cannot lie: a record can only be built from an
/// action type, and [`ActionRecord::payload_for`] refuses to hand the payload to an action type it
/// was not written for. Unlike [`ComponentRecord`](crate::component::ComponentRecord) it names no
/// entity, because a request is not state attached to one: the actor and the target are fields of
/// the [`ActionIntent`] that carries the record.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ActionRecord<P = Vec<u8>> {
    action_type: ActionTypeId,
    payload: P,
}

impl<P> ActionRecord<P> {
    /// Labels an already-encoded payload with the action type it was encoded from.
    ///
    /// The type comes from `A` rather than from the caller, so a record cannot be mislabelled:
    /// the only way to claim an action type is to name it as a type parameter.
    pub fn new<A: Action>(payload: P) -> Self {
        Self {
            action_type: A::ACTION_TYPE,
            payload,
        }
    }

    /// Labels an already-encoded payload with an action type held as a value.
    ///
    /// Crate-private: the only caller is
    /// [`Affordance::request`](crate::observation::Affordance::request), which labels a complete
    /// affordance's payload with the affordance's **own** action type — the one the owning system
    /// offered it as — so the label still comes from the offer rather than from whoever submits it
    /// (`ARC-34`). No trust is lost by having it: a record already deserializes from any label, and
    /// dispatch decodes the payload with the owning system's type.
    pub(crate) const fn labelled(action_type: ActionTypeId, payload: P) -> Self {
        Self {
            action_type,
            payload,
        }
    }

    /// The action type the payload was written from.
    pub const fn action_type(&self) -> &ActionTypeId {
        &self.action_type
    }

    /// The encoded payload, unchecked — for a transport moving a record it does not interpret.
    /// Code that intends to decode the payload into an action uses [`ActionRecord::payload_for`].
    pub const fn payload(&self) -> &P {
        &self.payload
    }

    /// The payload, if this record was written for `A`.
    ///
    /// A record of a different action type is a routing mistake by whoever delivered it, and must
    /// not be decoded on the chance that it fits: with `serde` defaults it might even succeed,
    /// producing a plausible request nobody made.
    pub fn payload_for<A: Action>(&self) -> Result<&P, ContractError> {
        if self.action_type != A::ACTION_TYPE {
            return Err(ContractError::ActionTypeMismatch {
                expected: A::ACTION_TYPE,
                actual: self.action_type.clone(),
            });
        }
        Ok(&self.payload)
    }
}

/// What a controller or a client *submits*: a request with no identity and no instant, because it
/// has neither to give.
///
/// This is the type-level form of `INV-6`. [`crate::ids`] says of every runtime identity that
/// "allocation belongs to the kernel, not to this crate" and that `from_raw` "is not a way to invent
/// one" — and yet an [`ActionIntent`] cannot be built without an [`ActionId`], so anything outside
/// the kernel that wanted to submit one had to invent identity it has no right to allocate. A
/// renderer-integration spike measured the consequence: both of its clients used a local counter, so
/// two clients collide on the first action each submits and a server cannot tell a collision from a
/// retransmission — while [`Causation::Action`](crate::event::Causation::Action) and
/// [`Provenance::controller_decision`](crate::event::Provenance::controller_decision) would anchor
/// the event log's causal chain on client-chosen identity (`spike/FINDINGS.md` F4).
///
/// So the division is now in the types rather than in prose:
///
/// ```text
/// a client or controller submits   ActionRequest    who, what, of whom, from where
/// the world makes it an intent     ActionIntent     + the ActionId and the WorldTime it assigns
/// ```
///
/// `issued_at` is absent for the same reason `action_id` is: a client does not have the world's
/// clock. The spike's clients echoed the `at` of the last observation they had received — the time of
/// a past frame rather than of the request — which was harmless there and wrong in principle.
///
/// Correlating an answer with a request is a *protocol* concern and is deliberately not here: a
/// transport pairs its own frames, with its own token, and a token in this type would be a protocol
/// field in the kernel's vocabulary.
///
/// The fields are private for the same reason [`ActionIntent`]'s are: `action_type` is the type the
/// payload record was written from, and no construction path — including deserialization — may let a
/// request claim one type while carrying another. That check matters more here than anywhere, because
/// a request is exactly the value a non-Rust client hand-builds.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(
    try_from = "ActionRequestFields<P>",
    bound(deserialize = "P: Deserialize<'de>")
)]
pub struct ActionRequest<P = Vec<u8>> {
    actor: EntityId,
    action_type: ActionTypeId,
    target: Option<EntityId>,
    payload: ActionRecord<P>,
    actor_location: Option<Location>,
}

impl<P> ActionRequest<P> {
    /// The two facts a request cannot omit: who is asking, and what is being asked with what
    /// payload.
    ///
    /// The action type is read off the payload record rather than taken as an argument, so the two
    /// cannot be given inconsistently. A target and a reported location are added with
    /// [`ActionRequest::with_target`] and [`ActionRequest::from_location`], because an action may
    /// genuinely have neither: `docs/ENGINEERING_RULES.md` §7 names sending a message and applying
    /// for a remote job as actions with no target and no spatial grounding at all.
    pub fn new(actor: EntityId, payload: ActionRecord<P>) -> Self {
        Self {
            actor,
            action_type: payload.action_type().clone(),
            target: None,
            payload,
            actor_location: None,
        }
    }

    /// Names the entity the request is about.
    #[must_use]
    pub fn with_target(mut self, target: EntityId) -> Self {
        self.target = Some(target);
        self
    }

    /// Reports where the client believes the actor was when the request was made. See
    /// [`ActionIntent`] for why this is a report and not an assertion.
    #[must_use]
    pub fn from_location(mut self, location: Location) -> Self {
        self.actor_location = Some(location);
        self
    }

    /// Who is asking.
    pub const fn actor(&self) -> EntityId {
        self.actor
    }

    /// What kind of request this is.
    pub const fn action_type(&self) -> &ActionTypeId {
        &self.action_type
    }

    /// The entity the request is about, if it is about one.
    pub const fn target(&self) -> Option<EntityId> {
        self.target
    }

    /// The request's payload, still encoded.
    pub const fn payload(&self) -> &ActionRecord<P> {
        &self.payload
    }

    /// Where the client reports the actor was, if it reported anything.
    pub const fn actor_location(&self) -> Option<&Location> {
        self.actor_location.as_ref()
    }
}

/// The serialized shape of an [`ActionRequest`], read back through the same agreement check that
/// construction applies.
///
/// This is the boundary that most needs the check. A Rust caller goes through
/// [`ActionRequest::new`], which reads the action type off the record; a client hand-building JSON
/// writes both and can write them differently.
#[derive(Deserialize)]
struct ActionRequestFields<P> {
    actor: EntityId,
    action_type: ActionTypeId,
    target: Option<EntityId>,
    payload: ActionRecord<P>,
    actor_location: Option<Location>,
}

impl<P> TryFrom<ActionRequestFields<P>> for ActionRequest<P> {
    type Error = ContractError;

    fn try_from(value: ActionRequestFields<P>) -> Result<Self, Self::Error> {
        if value.action_type != *value.payload.action_type() {
            return Err(ContractError::ActionIntentPayloadMismatch {
                action_type: value.action_type,
                payload_action_type: value.payload.action_type().clone(),
            });
        }
        Ok(Self {
            actor: value.actor,
            action_type: value.action_type,
            target: value.target,
            payload: value.payload,
            actor_location: value.actor_location,
        })
    }
}

/// What a controller or a client asks the world for, once the world has given it an identity.
///
/// A request, never a fact (`INV-2`). Nothing here resolves, validates or records anything; an
/// intent that is never dispatched has no effect on any world, and an intent that is dispatched
/// may be answered with any [`ActionResult`].
///
/// The identity and the instant are the world's, not the submitter's. [`ActionIntent::allocate`]
/// takes an [`ActionRequest`] and adds them, and that is the path anything outside the kernel uses:
/// see [`ActionRequest`] for why a client that allocated its own [`ActionId`] would anchor the event
/// log's causal chain on identity it invented. [`ActionIntent::new`] remains for code that is *in*
/// the world — a system, a test, the dispatcher's own callers — where the allocator is at hand and
/// there is no request to convert.
///
/// `actor_location` is what the *client* reports about where the actor was when the request was
/// made — not the authoritative position, which the server holds. It exists because a spatial
/// requirement is evaluated against locations (see
/// [`SpatialRequirement::evaluate`](crate::spatial::SpatialRequirement::evaluate)), and because a
/// client may legitimately know a position the server has not applied yet: a 3D client sends the
/// position it walked to, a 2D client that models no position sends none. A server that distrusts
/// the report is free to ignore it and evaluate against its own state — that choice belongs to
/// dispatch, not to this type, and `ENGINEERING_RULES.md` §8 is why the client may only *report*
/// it.
///
/// The fields are private because two of them must agree: `action_type` is the type the payload
/// record was written from, and there is no construction path — including deserialization — that
/// lets an intent claim one type while carrying another.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(
    try_from = "ActionIntentFields<P>",
    bound(deserialize = "P: Deserialize<'de>")
)]
pub struct ActionIntent<P = Vec<u8>> {
    action_id: ActionId,
    actor: EntityId,
    action_type: ActionTypeId,
    target: Option<EntityId>,
    payload: ActionRecord<P>,
    issued_at: WorldTime,
    actor_location: Option<Location>,
}

impl<P> ActionIntent<P> {
    /// The four facts a request cannot omit: which request this is, who is asking, what is being
    /// asked with what payload, and when it was issued.
    ///
    /// For code **inside** the world: a system, a test, or a caller that already holds the
    /// allocator. Anything submitting from outside builds an [`ActionRequest`] and the world calls
    /// [`ActionIntent::allocate`], because nothing outside the kernel may invent an [`ActionId`].
    ///
    /// The action type is read off the payload record rather than taken as an argument, so the two
    /// cannot be given inconsistently. A target and a reported location are added with
    /// [`ActionIntent::with_target`] and [`ActionIntent::from_location`], because an action may
    /// genuinely have neither: `docs/ENGINEERING_RULES.md` §7 names sending a message and applying
    /// for a remote job as actions with no target and no spatial grounding at all.
    pub fn new(
        action_id: ActionId,
        actor: EntityId,
        payload: ActionRecord<P>,
        issued_at: WorldTime,
    ) -> Self {
        Self {
            action_id,
            actor,
            action_type: payload.action_type().clone(),
            target: None,
            payload,
            issued_at,
            actor_location: None,
        }
    }

    /// Turns what a client submitted into what the world will dispatch, by supplying the two things
    /// only the world has: the identity it allocated and the instant it is working in.
    ///
    /// This is the only way to obtain an intent from a request, and there is deliberately no way for
    /// a request to acquire an identity by itself. Every other field is carried across unchanged —
    /// the world does not edit what was asked for; it decides whether to grant it.
    pub fn allocate(request: ActionRequest<P>, action_id: ActionId, issued_at: WorldTime) -> Self {
        Self {
            action_id,
            actor: request.actor,
            action_type: request.action_type,
            target: request.target,
            payload: request.payload,
            issued_at,
            actor_location: request.actor_location,
        }
    }

    /// Names the entity the request is about.
    #[must_use]
    pub fn with_target(mut self, target: EntityId) -> Self {
        self.target = Some(target);
        self
    }

    /// Reports where the client believes the actor was when the request was made.
    #[must_use]
    pub fn from_location(mut self, location: Location) -> Self {
        self.actor_location = Some(location);
        self
    }

    /// Identity of this request, and the key its answer and its events refer back to.
    pub const fn action_id(&self) -> ActionId {
        self.action_id
    }

    /// Who is asking.
    pub const fn actor(&self) -> EntityId {
        self.actor
    }

    /// What kind of request this is.
    pub const fn action_type(&self) -> &ActionTypeId {
        &self.action_type
    }

    /// The entity the request is about, if it is about one.
    pub const fn target(&self) -> Option<EntityId> {
        self.target
    }

    /// The request's payload, still encoded.
    pub const fn payload(&self) -> &ActionRecord<P> {
        &self.payload
    }

    /// When the request was issued, on the world's clock.
    pub const fn issued_at(&self) -> WorldTime {
        self.issued_at
    }

    /// Where the client reports the actor was, if it reported anything.
    pub const fn actor_location(&self) -> Option<&Location> {
        self.actor_location.as_ref()
    }
}

/// The serialized shape of an [`ActionIntent`], read back through the same agreement check that
/// construction applies.
///
/// Without this, a hand-written or corrupted frame could claim one action type in its envelope
/// and carry another in its payload, and dispatch would route it by the label while the owning
/// system decoded something else.
#[derive(Deserialize)]
struct ActionIntentFields<P> {
    action_id: ActionId,
    actor: EntityId,
    action_type: ActionTypeId,
    target: Option<EntityId>,
    payload: ActionRecord<P>,
    issued_at: WorldTime,
    actor_location: Option<Location>,
}

impl<P> TryFrom<ActionIntentFields<P>> for ActionIntent<P> {
    type Error = ContractError;

    fn try_from(value: ActionIntentFields<P>) -> Result<Self, Self::Error> {
        if value.action_type != *value.payload.action_type() {
            return Err(ContractError::ActionIntentPayloadMismatch {
                action_type: value.action_type,
                payload_action_type: value.payload.action_type().clone(),
            });
        }
        Ok(Self {
            action_id: value.action_id,
            actor: value.actor,
            action_type: value.action_type,
            target: value.target,
            payload: value.payload,
            issued_at: value.issued_at,
            actor_location: value.actor_location,
        })
    }
}
