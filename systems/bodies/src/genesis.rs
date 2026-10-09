//! What this pack checks before it writes anything a world begins with (step-11 SD-B4, SD-O5).
//!
//! ```text
//! place-shaped   the people authored into the place fit it                              (12b)
//!                and so does a person standing on each of its doorway points            (12d)
//! body-formed    a living Item with no shape yet; then `object-placed` is stated        (gen 1)
//! object-placed  the object fits its place: shaped, not a held kind, ≤ 32, inside the   (gen 2)
//!                floor, out of the solids, clear of the other objects and the people,
//!                and the place's capacity still holds the population with its objects
//! ```
//!
//! Genesis states every location, then the items' sections, then the places', in one generation
//! (step-11 F-O3): an item's `body-formed` is reduced before the `place-shaped` of the place it names.
//! So the object's own checks run on `object-placed`, which this pack states in reaction to
//! `body-formed` and the kernel reduces in the next generation, when every place's shape and every
//! person is in place — the kernel's own ordering (`BD-7`), not a new mechanism.

use mineworld_contracts::{
    EntityId, EntityType, LifecycleState, LocalPosition, Millimetres, PlaceId, Rejection,
    RejectionCode,
};
use mineworld_kernel::{Emission, WorldRead};

use crate::component::{BodyShape, LooseObjects, Lying, PlaceShape};
use crate::event::{BodyFormed, ObjectPlaced, object_placed};
use crate::footprint::{Flaw, Placed, blocks, flaw};
use crate::geometry::{CLEARANCE, GAP, OBJECTS_MAX, PERSON_RADIUS, Point, Room, closest_pair};
use crate::objects::lying_in;
use crate::system::{name, standing_in};
use mineworld_movement::Passages;

/// Why a place's shape was refused at genesis: the authored people do not fit it.
const OVERLAP: RejectionCode = RejectionCode::from_static("bodies-overlap");
const OUTSIDE: RejectionCode = RejectionCode::from_static("bodies-outside");
const IN_SOLID: RejectionCode = RejectionCode::from_static("bodies-in-solid");
const CAPACITY: RejectionCode = RejectionCode::from_static("bodies-capacity");
/// A doorway point of the place where a person does not fit (step-11 SD-D5).
const DOORWAY: RejectionCode = RejectionCode::from_static("bodies-doorway");

/// Why an object was refused at genesis (step-11 SD-O5), in the order the checks run.
const UNSHAPED: RejectionCode = RejectionCode::from_static("bodies-unshaped");
const HELD_KIND: RejectionCode = RejectionCode::from_static("bodies-held-kind");
const OBJECTS_FULL: RejectionCode = RejectionCode::from_static("bodies-objects-max");
const OBJECT_OUTSIDE: RejectionCode = RejectionCode::from_static("bodies-object-outside");
const OBJECT_IN_SOLID: RejectionCode = RejectionCode::from_static("bodies-object-in-solid");
const OBJECT_OVERLAP: RejectionCode = RejectionCode::from_static("bodies-object-overlap");
const OBJECT_ON_PERSON: RejectionCode = RejectionCode::from_static("bodies-object-on-person");

fn refuse(code: RejectionCode, detail: String) -> Rejection {
    Rejection::System {
        code,
        detail: Some(detail),
    }
}

/// The people of the world, living: the population every place's capacity must hold.
fn population(world: &WorldRead<'_>) -> usize {
    world
        .entities()
        .filter(|entity| {
            entity.entity_type() == EntityType::Person
                && entity.lifecycle() != LifecycleState::Destroyed
        })
        .count()
}

/// Whether the world as it stands fits `shape` in `place`: SD-B4's four checks, in order —
/// capacity, outside, in a solid, overlapping — then SD-D5's doorway points.
pub(crate) fn fits(
    world: &WorldRead<'_>,
    place: PlaceId,
    shape: &PlaceShape,
) -> Result<(), Rejection> {
    let is_place = world.entity(place.entity_id()).is_some_and(|entity| {
        entity.entity_type() == EntityType::Place && entity.lifecycle() != LifecycleState::Destroyed
    });
    if !is_place {
        return Err(Rejection::PreconditionFailed);
    }
    let room = shape.room();
    let r = PERSON_RADIUS.value();
    let named = |entity: EntityId| name(world, entity);

    let population = population(world);
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
    doorways_fit(world, place, &room)
}

/// SD-D5 (step-11 §19; `ARC-39` note 5): every doorway point of `place` lies where a person fits —
/// inside the floor shrunk by a radius and the gap, and at least that far from every solid. Otherwise
/// every crossing would be shifted silently by entry placement (E3) rather than land on the doorway
/// (F-B7). Read from `movement`'s [`Passages`], the one item of that crate this pack names (QD-5).
fn doorways_fit(world: &WorldRead<'_>, place: PlaceId, room: &Room) -> Result<(), Rejection> {
    let keep = PERSON_RADIUS.value() + GAP.value();
    for (other, at) in doorways(world, place) {
        let refused = |what: String| {
            refuse(
                DOORWAY,
                format!(
                    "the doorway between {} and {} lies at ({}, {}) in {}, {what}; a doorway point \
                     keeps {keep} mm from the floor's edge and from every solid",
                    name(world, place.entity_id()),
                    name(world, other),
                    at.x,
                    at.y,
                    name(world, place.entity_id()),
                ),
            )
        };
        let floor = &room.floor;
        let edge = i64::from(
            (at.x - floor.min.x)
                .min(floor.max.x - at.x)
                .min(at.y - floor.min.y)
                .min(floor.max.y - at.y),
        );
        if edge < 0 {
            return Err(refused(format!("outside its floor by {} mm", -edge)));
        }
        if edge < i64::from(keep) {
            return Err(refused(format!("{edge} mm from its floor's edge")));
        }
        if let Some(distance) = room.solid_within(at, keep) {
            return Err(refused(format!("{distance} mm from a solid")));
        }
    }
    Ok(())
}

/// Every doorway point in `place`'s frame, with the place it leads to: the `here` of each passage out
/// of it, and the `there` of each passage into it. A side the world does not model (a semantic
/// passage) has no point and nothing to check.
fn doorways(world: &WorldRead<'_>, place: PlaceId) -> Vec<(EntityId, Point)> {
    let point = |at: mineworld_contracts::LocalPosition| Point::new(at.x().value(), at.y().value());
    let mut points: Vec<(EntityId, Point)> = world
        .component::<Passages>(place.entity_id())
        .into_iter()
        .flat_map(Passages::iter)
        .filter_map(|passage| Some((passage.to().entity_id(), point(passage.here()?))))
        .collect();
    for (from, passages) in world.components::<Passages>() {
        if let Some(there) = passages.to(place).and_then(|passage| passage.there()) {
            points.push((from, point(there)));
        }
    }
    points
}

/// `body-formed`: a living Item that has no shape yet takes this one (the caller writes it), and the
/// object is then stated placed where it lies, its centre at its half-height above the floor.
pub(crate) fn formed(world: &WorldRead<'_>, formed: &BodyFormed) -> Result<Emission, Rejection> {
    let object = formed.object().entity_id();
    let living_item = world.entity(object).is_some_and(|entity| {
        entity.entity_type() == EntityType::Item && entity.lifecycle() != LifecycleState::Destroyed
    });
    if !living_item || world.component::<BodyShape>(object).is_some() {
        return Err(Rejection::PreconditionFailed);
    }
    let lies = formed.lies();
    let ground = lies.local().ok_or(Rejection::PreconditionFailed)?;
    let at = LocalPosition::new(
        ground.x(),
        ground.y(),
        Millimetres::new(formed.shape().half_height()),
    );
    Ok(object_placed(formed.object(), lies.place(), at))
}

/// `object-placed`: SD-O5's checks, in order, against the world as generation 2 sees it — every
/// place shaped, every person placed, every earlier object lying. Returns the place's row with the
/// object added, for the caller to write.
pub(crate) fn placed(
    world: &WorldRead<'_>,
    placed: &ObjectPlaced,
) -> Result<LooseObjects, Rejection> {
    let (object, place) = (placed.object(), placed.place());
    let named = |entity: EntityId| name(world, entity);
    let (thing, room_name) = (named(object.entity_id()), named(place.entity_id()));
    let shape = *world
        .component::<BodyShape>(object.entity_id())
        .ok_or(Rejection::PreconditionFailed)?;
    let Some(place_shape) = world.component::<PlaceShape>(place.entity_id()) else {
        return Err(refuse(
            UNSHAPED,
            format!(
                "{thing} lies in {room_name}, which has no `body:` section: no floor to lie on"
            ),
        ));
    };
    // QB-3 (ARC-36 note): an object is never a declared kind, so it is never held. The one read this
    // pack makes of the `item` pack (ARC-39 note 2, point 4).
    if mineworld_item::is_declared(world, object) {
        return Err(refuse(
            HELD_KIND,
            format!(
                "{thing} carries both `body:` and `item:`: an object is one physical thing and \
                 never a kind anybody holds (ARC-36 note)"
            ),
        ));
    }
    let row = world
        .component::<LooseObjects>(place.entity_id())
        .cloned()
        .unwrap_or_default();
    if row.objects().len() >= OBJECTS_MAX {
        return Err(refuse(
            OBJECTS_FULL,
            format!(
                "{room_name} already holds {OBJECTS_MAX} loose objects; {thing} would be one more"
            ),
        ));
    }
    let room = place_shape.room();
    let at = placed.at();
    let me = Placed {
        shape,
        centre: Point::new(at.x().value(), at.y().value()),
        z: at.z().value(),
    };
    let others = lying_in(world, place);
    let footprints: Vec<_> = others.iter().map(|(_, other)| other.footprint()).collect();
    let people = standing_in(world, place);
    let points: Vec<Point> = people.iter().map(|(_, at)| *at).collect();
    let (x, y) = (me.centre.x, me.centre.y);
    match flaw(&room, &me, &footprints, &points, 0) {
        None => {}
        Some(Flaw::Outside | Flaw::Floating) => {
            return Err(refuse(
                OBJECT_OUTSIDE,
                format!("{thing} at ({x}, {y}) in {room_name} reaches beyond its floor"),
            ));
        }
        Some(Flaw::InSolid(index)) => {
            return Err(refuse(
                OBJECT_IN_SOLID,
                format!("{thing} at ({x}, {y}) in {room_name} meets solid {index}"),
            ));
        }
        Some(Flaw::Overlap(index)) => {
            let (other, lying) = &others[index];
            return Err(refuse(
                OBJECT_OVERLAP,
                format!(
                    "{thing} at ({x}, {y}) overlaps {} at ({}, {}) in {room_name}",
                    named(other.entity_id()),
                    lying.centre.x,
                    lying.centre.y
                ),
            ));
        }
        Some(Flaw::OnPerson(index)) => {
            let (person, stands) = people[index];
            return Err(refuse(
                OBJECT_ON_PERSON,
                format!(
                    "{thing} at ({x}, {y}) lies under {}, who stands at ({}, {}) in {room_name}; a \
                     person keeps {} mm from an object",
                    named(person),
                    stands.x,
                    stands.y,
                    PERSON_RADIUS.value()
                ),
            ));
        }
    }
    let population = population(world);
    let covered: usize = others
        .iter()
        .map(|(_, other)| blocks(other.shape))
        .sum::<usize>()
        + blocks(shape);
    let needed = 4 * population.saturating_sub(1) + 1 + covered;
    let capacity = room.capacity();
    if capacity < needed {
        return Err(refuse(
            CAPACITY,
            format!(
                "{room_name}'s floor has room for a person at {capacity} points of the 650 mm grid; \
                 a world of {population} people with {} objects there needs {needed}",
                others.len() + 1
            ),
        ));
    }
    row.with(Lying::new(object, at))
        .map_err(|detail| refuse(OBJECTS_FULL, detail))
}
