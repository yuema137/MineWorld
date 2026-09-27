//! The action this pack provides: coming to be somewhere.

use mineworld_contracts::{Action, ActionTypeId, Location, SpatialRequirement, SystemId};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::system::PresenceSystem;

/// A person comes to be at a location.
///
/// The minimum that makes location *exist* in a world at all: a component is written only by its
/// owning system, so without an action there is no way for a World Pack, a controller or a client
/// to say where anybody is. It is one action rather than two — no separate "enter a place" and
/// "move within a place" — because a [`Location`] already spans both scales, and splitting them
/// would put the same fact in two contracts.
///
/// **What this is not.** It is not movement. Movement is a `MoveIntent`, a travel `Process` that
/// takes simulated time, and authoritative state that changes as it runs
/// (`ENGINEERING_RULES.md` §6), and none of those exist yet — the vertical slice this pack belongs
/// to excludes travel deliberately. `arrive` records an arrival that something else decided; a
/// world that installs a movement system lets that system drive it, and this pack keeps owning the
/// state either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Arrive {
    location: Location,
}

impl Action for Arrive {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("arrive");
    const OWNER: SystemId = PresenceSystem::ID;
}

impl Arrive {
    /// Asks to be at `location`.
    pub const fn new(location: Location) -> Self {
        Self { location }
    }

    /// Where the actor is asking to be.
    pub const fn location(&self) -> Location {
        self.location
    }
}

/// What `arrive` requires of space: nothing.
///
/// Declared rather than omitted, because `ENGINEERING_RULES.md` §7 asks an action to state that it
/// needs no proximity just as explicitly as it states that it does — and because the declaration
/// is what a client is *told*, inside an [`Affordance`](mineworld_contracts::Affordance), so that
/// it can render the action without implementing a rule.
///
/// It is [`SpatialRequirement::NONE`] for an honest reason and not a permissive one: in a world
/// with no movement system there is nothing to be walked, so a distance requirement here would
/// refuse the only mechanism a World Pack has for placing its people. When this pack does declare
/// a requirement for arriving — a door that must be open, a place that must have room — it is
/// evaluated in [`PresenceSystem::validate`](crate::PresenceSystem) with
/// [`SpatialRequirement::evaluate`], the same evaluator every other pack's requirement goes
/// through, and never in a client.
pub const fn arrive_requirement() -> SpatialRequirement {
    SpatialRequirement::NONE
}
