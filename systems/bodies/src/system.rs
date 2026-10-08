//! The installable system: what it declares, the facts it reduces and the checks it reduces them
//! under, and what it discloses.

use mineworld_contracts::{
    Action, ActionIntent, ComponentRecord, EntityId, EntityType, Event, EventEnvelope, EventTypeId,
    LocalPosition, Millimetres, PlaceId, Rejection, RejectionCode, SystemId,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    WorldRead, WorldView,
};
use mineworld_presence::{
    Arrived, PerceptionProvider, Presence, PresenceSystem, StoppedShort, require_registered,
};
use mineworld_sdk::SystemPack;
use serde_json::Value;

use crate::action::{Kick, Shove, Throw};
use crate::codec;
use crate::component::{BodyShape, LooseObjects, PlaceShape};
use crate::event::{
    BodyFormed, How, Movement, ObjectMoved, ObjectPlaced, PersonShoved, PlaceShaped, object_moved,
};
use crate::geometry::Point;
use crate::push::Lay;
use crate::{genesis, launch, objects, offer, shove};

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
    /// shapes of places and objects and where the objects lie; provides `kick`, `throw` and `shove`;
    /// states and reduces its own facts; and, for `shove` only, states presence's `arrived` and
    /// `stopped-short` through `arrivals()` (`ARC-26`; step-11 SD-O7: with `shove` it is a mover). It
    /// hears presence's `arrived`, its own included (step-11 F-R7).
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([PresenceSystem::ID])
            .owning::<PlaceShape>()
            .owning::<BodyShape>()
            .owning::<LooseObjects>()
            .providing::<Kick>()
            .providing::<Throw>()
            .providing::<Shove>()
            .emitting::<PlaceShaped>()
            .emitting::<BodyFormed>()
            .emitting::<ObjectPlaced>()
            .emitting::<ObjectMoved>()
            .emitting::<PersonShoved>()
            .emitting::<Arrived>()
            .emitting::<StoppedShort>()
            .subscribing_to::<PlaceShaped>()
            .subscribing_to::<BodyFormed>()
            .subscribing_to::<ObjectPlaced>()
            .subscribing_to::<ObjectMoved>()
            .subscribing_to::<Arrived>()
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
    /// arrived        (presence's)       → object-moved { pushed } for every object the person's
    ///                                   disc now overlaps (SD-O8); nothing is written
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
        } else if *kind == Arrived::EVENT_TYPE {
            let arrived: Arrived = codec::event_payload(event.payload())?;
            return pushed_by(&world.read(), &arrived);
        }
        Ok(Vec::new())
    }

    /// Whether this `kick`, `throw` or `shove` may happen (step-11 SD-O11, SD-O13, SD-O15;
    /// `launch.rs`, `shove.rs`).
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        let action = intent.action_type();
        if *action == Kick::ACTION_TYPE {
            launch::validate_kick(world, intent)
        } else if *action == Throw::ACTION_TYPE {
            launch::validate_throw(world, intent)
        } else if *action == Shove::ACTION_TYPE {
            shove::validate(world, intent)
        } else {
            Err(Rejection::NoSupportedInteraction)
        }
    }

    /// A `kick` or `throw`, resolved at the instant (QB-6): one `object-moved`. A `shove`:
    /// `person-shoved`, then presence's facts for the shoved person's arrival.
    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let action = intent.action_type();
        let read = world.read();
        if *action == Kick::ACTION_TYPE {
            launch::resolve_kick(&read, intent)
        } else if *action == Throw::ACTION_TYPE {
            launch::resolve_throw(&read, intent)
        } else if *action == Shove::ACTION_TYPE {
            shove::resolve(&read, intent)
        } else {
            Err(refused(
                ObjectMoved::EVENT_TYPE,
                Rejection::NoSupportedInteraction,
            ))
        }
    }
}

/// The reaction to a recorded `arrived` (step-11 SD-O8): the person pushes every loose object their
/// disc now overlaps, each stated as `object-moved { how: pushed }`, caused by the arrival and so by
/// the request (`AC-9`). Inert for an arrival without a position, a place without a shape or without
/// objects — which includes every genesis placement, since no object lies anywhere before genesis's
/// second generation.
///
/// The resolver predicted exactly these pushes from the same inputs (SD-O10), so for a resolved
/// arrival none jams. A jam here means an arrival escaped resolution — `ARC-39` item 7's guard, which
/// for objects can be built at the reaction — and the dispatch fails with `bodies-object-jammed`.
fn pushed_by(world: &WorldRead<'_>, arrived: &Arrived) -> Result<Vec<Emission>, KernelError> {
    let location = arrived.location();
    let Some(local) = location.local() else {
        return Ok(Vec::new());
    };
    let place = location.place();
    let Some(shape) = world.component::<PlaceShape>(place.entity_id()) else {
        return Ok(Vec::new());
    };
    let lying = objects::lying_in(world, place);
    if lying.is_empty() {
        return Ok(Vec::new());
    }
    let room = shape.room();
    let person = arrived.person();
    let at = Point::new(local.x().value(), local.y().value());
    let pushes = Lay::new(&room, &lying).pushes(at).map_err(|jam| {
        refused(
            ObjectMoved::EVENT_TYPE,
            Rejection::System {
                code: JAMMED,
                detail: Some(format!(
                    "{} cannot be pushed out of {}'s way at ({}, {}) in {}: the arrival was not \
                     resolved with objects solid (DECISIONS.md ARC-39 note 2)",
                    name(world, jam.object.entity_id()),
                    name(world, person.entity_id()),
                    at.x,
                    at.y,
                    name(world, place.entity_id()),
                )),
            },
        )
    })?;
    Ok(pushes
        .into_iter()
        .map(|push| {
            let from = lying[push.index].1;
            object_moved(Movement {
                object: push.object,
                place,
                from: LocalPosition::new(
                    Millimetres::new(from.centre.x),
                    Millimetres::new(from.centre.y),
                    Millimetres::new(from.z),
                ),
                to: LocalPosition::new(
                    Millimetres::new(push.to.x),
                    Millimetres::new(push.to.y),
                    Millimetres::new(from.z),
                ),
                how: How::Pushed,
                by: person,
                path: Vec::new(),
            })
        })
        .collect())
}

/// Why a push was refused at the reaction: it jams, so the arrival escaped resolution.
const JAMMED: RejectionCode = RejectionCode::from_static("bodies-object-jammed");

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
    /// This pack's complete affordances (step-11 SD-O12; `offer.rs`).
    fn offers(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        target: Option<EntityId>,
    ) -> Vec<mineworld_presence::Offer> {
        offer::offers(world, observer, target)
    }

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
