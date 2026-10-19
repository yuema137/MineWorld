//! The state this pack owns: where a person may sit, and who is seated.

use mineworld_contracts::WorldTime;
use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::key::SitPointKey;
use crate::point::SitPoints;
use crate::system::PostureSystem;

/// Where a person is sitting: on the sit point named `point`, since `since`.
///
/// Held on a Person. A person who stands holds no such row (`ARC-147`, SD-a-2): standing is the absence
/// of the row, so a person who never sits holds nothing, and a world without this pack is byte-identical.
/// The place is not stored here: a seated person is at their point's place, which is their
/// [`Presence`](mineworld_presence::Presence) place (I-S34-2). Serialized as an object, `{ "seated": {
/// "point": …, "since": … } }`, because a client reads a component payload as an object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Posture {
    /// Seated on `point`, since `since`.
    Seated {
        /// The sit point, in the place the person is in.
        point: SitPointKey,
        /// The instant the person sat down.
        since: WorldTime,
    },
}

impl Posture {
    /// Seated on `point` since `since`.
    pub const fn seated(point: SitPointKey, since: WorldTime) -> Self {
        Self::Seated { point, since }
    }

    /// The sit point the person is on.
    pub const fn point(&self) -> &SitPointKey {
        match self {
            Self::Seated { point, .. } => point,
        }
    }

    /// When the person sat down.
    pub const fn since(&self) -> WorldTime {
        match self {
            Self::Seated { since, .. } => *since,
        }
    }
}

owned_component! {
    component = Posture,
    owner = PostureSystem,
    component_type = "posture-state",
    schema_version = 1,
}

owned_component! {
    component = SitPoints,
    owner = PostureSystem,
    component_type = "sit-points",
    schema_version = 1,
}
