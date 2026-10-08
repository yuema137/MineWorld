//! The installable system: what it declares, the one fact it reduces and the checks it reduces it
//! under, and what it discloses.

use mineworld_contracts::{
    ComponentRecord, EntityId, EntityType, Event, EventEnvelope, LifecycleState, PlaceId,
    Rejection, RejectionCode, SystemId,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    WorldRead, WorldView,
};
use mineworld_presence::{PerceptionProvider, Presence, PresenceSystem, require_registered};
use mineworld_sdk::SystemPack;
use serde_json::Value;

use crate::codec;
use crate::component::PlaceShape;
use crate::event::PlaceShaped;
use crate::geometry::{CLEARANCE, PERSON_RADIUS, Point, closest_pair};

/// Bodies: places with walls and furniture, and people who neither pass through them nor through
/// each other.
///
/// A unit struct, like every System Pack: its state is the [`PlaceShape`]s it owns, held in the world,
/// and the resolver it registers keeps nothing between calls (`ARC-39`).
#[derive(Default)]
pub struct BodiesSystem;

impl SystemIdentity for BodiesSystem {
    const ID: SystemId = SystemId::from_static("bodies");
}

/// What the build needs to know about this pack beyond [`System`] (`DECISIONS.md` `ARC-33`): the
/// `body:` section of a place file, which its `AuthoredSection` impl (`src/section.rs`) describes. No
/// biographical fact: a place's shape is not an event in anybody's life.
impl SystemPack for BodiesSystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    mineworld_sdk::owns_section!();
}

/// Why a place's shape was refused at genesis: the authored people do not fit it.
const OVERLAP: RejectionCode = RejectionCode::from_static("bodies-overlap");
const OUTSIDE: RejectionCode = RejectionCode::from_static("bodies-outside");
const IN_SOLID: RejectionCode = RejectionCode::from_static("bodies-in-solid");
const CAPACITY: RejectionCode = RejectionCode::from_static("bodies-capacity");

impl System for BodiesSystem {
    /// Version 1. A Rapier upgrade changes results, so it bumps this version with it (`DEP-13`;
    /// `tests/rapier_pin.rs` holds the two together).
    const VERSION: SystemVersion = SystemVersion::new(1);

    /// Depends on presence (where people are, and the seam this pack resolves through); owns the
    /// shapes of places; states and reduces `place-shaped`. No action, and no fact in another pack's
    /// vocabulary: an arrival is stated by whoever moves the person, and this pack only answers what it
    /// achieves.
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([PresenceSystem::ID])
            .owning::<PlaceShape>()
            .emitting::<PlaceShaped>()
            .subscribing_to::<PlaceShaped>()
    }

    /// Refuses to join a world whose host never registered this pack's resolver — before anything
    /// else, as `ARC-39` item 7 requires — and then declares its one table.
    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        require_registered(&Self::ID);
        tables.component::<PlaceShape>()
    }

    /// Reduces `place-shaped` into the place's [`PlaceShape`] — the only write this pack makes — after
    /// checking that the people authored into the place fit it.
    ///
    /// Genesis states every location before any section, and reduces its facts in order, so by the
    /// time this fact is reduced every authored person stands where the world placed them (step-11
    /// F-B4). The checks run on integers and refuse with [`KernelError::FactRefusedByOwner`], naming
    /// the people, the place and the numbers; nothing is written unless all four pass:
    ///
    /// ```text
    /// bodies-capacity  the floor cannot hold the world's population: fewer than 4 × (people − 1) + 1
    ///                  points of the 650 mm grid where a person fits (what guarantees an arrival
    ///                  always finds room, step-11 SD-B4, SD-B8)
    /// bodies-outside   a person's centre lies outside the floor shrunk by 300 mm
    /// bodies-in-solid  a person's centre lies within 300 mm of a solid
    /// bodies-overlap   two people stand closer than 595 mm
    /// ```
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if *event.event_type() != PlaceShaped::EVENT_TYPE {
            return Ok(Vec::new());
        }
        let shaped: PlaceShaped = codec::event_payload(event.payload())?;
        let place = shaped.place();
        fits(&world.read(), place, shaped.shape()).map_err(|reason| {
            KernelError::FactRefusedByOwner {
                system: Self::ID,
                event_type: PlaceShaped::EVENT_TYPE,
                reason,
            }
        })?;
        world.insert(place.entity_id(), shaped.shape().clone())?;
        Ok(Vec::new())
    }
}

/// Whether the world as it stands fits `shape` in `place`: SD-B4's four checks, in the order above.
fn fits(world: &WorldRead<'_>, place: PlaceId, shape: &PlaceShape) -> Result<(), Rejection> {
    let is_place = world.entity(place.entity_id()).is_some_and(|entity| {
        entity.entity_type() == EntityType::Place && entity.lifecycle() != LifecycleState::Destroyed
    });
    if !is_place {
        return Err(Rejection::PreconditionFailed);
    }
    let room = shape.room();
    let r = PERSON_RADIUS.value();
    let named = |entity: EntityId| name(world, entity);
    let refuse = |code, detail: String| Rejection::System {
        code,
        detail: Some(detail),
    };

    let population = world
        .entities()
        .filter(|entity| {
            entity.entity_type() == EntityType::Person
                && entity.lifecycle() != LifecycleState::Destroyed
        })
        .count();
    let needed = 4 * population.saturating_sub(1) + 1;
    let capacity = room.capacity();
    if capacity < needed {
        return Err(refuse(
            CAPACITY,
            format!(
                "{}'s floor has room for a person at {capacity} points of the 650 mm grid; a world \
                 of {population} people needs {needed}",
                named(place.entity_id())
            ),
        ));
    }

    let people = standing_in(world, place);
    for (person, at) in &people {
        if !room.floor.holds(*at, r) {
            return Err(refuse(
                OUTSIDE,
                format!(
                    "{} stands at ({}, {}) in {}, outside its floor shrunk by {r} mm",
                    named(*person),
                    at.x,
                    at.y,
                    named(place.entity_id())
                ),
            ));
        }
        if let Some(distance) = room.solid_within(*at, r) {
            return Err(refuse(
                IN_SOLID,
                format!(
                    "{} stands {distance} mm from a solid in {}; a person keeps {r} mm from every \
                     solid",
                    named(*person),
                    named(place.entity_id())
                ),
            ));
        }
    }
    let points: Vec<Point> = people.iter().map(|(_, at)| *at).collect();
    if let Some((a, b, distance2)) = closest_pair(&points)
        && distance2 < i64::from(CLEARANCE.value()) * i64::from(CLEARANCE.value())
    {
        return Err(refuse(
            OVERLAP,
            format!(
                "{} and {} stand {} mm apart in {}; people stand at least {} mm apart",
                named(people[a].0),
                named(people[b].0),
                distance2.isqrt(),
                named(place.entity_id()),
                CLEARANCE.value()
            ),
        ));
    }
    Ok(())
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
    /// Discloses a place's [`PlaceShape`] — its floor and solids — to whoever perceives the place.
    ///
    /// Perception asks only about entities the observation already lists, and it lists only the
    /// observer's own place, so a person learns the walls of the room they stand in and of no other.
    /// It states where the walls are, never whether one may pass: that is decided on the server, when
    /// an arrival is resolved (`ENGINEERING_RULES.md` §8). A client builds its colliders from these
    /// numbers, so the walls it predicts are the walls the server resolves against (step-11 R-B4). A
    /// person, and a place without a shape, disclose nothing.
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
        world
            .component::<PlaceShape>(subject)
            .map(|shape| {
                vec![ComponentRecord::new::<PlaceShape>(
                    subject,
                    codec::to_value(shape),
                )]
            })
            .unwrap_or_default()
    }
}
