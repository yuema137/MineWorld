//! A Person whose actions are decided by a rule, and the demonstration that nothing about the world
//! knows the difference.
//!
//! ```text
//! Observation  ─►  RuleController::decide  ─►  ActionRequest  ─►  the same server, the same seat,
//!                                                                 the same actor check, the same
//!                                                                 server-allocated identity
//! ```
//!
//! `INV-1` says a Person is not an agent: *"a Person exists independently of whatever decides its
//! actions, and control may change at runtime without altering the Person."* That is a claim until
//! something drives a Person down the path a human client uses, which is what this crate is for.
//! `docs/MVP.md` §9 `AC-15` needs three participants — a 2D client, a 3D client and an agent-driven
//! Person — and the third one is here.
//!
//! # What a controller is, and the five things it may not do
//!
//! `docs/MODULE_SPEC.md` §5 and `docs/CORE_CONCEPTS.md` §14 are the contract, and every clause of it
//! is structural here rather than remembered:
//!
//! ```text
//! cannot create an interaction    it submits an action type an installed pack provides; a world
//!                                 without that pack answers Unavailable (INV-10)
//! consumes Observations only      decide() takes one argument and it is an Observation (INV-13)
//! emits requests only             it returns an ActionRequest; it has no handle to mutate anything
//! rebinding preserves the Person  it holds no world state — only which utterance it has answered
//! provider details stay behind    there is no provider: no model, no network, no clock
//!   the backend interface
//! ```
//!
//! It cannot allocate an `ActionId` or read the world's clock either, because `ActionRequest` has
//! neither field (PR 04, `spike/FINDINGS.md` F4).
//!
//! # Why a rule and not a model
//!
//! `docs/VISION.md`: *"a world with every model unplugged is still a valid MineWorld world"*, and
//! `ENGINEERING_STANDARDS.md` §10 requires the core tests to run with no external model reachable.
//! A deterministic controller is therefore not a placeholder for an `LMController` — it is what makes
//! the world testable, and MVP-0 ships it deliberately (`overall.md` S10's scope decision).
//!
//! # The rule, in one sentence
//!
//! *Answer the newest thing each person has said to me, once, when the server says I may — and say
//! something that proves I remember the rest.*
//!
//! The memory is not this crate's. It is the [`ConversationHistory`] the conversation pack projects
//! off the event log and discloses in this observer's own observation (`MVP.md` §9.2). What this
//! crate keeps is one fact a controller is allowed to keep: which utterance it has already answered.
//! A controller that stored what people said would be a second account of the world's history, and
//! the first one to disagree with the log.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::BTreeMap;

use mineworld_contracts::{ActionRecord, ActionRequest, EntityId, Observation, WorldTime};
use mineworld_conversation::{Heard, Talk};
use serde_json::Value;

use crate::act::reply_to;
use crate::read::{disclosed_history, may_talk_to, newest_per_speaker};

/// A controller that answers whoever has spoken to it, from what the world says it remembers.
///
/// Holds no world state. The one thing it keeps is which utterance it has already answered, per
/// speaker, so that an unchanged observation arriving ten times a second produces one reply rather
/// than ten. That is bookkeeping about its own past actions, which is a controller's own business —
/// and it is a [`BTreeMap`] because iteration order decides which person is answered first, and a
/// world must be reproducible from a seed (`AC-12`, `clippy.toml`).
#[derive(Debug, Default)]
pub struct RuleController {
    answered: BTreeMap<EntityId, Heard>,
    /// Lines heard at or before this instant are not this controller's to answer.
    bound_at: Option<WorldTime>,
}

impl RuleController {
    /// A controller that has answered nobody, and answers every line it is shown.
    pub fn new() -> Self {
        Self::default()
    }

    /// A controller bound to its Person at `at`: every line heard at or before `at` counts as already
    /// answered, because it was said before this controller drove the Person (`F-13`).
    ///
    /// A controller is rebuilt at every binding — a server's start on a saved world, a Person handed
    /// back after somebody else drove it — and keeps no memory across one, so without this it would
    /// answer the newest line again: the line it answered before a crash, or a line said to the
    /// person who drove the Person in between. Nothing is persisted to make this work; the instant is
    /// enough, because every heard line carries the world instant it was said at. The cost is a line
    /// said in the same world second the controller is bound, which it does not answer — the
    /// "missed the moment" limitation `ARC-27` already accepts.
    pub fn since(at: WorldTime) -> Self {
        Self {
            answered: BTreeMap::new(),
            bound_at: Some(at),
        }
    }

    /// Whether a line was heard after this controller was bound.
    fn is_after_binding(&self, heard: &Heard) -> bool {
        self.bound_at.is_none_or(|bound| heard.at() > bound)
    }

    /// What this Person attempts, given what it perceives — or nothing, which is the usual answer.
    ///
    /// Three things have to be true before it acts, and the second and third are both the server's
    /// judgement rather than this controller's:
    ///
    /// 1. somebody's newest words in this observer's own disclosed history are words this controller
    ///    has not answered;
    /// 2. the observation offers `talk` against that person;
    /// 3. and the server says that offer is **available** right now.
    ///
    /// Point 3 is where `ENGINEERING_RULES.md` §8 lands on a controller: it measures no distance,
    /// checks no permission and knows nothing about whether the conversation pack is installed. It
    /// reads the verdict the server computed and does as it is told — exactly as a client renders a
    /// prompt rather than deciding one.
    ///
    /// At most one request per observation, and the lowest [`EntityId`] first when several people
    /// are waiting. Both are so that one observation produces one reproducible decision.
    pub fn decide(&mut self, observation: &Observation<Value>) -> Option<ActionRequest> {
        let me = observation.observer();
        let history = disclosed_history(observation.entity(me)?)?;
        let newest = newest_per_speaker(&history, me);

        let (speaker, heard) = newest
            .iter()
            .map(|(speaker, heard)| (*speaker, *heard))
            .find(|(speaker, heard)| {
                self.is_after_binding(heard)
                    && self.answered.get(speaker) != Some(*heard)
                    && may_talk_to(observation, *speaker)
            })?;

        let said = reply_to(heard, &newest, speaker, observation)?;
        self.answered.insert(speaker, heard.clone());
        Some(
            ActionRequest::new(me, ActionRecord::new::<Talk>(encoded(&Talk::new(said))))
                .with_target(speaker),
        )
    }

    /// Whether this controller has answered anybody yet.
    ///
    /// The whole of its observable state, which is what makes "it did not act, and it did not mark
    /// anything answered either" a checkable claim — the difference between a controller that is
    /// waiting for the server to say yes and one that has silently given up.
    pub fn is_silent(&self) -> bool {
        self.answered.is_empty()
    }
}

/// This controller's own payload encoding, for the one action it submits.
///
/// `expect` for the same reason the conversation pack's codec uses one: `serde_json` fails only on
/// what cannot be represented as JSON — a non-finite number, a map with non-string keys — and a
/// [`Talk`] is a string. There is no float anywhere in this workspace.
fn encoded(value: &Talk) -> Vec<u8> {
    serde_json::to_vec(value).expect("a request payload is JSON-representable by construction")
}

mod agenda;
mod model;
mod offered;
mod paced;
mod social;

/// Request builders shared by the decision models: what to submit, built from an offered option
/// (`pr-dl-b-motivated.md` SD-DLb-1).
pub mod act;
/// When to set out for a punctual part of the day: the shared estimate and the commute policy
/// (`pr-le-leave-early.md` SD-LE-5, §0.4; `ARC-282`).
pub mod commute;
/// Observation readers shared by the decision models: what an observation discloses, decoded once
/// (`pr-dl-b-motivated.md` SD-DLb-1).
pub mod read;

pub use model::{Rule, RuleParameters};
pub use paced::PacedRuleController;
pub use read::walks;

/// This Controller Pack's package identity, from its `Cargo.toml` (`docs/DECISIONS.md` `ARC-53`).
pub const PACKAGE: mineworld_packages::Package = mineworld_packages::package!();

#[cfg(test)]
mod tests;

#[cfg(test)]
mod paced_tests;

#[cfg(test)]
mod social_tests;

#[cfg(test)]
mod agenda_tests;

#[cfg(test)]
mod offered_tests;

#[cfg(test)]
mod commute_tests;
