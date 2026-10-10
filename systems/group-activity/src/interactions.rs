//! This pack's section of the World's Interaction List (`docs/DECISIONS.md` `ARC-63`;
//! `configure/group-activity.yaml`).
//!
//! ```yaml
//! parameters:
//!   - { invitation_lifetime: 3600 }                # the world's
//!   - { target: regular, invitation_lifetime: 600 }
//! ```
//!
//! In S17's PR IL-b the section holds one parameter: `invitation_lifetime`, how long an invitation can
//! be answered — whole seconds, 1 … 86 400, default 1 800 ([`INVITATION_LIFETIME`]). It is looked up
//! when an `invited` fact is reduced, with the inviter as `actor`, the invitee as `target` and the
//! fact's place, and the invitation records the instant it lapses (`Invitation::until`), which is
//! what the pack and every reader of the invitation judge by. The section declares no action and no
//! fact yet. A world that does not configure the section decides exactly as before.

use mineworld_contracts::{ComponentTypeId, EventTypeId};
use mineworld_sdk::interactions::{InteractionSection, Resolved, Role};

#[cfg(test)]
use crate::component::INVITATION_LIFETIME;
use crate::system::GroupActivitySystem;

mineworld_sdk::parameters! {
    /// What a world may choose about group activities.
    pub struct GroupActivityParameters, partial GroupActivityPartial {
        /// Seconds an invitation can be answered for, from the instant it was made.
        invitation_lifetime: u32 = 1_800, 1 ..= 86_400;
    }
}

impl InteractionSection for GroupActivitySystem {
    type Parameters = GroupActivityParameters;
    type Knobs = ();
    const CONFIGURED: EventTypeId =
        EventTypeId::from_static("group-activity-interactions-configured");
    const COMPONENT: ComponentTypeId = ComponentTypeId::from_static("group-activity-interactions");
    const PARAMETER_ROLES: &'static [Role] = &[Role::Actor, Role::Target, Role::Place];

    fn encode(resolved: &Resolved<Self>) -> Vec<u8> {
        serde_json::to_vec(resolved).expect("a resolved section encodes")
    }

    fn decode(payload: &[u8]) -> Result<Resolved<Self>, String> {
        serde_json::from_slice(payload).map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use mineworld_kernel::System;

    use super::*;

    /// The section's `default` is pinned to the pack's version (step-18 §4.10 item 3).
    #[test]
    fn the_default_section_is_pinned_to_the_version() {
        assert_eq!(GroupActivitySystem::VERSION.get(), 2);
        assert_eq!(
            i64::from(GroupActivityParameters::default().invitation_lifetime),
            INVITATION_LIFETIME.seconds()
        );
        assert_eq!(INVITATION_LIFETIME.seconds(), 1_800);
        assert_eq!(
            <GroupActivitySystem as InteractionSection>::CONFIGURED.as_str(),
            "group-activity-interactions-configured"
        );
        assert_eq!(
            <GroupActivitySystem as InteractionSection>::COMPONENT.as_str(),
            "group-activity-interactions"
        );
    }
}
