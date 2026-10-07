//! The state this pack owns: a person's routine, and the part of it in force now.

use mineworld_contracts::{PlaceId, ProcessId, WorldTime};
use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::label::AgendaLabel;
use crate::segment::Segments;
use crate::system::ScheduleSystem;

/// A person's day, as assigned at genesis: the segments, places resolved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Routine {
    segments: Segments<PlaceId>,
}

impl Routine {
    /// This day.
    pub const fn new(segments: Segments<PlaceId>) -> Self {
        Self { segments }
    }

    /// Its segments.
    pub const fn segments(&self) -> &Segments<PlaceId> {
        &self.segments
    }
}

owned_component! {
    component = Routine,
    owner = ScheduleSystem,
    component_type = "routine",
    schema_version = 1,
}

/// Where a person's day says they should be now, and for what — an agenda, never a position
/// (`ARC-32`): nothing moves them there but whoever controls them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Agenda {
    place: PlaceId,
    label: AgendaLabel,
    since: WorldTime,
    until: WorldTime,
    routine: ProcessId,
}

impl Agenda {
    /// An agenda.
    pub const fn new(
        place: PlaceId,
        label: AgendaLabel,
        since: WorldTime,
        until: WorldTime,
        routine: ProcessId,
    ) -> Self {
        Self {
            place,
            label,
            since,
            until,
            routine,
        }
    }

    /// Where.
    pub const fn place(&self) -> PlaceId {
        self.place
    }

    /// For what.
    pub const fn label(&self) -> &AgendaLabel {
        &self.label
    }

    /// Since when this part of the day has been in force.
    pub const fn since(&self) -> WorldTime {
        self.since
    }

    /// When it ends: the next boundary.
    pub const fn until(&self) -> WorldTime {
        self.until
    }

    /// The person's routine process, whose wake will change this.
    pub const fn routine(&self) -> ProcessId {
        self.routine
    }
}

owned_component! {
    component = Agenda,
    owner = ScheduleSystem,
    component_type = "agenda",
    schema_version = 1,
}
