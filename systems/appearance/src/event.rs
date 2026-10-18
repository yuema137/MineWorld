//! The fact this pack records: somebody has a look.

use mineworld_contracts::{Event, EventSchemaVersion, EventTypeId, PersonId, SystemId, Visibility};
use mineworld_kernel::{Emission, SystemIdentity};
use serde::{Deserialize, Serialize};

use crate::codec;
use crate::look::LookKey;
use crate::system::AppearanceSystem;

/// `person` has the appearance called `look`.
///
/// Stated at genesis, from the person's authored `appearance:` section, and reduced by this pack alone
/// into [`Look`](crate::Look). Nothing changes a look in MVP-0; when something does (clothing, ageing,
/// disguise), it states this same fact, and this pack alone reduces it (`ARC-91`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Appeared {
    person: PersonId,
    look: LookKey,
}

impl Event for Appeared {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("appeared");
    const OWNER: SystemId = AppearanceSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl Appeared {
    /// States it.
    pub const fn new(person: PersonId, look: LookKey) -> Self {
        Self { person, look }
    }

    /// Who.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// The look they have.
    pub const fn look(&self) -> &LookKey {
        &self.look
    }

    /// As a fact ready to record: public, because what someone looks like is visible to whoever sees
    /// them (`ARC-91`), about the person.
    pub fn emission(&self) -> Emission {
        Emission::new::<Self>(codec::encode(self), Visibility::Public)
            .about(vec![self.person.entity_id()])
            .with_participants(vec![self.person.entity_id()])
    }
}
