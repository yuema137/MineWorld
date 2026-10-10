//! The state this pack owns: what a person has been told, and by whom.

use mineworld_contracts::{PersonId, WorldTime};
use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::system::ConversationSystem;
use crate::utterance::Utterance;

/// How many exchanges a person keeps.
///
/// A bound, because this is a projection and not a memory: an unbounded list would grow with every
/// word spoken to a person for as long as a world runs, and it would be persisted and replicated at
/// that size. Thirty-two is enough for *"somebody spoke to me three minutes ago about X"*, which is
/// exactly what `docs/MVP.md` §9.2 asks of MVP-0 and deliberately no more. Episodic memory,
/// summarization, retrieval and forgetting are the MVP-1 evolution of this path and belong to a
/// cognition pack, not to a larger number here.
///
/// The compiled default of the section's `remembered` since S17's PR IL-e, which a world may lower or
/// raise for a class of listener, to at most 64 — still a projection, not a memory.
pub const REMEMBERED_AT_MOST: usize = 32;

/// One thing a person was told: who said it, when, and what.
///
/// The three fields `docs/MVP.md` §9.2 names, and no fourth. In particular there is no importance, no
/// sentiment, no summary and no embedding: every one of those is an interpretation, interpretation is
/// a controller's business, and putting one here would make this component the beginning of the
/// cognition architecture this slice exists to postpone.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Heard {
    speaker: PersonId,
    at: WorldTime,
    utterance: Utterance,
}

impl Heard {
    /// Records that `speaker` said `utterance` at `at`.
    pub const fn new(speaker: PersonId, at: WorldTime, utterance: Utterance) -> Self {
        Self {
            speaker,
            at,
            utterance,
        }
    }

    /// Who spoke.
    pub const fn speaker(&self) -> PersonId {
        self.speaker
    }

    /// When they spoke, on the world's clock — never on a wall clock, so a replay remembers the same
    /// moment (`AC-12`).
    pub const fn at(&self) -> WorldTime {
        self.at
    }

    /// What they said.
    pub const fn utterance(&self) -> &Utterance {
        &self.utterance
    }
}

/// Everything a person has been told lately, oldest first.
///
/// This is the whole of Alice's memory in MVP-0, and it is a **projection**: it is written in one
/// place only — [`ConversationSystem::react`](crate::ConversationSystem), reducing this pack's own
/// [`Spoke`](crate::Spoke) fact — so a world rebuilt from its event log holds the same history rather
/// than a history that happens to agree. Nothing else may write it, and the compiler is what says so
/// (`INV-7`).
///
/// It records what a person *heard*, not what they said: the entry for "Alice spoke to Bob" lives on
/// Bob. A controller asking "who has spoken to me lately" therefore reads one component, which is the
/// question `MVP.md` §9.2 poses; the symmetric question — whom have I spoken to — is answered from
/// the event log, which is where the objective account belongs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationHistory {
    heard: Vec<Heard>,
}

impl ConversationHistory {
    /// A history holding one entry.
    pub fn of(heard: Heard) -> Self {
        Self { heard: vec![heard] }
    }

    /// Records an exchange under the compiled bound, [`REMEMBERED_AT_MOST`]: what a listener keeps in
    /// a world whose section says nothing about `remembered`.
    pub fn remember(&mut self, heard: Heard) {
        self.remember_within(heard, REMEMBERED_AT_MOST);
    }

    /// Records an exchange, dropping the oldest to keep at most `at_most` — the `remembered` the
    /// world's section gives this listener.
    ///
    /// The loop rather than a single removal is deliberate: a history kept under a larger bound — a
    /// snapshot from an older version, or a listener whose bound differs by place — is brought within
    /// this one here, rather than being left over-long forever.
    pub fn remember_within(&mut self, heard: Heard, at_most: usize) {
        self.heard.push(heard);
        while self.heard.len() > at_most {
            self.heard.remove(0);
        }
    }

    /// Everything remembered, oldest first.
    pub fn heard(&self) -> &[Heard] {
        &self.heard
    }

    /// The most recent thing this person was told by `speaker`, if it is still remembered.
    ///
    /// The question the pack asks to decide whether an exchange continues a conversation or starts
    /// one. "If it is still remembered" is load-bearing: past the bound the answer is honestly
    /// unknown, and a conversation is then treated as new — which is the safe direction, because it
    /// records a fact rather than omitting one.
    pub fn last_heard_from(&self, speaker: PersonId) -> Option<&Heard> {
        self.heard
            .iter()
            .rev()
            .find(|heard| heard.speaker() == speaker)
    }

    /// How many exchanges are remembered.
    pub fn len(&self) -> usize {
        self.heard.len()
    }

    /// Whether this person remembers being told anything.
    pub fn is_empty(&self) -> bool {
        self.heard.is_empty()
    }
}

owned_component! {
    component = ConversationHistory,
    owner = ConversationSystem,
    component_type = "conversation-history",
    schema_version = 1,
}
