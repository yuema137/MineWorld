//! The facts this pack records — only what a biography or a client can name (`step-09-social.md` Q5).

use mineworld_contracts::{Event, EventSchemaVersion, EventTypeId, PersonId, SystemId};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::level::Level;
use crate::system::RelationshipsSystem;

/// `person` now knows `counterpart`: the `knows` edge was formed. Once per direction, ever.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BecameAcquainted {
    person: PersonId,
    counterpart: PersonId,
}

impl Event for BecameAcquainted {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("became-acquainted");
    const OWNER: SystemId = RelationshipsSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl BecameAcquainted {
    /// States it.
    pub const fn new(person: PersonId, counterpart: PersonId) -> Self {
        Self {
            person,
            counterpart,
        }
    }

    /// Who now knows somebody.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// Whom.
    pub const fn counterpart(&self) -> PersonId {
        self.counterpart
    }
}

/// How `person` stands with `counterpart` crossed a level boundary, up or down.
///
/// Not stated for a change of value that stays within a level: those are reductions of logged facts,
/// derivable from the log, and a fact per change would double a run's history to say nothing a reader
/// can name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationshipChanged {
    person: PersonId,
    counterpart: PersonId,
    from: Level,
    to: Level,
}

impl Event for RelationshipChanged {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("relationship-changed");
    const OWNER: SystemId = RelationshipsSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl RelationshipChanged {
    /// States it.
    pub const fn new(person: PersonId, counterpart: PersonId, from: Level, to: Level) -> Self {
        Self {
            person,
            counterpart,
            from,
            to,
        }
    }

    /// Whose regard changed.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// For whom.
    pub const fn counterpart(&self) -> PersonId {
        self.counterpart
    }

    /// The level before.
    pub const fn from(&self) -> Level {
        self.from
    }

    /// The level after.
    pub const fn to(&self) -> Level {
        self.to
    }
}
