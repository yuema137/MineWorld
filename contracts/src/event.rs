//! The fact half of the central pipeline: what the world records, once, immutably.
//!
//! The event log is the single source of truth for history (`INV-11`), and every other
//! historical view — a biography, a relationship summary, a client's notion of what just
//! happened — is derived from it. Two properties follow, and this module exists to make both
//! mechanical rather than remembered.
//!
//! # Every event states its cause
//!
//! `INV-15` requires every significant mutation to be traceable to an action, a process or a
//! system-emitted event, and `AC-9` requires that of any window of a running world. A
//! [`Causation`] is therefore not an optional field: there is no constructor, and no
//! deserialization path, that produces an [`EventEnvelope`] without one. [`Causation::WorldGenesis`]
//! exists so that a world's initial state is *explained* rather than uncaused — the alternative
//! would be an `Option` that every genesis event sets to `None`, and an `Option` that must be
//! checked everywhere is how a causeless event eventually gets written.
//!
//! # Every event states its audience
//!
//! [`Visibility`] is likewise required. A perception system (S10) decides who *actually* learned
//! of an event, but it can only decide that from a declared audience; an event that did not say
//! who could have perceived it would default to omniscience, which is the failure `INV-13`
//! forbids. Declaring the audience is the emitting system's job, because only it knows whether
//! the fact was shouted across a room or noticed by nobody.
//!
//! # What is not here
//!
//! No event is named. `ItemTransferred` and `ConversationStarted` are System Pack vocabulary
//! (`INV-12`); this module provides the machinery for declaring an event type and carrying its
//! payload. Nothing here appends to a log, orders a log, replays one or reduces one: persistence
//! and reduction arrive with their own layers.

use core::fmt;
use std::borrow::Cow;
use std::collections::BTreeSet;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::{ContractError, IdentifierKind};
use crate::ids::{
    ActionId, EntityId, EventId, PlaceId, ProcessId, SystemId, check_identifier,
    validate_identifier,
};
use crate::time::WorldTime;

/// The declared name of a kind of event: the slug a system emits under.
///
/// Obeys the identifier rule documented in [`crate::ids`], checked by the same implementation, so
/// an event type written as a literal in code and one read from a persisted record cannot be
/// judged differently.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct EventTypeId(Cow<'static, str>);

impl EventTypeId {
    /// Validates a declared event type name against the identifier rule stated in [`crate::ids`].
    pub fn new(value: impl Into<String>) -> Result<Self, ContractError> {
        let value = value.into();
        validate_identifier(IdentifierKind::EventTypeId, &value)?;
        Ok(Self(Cow::Owned(value)))
    }

    /// Declares the name as a literal in code, checked while the declaring crate compiles.
    ///
    /// An illegal literal is a compile error, so a declaration that would be rejected when a
    /// record is read back never reaches a running world.
    pub const fn from_static(value: &'static str) -> Self {
        match check_identifier(value) {
            Ok(()) => Self(Cow::Borrowed(value)),
            Err(_) => panic!(
                "an event type id literal must be 1 to 64 bytes of lowercase ASCII letters, \
                 digits, '-' and '_', and must not begin or end with a separator"
            ),
        }
    }

    /// The name as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for EventTypeId {
    type Error = ContractError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for EventTypeId {
    type Error = ContractError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<EventTypeId> for String {
    fn from(value: EventTypeId) -> Self {
        value.0.into_owned()
    }
}

impl fmt::Display for EventTypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The version of an event payload's schema.
///
/// Deliberately a separate type from
/// [`ComponentSchemaVersion`](crate::component::ComponentSchemaVersion) rather than a shared
/// generic: component state and recorded facts version independently, and a value of one is
/// never a value of the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventSchemaVersion(u32);

impl EventSchemaVersion {
    /// Declares a version.
    pub const fn new(version: u32) -> Self {
        Self(version)
    }

    /// The version as a number, for a migration that has to compare or step through them.
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl core::fmt::Display for EventSchemaVersion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "v{}", self.0)
    }
}

/// A kind of fact one system emits.
///
/// Implementing this trait *is* the declaration, exactly as for
/// [`Component`](crate::component::Component) and [`Action`](crate::action::Action): the event
/// type and the emitting system are part of the type, so no event kind exists without an owner,
/// and a system cannot emit another system's events by accident.
///
/// ```
/// use mineworld_contracts::{Event, EventSchemaVersion, EventTypeId, SystemId};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct ExampleHappened {
///     count: u32,
/// }
///
/// impl Event for ExampleHappened {
///     const EVENT_TYPE: EventTypeId = EventTypeId::from_static("example-happened");
///     const OWNER: SystemId = SystemId::from_static("example-system");
///     const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
/// }
/// ```
pub trait Event: Serialize + DeserializeOwned + Sized {
    /// The name of this kind of fact.
    const EVENT_TYPE: EventTypeId;
    /// The system that emits it.
    const OWNER: SystemId;
    /// The version of this payload's schema.
    ///
    /// Required for the same reason a component declares one, and for a stronger one: an event
    /// log is append-only and permanent, so a payload written today is read by code that does
    /// not exist yet. Without a version on the record, the first schema change to an event
    /// payload would be undetectable on replay — the old bytes would decode into the new shape
    /// and silently rebuild a history that never happened.
    const SCHEMA_VERSION: EventSchemaVersion;
}

/// An event's payload as a log or a wire carries it: labelled and opaque.
///
/// The event family's one payload-erasure boundary, for the same reason
/// [`ComponentRecord`](crate::component::ComponentRecord) exists: a table row and a network frame
/// carry bytes, not Rust types. `P` is the encoded form the persistence or transport layer chose,
/// and this crate never interprets it.
///
/// The label cannot lie: a record is built from an event type, and [`EventRecord::payload_for`]
/// refuses to hand the payload to an event type it was not written for.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EventRecord<P = Vec<u8>> {
    event_type: EventTypeId,
    schema_version: EventSchemaVersion,
    payload: P,
}

impl<P> EventRecord<P> {
    /// Labels an already-encoded payload with the event type it was encoded from.
    ///
    /// The type comes from `E` rather than from the caller, so a record cannot be mislabelled.
    pub fn new<E: Event>(payload: P) -> Self {
        Self {
            event_type: E::EVENT_TYPE,
            schema_version: E::SCHEMA_VERSION,
            payload,
        }
    }

    /// The event type the payload was written from.
    pub const fn event_type(&self) -> &EventTypeId {
        &self.event_type
    }

    /// The schema version the payload was written against.
    ///
    /// A store or a migration reads this without decoding the payload, which is the point: an
    /// old record has to be recognizable as old before anything tries to interpret it.
    pub const fn schema_version(&self) -> EventSchemaVersion {
        self.schema_version
    }

    /// The encoded payload, unchecked — for a store or a transport moving a record it does not
    /// interpret. Code that intends to decode it uses [`EventRecord::payload_for`].
    pub const fn payload(&self) -> &P {
        &self.payload
    }

    /// The payload, if this record was written for `E`.
    ///
    /// A record of another event type must not be decoded on the chance that it fits: replaying a
    /// log by guessing at payloads would rebuild a state that never existed.
    pub fn payload_for<E: Event>(&self) -> Result<&P, ContractError> {
        if self.event_type != E::EVENT_TYPE {
            return Err(ContractError::EventTypeMismatch {
                expected: E::EVENT_TYPE,
                actual: self.event_type.clone(),
            });
        }
        if self.schema_version > E::SCHEMA_VERSION {
            return Err(ContractError::EventSchemaTooNew {
                event_type: E::EVENT_TYPE,
                record: self.schema_version,
                supported: E::SCHEMA_VERSION,
            });
        }
        if self.schema_version < E::SCHEMA_VERSION {
            return Err(ContractError::EventSchemaOutdated {
                event_type: E::EVENT_TYPE,
                record: self.schema_version,
                supported: E::SCHEMA_VERSION,
            });
        }
        Ok(&self.payload)
    }
}

/// Why an event happened: the link that makes history traceable (`INV-15`).
///
/// Closed on purpose. These five are the only ways anything can happen in a MineWorld world, and
/// a sixth would mean something can change a world without an actor, a process, a consequence or
/// a clock behind it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Causation {
    /// A controller or a client asked, and the owning system resolved the request. The identity is
    /// the [`ActionIntent`](crate::action::ActionIntent)'s, so a result and the facts it produced
    /// name the same request.
    Action(ActionId),
    /// A process that takes simulated time reached a point that produced a fact — a travel
    /// arriving, a meal ending.
    Process(ProcessId),
    /// Another event. This is how a consequence is recorded as a consequence rather than as a
    /// coincidence: a system that reacts to a fact names the fact it reacted to.
    Event(EventId),
    /// A system acting on its own schedule rather than on anyone's request.
    SystemTick {
        /// The system whose tick produced the fact.
        system: SystemId,
    },
    /// The world coming into existence. A loaded World Pack's initial facts are caused by this and
    /// by nothing else, so initial state is explained rather than uncaused.
    WorldGenesis,
}

/// Who could have learned of an event.
///
/// A declared audience, not an observed one: whether a particular person *did* learn of the fact
/// is a perception system's decision (S10), and it needs this to decide from. Closed, because a
/// perception system must be able to handle every case a world can produce.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    /// Anyone in the world could have learned of it — a public announcement, a change of season.
    Public,
    /// Anyone present in this place.
    Place(PlaceId),
    /// The entities the envelope lists as participants, and no one else.
    Participants,
    /// Exactly these entities. A [`BTreeSet`], so the audience iterates and serializes in one
    /// fixed order however it was assembled: an event log whose bytes depend on insertion order
    /// cannot be compared across two runs of the same seeded world.
    Entities(BTreeSet<EntityId>),
    /// No one. Bookkeeping a system records for itself; a perception system shows it to nobody,
    /// and a controller must never receive it.
    SystemInternal,
}

/// Which system emitted an event and, where there was one, which request led to it.
///
/// Debugging and auditing information about the *emission*, distinct from [`Causation`], which is
/// a claim about the world's history. `controller_decision` is what connects a fact back to the
/// controller that asked for it, which is what makes an LM controller's effect on a world
/// reviewable after the fact.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Provenance {
    emitted_by: SystemId,
    controller_decision: Option<ActionId>,
}

impl Provenance {
    /// Records the emitting system. Every event has one: a fact with no emitter would be a fact no
    /// system owns.
    pub const fn new(emitted_by: SystemId) -> Self {
        Self {
            emitted_by,
            controller_decision: None,
        }
    }

    /// Records the request a controller submitted, for an event that exists because one did.
    #[must_use]
    pub fn from_controller_decision(mut self, action: ActionId) -> Self {
        self.controller_decision = Some(action);
        self
    }

    /// The system that emitted the event.
    pub const fn emitted_by(&self) -> &SystemId {
        &self.emitted_by
    }

    /// The request a controller submitted, if the event came from one.
    pub const fn controller_decision(&self) -> Option<ActionId> {
        self.controller_decision
    }
}

/// One immutable fact, with everything needed to place it in history.
///
/// The ten fields are those of `docs/CORE_CONCEPTS.md` §11. Six of them are required and are
/// supplied to [`EventEnvelope::new`]: identity, time, type, cause, audience and provenance. A
/// fact missing any of those would be unplaceable, unexplainable or unperceivable.
///
/// The other four are optional because a world may legitimately not have them: an event about
/// nothing in particular has no subjects, an event with no bystanders has no participants, and an
/// event that is not about a location — a clock advancing, an organization's accounts closing — has
/// no place.
///
/// # What immutable means here
///
/// Every field is private and no accessor hands out a mutable reference, so no holder of an
/// envelope can alter a fact. The `with_*` methods are part of building one, not of editing it:
/// they consume the value, and there is no assignment path — see
/// `tests/compile_fail/event_cannot_be_edited_after_construction.rs`.
///
/// # Why the place is a [`PlaceId`] and not a [`Location`](crate::spatial::Location)
///
/// The authoritative record is semantic (`INV-5`, `ENGINEERING_RULES.md` §5): *what happened, and
/// where in the world's own vocabulary*. A millimetre position is a refinement of where an
/// **entity** is, and an orientation is a property an entity has; neither is a property of a fact,
/// and a `facing` field on every event would be a field that can never mean anything. A system
/// whose events genuinely carry continuous geometry — a movement system recording a new position —
/// puts it in its own typed payload, where it is that system's contract rather than the kernel's.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(
    try_from = "EventEnvelopeFields<P>",
    bound(deserialize = "P: Deserialize<'de>")
)]
pub struct EventEnvelope<P = Vec<u8>> {
    id: EventId,
    at: WorldTime,
    event_type: EventTypeId,
    subjects: Vec<EntityId>,
    participants: Vec<EntityId>,
    place: Option<PlaceId>,
    caused_by: Causation,
    payload: EventRecord<P>,
    visibility: Visibility,
    provenance: Provenance,
}

impl<P> EventEnvelope<P> {
    /// The six facts an event cannot exist without.
    ///
    /// The event type is read off the payload record rather than taken as an argument, so the
    /// envelope's label and its contents cannot be given inconsistently. Everything else a fact
    /// may have is attached with [`EventEnvelope::about`],
    /// [`EventEnvelope::with_participants`] and [`EventEnvelope::at_place`].
    pub fn new(
        id: EventId,
        at: WorldTime,
        payload: EventRecord<P>,
        caused_by: Causation,
        visibility: Visibility,
        provenance: Provenance,
    ) -> Self {
        Self {
            id,
            at,
            event_type: payload.event_type().clone(),
            subjects: Vec::new(),
            participants: Vec::new(),
            place: None,
            caused_by,
            payload,
            visibility,
            provenance,
        }
    }

    /// Names the entities the fact is *about*: those whose state the fact concerns, and which a
    /// reducer therefore reads it for.
    ///
    /// Ordered rather than a set, because the order can be part of the fact — a transfer's two
    /// ends are not interchangeable — and because a system that means a set can supply one.
    #[must_use]
    pub fn about(mut self, subjects: Vec<EntityId>) -> Self {
        self.subjects = subjects;
        self
    }

    /// Names the entities that took part without necessarily being what the fact is about: those
    /// present at a conversation, in a room, at a meal. This is also the audience
    /// [`Visibility::Participants`] refers to.
    #[must_use]
    pub fn with_participants(mut self, participants: Vec<EntityId>) -> Self {
        self.participants = participants;
        self
    }

    /// Records where in the world's semantic space the fact happened.
    #[must_use]
    pub fn at_place(mut self, place: PlaceId) -> Self {
        self.place = Some(place);
        self
    }

    /// Identity of this fact in the log.
    pub const fn id(&self) -> EventId {
        self.id
    }

    /// When it happened, on the world's clock.
    pub const fn at(&self) -> WorldTime {
        self.at
    }

    /// What kind of fact it is.
    pub const fn event_type(&self) -> &EventTypeId {
        &self.event_type
    }

    /// The entities the fact is about, in the order the emitting system stated them.
    pub fn subjects(&self) -> &[EntityId] {
        &self.subjects
    }

    /// The entities that took part.
    pub fn participants(&self) -> &[EntityId] {
        &self.participants
    }

    /// Where it happened, if it happened anywhere in particular.
    pub const fn place(&self) -> Option<PlaceId> {
        self.place
    }

    /// Why it happened.
    pub const fn caused_by(&self) -> &Causation {
        &self.caused_by
    }

    /// The fact's contents, still encoded.
    pub const fn payload(&self) -> &EventRecord<P> {
        &self.payload
    }

    /// Who could have learned of it.
    pub const fn visibility(&self) -> &Visibility {
        &self.visibility
    }

    /// Which system emitted it, and which controller decision led to it.
    pub const fn provenance(&self) -> &Provenance {
        &self.provenance
    }
}

/// The serialized shape of an [`EventEnvelope`], read back through the same agreement check that
/// construction applies.
///
/// A required field is required here too: a record with no `caused_by` or no `visibility` fails to
/// deserialize, which is what keeps `INV-15` and the declared-audience rule true for a log read
/// from disk and not merely for one built in memory.
#[derive(Deserialize)]
struct EventEnvelopeFields<P> {
    id: EventId,
    at: WorldTime,
    event_type: EventTypeId,
    subjects: Vec<EntityId>,
    participants: Vec<EntityId>,
    place: Option<PlaceId>,
    caused_by: Causation,
    payload: EventRecord<P>,
    visibility: Visibility,
    provenance: Provenance,
}

impl<P> TryFrom<EventEnvelopeFields<P>> for EventEnvelope<P> {
    type Error = ContractError;

    fn try_from(value: EventEnvelopeFields<P>) -> Result<Self, Self::Error> {
        if value.event_type != *value.payload.event_type() {
            return Err(ContractError::EventEnvelopePayloadMismatch {
                event_type: value.event_type,
                payload_event_type: value.payload.event_type().clone(),
            });
        }
        Ok(Self {
            id: value.id,
            at: value.at,
            event_type: value.event_type,
            subjects: value.subjects,
            participants: value.participants,
            place: value.place,
            caused_by: value.caused_by,
            payload: value.payload,
            visibility: value.visibility,
            provenance: value.provenance,
        })
    }
}
