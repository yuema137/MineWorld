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

use mineworld_contracts::{
    Action, ActionRecord, ActionRequest, Component, EntityId, Observation, PerceivedEntity,
};
use mineworld_conversation::{ConversationHistory, Heard, Talk, UTTERANCE_MAX_BYTES, Utterance};
use serde_json::Value;

/// How much of somebody's words this controller quotes back.
///
/// A bound, because an utterance is bounded ([`UTTERANCE_MAX_BYTES`]) and a reply that quotes two
/// of them plus its own words would otherwise be refused by the contract for length. Quoting is
/// deliberately literal within the bound: this controller does not summarize, because summarizing
/// is interpretation and interpretation is what MVP-1's cognition layer is for.
const QUOTED_AT_MOST: usize = 80;

/// What this controller marks a truncated quotation with.
const ELLIPSIS: &str = "…";

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
}

impl RuleController {
    /// A controller that has answered nobody.
    pub fn new() -> Self {
        Self::default()
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
                self.answered.get(speaker) != Some(*heard) && may_talk_to(observation, *speaker)
            })?;

        let said = reply_to(heard, &newest, speaker)?;
        self.answered.insert(speaker, heard.clone());
        Some(
            ActionRequest::new(me, ActionRecord::new::<Talk>(encoded(&Talk::new(said))))
                .with_target(speaker),
        )
    }

    /// Whether this controller has answered anybody yet. For a driver that wants to log the first
    /// time it does.
    pub fn is_silent(&self) -> bool {
        self.answered.is_empty()
    }
}

/// The observer's own conversation history, as the observation disclosed it.
///
/// Through [`ComponentRecord::payload_for`](mineworld_contracts::ComponentRecord::payload_for), so a
/// record labelled with a different component type is not read as this one — the same check every
/// other reader of a record gets. An observation that disclosed nothing yields [`None`], which is
/// *this person has been told nothing they may know about*, not an error: a controller cannot ask
/// again, because there is nothing to ask (`INV-13`).
fn disclosed_history(me: &PerceivedEntity<Value>) -> Option<ConversationHistory> {
    let record = me
        .components()
        .iter()
        .find(|record| *record.component_type() == ConversationHistory::COMPONENT_TYPE)?;
    serde_json::from_value(record.payload_for::<ConversationHistory>().ok()?.clone()).ok()
}

/// The newest thing each person has said to this observer, by speaker.
///
/// Oldest first in the history, so a later entry overwrites an earlier one and the map holds the
/// newest. The observer's own entry is dropped if one ever appears: answering oneself is not a
/// conversation, and the conversation pack refuses it anyway.
fn newest_per_speaker(history: &ConversationHistory, me: EntityId) -> BTreeMap<EntityId, &Heard> {
    let mut newest = BTreeMap::new();
    for heard in history.heard() {
        let speaker = heard.speaker().entity_id();
        if speaker == me {
            continue;
        }
        newest.insert(speaker, heard);
    }
    newest
}

/// Whether the **server** says this observer may speak to that person right now.
///
/// Read, never computed. A controller that worked out for itself whether somebody was in reach would
/// be the duplicated rule `ENGINEERING_RULES.md` §9 calls an architectural failure, and it would
/// disagree with dispatch the first time either changed.
fn may_talk_to(observation: &Observation<Value>, person: EntityId) -> bool {
    observation.affordances().iter().any(|affordance| {
        *affordance.action_type() == Talk::ACTION_TYPE
            && affordance.target() == Some(person)
            && affordance.is_available()
    })
}

/// What to say back: what you just said, and evidence that I remember somebody else saying
/// something.
///
/// The second clause is the whole point of the controller in `AC-15`. A player speaks to Alice in one
/// window; a *different* player walks up to her in another and is told what the first one said. That
/// sentence can only be produced by a Person who holds both conversations, which is what "there is
/// only one Alice" means.
///
/// People are named by identity, because in this world nothing owns a display name: a name is
/// component state, and a controller that invented one would be inventing world data (`DD-13`). The
/// day a pack owns and discloses a name, this reads it out of the same observation.
fn reply_to(
    heard: &Heard,
    newest: &BTreeMap<EntityId, &Heard>,
    speaker: EntityId,
) -> Option<Utterance> {
    let mine = quoted(heard.utterance().as_str());
    let said = match newest
        .iter()
        .find(|(other, _)| **other != speaker)
        .map(|(other, theirs)| (other, quoted(theirs.utterance().as_str())))
    {
        Some((other, theirs)) => format!(
            "I remember you. You said \"{mine}\". Earlier, person {other} said \"{theirs}\" to me."
        ),
        None => format!(
            "I remember you. You said \"{mine}\". You are the first person to speak to me here."
        ),
    };
    // The bound is the contract's, and it is checked rather than assumed: `quoted` keeps each
    // quotation short enough that the whole sentence fits, and if a future wording stopped fitting
    // the controller would fall silent instead of submitting a request the contract refuses.
    debug_assert!(
        said.len() <= UTTERANCE_MAX_BYTES,
        "a reply fits an utterance"
    );
    Utterance::new(said).ok()
}

/// Somebody's words, short enough to quote inside a reply.
///
/// Truncated on a character boundary, because an utterance is UTF-8 text and a byte slice through the
/// middle of a character is not.
fn quoted(said: &str) -> String {
    match said.char_indices().nth(QUOTED_AT_MOST) {
        None => said.to_owned(),
        Some((boundary, _)) => format!("{}{ELLIPSIS}", &said[..boundary]),
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

#[cfg(test)]
mod tests;
