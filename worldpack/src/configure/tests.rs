//! In-crate tests of `configure` with a probe configuration (`DECISIONS.md` `ARC-61`).
//!
//! No pack this build installs is configurable, so the checks after "is this key a configurable
//! system" are exercised here with a probe `PackConfiguration`, labelled with a real capability as the
//! sections' in-crate tests label theirs (step-10 F-22).

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use mineworld_authoring::{
    AuthoredConfiguration, AuthoredSection, ContentKind, Decode, DecodeConfiguration,
    PackConfiguration, Reference, SectionName, Seeding,
};
use mineworld_contracts::{
    Causation, EntityId, EntityKey, EntityType, EventEnvelope, EventId, EventSchemaVersion,
    EventTypeId, Provenance, Rejection, SystemId, Visibility, WorldTime,
};
use mineworld_kernel::{Emission, SystemIdentity};
use serde::Deserialize;
use serde::de::DeserializeSeed;

use super::{Drift, check_references, check_requires, compare, read_files};
use crate::catalog::Capability;
use crate::error::PackError;
use crate::format::{
    AuthoredLocation, AuthoredPerson, AuthoredPlace, FoundConfiguration, FoundSection, SectionState,
};
use crate::read::WorldPack;

/// The probe's configuration: a step, bounded by its own type, and optionally a place it names and
/// a system it needs.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Settings {
    step: Step,
    #[serde(default)]
    at: Option<EntityKey>,
    #[serde(default)]
    needs: Option<SystemId>,
    /// What the probe states besides its one configuration fact, to be refused.
    #[serde(default)]
    stray: Option<Stray>,
}

/// A fact a configuration must not seed.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Stray {
    /// A fact of another pack's vocabulary.
    Foreign,
    /// A fact of the probe's own vocabulary that it did not declare as a configuration fact.
    Undeclared,
}

/// The probe's configuration fact.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Configured {
    step: u32,
}

impl mineworld_contracts::Event for Configured {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("probe-configured");
    const OWNER: SystemId = Probe::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// A probe fact that is not a configuration fact: also what the probe's section states.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Sectioned {
    subject: u64,
}

impl mineworld_contracts::Event for Sectioned {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("probe-sectioned");
    const OWNER: SystemId = Probe::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// A fact of another vocabulary.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Elsewhere {}

impl mineworld_contracts::Event for Elsewhere {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("elsewhere-stated");
    const OWNER: SystemId = SystemId::from_static("elsewhere");
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

fn emission<E: mineworld_contracts::Event + serde::Serialize>(fact: &E) -> Emission {
    let payload = serde_json::to_vec(fact).expect("a probe fact encodes");
    Emission::new::<E>(payload, Visibility::SystemInternal)
}

/// 1 … 100: the owner's bound, enforced by decoding.
#[derive(Debug, Deserialize)]
#[serde(try_from = "u32")]
struct Step(u32);

impl TryFrom<u32> for Step {
    type Error = String;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if (1..=100).contains(&value) {
            Ok(Self(value))
        } else {
            Err(format!("a step is 1 to 100, not {value}"))
        }
    }
}

/// A configurable probe pack.
pub(super) struct Probe;

impl SystemIdentity for Probe {
    const ID: SystemId = SystemId::from_static("probe");
}

impl PackConfiguration for Probe {
    type Configuration = Settings;
    const FACTS: &'static [EventTypeId] = &[<Configured as mineworld_contracts::Event>::EVENT_TYPE];

    fn references(settings: &Settings) -> Vec<Reference<'_>> {
        settings
            .at
            .iter()
            .map(|key| Reference {
                key,
                entity_type: EntityType::Place,
            })
            .collect()
    }

    fn requires(settings: &Settings) -> Vec<SystemId> {
        settings.needs.iter().cloned().collect()
    }

    /// States `probe-configured { step }`, `SystemInternal`; refuses a step of 100 — a refusal only the
    /// owner can state, at genesis.
    fn seed(_: &Seeding<'_, '_>, settings: &Settings) -> Result<Vec<Emission>, Rejection> {
        if settings.step.0 == 100 {
            return Err(Rejection::System {
                code: mineworld_contracts::RejectionCode::from_static("probe-step-max"),
                detail: None,
            });
        }
        let mut facts = vec![emission(&Configured {
            step: settings.step.0,
        })];
        match settings.stray {
            Some(Stray::Foreign) => facts.push(emission(&Elsewhere {})),
            Some(Stray::Undeclared) => facts.push(emission(&Sectioned { subject: 0 })),
            None => {}
        }
        Ok(facts)
    }
}

/// The probe also owns a person section, `{}`, stating `probe-sectioned` about its subject: what a
/// section's reduction would check against the configured state.
impl AuthoredSection for Probe {
    const SECTION: SectionName = SectionName::from_static("probe");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Person];
    type Authored = serde_json::Value;

    fn seed(
        _: &Seeding<'_, '_>,
        subject: EntityId,
        _: &serde_json::Value,
    ) -> Result<Vec<Emission>, Rejection> {
        Ok(vec![emission(&Sectioned {
            subject: subject.raw(),
        })])
    }
}

fn key(value: &str) -> EntityKey {
    EntityKey::new(value).expect("a valid key")
}

/// A probe configuration from JSON, labelled with `owner`.
fn found(owner: Capability, value: serde_json::Value) -> FoundConfiguration {
    FoundConfiguration {
        owner,
        path: PathBuf::from(format!("in-memory/configure/{owner}.yaml")),
        configuration: DecodeConfiguration::<Probe>::new()
            .deserialize(value)
            .expect("a probe configuration decodes"),
    }
}

/// A scratch pack root under the system's temporary directory, removed on drop.
struct ScratchRoot(PathBuf);

impl ScratchRoot {
    fn new(name: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("mineworld-il-a-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("configure")).expect("a writable temporary directory");
        Self(root)
    }
}

impl Drop for ScratchRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn decode_probe(
    _: Capability,
    text: &str,
) -> Result<Arc<dyn AuthoredConfiguration>, serde_saphyr::Error> {
    serde_saphyr::with_deserializer_from_str(text, |file| {
        DecodeConfiguration::<Probe>::new().deserialize(file)
    })
}

/// A listed file that is absent is refused, naming the system and the path; a present one is decoded
/// by its owner's type through the YAML stream, so the owner's own refusal keeps line and column.
#[test]
fn files_are_required_and_decoded_by_the_owner_with_line_and_column() {
    let scratch = ScratchRoot::new("files");
    let owners = [Capability::Presence];
    match read_files(&scratch.0, &owners, decode_probe) {
        Err(PackError::ConfigurationFileMissing { system, path }) => {
            assert_eq!(system, SystemId::from_static("presence"));
            assert_eq!(path, scratch.0.join("configure/presence.yaml"));
        }
        other => panic!("expected ConfigurationFileMissing, got {other:?}"),
    }

    let file = scratch.0.join("configure/presence.yaml");
    std::fs::write(&file, "# a probe\nstep: 0\n").expect("writable");
    match read_files(&scratch.0, &owners, decode_probe) {
        Err(PackError::Malformed { path, detail, .. }) => {
            assert_eq!(path, file);
            assert!(
                detail.contains("line 2 column 7") && detail.contains("a step is 1 to 100, not 0"),
                "the owner's message, at its line and column: {detail}"
            );
        }
        other => panic!("expected Malformed, got {other:?}"),
    }

    std::fs::write(&file, "step: 7\nat: square\n").expect("writable");
    let found = read_files(&scratch.0, &owners, decode_probe).expect("a valid configuration");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].owner, Capability::Presence);
    assert_eq!(
        found[0].configuration.owner(),
        SystemId::from_static("probe")
    );
}

/// A system a configuration needs must be enabled.
#[test]
fn a_configuration_that_needs_a_system_the_world_does_not_enable_is_refused() {
    let configured = [found(
        Capability::Presence,
        serde_json::json!({ "step": 5, "needs": "movement" }),
    )];
    match check_requires(&configured, &[Capability::Presence]) {
        Err(PackError::ConfigurationRequiresSystem { requires, by, .. }) => {
            assert_eq!(requires, SystemId::from_static("movement"));
            assert_eq!(by, SystemId::from_static("presence"));
        }
        other => panic!("expected ConfigurationRequiresSystem, got {other:?}"),
    }
    check_requires(&configured, &[Capability::Presence, Capability::Movement])
        .expect("movement is enabled");
}

/// Every entity a configuration names is declared, of the type its owner needs.
#[test]
fn a_configuration_that_names_an_undeclared_or_mistyped_entity_is_refused() {
    let pack = |at: &str| {
        WorldPack::in_memory(
            vec![Capability::Presence],
            BTreeMap::from([(key("square"), AuthoredPlace::default())]),
            BTreeMap::from([(key("ada"), AuthoredPerson::default())]),
            BTreeMap::new(),
            BTreeMap::new(),
        )
        .with_configuration(vec![found(
            Capability::Presence,
            serde_json::json!({ "step": 5, "at": at }),
        )])
    };
    check_references(&pack("square")).expect("a declared place");
    for (at, why) in [("nowhere", "undeclared"), ("ada", "a person, not a place")] {
        match check_references(&pack(at)) {
            Err(PackError::ConfigurationNamesUnknownEntity {
                system,
                key: named,
                expected,
                ..
            }) => {
                assert_eq!(system, SystemId::from_static("presence"));
                assert_eq!(named, key(at));
                assert_eq!(expected, EntityType::Place);
            }
            other => panic!("{why}: expected ConfigurationNamesUnknownEntity, got {other:?}"),
        }
    }
}

// ---- Seeding (SD-IA-9) and drift (SD-IA-10) -------------------------------------------------

/// A pack with one place, one located person carrying the probe's section, and `configuration`.
fn seeded_pack(configuration: Vec<FoundConfiguration>) -> WorldPack {
    let section = Decode::<Probe>::new()
        .deserialize(serde_json::json!({}))
        .expect("the probe section decodes");
    let person = AuthoredPerson {
        location: Some(AuthoredLocation {
            place: key("square"),
            position: None,
            facing: None,
        }),
        sections: vec![FoundSection {
            owner: Capability::Naming,
            name: <Probe as AuthoredSection>::SECTION,
            state: SectionState::Decoded(section),
        }],
        ..AuthoredPerson::default()
    };
    WorldPack::in_memory(
        vec![Capability::Presence, Capability::Naming],
        BTreeMap::from([(key("square"), AuthoredPlace::default())]),
        BTreeMap::from([(key("ada"), person)]),
        BTreeMap::new(),
        BTreeMap::new(),
    )
    .with_configuration(configuration)
}

fn event_types(facts: &[Emission]) -> Vec<&str> {
    facts
        .iter()
        .map(|fact| fact.event_type().as_str())
        .collect()
}

/// IA-2: configuration facts follow every location and precede every section, in `configure:` order;
/// a world with no configuration seeds exactly locations and sections.
#[test]
fn configuration_is_seeded_after_locations_and_before_sections_in_configure_order() {
    let unconfigured = seeded_pack(Vec::new()).assemble().expect("assembles");
    assert_eq!(
        event_types(&unconfigured.facts),
        ["arrived", "probe-sectioned"]
    );

    let configured = seeded_pack(vec![
        found(Capability::Presence, serde_json::json!({ "step": 9 })),
        found(Capability::Naming, serde_json::json!({ "step": 4 })),
    ])
    .assemble()
    .expect("assembles");
    assert_eq!(
        event_types(&configured.facts),
        [
            "arrived",
            "probe-configured",
            "probe-configured",
            "probe-sectioned"
        ]
    );
    assert_eq!(configured.facts[1], emission(&Configured { step: 9 }));
    assert_eq!(configured.facts[2], emission(&Configured { step: 4 }));
    assert_eq!(configured.facts[0], unconfigured.facts[0]);
    assert_eq!(configured.facts[3], unconfigured.facts[1]);
}

/// The owner's refusal, a fact of another vocabulary, and a fact the owner did not declare as a
/// configuration fact are each refused at genesis, naming the system and the file.
#[test]
fn seeding_refuses_the_owners_refusal_another_vocabulary_and_an_undeclared_fact() {
    let refusal = |value| {
        seeded_pack(vec![found(Capability::Presence, value)])
            .assemble()
            .err()
            .expect("refused")
    };
    match refusal(serde_json::json!({ "step": 100 })) {
        PackError::ConfigurationRefusedByOwner { system, path, .. } => {
            assert_eq!(system, Probe::ID);
            assert!(path.ends_with("configure/presence.yaml"));
        }
        other => panic!("expected ConfigurationRefusedByOwner, got {other:?}"),
    }
    match refusal(serde_json::json!({ "step": 5, "stray": "foreign" })) {
        PackError::ConfigurationStatedAnotherPacksFact {
            system,
            event_type,
            owner,
            ..
        } => {
            assert_eq!(system, Probe::ID);
            assert_eq!(event_type.as_str(), "elsewhere-stated");
            assert_eq!(owner, SystemId::from_static("elsewhere"));
        }
        other => panic!("expected ConfigurationStatedAnotherPacksFact, got {other:?}"),
    }
    match refusal(serde_json::json!({ "step": 5, "stray": "undeclared" })) {
        PackError::ConfigurationStatedUndeclaredFact {
            system, event_type, ..
        } => {
            assert_eq!(system, Probe::ID);
            assert_eq!(event_type.as_str(), "probe-sectioned");
        }
        other => panic!("expected ConfigurationStatedUndeclaredFact, got {other:?}"),
    }
}

/// A save's genesis as a host reads it back: each emission as the envelope genesis recorded.
fn saved(facts: &[Emission]) -> Vec<EventEnvelope> {
    facts
        .iter()
        .zip(1..)
        .map(|(fact, id)| {
            EventEnvelope::new(
                EventId::from_raw(id),
                WorldTime::EPOCH,
                fact.record().clone(),
                Causation::WorldGenesis,
                fact.visibility().clone(),
                Provenance::new(fact.owner().clone()),
            )
        })
        .collect()
}

/// IA-4 (a): the comparator accepts an unchanged configuration and refuses a changed, a removed and an
/// added one, each naming the probe.
#[test]
fn the_comparator_refuses_changed_removed_and_added_configuration() {
    let genesis = |configuration| {
        seeded_pack(configuration)
            .assemble()
            .expect("assembles")
            .facts
    };
    let owners = BTreeMap::from([(
        <Configured as mineworld_contracts::Event>::EVENT_TYPE,
        Probe::ID,
    )]);
    let step = |step: u32| {
        vec![found(
            Capability::Presence,
            serde_json::json!({ "step": step }),
        )]
    };

    let save = saved(&genesis(step(5)));
    assert_eq!(compare(&save, &genesis(step(5)), &owners), Ok(()));

    let changed = compare(&save, &genesis(step(6)), &owners).expect_err("a changed step");
    assert_eq!(changed.system, Probe::ID);
    assert!(changed.saved.contains(r#"{"step":5}"#), "{changed:?}");
    assert!(changed.here.contains(r#"{"step":6}"#), "{changed:?}");

    let removed = compare(&save, &genesis(Vec::new()), &owners).expect_err("removed");
    assert_eq!(
        removed,
        Drift {
            system: Probe::ID,
            saved: removed.saved.clone(),
            here: "nothing".to_owned(),
        }
    );

    let added =
        compare(&saved(&genesis(Vec::new())), &genesis(step(5)), &owners).expect_err("added");
    assert_eq!(added.system, Probe::ID);
    assert_eq!(added.saved, "nothing");
}
