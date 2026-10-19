//! The two actions this pack provides, and what each requires of space.
//!
//! Neither is directed at anybody: the intent's target is [`None`]. `sit` names its sit point in the
//! payload, because a sit point is data of a place, not an entity a client can point at (`ARC-147`).

use mineworld_contracts::{
    Action, ActionTypeId, Millimetres, PlaceId, SpatialRequirement, SystemId,
};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::key::SitPointKey;
use crate::system::PostureSystem;

/// How far a person may stand from a sit point and still sit on it, by default: 750 mm, a step and a
/// turn (`step-34-posture.md` Q-S34-8). The compiled default of the section's `reach`.
pub const REACH: Millimetres = Millimetres::new(750);

/// Sit down on the sit point `point` of the place the actor is in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sit {
    point: SitPointKey,
}

impl Action for Sit {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("sit");
    const OWNER: SystemId = PostureSystem::ID;
}

impl Sit {
    /// Sit on `point`.
    pub const fn new(point: SitPointKey) -> Self {
        Self { point }
    }

    /// Which sit point.
    pub const fn point(&self) -> &SitPointKey {
        &self.point
    }
}

/// Stand up from the sit point the actor is seated on.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stand {}

impl Action for Stand {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("stand");
    const OWNER: SystemId = PostureSystem::ID;
}

/// `sit`: the actor is in `place`, the place whose sit points are offered. Every other condition —
/// reach, occupancy, walking — is this pack's own judgement and travels as the offer's refusal, so the
/// requirement a client is shown is exactly what perception can evaluate (BC-5 of the PR design:
/// `SamePlaceAsActor` needs a target entity, and a sit point is not one).
pub const fn sit_requirement(place: PlaceId) -> SpatialRequirement {
    SpatialRequirement::at_place(place)
}

/// `stand`: nothing of space.
pub const fn stand_requirement() -> SpatialRequirement {
    SpatialRequirement::NONE
}
