//! The five actions this pack provides, and what each requires of space.
//!
//! Who acts and on whom is the [`ActionIntent`](mineworld_contracts::ActionIntent)'s actor and
//! target, never restated in a payload: `invite` targets the invitee, `accept-invitation` and
//! `decline-invitation` target the **inviter**, and `join-group-activity` targets **a member** of the
//! activity — because a client points at people, and a process is not an entity anybody can point at
//! (`step-09-social.md` B-4). `leave-group-activity` targets nobody.

use mineworld_contracts::{Action, ActionTypeId, Millimetres, SpatialRequirement, SystemId};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::kind::ActivityKind;
use crate::system::GroupActivitySystem;

/// How close an inviter must be to the person invited: three metres, in reach of being asked.
///
/// This pack's own constant, not conversation's — it does not depend on conversation, and an
/// invitation is a different act from speech even where the distances agree. Integer millimetres,
/// so every platform decides alike (`AC-12`).
pub const INVITE_RANGE: Millimetres = Millimetres::new(3_000);

/// Ask the target to do something together.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Invite {
    kind: ActivityKind,
}

impl Action for Invite {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("invite");
    const OWNER: SystemId = GroupActivitySystem::ID;
}

impl Invite {
    /// Proposes `kind`.
    pub const fn new(kind: ActivityKind) -> Self {
        Self { kind }
    }

    /// What is proposed.
    pub const fn kind(&self) -> &ActivityKind {
        &self.kind
    }
}

/// Say yes to the target's invitation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptInvitation {}

impl Action for AcceptInvitation {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("accept-invitation");
    const OWNER: SystemId = GroupActivitySystem::ID;
}

/// Say no to the target's invitation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclineInvitation {}

impl Action for DeclineInvitation {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("decline-invitation");
    const OWNER: SystemId = GroupActivitySystem::ID;
}

/// Join what the target is doing.
///
/// Named in full rather than `join`, because action types are one namespace across every installed
/// pack and the registry refuses two providers of one name: a later pack's `join` (an organization, a
/// queue) must not collide with this one.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinGroupActivity {}

impl Action for JoinGroupActivity {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("join-group-activity");
    const OWNER: SystemId = GroupActivitySystem::ID;
}

/// Stop taking part in one's activity.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaveGroupActivity {}

impl Action for LeaveGroupActivity {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("leave-group-activity");
    const OWNER: SystemId = GroupActivitySystem::ID;
}

/// `invite`: the same place, within [`INVITE_RANGE`], of somebody available — not already part of an
/// activity, which is this pack's judgement and travels as the offer's availability.
pub fn invite_requirement() -> SpatialRequirement {
    SpatialRequirement::same_place()
        .within(INVITE_RANGE)
        .expect("a positive invite range")
        .requiring_target_available()
}

/// `accept-invitation`: the same place as the inviter — an activity happens somewhere, and one
/// cannot join it from across town.
pub fn accept_requirement() -> SpatialRequirement {
    SpatialRequirement::same_place()
}

/// `decline-invitation`: nothing of space. Saying no needs no reach.
pub fn decline_requirement() -> SpatialRequirement {
    SpatialRequirement::NONE
}

/// `join-group-activity`: the same place as a member who is part of an activity.
pub fn join_requirement() -> SpatialRequirement {
    SpatialRequirement::same_place().requiring_target_available()
}

/// `leave-group-activity`: nothing of space.
pub fn leave_requirement() -> SpatialRequirement {
    SpatialRequirement::NONE
}
