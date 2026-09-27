//! The fact this pack records: somebody is now somewhere.

use mineworld_contracts::{Event, EventSchemaVersion, EventTypeId, Location, PersonId, SystemId};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::system::PresenceSystem;

/// Somebody is now at a location.
///
/// The whole of what this pack writes to history, and the only thing that changes a
/// [`Presence`](crate::Presence): [`PresenceSystem`] subscribes to its own fact and reduces it into
/// the component (`kernel/src/system.rs` on `subscribing_to`). That is not ceremony — it is what
/// makes the state a projection of the log, so a world rebuilt from its events holds the same
/// positions rather than positions that happen to agree.
///
/// The payload names the person, although the envelope already lists subjects and participants.
/// Deliberately: a reducer that recovered "who arrived" from the position of an entity in a list of
/// participants would be reading a convention, and the first system to state that list differently
/// would break it silently. A typed payload cannot be misread that way.
///
/// The person is a [`PersonId`] rather than an
/// [`EntityId`](mineworld_contracts::EntityId), so a fact about a place or an item cannot be
/// recorded as somebody arriving — and cannot be *read back* as one either, because the reference
/// carries the entity type it was checked against through serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Arrived {
    person: PersonId,
    location: Location,
}

impl Event for Arrived {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("arrived");
    const OWNER: SystemId = PresenceSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl Arrived {
    /// States that `person` is now at `location`.
    pub const fn new(person: PersonId, location: Location) -> Self {
        Self { person, location }
    }

    /// Who arrived.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// Where they now are.
    pub const fn location(&self) -> Location {
        self.location
    }
}
