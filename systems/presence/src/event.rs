//! The fact this pack records: somebody is now somewhere.

use mineworld_contracts::{
    Event, EventSchemaVersion, EventTypeId, Location, PersonId, SystemId, Visibility,
};
use mineworld_kernel::{Emission, SystemIdentity};
use serde::{Deserialize, Serialize};

use crate::codec;
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

/// This pack's [`Arrived`] as a fact ready to be recorded: the payload, the audience, the subject,
/// the participants and the place.
///
/// Public because a world is *assembled* as well as run. A World Pack that wants Alice to start
/// behind the counter needs a `Presence` component, a component is written only by its owner while
/// reducing, and the only fact that writes this one is `Arrived` — so world assembly has to be able
/// to state that fact. [`World::genesis`](mineworld_kernel::World::genesis) records it with
/// [`Causation::WorldGenesis`](mineworld_contracts::Causation), and this pack reduces it through
/// exactly the path a runtime `arrive` takes, which is what makes an authored position and a walked
/// one the same kind of thing in the log.
///
/// It is a function here rather than a payload a loader encodes for itself, because the encoding is
/// this pack's own business: "a payload's format is a contract between the system that declares the
/// event type and whoever reads it back" (`kernel/src/system.rs`). A loader that built these bytes
/// would be a second implementation of this pack's codec, and the first change to the codec would
/// silently make a loaded world unreadable.
///
/// Used by [`PresenceSystem::resolve`](crate::PresenceSystem) too, so a `arrive` request and a
/// genesis arrival cannot describe the same arrival differently.
pub fn arrival(person: PersonId, location: Location) -> Emission {
    Emission::new::<Arrived>(
        codec::encode(&Arrived::new(person, location)),
        Visibility::Place(location.place()),
    )
    .about(vec![person.entity_id()])
    .with_participants(vec![person.entity_id()])
    .at_place(location.place())
}
