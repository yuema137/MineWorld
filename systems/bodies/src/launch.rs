//! `kick` and `throw`: whether one may happen, and what it becomes (step-11 SD-O11, SD-O13, SD-O14;
//! QB-6: resolved at the instant).
//!
//! ```text
//! validate   the payload decodes (else PreconditionFailed)
//!            the actor is a living Person with a position              PreconditionFailed
//!            the object lies somewhere: an Item with a shape in a place  TargetUnavailable
//!            kick: it lies on the floor                                 TargetUnavailable
//!            the declared requirement against its ground point           TooFarAway
//!            throw, aimed: the point inside the floor shrunk by the object's half-size, and within
//!            6 000 mm of the object's ground point                       PreconditionFailed
//! resolve    the flight (rapier.rs), the landing ladder (flight.rs), one `object-moved`
//! ```

use mineworld_contracts::{
    ActionIntent, EntityId, EntityType, LifecycleState, LocalPosition, Location, Millimetres,
    PersonId, PlaceId, Rejection,
};
use mineworld_kernel::{Emission, KernelError, WorldRead};
use mineworld_presence::Presence;
use serde::de::DeserializeOwned;

use crate::action::{Kick, Throw, kick_requirement, throw_requirement};
use crate::component::PlaceShape;
use crate::event::{How, Movement, ObjectMoved, object_moved};
use crate::flight::{default_aim, kick_velocity, land, rest_height, throw_velocity};
use crate::footprint::{Footprint, Placed, Resting};
use crate::geometry::{KICK_STEPS, Point, THROW_RANGE_MAX, THROW_STEPS, TOLERANCE, distance2};
use crate::objects::{lying_in, where_lies};
use crate::rapier::{At, Launch, fly};
use crate::system::{refused, standing_in};
use mineworld_contracts::{Action, Event, ItemId};

/// The payload of an `A`, or `PreconditionFailed`, as every pack's codec refuses a malformed one.
pub(crate) fn read<A: Action + DeserializeOwned>(intent: &ActionIntent) -> Result<A, Rejection> {
    let payload = intent
        .payload()
        .payload_for::<A>()
        .map_err(|_| Rejection::PreconditionFailed)?;
    serde_json::from_slice(payload).map_err(|_| Rejection::PreconditionFailed)
}

/// Where a living Person stands, with a position; `PreconditionFailed` otherwise.
pub(crate) fn standing(world: &WorldRead<'_>, person: EntityId) -> Result<Location, Rejection> {
    let living = world.entity(person).is_some_and(|record| {
        record.entity_type() == EntityType::Person
            && record.lifecycle() != LifecycleState::Destroyed
    });
    let location = world
        .component::<Presence>(person)
        .map(Presence::location)
        .filter(|location| location.local().is_some());
    match (living, location) {
        (true, Some(location)) => Ok(location),
        _ => Err(Rejection::PreconditionFailed),
    }
}

/// An object as a kick or a throw finds it: its place, how it lies, and its ground point there.
struct Found {
    place: PlaceId,
    placed: Placed,
    ground: Location,
}

/// The object, lying somewhere — `TargetUnavailable` for an Item that does not lie anywhere.
fn found(world: &WorldRead<'_>, object: ItemId) -> Result<Found, Rejection> {
    let (place, placed) = where_lies(world, object).ok_or(Rejection::TargetUnavailable)?;
    let ground = Location::in_place(place).with_local(LocalPosition::new(
        Millimetres::new(placed.centre.x),
        Millimetres::new(placed.centre.y),
        Millimetres::ZERO,
    ));
    Ok(Found {
        place,
        placed,
        ground,
    })
}

/// Whether `object`'s ground point is within reach of somebody standing at `here`, by the declared
/// requirement — the one answer `validate` and the offers share.
pub(crate) fn in_reach(world: &WorldRead<'_>, here: &Location, object: ItemId, kick: bool) -> bool {
    found(world, object).is_ok_and(|found| {
        let requirement = if kick {
            kick_requirement()
        } else {
            throw_requirement()
        };
        requirement
            .evaluate(here, Some(&found.ground), true)
            .is_ok()
    })
}

/// SD-O11's `validate` for a kick.
pub(crate) fn validate_kick(world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
    let kick: Kick = read(intent)?;
    let here = standing(world, intent.actor())?;
    let found = found(world, kick.object())?;
    let room = shaped(world, found.place)?;
    let on_floor = found.placed.resting(&room, TOLERANCE.value()) == Some(Resting::Floor);
    kick_requirement().evaluate(&here, Some(&found.ground), on_floor)
}

/// SD-O13's `validate` for a throw.
pub(crate) fn validate_throw(
    world: &WorldRead<'_>,
    intent: &ActionIntent,
) -> Result<(), Rejection> {
    let throw: Throw = read(intent)?;
    let here = standing(world, intent.actor())?;
    let found = found(world, throw.object())?;
    throw_requirement().evaluate(&here, Some(&found.ground), true)?;
    if let Some(toward) = throw.toward() {
        let room = shaped(world, found.place)?;
        let point = Point::new(toward.x().value(), toward.y().value());
        let fits = found.placed.footprint().at(point).inside(room.floor);
        let range = i64::from(THROW_RANGE_MAX.value());
        if !fits || distance2(point, found.placed.centre) > range * range {
            return Err(Rejection::PreconditionFailed);
        }
    }
    Ok(())
}

fn shaped(world: &WorldRead<'_>, place: PlaceId) -> Result<crate::geometry::Room, Rejection> {
    world
        .component::<PlaceShape>(place.entity_id())
        .map(PlaceShape::room)
        .ok_or(Rejection::TargetUnavailable)
}

/// A kick, resolved at the instant: the flight, the landing, one `object-moved { kicked }`.
pub(crate) fn resolve_kick(
    world: &WorldRead<'_>,
    intent: &ActionIntent,
) -> Result<Vec<Emission>, KernelError> {
    let refuse = |reason| refused(ObjectMoved::EVENT_TYPE, reason);
    validate_kick(world, intent).map_err(refuse)?;
    let kick: Kick = read(intent).map_err(refuse)?;
    let here = standing(world, intent.actor()).map_err(refuse)?;
    let found = found(world, kick.object()).map_err(refuse)?;
    let kicker = ground(&here);
    let velocity = kick_velocity(kicker, found.placed.centre);
    flown(
        world,
        intent,
        kick.object(),
        &found,
        velocity,
        KICK_STEPS,
        How::Kicked,
    )
}

/// A throw, resolved at the instant: aimed at its point (or the default one), the flight, the
/// landing, one `object-moved { thrown }`.
pub(crate) fn resolve_throw(
    world: &WorldRead<'_>,
    intent: &ActionIntent,
) -> Result<Vec<Emission>, KernelError> {
    let refuse = |reason| refused(ObjectMoved::EVENT_TYPE, reason);
    validate_throw(world, intent).map_err(refuse)?;
    let throw: Throw = read(intent).map_err(refuse)?;
    let here = standing(world, intent.actor()).map_err(refuse)?;
    let found = found(world, throw.object()).map_err(refuse)?;
    let room = shaped(world, found.place).map_err(refuse)?;
    let point = match throw.toward() {
        Some(toward) => Point::new(toward.x().value(), toward.y().value()),
        None => default_aim(&room, &found.placed, ground(&here)),
    };
    let z_rest = rest_height(&room, &found.placed, point);
    let velocity = throw_velocity(&found.placed, point, z_rest);
    flown(
        world,
        intent,
        throw.object(),
        &found,
        velocity,
        THROW_STEPS,
        How::Thrown,
    )
}

fn ground(location: &Location) -> Point {
    let local = location.local().expect("a position, checked by validate");
    Point::new(local.x().value(), local.y().value())
}

/// The flight of `object` from where it lies, its landing, and the fact.
fn flown(
    world: &WorldRead<'_>,
    intent: &ActionIntent,
    object: ItemId,
    found: &Found,
    velocity: (i32, i32, i32),
    steps: u32,
    how: How,
) -> Result<Vec<Emission>, KernelError> {
    let refuse = |reason| refused(ObjectMoved::EVENT_TYPE, reason);
    let room = shaped(world, found.place).map_err(refuse)?;
    let people: Vec<Point> = standing_in(world, found.place)
        .into_iter()
        .map(|(_, at)| at)
        .collect();
    let others: Vec<Placed> = lying_in(world, found.place)
        .into_iter()
        .filter(|(other, _)| *other != object)
        .map(|(_, placed)| placed)
        .collect();
    let footprints: Vec<Footprint> = others.iter().map(Placed::footprint).collect();
    let flight = fly(&Launch {
        room: &room,
        people: &people,
        others: &others,
        flying: found.placed,
        velocity,
        steps,
    });
    let end = land(&room, &found.placed, flight.end, &footprints, &people);
    let by = PersonId::new(intent.actor(), EntityType::Person)
        .map_err(|_| refuse(Rejection::PreconditionFailed))?;
    Ok(vec![object_moved(Movement {
        object,
        place: found.place,
        from: local((found.placed.centre, found.placed.z)),
        to: local(end),
        how,
        by,
        path: flight.path.into_iter().map(local).collect(),
    })])
}

fn local((point, z): At) -> LocalPosition {
    LocalPosition::new(
        Millimetres::new(point.x),
        Millimetres::new(point.y),
        Millimetres::new(z),
    )
}
