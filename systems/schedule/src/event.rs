//! The facts this pack records.

use mineworld_contracts::{
    Event, EventSchemaVersion, EventTypeId, PersonId, PlaceId, ProcessId, SystemId, Visibility,
    WorldTime,
};
use mineworld_kernel::{Emission, SystemIdentity};
use serde::{Deserialize, Serialize};

use crate::codec;
use crate::label::AgendaLabel;
use crate::segment::Segments;
use crate::system::ScheduleSystem;

/// `person`'s day is `segments`. Stated at genesis, from the person's authored `routine:` section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutineAssigned {
    person: PersonId,
    segments: Segments<PlaceId>,
}

impl Event for RoutineAssigned {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("routine-assigned");
    const OWNER: SystemId = ScheduleSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl RoutineAssigned {
    /// States it.
    pub const fn new(person: PersonId, segments: Segments<PlaceId>) -> Self {
        Self { person, segments }
    }

    /// Whose day.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// The day.
    pub const fn segments(&self) -> &Segments<PlaceId> {
        &self.segments
    }

    /// As a fact: the person's own business, about them.
    pub fn emission(&self) -> Emission {
        personal::<Self>(codec::encode(self), self.person)
    }
}

/// `person`'s agenda is now `label` at `place`, until `until`.
///
/// Caused by the person's routine process when a boundary comes (`Causation::Process`), or — the
/// first one — by the `routine-assigned` it follows. It moves nobody (`ARC-32`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgendaChanged {
    person: PersonId,
    place: PlaceId,
    label: AgendaLabel,
    until: WorldTime,
    routine: ProcessId,
}

impl Event for AgendaChanged {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("agenda-changed");
    const OWNER: SystemId = ScheduleSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl AgendaChanged {
    /// States it.
    pub const fn new(
        person: PersonId,
        place: PlaceId,
        label: AgendaLabel,
        until: WorldTime,
        routine: ProcessId,
    ) -> Self {
        Self {
            person,
            place,
            label,
            until,
            routine,
        }
    }

    /// Whose.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// Where the agenda now is.
    pub const fn place(&self) -> PlaceId {
        self.place
    }

    /// For what.
    pub const fn label(&self) -> &AgendaLabel {
        &self.label
    }

    /// Until the next boundary.
    pub const fn until(&self) -> WorldTime {
        self.until
    }

    /// The routine process this agenda belongs to.
    pub const fn routine(&self) -> ProcessId {
        self.routine
    }

    /// As a fact: the person's own business, about them, at no place — the change does not happen at
    /// the agenda's place, it happens to the person, wherever they are.
    pub fn emission(&self) -> Emission {
        personal::<Self>(codec::encode(self), self.person)
    }
}

/// One of this pack's facts about `person`, heard by them alone (`INV-13`).
fn personal<E: Event>(payload: Vec<u8>, person: PersonId) -> Emission {
    Emission::new::<E>(payload, Visibility::Participants)
        .about(vec![person.entity_id()])
        .with_participants(vec![person.entity_id()])
}
