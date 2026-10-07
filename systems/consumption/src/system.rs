//! The installable system: what it declares, how it decides a meal, and what it offers.

use mineworld_contracts::{ActionIntent, EntityId, Event, Rejection, SystemId};
use mineworld_inventory::{InventorySystem, ItemsConsumed};
use mineworld_kernel::{
    Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion, WorldRead,
    WorldView,
};
use mineworld_presence::{Offer, PerceptionProvider};
use mineworld_sdk::SystemPack;

use crate::action::{Drink, Eat};
use crate::codec::read_meal;
use crate::offer::{category, living_person, meals};

/// Eating and drinking what one carries.
#[derive(Default)]
pub struct ConsumptionSystem;

impl SystemIdentity for ConsumptionSystem {
    const ID: SystemId = SystemId::from_static("consumption");
}

/// Nothing biographical and no section: what the build needs to know about this pack beyond
/// [`System`] is nothing (`DECISIONS.md` `ARC-33`). Meals are thousands of facts that would bury a
/// biography (step-10 QS-49).
impl SystemPack for ConsumptionSystem {}

/// A refusal of inventory's, reached at resolve. Validation asked the same rule a moment earlier, so
/// reaching it is a defect, reported as the owner's refusal rather than recorded.
fn refused(reason: Rejection) -> KernelError {
    KernelError::FactRefusedByOwner {
        system: InventorySystem::ID,
        event_type: ItemsConsumed::EVENT_TYPE,
        reason,
    }
}

impl System for ConsumptionSystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    /// Depends on inventory, whose fact it states (`ARC-26`). Owns nothing.
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([InventorySystem::ID])
            .providing::<Eat>()
            .providing::<Drink>()
            .emitting::<ItemsConsumed>()
    }

    /// Whether this meal may happen, in order:
    ///
    /// 1. the payload is an `eat` or a `drink`;
    /// 2. the actor is a living Person and there is no target — `NoSupportedInteraction` otherwise;
    /// 3. the kind's category is the one the action needs: food is eaten, drink drunk, goods neither —
    ///    `NoSupportedInteraction` otherwise;
    /// 4. inventory's own rule, [`mineworld_inventory::admit_consumption`]: one is held.
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        let (item, wants) = read_meal(intent)?;
        let eater = intent.actor();
        if !living_person(world, eater) || intent.target().is_some() {
            return Err(Rejection::NoSupportedInteraction);
        }
        if category(world, item) != Some(wants) {
            return Err(Rejection::NoSupportedInteraction);
        }
        mineworld_inventory::admit_consumption(world, eater, item, 1)
    }

    /// States inventory's `items-consumed` through its checked constructor — and nothing else.
    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let (item, _) = read_meal(intent).map_err(refused)?;
        let fact = mineworld_inventory::consume(&world.read(), intent.actor(), item, 1)
            .map_err(refused)?;
        Ok(vec![fact])
    }
}

impl PerceptionProvider for ConsumptionSystem {
    /// One complete eat or drink per edible or drinkable kind the observer carries (`src/offer.rs`).
    /// Discloses nothing: holdings are inventory's to disclose.
    fn offers(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        target: Option<EntityId>,
    ) -> Vec<Offer> {
        meals(world, observer, target)
    }
}
