//! The installable system: what it declares, the daily pass, the facts it folds, the wakes that begin
//! and end each happening, and what it discloses (design §5.2 … §5.4, §5.7).

use mineworld_authoring::{ConfigurationContext, PackConfiguration, Reference, Seeding};
use mineworld_calendar::{CalendarSystem, DayBegan};
use mineworld_contracts::{
    ComponentRecord, ContractError, EntityId, EntityKey, EntityType, Event, EventEnvelope,
    EventRecord, EventTypeId, PlaceId, ProcessId, Rejection, RejectionCode, SystemId, Visibility,
    WorldTime,
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

use crate::component::{HappeningsAnnounced, HappeningsHere, ShownHappening};
use crate::configuration::HappeningsConfiguration;
use crate::event::{
    HappeningAnnounced, HappeningBegan, HappeningEnded, HappeningWithheld, HappeningsConfigured,
};
use crate::host::HappeningRef;
use crate::kind::Audience;
use crate::pass::pass;
use crate::process::{
    AlmanacProcess, AlmanacState, HappeningPhase, HappeningProcess, HappeningState,
};

/// The world's happenings.
#[derive(Default)]
pub struct HappeningsSystem;

impl SystemIdentity for HappeningsSystem {
    const ID: SystemId = SystemId::from_static("happenings");
}

/// What the build needs to know about this pack beyond [`System`] (`ARC-33`): it takes
/// `configure/happenings.yaml` (`ARC-61`), and none of its facts is biographical — a market day is not
/// something a person did (`ARC-29`).
impl SystemPack for HappeningsSystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    mineworld_sdk::configures!();
}

impl PackConfiguration for HappeningsSystem {
    type Configuration = HappeningsConfiguration;
    const FACTS: &'static [EventTypeId] = &[HappeningsConfigured::EVENT_TYPE];

    /// Every place a kind may happen at; an entry's place is one of its kind's, so this is all of them.
    fn references(configuration: &HappeningsConfiguration) -> Vec<Reference<'_>> {
        configuration
            .kinds()
            .iter()
            .flat_map(|(_, kind)| kind.places())
            .map(|key| Reference {
                key,
                entity_type: EntityType::Place,
            })
            .collect()
    }

    /// One `happenings-configured`, SystemInternal, about nobody (`ARC-61` item 8), every place
    /// resolved to its id.
    fn seed(
        seeding: &Seeding<'_, '_>,
        configuration: &HappeningsConfiguration,
        _: &ConfigurationContext<'_>,
    ) -> Result<Vec<Emission>, Rejection> {
        let resolve = |key: &EntityKey| {
            seeding
                .resolve(key, EntityType::Place)
                .and_then(|entity| PlaceId::new(entity, EntityType::Place).ok())
                .ok_or_else(|| Rejection::System {
                    code: RejectionCode::from_static("happenings-place-unknown"),
                    detail: Some(format!("'{}' is no place of this world", key.as_str())),
                })
        };
        let configured = HappeningsConfigured::of(configuration, resolve)?;
        Ok(vec![Emission::new::<HappeningsConfigured>(
            encode(&configured),
            Visibility::SystemInternal,
        )])
    }
}

/// Encodes a payload this pack declared; infallible for its structs of integers and strings.
pub(crate) fn encode<T: Serialize>(value: &T) -> Vec<u8> {
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
        system: HappeningsSystem::ID,
        event_type,
        reason: Rejection::System {
            code: RejectionCode::from_static(code),
            detail: None,
        },
    }
}

/// The world's almanac, if it has one.
fn almanac<'a>(world: &WorldRead<'a>) -> Option<&'a Process> {
    world.processes().find(|process| {
        *process.owner() == HappeningsSystem::ID
            && *process.process_type() == AlmanacProcess::PROCESS_TYPE
    })
}

/// A process's state, read back as `P`'s.
fn state_of<P: ProcessKind, S: DeserializeOwned>(process: &Process) -> Result<S, KernelError> {
    let bytes = process.state_for::<P>()?;
    serde_json::from_slice(bytes).map_err(|_| KernelError::ProcessNotRunning {
        process: process.id(),
    })
}

/// The almanac — its id and the instant it was started — and its state, if the world is configured.
fn configured_almanac(world: &WorldRead<'_>) -> Result<Option<Almanac>, KernelError> {
    almanac(world)
        .map(|process| {
            Ok(Almanac {
                id: process.id(),
                started: process.started(),
                state: state_of::<AlmanacProcess, _>(process)?,
            })
        })
        .transpose()
}

/// The almanac as a reaction needs it.
struct Almanac {
    id: ProcessId,
    started: WorldTime,
    state: AlmanacState,
}

/// Every happening process and its state, in process order.
fn happenings<'a>(world: &WorldRead<'a>) -> Vec<(&'a Process, HappeningState)> {
    world
        .processes()
        .filter(|process| {
            *process.owner() == HappeningsSystem::ID
                && *process.process_type() == HappeningProcess::PROCESS_TYPE
        })
        .filter_map(|process| {
            state_of::<HappeningProcess, HappeningState>(process)
                .ok()
                .map(|state| (process, state))
        })
        .collect()
}

/// The process of `happening`, and its state.
fn happening_process(
    world: &WorldRead<'_>,
    happening: &HappeningRef,
) -> Option<(ProcessId, HappeningState)> {
    happenings(world)
        .into_iter()
        .find(|(_, state)| state.happening() == happening)
        .map(|(process, state)| (process.id(), state))
}

/// Who hears a happening begin and end: everybody, or those at its place.
fn audience(configured: &HappeningsConfigured, happening: &HappeningRef) -> Visibility {
    match configured
        .kinds()
        .get(happening.kind())
        .map(|kind| kind.audience())
    {
        Some(Audience::Place) => Visibility::Place(happening.place()),
        _ => Visibility::Public,
    }
}

/// A lifecycle fact about `happening`'s place, at it.
pub(crate) fn about_place<E: Event>(
    fact: &impl Serialize,
    happening: &HappeningRef,
    to: Visibility,
) -> Emission {
    Emission::new::<E>(encode(fact), to)
        .about(vec![happening.place().entity_id()])
        .at_place(happening.place())
}

impl System for HappeningsSystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    /// Depends on presence to know where an observer is, and on calendar for its days (calendar never
    /// knows happenings). Provides no action in WE-a: nobody does anything to make it market day.
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([PresenceSystem::ID, CalendarSystem::ID])
            .owning::<HappeningsHere>()
            .owning::<HappeningsAnnounced>()
            .emitting::<HappeningsConfigured>()
            .emitting::<HappeningAnnounced>()
            .emitting::<HappeningBegan>()
            .emitting::<HappeningEnded>()
            .emitting::<HappeningWithheld>()
            .subscribing_to::<HappeningsConfigured>()
            .subscribing_to::<DayBegan>()
            .subscribing_to::<HappeningAnnounced>()
            .subscribing_to::<HappeningBegan>()
            .subscribing_to::<HappeningEnded>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<HappeningsHere>()?;
        tables.component::<HappeningsAnnounced>()
    }

    /// `happenings-configured` → the almanac. `day-began` → the pass (nothing when unconfigured).
    /// `happening-announced` → folded into the almanac, and the happening's process started, due at its
    /// start (or begun at once when it starts now). `happening-began` → running, due at its end.
    /// `happening-ended` → its process ended and the almanac folded. These are the only writes.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let kind = event.event_type().clone();
        if kind == HappeningsConfigured::EVENT_TYPE {
            let configured: HappeningsConfigured = payload(event.payload())?;
            if almanac(&world.read()).is_some() {
                return Err(refused(kind, "happenings-configured-twice"));
            }
            world.start_process(
                ProcessStart::<AlmanacProcess>::new(encode(&AlmanacState::new(configured)))
                    .uninterruptible(),
            )?;
            return Ok(Vec::new());
        }
        let Some(almanac) = configured_almanac(&world.read())? else {
            if kind == DayBegan::EVENT_TYPE {
                // Enabled but not configured: no almanac, no happenings (INV-WE-5).
                return Ok(Vec::new());
            }
            return Err(refused(kind, "happenings-unconfigured"));
        };
        if kind == DayBegan::EVENT_TYPE {
            let began: DayBegan = payload(event.payload())?;
            let (now, read) = (world.at(), world.read());
            return Ok(pass(&read, now, almanac.started, &almanac.state, &began));
        }
        if kind == HappeningAnnounced::EVENT_TYPE {
            let announced: HappeningAnnounced = payload(event.payload())?;
            return announce(world, almanac.id, almanac.state, &announced);
        }
        if kind == HappeningBegan::EVENT_TYPE {
            let began: HappeningBegan = payload(event.payload())?;
            let Some((id, current)) = happening_process(&world.read(), began.happening()) else {
                return Err(refused(kind, "happening-unknown"));
            };
            world.reschedule_process::<HappeningProcess>(id, Some(current.until()))?;
            world.set_process_state::<HappeningProcess>(id, encode(&current.running()))?;
            return Ok(Vec::new());
        }
        let ended: HappeningEnded = payload(event.payload())?;
        let Some((id, _)) = happening_process(&world.read(), ended.happening()) else {
            return Err(refused(kind, "happening-unknown"));
        };
        world.end_process::<HappeningProcess>(id)?;
        let folded = almanac.state.with_ended(ended.happening());
        world.set_process_state::<AlmanacProcess>(almanac.id, encode(&folded))?;
        Ok(Vec::new())
    }

    /// A happening due: announced → it begins; running → it ends.
    fn wake(
        &self,
        world: &mut WorldView<'_, Self>,
        process: &Process,
    ) -> Result<Vec<Emission>, KernelError> {
        let current: HappeningState = state_of::<HappeningProcess, _>(process)?;
        let Some(almanac) = configured_almanac(&world.read())? else {
            return Err(refused(
                HappeningBegan::EVENT_TYPE,
                "happenings-unconfigured",
            ));
        };
        let to = audience(almanac.state.configured(), current.happening());
        let fact = match current.phase() {
            HappeningPhase::Announced => about_place::<HappeningBegan>(
                &HappeningBegan::new(current.happening().clone(), current.until()),
                current.happening(),
                to,
            ),
            HappeningPhase::Running => about_place::<HappeningEnded>(
                &HappeningEnded::new(current.happening().clone()),
                current.happening(),
                to,
            ),
        };
        Ok(vec![fact])
    }
}

/// `happening-announced` folded: the almanac gains it, and its process starts at its place, due at its
/// start — or, when it starts this instant, due at its end with `happening-began` stated at once, since
/// a process cannot be due at the instant it starts (`INV-3`).
fn announce(
    world: &mut WorldView<'_, HappeningsSystem>,
    almanac: ProcessId,
    state: AlmanacState,
    announced: &HappeningAnnounced,
) -> Result<Vec<Emission>, KernelError> {
    let happening = announced.happening().clone();
    let begins_now = happening.start() <= world.at();
    let due = if begins_now {
        announced.until()
    } else {
        happening.start()
    };
    let to = audience(state.configured(), &happening);
    world.set_process_state::<AlmanacProcess>(
        almanac,
        encode(&state.with_announced(announced.open())),
    )?;
    world.start_process(
        ProcessStart::<HappeningProcess>::new(encode(&HappeningState::announced(
            happening.clone(),
            announced.until(),
        )))
        .at_place(happening.place())
        .ending_at(due)
        .uninterruptible(),
    )?;
    if !begins_now {
        return Ok(Vec::new());
    }
    Ok(vec![about_place::<HappeningBegan>(
        &HappeningBegan::new(happening.clone(), announced.until()),
        &happening,
        to,
    )])
}

impl PerceptionProvider for HappeningsSystem {
    /// Two records about **the place the observer is in**, and about nothing else: the happenings
    /// running there now, and the happenings announced and not yet begun anywhere (public news). Each
    /// only when it lists something. An observer in no place gets neither.
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
        let Ok(Some(Almanac { state, .. })) = configured_almanac(world) else {
            return Vec::new();
        };
        let shown = |current: &HappeningState| ShownHappening {
            happening: current.happening().clone(),
            title: state
                .configured()
                .kinds()
                .get(current.happening().kind())
                .and_then(|kind| kind.title().cloned()),
            until: current.until(),
        };
        let mut running: Vec<ShownHappening> = Vec::new();
        let mut announced: Vec<ShownHappening> = Vec::new();
        for (_, current) in happenings(world).into_iter() {
            match current.phase() {
                HappeningPhase::Running if current.happening().place().entity_id() == subject => {
                    running.push(shown(&current));
                }
                HappeningPhase::Announced => announced.push(shown(&current)),
                HappeningPhase::Running => {}
            }
        }
        running.sort_by(|a, b| {
            (a.happening.start(), a.happening.kind())
                .cmp(&(b.happening.start(), b.happening.kind()))
        });
        announced.sort_by(|a, b| {
            let key = |shown: &ShownHappening| {
                (
                    shown.happening.start(),
                    shown.happening.place(),
                    shown.happening.kind().clone(),
                )
            };
            key(a).cmp(&key(b))
        });
        let mut records = Vec::new();
        if !running.is_empty() {
            records.push(ComponentRecord::new::<HappeningsHere>(
                subject,
                to_value(&HappeningsHere(running)),
            ));
        }
        if !announced.is_empty() {
            records.push(ComponentRecord::new::<HappeningsAnnounced>(
                subject,
                to_value(&HappeningsAnnounced(announced)),
            ));
        }
        records
    }
}

/// A disclosed record as the self-describing value an observation carries.
fn to_value<T: Serialize>(record: &T) -> Value {
    serde_json::to_value(record)
        .expect("this pack's records are JSON-representable by construction")
}
