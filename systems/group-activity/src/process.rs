//! The activity itself: a Process this pack starts, keeps, and ends.

use mineworld_contracts::{PersonId, ProcessTypeId, SimDuration};
use mineworld_kernel::ProcessKind;
use serde::{Deserialize, Serialize};

use crate::kind::ActivityKind;
use crate::system::GroupActivitySystem;

/// How long an activity runs unless it ends sooner: one simulated hour (`step-09-social.md` Q7).
pub const ACTIVITY_LENGTH: SimDuration = SimDuration::from_seconds(3_600);

/// The kind of process a group activity is, owned — as a type — by [`GroupActivitySystem`], so no
/// other system can start, change or end one (`INV-7`).
pub struct GroupActivity;

impl ProcessKind for GroupActivity {
    const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("group-activity");
    type Owner = GroupActivitySystem;
}

/// This pack's state for one running activity: what it is, who is in it now, and everyone who has
/// taken part.
///
/// The live member list is here rather than in the kernel's `participants`, which are fixed when a
/// process starts (`step-09-social.md` B-3). Both lists are kept in the order people came, so the
/// state — and every fact built from it — is the same on every run (`AC-12`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityState {
    kind: ActivityKind,
    members: Vec<PersonId>,
    took_part: Vec<PersonId>,
}

impl ActivityState {
    /// An activity of `kind` begun by `founders`.
    pub fn begun(kind: ActivityKind, founders: Vec<PersonId>) -> Self {
        Self {
            kind,
            took_part: founders.clone(),
            members: founders,
        }
    }

    /// What it is.
    pub const fn kind(&self) -> &ActivityKind {
        &self.kind
    }

    /// Who is part of it now.
    pub fn members(&self) -> &[PersonId] {
        &self.members
    }

    /// Everyone who has been part of it, in the order they came.
    pub fn took_part(&self) -> &[PersonId] {
        &self.took_part
    }

    /// Adds a member.
    pub fn join(&mut self, person: PersonId) {
        if !self.members.contains(&person) {
            self.members.push(person);
        }
        if !self.took_part.contains(&person) {
            self.took_part.push(person);
        }
    }

    /// Removes a member; everyone who took part is still remembered.
    pub fn leave(&mut self, person: PersonId) {
        self.members.retain(|member| *member != person);
    }

    /// Whether so few remain that it is no longer something done together.
    pub fn is_over(&self) -> bool {
        self.members.len() < 2
    }
}
