//! The installable system: what it declares, the state it reduces, and what it may do.

use mineworld_contracts::{Event, EventEnvelope, EventTypeId, SystemId};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    WorldView,
};
use mineworld_presence::PerceptionProvider;
use mineworld_sdk::SystemPack;

use crate::codec;
use crate::component::Posture;
use crate::event::PlaceSitPoints;
use crate::point::SitPoints;

/// Which of this pack's facts belong in a person's objective biography (`ARC-29`): none. Sitting down is
/// not a life event, and the Interaction List cannot make it one (`ARC-147`).
pub const BIOGRAPHICAL: &[EventTypeId] = &[];

/// Where people may sit, and who is seated.
///
/// A unit struct: its state is the components it owns, held in the world (`INV-7`).
#[derive(Default)]
pub struct PostureSystem;

impl SystemIdentity for PostureSystem {
    const ID: SystemId = SystemId::from_static("posture");
}

/// What the build needs to know about this pack beyond [`System`] (`DECISIONS.md` `ARC-33`): its
/// biographical facts (none), and the `sit_points:` section of a place's file, which its `AuthoredSection`
/// impl (`src/section.rs`) describes.
impl SystemPack for PostureSystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    const BIOGRAPHICAL: &'static [EventTypeId] = BIOGRAPHICAL;
    mineworld_sdk::owns_section!();
}

impl System for PostureSystem {
    /// Version 1: the first release of the pack (`ARC-25`).
    const VERSION: SystemVersion = SystemVersion::new(1);

    /// Owns sit points and posture. Depends on nothing it reads at run time: presence and movement are
    /// consulted through their public functions, which need no declaration.
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<SitPoints>()
            .owning::<Posture>()
            .emitting::<PlaceSitPoints>()
            .subscribing_to::<PlaceSitPoints>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<SitPoints>()?;
        tables.component::<Posture>()
    }

    /// Reduces `place-sit-points` into the place's [`SitPoints`]: the only write of genesis this pack
    /// makes. The list was checked when it was decoded, so there is nothing to refuse here.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if *event.event_type() == PlaceSitPoints::EVENT_TYPE {
            let stated: PlaceSitPoints = codec::event_payload(event.payload())?;
            world.insert(stated.place().entity_id(), stated.points().clone())?;
        }
        Ok(Vec::new())
    }
}

/// Perception is added in a later step of this pack; until then it discloses and offers nothing.
impl PerceptionProvider for PostureSystem {}
