//! The installable system: what it declares, the one fact it reduces, and what it discloses.

use std::collections::BTreeMap;

use mineworld_contracts::{
    ComponentRecord, EntityId, EntityType, Event, EventEnvelope, EventTypeId, SystemId,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    WorldRead, WorldView,
};
use mineworld_movement::Passages;
use mineworld_presence::PerceptionProvider;
use mineworld_sdk::SystemPack;
use serde_json::Value;

use crate::codec;
use crate::component::DisplayName;
use crate::door::DoorNames;
use crate::event::{Named, PlaceNamed};
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
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    const BIOGRAPHICAL: &'static [EventTypeId] = BIOGRAPHICAL;
    mineworld_sdk::owns_section!();
}

impl System for NamingSystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    /// No dependency, no action, no process: a name is stated at genesis and reduced here. Door names
    /// read movement's public `Passages` when the world has them, and need no declared dependency:
    /// a world without movement has no doorways and so no door names (`ARC-363`).
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<DisplayName>()
            .owning::<DoorNames>()
            .emitting::<Named>()
            .emitting::<PlaceNamed>()
            .subscribing_to::<Named>()
            .subscribing_to::<PlaceNamed>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<DisplayName>()?;
        tables.component::<DoorNames>()
    }

    /// Reduces `named` and `place-named` into the subject's [`DisplayName`] — the only write this pack
    /// makes.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if *event.event_type() == Named::EVENT_TYPE {
            let named: Named = codec::event_payload(event.payload())?;
            world.insert(
                named.person().entity_id(),
                DisplayName::with_forms(named.name().clone(), named.forms().clone()),
            )?;
        } else if *event.event_type() == PlaceNamed::EVENT_TYPE {
            let named: PlaceNamed = codec::event_payload(event.payload())?;
            world.insert(
                named.place().entity_id(),
                DisplayName::with_forms(named.name().clone(), named.forms().clone()),
            )?;
        }
        Ok(Vec::new())
    }
}

impl PerceptionProvider for NamingSystem {
    /// The subject's name, to **every** observer who perceives them, oneself included: names are
    /// public in MVP-0 (`ARC-31`). Perception asks only about entities the observer already perceives,
    /// so nobody learns the name of somebody they cannot see — and a person perceives no place but the
    /// one they stand in (`ARC-26`), so a place's name reaches exactly those who see its tags.
    ///
    /// About a place, also its [`DoorNames`]: the name of each place a doorway of it leads to, the
    /// names beside movement's `to_tags` on the same doorways, to the same observers (`ARC-363`).
    fn discloses(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        subject: EntityId,
    ) -> Vec<ComponentRecord<Value>> {
        let _ = observer;
        let mut records: Vec<ComponentRecord<Value>> = world
            .component::<DisplayName>(subject)
            .map(|name| ComponentRecord::new::<DisplayName>(subject, codec::to_value(name)))
            .into_iter()
            .collect();
        let is_place = world
            .entity(subject)
            .is_some_and(|record| record.entity_type() == EntityType::Place);
        if is_place
            && let Some(doors) = world
                .component::<Passages>(subject)
                .and_then(|passages| DoorNames::of(world, passages))
        {
            records.push(ComponentRecord::new::<DoorNames>(
                subject,
                codec::to_value(&doors),
            ));
        }
        records
    }
}

/// Every person's and place's name according to `facts` — the latest `named` or `place-named` per
/// subject.
///
/// This pack's own projection of its own facts, published for readers of a saved history
/// (`mineworld biography`), so that no reader decodes this pack's payload with a copy of its shape
/// (`step-09-social.md` Q6's condition, applied to readers). A fact of any other type is skipped.
pub fn names_in(facts: &[EventEnvelope]) -> BTreeMap<EntityId, Name> {
    let mut names = BTreeMap::new();
    for fact in facts {
        if *fact.event_type() == Named::EVENT_TYPE {
            if let Ok(named) = codec::event_payload::<Named>(fact.payload()) {
                names.insert(named.person().entity_id(), named.name().clone());
            }
        } else if *fact.event_type() == PlaceNamed::EVENT_TYPE
            && let Ok(named) = codec::event_payload::<PlaceNamed>(fact.payload())
        {
            names.insert(named.place().entity_id(), named.name().clone());
        }
    }
    names
}
