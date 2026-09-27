//! What a controller is given: a filtered view, and never the world.
//!
//! `INV-13` is the invariant this module exists to make mechanical: *controllers receive
//! `Observation`s, never global world state, and omniscience must be impossible by construction,
//! not by convention.* The construction is simple and it is the whole point — an [`Observation`]
//! is a **value** listing what the world chose to expose. It holds no component store, no entity
//! registry, no world handle and no query method, so there is nothing in it to widen: a controller
//! that wants to know something the world did not put in its observation has nowhere to look.
//!
//! *Which* facts a world exposes is a perception system's decision (S10) and depends on which
//! systems a world installs — vision, hearing, rumour, telephone, news. This module is the shape
//! their answers take.
//!
//! # Why the server computes what a client may attempt
//!
//! `docs/ENGINEERING_RULES.md` §§4 and 8 forbid a renderer from deciding whether an interaction is
//! valid: the client gathers intent and the authoritative server answers. But a client still has to
//! draw something — "press E to talk", or a greyed-out menu entry — and if the contract did not
//! carry the answer, every client would compute it from whatever it could see, which is exactly the
//! duplicated rule logic §9 calls an architectural failure.
//!
//! [`Affordance`] is that answer. The server states which action types this observer may attempt
//! against which target, whether each is available *right now*, and why not when it is not. A 2D
//! client greys out a menu entry; a 3D client shows a prompt over the thing the camera is pointing
//! at; neither evaluates distance, permission, availability or system presence, and both send the
//! same [`ActionIntent`](crate::action::ActionIntent) when the player acts.
//!
//! An affordance deliberately carries no display text (`DD-13`). The words and the language are
//! presentation, and a contract that carried them would pull localization into the kernel. It
//! carries the action type and the target's identity; the client maps the type to its own wording
//! and reads the target's *name* from a component in the same observation, which is world data and
//! must not be invented by a client.

use serde::{Deserialize, Serialize};

use crate::action::{ActionTypeId, Rejection};
use crate::component::ComponentRecord;
use crate::entity::Tags;
use crate::error::ContractError;
use crate::event::EventEnvelope;
use crate::ids::{EntityId, EntityType};
use crate::relation::Relation;
use crate::spatial::{Location, SpatialRequirement};
use crate::time::WorldTime;

/// One entity as an observer perceives it — and only as far as a perception system chose to expose
/// it.
///
/// Not a copy of the entity: the components are the ones the world decided this observer may know
/// about, so two observers of the same entity can legitimately receive different lists, and a
/// controller cannot tell the difference between a component that does not exist and one it was not
/// shown. That asymmetry is the feature.
///
/// The `id` is also the join key a client needs in order to bind a rendered body back to a world
/// entity (`DD-14`): a 3D client attaches it to whatever node it drew and recovers it when the
/// player points at that node, and a 2D client attaches it to a sprite or a list row. How it does
/// that is the client's business, and the contract says nothing about it — which is the evidence
/// that no engine concept has to enter this crate.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PerceivedEntity<P = Vec<u8>> {
    id: EntityId,
    entity_type: EntityType,
    location: Option<Location>,
    tags: Tags,
    components: Vec<ComponentRecord<P>>,
}

impl<P> PerceivedEntity<P> {
    /// The two facts an observer always has about something it perceives at all: which entity it is
    /// and what kind of thing it is.
    pub fn new(id: EntityId, entity_type: EntityType) -> Self {
        Self {
            id,
            entity_type,
            location: None,
            tags: Tags::default(),
            components: Vec::new(),
        }
    }

    /// Exposes where the entity is. Absent when the world does not model a location for it, or when
    /// the observer is not entitled to know it.
    #[must_use]
    pub fn at(mut self, location: Location) -> Self {
        self.location = Some(location);
        self
    }

    /// Exposes the entity's tags.
    #[must_use]
    pub fn with_tags(mut self, tags: Tags) -> Self {
        self.tags = tags;
        self
    }

    /// Exposes exactly these components, still encoded. This is where a client reads a target's
    /// name from (`DD-13`) — as world data owned by whichever system provides it, never as text the
    /// kernel invented.
    #[must_use]
    pub fn with_components(mut self, components: Vec<ComponentRecord<P>>) -> Self {
        self.components = components;
        self
    }

    /// Which entity this is, and the key a client binds its own rendering to.
    pub const fn id(&self) -> EntityId {
        self.id
    }

    /// Which of the four kinds of thing it is.
    pub const fn entity_type(&self) -> EntityType {
        self.entity_type
    }

    /// Where it is, as far as the observer is entitled to know.
    pub const fn location(&self) -> Option<&Location> {
        self.location.as_ref()
    }

    /// Its tags, as far as the observer is entitled to know.
    pub const fn tags(&self) -> &Tags {
        &self.tags
    }

    /// The components the world chose to expose to this observer.
    pub fn components(&self) -> &[ComponentRecord<P>] {
        &self.components
    }
}

/// An event the observer is entitled to have learned of.
///
/// A distinct type from [`EventEnvelope`] on purpose: an envelope is a log entry, and handing log
/// entries to a controller is precisely the omniscience `INV-13` forbids. A perception system
/// decides, from the event's declared [`Visibility`](crate::event::Visibility), which envelopes
/// become perceived events for which observer, and this type is the result of that decision. Code
/// that receives one knows the filtering already happened.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PerceivedEvent<P = Vec<u8>>(EventEnvelope<P>);

impl<P> PerceivedEvent<P> {
    /// Marks an envelope as one this observer is entitled to. Called by whatever made that
    /// judgement — a perception system — and by nothing else.
    pub const fn new(envelope: EventEnvelope<P>) -> Self {
        Self(envelope)
    }

    /// The fact, as the log recorded it.
    pub const fn envelope(&self) -> &EventEnvelope<P> {
        &self.0
    }
}

/// One thing the observer may attempt, and the server's answer about whether it can right now.
///
/// The server computes this; a client renders it (`DD-11`). That division is what lets a 2D and a
/// 3D client offer the same interactions without either implementing a rule — and it is why the
/// type carries a [`Rejection`] rather than a bare `false`: a client that knows only *that*
/// something is unavailable can only grey it out, while one told `TooFarAway` can say so, and a
/// player who is told nothing learns nothing.
///
/// `requirement` is the declaration the action's owning system made, passed through unevaluated, so
/// a client can *show* what an action needs — a reach, a place — without checking it.
///
/// Availability and its reason cannot disagree: an affordance is built by
/// [`Affordance::available`] or [`Affordance::unavailable`], and deserialization applies the same
/// check. Without that, "available, because too far away" would be representable, and a client
/// would have to decide which half to believe.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "AffordanceFields")]
pub struct Affordance {
    action_type: ActionTypeId,
    target: Option<EntityId>,
    available: bool,
    unavailable_reason: Option<Rejection>,
    requirement: SpatialRequirement,
}

impl Affordance {
    /// Something the observer may attempt right now.
    pub const fn available(
        action_type: ActionTypeId,
        target: Option<EntityId>,
        requirement: SpatialRequirement,
    ) -> Self {
        Self {
            action_type,
            target,
            available: true,
            unavailable_reason: None,
            requirement,
        }
    }

    /// Something the observer may attempt in principle but not right now, with the reason a client
    /// should show.
    pub const fn unavailable(
        action_type: ActionTypeId,
        target: Option<EntityId>,
        requirement: SpatialRequirement,
        reason: Rejection,
    ) -> Self {
        Self {
            action_type,
            target,
            available: false,
            unavailable_reason: Some(reason),
            requirement,
        }
    }

    /// Which kind of action this is. A client maps it to its own wording; the kernel has none
    /// (`DD-13`).
    pub const fn action_type(&self) -> &ActionTypeId {
        &self.action_type
    }

    /// What the action would be directed at, if anything.
    pub const fn target(&self) -> Option<EntityId> {
        self.target
    }

    /// Whether the server says this can be attempted right now.
    pub const fn is_available(&self) -> bool {
        self.available
    }

    /// Why not, when it cannot.
    pub const fn unavailable_reason(&self) -> Option<&Rejection> {
        self.unavailable_reason.as_ref()
    }

    /// What the owning system declared the action requires of space, unevaluated.
    pub const fn requirement(&self) -> &SpatialRequirement {
        &self.requirement
    }
}

/// The serialized shape of an [`Affordance`], with the same agreement check construction applies.
#[derive(Deserialize)]
struct AffordanceFields {
    action_type: ActionTypeId,
    target: Option<EntityId>,
    available: bool,
    unavailable_reason: Option<Rejection>,
    requirement: SpatialRequirement,
}

impl TryFrom<AffordanceFields> for Affordance {
    type Error = ContractError;

    fn try_from(value: AffordanceFields) -> Result<Self, Self::Error> {
        if value.available != value.unavailable_reason.is_none() {
            return Err(ContractError::AffordanceAvailabilityDisagreement {
                available: value.available,
                has_reason: value.unavailable_reason.is_some(),
            });
        }
        Ok(Self {
            action_type: value.action_type,
            target: value.target,
            available: value.available,
            unavailable_reason: value.unavailable_reason,
            requirement: value.requirement,
        })
    }
}

/// Everything one observer knows at one moment, and nothing else.
///
/// This is the whole of `INV-13` in one sentence: the type is a list of what was exposed. It has no
/// field that could hold a world, no handle to a store, no accessor that takes an identity and goes
/// looking, and no method that could return an entity it does not list. [`Observation::entity`]
/// searches the list and nothing but the list, which is why asking it about an unperceived entity
/// answers `None` rather than reaching for the answer.
///
/// `self_location` is the observer's own position, which is what lets a controller decide to move
/// before deciding to act.
///
/// # Why relations are here, and what stops them widening the view
///
/// Place hierarchy and place adjacency are *relations*, deliberately: `spatial.rs` states that "a
/// kitchen is inside a café is a relation (S1), not a field of this type". A place entity therefore
/// has no [`Location`] of its own — giving the café a location inside itself is nonsense — and
/// without relations an observation can describe one room and never a world. "Walk out of the café
/// and along the promenade" is what this project exists to build, and it was unrepresentable in this
/// type (`spike/FINDINGS.md` F3).
///
/// `relations` closes that, and is bounded the same way every other field here is: the perception
/// system that decided which entities and which events this observer may know about decides the
/// edges too. It is **not** the world's relation graph, not a subgraph the observer may query, and
/// not a table a client filters — it is the edges the world chose to state, and `INV-13` holds for
/// exactly the reason it holds for `entities`: there is nothing here to widen and nothing to ask
/// again.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Observation<P = Vec<u8>> {
    observer: EntityId,
    at: WorldTime,
    self_location: Option<Location>,
    entities: Vec<PerceivedEntity<P>>,
    /// The edges the observer is entitled to know about — never the world's relation graph.
    ///
    /// Filled by the same perception step that fills `entities`, and subject to the same rule: an
    /// edge appears here because a perception system decided this observer may know it. A
    /// perception system that put every edge in a world here would defeat `INV-13` as surely as one
    /// that listed every entity, and the two mistakes are the same mistake.
    ///
    /// An edge may name an entity the observation does not list. That is not a defect: knowing that
    /// the café adjoins the promenade is a different piece of knowledge from perceiving the
    /// promenade, and a client must treat an unlisted endpoint as something it has been told about
    /// rather than something it can draw.
    relations: Vec<Relation>,
    events: Vec<PerceivedEvent<P>>,
    affordances: Vec<Affordance>,
}

impl<P> Observation<P> {
    /// An observation with nothing in it: who is observing, and when. A world where a person
    /// perceives nothing at all is a legitimate world, and it is also the safe starting point —
    /// what is exposed has to be added deliberately.
    pub fn new(observer: EntityId, at: WorldTime) -> Self {
        Self {
            observer,
            at,
            self_location: None,
            entities: Vec::new(),
            relations: Vec::new(),
            events: Vec::new(),
            affordances: Vec::new(),
        }
    }

    /// Tells the observer where it is.
    #[must_use]
    pub fn at_location(mut self, location: Location) -> Self {
        self.self_location = Some(location);
        self
    }

    /// Exposes exactly these entities, and by that act exactly no others.
    #[must_use]
    pub fn perceiving(mut self, entities: Vec<PerceivedEntity<P>>) -> Self {
        self.entities = entities;
        self
    }

    /// Exposes exactly these relations, and by that act exactly no others.
    ///
    /// Called by the perception system that also chose the entities, because the judgement is the
    /// same judgement. A caller that means "every edge in the world" is not exposing an
    /// observation.
    #[must_use]
    pub fn relating(mut self, relations: Vec<Relation>) -> Self {
        self.relations = relations;
        self
    }

    /// Exposes the events the observer is entitled to have learned of.
    #[must_use]
    pub fn with_events(mut self, events: Vec<PerceivedEvent<P>>) -> Self {
        self.events = events;
        self
    }

    /// States what the observer may attempt, with the server's answer for each.
    #[must_use]
    pub fn offering(mut self, affordances: Vec<Affordance>) -> Self {
        self.affordances = affordances;
        self
    }

    /// Whose view this is.
    pub const fn observer(&self) -> EntityId {
        self.observer
    }

    /// When it was taken, on the world's clock.
    pub const fn at(&self) -> WorldTime {
        self.at
    }

    /// Where the observer itself is, as far as it is told.
    pub const fn self_location(&self) -> Option<&Location> {
        self.self_location.as_ref()
    }

    /// Everything the observer perceives. This list is exhaustive by definition: it is not a cache,
    /// a page or a first result set, and there is no second call that would return more.
    pub fn entities(&self) -> &[PerceivedEntity<P>] {
        &self.entities
    }

    /// The edges the observer was shown. Exhaustive in the same sense as
    /// [`Observation::entities`]: this is the whole of what was exposed, and there is no second
    /// call that would return more.
    pub fn relations(&self) -> &[Relation] {
        &self.relations
    }

    /// The events the observer is entitled to.
    pub fn events(&self) -> &[PerceivedEvent<P>] {
        &self.events
    }

    /// What the observer may attempt.
    pub fn affordances(&self) -> &[Affordance] {
        &self.affordances
    }

    /// The perceived entity with this identity, if the observation lists it.
    ///
    /// A search of [`Observation::entities`], not a lookup in a world: this is how a client resolves
    /// an affordance's target to something it can name and draw (`DD-13`). `None` means *this
    /// observer was not shown that entity*, which a controller must treat as ignorance rather than
    /// as absence — and which it cannot resolve by asking again, because there is nothing else to
    /// ask.
    pub fn entity(&self, id: EntityId) -> Option<&PerceivedEntity<P>> {
        self.entities.iter().find(|entity| entity.id() == id)
    }
}
