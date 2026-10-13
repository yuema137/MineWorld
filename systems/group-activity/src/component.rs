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
///
/// This is the compiled default. Since S17's PR IL-b a world may choose another lifetime through this
/// pack's section of the World's Interaction List (`invitation_lifetime`, [`crate::interactions`],
/// `ARC-63`); each invitation carries the instant it lapses, [`Invitation::until`], so whoever reads it
/// judges by the world's number and not by this one.
pub const INVITATION_LIFETIME: SimDuration = SimDuration::from_seconds(1_800);

/// One invitation a person has been given: from whom, to do what, when, and until when it can be
/// answered.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Invitation {
    from: PersonId,
    kind: ActivityKind,
    at: WorldTime,
    until: WorldTime,
}

impl Invitation {
    /// An invitation from `from` to do `kind`, made at `at`, open until `until` inclusive.
    pub const fn new(from: PersonId, kind: ActivityKind, at: WorldTime, until: WorldTime) -> Self {
        Self {
            from,
            kind,
            at,
            until,
        }
    }

    /// The last instant it can be answered: `at` plus the lifetime the world's section gave the pair
    /// where it was made ([`INVITATION_LIFETIME`] when it configures none).
    pub const fn until(&self) -> WorldTime {
        self.until
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

    /// Whether it can still be answered at `now`: not past [`Invitation::until`], inclusive. With
    /// `until = at + INVITATION_LIFETIME` this is exactly the test the pack made before invitations
    /// carried their expiry — `now − at ≤ lifetime`, which an instant before `at` also passed.
    pub fn is_open_at(&self, now: WorldTime) -> bool {
        now <= self.until
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
    // 2 since S17's PR IL-b: an invitation carries `until`.
    schema_version = 2,
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

#[cfg(test)]
mod tests {
    use mineworld_contracts::{EntityId, EntityType};

    use super::*;

    fn invitation(at: i64, lifetime: i64) -> Invitation {
        Invitation::new(
            PersonId::new(EntityId::from_raw(1), EntityType::Person).expect("a person"),
            ActivityKind::new("coffee").expect("a kind"),
            WorldTime::from_seconds(at),
            WorldTime::from_seconds(at + lifetime),
        )
    }

    /// SD-IB-15: with `until = at + INVITATION_LIFETIME`, `is_open_at` is the old test exactly —
    /// `now − at ≤ 1 800`, an instant before `at` included — over a table of instants on both sides of
    /// each bound; and an invitation
    /// a world gave 60 s lapses at its own `until`.
    #[test]
    fn an_invitation_is_open_exactly_until_it_says() {
        let old = |at: i64, now: i64| {
            WorldTime::from_seconds(now)
                .duration_since(WorldTime::from_seconds(at))
                .is_some_and(|age| age.seconds() <= INVITATION_LIFETIME.seconds())
        };
        for at in [0, 7, 900, 86_399] {
            let held = invitation(at, INVITATION_LIFETIME.seconds());
            for now in [
                at - 1,
                at,
                at + 1,
                at + 1_799,
                at + 1_800,
                at + 1_801,
                at + 5_000,
            ] {
                assert_eq!(
                    held.is_open_at(WorldTime::from_seconds(now)),
                    old(at, now),
                    "at {at}, now {now}"
                );
            }
        }
        let short = invitation(100, 60);
        assert!(short.is_open_at(WorldTime::from_seconds(160)));
        assert!(!short.is_open_at(WorldTime::from_seconds(161)));
    }
}
