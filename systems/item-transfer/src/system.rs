//! The installable system: what it declares, how it decides a give, a put-down and a pick-up, and what
//! it offers.

use mineworld_contracts::{Action, ActionIntent, EntityId, Event, Location, Rejection, SystemId};
use mineworld_inventory::{InventorySystem, ItemsTransferred};
use mineworld_kernel::{
    Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion, WorldRead,
    WorldView,
};
use mineworld_presence::{Offer, PerceptionProvider, Presence, PresenceSystem};
use mineworld_sdk::SystemPack;

use crate::action::{Give, give_requirement};
use crate::codec::{read, read_give};
use crate::hands::{self, PickUp, PutDown, hands_requirement, judge_pick_up, judge_put_down};
use crate::offer::{gives, living_person};

/// Giving things to people, and putting them down and picking them up.
#[derive(Default)]
pub struct ItemTransferSystem;

impl SystemIdentity for ItemTransferSystem {
    const ID: SystemId = SystemId::from_static("item-transfer");
}

/// Nothing biographical and no section: what the build needs to know about this pack beyond
/// [`System`] is nothing (`DECISIONS.md` `ARC-33`).
impl SystemPack for ItemTransferSystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
}

fn located(world: &WorldRead<'_>, entity: EntityId) -> Option<Location> {
    world.component::<Presence>(entity).map(Presence::location)
}

/// A refusal of inventory's, reached at resolve. Validation asked the same rule a moment earlier, so
/// reaching it is a defect, reported as the owner's refusal rather than recorded.
fn refused(reason: Rejection) -> KernelError {
    KernelError::FactRefusedByOwner {
        system: InventorySystem::ID,
        event_type: ItemsTransferred::EVENT_TYPE,
        reason,
    }
}

impl System for ItemTransferSystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    /// Depends on inventory, whose fact it states (`ARC-26`), and on presence, whose positions it
    /// reads. Owns nothing.
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([InventorySystem::ID, PresenceSystem::ID])
            .providing::<Give>()
            .providing::<PutDown>()
            .providing::<PickUp>()
            .emitting::<ItemsTransferred>()
    }

    /// Whether this request may happen. A `put-down` or `pick-up` is judged by `hands.rs`, then its
    /// requirement. A `give`, in order:
    ///
    /// 1. the payload is a `give`;
    /// 2. the giver is a living Person, and the target a *different* living Person —
    ///    `NoSupportedInteraction` otherwise, a missing target included;
    /// 3. the requirement, against presence's positions and inventory's answer to whether the target
    ///    can take `count` (`TargetUnavailable`, `TooFarAway`);
    /// 4. inventory's own rule, [`mineworld_inventory::admit_transfer`].
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        let action = intent.action_type();
        if *action == PutDown::ACTION_TYPE || *action == PickUp::ACTION_TYPE {
            if intent.target().is_some() {
                return Err(Rejection::NoSupportedInteraction);
            }
            let place = if *action == PutDown::ACTION_TYPE {
                judge_put_down(world, intent.actor(), read::<PutDown>(intent)?.item())?
            } else {
                judge_pick_up(world, intent.actor(), read::<PickUp>(intent)?.item())?
            };
            let here = located(world, intent.actor()).ok_or(Rejection::PreconditionFailed)?;
            return hands_requirement(place).evaluate(&here, None, true);
        }
        let give = read_give(intent)?;
        let giver = intent.actor();
        let taker = intent.target().ok_or(Rejection::NoSupportedInteraction)?;
        if giver == taker || !living_person(world, giver) || !living_person(world, taker) {
            return Err(Rejection::NoSupportedInteraction);
        }
        let here = located(world, giver).ok_or(Rejection::PreconditionFailed)?;
        let there = located(world, taker);
        give_requirement().evaluate(
            &here,
            there.as_ref(),
            mineworld_inventory::can_take(world, taker, give.count()),
        )?;
        mineworld_inventory::admit_transfer(world, giver, taker, give.item(), give.count())
    }

    /// States inventory's `items-transferred` through its checked constructor — and nothing else: to
    /// the target for a give, to the place for a put-down, from it for a pick-up (heard in the place).
    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let read_world = world.read();
        let actor = intent.actor();
        if *intent.action_type() == PutDown::ACTION_TYPE {
            let item = read::<PutDown>(intent).map_err(refused)?.item();
            let place = judge_put_down(&read_world, actor, item).map_err(refused)?;
            let fact = hands::moved(&read_world, actor, place.entity_id(), item, place)
                .map_err(refused)?;
            return Ok(vec![fact]);
        }
        if *intent.action_type() == PickUp::ACTION_TYPE {
            let item = read::<PickUp>(intent).map_err(refused)?.item();
            let place = judge_pick_up(&read_world, actor, item).map_err(refused)?;
            let fact = hands::moved(&read_world, place.entity_id(), actor, item, place)
                .map_err(refused)?;
            return Ok(vec![fact]);
        }
        let give = read_give(intent).map_err(refused)?;
        let taker = intent
            .target()
            .ok_or_else(|| refused(Rejection::NoSupportedInteraction))?;
        let fact =
            mineworld_inventory::transfer(&read_world, actor, taker, give.item(), give.count())
                .map_err(refused)?;
        Ok(vec![fact])
    }
}

impl PerceptionProvider for ItemTransferSystem {
    /// One complete give per kind the observer holds, to each other living person present
    /// (`src/offer.rs`); with no target, one complete put-down per kind held and one pick-up per kind
    /// the place holds (`src/hands.rs`). Discloses nothing: holdings are inventory's to disclose.
    fn offers(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        target: Option<EntityId>,
    ) -> Vec<Offer> {
        let mut offered = gives(world, observer, target);
        offered.extend(hands::offers(world, observer, target));
        offered
    }
}
