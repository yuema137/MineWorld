//! The installable system: what it declares, the one fact it reduces, and what it discloses.

use mineworld_contracts::{ComponentRecord, EntityId, Event, EventEnvelope, EventTypeId, SystemId};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    WorldRead, WorldView,
};
use mineworld_presence::PerceptionProvider;
use mineworld_sdk::SystemPack;
use serde_json::Value;

use crate::codec;
use crate::component::Look;
use crate::event::Appeared;

/// Which of this pack's facts belong in a person's objective biography (`ARC-29`): none. Looking like
/// somebody is not an event in a life.
pub const BIOGRAPHICAL: &[EventTypeId] = &[];

/// What people look like.
#[derive(Default)]
pub struct AppearanceSystem;

impl SystemIdentity for AppearanceSystem {
    const ID: SystemId = SystemId::from_static("appearance");
}

/// What the build needs to know about this pack beyond [`System`] (`DECISIONS.md` `ARC-33`): its
/// biographical facts (none), and the `appearance:` section of a person's file, which its
/// `AuthoredSection` impl (`src/section.rs`) describes.
impl SystemPack for AppearanceSystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    const BIOGRAPHICAL: &'static [EventTypeId] = BIOGRAPHICAL;
    mineworld_sdk::owns_section!();
}

impl System for AppearanceSystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    /// No dependency, no action, no process: a look is stated at genesis and reduced here.
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<Look>()
            .emitting::<Appeared>()
            .subscribing_to::<Appeared>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Look>()
    }

    /// Reduces `appeared` into the person's [`Look`] — the only write this pack makes.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if *event.event_type() == Appeared::EVENT_TYPE {
            let appeared: Appeared = codec::event_payload(event.payload())?;
            world.insert(
                appeared.person().entity_id(),
                Look::new(appeared.look().clone()),
            )?;
        }
        Ok(Vec::new())
    }
}

impl PerceptionProvider for AppearanceSystem {
    /// The subject's look, to **every** observer who perceives them, oneself included. Perception asks
    /// only about entities the observer already perceives, so nobody learns how somebody they cannot
    /// see looks. Appearance never depends on what the observer knows (`ARC-91`).
    fn discloses(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        subject: EntityId,
    ) -> Vec<ComponentRecord<Value>> {
        let _ = observer;
        world
            .component::<Look>(subject)
            .map(|look| vec![ComponentRecord::new::<Look>(subject, codec::to_value(look))])
            .unwrap_or_default()
    }
}
