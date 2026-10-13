//! Doing something together: inviting somebody, answering, joining, leaving.
//!
//! A **System Pack** (`docs/MVP.md` §5: *invite, accept invitation, reject invitation, join group
//! activity, leave group activity*). A world that installs it has people who spend an hour together;
//! a world that does not has people who only meet in passing, and is not broken.
//!
//! ```text
//! owns        Invitations           the invitations a person has been given and not yet answered
//!             Participation         which activity a person is part of right now
//!             group-activity        the activity itself: a Process at a place, ended by this pack
//! provides    invite                ask somebody within reach to do something with you
//!             accept-invitation     say yes: join the inviter's activity, or start one with them
//!             decline-invitation    say no
//!             join-group-activity   join what somebody here is doing
//!             leave-group-activity  stop
//! emits       invited, invitation-accepted, invitation-declined, group-activity-started,
//!             joined-group-activity, left-group-activity, group-activity-ended
//! subscribes  its own facts         Invitations and Participation are reductions of them
//!             person-entered-place  presence's: a member who walks into another place leaves
//! depends on  presence              where people are is presence's state, never this pack's
//! ```
//!
//! # An activity is a Process, and only this pack ends it
//!
//! `docs/CORE_CONCEPTS.md` §10: something that happens over time is a [`Process`], not an event, and
//! its owning system decides how it ends (`INV-3`, `INV-7`). An activity starts when an invitation is
//! accepted, runs for [`ACTIVITY_LENGTH`], and ends at its expected end (the kernel wakes this pack)
//! or as soon as fewer than two people remain in it. Every ending states `group-activity-ended`,
//! naming everyone who took part, which is what other packs (relationships) and a biography read.
//!
//! A process's participants are fixed when it starts (`kernel/src/process.rs`), so the kernel's
//! `participants` are the two who founded it, and the live member list is this pack's own process
//! state (`step-09-social.md` B-3).
//!
//! # What an activity *is* is content, not code
//!
//! The `kind` an inviter proposes — `coffee`, `walk` — is a validated slug this pack carries and never
//! interprets (`INV-12`). An activity is being together, at a place, for a while.
//!
//! [`Process`]: mineworld_kernel::Process

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;

pub mod action;
pub mod component;
pub mod error;
pub mod event;
pub mod interactions;
pub mod kind;
pub mod perception;
pub mod process;
pub mod system;

pub use action::{
    AcceptInvitation, DeclineInvitation, INVITE_RANGE, Invite, JoinGroupActivity,
    LeaveGroupActivity, accept_requirement, decline_requirement, invite_requirement,
    invite_requirement_within, join_requirement, leave_requirement,
};
pub use component::{INVITATION_LIFETIME, Invitation, Invitations, Participation};
pub use error::GroupActivityError;
pub use event::{
    GroupActivityEnded, GroupActivityStarted, InvitationAccepted, InvitationDeclined, Invited,
    JoinedGroupActivity, LeftGroupActivity,
};
pub use kind::ActivityKind;
pub use process::{ACTIVITY_LENGTH, ActivityState, GroupActivity};
pub use system::{BIOGRAPHICAL, GroupActivitySystem};
