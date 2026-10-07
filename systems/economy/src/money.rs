//! What a payment may be: the one rule, asked before money moves for a purchase or a wage, and again
//! when this pack reduces the payment (`ARC-26`, `ARC-38` item 3).

use mineworld_contracts::{EntityId, EntityType, LifecycleState, Rejection, Visibility};
use mineworld_kernel::{Emission, WorldRead};

use crate::codec;
use crate::component::Wallet;
use crate::event::MoneyTransferred;

/// Whether `entity` may hold money: a living Person or Organization.
pub(crate) fn is_holder(world: &WorldRead<'_>, entity: EntityId) -> bool {
    world.entity(entity).is_some_and(|record| {
        matches!(
            record.entity_type(),
            EntityType::Person | EntityType::Organization
        ) && record.lifecycle() != LifecycleState::Destroyed
    })
}

/// What `holder` has, in minor units; zero without a wallet.
pub fn balance(world: &WorldRead<'_>, holder: EntityId) -> u64 {
    world.component::<Wallet>(holder).map_or(0, Wallet::balance)
}

/// Whether `amount` may pass from `from` to `to`: the whole of what this pack refuses about a payment.
///
/// # Errors
///
/// [`Rejection::PreconditionFailed`]: an amount of zero; `from` and `to` the same; either not a living
/// Person or Organization; `from` holding less than `amount`.
pub fn admit_payment(
    world: &WorldRead<'_>,
    from: EntityId,
    to: EntityId,
    amount: u64,
) -> Result<(), Rejection> {
    if amount == 0 || from == to || !is_holder(world, from) || !is_holder(world, to) {
        return Err(Rejection::PreconditionFailed);
    }
    if balance(world, from) < amount {
        return Err(Rejection::PreconditionFailed);
    }
    Ok(())
}

/// A payment, as this pack's [`MoneyTransferred`], ready to record, heard by `audience`. Asks
/// [`admit_payment`] first.
pub(crate) fn pay(
    world: &WorldRead<'_>,
    from: EntityId,
    to: EntityId,
    amount: u64,
    audience: Visibility,
) -> Result<Emission, Rejection> {
    admit_payment(world, from, to, amount)?;
    Ok(Emission::new::<MoneyTransferred>(
        codec::encode(&MoneyTransferred::new(from, to, amount)),
        audience,
    )
    .about(vec![from, to])
    .with_participants(vec![from, to]))
}
