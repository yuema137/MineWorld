//! `survey`: copy the tracked asks on display where the surveyor stands into its book.

use mineworld_contracts::{Action, ActionTypeId, PlaceId, SpatialRequirement, SystemId};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::system::PriceSurveySystem;

/// Survey the asks on display here. No target, no fields.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Survey {}

impl Action for Survey {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("survey");
    const OWNER: SystemId = PriceSurveySystem::ID;
}

/// What `survey` requires of space: the surveyor in `place`, the place it stands in (SD-S31c-4;
/// `SamePlaceAsActor` would need a target, F-S31c-1).
pub const fn survey_requirement(place: PlaceId) -> SpatialRequirement {
    SpatialRequirement::at_place(place)
}
