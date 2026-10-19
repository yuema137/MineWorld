//! The installable system: what it declares, the one fact it reduces, and the question others ask.

use mineworld_contracts::{
    ComponentRecord, EntityId, EntityType, Event, EventEnvelope, ItemId, LifecycleState, Rejection,
    SystemId,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    WorldRead, WorldView,
};
use mineworld_presence::PerceptionProvider;
use mineworld_sdk::SystemPack;
use serde_json::Value;

use crate::codec;
use crate::component::{ItemCatalogue, ItemKind};
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
    /// Version 2: a kind has a name, and the catalogue is disclosed (step-13 R-PK-2; `ARC-37` note).
    const VERSION: SystemVersion = SystemVersion::new(2);

    /// No dependency, no action, no process: a kind is stated at genesis and reduced here.
    /// [`ItemCatalogue`] is owned so that it may be disclosed; its table is never written.
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<ItemKind>()
            .owning::<ItemCatalogue>()
            .emitting::<ItemKindDeclared>()
            .subscribing_to::<ItemKindDeclared>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<ItemKind>()?;
        tables.component::<ItemCatalogue>()
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
            ItemKind::new(declared.category().clone(), declared.name().clone()),
        )?;
        Ok(Vec::new())
    }
}

/// Offers nothing. Items are never perceived as entities (`ARC-37`); a place discloses the world's
/// kinds as a catalogue instead (`ARC-37` note, step-11 SD-D10).
impl PerceptionProvider for ItemSystem {
    /// To whoever perceives a place, an [`ItemCatalogue`] on it: every declared kind, its category and
    /// its name, in `ItemId` order, built now from [`ItemKind`]. A person, an item, a world with no
    /// kinds: nothing.
    fn discloses(
        &self,
        world: &WorldRead<'_>,
        _observer: EntityId,
        subject: EntityId,
    ) -> Vec<ComponentRecord<Value>> {
        let is_place = world
            .entity(subject)
            .is_some_and(|entity| entity.entity_type() == EntityType::Place);
        if !is_place {
            return Vec::new();
        }
        ItemCatalogue::of(world)
            .map(|catalogue| {
                ComponentRecord::new::<ItemCatalogue>(subject, codec::to_value(&catalogue))
            })
            .into_iter()
            .collect()
    }
}

/// Whether `item` is a kind this pack has declared: a living Item entity with an [`ItemKind`].
///
/// The question inventory, and later the market packs, ask before they let anybody hold, give, sell
/// or produce it. A declared kind is never undeclared in MVP-0.
pub fn is_declared(world: &WorldRead<'_>, item: ItemId) -> bool {
    living_item(world, item) && world.component::<ItemKind>(item.entity_id()).is_some()
}
