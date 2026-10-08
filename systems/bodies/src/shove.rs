//! `shove`: whether one may happen, and what it becomes (step-11 SD-O15; QB-8: no `Busy`).
//!
//! ```text
//! validate   the payload decodes                                          PreconditionFailed
//!            the actor is a living Person with a position                 PreconditionFailed
//!            the target is a different, living Person                     NoSupportedInteraction
//!            the target stands at a position in a shaped place            TargetUnavailable
//!            the declared requirement: same place, within 1 000 mm        TooFarAway
//! resolve    person-shoved { by, person }, then presence's arrivals() for the target 500 mm on,
//!            straight away from the shover — resolved by the registered resolvers like any arrival
//! ```
//!
//! Bodies never writes a position: presence records the shoved person's arrival, and presence's
//! constructor asks this pack's own resolver first, so walls stop them, the people behind them are
//! nudged within the bounds of `ARC-39`'s note, and objects are pushed or block (`ARC-39` note 2).

use mineworld_contracts::{
    ActionIntent, EntityId, EntityType, Event, LifecycleState, LocalPosition, Location,
    Millimetres, PersonId, Rejection,
};
use mineworld_kernel::{Emission, KernelError, SystemIdentity, WorldRead};
use mineworld_presence::{Arrived, Presence, PresenceSystem, arrivals};

use crate::action::{Shove, shove_requirement};
use crate::component::PlaceShape;
use crate::event::{PersonShoved, person_shoved};
use crate::geometry::{Point, SHOVE_DISTANCE, distance2, scaled_down};
use crate::launch::{read, standing};
use crate::system::refused;

/// Whether `target` is somebody a person could shove: a living Person other than `actor`.
fn shovable(world: &WorldRead<'_>, actor: EntityId, target: EntityId) -> bool {
    target != actor
        && world.entity(target).is_some_and(|record| {
            record.entity_type() == EntityType::Person
                && record.lifecycle() != LifecycleState::Destroyed
        })
}

/// Where `target` stands, and whether a shove can move them: a position, in a shaped place.
pub(crate) fn has_body(world: &WorldRead<'_>, target: EntityId) -> (Option<Location>, bool) {
    let there = world.component::<Presence>(target).map(Presence::location);
    let available = there.is_some_and(|location| {
        location.local().is_some()
            && world
                .component::<PlaceShape>(location.place().entity_id())
                .is_some()
    });
    (there, available)
}

/// SD-O15's `validate`, in order.
pub(crate) fn validate(world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
    let _: Shove = read(intent)?;
    let here = standing(world, intent.actor())?;
    let target = intent.target().ok_or(Rejection::NoSupportedInteraction)?;
    if !shovable(world, intent.actor(), target) {
        return Err(Rejection::NoSupportedInteraction);
    }
    let (there, available) = has_body(world, target);
    shove_requirement().evaluate(&here, there.as_ref(), available)
}

/// The shove: who shoved whom, then the shoved person's arrival 500 mm on, through presence.
pub(crate) fn resolve(
    world: &WorldRead<'_>,
    intent: &ActionIntent,
) -> Result<Vec<Emission>, KernelError> {
    validate(world, intent).map_err(|reason| refused(PersonShoved::EVENT_TYPE, reason))?;
    let here = standing(world, intent.actor())
        .map_err(|reason| refused(PersonShoved::EVENT_TYPE, reason))?;
    let target = intent.target().expect("a target, checked by validate");
    let there = world
        .component::<Presence>(target)
        .map(Presence::location)
        .expect("the target stands somewhere, checked by validate");
    let (from, at) = (ground(&here), ground(&there));
    // Scaled by 500 / ⌈|d|⌉, each component truncated toward zero, so the shove asks for at most
    // SHOVE_DISTANCE (a floor of |d| could make it a fraction of a millimetre longer).
    let d = at.minus(from);
    let squared = distance2(d, Point::new(0, 0));
    let floor = squared.isqrt();
    let length = if floor * floor == squared {
        floor
    } else {
        floor + 1
    }
    .max(1);
    let on = at.plus(scaled_down(d, i64::from(SHOVE_DISTANCE.value()), length));
    let local = there.local().expect("a position, checked by validate");
    let to = Location::in_place(there.place()).with_local(LocalPosition::new(
        Millimetres::new(on.x),
        Millimetres::new(on.y),
        local.z(),
    ));
    let to = match there.facing() {
        Some(facing) => to.with_facing(facing),
        None => to,
    };
    let person = |entity| {
        PersonId::new(entity, EntityType::Person)
            .map_err(|_| refused(PersonShoved::EVENT_TYPE, Rejection::PreconditionFailed))
    };
    let (by, shoved) = (person(intent.actor())?, person(target)?);
    let mut facts = vec![person_shoved(by, shoved, there.place())];
    facts.extend(arrivals(world, shoved, to).map_err(|reason| {
        KernelError::FactRefusedByOwner {
            system: PresenceSystem::ID,
            event_type: Arrived::EVENT_TYPE,
            reason,
        }
    })?);
    Ok(facts)
}

fn ground(location: &Location) -> Point {
    let local = location.local().expect("a position, checked by validate");
    Point::new(local.x().value(), local.y().value())
}
