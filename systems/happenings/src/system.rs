//! The installable system: what it declares, the facts it folds, and what it discloses (design §5.2).

use mineworld_authoring::{ConfigurationContext, PackConfiguration, Reference, Seeding};
use mineworld_calendar::CalendarSystem;
use mineworld_contracts::{
    ContractError, EntityKey, EntityType, Event, EventEnvelope, EventRecord, EventTypeId, PlaceId,
    Rejection, RejectionCode, SystemId, Visibility,
};
use mineworld_kernel::{
    Emission, KernelError, Process, ProcessKind, ProcessStart, System, SystemDeclaration,
    SystemIdentity, SystemVersion, WorldRead, WorldView,
};
use mineworld_presence::{PerceptionProvider, PresenceSystem};
use mineworld_sdk::SystemPack;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::configuration::HappeningsConfiguration;
use crate::event::HappeningsConfigured;
use crate::process::{AlmanacProcess, AlmanacState};

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

impl System for HappeningsSystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    /// Depends on presence to know where an observer is, and on calendar for its days (calendar never
    /// knows happenings). Provides no action in WE-a.
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([PresenceSystem::ID, CalendarSystem::ID])
            .emitting::<HappeningsConfigured>()
            .subscribing_to::<HappeningsConfigured>()
    }

    /// `happenings-configured` → the almanac, open-ended and never woken. A second configuration is
    /// refused: one world has one almanac.
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
        }
        Ok(Vec::new())
    }
}

impl PerceptionProvider for HappeningsSystem {}
