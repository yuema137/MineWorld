//! An affordance must not be able to say it is available and also carry a reason it is not. A
//! client shown both halves would have to decide which to believe, so the only ways in are the two
//! constructors that make the pair agree.

use mineworld_contracts::{ActionTypeId, Affordance, EntityId, Rejection, SpatialRequirement};

fn main() {
    let _offered = Affordance {
        action_type: ActionTypeId::from_static("talk"),
        target: Some(EntityId::from_raw(42)),
        available: true,
        unavailable_reason: Some(Rejection::TooFarAway),
        requirement: SpatialRequirement::NONE,
    };
}
