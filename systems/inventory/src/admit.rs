//! What holdings may take: the one rule, asked by every pack that decides a change and by this pack
//! again when it reduces one (`ARC-26`).

use mineworld_contracts::{EntityId, EntityType, ItemId, LifecycleState, Rejection, Visibility};
use mineworld_kernel::{Emission, WorldRead};

use crate::codec;
use crate::component::Holdings;
use crate::event::{ItemsTransferred, Stocked};

/// How many items a person can carry, every kind together.
///
/// An organization is not bounded. The bound exists because nothing in MVP-0 uses an item up: a person
/// nobody drives is given things and never gives, and without a bound such a person absorbs the
/// town's items within a month (step-10 F-39). With six, every seat keeps giving and the undriven
/// person ends holding six (`ARC-37` item 5). A published constant, not world configuration (the
/// `ARC-26` note's rule).
pub const PERSON_CAPACITY: u32 = 6;

/// Whether `entity` may hold anything: a living Person or Organization.
pub(crate) fn is_holder(world: &WorldRead<'_>, entity: EntityId) -> bool {
    world.entity(entity).is_some_and(|record| {
        matches!(
            record.entity_type(),
            EntityType::Person | EntityType::Organization
        ) && record.lifecycle() != LifecycleState::Destroyed
    })
}

/// Whether `holder` can take `count` more items: a living Organization always, a living Person only
/// within [`PERSON_CAPACITY`], anything else never.
///
/// The question a pack asks to say whether somebody is available to be given to, without asking
/// anything else about the transfer.
pub fn can_take(world: &WorldRead<'_>, holder: EntityId, count: u32) -> bool {
    let Some(record) = world.entity(holder) else {
        return false;
    };
    if !is_holder(world, holder) {
        return false;
    }
    if record.entity_type() != EntityType::Person {
        return true;
    }
    let held = world
        .component::<Holdings>(holder)
        .map_or(0, Holdings::total);
    held + u64::from(count) <= u64::from(PERSON_CAPACITY)
}

/// Whether holdings may take this transfer: the whole of what this pack refuses about one.
///
/// Asked at three moments — by a deciding pack's `validate`, by [`transfer`] before a fact is built,
/// and by this pack's reduction before anything is written — so the three cannot come to disagree.
/// It reads only.
///
/// # Errors
///
/// - [`Rejection::PreconditionFailed`]: a count of zero; `from` and `to` the same; either not a living
///   Person or Organization; `item` not a declared kind; `from` holding fewer than `count`.
/// - [`Rejection::TargetUnavailable`]: `to` cannot take `count` more ([`can_take`]).
pub fn admit_transfer(
    world: &WorldRead<'_>,
    from: EntityId,
    to: EntityId,
    item: ItemId,
    count: u32,
) -> Result<(), Rejection> {
    if count == 0 || from == to || !is_holder(world, from) || !is_holder(world, to) {
        return Err(Rejection::PreconditionFailed);
    }
    if !mineworld_item::is_declared(world, item) {
        return Err(Rejection::PreconditionFailed);
    }
    let has = world
        .component::<Holdings>(from)
        .map_or(0, |held| held.count(item));
    if has < count {
        return Err(Rejection::PreconditionFailed);
    }
    if !can_take(world, to, count) {
        return Err(Rejection::TargetUnavailable);
    }
    Ok(())
}

/// The checked constructor: `count` of `item` from `from` to `to`, as this pack's
/// [`ItemsTransferred`], ready to record.
///
/// The only way another pack states the fact (`ARC-26`): it must declare the emission and depend on
/// this pack, and it gets the bytes, the audience and the participants from here. Asks
/// [`admit_transfer`] first, so a pack that follows the contract cannot record a transfer this pack
/// would refuse.
///
/// # Errors
///
/// What [`admit_transfer`] refuses.
pub fn transfer(
    world: &WorldRead<'_>,
    from: EntityId,
    to: EntityId,
    item: ItemId,
    count: u32,
) -> Result<Emission, Rejection> {
    admit_transfer(world, from, to, item, count)?;
    Ok(Emission::new::<ItemsTransferred>(
        codec::encode(&ItemsTransferred::new(from, to, item, count)),
        Visibility::Participants,
    )
    .about(vec![from, to])
    .with_participants(vec![from, to]))
}

/// Whether holdings may take a `stocked` fact: a living holder, a count of at least one, a declared
/// kind, and room for it.
///
/// Asked at reduction. The seed cannot ask about the kind, because every genesis fact is computed
/// before any is reduced (step-10 F-37); the reduction follows `item-kind-declared` in genesis order.
pub(crate) fn admit_stock(
    world: &WorldRead<'_>,
    holder: EntityId,
    item: ItemId,
    count: u32,
) -> Result<(), Rejection> {
    if count == 0 || !is_holder(world, holder) || !mineworld_item::is_declared(world, item) {
        return Err(Rejection::PreconditionFailed);
    }
    if !can_take(world, holder, count) {
        return Err(Rejection::TargetUnavailable);
    }
    Ok(())
}

/// A `stocked` fact, ready to record: visible to the holder, about the holder.
pub(crate) fn stocked(holder: EntityId, item: ItemId, count: u32) -> Emission {
    Emission::new::<Stocked>(
        codec::encode(&Stocked::new(holder, item, count)),
        Visibility::Entities([holder].into_iter().collect()),
    )
    .about(vec![holder])
    .with_participants(vec![holder])
}
