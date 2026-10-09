//! What a pack declares its section may say: roles, actions, facts, effects and audiences
//! (`docs/DECISIONS.md` `ARC-63` item 2, `ARC-65`).

use mineworld_contracts::{ActionTypeId, EventTypeId};
use serde::{Deserialize, Serialize};

/// A position in an interaction, declared by the owning pack per action, per fact and for its
/// parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Who acts.
    Actor,
    /// Whom, or what, the act is directed at.
    Target,
    /// What is used or moved: an item kind or a loose object.
    Object,
    /// Where it happens.
    Place,
}

impl Role {
    /// Every role, in their fixed order: the order a selector tuple is kept in.
    pub const ALL: [Self; 4] = [Self::Actor, Self::Target, Self::Object, Self::Place];

    /// The role's index in [`Role::ALL`].
    pub const fn index(self) -> usize {
        match self {
            Self::Actor => 0,
            Self::Target => 1,
            Self::Object => 2,
            Self::Place => 3,
        }
    }

    /// The role as a section writes it.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Actor => "actor",
            Self::Target => "target",
            Self::Object => "object",
            Self::Place => "place",
        }
    }

    /// The role a section's key names, if it names one.
    pub fn named(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|role| role.name() == key)
    }
}

/// What a rule says about an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    /// Allowed, as far as the list is concerned; the pack's own checks still apply.
    Permit,
    /// Refused `PermissionDenied`.
    Forbid,
}

/// Who a fact may be perceived by, as a list may choose it: widest first (`ARC-65` item 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Audience {
    /// Anyone.
    Public,
    /// Anyone present where it happened.
    Place,
    /// Only the fact's participants.
    Participants,
}

/// One action a pack lets its section govern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionDecl {
    /// The action.
    pub action: ActionTypeId,
    /// The roles a rule about it may name.
    pub roles: &'static [Role],
    /// Whether a region may carry a rule about it.
    pub regional: bool,
}

/// Where in a fact's envelope a role is found, so a fact's roles can be read from the envelope alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    /// The envelope's `subjects[i]`.
    Subject(usize),
    /// The envelope's `participants[i]`.
    Participant(usize),
    /// The envelope's place.
    Place,
}

/// One of a pack's own fact types whose consequence a section may govern (`ARC-65`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactDecl {
    /// The fact type.
    pub fact: EventTypeId,
    /// Each role a consequence entry may name, and where the envelope holds it.
    pub roles: &'static [(Role, Position)],
    /// The audience the owner states it with; a list may only narrow it.
    pub default_audience: Audience,
    /// The narrowest audience a list may choose.
    pub narrowest: Audience,
    /// Whether a list may switch its biographical flag.
    pub biography_configurable: bool,
}

/// What the installed set says about a pack's section, for the tools (`Capability::interaction_section`).
#[derive(Debug, Clone)]
pub struct SectionDecl {
    /// The pack's configured fact type, `<pack>-interactions-configured`.
    pub configured: EventTypeId,
    /// Its declared actions.
    pub actions: &'static [ActionDecl],
    /// Its declared facts.
    pub facts: &'static [FactDecl],
    /// Its compiled biographical fact types (`ARC-29`).
    pub biographical: &'static [EventTypeId],
    /// Reads a configured fact's payload into what the biography projection needs.
    pub consequences: fn(&[u8]) -> Result<crate::interactions::ConsequenceTable, String>,
}
