//! The fact this pack records: somebody is called something.

use mineworld_contracts::{Event, EventSchemaVersion, EventTypeId, PersonId, SystemId, Visibility};
use mineworld_kernel::{Emission, SystemIdentity};
use serde::{Deserialize, Serialize};

use crate::codec;
use crate::name::Name;
use crate::system::NamingSystem;

/// `person` is called `name`.
///
/// Stated at genesis, from the person's authored `name:` section, and reduced by this pack alone into
/// [`DisplayName`](crate::DisplayName). Nothing renames a person in MVP-0; when something does, it
/// states this same fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Named {
    person: PersonId,
    name: Name,
}

impl Event for Named {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("named");
    const OWNER: SystemId = NamingSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl Named {
    /// States it.
    pub const fn new(person: PersonId, name: Name) -> Self {
        Self { person, name }
    }

    /// Who.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// What they are called.
    pub const fn name(&self) -> &Name {
        &self.name
    }

    /// As a fact ready to record: public, because names are public (`ARC-31`), about the person.
    pub fn emission(&self) -> Emission {
        Emission::new::<Self>(codec::encode(self), Visibility::Public)
            .about(vec![self.person.entity_id()])
            .with_participants(vec![self.person.entity_id()])
    }
}
