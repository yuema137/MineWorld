//! The facts this pack records.
//!
//! All three are this pack's own state changes, reduced by it alone: `alarm-set` writes [`Asleep`],
//! `woke` removes it. `fell-asleep` writes nothing; it is what the room perceives, and it carries no
//! alarm (step-35 §4.1). None has a public constructor: only this pack states them.
//!
//! [`Asleep`]: crate::Asleep

use mineworld_contracts::{
    Event, EventSchemaVersion, EventTypeId, PersonId, PlaceId, SystemId, WorldTime,
};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::system::SleepSystem;

/// `person` fell asleep at `place`. Perceived by the place unless the world's list narrows it to the
/// participants. Not biographical unless the list says so.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FellAsleep {
    person: PersonId,
    place: PlaceId,
}

impl Event for FellAsleep {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("fell-asleep");
    const OWNER: SystemId = SleepSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl FellAsleep {
    pub(crate) const fn new(person: PersonId, place: PlaceId) -> Self {
        Self { person, place }
    }

    /// Who.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// Where.
    pub const fn place(&self) -> PlaceId {
        self.place
    }
}

/// `person`'s alarm is set for `alarm`. Stated with `fell-asleep`, to the sleeper alone, so that the
/// alarm never reaches the room and replay still rebuilds [`Asleep`](crate::Asleep). Never
/// biographical; its audience is not a world's choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlarmSet {
    person: PersonId,
    alarm: WorldTime,
}

impl Event for AlarmSet {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("alarm-set");
    const OWNER: SystemId = SleepSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl AlarmSet {
    pub(crate) const fn new(person: PersonId, alarm: WorldTime) -> Self {
        Self { person, alarm }
    }

    /// Whose.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// When it rings.
    pub const fn alarm(&self) -> WorldTime {
        self.alarm
    }
}

/// Why a sleep ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WokeReason {
    /// The alarm rang.
    Alarm,
    /// The sleeper asked to get up (`wake`).
    Asked,
    /// Somebody spoke to the sleeper, and the world's `woken_by_talk` held.
    Addressed,
    /// The sleeper acted: walked (presence's `arrived`) or spoke (conversation's `spoke`).
    Acted,
}

/// `person` woke after `slept` seconds asleep, for `reason`. Perceived by the place the sleeper is in
/// at that instant unless the world's list narrows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Woke {
    person: PersonId,
    slept: u64,
    reason: WokeReason,
}

impl Event for Woke {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("woke");
    const OWNER: SystemId = SleepSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl Woke {
    pub(crate) const fn new(person: PersonId, slept: u64, reason: WokeReason) -> Self {
        Self {
            person,
            slept,
            reason,
        }
    }

    /// Who.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// Whole seconds asleep: the instant of this fact less the instant the sleep began.
    pub const fn slept(&self) -> u64 {
        self.slept
    }

    /// Why.
    pub const fn reason(&self) -> WokeReason {
        self.reason
    }
}
