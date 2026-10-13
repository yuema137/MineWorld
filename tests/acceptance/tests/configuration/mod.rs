//! Test-only System Packs that take a world-level configuration (`docs/DECISIONS.md` `ARC-61`,
//! `ARC-63`; step-18-interaction-list §11 IA-C7, §12 IB-C7).
//!
//! ```text
//! test-tuning   its configuration is its section of the World's Interaction List (interactions!()):
//!               parameters { step 1 … 100 = 1 }; the action `advance` (roles actor, target;
//!               regional); the fact `advanced` (actor = subjects[0], target = subjects[1]; audience
//!               Place, narrowest Participants; biography configurable, compiled on); reference lists
//!               `default`, `gentle` (step 2), a cycle (`loop-a` ⇄ `loop-b`) and a chain five long
//!               (`deep-1` … `deep-5`) for the extends refusals
//!   section     `tuned:` on a person file, { start, at? }: where a person's count starts and where
//!               they are; its reduction refuses a start that is not a multiple of the step there, and
//!               a world with no configured section — so it must be reduced after the configuration
//!   action      `advance` [at a target]: refused by the list (PermissionDenied) before anything
//!               else; states `advanced { person, by: step }` with the audience the list routes
//!   perception  offers `advance` against every other tuned person, refused when the list forbids it
//! test-table    a plain configuration { table: data/<file> }: the attachment's rows of three
//!               integers, decoded by the pack and stated in `table-configured`
//! ```
//!
//! Neither names a word of any installed pack's vocabulary: they exist to show the seam and the
//! schema carry what a pack types, seeds, checks against and acts on, and nothing else.

#![allow(dead_code)]

use std::collections::BTreeMap;

use mineworld_authoring::{
    Attached, Attachment, AuthoredSection, ConfigurationContext, ContentKind, Decode,
    EntityClasses, PackConfiguration, Reference, SectionName, Seeding,
};
use mineworld_contracts::{
    Action, ActionIntent, ActionTypeId, ComponentTypeId, EntityId, EntityKey, EntityType, Event,
    EventEnvelope, EventSchemaVersion, EventTypeId, PlaceId, Rejection, RejectionCode,
    SpatialRequirement, SystemId, Tag, Tags, Visibility,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    World, WorldRead, WorldView, owned_component,
};
use mineworld_presence::{Offer, PerceptionProvider};
use mineworld_sdk::SystemPack;
use mineworld_sdk::interactions::{
    self, ActionDecl, Audience, FactDecl, InteractionSection, Interactions, Position, Resolved,
    Role, Roles, Section,
};
use serde::de::DeserializeSeed;
use serde::{Deserialize, Serialize};

/// The pack.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Tuning;

impl SystemIdentity for Tuning {
    const ID: SystemId = SystemId::from_static("test-tuning");
}

mineworld_sdk::parameters! {
    /// What a world may choose about test-tuning.
    pub struct TuningParameters, partial TuningPartial {
        /// How far one `advance` moves a count.
        step: u32 = 1, 1 ..= 100;
    }
}

/// The action and the fact a list may govern.
pub const ADVANCE: ActionTypeId = ActionTypeId::from_static("advance");
pub const ADVANCED: EventTypeId = EventTypeId::from_static("advanced");

/// The section: where a person's count starts, and where they are (the place a lookup asks about).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Start {
    start: u32,
    #[serde(default)]
    at: Option<EntityKey>,
}

/// `tuned`: where one person's count starts, and where.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tuned {
    pub person: EntityId,
    pub start: u32,
    pub place: EntityId,
}

impl Event for Tuned {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("tuned");
    const OWNER: SystemId = Tuning::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// `advanced`: a person's count moved by the step that applied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Advanced {
    pub person: EntityId,
    pub by: u32,
}

impl Event for Advanced {
    const EVENT_TYPE: EventTypeId = ADVANCED;
    const OWNER: SystemId = Tuning::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// The request: advance my count (at a target, optionally).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Advance {}

impl Action for Advance {
    const ACTION_TYPE: ActionTypeId = ADVANCE;
    const OWNER: SystemId = Tuning::ID;
}

/// A tuned person's count, and where they are.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Count {
    pub value: u32,
    pub place: EntityId,
}

owned_component! {
    component = Count,
    owner = Tuning,
    component_type = "test-tuning-count",
    schema_version = 1,
}

fn encode<E: Event + Serialize>(fact: &E, visibility: Visibility) -> Emission {
    Emission::new::<E>(
        serde_json::to_vec(fact).expect("a fact encodes"),
        visibility,
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

fn place_id(place: EntityId) -> PlaceId {
    PlaceId::new(place, EntityType::Place).expect("a place")
}

/// Whether the world configures test-tuning's section at all.
fn configured(world: &WorldRead<'_>) -> bool {
    world.components::<Interactions<Tuning>>().next().is_some()
}

/// A named reference list with one step and an `extends`.
fn list(step: Option<u32>, extends: Option<&'static str>) -> Section<Tuning> {
    let mut section = Section::empty();
    section.extends = extends;
    if let Some(step) = step {
        section
            .entries
            .parameters
            .push(mineworld_sdk::interactions::ParamEntry {
                selectors: mineworld_sdk::interactions::Selectors::any(),
                fields: TuningPartial { step: Some(step) },
            });
    }
    section
}

impl InteractionSection for Tuning {
    type Parameters = TuningParameters;
    type Knobs = ();
    const CONFIGURED: EventTypeId = EventTypeId::from_static("test-tuning-interactions-configured");
    const COMPONENT: ComponentTypeId = ComponentTypeId::from_static("test-tuning-interactions");
    const ACTIONS: &'static [ActionDecl] = &[ActionDecl {
        action: ADVANCE,
        roles: &[Role::Actor, Role::Target],
        regional: true,
    }];
    const FACTS: &'static [FactDecl] = &[FactDecl {
        fact: ADVANCED,
        roles: &[
            (Role::Actor, Position::Subject(0)),
            (Role::Target, Position::Subject(1)),
        ],
        default_audience: Audience::Place,
        narrowest: Audience::Participants,
        biography_configurable: true,
    }];
    const PARAMETER_ROLES: &'static [Role] = &[Role::Actor, Role::Target];

    fn reference_lists() -> Vec<(&'static str, Section<Self>)> {
        vec![
            ("default", Section::empty()),
            ("gentle", list(Some(2), None)),
            ("loop-a", list(None, Some("loop-b"))),
            ("loop-b", list(None, Some("loop-a"))),
            ("deep-1", list(None, Some("deep-2"))),
            ("deep-2", list(None, Some("deep-3"))),
            ("deep-3", list(None, Some("deep-4"))),
            ("deep-4", list(None, Some("deep-5"))),
            ("deep-5", list(Some(3), None)),
        ]
    }

    fn encode(resolved: &Resolved<Self>) -> Vec<u8> {
        serde_json::to_vec(resolved).expect("a resolved section encodes")
    }

    fn decode(payload: &[u8]) -> Result<Resolved<Self>, String> {
        serde_json::from_slice(payload).map_err(|error| error.to_string())
    }
}

impl AuthoredSection for Tuning {
    const SECTION: SectionName = SectionName::from_static("tuned");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Person];
    type Authored = Start;

    fn references(authored: &Start) -> Vec<Reference<'_>> {
        authored
            .at
            .iter()
            .map(|key| Reference {
                key,
                entity_type: EntityType::Place,
            })
            .collect()
    }

    fn seed(
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
        authored: &Start,
    ) -> Result<Vec<Emission>, Rejection> {
        let square = EntityKey::new("square").expect("a key");
        let at = authored.at.as_ref().unwrap_or(&square);
        let place = seeding
            .resolve(at, EntityType::Place)
            .ok_or(Rejection::PreconditionFailed)?;
        Ok(vec![encode(
            &Tuned {
                person: subject,
                start: authored.start,
                place,
            },
            Visibility::SystemInternal,
        )])
    }
}

impl SystemPack for Tuning {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    const BIOGRAPHICAL: &'static [EventTypeId] = &[ADVANCED];
    mineworld_sdk::owns_section!();
    mineworld_sdk::interactions!();
}

/// The roles of a request: its actor, and its target if it has one.
fn roles(intent: &ActionIntent) -> Roles {
    let roles = Roles::new().with(Role::Actor, intent.actor());
    match intent.target() {
        Some(target) => roles.with(Role::Target, target),
        None => roles,
    }
}

impl System for Tuning {
    const VERSION: SystemVersion = SystemVersion::new(2);

    fn declaration(&self) -> SystemDeclaration {
        interactions::declare::<Self>(
            SystemDeclaration::of::<Self>()
                .owning::<Count>()
                .providing::<Advance>()
                .emitting::<Tuned>()
                .emitting::<Advanced>()
                .subscribing_to::<Tuned>()
                .subscribing_to::<Advanced>(),
        )
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        interactions::install::<Self>(tables)?;
        tables.component::<Count>()
    }

    /// Only a tuned person may advance, only in a world whose section is configured, and only where
    /// the list permits it — the list's answer first, before any other condition of the pack's.
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        intent
            .payload()
            .payload_for::<Advance>()
            .map_err(|_| Rejection::PreconditionFailed)?;
        let Some(count) = world.component::<Count>(intent.actor()) else {
            return Err(Rejection::PreconditionFailed);
        };
        if let Some(target) = intent.target()
            && world.entity(target).is_none()
        {
            return Err(Rejection::TargetUnavailable);
        }
        interactions::permits::<Self>(world, place_id(count.place), &ADVANCE, &roles(intent))?;
        if !configured(world) {
            return Err(Rejection::PreconditionFailed);
        }
        Ok(())
    }

    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let read = world.read();
        let place = place_id(
            read.component::<Count>(intent.actor())
                .expect("validated")
                .place,
        );
        let roles = roles(intent);
        let by = interactions::parameters::<Self>(&read, place, &roles).step;
        let routed = interactions::consequence::<Self>(
            &read,
            Some(place),
            &ADVANCED,
            &roles,
            Visibility::Place(place),
        );
        let mut subjects = vec![intent.actor()];
        subjects.extend(intent.target());
        Ok(vec![
            encode(
                &Advanced {
                    person: intent.actor(),
                    by,
                },
                routed.visibility,
            )
            .about(subjects.clone())
            .with_participants(subjects)
            .at_place(place),
        ])
    }

    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if interactions::reduce(world, event)? {
            return Ok(Vec::new());
        }
        let kind = event.event_type();
        if *kind == Tuned::EVENT_TYPE {
            let tuned: Tuned = decoded(event)?;
            if !configured(&world.read()) {
                return Err(refused(Tuned::EVENT_TYPE, "tuning-unconfigured"));
            }
            let roles = Roles::new().with(Role::Actor, tuned.person);
            let step =
                interactions::parameters::<Self>(&world.read(), place_id(tuned.place), &roles).step;
            if !tuned.start.is_multiple_of(step) {
                return Err(refused(Tuned::EVENT_TYPE, "tuning-start-off-step"));
            }
            world.insert(
                tuned.person,
                Count {
                    value: tuned.start,
                    place: tuned.place,
                },
            )?;
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

impl PerceptionProvider for Tuning {
    /// `advance` against every other tuned person, requiring nothing of space; refused when the
    /// world's list forbids it (`Offer::refused`, `ARC-34` note), through the same `permits` the
    /// dispatch asks.
    fn offers(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        target: Option<EntityId>,
    ) -> Vec<Offer> {
        let (Some(target), Some(count)) = (target, world.component::<Count>(observer)) else {
            return Vec::new();
        };
        if target == observer || world.component::<Count>(target).is_none() {
            return Vec::new();
        }
        let roles = Roles::new()
            .with(Role::Actor, observer)
            .with(Role::Target, target);
        let offer = Offer::new::<Advance>(SpatialRequirement::NONE);
        vec![
            match interactions::permits::<Self>(world, place_id(count.place), &ADVANCE, &roles) {
                Ok(()) => offer,
                Err(reason) => offer.refused(reason),
            },
        ]
    }
}

// ---- test-table: a plain configuration with a data: attachment ---------------------------------

/// The pack.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Table;

impl SystemIdentity for Table {
    const ID: SystemId = SystemId::from_static("test-table");
}

/// The configuration: the file the rows are in.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableFile {
    table: Attachment,
}

/// `table-configured`: the rows, as the pack decoded them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableConfigured {
    pub rows: Vec<[u32; 3]>,
}

impl Event for TableConfigured {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("table-configured");
    const OWNER: SystemId = Table::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

fn table_refused(detail: String) -> Rejection {
    Rejection::System {
        code: RejectionCode::from_static("table-unreadable"),
        detail: Some(detail),
    }
}

/// Three integers a line, comma-separated, with either line ending; blank lines skipped.
pub fn rows(bytes: &[u8]) -> Result<Vec<[u32; 3]>, String> {
    let text = std::str::from_utf8(bytes).map_err(|error| error.to_string())?;
    text.lines()
        .map(|line| line.trim_end_matches('\r'))
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let values: Vec<u32> = line
                .split(',')
                .map(|value| {
                    value
                        .trim()
                        .parse::<u32>()
                        .map_err(|e| format!("{line}: {e}"))
                })
                .collect::<Result<_, _>>()?;
            <[u32; 3]>::try_from(values).map_err(|_| format!("{line}: not three values"))
        })
        .collect()
}

impl PackConfiguration for Table {
    type Configuration = TableFile;
    const FACTS: &'static [EventTypeId] = &[TableConfigured::EVENT_TYPE];

    fn attachments(configuration: &TableFile) -> Vec<&Attachment> {
        vec![&configuration.table]
    }

    fn seed(
        _: &Seeding<'_, '_>,
        configuration: &TableFile,
        context: &ConfigurationContext<'_>,
    ) -> Result<Vec<Emission>, Rejection> {
        let bytes = context
            .attached()
            .get(&configuration.table)
            .ok_or_else(|| table_refused(format!("{} was not read", configuration.table)))?;
        let rows = rows(bytes).map_err(table_refused)?;
        Ok(vec![encode(
            &TableConfigured { rows },
            Visibility::SystemInternal,
        )])
    }
}

impl SystemPack for Table {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    mineworld_sdk::configures!();
}

impl System for Table {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>().emitting::<TableConfigured>()
    }
}

// ---- assembling a world through the seam, as the loader does ---------------------------------

/// What a world is configured with: its classes (`configure/classes.yaml`), test-tuning's section
/// (`configure/test-tuning.yaml`), and the people's `tuned:` sections (person, start, place).
#[derive(Debug, Clone, Default)]
pub struct Setup<'a> {
    pub classes: Option<&'a str>,
    pub section: Option<&'a str>,
    pub starts: Vec<(&'a str, u32, &'a str)>,
    pub sections_first: bool,
}

/// The genesis facts `setup` seeds, through the seam exactly as the loader decodes, checks and seeds
/// them — the section with the pack's `decode_configuration` and checked against the classes,
/// `tuned:` with authoring's `Decode` — in the loader's order, configuration first unless
/// `sections_first`. A refusal is its text, with line and column where the decoder reports one.
pub fn genesis(
    world: &World,
    ids: &BTreeMap<EntityKey, EntityId>,
    setup: &Setup<'_>,
) -> Result<Vec<Emission>, String> {
    let read = world.read();
    let seeding = Seeding::new(&read, ids);
    let classes: EntityClasses = match setup.classes {
        Some(text) => serde_saphyr::from_str(text).map_err(|error| error.to_string())?,
        None => EntityClasses::default(),
    };
    let attached = Attached::none();
    let context = ConfigurationContext::new(&classes, &attached);
    let mut configured = Vec::new();
    if let Some(text) = setup.section {
        let decoded = serde_saphyr::with_deserializer_from_str(text, |file| {
            Tuning::decode_configuration(file)
        })
        .map_err(|error| error.to_string())?;
        decoded
            .check(&context)
            .map_err(|refusal| format!("{refusal:?}"))?;
        configured = decoded
            .seed(&seeding, &context)
            .map_err(|error| format!("{error:?}"))?;
    }
    let mut sections = Vec::new();
    for (person, start, place) in &setup.starts {
        let text = format!("start: {start}\nat: {place}\n");
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
    Ok(if setup.sections_first {
        sections.into_iter().chain(configured).collect()
    } else {
        configured.into_iter().chain(sections).collect()
    })
}

/// IL-a's form: a section text (or none) and starts in the square.
pub fn genesis_facts(
    world: &World,
    ids: &BTreeMap<EntityKey, EntityId>,
    section: Option<&str>,
    starts: &[(&str, u32)],
    sections_first: bool,
) -> Result<Vec<Emission>, String> {
    genesis(
        world,
        ids,
        &Setup {
            classes: None,
            section,
            starts: starts
                .iter()
                .map(|(person, start)| (*person, *start, "square"))
                .collect(),
            sections_first,
        },
    )
}

/// The world with `test-tuning` installed and nothing else: what a save is resumed into.
pub fn composed() -> World {
    let mut world = World::new();
    world.install(Tuning).expect("test-tuning installs");
    world
}

/// A world with `test-tuning` installed: `places` and `people`, each with its tags, every entity in
/// key order within its kind as the loader allocates them (places first).
pub fn world_with(
    places: &[(&str, &[&str])],
    people: &[(&str, &[&str])],
) -> (World, BTreeMap<EntityKey, EntityId>) {
    let mut world = composed();
    let mut ids = BTreeMap::new();
    for (list, entity_type) in [(places, EntityType::Place), (people, EntityType::Person)] {
        let mut sorted: Vec<&(&str, &[&str])> = list.iter().collect();
        sorted.sort_unstable_by_key(|(key, _)| *key);
        for (key, tags) in sorted {
            let key = EntityKey::new(*key).expect("a key");
            let tags = Tags::new(tags.iter().map(|tag| Tag::new(*tag).expect("a tag")));
            let id = world
                .create_authored_entity(key.clone(), entity_type, tags, None)
                .expect("an entity");
            ids.insert(key, id);
        }
    }
    (world, ids)
}

/// IL-a's world: one Place, `square`, and the people named, untagged.
pub fn world(people: &[&str]) -> (World, BTreeMap<EntityKey, EntityId>) {
    let people: Vec<(&str, &[&str])> = people.iter().map(|person| (*person, &[][..])).collect();
    world_with(&[("square", &[])], &people)
}
