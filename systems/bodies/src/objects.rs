//! The loose objects of a place as this pack reads them, the owner's check on a move, and what a
//! place discloses about them (step-11 SD-O2, SD-O6).

use mineworld_contracts::{EntityId, ItemId, LocalPosition, PlaceId, Rejection, RejectionCode};
use mineworld_kernel::WorldRead;
use serde::Serialize;

use crate::component::{BodyShape, LooseObjects, PlaceShape};
use crate::event::ObjectMoved;
use crate::footprint::{Footprint, Placed, flaw};
use crate::geometry::{Point, TOLERANCE};
use crate::system::{name, standing_in};

/// Why a move was refused at reduction: the object is not where the fact says it was, or where it
/// says it went breaks the invariant of the stored state. A stating defect, reported as the owner's
/// refusal (`ARC-26`).
const MOVE_REFUSED: RejectionCode = RejectionCode::from_static("bodies-object-move");

/// Every object lying in `place`, in `ItemId` order, with its shape: the canonical order a scene
/// inserts objects in (step-11 DC-2). Empty for a place with none, or a world without this pack.
pub(crate) fn lying_in(world: &WorldRead<'_>, place: PlaceId) -> Vec<(ItemId, Placed)> {
    let Some(row) = world.component::<LooseObjects>(place.entity_id()) else {
        return Vec::new();
    };
    row.objects()
        .iter()
        .filter_map(|lying| {
            let shape = *world.component::<BodyShape>(lying.object().entity_id())?;
            Some((lying.object(), Placed::of(lying, shape)))
        })
        .collect()
}

/// The place an object lies in, and how it lies: the one place whose row lists it.
pub(crate) fn where_lies(world: &WorldRead<'_>, object: ItemId) -> Option<(PlaceId, Placed)> {
    let shape = *world.component::<BodyShape>(object.entity_id())?;
    world.components::<LooseObjects>().find_map(|(place, row)| {
        let lying = row.of(object)?;
        let place = PlaceId::new(place, mineworld_contracts::EntityType::Place).ok()?;
        Some((place, Placed::of(&lying, shape)))
    })
}

/// `object-moved`'s reduction, the owner's check (step-11 SD-O6): the object lies in `place` at
/// `from`, and `to` keeps SD-O2's invariant against the state being reduced into — the floor, the
/// solids, the other objects as they lie now and the people as they stand now. Returns the row with
/// the object moved, for the caller to write.
pub(crate) fn moved(world: &WorldRead<'_>, moved: &ObjectMoved) -> Result<LooseObjects, Rejection> {
    let (object, place) = (moved.object(), moved.place());
    let refuse = |detail: String| Rejection::System {
        code: MOVE_REFUSED,
        detail: Some(detail),
    };
    let thing = name(world, object.entity_id());
    let row = world
        .component::<LooseObjects>(place.entity_id())
        .ok_or_else(|| refuse(format!("{thing}: its place has no loose objects")))?;
    let lying = row.of(object).ok_or_else(|| {
        refuse(format!(
            "{thing} does not lie in {}",
            name(world, place.entity_id())
        ))
    })?;
    if lying.at() != moved.from() {
        return Err(refuse(format!(
            "{thing} lies at {:?}, not at {:?}",
            lying.at(),
            moved.from()
        )));
    }
    let shape = world
        .component::<PlaceShape>(place.entity_id())
        .ok_or_else(|| refuse(format!("{thing}'s place has no shape")))?;
    let room = shape.room();
    let others: Vec<Footprint> = lying_in(world, place)
        .into_iter()
        .filter(|(other, _)| *other != object)
        .map(|(_, other)| other.footprint())
        .collect();
    let people: Vec<Point> = standing_in(world, place)
        .into_iter()
        .map(|(_, at)| at)
        .collect();
    let to = moved.to();
    let shape_of = *world
        .component::<BodyShape>(object.entity_id())
        .ok_or_else(|| refuse(format!("{thing} has no shape")))?;
    let end = Placed {
        shape: shape_of,
        centre: Point::new(to.x().value(), to.y().value()),
        z: to.z().value(),
    };
    if let Some(found) = flaw(&room, &end, &others, &people, TOLERANCE.value()) {
        return Err(refuse(format!(
            "{thing} cannot end at {to:?}: {found:?} (step-11 SD-O2)"
        )));
    }
    Ok(row.moved(object, to))
}

/// One object of a place's listing: which, its shape, where it lies (step-11 SD-O2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Listed {
    object: ItemId,
    shape: BodyShape,
    at: LocalPosition,
}

/// The listing a place discloses: every object lying in it, its shape joined from [`BodyShape`] at
/// disclosure, in `ItemId` order. Built from current state, never stored (the `Shop` precedent,
/// `ARC-38`).
pub(crate) fn listing(world: &WorldRead<'_>, place: EntityId) -> Option<Vec<Listed>> {
    let row = world.component::<LooseObjects>(place)?;
    Some(
        row.objects()
            .iter()
            .filter_map(|lying| {
                let shape = *world.component::<BodyShape>(lying.object().entity_id())?;
                Some(Listed {
                    object: lying.object(),
                    shape,
                    at: lying.at(),
                })
            })
            .collect(),
    )
}
