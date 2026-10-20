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

/// The part of the day that begins when the agenda in force ends: one part of lookahead, what a person
/// keeps in mind ("next: work at 05:30"). Computed at disclosure from the holder's Routine; never
/// stored (`ARC-280`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextPart {
    /// Where.
    pub place: PlaceId,
    /// For what.
    pub label: AgendaLabel,
    /// When it begins: the agenda's `until`.
    pub from: WorldTime,
    /// Whether the person means to be there when it begins.
    #[serde(default)]
    pub punctual: bool,
}

/// The `agenda` record as its holder is told it: the stored [`Agenda`]'s fields, plus the next part
/// when the holder has a routine. Decoders of [`Agenda`] read it unchanged (no field is removed and
/// `Agenda` accepts unknown fields), so the component type stays `agenda`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisclosedAgenda {
    /// The part in force.
    #[serde(flatten)]
    pub agenda: Agenda,
    /// The part after it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<NextPart>,
}

impl DisclosedAgenda {
    /// The agenda as disclosed: the next part is the one in force at `agenda.until()`.
    pub fn of(agenda: &Agenda, routine: Option<&Routine>) -> Self {
        let next = routine.map(|routine| {
            let part = routine.segments().at(agenda.until()).segment;
            NextPart {
                place: *part.place(),
                label: part.label().clone(),
                from: agenda.until(),
                punctual: part.punctual(),
            }
        });
        Self {
            agenda: agenda.clone(),
            next,
        }
    }
}

owned_component! {
    component = Agenda,
    owner = ScheduleSystem,
    component_type = "agenda",
    schema_version = 1,
}
