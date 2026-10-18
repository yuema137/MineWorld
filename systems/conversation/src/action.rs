//! The action this pack provides, and what it requires of space.

use mineworld_contracts::{Action, ActionTypeId, Millimetres, SpatialRequirement, SystemId};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::system::ConversationSystem;
use crate::utterance::Utterance;

/// How close two people must be to speak to each other: three metres.
///
/// Millimetres, as an integer, because positions reach the event log and floating point arithmetic is
/// not reproducible across platforms (`AC-12`). Three metres is conversational distance — across a
/// café table, not across a café — and it is this pack's policy rather than a physical constant: since
/// S17's PR IL-e it is the compiled default of the section's `range` ([`crate::interactions`]).
pub const INTERACTION_RANGE: Millimetres = Millimetres::new(3_000);

/// One person speaks to another.
///
/// The payload carries only what was said. Who says it and to whom are the
/// [`ActionIntent`](mineworld_contracts::ActionIntent)'s actor and target, and restating them here
/// would be two statements of one fact that a hand-built client frame could make disagree.
///
/// This is the action `AC-13` is about: a 2D client clicks a person and a 3D client walks up, looks at
/// them and presses a key, and both send *this* — the same actor, the same action type, the same
/// target, the same payload. Neither decides whether it is allowed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Talk {
    utterance: Utterance,
}

impl Action for Talk {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("talk");
    const OWNER: SystemId = ConversationSystem::ID;
}

impl Talk {
    /// Asks to say `utterance` to the intent's target.
    pub const fn new(utterance: Utterance) -> Self {
        Self { utterance }
    }

    /// What is being said.
    pub const fn utterance(&self) -> &Utterance {
        &self.utterance
    }

    /// What is being said, taken out of the request.
    pub fn into_utterance(self) -> Utterance {
        self.utterance
    }
}

/// What `talk` requires of space: the same place, within reach, of somebody available.
///
/// A **value**, which is the point of `ENGINEERING_RULES.md` §7: the requirement belongs to this
/// system's contract, it is evaluated by the one implementation in the contract layer
/// ([`SpatialRequirement::evaluate`]), and because it is data it can be *sent to a client* inside an
/// [`Affordance`](mineworld_contracts::Affordance) so that the client can show what the action needs
/// without checking it.
///
/// Three clauses, and each one is a decision:
///
/// ```text
/// same place as the actor   you cannot speak to somebody in another room
/// within INTERACTION_RANGE  and not from the far side of this one — in a world that models
///                           position at all; where it does not, the evaluator degenerates this
///                           to *same place*, which is the finest proximity such a world has
/// target available          what that means is this pack's business: a person still simulated
/// ```
///
/// No line of access. A wall between two people ought to stop a conversation, and the contract layer
/// declines to answer that from place identity and a distance because doing so would be a check that
/// only looks real (`DD-7`); a world that installs a geometry provider gains the answer, and this
/// requirement gains the clause on the same day, here, without touching a client.
///
/// This is the requirement of the compiled default. A world's section may give a speaker, a listener or
/// a place another `range` (S17's PR IL-e); the pack then validates and offers
/// [`talk_requirement_within`] that range, so the requirement shown is the one enforced.
pub fn talk_requirement() -> SpatialRequirement {
    talk_requirement_within(INTERACTION_RANGE)
}

/// What `talk` requires of space when a voice reaches `range`: the same place, within `range`, of
/// somebody available. `range` is positive: the section's bound starts at one millimetre.
pub fn talk_requirement_within(range: Millimetres) -> SpatialRequirement {
    SpatialRequirement::same_place()
        .within(range)
        .expect("a positive interaction range")
        .requiring_target_available()
}

/// A `range` parameter as a distance. The section bounds it to 1 … 100 000, well inside `i32`.
pub(crate) fn range(millimetres: u32) -> Millimetres {
    Millimetres::new(i32::try_from(millimetres).expect("a range within the section's bound"))
}
