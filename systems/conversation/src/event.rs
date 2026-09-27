//! The facts this pack records.
//!
//! Two of them, because two different things happen when somebody speaks: an exchange, which happens
//! every time, and the beginning of a conversation, which does not. A single fact carrying a
//! `started: bool` would make every reader re-derive the distinction, and a client that wanted to show
//! "Alice has started talking to you" would have to know the rule.

use mineworld_contracts::{Event, EventSchemaVersion, EventTypeId, PersonId, SystemId};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::system::ConversationSystem;
use crate::utterance::Utterance;

/// Two people have begun talking.
///
/// Emitted when an exchange follows a silence between these two longer than
/// [`CONVERSATION_GAP`](crate::CONVERSATION_GAP) — not on every `talk`. It carries no utterance: what
/// was said is the [`Spoke`] fact that accompanies this one, and duplicating it here would put the same
/// words in the log twice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationStarted {
    speaker: PersonId,
    listener: PersonId,
}

impl Event for ConversationStarted {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("conversation-started");
    const OWNER: SystemId = ConversationSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl ConversationStarted {
    /// States that `speaker` has begun talking to `listener`.
    pub const fn new(speaker: PersonId, listener: PersonId) -> Self {
        Self { speaker, listener }
    }

    /// Who spoke first.
    pub const fn speaker(&self) -> PersonId {
        self.speaker
    }

    /// Who was spoken to.
    pub const fn listener(&self) -> PersonId {
        self.listener
    }
}

/// One person said something to another.
///
/// The fact the whole pack turns on: it is what a client renders, what a controller reads, and what
/// this pack itself reduces into a [`ConversationHistory`](crate::ConversationHistory). Because the
/// history is written from this fact and from nothing else, a world replayed from its log remembers
/// exactly what the original run remembered.
///
/// Speaker and listener are named in the payload as well as in the envelope's subjects and
/// participants. Deliberately: a reducer that recovered them from a position in a participant list
/// would be reading a convention, and the first system to state that list differently would break it
/// silently. The typed payload cannot be misread that way, and
/// [`PersonId`] carries the entity type through serialization, so a place cannot be read back as a
/// speaker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spoke {
    speaker: PersonId,
    listener: PersonId,
    utterance: Utterance,
}

impl Event for Spoke {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("spoke");
    const OWNER: SystemId = ConversationSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl Spoke {
    /// States that `speaker` said `utterance` to `listener`.
    pub const fn new(speaker: PersonId, listener: PersonId, utterance: Utterance) -> Self {
        Self {
            speaker,
            listener,
            utterance,
        }
    }

    /// Who spoke.
    pub const fn speaker(&self) -> PersonId {
        self.speaker
    }

    /// Who was spoken to.
    pub const fn listener(&self) -> PersonId {
        self.listener
    }

    /// What was said.
    pub const fn utterance(&self) -> &Utterance {
        &self.utterance
    }

    /// What was said, taken out of the fact.
    pub fn into_utterance(self) -> Utterance {
        self.utterance
    }
}
