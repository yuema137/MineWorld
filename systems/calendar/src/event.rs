//! The facts this pack records. None has a public constructor: only this pack states them.
//!
//! `calendar-configured` is the world's configuration (`ARC-61`), SystemInternal with no subjects.
//! `day-began` and `daylight-changed` are Public — "a change of season" is `Visibility::Public`'s own
//! example (`contracts/src/event.rs`) — so other packs may react to dawn and dusk (`ARC-26`, `ARC-28`)
//! without this pack knowing them.

use mineworld_contracts::{Event, EventSchemaVersion, EventTypeId, SystemId};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::civil::CalendarDate;
use crate::configuration::CalendarConfiguration;
use crate::day::{CalendarDay, MicroDegrees, Phase};
use crate::system::CalendarSystem;

/// The world's calendar, as configured: the epoch date, the fixed UTC offset in seconds, and the
/// position in micro-degrees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalendarConfigured {
    epoch: CalendarDate,
    utc_offset: i32,
    latitude: MicroDegrees,
    longitude: MicroDegrees,
}

impl Event for CalendarConfigured {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("calendar-configured");
    const OWNER: SystemId = CalendarSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl CalendarConfigured {
    pub(crate) const fn of(configuration: &CalendarConfiguration) -> Self {
        Self {
            epoch: configuration.epoch(),
            utc_offset: configuration.utc_offset(),
            latitude: configuration.latitude(),
            longitude: configuration.longitude(),
        }
    }

    /// The local date at instant 0.
    pub const fn epoch(&self) -> CalendarDate {
        self.epoch
    }

    /// Seconds the local time is ahead of UTC.
    pub const fn utc_offset(&self) -> i32 {
        self.utc_offset
    }

    /// Latitude, north positive.
    pub const fn latitude(&self) -> MicroDegrees {
        self.latitude
    }

    /// Longitude, east positive.
    pub const fn longitude(&self) -> MicroDegrees {
        self.longitude
    }
}

/// A local day began: its record, and the light phase at its first instant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DayBegan {
    day: CalendarDay,
    phase: Phase,
}

impl Event for DayBegan {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("day-began");
    const OWNER: SystemId = CalendarSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl DayBegan {
    pub(crate) const fn new(day: CalendarDay, phase: Phase) -> Self {
        Self { day, phase }
    }

    /// The day.
    pub const fn day(&self) -> &CalendarDay {
        &self.day
    }

    /// The light at its first instant.
    pub const fn phase(&self) -> Phase {
        self.phase
    }
}

/// The light changed: the phase that begins now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaylightChanged {
    phase: Phase,
}

impl Event for DaylightChanged {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("daylight-changed");
    const OWNER: SystemId = CalendarSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl DaylightChanged {
    pub(crate) const fn new(phase: Phase) -> Self {
        Self { phase }
    }

    /// The phase that begins.
    pub const fn phase(&self) -> Phase {
        self.phase
    }
}
