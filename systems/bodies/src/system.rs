//! The installable system: what it declares, the facts it reduces and the checks it reduces them
//! under, and what it discloses.

use mineworld_contracts::{
    ComponentRecord, EntityId, EntityType, Event, EventEnvelope, EventTypeId, PlaceId, Rejection,
    SystemId,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    WorldRead, WorldView,
};
use mineworld_presence::{PerceptionProvider, Presence, PresenceSystem, require_registered};
use mineworld_sdk::SystemPack;
use serde_json::Value;

use crate::codec;
use crate::component::{BodyShape, LooseObjects, PlaceShape};
use crate::event::{BodyFormed, ObjectMoved, ObjectPlaced, PlaceShaped};
use crate::geometry::Point;
use crate::{genesis, objects};

/// Bodies: places with walls and furniture, loose objects lying in them, and people who neither pass
/// through them nor through each other.
///
/// A unit struct, like every System Pack: its state is the [`PlaceShape`]s, [`BodyShape`]s and
/// [`LooseObjects`] it owns, held in the world, and the resolver it registers keeps nothing between
/// calls (`ARC-39`).
#[derive(Default)]
pub struct BodiesSystem;

impl SystemIdentity for BodiesSystem {
    const ID: SystemId = SystemId::from_static("bodies");
}

/// What the build needs to know about this pack beyond [`System`] (`DECISIONS.md` `ARC-33`): the
/// `body:` section of a place or an item file, which its `AuthoredSection` impl (`src/section.rs`)
/// describes. No biographical fact (step-11 QO-17).
impl SystemPack for BodiesSystem {
    mineworld_sdk::owns_section!();
}

/// A fact this pack may not take, reached at reduction: reported as the owner's refusal (`ARC-26`).
pub(crate) fn refused(event_type: EventTypeId, reason: Rejection) -> KernelError {
    KernelError::FactRefusedByOwner {
        system: BodiesSystem::ID,
        event_type,
        reason,
    }
}

impl System for BodiesSystem {
    /// Version 2: loose objects, pushes, `kick`, `throw` and `shove` change results and vocabulary, so
    /// a save written by version 1 is refused by name (`ARC-25`; step-11 SD-O20). A Rapier upgrade
    /// also changes results, so it bumps this version with it (`DEP-13`; `tests/rapier_pin.rs` holds
    /// the two together).
    const VERSION: SystemVersion = SystemVersion::new(2);

    /// Depends on presence (where people are, and the seam this pack resolves through); owns the
    /// shapes of places and objects and where the objects lie; states and reduces its own genesis
    /// facts.
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([PresenceSystem::ID])
            .owning::<PlaceShape>()
            .owning::<BodyShape>()
            .owning::<LooseObjects>()
            .emitting::<PlaceShaped>()
            .emitting::<BodyFormed>()
            .emitting::<ObjectPlaced>()
            .emitting::<ObjectMoved>()
            .subscribing_to::<PlaceShaped>()
            .subscribing_to::<BodyFormed>()
            .subscribing_to::<ObjectPlaced>()
            .subscribing_to::<ObjectMoved>()
    }

    /// Refuses to join a world whose host never registered this pack's resolver — before anything
    /// else, as `ARC-39` item 7 requires — and then declares its three tables.
    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        require_registered(&Self::ID);
        tables.component::<PlaceShape>()?;
        tables.component::<BodyShape>()?;
        tables.component::<LooseObjects>()
    }

    /// Reduces this pack's facts — the only writes it makes — each after the owner's check:
    ///
    /// ```text
    /// place-shaped   → PlaceShape       the people authored into the place fit it (SD-B4)
    /// body-formed    → BodyShape        a living Item with no shape yet; states object-placed
    /// object-placed  → LooseObjects     SD-O5's checks, in order
    /// object-moved   → LooseObjects     the object lies at `from`; `to` keeps SD-O2's invariant
    /// ```
    ///
    /// A refusal writes nothing and fails with [`KernelError::FactRefusedByOwner`], naming the
    /// subjects and the numbers.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let kind = event.event_type();
        if *kind == PlaceShaped::EVENT_TYPE {
            let shaped: PlaceShaped = codec::event_payload(event.payload())?;
            genesis::fits(&world.read(), shaped.place(), shaped.shape())
                .map_err(|reason| refused(PlaceShaped::EVENT_TYPE, reason))?;
            world.insert(shaped.place().entity_id(), shaped.shape().clone())?;
        } else if *kind == BodyFormed::EVENT_TYPE {
            let formed: BodyFormed = codec::event_payload(event.payload())?;
            let placed = genesis::formed(&world.read(), &formed)
                .map_err(|reason| refused(BodyFormed::EVENT_TYPE, reason))?;
            world.insert(formed.object().entity_id(), formed.shape())?;
            return Ok(vec![placed]);
        } else if *kind == ObjectPlaced::EVENT_TYPE {
            let placed: ObjectPlaced = codec::event_payload(event.payload())?;
            let row = genesis::placed(&world.read(), &placed)
                .map_err(|reason| refused(ObjectPlaced::EVENT_TYPE, reason))?;
            world.insert(placed.place().entity_id(), row)?;
        } else if *kind == ObjectMoved::EVENT_TYPE {
            let moved: ObjectMoved = codec::event_payload(event.payload())?;
            let row = objects::moved(&world.read(), &moved)
                .map_err(|reason| refused(ObjectMoved::EVENT_TYPE, reason))?;
            world.insert(moved.place().entity_id(), row)?;
        }
        Ok(Vec::new())
    }
}

/// Every person whose presence puts them in `place` at a position, in `EntityId` order — the order a
/// scene inserts people in (step-11 DC-2). A person in the place with no position has no body here.
pub(crate) fn standing_in(world: &WorldRead<'_>, place: PlaceId) -> Vec<(EntityId, Point)> {
    world
        .components::<Presence>()
        .filter_map(|(person, presence)| {
            let location = presence.location();
            let local = location.local()?;
            (location.place() == place)
                .then(|| (person, Point::new(local.x().value(), local.y().value())))
        })
        .collect()
}

/// An entity as a message names it: its authoring key, which a world author recognizes.
pub(crate) fn name(world: &WorldRead<'_>, entity: EntityId) -> String {
    world.entity(entity).map_or_else(
        || format!("{entity:?}"),
        |record| record.key().as_str().to_owned(),
    )
}

impl PerceptionProvider for BodiesSystem {
    /// Discloses a place's [`PlaceShape`] — its floor and solids — and a listing of the loose objects
    /// lying in it — each one's shape and position — to whoever perceives the place.
    ///
    /// Perception asks only about entities the observation already lists, and it lists only the
    /// observer's own place, so a person learns the walls and the objects of the room they stand in
    /// and of no other. It states where things are, never whether one may pass: that is decided on
    /// the server, when an arrival is resolved (`ENGINEERING_RULES.md` §8). A client builds its
    /// colliders from these numbers, so what it predicts is what the server resolves against
    /// (step-11 R-B4). A person, and a place without a shape, disclose nothing.
    fn discloses(
        &self,
        world: &WorldRead<'_>,
        _observer: EntityId,
        subject: EntityId,
    ) -> Vec<ComponentRecord<Value>> {
        let is_place = world
            .entity(subject)
            .is_some_and(|entity| entity.entity_type() == EntityType::Place);
        if !is_place {
            return Vec::new();
        }
        let mut records: Vec<ComponentRecord<Value>> = world
            .component::<PlaceShape>(subject)
            .map(|shape| ComponentRecord::new::<PlaceShape>(subject, codec::to_value(shape)))
            .into_iter()
            .collect();
        if let Some(listing) = objects::listing(world, subject) {
            records.push(ComponentRecord::new::<LooseObjects>(
                subject,
                codec::to_value(&listing),
            ));
        }
        records
    }
}
