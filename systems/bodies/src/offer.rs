//! What this pack offers an observer (step-11 SD-O12): complete affordances (`ARC-34`), so the
//! unchanged paced controller attempts them.
//!
//! ```text
//! target none     for each object lying in the observer's place whose ground point is within
//!                 800 mm, in ItemId order: kick { object } (on the floor only), then
//!                 throw { object, toward: null } — each at_place(the place), available.
//!                 Objects beyond reach are not offered (QO-7)
//! target a person shove {} to a different, living Person — same_place().within(1 000 mm),
//!                 available iff they stand at a position in a shaped place
//! ```
//!
//! Nothing to an observer without a position, and nothing in a place without a shape.

use mineworld_contracts::{EntityId, EntityType, LifecycleState, Location, SpatialRequirement};
use mineworld_kernel::WorldRead;
use mineworld_presence::Offer;

use crate::action::{Kick, Shove, Throw, shove_requirement};
use crate::component::PlaceShape;
use crate::footprint::Resting;
use crate::geometry::{SHOVE_OFFER_REACH, TOLERANCE};
use crate::launch::{in_reach, standing};
use crate::objects::lying_in;
use crate::shove::has_body;

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
    if let Some(target) = target {
        return shoves(world, observer, &here, target);
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

/// `shove {}` to `target`, a different, living Person (step-11 SD-O12): available iff they stand at
/// a position in a shaped place; perception prices the distance (`TooFarAway` beyond 1 000 mm). Never
/// to oneself (step-10 F-40). Complete only when the target stands within `SHOVE_OFFER_REACH` of the
/// observer (SD-O18's rung p1).
fn shoves(
    world: &WorldRead<'_>,
    observer: EntityId,
    here: &Location,
    target: EntityId,
) -> Vec<Offer> {
    let person = world.entity(target).is_some_and(|record| {
        record.entity_type() == EntityType::Person
            && record.lifecycle() != LifecycleState::Destroyed
    });
    if target == observer || !person {
        return Vec::new();
    }
    let (there, available) = has_body(world, target);
    let close = available
        && SpatialRequirement::same_place()
            .within(SHOVE_OFFER_REACH)
            .is_ok_and(|near| near.evaluate(here, there.as_ref(), true).is_ok());
    let offer = if close {
        Offer::complete(&Shove {}, shove_requirement()).expect("a shove encodes")
    } else {
        Offer::new::<Shove>(shove_requirement())
    };
    vec![offer.with_target_available(available)]
}
