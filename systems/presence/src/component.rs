//! The state this pack owns: where a person is.

use mineworld_contracts::Location;
use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::system::PresenceSystem;

/// Where a person is, as the world's authoritative record of it.
///
/// One field, and it is a [`Location`] — a place, with an optional position inside it
/// (`ENGINEERING_RULES.md` §5). That is what lets a headless world and a 2D client say "in the
/// café" while a 3D client says "1.2 m from the counter", with no second component and no second
/// contract: the refinement is absent rather than approximated.
///
/// Written only by [`PresenceSystem`], and only while reducing an
/// [`Arrived`](crate::event::Arrived) — so the component is a projection of the event log and a
/// replay rebuilds it exactly (`AC-12`). Read by anyone: a system that needs to know whether two
/// people are close enough to interact reads this and evaluates its own declared
/// [`SpatialRequirement`](mineworld_contracts::SpatialRequirement) against it.
///
/// A person with no `Presence` is not "nowhere" as a state of the world; they are someone this
/// pack has not been told about. Perception treats that as knowing nothing rather than as an
/// error, which is what lets a world install this pack after entities already exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Presence {
    location: Location,
}

owned_component! {
    component = Presence,
    owner = PresenceSystem,
    component_type = "presence",
    schema_version = 1,
}

impl Presence {
    /// Records that somebody is at `location`.
    pub const fn at(location: Location) -> Self {
        Self { location }
    }

    /// Where they are.
    pub const fn location(&self) -> Location {
        self.location
    }
}
