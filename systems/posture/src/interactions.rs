//! This pack's section of the World's Interaction List (`docs/DECISIONS.md` `ARC-63`, `ARC-65`,
//! `ARC-147`; `configure/posture.yaml`).
//!
//! ```yaml
//! rules:
//!   - { action: sit, actor: guest, place: chapel, effect: forbid }
//! parameters:
//!   - { reach: 750 }
//!   - { place: park, reach: 1000 }
//! consequences:
//!   - { fact: sat-down, audience: participants }
//! ```
//!
//! ```text
//! rules        sit        actor, place; regional
//!              stand is never governed: a list cannot keep a seated person seated
//! parameters   reach      millimetres a person may stand from a sit point, 300 … 1 500, default 750
//! facts        sat-down, stood-up    actor = participant 0, place; place → participants; no biography
//! ```
//!
//! The default is today's constant ([`REACH`](crate::REACH)), so a world that does not configure the
//! section decides exactly as the compiled defaults say.

use mineworld_contracts::{
    Action, ComponentTypeId, EntityId, Event, EventTypeId, Millimetres, PlaceId, Rejection,
    Visibility,
};
use mineworld_kernel::WorldRead;
use mineworld_sdk::interactions::{
    self, ActionDecl, Audience, FactDecl, InteractionSection, Position, Resolved, Role, Roles,
};

use crate::action::Sit;
use crate::event::{SatDown, StoodUp};
use crate::system::PostureSystem;

mineworld_sdk::parameters! {
    /// What a world may choose about sitting.
    pub struct PostureParameters, partial PosturePartial {
        /// Millimetres a person may stand from a sit point and still sit on it.
        reach: u32 = 750, 300 ..= 1_500;
    }
}

/// The roles `sit` and the parameter block may name.
const ROLES: &[Role] = &[Role::Actor, Role::Place];

/// A fact about one person at a place: that person, and the place.
const PERSON_ROLES: &[(Role, Position)] = &[
    (Role::Actor, Position::Participant(0)),
    (Role::Place, Position::Place),
];

const fn fact(fact: EventTypeId) -> FactDecl {
    FactDecl {
        fact,
        roles: PERSON_ROLES,
        default_audience: Audience::Place,
        narrowest: Audience::Participants,
        biography_configurable: false,
    }
}

impl InteractionSection for PostureSystem {
    type Parameters = PostureParameters;
    type Knobs = ();
    const CONFIGURED: EventTypeId = EventTypeId::from_static("posture-interactions-configured");
    const COMPONENT: ComponentTypeId = ComponentTypeId::from_static("posture-interactions");
    const ACTIONS: &'static [ActionDecl] = &[ActionDecl {
        action: Sit::ACTION_TYPE,
        roles: ROLES,
        regional: true,
    }];
    const FACTS: &'static [FactDecl] = &[fact(SatDown::EVENT_TYPE), fact(StoodUp::EVENT_TYPE)];
    const PARAMETER_ROLES: &'static [Role] = ROLES;

    fn encode(resolved: &Resolved<Self>) -> Vec<u8> {
        serde_json::to_vec(resolved).expect("a resolved section encodes")
    }

    fn decode(payload: &[u8]) -> Result<Resolved<Self>, String> {
        serde_json::from_slice(payload).map_err(|error| error.to_string())
    }
}

/// The roles of a request by `actor`.
pub(crate) fn acting(actor: EntityId) -> Roles {
    Roles::new().with(Role::Actor, actor)
}

/// Whether the world's list permits `sit` for `actor` at `place`: the one call `validate` and `offers`
/// both make (`ARC-63` item 8).
pub(crate) fn permits_sit(
    world: &WorldRead<'_>,
    place: PlaceId,
    actor: EntityId,
) -> Result<(), Rejection> {
    interactions::permits::<PostureSystem>(world, place, &Sit::ACTION_TYPE, &acting(actor))
}

/// The reach the world's list gives `actor` at `place`, or the compiled [`REACH`](crate::REACH).
pub(crate) fn reach(world: &WorldRead<'_>, place: PlaceId, actor: EntityId) -> Millimetres {
    let reach = interactions::parameters::<PostureSystem>(world, place, &acting(actor)).reach;
    Millimetres::new(i32::try_from(reach).expect("a reach within the section's bound"))
}

/// The audience `fact` is stated with at `place` for `actor`: the place, or the list's narrowing of it
/// (`ARC-65`).
pub(crate) fn audience(
    world: &WorldRead<'_>,
    place: PlaceId,
    fact: &EventTypeId,
    actor: EntityId,
) -> Visibility {
    interactions::consequence::<PostureSystem>(
        world,
        Some(place),
        fact,
        &acting(actor),
        Visibility::Place(place),
    )
    .visibility
}
