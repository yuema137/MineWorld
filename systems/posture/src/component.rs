//! The state this pack owns: where a person may sit, and who is seated.

use mineworld_contracts::{PlaceId, WorldTime};
use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::key::SitPointKey;
use crate::point::SitPoints;
use crate::system::PostureSystem;

/// Where a person is sitting: on the sit point named `point` of `place`, since `since`.
///
/// Held on a Person. A person who stands holds no such row (`ARC-147`, SD-a-2): standing is the absence
/// of the row, so a person who never sits holds nothing, and a world without this pack is byte-identical.
///
/// The place is stored beside the key (BC-6 of the PR design): a key is unique only within its place,
/// and the auto-stand reaction to presence's `arrived` runs after presence has already written the new
/// location, so the row is the only record of which place the person sat in. It always equals the
/// person's [`Presence`](mineworld_presence::Presence) place while the row exists (I-S34-2), because any
/// arrival elsewhere stands them up in the same dispatch.
///
/// Serialized as an object, `{ "seated": { "place": …, "point": …, "since": … } }`, because a client
/// reads a component payload as an object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Posture {
    /// Seated on `point` of `place`, since `since`.
    Seated {
        /// The place the sit point belongs to.
        place: PlaceId,
        /// The sit point.
        point: SitPointKey,
        /// The instant the person sat down.
        since: WorldTime,
    },
}

impl Posture {
    /// Seated on `point` of `place` since `since`.
    pub const fn seated(place: PlaceId, point: SitPointKey, since: WorldTime) -> Self {
        Self::Seated {
            place,
            point,
            since,
        }
    }

    /// The place of the sit point the person is on.
    pub const fn place(&self) -> PlaceId {
        match self {
            Self::Seated { place, .. } => *place,
        }
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
