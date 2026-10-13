//! The installable system: what it declares, the facts it folds, the wake that turns the weather, and
//! what it discloses (SD-TW-b-2, -10, -11).

use mineworld_authoring::{Attachment, ConfigurationContext, PackConfiguration, Seeding};
use mineworld_calendar::{CalendarSystem, DayBegan};
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

use crate::component::{WeatherNow, WeatherToday};
use crate::configuration::{RecordConfiguration, WeatherConfiguration};
use crate::day::{HOURS, Origin, WeatherDay};
use crate::event::{WeatherChanged, WeatherConfigured};
use crate::generate::{record_day, weather_day};
use crate::process::{ClimateProcess, ClimateState, RecordProcess, RecordState};
use crate::record::{DayOrigin, Fill, RecordSeries};

/// Seconds in an hour.
const HOUR: i64 = 3_600;

/// The world's weather.
#[derive(Default)]
pub struct WeatherSystem;

impl SystemIdentity for WeatherSystem {
    const ID: SystemId = SystemId::from_static("weather");
}

/// What the build needs to know about this pack beyond [`System`] (`ARC-33`): it takes
/// `configure/weather.yaml` (`ARC-61`), and none of its facts is biographical.
impl SystemPack for WeatherSystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    mineworld_sdk::configures!();
}

impl PackConfiguration for WeatherSystem {
    type Configuration = WeatherConfiguration;
    const FACTS: &'static [EventTypeId] = &[WeatherConfigured::EVENT_TYPE];

    /// The record's file, when the world replays one (`ARC-61` note).
    fn attachments(configuration: &WeatherConfiguration) -> Vec<&Attachment> {
        configuration
            .record()
            .map(RecordConfiguration::data)
            .into_iter()
            .collect()
    }

    /// One `weather-configured`, SystemInternal, about nobody (`ARC-61` item 8). With `source: record` the
    /// attachment's bytes are decoded and packed into it here; a file that is not the format, a
    /// `first_year` outside it, or a long gap under `fill: none` is refused as `weather-record-invalid`,
    /// naming the attachment and the line (SD-TW-d-1 … 4).
    fn seed(
        _: &Seeding<'_, '_>,
        configuration: &WeatherConfiguration,
        context: &ConfigurationContext<'_>,
    ) -> Result<Vec<Emission>, Rejection> {
        let record = configuration
            .record()
            .map(|record| series(record, configuration.fill(), context))
            .transpose()?;
        Ok(vec![Emission::new::<WeatherConfigured>(
            encode(&WeatherConfigured::new(
                configuration.seed(),
                configuration.rules().clone(),
                record,
            )),
            Visibility::SystemInternal,
        )])
    }
}

/// The record a configuration names, decoded from the bytes the loader read and packed.
fn series(
    record: &RecordConfiguration,
    fill: Fill,
    context: &ConfigurationContext<'_>,
) -> Result<RecordSeries, Rejection> {
    let invalid = |detail: String| Rejection::System {
        code: RejectionCode::from_static("weather-record-invalid"),
        detail: Some(format!("{}: {detail}", record.data())),
    };
    let bytes = context
        .attached()
        .get(record.data())
        .ok_or_else(|| invalid("was not read".to_owned()))?;
    let file = crate::record::decode(bytes).map_err(|error| invalid(error.to_string()))?;
    RecordSeries::new(record.station().clone(), &file, record.first_year(), fill).map_err(invalid)
}

/// Encodes a payload this pack declared; infallible for its structs of integers.
fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("this pack's payloads are JSON-representable by construction")
}

/// Reads an event payload, this pack's or calendar's.
fn payload<E: Event + DeserializeOwned>(record: &EventRecord) -> Result<E, ContractError> {
    serde_json::from_slice(record.payload_for::<E>()?).map_err(|_| {
        ContractError::EventTypeMismatch {
            expected: E::EVENT_TYPE,
            actual: record.event_type().clone(),
        }
    })
}

/// A refusal of this pack's own, naming the fact it could not state or fold.
fn refused(event_type: EventTypeId, code: &'static str) -> KernelError {
    KernelError::FactRefusedByOwner {
        system: WeatherSystem::ID,
        event_type,
        reason: Rejection::System {
            code: RejectionCode::from_static(code),
            detail: None,
        },
    }
}

/// The world's climate process, if it has one.
fn climate<'a>(world: &WorldRead<'a>) -> Option<&'a Process> {
    world.processes().find(|process| {
        *process.owner() == WeatherSystem::ID
            && *process.process_type() == ClimateProcess::PROCESS_TYPE
    })
}

/// The world's replayed record, if it has one: the `weather-record` Process's series, read only.
fn recorded(world: &WorldRead<'_>) -> Result<Option<RecordState>, KernelError> {
    let Some(process) = record_process(world) else {
        return Ok(None);
    };
    let bytes = process.state_for::<RecordProcess>()?;
    serde_json::from_slice::<RecordState>(bytes)
        .map(Some)
        .map_err(|_| KernelError::ProcessNotRunning {
            process: process.id(),
        })
}

/// The `weather-record` Process, if the world replays a record.
fn record_process<'a>(world: &WorldRead<'a>) -> Option<&'a Process> {
    world.processes().find(|process| {
        *process.owner() == WeatherSystem::ID
            && *process.process_type() == RecordProcess::PROCESS_TYPE
    })
}

/// The climate's state, read back.
fn state(process: &Process) -> Result<ClimateState, KernelError> {
    let bytes = process.state_for::<ClimateProcess>()?;
    serde_json::from_slice(bytes).map_err(|_| KernelError::ProcessNotRunning {
        process: process.id(),
    })
}

/// The hour of `day` that `at` falls in, if `at` is within the day.
fn hour_of(day: &WeatherDay, at: WorldTime) -> Option<u8> {
    let since = at.seconds() - day.day_start().seconds();
    u8::try_from(since.div_euclid(HOUR))
        .ok()
        .filter(|hour| usize::from(*hour) < HOURS && since >= 0)
}

/// The first hour after `hour` whose condition differs from `hour`'s, as an instant; none when the
/// condition holds to midnight (the next `day-began` takes over).
fn next_change(day: &WeatherDay, hour: u8) -> Option<WorldTime> {
    let hours = day.hours();
    let holding = hours[usize::from(hour)].condition;
    (usize::from(hour) + 1..HOURS)
        .find(|later| hours[*later].condition != holding)
        .map(|later| {
            let later = i64::try_from(later).expect("an hour");
            WorldTime::from_seconds(day.day_start().seconds() + later * HOUR)
        })
}

/// `weather-changed` for `hour` of `day`, Public, about nobody.
fn changed(day: &WeatherDay, hour: u8) -> Emission {
    Emission::new::<WeatherChanged>(
        encode(&WeatherChanged::at_hour(day, hour)),
        Visibility::Public,
    )
}

impl System for WeatherSystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    /// Depends on presence to know where an observer is, and on calendar for its days (`INV-TW-10`).
    /// Provides no action: nobody does anything to make it rain.
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([PresenceSystem::ID, CalendarSystem::ID])
            .owning::<WeatherToday>()
            .owning::<WeatherNow>()
            .emitting::<WeatherConfigured>()
            .emitting::<WeatherDay>()
            .emitting::<WeatherChanged>()
            .subscribing_to::<WeatherConfigured>()
            .subscribing_to::<DayBegan>()
            .subscribing_to::<WeatherDay>()
            .subscribing_to::<WeatherChanged>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<WeatherToday>()?;
        tables.component::<WeatherNow>()
    }

    /// `weather-configured` → the climate process, unstarted and open-ended. `day-began` → that day's
    /// weather, stated as `weather-day` (nothing when weather is not configured). `weather-day` → folded,
    /// with `weather-changed` at once if the condition differs from the one in force, and the process
    /// due at the day's next change. `weather-changed` → folded. These are the only writes of the
    /// process's state.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let kind = event.event_type().clone();
        if kind == WeatherConfigured::EVENT_TYPE {
            let configured: WeatherConfigured = payload(event.payload())?;
            if climate(&world.read()).is_some() {
                return Err(refused(kind, "weather-configured-twice"));
            }
            world.start_process(
                ProcessStart::<ClimateProcess>::new(encode(&ClimateState::unstarted(&configured)))
                    .uninterruptible(),
            )?;
            // A replayed record's days get a Process of their own, open-ended and never woken, written
            // once here and only read after (SD-TW-d-5).
            if let Some(series) = configured.into_record() {
                world.start_process(
                    ProcessStart::<RecordProcess>::new(encode(&RecordState::new(series)))
                        .uninterruptible(),
                )?;
            }
            return Ok(Vec::new());
        }
        let current = {
            let read = world.read();
            climate(&read)
                .map(|process| state(process).map(|state| (process.id(), state)))
                .transpose()?
        };
        if kind == DayBegan::EVENT_TYPE {
            // Weather that is enabled but not configured has no climate, and no weather.
            let Some((_, current)) = current else {
                return Ok(Vec::new());
            };
            let began: DayBegan = payload(event.payload())?;
            let configured = current.configured();
            let (rules, seed, calendar) = (configured.rules(), configured.seed(), began.day());
            let record = recorded(&world.read())?;
            // A record world replays the day's record date (SD-TW-d-4); a day the record lacks for
            // longer than a short gap, and every day of a world of rules, is drawn by the rules.
            let replayed = record.map(|record| {
                let epoch = current.epoch_year().unwrap_or(calendar.date().year());
                record.series().day(calendar.date(), epoch)
            });
            let day = match replayed {
                Some((date, recorded)) if recorded.origin() == DayOrigin::Record => {
                    record_day(rules, seed, calendar, &recorded, Origin::Record { date })
                }
                Some((date, recorded)) if recorded.origin() == DayOrigin::Filled => {
                    record_day(rules, seed, calendar, &recorded, Origin::Filled { date })
                }
                _ => weather_day(rules, seed, calendar, current.chain()),
            };
            return Ok(vec![Emission::new::<WeatherDay>(
                encode(&day),
                Visibility::SystemInternal,
            )]);
        }
        let Some((id, current)) = current else {
            return Err(refused(kind, "weather-unconfigured"));
        };
        if kind == WeatherDay::EVENT_TYPE {
            let day: WeatherDay = payload(event.payload())?;
            let hour = hour_of(&day, world.at()).unwrap_or(0);
            let in_force = current.condition();
            let emissions = if in_force == Some(day.hours()[usize::from(hour)].condition) {
                Vec::new()
            } else {
                vec![changed(&day, hour)]
            };
            // A record world folds the year of its first day, from which every later day's record year
            // is counted (SD-TW-d-4); a world of rules keeps none, so its state is TW-b's.
            let replays = record_process(&world.read()).is_some();
            let year = day.date().year();
            let next = next_change(&day, hour);
            let mut folded = current.with_day(day, hour);
            if replays {
                folded = folded.with_epoch(year);
            }
            world.reschedule_process::<ClimateProcess>(id, next)?;
            world.set_process_state::<ClimateProcess>(id, encode(&folded))?;
            return Ok(emissions);
        }
        let turned: WeatherChanged = payload(event.payload())?;
        if current.today().is_none() {
            return Err(refused(kind, "weather-unstarted"));
        }
        world.set_process_state::<ClimateProcess>(id, encode(&current.with_now(turned.hour())))?;
        Ok(Vec::new())
    }

    /// The hour due: if its condition differs from the one in force, it is stated. The process is
    /// rescheduled to the day's next change, or left open-ended until the next `day-began`.
    fn wake(
        &self,
        world: &mut WorldView<'_, Self>,
        process: &Process,
    ) -> Result<Vec<Emission>, KernelError> {
        let current = state(process)?;
        let Some(day) = current.today() else {
            world.reschedule_process::<ClimateProcess>(process.id(), None)?;
            return Ok(Vec::new());
        };
        let Some(hour) = hour_of(day, world.at()) else {
            world.reschedule_process::<ClimateProcess>(process.id(), None)?;
            return Ok(Vec::new());
        };
        let emissions = if current.condition() == Some(day.hours()[usize::from(hour)].condition) {
            Vec::new()
        } else {
            vec![changed(day, hour)]
        };
        world.reschedule_process::<ClimateProcess>(process.id(), next_change(day, hour))?;
        Ok(emissions)
    }
}

impl PerceptionProvider for WeatherSystem {
    /// Today's weather and the condition now, as two records about **the place the observer is in**,
    /// and about nothing else: the weather is the world's, and it is known where one is. An observer
    /// in no place gets neither.
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
        let Some(current) = climate(world).and_then(|process| state(process).ok()) else {
            return Vec::new();
        };
        let (Some(today), Some(condition)) = (current.today(), current.condition()) else {
            return Vec::new();
        };
        vec![
            ComponentRecord::new::<WeatherToday>(subject, to_value(&WeatherToday(today.clone()))),
            ComponentRecord::new::<WeatherNow>(
                subject,
                to_value(&WeatherNow {
                    hour: current.now(),
                    condition,
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
