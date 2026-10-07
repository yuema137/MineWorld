//! The installable system: what it declares, the facts it reduces, and the wake that keeps a day.

use mineworld_contracts::{
    ComponentRecord, EntityId, Event, EventEnvelope, EventTypeId, PersonId, PlaceId, SystemId,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, Process, ProcessStart, System, SystemDeclaration,
    SystemIdentity, SystemVersion, WorldRead, WorldView,
};
use mineworld_presence::PerceptionProvider;
use serde_json::Value;

use crate::codec;
use crate::component::{Agenda, Routine};
use crate::event::{AgendaChanged, RoutineAssigned};
use crate::process::{RoutineProcess, RoutineState};
use crate::segment::Segments;

/// Which of this pack's facts belong in a person's objective biography (`ARC-29`): each change of
/// agenda — the day as it was lived through (`step-09-social.md` SD-12). Validation makes every one a
/// real change.
pub const BIOGRAPHICAL: &[EventTypeId] = &[AgendaChanged::EVENT_TYPE];

/// A person's day: a routine, and the agenda in force.
pub struct ScheduleSystem;

impl SystemIdentity for ScheduleSystem {
    const ID: SystemId = SystemId::from_static("schedule");
}

impl System for ScheduleSystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    /// No dependency and no action. It emits only its own vocabulary, which is why it cannot move
    /// anybody: stating presence's `arrived` would need a dependency on presence, and the registry
    /// would refuse it (`ARC-26`, `ARC-32`).
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<Routine>()
            .owning::<Agenda>()
            .emitting::<RoutineAssigned>()
            .emitting::<AgendaChanged>()
            .subscribing_to::<RoutineAssigned>()
            .subscribing_to::<AgendaChanged>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Routine>()?;
        tables.component::<Agenda>()
    }

    /// `routine-assigned` → the Routine, the routine process, and the first agenda; `agenda-changed`
    /// → the Agenda, whose only writer is this reduction.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let kind = event.event_type();
        if *kind == RoutineAssigned::EVENT_TYPE {
            let assigned: RoutineAssigned = codec::event_payload(event.payload())?;
            let person = assigned.person();
            let segments = assigned.segments().clone();
            let now = segments.at(world.at());
            let routine = world.start_process(
                ProcessStart::<RoutineProcess>::new(codec::encode(&RoutineState::new(person)))
                    .with_participants(vec![person.entity_id()])
                    .ending_at(now.until)
                    .uninterruptible(),
            )?;
            let first = changed(person, &segments, world.at(), routine);
            world.insert(person.entity_id(), Routine::new(segments))?;
            return Ok(vec![first]);
        }
        if *kind == AgendaChanged::EVENT_TYPE {
            let changed: AgendaChanged = codec::event_payload(event.payload())?;
            let since = event.at();
            world.insert(
                changed.person().entity_id(),
                Agenda::new(
                    changed.place(),
                    changed.label().clone(),
                    since,
                    changed.until(),
                    changed.routine(),
                ),
            )?;
        }
        Ok(Vec::new())
    }

    /// A boundary: the next part of the day begins. The process is rescheduled to the boundary after
    /// it and never ends — a person's day goes on as long as the world does.
    fn wake(
        &self,
        world: &mut WorldView<'_, Self>,
        process: &Process,
    ) -> Result<Vec<Emission>, KernelError> {
        let missing = || KernelError::ProcessNotRunning {
            process: process.id(),
        };
        let state: RoutineState =
            codec::state(process.state_for::<RoutineProcess>()?).ok_or_else(missing)?;
        let person = state.person();
        let segments = world
            .read()
            .component::<Routine>(person.entity_id())
            .map(|routine| routine.segments().clone())
            .ok_or_else(missing)?;
        let next = segments.at(world.at()).until;
        world.reschedule_process::<RoutineProcess>(process.id(), Some(next))?;
        Ok(vec![changed(person, &segments, world.at(), process.id())])
    }
}

/// The `agenda-changed` for the segment in force at `at`.
fn changed(
    person: PersonId,
    segments: &Segments<PlaceId>,
    at: mineworld_contracts::WorldTime,
    routine: mineworld_contracts::ProcessId,
) -> Emission {
    let now = segments.at(at);
    AgendaChanged::new(
        person,
        *now.segment.place(),
        now.segment.label().clone(),
        now.until,
        routine,
    )
    .emission()
}

impl PerceptionProvider for ScheduleSystem {
    /// A person's Agenda, **to that person and nobody else** (`INV-13`): where my day says I should be
    /// is mine to know. The Routine is not disclosed.
    fn discloses(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        subject: EntityId,
    ) -> Vec<ComponentRecord<Value>> {
        if subject != observer {
            return Vec::new();
        }
        world
            .component::<Agenda>(observer)
            .map(|agenda| {
                vec![ComponentRecord::new::<Agenda>(
                    observer,
                    codec::to_value(agenda),
                )]
            })
            .unwrap_or_default()
    }
}
