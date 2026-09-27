//! The installable system: what it declares, and the four things a world asks of it.

use mineworld_contracts::{
    ActionIntent, EntityId, EntityType, EntityTypeSet, Event, EventEnvelope, LifecycleState,
    PersonId, Rejection, RejectionCode, Relation, RelationTypeDeclaration, RelationTypeId,
    SystemId, Visibility,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    WorldRead, WorldView,
};

use crate::action::{Arrive, arrive_requirement};
use crate::codec;
use crate::component::Presence;
use crate::event::Arrived;
use crate::interaction::{InteractionProvider, Offer};

/// Where people are, and what each of them perceives.
///
/// A unit struct, like most System Packs: a system owns no fields, because its mutable state is the
/// components it owns and those live in the world behind a gated view (`INV-7`). That is also why
/// holding a second value of this type — to ask it for [`Offer`]s after a world has taken the first
/// one — is safe rather than merely convenient.
pub struct PresenceSystem;

impl SystemIdentity for PresenceSystem {
    const ID: SystemId = SystemId::from_static("presence");
}

/// The name of the edge this pack declares: a person is in a place.
///
/// A function rather than a constant because [`RelationTypeId`] is validated at construction and has
/// no `const` path; the literal is checked here, once, and every caller gets the same value.
pub fn present_in() -> RelationTypeId {
    RelationTypeId::new("present-in").expect("a legal relation type name")
}

/// What this pack declares about that edge: directed, from a person to a place.
///
/// The edge and the [`Presence`] component are the same fact at two resolutions, and both are
/// written in one place — [`PresenceSystem::react`] — so they cannot drift. Keeping both is
/// deliberate: the component carries the position a 3D client needs, while the edge is what makes
/// "who is in this place" a question the relation graph can answer, and what lets an
/// [`Observation`](mineworld_contracts::Observation) state containment as a relation rather than
/// leaving a client to infer it (`contracts/src/observation.rs`, `spike/FINDINGS.md` F3).
///
/// Only people, for now. An item on a table is a relation an inventory or interior pack will
/// declare about state it owns; widening this endpoint set to cover it here would be this pack
/// claiming that state.
pub fn present_in_declaration() -> RelationTypeDeclaration {
    RelationTypeDeclaration::directed(
        present_in(),
        PresenceSystem::ID,
        EntityTypeSet::new([EntityType::Person]).expect("a non-empty set"),
        EntityTypeSet::new([EntityType::Place]).expect("a non-empty set"),
    )
}

/// This pack's own reason for refusing a request it cannot read.
///
/// A [`Rejection::System`] code rather than one of the kernel's five, because a malformed payload is
/// not a fact about the world — it is a fact about the request. A client that does not recognize the
/// code shows the request as refused, which is the correct outcome, and a developer building a
/// client by hand gets the detail that tells them why.
const MALFORMED_PAYLOAD: RejectionCode = RejectionCode::from_static("malformed-payload");

impl System for PresenceSystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<Presence>()
            .providing::<Arrive>()
            .emitting::<Arrived>()
            .subscribing_to::<Arrived>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Presence>()?;
        tables.relation(present_in_declaration())
    }

    /// Decides whether somebody can be somewhere: the actor is a person who is still in the world,
    /// and the destination is a place this world has.
    ///
    /// `arrive` requires nothing of space (see [`arrive_requirement`]), so there is no distance to
    /// check here — and this is the one place in this pack that says so out loud. The checks that do
    /// run are about identity rather than geometry, and none of them writes anything: the view is
    /// read-only, which is the type's guarantee and not this function's discipline (`BD-6`).
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        let arrive: Arrive =
            codec::action_payload(intent.payload()).map_err(|error| Rejection::System {
                code: MALFORMED_PAYLOAD,
                detail: Some(error.to_string()),
            })?;

        let actor = world
            .entity(intent.actor())
            .ok_or(Rejection::PreconditionFailed)?;
        if actor.entity_type() != EntityType::Person {
            return Err(Rejection::NoSupportedInteraction);
        }
        if actor.lifecycle() == LifecycleState::Destroyed {
            return Err(Rejection::PreconditionFailed);
        }

        let place = world
            .entity(arrive.location().place().entity_id())
            .ok_or(Rejection::PreconditionFailed)?;
        if place.entity_type() != EntityType::Place {
            return Err(Rejection::PreconditionFailed);
        }
        Ok(())
    }

    /// States the arrival as a fact, and writes nothing.
    ///
    /// The component is written while reducing that fact, not here, so that the world's positions
    /// are a projection of its history rather than a parallel account of it.
    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let arrive: Arrive = codec::action_payload(intent.payload())?;
        let person = {
            let read = world.read();
            let actor = read.require_entity(intent.actor())?;
            PersonId::new(actor.id(), actor.entity_type())?
        };
        let location = arrive.location();

        Ok(vec![
            Emission::new::<Arrived>(
                codec::encode(&Arrived::new(person, location)),
                Visibility::Place(location.place()),
            )
            .about(vec![intent.actor()])
            .with_participants(vec![intent.actor()])
            .at_place(location.place()),
        ])
    }

    /// Reduces an arrival into the state this pack owns: the position, and the edge that says which
    /// place it is in.
    ///
    /// The old edge is removed when the place changes. Leaving it would make a person present in two
    /// places at once — a stale edge is not a harmless leftover, it is a false fact about the world,
    /// and it is exactly what a reader of the relation graph would believe.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if *event.event_type() != Arrived::EVENT_TYPE {
            return Ok(Vec::new());
        }
        let arrived: Arrived = codec::event_payload(event.payload())?;
        let person = arrived.person().entity_id();
        let arriving_at = arrived.location();

        let left = world
            .read()
            .component::<Presence>(person)
            .map(Presence::location)
            .filter(|previous| previous.place() != arriving_at.place());
        if let Some(left) = left {
            let edge = {
                let read = world.read();
                Relation::between(
                    &present_in_declaration(),
                    read.require_entity(person)?,
                    read.require_entity(left.place().entity_id())?,
                )?
            };
            world.unrelate(&edge)?;
        }

        world.insert(person, Presence::at(arriving_at))?;
        world.relate(&present_in(), person, arriving_at.place().entity_id())?;
        Ok(Vec::new())
    }
}

impl InteractionProvider for PresenceSystem {
    /// Offers `arrive` to any person: it is directed at nobody, so it is offered exactly once, with
    /// no target.
    ///
    /// This pack answers for its own action through the same seam every other pack uses, rather than
    /// through a shortcut into perception. That is not tidiness — a shortcut would be a second way of
    /// producing an affordance, and the second way is the one that stops agreeing with dispatch.
    fn offers(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        target: Option<EntityId>,
    ) -> Vec<Offer> {
        if target.is_some() {
            return Vec::new();
        }
        let is_living_person = world.entity(observer).is_some_and(|entity| {
            entity.entity_type() == EntityType::Person
                && entity.lifecycle() != LifecycleState::Destroyed
        });
        if !is_living_person {
            return Vec::new();
        }
        vec![Offer::new::<Arrive>(arrive_requirement())]
    }
}
