//! The installable system: what it declares, the two facts it reduces, and what it discloses.

use mineworld_contracts::SystemId;
use mineworld_contracts::{
    ComponentRecord, EntityId, Event, EventEnvelope, EventTypeId, Rejection,
};
use mineworld_item::ItemSystem;
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    WorldRead, WorldView,
};
use mineworld_presence::PerceptionProvider;
use mineworld_sdk::SystemPack;
use serde_json::Value;

use crate::admit::{admit_stock, admit_transfer};
use crate::codec;
use crate::component::Holdings;
use crate::event::{ItemsTransferred, Stocked};

/// What people and organizations hold.
#[derive(Default)]
pub struct InventorySystem;

impl SystemIdentity for InventorySystem {
    const ID: SystemId = SystemId::from_static("inventory");
}

/// What the build needs to know about this pack beyond [`System`] (`DECISIONS.md` `ARC-33`): nothing
/// biographical — owning a coffee is not an event in a life, and eighteen thousand gives would bury a
/// biography (step-10 QS-29) — and the `holdings:` section, which its `AuthoredSection` impl
/// (`src/section.rs`) describes.
impl SystemPack for InventorySystem {
    mineworld_sdk::owns_section!();
}

/// A fact another system stated, which holdings may not take. Not a [`Rejection`] result: every
/// stater was required to ask [`admit_transfer`] first, so reaching this is a defect in the stater
/// (`ARC-26`).
fn refused(event_type: EventTypeId, reason: Rejection) -> KernelError {
    KernelError::FactRefusedByOwner {
        system: InventorySystem::ID,
        event_type,
        reason,
    }
}

impl System for InventorySystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    /// Depends on `item`, whose declared kinds are the only things anybody may hold. Provides no
    /// action: a give is `item-transfer`'s decision, stated in this pack's vocabulary.
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([ItemSystem::ID])
            .owning::<Holdings>()
            .emitting::<Stocked>()
            .emitting::<ItemsTransferred>()
            .subscribing_to::<Stocked>()
            .subscribing_to::<ItemsTransferred>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Holdings>()
    }

    /// Reduces `stocked` and `items-transferred` into [`Holdings`] — the only writes of holdings
    /// anywhere.
    ///
    /// **The owner still decides** (`ARC-26`). Before writing, the fact is put to the same rule its
    /// constructor asked: [`admit_transfer`] for a transfer, the stock rule for `stocked`. A refusal
    /// writes nothing and fails with [`KernelError::FactRefusedByOwner`].
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if *event.event_type() == Stocked::EVENT_TYPE {
            let stocked: Stocked = codec::event_payload(event.payload())?;
            let read = world.read();
            admit_stock(&read, stocked.holder(), stocked.item(), stocked.count())
                .map_err(|reason| refused(Stocked::EVENT_TYPE, reason))?;
            let next = held_by(&read, stocked.holder())
                .adding(stocked.item(), stocked.count())
                .ok_or_else(|| refused(Stocked::EVENT_TYPE, Rejection::PreconditionFailed))?;
            world.insert(stocked.holder(), next)?;
        } else if *event.event_type() == ItemsTransferred::EVENT_TYPE {
            let moved: ItemsTransferred = codec::event_payload(event.payload())?;
            let read = world.read();
            admit_transfer(&read, moved.from(), moved.to(), moved.item(), moved.count())
                .map_err(|reason| refused(ItemsTransferred::EVENT_TYPE, reason))?;
            let unchanged = || refused(ItemsTransferred::EVENT_TYPE, Rejection::PreconditionFailed);
            let from = held_by(&read, moved.from())
                .removing(moved.item(), moved.count())
                .ok_or_else(unchanged)?;
            let to = held_by(&read, moved.to())
                .adding(moved.item(), moved.count())
                .ok_or_else(unchanged)?;
            world.insert(moved.from(), from)?;
            world.insert(moved.to(), to)?;
        }
        Ok(Vec::new())
    }
}

fn held_by(world: &WorldRead<'_>, holder: EntityId) -> Holdings {
    world
        .component::<Holdings>(holder)
        .cloned()
        .unwrap_or_default()
}

impl PerceptionProvider for InventorySystem {
    /// The subject's [`Holdings`] to the subject alone: what somebody carries is theirs to know
    /// (`INV-13`). A stranger learns of a give only as its participant.
    fn discloses(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        subject: EntityId,
    ) -> Vec<ComponentRecord<Value>> {
        if observer != subject {
            return Vec::new();
        }
        world
            .component::<Holdings>(subject)
            .map(|held| {
                vec![ComponentRecord::new::<Holdings>(
                    subject,
                    codec::to_value(held),
                )]
            })
            .unwrap_or_default()
    }
}
