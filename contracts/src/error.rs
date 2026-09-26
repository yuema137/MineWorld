//! The single error type every validating constructor in this crate returns.
//!
//! One enum rather than one error type per module: a caller loading a World Pack or decoding a
//! persisted record handles all of these at the same boundary, and each variant carries the
//! offending values as fields so that the caller can react to them, rather than a
//! pre-formatted message it would have to parse.

use core::fmt;

use thiserror::Error;

use crate::action::ActionTypeId;
use crate::component::ComponentSchemaVersion;
use crate::entity::LifecycleState;
use crate::event::EventSchemaVersion;
use crate::event::EventTypeId;
use crate::ids::{ComponentTypeId, EntityId, EntityType, RelationTypeId};
use crate::relation::{EntityTypeSet, RelationEnd};

/// Which validated textual identifier a [`ContractError`] is describing.
///
/// Every identifier in this crate obeys the same character rule, so the error variants are
/// shared and this enum says which kind of name was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IdentifierKind {
    /// An [`crate::ids::EntityKey`]: the authoring name of an entity in a World Pack.
    EntityKey,
    /// A [`crate::ids::SystemId`].
    SystemId,
    /// A [`crate::ids::ComponentTypeId`].
    ComponentTypeId,
    /// A [`crate::ids::RelationTypeId`].
    RelationTypeId,
    /// An [`crate::action::ActionTypeId`]: the name of a kind of action a system provides.
    ActionTypeId,
    /// A [`crate::action::RejectionCode`]: a rejecting system's own reason code.
    RejectionCode,
    /// An [`crate::event::EventTypeId`]: the name of a kind of event a system emits.
    EventTypeId,
    /// A [`crate::entity::Tag`]: one semantic label on an entity.
    Tag,
}

impl fmt::Display for IdentifierKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::EntityKey => "entity key",
            Self::SystemId => "system id",
            Self::ComponentTypeId => "component type id",
            Self::RelationTypeId => "relation type id",
            Self::ActionTypeId => "action type id",
            Self::RejectionCode => "rejection code",
            Self::EventTypeId => "event type id",
            Self::Tag => "tag",
        };
        f.write_str(name)
    }
}

/// Every way a contract value can fail to be constructed or decoded.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContractError {
    /// A textual identifier was empty.
    #[error("an {kind} must not be empty")]
    IdentifierEmpty {
        /// Which identifier was rejected.
        kind: IdentifierKind,
    },

    /// A textual identifier exceeded [`crate::ids::MAX_IDENTIFIER_LENGTH`].
    #[error("an {kind} must be at most {max} bytes long, but this one is {length}")]
    IdentifierTooLong {
        /// Which identifier was rejected.
        kind: IdentifierKind,
        /// The rejected length, in bytes.
        length: usize,
        /// The permitted maximum, in bytes.
        max: usize,
    },

    /// A textual identifier contained a character outside the permitted set.
    #[error(
        "an {kind} may contain only lowercase ASCII letters, digits, '-' and '_', \
         but this one contains {character:?} at byte {position}"
    )]
    IdentifierIllegalCharacter {
        /// Which identifier was rejected.
        kind: IdentifierKind,
        /// The first offending character.
        character: char,
        /// Its byte offset in the rejected value.
        position: usize,
    },

    /// A textual identifier began or ended with a separator.
    #[error("an {kind} must not begin or end with {character:?}")]
    IdentifierSeparatorAtEdge {
        /// Which identifier was rejected.
        kind: IdentifierKind,
        /// The separator found at the beginning or the end.
        character: char,
    },

    /// A typed entity reference was constructed for an entity of a different type.
    #[error("entity {entity} is of type {actual}, so it cannot be referenced as {expected}")]
    EntityTypeMismatch {
        /// The entity that was referenced.
        entity: EntityId,
        /// The type the reference requires.
        expected: EntityType,
        /// The type the entity actually has.
        actual: EntityType,
    },

    /// A lifecycle change the state machine does not permit, such as anything at all out of
    /// `Destroyed`.
    #[error("entity {entity} cannot move from {from} to {to}")]
    IllegalLifecycleTransition {
        /// The entity whose lifecycle was to change.
        entity: EntityId,
        /// The state it is in.
        from: LifecycleState,
        /// The state it was asked to move to.
        to: LifecycleState,
    },

    /// A component record was read as a component type it was not written for.
    #[error("a {actual} record cannot be read as {expected}")]
    ComponentTypeMismatch {
        /// The component type the caller asked for.
        expected: ComponentTypeId,
        /// The component type the record was written from.
        actual: ComponentTypeId,
    },

    /// A component record was written by a newer schema than the reading code knows. Refused
    /// rather than guessed at: a newer schema may carry fields this code would drop.
    #[error("{component_type} record is {record}, newer than the {supported} this code reads")]
    ComponentSchemaTooNew {
        /// The component type being read.
        component_type: ComponentTypeId,
        /// The version the record was written against.
        record: ComponentSchemaVersion,
        /// The version the reading code supports.
        supported: ComponentSchemaVersion,
    },

    /// A component record predates the reading code's schema. Not a corruption: it is ordinary
    /// history, and a migration has to bring it forward rather than the reader assuming.
    #[error("{component_type} record is {record}, older than the {supported} this code reads")]
    ComponentSchemaOutdated {
        /// The component type being read.
        component_type: ComponentTypeId,
        /// The version the record was written against.
        record: ComponentSchemaVersion,
        /// The version the reading code supports.
        supported: ComponentSchemaVersion,
    },

    /// An event record was written by a newer schema than the reading code knows. Refused
    /// rather than guessed at, and more consequentially than for a component: the event log is
    /// permanent, so a misread payload rebuilds a history that never happened.
    #[error("{event_type} record is {record}, newer than the {supported} this code reads")]
    EventSchemaTooNew {
        /// The event type being read.
        event_type: EventTypeId,
        /// The version the record was written against.
        record: EventSchemaVersion,
        /// The version the reading code supports.
        supported: EventSchemaVersion,
    },

    /// An event record predates the reading code's schema. Ordinary history in an append-only
    /// log, and precisely what a replay of an old world will meet: a migration brings it
    /// forward, the reader does not assume.
    #[error("{event_type} record is {record}, older than the {supported} this code reads")]
    EventSchemaOutdated {
        /// The event type being read.
        event_type: EventTypeId,
        /// The version the record was written against.
        record: EventSchemaVersion,
        /// The version the reading code supports.
        supported: EventSchemaVersion,
    },

    /// A relation type declared an endpoint that permits no entity type at all.
    #[error("a relation type must permit at least one entity type at each end")]
    EmptyEntityTypeSet,

    /// An entity of the wrong type was offered as one end of a relation.
    #[error(
        "entity {entity} is of type {actual}, which {relation_type} does not permit          as its {end} end (permitted: {permitted})"
    )]
    RelationEndpointNotPermitted {
        /// The relation type that refused the endpoint.
        relation_type: RelationTypeId,
        /// Which end was refused.
        end: RelationEnd,
        /// The entity offered there.
        entity: EntityId,
        /// The type it has.
        actual: EntityType,
        /// The types the declaration permits at that end.
        permitted: EntityTypeSet,
    },

    /// An entity was offered as both ends of a relation type that forbids it.
    #[error("{relation_type} does not permit entity {entity} to relate to itself")]
    SelfEdgeNotPermitted {
        /// The relation type that refused the edge.
        relation_type: RelationTypeId,
        /// The entity offered at both ends.
        entity: EntityId,
    },

    /// An action record was read as an action type it was not written for.
    #[error("a {actual} action payload cannot be read as {expected}")]
    ActionTypeMismatch {
        /// The action type the caller asked for.
        expected: ActionTypeId,
        /// The action type the record was written from.
        actual: ActionTypeId,
    },

    /// A serialized action intent named one action type in its envelope and carried another in
    /// its payload. Refused rather than resolved either way: dispatch routes by the envelope and
    /// the owning system decodes the payload, so the two disagreeing means one of them would act
    /// on a request nobody made.
    #[error("an action intent for {action_type} cannot carry a {payload_action_type} payload")]
    ActionIntentPayloadMismatch {
        /// The action type the intent's envelope named.
        action_type: ActionTypeId,
        /// The action type its payload record was written from.
        payload_action_type: ActionTypeId,
    },

    /// An event record was read as an event type it was not written for.
    #[error("a {actual} event payload cannot be read as {expected}")]
    EventTypeMismatch {
        /// The event type the caller asked for.
        expected: EventTypeId,
        /// The event type the record was written from.
        actual: EventTypeId,
    },

    /// A serialized event envelope named one event type and carried a payload written from
    /// another. Refused rather than decoded: replaying a log whose labels disagree with its
    /// contents would rebuild a state that never existed.
    #[error("an event envelope of {event_type} cannot carry a {payload_event_type} payload")]
    EventEnvelopePayloadMismatch {
        /// The event type the envelope named.
        event_type: EventTypeId,
        /// The event type its payload record was written from.
        payload_event_type: EventTypeId,
    },

    /// An orientation was given a pitch steeper than straight up or straight down. Refused rather
    /// than clamped: unlike a yaw past a full turn, such a value is not another way of writing a
    /// legal one.
    #[error("a pitch of {millidegrees}mdeg is steeper than the {limit}mdeg limit")]
    PitchOutOfRange {
        /// The rejected pitch, in millidegrees.
        millidegrees: i32,
        /// The steepest legal pitch, in millidegrees.
        limit: i32,
    },

    /// An affordance claimed to be available and also carried a reason it was not, or claimed to
    /// be unavailable and gave no reason. A client shown both would have to decide which half to
    /// believe, and one shown neither has nothing to tell the player.
    #[error(
        "an affordance that is available must carry no reason and one that is not must carry one, \
         but this one says available={available} with has_reason={has_reason}"
    )]
    AffordanceAvailabilityDisagreement {
        /// What the affordance claimed about its availability.
        available: bool,
        /// Whether it carried a reason.
        has_reason: bool,
    },

    /// A spatial requirement declared a negative interaction range. Rejected where the
    /// declaration is written, because an evaluator comparing squared distances would treat it as
    /// its own absolute value and silently accept interactions at that distance.
    #[error("an interaction range must not be negative, but this one is {millimetres}mm")]
    NegativeInteractionRange {
        /// The rejected range, in millimetres.
        millimetres: i32,
    },
}
