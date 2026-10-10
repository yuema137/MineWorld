//! The installable system: what it declares, the facts it folds, the wake that keeps the calendar,
//! and what it discloses.

use mineworld_authoring::{ConfigurationContext, PackConfiguration, Seeding};
use mineworld_contracts::{
    ComponentRecord, ContractError, EntityId, Event, EventEnvelope, EventRecord, EventTypeId,
    Rejection, RejectionCode, SystemId, Visibility, WorldTime,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, Process, ProcessKind, ProcessStart, System,
    SystemDeclaration, SystemIdentity, SystemVersion, WorldRead, WorldView,
};
use mineworld_presence::{PerceptionProvider, Presence, PresenceSystem};
use mineworld_sdk::SystemPack;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::civil::CalendarDate;
use crate::component::{CalendarDayRecord, CalendarLight};
use crate::configuration::CalendarConfiguration;
use crate::day::{CalendarDay, DAY, Phase};
use crate::event::{CalendarConfigured, DayBegan, DaylightChanged};
use crate::process::{CalendarProcess, CalendarState};
use crate::sun::{self, SunUnavailable};

/// The world's civil calendar and sun.
#[derive(Default)]
pub struct CalendarSystem;

impl SystemIdentity for CalendarSystem {
    const ID: SystemId = SystemId::from_static("calendar");
}

/// What the build needs to know about this pack beyond [`System`] (`ARC-33`): it takes
/// `configure/calendar.yaml` (`ARC-61`), and none of its facts is biographical.
impl SystemPack for CalendarSystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    mineworld_sdk::configures!();
}

impl PackConfiguration for CalendarSystem {
    type Configuration = CalendarConfiguration;
    const FACTS: &'static [EventTypeId] = &[CalendarConfigured::EVENT_TYPE];

    /// One `calendar-configured`, SystemInternal, about nobody (`ARC-61` item 8).
    fn seed(
        _: &Seeding<'_, '_>,
        configuration: &CalendarConfiguration,
        _: &ConfigurationContext<'_>,
    ) -> Result<Vec<Emission>, Rejection> {
        Ok(vec![Emission::new::<CalendarConfigured>(
            encode(&CalendarConfigured::of(configuration)),
            Visibility::SystemInternal,
        )])
    }
}

/// Encodes a payload this pack declared; infallible for its structs of integers.
fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("this pack's payloads are JSON-representable by construction")
}

/// Reads one of this pack's event payloads.
fn payload<E: Event + DeserializeOwned>(record: &EventRecord) -> Result<E, ContractError> {
    serde_json::from_slice(record.payload_for::<E>()?).map_err(|_| {
        ContractError::EventTypeMismatch {
            expected: E::EVENT_TYPE,
            actual: record.event_type().clone(),
        }
    })
}

/// A refusal of this pack's own, naming the fact it could not state or fold.
fn refused(event_type: EventTypeId, code: &'static str, detail: Option<String>) -> KernelError {
    KernelError::FactRefusedByOwner {
        system: CalendarSystem::ID,
        event_type,
        reason: Rejection::System {
            code: RejectionCode::from_static(code),
            detail,
        },
    }
}

fn sun_unavailable(error: &SunUnavailable) -> KernelError {
    refused(
        DayBegan::EVENT_TYPE,
        "calendar-sun-unavailable",
        Some(error.to_string()),
    )
}

/// The world's calendar process, if it has one.
fn calendar<'a>(world: &WorldRead<'a>) -> Option<&'a Process> {
    world.processes().find(|process| {
        *process.owner() == CalendarSystem::ID
            && *process.process_type() == CalendarProcess::PROCESS_TYPE
    })
}

/// The calendar's state, read back.
fn state(process: &Process) -> Result<CalendarState, KernelError> {
    let bytes = process.state_for::<CalendarProcess>()?;
    serde_json::from_slice(bytes).map_err(|_| KernelError::ProcessNotRunning {
        process: process.id(),
    })
}

/// The day that contains `at`, for a world configured `configured`, and the light at `at`.
fn day_at(
    configured: &CalendarConfigured,
    at: WorldTime,
) -> Result<(CalendarDay, Phase), KernelError> {
    let index = at.seconds().div_euclid(DAY);
    let epoch_days = configured.epoch().days();
    let date = CalendarDate::from_days(epoch_days + index).ok_or_else(|| {
        refused(
            DayBegan::EVENT_TYPE,
            "calendar-date-out-of-range",
            Some(format!("day {index} after the epoch")),
        )
    })?;
    let day_start = WorldTime::from_seconds(index * DAY);
    let (day, opening) = CalendarDay::compute(
        date,
        day_start,
        epoch_days,
        configured.utc_offset(),
        configured.latitude(),
        configured.longitude(),
    )
    .map_err(|error| sun_unavailable(&error))?;
    if at == day_start {
        return Ok((day, opening));
    }
    // A world that begins mid-day: the light at its first instant, not at the midnight before it.
    let utc = epoch_days * DAY + at.seconds() - i64::from(configured.utc_offset());
    let phase = sun::phase_at(configured.latitude(), configured.longitude(), utc)
        .map_err(|error| sun_unavailable(&error))?;
    Ok((day, phase))
}

/// When the calendar is next due after `now` within `day`: its next light change, else the next
/// midnight. Always strictly later than `now`.
fn next_due(day: &CalendarDay, now: WorldTime) -> WorldTime {
    let offset = now.seconds() - day.day_start().seconds();
    day.events()
        .light_changes()
        .into_iter()
        .map(|(at, _)| i64::from(at))
        .find(|at| *at > offset)
        .map_or_else(
            || WorldTime::from_seconds(day.day_start().seconds() + DAY),
            |at| WorldTime::from_seconds(day.day_start().seconds() + at),
        )
}

/// `day-began`, Public, about nobody.
fn began(day: CalendarDay, phase: Phase) -> Emission {
    Emission::new::<DayBegan>(encode(&DayBegan::new(day, phase)), Visibility::Public)
}

impl System for CalendarSystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    /// Depends on presence only to know where an observer is, for disclosure. Provides no action:
    /// nobody does anything to make a day pass.
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([PresenceSystem::ID])
            .owning::<CalendarDayRecord>()
            .owning::<CalendarLight>()
            .emitting::<CalendarConfigured>()
            .emitting::<DayBegan>()
            .emitting::<DaylightChanged>()
            .subscribing_to::<CalendarConfigured>()
            .subscribing_to::<DayBegan>()
            .subscribing_to::<DaylightChanged>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<CalendarDayRecord>()?;
        tables.component::<CalendarLight>()
    }

    /// `calendar-configured` → the first day, stated as `day-began`, and the calendar process, due at
    /// the day's next light change or midnight. `day-began` and `daylight-changed` → folded into the
    /// process's state. These are the only writes of that state.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let kind = event.event_type();
        if *kind == CalendarConfigured::EVENT_TYPE {
            let configured: CalendarConfigured = payload(event.payload())?;
            if calendar(&world.read()).is_some() {
                return Err(refused(
                    CalendarConfigured::EVENT_TYPE,
                    "calendar-configured-twice",
                    None,
                ));
            }
            let now = world.at();
            let (day, phase) = day_at(&configured, now)?;
            world.start_process(
                ProcessStart::<CalendarProcess>::new(encode(&CalendarState::new(
                    configured,
                    day.clone(),
                    phase,
                )))
                .ending_at(next_due(&day, now))
                .uninterruptible(),
            )?;
            return Ok(vec![began(day, phase)]);
        }
        let folded = |state: CalendarState| -> Result<CalendarState, KernelError> {
            if *kind == DayBegan::EVENT_TYPE {
                let began: DayBegan = payload(event.payload())?;
                Ok(state.with_day(began.day().clone(), began.phase()))
            } else {
                let changed: DaylightChanged = payload(event.payload())?;
                Ok(state.with_phase(changed.phase()))
            }
        };
        let (id, current) = {
            let read = world.read();
            let Some(process) = calendar(&read) else {
                return Err(refused(kind.clone(), "calendar-unconfigured", None));
            };
            (process.id(), state(process)?)
        };
        let next = folded(current)?;
        world.set_process_state::<CalendarProcess>(id, encode(&next))?;
        Ok(Vec::new())
    }

    /// A light change or a midnight, whichever is due. At midnight the next day is computed and
    /// stated; otherwise each light change at this instant is stated, in order. The process is
    /// rescheduled to what is due next and never ends.
    fn wake(
        &self,
        world: &mut WorldView<'_, Self>,
        process: &Process,
    ) -> Result<Vec<Emission>, KernelError> {
        let current = state(process)?;
        let now = world.at();
        let day = current.day();
        let offset = now.seconds() - day.day_start().seconds();
        if offset >= DAY {
            let (next, phase) = day_at(current.configured(), now)?;
            world
                .reschedule_process::<CalendarProcess>(process.id(), Some(next_due(&next, now)))?;
            return Ok(vec![began(next, phase)]);
        }
        let changes: Vec<Emission> = day
            .events()
            .light_changes()
            .into_iter()
            .filter(|(at, _)| i64::from(*at) == offset)
            .map(|(_, phase)| {
                Emission::new::<DaylightChanged>(
                    encode(&DaylightChanged::new(phase)),
                    Visibility::Public,
                )
            })
            .collect();
        world.reschedule_process::<CalendarProcess>(process.id(), Some(next_due(day, now)))?;
        Ok(changes)
    }
}

impl PerceptionProvider for CalendarSystem {
    /// Today and the light, as two records about **the place the observer is in**, and about nothing
    /// else: the calendar is the world's, and it is known where one is. An observer in no place gets
    /// neither (step-19 §16.8 (c)).
    fn discloses(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        subject: EntityId,
    ) -> Vec<ComponentRecord<Value>> {
        let here = world
            .component::<Presence>(observer)
            .map(|presence| presence.location().place().entity_id());
        if here != Some(subject) {
            return Vec::new();
        }
        let Some(current) = calendar(world).and_then(|process| state(process).ok()) else {
            return Vec::new();
        };
        vec![
            ComponentRecord::new::<CalendarDayRecord>(
                subject,
                to_value(&CalendarDayRecord(current.day().clone())),
            ),
            ComponentRecord::new::<CalendarLight>(
                subject,
                to_value(&CalendarLight {
                    phase: current.phase(),
                }),
            ),
        ]
    }
}

/// A disclosed record as the self-describing value an observation carries.
fn to_value<T: Serialize>(record: &T) -> Value {
    serde_json::to_value(record)
        .expect("this pack's records are JSON-representable by construction")
}
