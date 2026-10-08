//! The installable system: what it declares, how it decides a give, and what it offers.

use mineworld_contracts::{ActionIntent, EntityId, Event, Location, Rejection, SystemId};
use mineworld_inventory::{InventorySystem, ItemsTransferred};
use mineworld_kernel::{
    Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion, WorldRead,
    WorldView,
};
use mineworld_presence::{Offer, PerceptionProvider, Presence, PresenceSystem};
use mineworld_sdk::SystemPack;

use crate::action::{Give, give_requirement};
use crate::codec::read_give;
use crate::offer::{gives, living_person};

/// Giving things to people.
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
            .emitting::<ItemsTransferred>()
    }

    /// Whether this give may happen, in order:
    ///
    /// 1. the payload is a `give`;
    /// 2. the giver is a living Person, and the target a *different* living Person —
    ///    `NoSupportedInteraction` otherwise, a missing target included;
    /// 3. the requirement, against presence's positions and inventory's answer to whether the target
    ///    can take `count` (`TargetUnavailable`, `TooFarAway`);
    /// 4. inventory's own rule, [`mineworld_inventory::admit_transfer`].
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
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

    /// States inventory's `items-transferred` through its checked constructor — and nothing else.
    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let give = read_give(intent).map_err(refused)?;
        let taker = intent
            .target()
            .ok_or_else(|| refused(Rejection::NoSupportedInteraction))?;
        let fact = mineworld_inventory::transfer(
            &world.read(),
            intent.actor(),
            taker,
            give.item(),
            give.count(),
        )
        .map_err(refused)?;
        Ok(vec![fact])
    }
}

impl PerceptionProvider for ItemTransferSystem {
    /// One complete give per kind the observer holds, to each other living person present
    /// (`src/offer.rs`). Discloses nothing: holdings are inventory's to disclose.
    fn offers(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        target: Option<EntityId>,
    ) -> Vec<Offer> {
        gives(world, observer, target)
    }
}
