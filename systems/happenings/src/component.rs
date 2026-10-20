//! The two records this pack discloses (design §5.7). They are declared component types so that
//! presence keeps them (`owned_by_an_enabled_system`), but no entity carries either: the state lives in
//! the Processes, and these are how it is shown, on the place an observer is in (calendar's and
//! weather's precedent). Neither says anything a fact did not already make public.

use mineworld_contracts::WorldTime;
use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::host::HappeningRef;
use crate::kind::Title;
use crate::system::HappeningsSystem;

/// One happening as a record shows it: which it is, what it is called, and when it ends.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShownHappening {
    /// Its identity: kind, place, first instant.
    pub happening: HappeningRef,
    /// Its kind's title, if the kind has one.
    pub title: Option<Title>,
    /// The instant it ends.
    pub until: WorldTime,
}

/// `happenings-here`: the happenings running at this place now, ordered by start, then kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HappeningsHere(pub Vec<ShownHappening>);

owned_component! {
    component = HappeningsHere,
    owner = HappeningsSystem,
    component_type = "happenings-here",
    schema_version = 1,
}

/// `happenings-announced`: the happenings announced and not yet begun anywhere in the world — public
/// news — ordered by start, then place, then kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HappeningsAnnounced(pub Vec<ShownHappening>);

owned_component! {
    component = HappeningsAnnounced,
    owner = HappeningsSystem,
    component_type = "happenings-announced",
    schema_version = 1,
}
