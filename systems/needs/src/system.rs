//! The installable system: what it declares and the facts it reduces.

use mineworld_contracts::{Event, EventEnvelope, SystemId};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    WorldView,
};
use mineworld_sdk::SystemPack;

use crate::codec;
use crate::component::{Needs, Nourishment};
use crate::event::{NeedsAssigned, NourishmentAssigned};

/// Hunger and tiredness.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NeedsSystem;

impl SystemIdentity for NeedsSystem {
    const ID: SystemId = SystemId::from_static("needs");
}

/// What the build needs to know about this pack beyond [`System`] (`DECISIONS.md` `ARC-33`): the
/// `needs:` section it owns (`ARC-31`). No fact is biographical.
impl SystemPack for NeedsSystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    mineworld_sdk::owns_section!();
}

impl System for NeedsSystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    /// Depends on nothing in the registry: it hears inventory's and sleep's facts through their
    /// published types (`ARC-28`).
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<Needs>()
            .owning::<Nourishment>()
            .emitting::<NeedsAssigned>()
            .emitting::<NourishmentAssigned>()
            .subscribing_to::<NeedsAssigned>()
            .subscribing_to::<NourishmentAssigned>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Needs>()?;
        tables.component::<Nourishment>()
    }

    /// The genesis facts: `needs-assigned` inserts [`Needs`] at the fact's instant, awake;
    /// `nourishment-assigned` inserts [`Nourishment`] on the item kind.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let kind = event.event_type();
        if *kind == NeedsAssigned::EVENT_TYPE {
            let assigned: NeedsAssigned = codec::event_payload(event.payload())?;
            let levels = assigned.levels();
            world.insert(
                assigned.person().entity_id(),
                Needs::assigned(levels.hunger, levels.tiredness, event.at()),
            )?;
        } else if *kind == NourishmentAssigned::EVENT_TYPE {
            let assigned: NourishmentAssigned = codec::event_payload(event.payload())?;
            world.insert(
                assigned.item().entity_id(),
                Nourishment::new(assigned.hunger()),
            )?;
        }
        Ok(Vec::new())
    }
}

/// Discloses nothing yet; offers nothing (a need is a condition, not an action).
impl mineworld_presence::PerceptionProvider for NeedsSystem {}
