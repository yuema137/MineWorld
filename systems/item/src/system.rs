//! The installable system: what it declares, the one fact it reduces, and the question others ask.

use mineworld_contracts::{
    EntityType, Event, EventEnvelope, ItemId, LifecycleState, Rejection, SystemId,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    WorldRead, WorldView,
};
use mineworld_presence::PerceptionProvider;
use mineworld_sdk::SystemPack;

use crate::codec;
use crate::component::ItemKind;
use crate::event::ItemKindDeclared;

/// What kinds of things exist.
#[derive(Default)]
pub struct ItemSystem;

impl SystemIdentity for ItemSystem {
    const ID: SystemId = SystemId::from_static("item");
}

/// What the build needs to know about this pack beyond [`System`] (`DECISIONS.md` `ARC-33`): nothing
/// biographical, and the `item:` section of an item file, which its `AuthoredSection` impl
/// (`src/section.rs`) describes.
impl SystemPack for ItemSystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    mineworld_sdk::owns_section!();
}

/// Whether `item` is a living Item entity.
fn living_item(world: &WorldRead<'_>, item: ItemId) -> bool {
    world.entity(item.entity_id()).is_some_and(|record| {
        record.entity_type() == EntityType::Item && record.lifecycle() != LifecycleState::Destroyed
    })
}

impl System for ItemSystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    /// No dependency, no action, no process: a kind is stated at genesis and reduced here.
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<ItemKind>()
            .emitting::<ItemKindDeclared>()
            .subscribing_to::<ItemKindDeclared>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<ItemKind>()
    }

    /// Reduces `item-kind-declared` into the kind's [`ItemKind`] — the only write this pack makes.
    ///
    /// The owner still decides (`ARC-26`): a declaration about an entity that is not a living Item
    /// writes nothing and fails with [`KernelError::FactRefusedByOwner`].
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if *event.event_type() != ItemKindDeclared::EVENT_TYPE {
            return Ok(Vec::new());
        }
        let declared: ItemKindDeclared = codec::event_payload(event.payload())?;
        if !living_item(&world.read(), declared.item()) {
            return Err(KernelError::FactRefusedByOwner {
                system: Self::ID,
                event_type: ItemKindDeclared::EVENT_TYPE,
                reason: Rejection::PreconditionFailed,
            });
        }
        world.insert(
            declared.item().entity_id(),
            ItemKind::new(declared.category().clone()),
        )?;
        Ok(Vec::new())
    }
}

/// Offers nothing and discloses nothing: items are never perceived, so there is nobody to tell what a
/// kind is (`ARC-37`).
impl PerceptionProvider for ItemSystem {}

/// Whether `item` is a kind this pack has declared: a living Item entity with an [`ItemKind`].
///
/// The question inventory, and later the market packs, ask before they let anybody hold, give, sell
/// or produce it. A declared kind is never undeclared in MVP-0.
pub fn is_declared(world: &WorldRead<'_>, item: ItemId) -> bool {
    living_item(world, item) && world.component::<ItemKind>(item.entity_id()).is_some()
}
