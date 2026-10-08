//! What this pack offers an observer (step-11 SD-O12): complete affordances (`ARC-34`), so the
//! unchanged paced controller attempts them.
//!
//! ```text
//! target none     for each object lying in the observer's place whose ground point is within
//!                 800 mm, in ItemId order: kick { object } (on the floor only), then
//!                 throw { object, toward: null } — each at_place(the place), available.
//!                 Objects beyond reach are not offered (QO-7)
//! ```
//!
//! Nothing to an observer without a position, and nothing in a place without a shape.

use mineworld_contracts::{EntityId, SpatialRequirement};
use mineworld_kernel::WorldRead;
use mineworld_presence::Offer;

use crate::action::{Kick, Throw};
use crate::component::PlaceShape;
use crate::footprint::Resting;
use crate::geometry::TOLERANCE;
use crate::launch::{in_reach, standing};
use crate::objects::lying_in;

/// Every offer of this pack to `observer` about `target`, in a fixed order (`AC-12`).
pub(crate) fn offers(
    world: &WorldRead<'_>,
    observer: EntityId,
    target: Option<EntityId>,
) -> Vec<Offer> {
    let Ok(here) = standing(world, observer) else {
        return Vec::new();
    };
    let place = here.place();
    let Some(shape) = world.component::<PlaceShape>(place.entity_id()) else {
        return Vec::new();
    };
    if target.is_some() {
        return Vec::new();
    }
    let room = shape.room();
    let requirement = SpatialRequirement::at_place(place).requiring_target_available();
    let near: Vec<_> = lying_in(world, place)
        .into_iter()
        .filter(|(object, _)| in_reach(world, &here, *object, false))
        .collect();
    let kicks = near
        .iter()
        .filter(|(_, placed)| placed.resting(&room, TOLERANCE.value()) == Some(Resting::Floor))
        .map(|(object, _)| {
            Offer::complete(&Kick::new(*object), requirement).expect("a kick encodes")
        });
    let throws = near.iter().map(|(object, _)| {
        Offer::complete(&Throw::new(*object, None), requirement).expect("a throw encodes")
    });
    kicks.chain(throws).collect()
}
