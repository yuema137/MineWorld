//! `test-tuning`: a System Pack defined only in tests, that takes a world-level configuration
//! (`docs/DECISIONS.md` `ARC-61`; step-18-interaction-list §11, IA-C7).
//!
//! ```text
//! configuration   configure/test-tuning.yaml: { step: 1 … 100 }, decoded by the pack's own type
//!                 and seeded as `tuning-configured { step }`, SystemInternal, no subjects; reduced
//!                 into a `Stride` component on every Place
//! section         `tuned:` on a person file, { start }, seeded as `tuned { person, start }`; its
//!                 reduction refuses a start that is not a multiple of the configured step, and a
//!                 world with no configured step — so it must be reduced after the configuration
//! action          `advance`, by a tuned person: states `advanced { person, by: step }`, reduced into
//!                 the person's `Count`
//! ```
//!
//! It names no word of any installed pack's vocabulary: it exists to show the seam carries a value a
//! pack types, seeds, checks against and acts on, and nothing else.

#![allow(dead_code)]

use std::collections::BTreeMap;

use mineworld_authoring::{
    AuthoredSection, ContentKind, Decode, PackConfiguration, SectionName, Seeding,
};
use mineworld_contracts::{
    Action, ActionIntent, ActionTypeId, EntityId, EntityKey, EntityType, Event, EventEnvelope,
    EventSchemaVersion, EventTypeId, Rejection, RejectionCode, SystemId, Visibility,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    World, WorldRead, WorldView, owned_component,
};
use mineworld_sdk::SystemPack;
use serde::de::DeserializeSeed;
use serde::{Deserialize, Serialize};

/// The pack.
#[derive(Default)]
pub struct Tuning;

impl SystemIdentity for Tuning {
    const ID: SystemId = SystemId::from_static("test-tuning");
}

/// The configuration: the step every count moves by.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configured {
    step: Step,
}

/// 1 … 100, refused otherwise by decoding.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(try_from = "u32")]
pub struct Step(u32);

impl TryFrom<u32> for Step {
    type Error = String;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if (1..=100).contains(&value) {
            Ok(Self(value))
        } else {
            Err(format!("a test-tuning step is 1 to 100, not {value}"))
        }
    }
}

/// The section: where a person's count starts.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Start {
    start: u32,
}

/// `tuning-configured`: the world's step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TuningConfigured {
    pub step: u32,
}

impl Event for TuningConfigured {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("tuning-configured");
    const OWNER: SystemId = Tuning::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// `tuned`: where one person's count starts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tuned {
    pub person: EntityId,
    pub start: u32,
}

impl Event for Tuned {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("tuned");
    const OWNER: SystemId = Tuning::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// `advanced`: a person's count moved by the configured step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Advanced {
    pub person: EntityId,
    pub by: u32,
}

impl Event for Advanced {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("advanced");
    const OWNER: SystemId = Tuning::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// The request: advance my count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Advance {}

impl Action for Advance {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("advance");
    const OWNER: SystemId = Tuning::ID;
}

/// The configured step, on every Place.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stride {
    pub step: u32,
}

owned_component! {
    component = Stride,
    owner = Tuning,
    component_type = "test-tuning-stride",
    schema_version = 1,
}

/// A tuned person's count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Count {
    pub value: u32,
}

owned_component! {
    component = Count,
    owner = Tuning,
    component_type = "test-tuning-count",
    schema_version = 1,
}

fn encode<E: Event + Serialize>(fact: &E) -> Emission {
    Emission::new::<E>(
        serde_json::to_vec(fact).expect("a fact encodes"),
        Visibility::SystemInternal,
    )
}

fn decoded<E: Event + for<'de> Deserialize<'de>>(event: &EventEnvelope) -> Result<E, KernelError> {
    let bytes = event.payload().payload_for::<E>()?;
    serde_json::from_slice(bytes).map_err(|_| {
        KernelError::Contract(mineworld_contracts::ContractError::EventTypeMismatch {
            expected: E::EVENT_TYPE,
            actual: event.event_type().clone(),
        })
    })
}

fn refused(event_type: EventTypeId, code: &'static str) -> KernelError {
    KernelError::FactRefusedByOwner {
        system: Tuning::ID,
        event_type,
        reason: Rejection::System {
            code: RejectionCode::from_static(code),
            detail: None,
        },
    }
}

/// The configured step anywhere in the world: the same on every Place.
fn step(world: &WorldRead<'_>) -> Option<u32> {
    world
        .components::<Stride>()
        .next()
        .map(|(_, stride)| stride.step)
}

impl PackConfiguration for Tuning {
    type Configuration = Configured;
    const FACTS: &'static [EventTypeId] = &[TuningConfigured::EVENT_TYPE];

    fn seed(_: &Seeding<'_, '_>, configured: &Configured) -> Result<Vec<Emission>, Rejection> {
        Ok(vec![encode(&TuningConfigured {
            step: configured.step.0,
        })])
    }
}

impl AuthoredSection for Tuning {
    const SECTION: SectionName = SectionName::from_static("tuned");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Person];
    type Authored = Start;

    fn seed(
        _: &Seeding<'_, '_>,
        subject: EntityId,
        authored: &Start,
    ) -> Result<Vec<Emission>, Rejection> {
        Ok(vec![encode(&Tuned {
            person: subject,
            start: authored.start,
        })])
    }
}

impl SystemPack for Tuning {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    mineworld_sdk::owns_section!();
    mineworld_sdk::configures!();
}

impl System for Tuning {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<Stride>()
            .owning::<Count>()
            .providing::<Advance>()
            .emitting::<TuningConfigured>()
            .emitting::<Tuned>()
            .emitting::<Advanced>()
            .subscribing_to::<TuningConfigured>()
            .subscribing_to::<Tuned>()
            .subscribing_to::<Advanced>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Stride>()?;
        tables.component::<Count>()
    }

    /// Only a tuned person may advance, and only in a world with a configured step.
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        intent
            .payload()
            .payload_for::<Advance>()
            .map_err(|_| Rejection::PreconditionFailed)?;
        if world.component::<Count>(intent.actor()).is_none() || step(world).is_none() {
            return Err(Rejection::PreconditionFailed);
        }
        Ok(())
    }

    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let by = step(&world.read()).expect("validated");
        Ok(vec![
            encode(&Advanced {
                person: intent.actor(),
                by,
            })
            .about(vec![intent.actor()]),
        ])
    }

    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let kind = event.event_type();
        if *kind == TuningConfigured::EVENT_TYPE {
            let configured: TuningConfigured = decoded(event)?;
            let places: Vec<EntityId> = world
                .read()
                .entities()
                .filter(|entity| entity.entity_type() == EntityType::Place)
                .map(|entity| entity.id())
                .collect();
            for place in places {
                world.insert(
                    place,
                    Stride {
                        step: configured.step,
                    },
                )?;
            }
        } else if *kind == Tuned::EVENT_TYPE {
            let tuned: Tuned = decoded(event)?;
            let Some(step) = step(&world.read()) else {
                return Err(refused(Tuned::EVENT_TYPE, "tuning-unconfigured"));
            };
            if !tuned.start.is_multiple_of(step) {
                return Err(refused(Tuned::EVENT_TYPE, "tuning-start-off-step"));
            }
            world.insert(tuned.person, Count { value: tuned.start })?;
        } else if *kind == Advanced::EVENT_TYPE {
            let advanced: Advanced = decoded(event)?;
            let Some(count) = world.component_mut::<Count>(advanced.person)? else {
                return Err(refused(Advanced::EVENT_TYPE, "tuning-untuned"));
            };
            count.value += advanced.by;
        }
        Ok(Vec::new())
    }
}

/// The genesis facts of a world configured by `configuration` (the text of `configure/test-tuning.yaml`,
/// or none) whose people carry `tuned:` sections (`starts`, by key): through the seam exactly as the
/// loader decodes and seeds them — the configuration with the pack's `decode_configuration`, a section
/// with authoring's `Decode` — and in the loader's order, configuration first unless
/// `sections_first`.
pub fn genesis_facts(
    world: &World,
    ids: &BTreeMap<EntityKey, EntityId>,
    configuration: Option<&str>,
    starts: &[(&str, u32)],
    sections_first: bool,
) -> Result<Vec<Emission>, String> {
    let read = world.read();
    let seeding = Seeding::new(&read, ids);
    let mut configured = Vec::new();
    if let Some(text) = configuration {
        let decoded = serde_saphyr::with_deserializer_from_str(text, |file| {
            Tuning::decode_configuration(file)
        })
        .map_err(|error| error.to_string())?;
        configured = decoded
            .seed(&seeding)
            .map_err(|error| format!("{error:?}"))?;
    }
    let mut sections = Vec::new();
    for (person, start) in starts {
        let text = format!("start: {start}\n");
        let content = serde_saphyr::with_deserializer_from_str(&text, |file| {
            Decode::<Tuning>::new().deserialize(file)
        })
        .map_err(|error| error.to_string())?;
        let id = ids[&EntityKey::new(*person).expect("a key")];
        sections.extend(
            content
                .seed(&seeding, id)
                .map_err(|error| format!("{error:?}"))?,
        );
    }
    Ok(if sections_first {
        sections.into_iter().chain(configured).collect()
    } else {
        configured.into_iter().chain(sections).collect()
    })
}

/// The world with `test-tuning` installed and nothing else: what a save is resumed into.
pub fn composed() -> World {
    let mut world = World::new();
    world.install(Tuning).expect("test-tuning installs");
    world
}

/// A world with `test-tuning` installed: one Place, `square`, and the people named, each entity in
/// key order as the loader allocates them.
pub fn world(people: &[&str]) -> (World, BTreeMap<EntityKey, EntityId>) {
    let mut world = composed();
    let mut ids = BTreeMap::new();
    let key = EntityKey::new("square").expect("a key");
    let id = world
        .create_entity(key.clone(), EntityType::Place)
        .expect("a place");
    ids.insert(key, id);
    let mut people: Vec<&str> = people.to_vec();
    people.sort_unstable();
    for person in people {
        let key = EntityKey::new(person).expect("a key");
        let id = world
            .create_entity(key.clone(), EntityType::Person)
            .expect("a person");
        ids.insert(key, id);
    }
    (world, ids)
}
