//! The fact this pack records: somebody's character is so-and-so.

use std::collections::BTreeSet;

use mineworld_contracts::{Event, EventSchemaVersion, EventTypeId, PersonId, SystemId, Visibility};
use mineworld_kernel::{Emission, SystemIdentity};
use serde::{Deserialize, Serialize};

use crate::codec;
use crate::persona::Persona;
use crate::system::PersonaSystem;

/// `person`'s character is `persona`.
///
/// Stated at genesis, from the person's authored `persona:` section, and reduced by this pack alone
/// into [`PersonaComponent`](crate::PersonaComponent). It is private: its audience is the holder and
/// nobody else (`ARC-74`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonaAuthored {
    person: PersonId,
    persona: Persona,
}

impl Event for PersonaAuthored {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("persona-authored");
    const OWNER: SystemId = PersonaSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl PersonaAuthored {
    /// States it.
    pub const fn new(person: PersonId, persona: Persona) -> Self {
        Self { person, persona }
    }

    /// Whose character.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// The character.
    pub const fn persona(&self) -> &Persona {
        &self.persona
    }

    /// As a fact ready to record: visible to the person it is about, and to nobody else.
    pub fn emission(&self) -> Emission {
        let holder = self.person.entity_id();
        Emission::new::<Self>(
            codec::encode(self),
            Visibility::Entities(BTreeSet::from([holder])),
        )
        .about(vec![holder])
        .with_participants(vec![holder])
    }
}
