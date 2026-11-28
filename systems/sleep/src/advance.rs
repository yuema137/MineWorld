//! Sleep as an advance stop (`docs/DECISIONS.md` `ARC-374`): what a host reads to run a sleeping
//! player's night faster than wall time, and when to stop.
//!
//! An occurrence is one sleep, identified by its `sleep` Process. It holds while the person still has
//! [`Asleep`] for that Process — whichever of the four ways a sleep ends removes it — and its horizon is
//! the alarm. Read-only: this module changes nothing, and the host learns nothing about sleep from it
//! beyond these answers.

use mineworld_contracts::{EntityId, SystemId, WorldTime};
use mineworld_kernel::{SystemIdentity, WorldRead};
use mineworld_sdk::advance::{AdvanceStop, StopIdentity};

use crate::component::Asleep;
use crate::system::SleepSystem;

impl SleepSystem {
    /// The sleep `person` is in, if its Process is `occurrence`.
    fn asleep_in<'a>(
        world: &WorldRead<'a>,
        person: EntityId,
        occurrence: StopIdentity,
    ) -> Option<&'a Asleep> {
        world
            .component::<Asleep>(person)
            .filter(|asleep| asleep.process().raw() == occurrence.raw())
    }
}

impl AdvanceStop for SleepSystem {
    fn name(&self) -> &'static str {
        "sleep"
    }

    fn system(&self) -> SystemId {
        <Self as SystemIdentity>::ID
    }

    fn identify(&self, world: &WorldRead<'_>, person: EntityId) -> Option<StopIdentity> {
        world
            .component::<Asleep>(person)
            .map(|asleep| StopIdentity::from_raw(asleep.process().raw()))
    }

    fn horizon(
        &self,
        world: &WorldRead<'_>,
        person: EntityId,
        occurrence: StopIdentity,
    ) -> Option<WorldTime> {
        Self::asleep_in(world, person, occurrence).map(Asleep::alarm)
    }

    fn holds(&self, world: &WorldRead<'_>, person: EntityId, occurrence: StopIdentity) -> bool {
        Self::asleep_in(world, person, occurrence).is_some()
    }
}
