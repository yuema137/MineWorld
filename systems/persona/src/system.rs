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
use crate::component::PersonaComponent;
use crate::event::PersonaAuthored;

/// Which of this pack's facts belong in a person's objective biography (`ARC-29`): none. A character
/// is not an event in a life.
pub const BIOGRAPHICAL: &[EventTypeId] = &[];

/// A person's authored character.
#[derive(Default)]
pub struct PersonaSystem;

impl SystemIdentity for PersonaSystem {
    const ID: SystemId = SystemId::from_static("persona");
}

/// What the build needs to know about this pack beyond [`System`] (`DECISIONS.md` `ARC-33`): its
/// biographical facts (none), and the `persona:` section of a person's file, which its
/// `AuthoredSection` impl (`src/section.rs`) describes.
impl SystemPack for PersonaSystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    const BIOGRAPHICAL: &'static [EventTypeId] = BIOGRAPHICAL;
    mineworld_sdk::owns_section!();
}

impl System for PersonaSystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    /// No dependency, no action, no process, no reaction: a character is stated at genesis and
    /// reduced here.
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<PersonaComponent>()
            .emitting::<PersonaAuthored>()
            .subscribing_to::<PersonaAuthored>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<PersonaComponent>()
    }

    /// Reduces `persona-authored` into the person's [`PersonaComponent`] — the only write this pack
    /// makes.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if *event.event_type() == PersonaAuthored::EVENT_TYPE {
            let authored: PersonaAuthored = codec::event_payload(event.payload())?;
            world.insert(
                authored.person().entity_id(),
                PersonaComponent::new(authored.persona().clone()),
            )?;
        }
        Ok(Vec::new())
    }
}

impl PerceptionProvider for PersonaSystem {
    /// The persona, to its holder and to nobody else: a character is private to the person it
    /// describes (`ARC-74`), as relationships are disclosed to their parties (`ARC-28` point 6).
    fn discloses(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        subject: EntityId,
    ) -> Vec<ComponentRecord<Value>> {
        if subject != observer {
            return Vec::new();
        }
        world
            .component::<PersonaComponent>(subject)
            .map(|persona| {
                vec![ComponentRecord::new::<PersonaComponent>(
                    subject,
                    codec::to_value(persona),
                )]
            })
            .unwrap_or_default()
    }
}
