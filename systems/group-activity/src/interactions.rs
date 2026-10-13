//! This pack's section of the World's Interaction List (`docs/DECISIONS.md` `ARC-63`, `ARC-65`;
//! `configure/group-activity.yaml`).
//!
//! ```yaml
//! rules:
//!   - { action: invite, actor: noble, target: commoner, effect: forbid }
//!   - { action: accept-invitation, actor: noble, target: commoner, effect: forbid }
//!   - { action: join-group-activity, actor: noble, target: commoner, effect: forbid }
//! parameters:
//!   - { invitation_lifetime: 1800, invite_range: 3000, activity_length: 3600 }
//!   - { place: park, activity_length: 7200 }
//! consequences:
//!   - { fact: group-activity-started, audience: participants }
//!   - { fact: joined-group-activity, actor: servant, biography: off }
//! ```
//!
//! ```text
//! rules        invite                actor = inviter, target = invitee, place; regional
//!              accept-invitation     actor = the one accepting, target = the inviter, place; regional
//!              join-group-activity   actor = joiner, target = the member joined, place; regional
//!              decline-invitation and leave-group-activity are never governed: a list can stop
//!              people coming together, not trap anyone in an invitation or an activity
//! parameters   invitation_lifetime   seconds an invitation can be answered, 1 … 86 400, default 1 800
//!              invite_range          millimetres an invitation reaches, 1 … 100 000, default 3 000
//!              activity_length       seconds an activity runs, 60 … 86 400, default 3 600
//! facts        invited, invitation-accepted, invitation-declined   participants; biography (off)
//!              group-activity-started, -ended    by place only; place → participants; biography (on)
//!              joined-, left-group-activity      actor, place; place → participants; biography (on)
//! ```
//!
//! Every default is today's constant ([`INVITATION_LIFETIME`](crate::INVITATION_LIFETIME),
//! [`INVITE_RANGE`](crate::INVITE_RANGE), [`ACTIVITY_LENGTH`](crate::ACTIVITY_LENGTH)), so a world
//! that does not configure the section decides exactly as before.

use mineworld_contracts::{
    Action, ActionTypeId, ComponentTypeId, EntityId, Event, EventTypeId, PlaceId, Rejection,
    SpatialRequirement, Visibility,
};
use mineworld_kernel::WorldRead;
use mineworld_presence::Offer;
use mineworld_sdk::interactions::{
    self, ActionDecl, Audience, FactDecl, InteractionSection, Position, Resolved, Role, Roles,
};

use crate::action::{
    AcceptInvitation, Invite, JoinGroupActivity, invite_requirement_within, range,
};
use crate::event::{
    GroupActivityEnded, GroupActivityStarted, InvitationAccepted, InvitationDeclined, Invited,
    JoinedGroupActivity, LeftGroupActivity,
};
use crate::system::GroupActivitySystem;

mineworld_sdk::parameters! {
    /// What a world may choose about group activities.
    pub struct GroupActivityParameters, partial GroupActivityPartial {
        /// Seconds an invitation can be answered for, from the instant it was made.
        invitation_lifetime: u32 = 1_800, 1 ..= 86_400;
        /// Millimetres an invitation reaches: how close an inviter must be.
        invite_range: u32 = 3_000, 1 ..= 100_000;
        /// Seconds an activity runs unless it ends sooner.
        activity_length: u32 = 3_600, 60 ..= 86_400;
    }
}

/// The roles each governed action and the parameter block may name.
const ROLES: &[Role] = &[Role::Actor, Role::Target, Role::Place];

/// `invited`: the inviter and the invitee are the envelope's participants, in that order.
const INVITED_ROLES: &[(Role, Position)] = &[
    (Role::Actor, Position::Participant(0)),
    (Role::Target, Position::Participant(1)),
    (Role::Place, Position::Place),
];

/// An answer: its actor is the invitee, who answered, the second participant.
const ANSWER_ROLES: &[(Role, Position)] = &[
    (Role::Actor, Position::Participant(1)),
    (Role::Target, Position::Participant(0)),
    (Role::Place, Position::Place),
];

/// A whole activity's facts: their members list has no fixed positions, so only the place.
const ACTIVITY_ROLES: &[(Role, Position)] = &[(Role::Place, Position::Place)];

/// One person and an activity: that person, and the place.
const MEMBERSHIP_ROLES: &[(Role, Position)] = &[
    (Role::Actor, Position::Participant(0)),
    (Role::Place, Position::Place),
];

const fn governed(action: ActionTypeId) -> ActionDecl {
    ActionDecl {
        action,
        roles: ROLES,
        regional: true,
    }
}

const fn fact(
    fact: EventTypeId,
    roles: &'static [(Role, Position)],
    default_audience: Audience,
) -> FactDecl {
    FactDecl {
        fact,
        roles,
        default_audience,
        narrowest: Audience::Participants,
        biography_configurable: true,
    }
}

impl InteractionSection for GroupActivitySystem {
    type Parameters = GroupActivityParameters;
    type Knobs = ();
    const CONFIGURED: EventTypeId =
        EventTypeId::from_static("group-activity-interactions-configured");
    const COMPONENT: ComponentTypeId = ComponentTypeId::from_static("group-activity-interactions");
    const ACTIONS: &'static [ActionDecl] = &[
        governed(Invite::ACTION_TYPE),
        governed(AcceptInvitation::ACTION_TYPE),
        governed(JoinGroupActivity::ACTION_TYPE),
    ];
    const FACTS: &'static [FactDecl] = &[
        fact(Invited::EVENT_TYPE, INVITED_ROLES, Audience::Participants),
        fact(
            InvitationAccepted::EVENT_TYPE,
            ANSWER_ROLES,
            Audience::Participants,
        ),
        fact(
            InvitationDeclined::EVENT_TYPE,
            ANSWER_ROLES,
            Audience::Participants,
        ),
        fact(
            GroupActivityStarted::EVENT_TYPE,
            ACTIVITY_ROLES,
            Audience::Place,
        ),
        fact(
            GroupActivityEnded::EVENT_TYPE,
            ACTIVITY_ROLES,
            Audience::Place,
        ),
        fact(
            JoinedGroupActivity::EVENT_TYPE,
            MEMBERSHIP_ROLES,
            Audience::Place,
        ),
        fact(
            LeftGroupActivity::EVENT_TYPE,
            MEMBERSHIP_ROLES,
            Audience::Place,
        ),
    ];
    const PARAMETER_ROLES: &'static [Role] = ROLES;

    fn encode(resolved: &Resolved<Self>) -> Vec<u8> {
        serde_json::to_vec(resolved).expect("a resolved section encodes")
    }

    fn decode(payload: &[u8]) -> Result<Resolved<Self>, String> {
        serde_json::from_slice(payload).map_err(|error| error.to_string())
    }
}

/// The roles of a request between two people: who acts, and whom it is directed at.
pub(crate) fn pair(actor: EntityId, target: EntityId) -> Roles {
    Roles::new()
        .with(Role::Actor, actor)
        .with(Role::Target, target)
}

/// Whether the world's list permits `action` between these two at `place` — the one call `validate`
/// and `offers` both make (`ARC-63` item 8).
pub(crate) fn permits(
    world: &WorldRead<'_>,
    place: PlaceId,
    action: &ActionTypeId,
    roles: &Roles,
) -> Result<(), Rejection> {
    interactions::permits::<GroupActivitySystem>(world, place, action, roles)
}

/// What the world's list says about an invitation between these two at `place`: whether it is
/// permitted, and what it requires of space there. One function for `validate` and `offers`, so the
/// requirement shown is the one enforced. With nothing configured: permitted, and
/// [`invite_requirement`](crate::invite_requirement).
pub(crate) fn invite_terms(
    world: &WorldRead<'_>,
    place: PlaceId,
    roles: &Roles,
) -> (Result<(), Rejection>, SpatialRequirement) {
    let permitted = permits(world, place, &Invite::ACTION_TYPE, roles);
    let reach = interactions::parameters::<GroupActivitySystem>(world, place, roles).invite_range;
    (permitted, invite_requirement_within(range(reach)))
}

/// `offer`, refused for the list's reason when `permitted` is one.
pub(crate) fn answered(offer: Offer, permitted: Result<(), Rejection>) -> Offer {
    match permitted {
        Ok(()) => offer,
        Err(reason) => offer.refused(reason),
    }
}

/// The audience this pack states `fact` with when no list narrows it, from the fact's `FactDecl` — the
/// one place each owner default is written (SD-IE-5).
pub(crate) fn owner_default(fact: &EventTypeId, place: PlaceId) -> Visibility {
    let declared = <GroupActivitySystem as InteractionSection>::FACTS
        .iter()
        .find(|declared| declared.fact == *fact)
        .expect("group-activity states only the facts it declares");
    match declared.default_audience {
        Audience::Public => Visibility::Public,
        Audience::Place => Visibility::Place(place),
        Audience::Participants => Visibility::Participants,
    }
}

/// The audience `fact` is stated with at `place`, for these roles: its owner default, or the list's
/// narrowing of it (`ARC-65`).
pub(crate) fn audience(
    world: &WorldRead<'_>,
    place: PlaceId,
    fact: &EventTypeId,
    roles: &Roles,
) -> Visibility {
    interactions::consequence::<GroupActivitySystem>(
        world,
        Some(place),
        fact,
        roles,
        owner_default(fact, place),
    )
    .visibility
}

#[cfg(test)]
mod tests {
    use mineworld_contracts::EntityType;
    use mineworld_kernel::System;

    use super::*;
    use crate::{ACTIVITY_LENGTH, INVITATION_LIFETIME, INVITE_RANGE};

    /// The section's `default` is pinned to the pack's version (step-18 §4.10 item 3).
    #[test]
    fn the_default_section_is_pinned_to_the_version() {
        assert_eq!(GroupActivitySystem::VERSION.get(), 3);
        let defaults = GroupActivityParameters::default();
        assert_eq!(
            i64::from(defaults.invitation_lifetime),
            INVITATION_LIFETIME.seconds()
        );
        assert_eq!(INVITATION_LIFETIME.seconds(), 1_800);
        assert_eq!(
            i64::from(defaults.invite_range),
            i64::from(INVITE_RANGE.value())
        );
        assert_eq!(INVITE_RANGE.value(), 3_000);
        assert_eq!(
            i64::from(defaults.activity_length),
            ACTIVITY_LENGTH.seconds()
        );
        assert_eq!(ACTIVITY_LENGTH.seconds(), 3_600);
        assert_eq!(
            <GroupActivitySystem as InteractionSection>::CONFIGURED.as_str(),
            "group-activity-interactions-configured"
        );
        assert_eq!(
            <GroupActivitySystem as InteractionSection>::COMPONENT.as_str(),
            "group-activity-interactions"
        );
    }

    /// Unconfigured, each emission's audience is its `FactDecl`'s default, and that default is the
    /// audience the pack stated before it had a section (IL-I1, SD-IE-5): the invitation facts are
    /// the two people's, the activity facts the place's.
    #[test]
    fn each_fact_is_stated_with_its_declared_default_which_is_todays_audience() {
        let place = PlaceId::new(EntityId::from_raw(9), EntityType::Place).expect("a place");
        for fact in [
            Invited::EVENT_TYPE,
            InvitationAccepted::EVENT_TYPE,
            InvitationDeclined::EVENT_TYPE,
        ] {
            assert_eq!(
                owner_default(&fact, place),
                Visibility::Participants,
                "{fact}"
            );
        }
        for fact in [
            GroupActivityStarted::EVENT_TYPE,
            GroupActivityEnded::EVENT_TYPE,
            JoinedGroupActivity::EVENT_TYPE,
            LeftGroupActivity::EVENT_TYPE,
        ] {
            assert_eq!(
                owner_default(&fact, place),
                Visibility::Place(place),
                "{fact}"
            );
        }
    }
}
