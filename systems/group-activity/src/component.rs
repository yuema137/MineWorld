//! The state this pack owns on people: invitations not yet answered, and being part of an activity.

use mineworld_contracts::{PersonId, ProcessId, SimDuration, WorldTime};
use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::kind::ActivityKind;
use crate::system::GroupActivitySystem;

/// How long an invitation stays open: thirty simulated minutes.
///
/// **Derived, not chosen** (`step-09-social.md` QB-1). The rule frozen in Q7 is *an invitation lives
/// long enough to be answered at the invitee's next consult*. `mineworld run` consults seat `k` at
/// `genesis + k + m·PACE`, so an invitation from seat `a` reaches seat `b` after `b − a` seconds when
/// `b > a`, and after `PACE − (a − b)` seconds — up to `PACE − 1` — when `b < a`. With the 600 s pace
/// Q7 was answered under, 600 s met the rule; with the 900 s pace S8 runs at (`ARC-27` note), 600 s
/// let every invitation to a lower-numbered seat expire unanswered. Two paces, 1 800 s, meets the rule
/// with a pace to spare for an invitee who answers another invitation first at that consult.
///
/// It is a published constant of this pack, not derived from the pace at run time: a pack that knew a
/// driver's schedule would be a pack that knew who drives it. It is checked against the instant the
/// answer is made (the request's `issued_at`), inclusive.
pub const INVITATION_LIFETIME: SimDuration = SimDuration::from_seconds(1_800);

/// One invitation a person has been given: from whom, to do what, and when.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Invitation {
    from: PersonId,
    kind: ActivityKind,
    at: WorldTime,
}

impl Invitation {
    /// An invitation from `from` to do `kind`, made at `at`.
    pub const fn new(from: PersonId, kind: ActivityKind, at: WorldTime) -> Self {
        Self { from, kind, at }
    }

    /// Who invited.
    pub const fn from(&self) -> PersonId {
        self.from
    }

    /// What was proposed.
    pub const fn kind(&self) -> &ActivityKind {
        &self.kind
    }

    /// When it was made, on the world's clock.
    pub const fn at(&self) -> WorldTime {
        self.at
    }

    /// Whether it can still be answered at `now`: no older than [`INVITATION_LIFETIME`], inclusive.
    pub fn is_open_at(&self, now: WorldTime) -> bool {
        now.duration_since(self.at)
            .is_some_and(|age| age.seconds() <= INVITATION_LIFETIME.seconds())
    }
}

/// The invitations a person has been given and not answered, at most one per inviter, by inviter.
///
/// A **reduction** of this pack's own `invited`, `invitation-accepted` and `invitation-declined`
/// facts, and nothing else, so a world rebuilt from its log holds the same invitations. Expired
/// entries are dropped whenever the component is written, so it stays bounded by the people who
/// invited recently; an expired entry that has not been written over yet is refused at the answer,
/// never honoured.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Invitations {
    pending: Vec<Invitation>,
}

impl Invitations {
    /// Every invitation held, by inviter.
    pub fn pending(&self) -> &[Invitation] {
        &self.pending
    }

    /// The invitation from `inviter`, open or not.
    pub fn from(&self, inviter: PersonId) -> Option<&Invitation> {
        self.pending
            .iter()
            .find(|invitation| invitation.from == inviter)
    }

    /// Records an invitation, replacing an earlier one from the same inviter, and drops what has
    /// expired by `now`.
    pub fn receive(&mut self, invitation: Invitation, now: WorldTime) {
        self.pending.retain(|held| held.from != invitation.from);
        self.pending.push(invitation);
        self.pending.sort();
        self.prune(now);
    }

    /// Forgets the invitation from `inviter` — it was answered — and drops what has expired by `now`.
    pub fn answered(&mut self, inviter: PersonId, now: WorldTime) {
        self.pending.retain(|held| held.from != inviter);
        self.prune(now);
    }

    fn prune(&mut self, now: WorldTime) {
        self.pending.retain(|held| held.is_open_at(now));
    }

    /// Whether nothing is held.
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

owned_component! {
    component = Invitations,
    owner = GroupActivitySystem,
    component_type = "invitations",
    schema_version = 1,
}

/// That a person is part of an activity: which, doing what, since when.
///
/// A reduction of `group-activity-started` and `joined-group-activity`, removed by
/// `left-group-activity` and `group-activity-ended`. Present exactly while the person is a member.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Participation {
    activity: ProcessId,
    kind: ActivityKind,
    since: WorldTime,
}

impl Participation {
    /// Part of `activity`, doing `kind`, since `since`.
    pub const fn new(activity: ProcessId, kind: ActivityKind, since: WorldTime) -> Self {
        Self {
            activity,
            kind,
            since,
        }
    }

    /// The activity's process.
    pub const fn activity(&self) -> ProcessId {
        self.activity
    }

    /// What it is.
    pub const fn kind(&self) -> &ActivityKind {
        &self.kind
    }

    /// Since when this person has been part of it.
    pub const fn since(&self) -> WorldTime {
        self.since
    }
}

owned_component! {
    component = Participation,
    owner = GroupActivitySystem,
    component_type = "participation",
    schema_version = 1,
}
