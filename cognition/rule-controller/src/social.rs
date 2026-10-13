//! The paced rule's social initiative: answering invitations, inviting, joining and leaving
//! (`step-09-social.md` SD-15, §4.2.2).
//!
//! Without this a headless run never invites anybody, and "zero group activities" would be the clean,
//! confident, wrong number `ARC-23` warns about: the systems act only when somebody acts, and headless,
//! only this controller acts.
//!
//! Everything here reads the observation and nothing else (`ARC-27`, I-3):
//!
//! ```text
//! my own Invitations       disclosed to me by group-activity: who invited me, to what, and when
//! my own Participation     disclosed to me: whether I am part of an activity
//! the affordances          the server's verdict for invite / accept / decline / join / leave
//! ```
//!
//! Whether a person may be invited or joined is never worked out here — the affordance says so
//! (`ENGINEERING_RULES.md` §8). The one thing the controller judges for itself is whether an
//! invitation is still open, because an offer cannot see the clock (`step-09-social.md` C2, D-B4): it
//! reads the instant the invitation itself says it lapses (`Invitation::until`, the world's lifetime
//! since S17's PR IL-b), never a lifetime of its own.

use mineworld_contracts::{
    Action, ActionRecord, ActionRequest, Component, EntityId, Observation, PerceivedEntity,
};
use mineworld_group_activity::{
    AcceptInvitation, ActivityKind, DeclineInvitation, Invitations, Invite, JoinGroupActivity,
    LeaveGroupActivity, Participation,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::paced::Draw;

/// Out of 100: how often an invitation the controller may accept is accepted; otherwise declined.
const ACCEPTS: u64 = 70;
/// Out of 100, at a consult while part of an activity: how often the person leaves it.
const LEAVES_ACTIVITY: u64 = 10;
/// Out of 100, at a consult while part of nothing: invite somebody below this, join somebody below the
/// next. The rest falls through to the walking-and-talking scheme, unchanged.
const INVITES_BELOW: u64 = 10;
const JOINS_BELOW: u64 = 18;
/// What a rule proposes doing. A fixed set, because a rule does not invent pastimes.
const KINDS: [&str; 3] = ["coffee", "chat", "walk"];

/// The draw indices this module uses: new ones, independent of the walking scheme's 0–6, so an
/// observation with no group-activity in it decides exactly what it decided before.
const ANSWER_DRAW: u64 = 8;
const INITIATIVE_DRAW: u64 = 9;
const INVITEE_DRAW: u64 = 10;
const KIND_DRAW: u64 = 11;
const JOINED_DRAW: u64 = 12;

/// Whether this observer is part of an activity, by its own disclosed `Participation`.
pub(crate) fn in_activity(observation: &Observation<Value>) -> bool {
    observation
        .entity(observation.observer())
        .and_then(disclosed::<Participation>)
        .is_some()
}

/// The answer to the oldest-by-inviter open invitation the server says I may accept, if any.
pub(crate) fn answer_invitation(
    observation: &Observation<Value>,
    draw: &Draw,
) -> Option<ActionRequest> {
    let me = observation.observer();
    let held: Invitations = disclosed(observation.entity(me)?)?;
    let now = observation.at();
    let invitation = held.pending().iter().find(|invitation| {
        let inviter = invitation.from().entity_id();
        // The invitation states when it lapses — the world's lifetime, not a number this controller
        // was compiled with (S17's PR IL-b, F-IB-3).
        invitation.is_open_at(now) && available::<AcceptInvitation>(observation, Some(inviter))
    })?;
    let inviter = invitation.from().entity_id();
    if draw.below(100, ANSWER_DRAW) < ACCEPTS {
        Some(request(me, &AcceptInvitation {}).with_target(inviter))
    } else if available::<DeclineInvitation>(observation, Some(inviter)) {
        Some(request(me, &DeclineInvitation {}).with_target(inviter))
    } else {
        None
    }
}

/// Leaving one's activity, sometimes — or nothing, and the walking scheme carries on.
pub(crate) fn maybe_leave(observation: &Observation<Value>, draw: &Draw) -> Option<ActionRequest> {
    (draw.below(100, INITIATIVE_DRAW) < LEAVES_ACTIVITY
        && available::<LeaveGroupActivity>(observation, None))
    .then(|| request(observation.observer(), &LeaveGroupActivity {}))
}

/// Inviting somebody, or joining what somebody here is doing — or nothing.
pub(crate) fn initiative(observation: &Observation<Value>, draw: &Draw) -> Option<ActionRequest> {
    let me = observation.observer();
    let roll = draw.below(100, INITIATIVE_DRAW);
    if roll < INVITES_BELOW {
        let invitable = people_offering::<Invite>(observation);
        let invitee = *pick(&invitable, draw, INVITEE_DRAW)?;
        let kind = KINDS[usize::try_from(draw.below(KINDS.len() as u64, KIND_DRAW)).ok()?];
        let kind = ActivityKind::new(kind).ok()?;
        return Some(request(me, &Invite::new(kind)).with_target(invitee));
    }
    if roll < JOINS_BELOW {
        let joinable = people_offering::<JoinGroupActivity>(observation);
        let member = *pick(&joinable, draw, JOINED_DRAW)?;
        return Some(request(me, &JoinGroupActivity {}).with_target(member));
    }
    None
}

/// Everybody the server says `A` is available against right now, in the order the observation lists.
fn people_offering<A: Action>(observation: &Observation<Value>) -> Vec<EntityId> {
    observation
        .entities()
        .iter()
        .map(PerceivedEntity::id)
        .filter(|entity| *entity != observation.observer())
        .filter(|entity| available::<A>(observation, Some(*entity)))
        .collect()
}

/// Whether the server offers `A` against `target` and says it is possible now.
fn available<A: Action>(observation: &Observation<Value>, target: Option<EntityId>) -> bool {
    observation.affordances().iter().any(|affordance| {
        *affordance.action_type() == A::ACTION_TYPE
            && affordance.target() == target
            && affordance.is_available()
    })
}

/// A component this entity's record discloses, decoded with its owner's type.
fn disclosed<C: Component + DeserializeOwned>(entity: &PerceivedEntity<Value>) -> Option<C> {
    let record = entity
        .components()
        .iter()
        .find(|record| *record.component_type() == C::COMPONENT_TYPE)?;
    serde_json::from_value(record.payload_for::<C>().ok()?.clone()).ok()
}

fn pick<'a>(choices: &'a [EntityId], draw: &Draw, n: u64) -> Option<&'a EntityId> {
    if choices.is_empty() {
        return None;
    }
    choices.get(usize::try_from(draw.below(choices.len() as u64, n)).ok()?)
}

fn request<A: Action + Serialize>(actor: EntityId, action: &A) -> ActionRequest {
    ActionRequest::new(
        actor,
        ActionRecord::new::<A>(
            serde_json::to_vec(action)
                .expect("a request payload is JSON-representable by construction"),
        ),
    )
}
