//! The installable system: what it declares, the one fact it reduces, and what it discloses.

use std::collections::BTreeMap;

use mineworld_contracts::{ComponentRecord, EntityId, Event, EventEnvelope, EventTypeId, SystemId};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    WorldRead, WorldView,
};
use mineworld_presence::PerceptionProvider;
use mineworld_sdk::SystemPack;
use serde_json::Value;

use crate::codec;
use crate::component::DisplayName;
use crate::event::Named;
use crate::name::Name;

/// Which of this pack's facts belong in a person's objective biography (`ARC-29`): none. Being called
/// something is not an event in a life.
pub const BIOGRAPHICAL: &[EventTypeId] = &[];

/// What people are called.
#[derive(Default)]
pub struct NamingSystem;

impl SystemIdentity for NamingSystem {
    const ID: SystemId = SystemId::from_static("naming");
}

/// What the build needs to know about this pack beyond [`System`] (`DECISIONS.md` `ARC-33`): its
/// biographical facts (none), and the `name:` section of a person's file, which its
/// `AuthoredSection` impl (`src/section.rs`) describes.
impl SystemPack for NamingSystem {
    const BIOGRAPHICAL: &'static [EventTypeId] = BIOGRAPHICAL;
    mineworld_sdk::owns_section!();
}

impl System for NamingSystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    /// No dependency, no action, no process: a name is stated at genesis and reduced here.
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<DisplayName>()
            .emitting::<Named>()
            .subscribing_to::<Named>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<DisplayName>()
    }

    /// Reduces `named` into the person's [`DisplayName`] — the only write this pack makes.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if *event.event_type() == Named::EVENT_TYPE {
            let named: Named = codec::event_payload(event.payload())?;
            world.insert(
                named.person().entity_id(),
                DisplayName::new(named.name().clone()),
            )?;
        }
        Ok(Vec::new())
    }
}

impl PerceptionProvider for NamingSystem {
    /// The subject's name, to **every** observer who perceives them, oneself included: names are
    /// public in MVP-0 (`ARC-31`). Perception asks only about entities the observer already perceives,
    /// so nobody learns the name of somebody they cannot see.
    fn discloses(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        subject: EntityId,
    ) -> Vec<ComponentRecord<Value>> {
        let _ = observer;
        world
            .component::<DisplayName>(subject)
            .map(|name| {
                vec![ComponentRecord::new::<DisplayName>(
                    subject,
                    codec::to_value(name),
                )]
            })
            .unwrap_or_default()
    }
}

/// Every person's name according to `facts` — the latest `named` per person.
///
/// This pack's own projection of its own facts, published for readers of a saved history
/// (`mineworld biography`), so that no reader decodes this pack's payload with a copy of its shape
/// (`step-09-social.md` Q6's condition, applied to readers). A fact of any other type is skipped.
pub fn names_in(facts: &[EventEnvelope]) -> BTreeMap<EntityId, Name> {
    let mut names = BTreeMap::new();
    for fact in facts {
        if *fact.event_type() != Named::EVENT_TYPE {
            continue;
        }
        if let Ok(named) = codec::event_payload::<Named>(fact.payload()) {
            names.insert(named.person().entity_id(), named.name().clone());
        }
    }
    names
}
