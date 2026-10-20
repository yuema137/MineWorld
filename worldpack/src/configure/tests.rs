//! In-crate tests of `configure` with a probe configuration (`DECISIONS.md` `ARC-61`).
//!
//! No pack this build installs is configurable, so the checks after "is this key a configurable
//! system" are exercised here with a probe `PackConfiguration`, labelled with a real capability as the
//! sections' in-crate tests label theirs (step-10 F-22).

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use mineworld_authoring::{
    AuthoredConfiguration, AuthoredSection, ConfigurationContext, ContentKind, Decode,
    DecodeConfiguration, PackConfiguration, Reference, SectionName, Seeding,
};
use mineworld_contracts::{
    Causation, EntityId, EntityKey, EntityType, EventEnvelope, EventId, EventSchemaVersion,
    EventTypeId, Provenance, Rejection, SystemId, Visibility, WorldTime,
};
use mineworld_kernel::{Emission, SystemIdentity};
use serde::Deserialize;
use serde::de::DeserializeSeed;

use super::{Drift, check_references, check_requires, compare, read_attachments, read_files};
use crate::catalog::Capability;
use crate::error::PackError;
use crate::format::{
    AuthoredLocation, AuthoredPerson, AuthoredPlace, FoundConfiguration, FoundSection, SectionState,
};
use crate::read::WorldPack;

/// The probe's configuration: a step, bounded by its own type, and optionally a place it names and
/// a system it needs.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct Settings {
    step: Step,
    #[serde(default)]
    #[schemars(with = "Option<mineworld_authoring::schema::Key>")]
    at: Option<EntityKey>,
    #[serde(default)]
    #[schemars(with = "Option<mineworld_authoring::schema::SystemName>")]
    needs: Option<SystemId>,
    /// What the probe states besides its one configuration fact, to be refused.
    #[serde(default)]
    stray: Option<Stray>,
    /// A `data/` file whose text the probe states in its fact (SD-IB-5).
    #[serde(default)]
    table: Option<mineworld_authoring::Attachment>,
}

/// A fact a configuration must not seed.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    table: Option<String>,
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
#[derive(Debug, Deserialize, schemars::JsonSchema)]
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

    fn attachments(settings: &Settings) -> Vec<&mineworld_authoring::Attachment> {
        settings.table.iter().collect()
    }

    /// States `probe-configured { step }`, `SystemInternal`; refuses a step of 100 — a refusal only the
    /// owner can state, at genesis.
    fn seed(
        _: &Seeding<'_, '_>,
        settings: &Settings,
        context: &ConfigurationContext<'_>,
    ) -> Result<Vec<Emission>, Rejection> {
        if settings.step.0 == 100 {
            return Err(Rejection::System {
                code: mineworld_contracts::RejectionCode::from_static("probe-step-max"),
                detail: None,
            });
        }
        let table = settings.table.as_ref().map(|table| {
            String::from_utf8_lossy(context.attached().get(table).unwrap_or_default()).into_owned()
        });
        let mut facts = vec![emission(&Configured {
            step: settings.step.0,
            table,
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

    /// `{ needs: <system> }` needs that system (SD-Nb-5 item 1).
    fn requires(authored: &serde_json::Value) -> Vec<SystemId> {
        authored
            .get("needs")
            .and_then(serde_json::Value::as_str)
            .and_then(|name| SystemId::new(name).ok())
            .into_iter()
            .collect()
    }

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
        attached: mineworld_authoring::Attached::none(),
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

/// The framework types a configuration names decode through the loader's YAML stream with line and
/// column, whatever the file's line endings: an attachment that leaves `data/`, and a classes list whose
/// class is defined twice or whose `of` is no entity type (SD-IB-4, SD-IB-5; the operator's platform
/// requirement: CRLF and Windows-style paths).
#[test]
fn attachments_and_classes_are_refused_at_their_line_and_column_with_either_line_ending() {
    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct Named {
        note: String,
        table: mineworld_authoring::Attachment,
    }
    for ending in ["\n", "\r\n"] {
        let text = |table: &str| format!("note: rows{ending}table: {table}{ending}");
        for (table, why) in [
            ("data/../world.yaml", "'..'"),
            ("/etc/passwd", "is absolute"),
            ("tables/x.csv", "is not under data/"),
            ("'data\\x.csv'", "uses '\\'"),
            ("'C:\\data\\x.csv'", "uses '\\'"),
        ] {
            let refusal =
                serde_saphyr::from_str::<Named>(&text(table)).expect_err("refused at decode");
            let detail = refusal.to_string();
            assert!(
                detail.contains("line 2") && detail.contains(why),
                "{table:?} with {ending:?}: {detail}"
            );
        }
        let named = serde_saphyr::from_str::<Named>(&text("data/rows.csv")).expect("a plain path");
        assert_eq!(named.table.to_string(), "data/rows.csv");

        let classes = |text: &str| {
            serde_saphyr::from_str::<mineworld_authoring::EntityClasses>(
                &text.replace('\n', ending),
            )
        };
        let twice = classes(
            "- { class: noble, of: person, tag: noble }\n- { class: noble, of: item, tag: gold }\n",
        )
        .expect_err("defined twice")
        .to_string();
        assert!(
            twice.contains("'noble' is defined twice") && twice.contains("line 2"),
            "{twice}"
        );
        let of = classes(
            "- { class: noble, of: person, tag: noble }\n- { class: heir, of: castle, tag: x }\n",
        )
        .expect_err("no such type")
        .to_string();
        assert!(of.contains("line 2") && of.contains("castle"), "{of}");
        let fine = classes("- { class: noble, of: person, tag: noble }\n").expect("valid");
        assert_eq!(fine.definitions().len(), 1);
    }
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

/// An unconfigured pack whose defaults need a system the world does not enable is refused, naming
/// both and `world.yaml`; enabling the system accepts it (SD-Nb-5 item 2).
#[test]
fn an_unconfigured_pack_whose_defaults_need_a_disabled_system_is_refused() {
    let manifest = PathBuf::from("in-memory/world.yaml");
    let defaults = || {
        [(
            SystemId::from_static("probe"),
            vec![SystemId::from_static("movement")],
        )]
    };
    let refusal = super::check_default_requires(&manifest, defaults(), &[Capability::Presence])
        .expect_err("movement is not enabled");
    match &refusal {
        PackError::DefaultConfigurationRequiresSystem { requires, by, path } => {
            assert_eq!(*requires, SystemId::from_static("movement"));
            assert_eq!(*by, SystemId::from_static("probe"));
            assert_eq!(path, &manifest);
        }
        other => panic!("expected DefaultConfigurationRequiresSystem, got {other:?}"),
    }
    let text = refusal.to_string();
    assert!(
        text.contains("world.yaml enables 'probe'")
            && text.contains("needs 'movement'")
            && text.contains("configure/probe.yaml"),
        "{text}"
    );
    super::check_default_requires(
        &manifest,
        defaults(),
        &[Capability::Presence, Capability::Movement],
    )
    .expect("movement is enabled");
    super::check_default_requires(&manifest, [], &[Capability::Presence])
        .expect("no unconfigured pack needs anything");
}

/// A section that needs a system the world does not enable is refused by name; enabled, it is
/// accepted; a section needing nothing is untouched (SD-Nb-5 item 1).
#[test]
fn a_section_that_needs_a_disabled_system_is_refused_by_name() {
    let pack = |section: serde_json::Value, systems: Vec<Capability>| {
        let decoded = Decode::<Probe>::new()
            .deserialize(section)
            .expect("the probe section decodes");
        let person = AuthoredPerson {
            sections: vec![FoundSection {
                owner: Capability::Naming,
                name: <Probe as AuthoredSection>::SECTION,
                state: SectionState::Decoded(decoded),
            }],
            ..AuthoredPerson::default()
        };
        WorldPack::in_memory(
            systems,
            BTreeMap::new(),
            BTreeMap::from([(key("ada"), person)]),
            BTreeMap::new(),
            BTreeMap::new(),
        )
    };
    let needs = serde_json::json!({ "needs": "movement" });
    let refusal = crate::read::check_sections(&pack(needs.clone(), vec![Capability::Naming]))
        .expect_err("movement is not enabled");
    match &refusal {
        PackError::SectionRequiresSystem {
            subject,
            section,
            requires,
            ..
        } => {
            assert_eq!(subject.as_str(), "ada");
            assert_eq!(section.as_str(), "probe");
            assert_eq!(*requires, SystemId::from_static("movement"));
        }
        other => panic!("expected SectionRequiresSystem, got {other:?}"),
    }
    assert!(
        refusal
            .to_string()
            .contains("'ada' a 'probe' section that needs 'movement'"),
        "{refusal}"
    );
    crate::read::check_sections(&pack(needs, vec![Capability::Naming, Capability::Movement]))
        .expect("movement is enabled");
    crate::read::check_sections(&pack(serde_json::json!({}), vec![Capability::Naming]))
        .expect("needs nothing");
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
    let configured_fact = |step| emission(&Configured { step, table: None });
    assert_eq!(configured.facts[1], configured_fact(9));
    assert_eq!(configured.facts[2], configured_fact(4));
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

/// IA-4 (a) through the hosts' own entry point (R-IA-1, M-IA4d): `WorldPack::check_configuration`
/// builds its filter, assembles the pack and maps the drift. An unchanged configuration is accepted; a
/// changed and an added one are each `ConfigurationDrift` naming the probe. (A configuration removed
/// from the pack is seen through the capability's declared facts, which a probe label has none of: it
/// is held by the comparator test above and by the canary, E-IA-8.)
#[test]
fn check_configuration_refuses_a_changed_and_an_added_configuration_naming_the_owner() {
    let step = |step: u32| {
        vec![found(
            Capability::Presence,
            serde_json::json!({ "step": step }),
        )]
    };
    let save = |configuration| {
        saved(
            &seeded_pack(configuration)
                .assemble()
                .expect("assembles")
                .facts,
        )
    };
    let drift = |refusal: Result<(), PackError>| match refusal {
        Err(PackError::ConfigurationDrift {
            system,
            saved,
            here,
        }) => (system, saved, here),
        other => panic!("expected ConfigurationDrift, got {other:?}"),
    };

    let configured = save(step(5));
    assert!(
        seeded_pack(step(5))
            .check_configuration(&configured)
            .is_ok(),
        "the save's own configuration is accepted"
    );

    let (system, saved_side, here) = drift(seeded_pack(step(6)).check_configuration(&configured));
    assert_eq!(system, Probe::ID);
    assert!(saved_side.contains(r#"{"step":5}"#), "{saved_side}");
    assert!(here.contains(r#"{"step":6}"#), "{here}");

    let (system, saved_side, _) =
        drift(seeded_pack(step(5)).check_configuration(&save(Vec::new())));
    assert_eq!((system, saved_side.as_str()), (Probe::ID, "nothing"));
}

// ---- A data owner's world-level facts (`ARC-121` item 5, S30-c) -----------------------------

/// A data owner whose context is one number, which it states as its one context fact — or, for the
/// numbers below, states something it must not, or refuses.
struct Composer;

impl SystemIdentity for Composer {
    const ID: SystemId = SystemId::from_static("composer");
}

/// The composer's context fact.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Composed {
    n: u32,
}

impl mineworld_contracts::Event for Composed {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("composer-composed");
    const OWNER: SystemId = Composer::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// A fact of the composer's own vocabulary that is not one of its context facts.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Stray2 {}

impl mineworld_contracts::Event for Stray2 {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("composer-stray");
    const OWNER: SystemId = Composer::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// States `n` as its fact, except: 100 states another pack's fact, 101 an undeclared one of its own,
/// 102 refuses.
const FOREIGN: u32 = 100;
const UNDECLARED: u32 = 101;
const REFUSES: u32 = 102;

impl mineworld_authoring::OwnsData for Composer {
    const DIRECTORIES: &'static [mineworld_authoring::DataDirectory] =
        &[mineworld_authoring::DataDirectory::from_static("composed")];
    type File = u32;
    type Context = u32;

    fn decode_file<'de, D: serde::Deserializer<'de>>(
        _: mineworld_authoring::DataDirectory,
        deserializer: D,
    ) -> Result<u32, D::Error> {
        u32::deserialize(deserializer)
    }

    fn context(
        files: Vec<(mineworld_authoring::DataOrigin, u32)>,
        _: Option<&dyn std::any::Any>,
    ) -> Result<u32, mineworld_authoring::DataRefusal> {
        Ok(files.into_iter().map(|(_, n)| n).sum())
    }

    const CONTEXT_FACTS: &'static [EventTypeId] =
        &[<Composed as mineworld_contracts::Event>::EVENT_TYPE];

    fn seed_context(_: &Seeding<'_, '_>, n: &u32) -> Result<Vec<Emission>, Rejection> {
        match *n {
            FOREIGN => Ok(vec![emission(&Elsewhere {})]),
            UNDECLARED => Ok(vec![emission(&Stray2 {})]),
            REFUSES => Err(Rejection::PreconditionFailed),
            n => Ok(vec![emission(&Composed { n })]),
        }
    }
}

/// The composer's context, built from one file holding `n`, as the loader holds it.
fn composer(n: u32) -> Arc<dyn mineworld_authoring::OwnerContext> {
    let directory = mineworld_authoring::DataDirectory::from_static("composed");
    let file = mineworld_authoring::decode_data_file::<Composer, _>(
        directory,
        serde::de::IntoDeserializer::<serde::de::value::Error>::into_deserializer(n),
    )
    .expect("a number decodes");
    let origin = mineworld_authoring::DataOrigin {
        pack: "a-world".to_owned(),
        source: mineworld_authoring::DataSource::World,
        directory,
        path: PathBuf::from("composed/n.yaml"),
    };
    mineworld_authoring::data_context::<Composer>(vec![(origin, file)], None).expect("a context")
}

/// `seeded_pack` configured with one probe configuration, the composer's context held for `naming`.
fn composed_pack(n: u32) -> WorldPack {
    seeded_pack(vec![found(
        Capability::Presence,
        serde_json::json!({ "step": 5 }),
    )])
    .with_contexts(BTreeMap::from([(Capability::Naming, composer(n))]))
}

/// Item 5's position (M-c-1's test): a data owner's context facts follow every configuration fact and
/// precede every section fact; a pack holding no context seeds exactly as before.
#[test]
fn context_facts_follow_configuration_and_precede_sections() {
    let facts = composed_pack(3).assemble().expect("assembles").facts;
    assert_eq!(
        event_types(&facts),
        [
            "arrived",
            "probe-configured",
            "composer-composed",
            "probe-sectioned"
        ]
    );
    assert_eq!(facts[2], emission(&Composed { n: 3 }));
    let without = seeded_pack(vec![found(
        Capability::Presence,
        serde_json::json!({ "step": 5 }),
    )])
    .assemble()
    .expect("assembles")
    .facts;
    assert_eq!(
        event_types(&without),
        ["arrived", "probe-configured", "probe-sectioned"]
    );
}

/// A context fact of another vocabulary, one the owner did not declare, and the owner's refusal are
/// each refused at genesis, naming the owner.
#[test]
fn a_context_that_states_another_or_an_undeclared_fact_or_refuses_is_refused_by_name() {
    match composed_pack(FOREIGN).assemble().map(|_| ()) {
        Err(PackError::ContextStatedAnotherPacksFact {
            system,
            event_type,
            owner,
        }) => {
            assert_eq!(system, Composer::ID);
            assert_eq!(event_type.as_str(), "elsewhere-stated");
            assert_eq!(owner.as_str(), "elsewhere");
        }
        other => panic!("expected ContextStatedAnotherPacksFact, got {other:?}"),
    }
    match composed_pack(UNDECLARED).assemble().map(|_| ()) {
        Err(PackError::ContextStatedUndeclaredFact { system, event_type }) => {
            assert_eq!(system, Composer::ID);
            assert_eq!(event_type.as_str(), "composer-stray");
        }
        other => panic!("expected ContextStatedUndeclaredFact, got {other:?}"),
    }
    match composed_pack(REFUSES).assemble().map(|_| ()) {
        Err(PackError::ContextRefusedByOwner { system, .. }) => assert_eq!(system, Composer::ID),
        other => panic!("expected ContextRefusedByOwner, got {other:?}"),
    }
}

/// R-S30c-b-2 (M-c-2's test): a resumed world whose data states a different context fact from its
/// save's is `ConfigurationDrift` naming the data owner; the same data is accepted.
#[test]
fn check_configuration_refuses_a_changed_context_fact_naming_the_owner() {
    let save = saved(&composed_pack(3).assemble().expect("assembles").facts);
    assert!(
        composed_pack(3).check_configuration(&save).is_ok(),
        "the save's own data is accepted"
    );
    match composed_pack(4).check_configuration(&save) {
        Err(PackError::ConfigurationDrift {
            system,
            saved,
            here,
        }) => {
            assert_eq!(system, Composer::ID);
            assert!(saved.contains(r#"{"n":3}"#), "{saved}");
            assert!(here.contains(r#"{"n":4}"#), "{here}");
        }
        other => panic!("expected ConfigurationDrift, got {other:?}"),
    }
}

// ---- data: attachments (SD-IB-5; IB-8's attachment half, IB-10's refusals) ---------------------

/// A probe configuration naming `data/<table>`, labelled as presence, at `root`.
fn tabled(root: &std::path::Path, table: &str) -> FoundConfiguration {
    FoundConfiguration {
        path: root.join("configure/presence.yaml"),
        ..found(
            Capability::Presence,
            serde_json::json!({ "step": 5, "table": format!("data/{table}") }),
        )
    }
}

/// A named file that is missing, one that is over 4 MiB and one whose link leads outside the pack are
/// each refused by name; a present one is read, its bytes unchanged — CRLF included — and a file
/// nothing names is left alone.
#[test]
fn attachments_are_read_whole_and_refused_when_missing_outside_or_over_size() {
    let scratch = ScratchRoot::new("attachments");
    let data = scratch.0.join("data");
    std::fs::create_dir_all(&data).expect("writable");
    std::fs::write(data.join("unnamed.txt"), "left alone").expect("writable");

    match read_attachments(&scratch.0, &tabled(&scratch.0, "rows.csv")) {
        Err(PackError::AttachmentMissing {
            system, attachment, ..
        }) => {
            assert_eq!(system, Probe::ID);
            assert_eq!(attachment, "data/rows.csv");
        }
        other => panic!("expected AttachmentMissing, got {other:?}"),
    }

    std::fs::write(data.join("rows.csv"), "1,2,3\r\n4,5,6\r\n").expect("writable");
    let attached = read_attachments(&scratch.0, &tabled(&scratch.0, "rows.csv")).expect("read");
    let table = mineworld_authoring::Attachment::try_from("data/rows.csv".to_owned()).expect("ok");
    assert_eq!(attached.get(&table), Some(&b"1,2,3\r\n4,5,6\r\n"[..]));

    let big = std::fs::File::create(data.join("big.bin")).expect("writable");
    big.set_len(mineworld_authoring::ATTACHMENT_MAX_BYTES + 1)
        .expect("a sparse file");
    match read_attachments(&scratch.0, &tabled(&scratch.0, "big.bin")) {
        Err(PackError::AttachmentTooLarge { bytes, max, .. }) => {
            assert_eq!((bytes, max), (4 * 1024 * 1024 + 1, 4 * 1024 * 1024));
        }
        other => panic!("expected AttachmentTooLarge, got {other:?}"),
    }

    // A link out of the pack, on every platform (step-14 §15.13, D-13w-3); the check itself is
    // platform-neutral (canonical paths).
    let outside = ScratchRoot::new("attachments-outside");
    std::fs::write(outside.0.join("secret.txt"), "not the pack's").expect("writable");
    let linked = link_out_of(&data, &outside.0);
    match read_attachments(&scratch.0, &tabled(&scratch.0, linked)) {
        Err(PackError::AttachmentOutside { attachment, .. }) => {
            assert_eq!(attachment, format!("data/{linked}"));
        }
        other => panic!("expected AttachmentOutside, got {other:?}"),
    }
}

/// Makes a link under `data` through which `outside/secret.txt` is reached, and returns the path under
/// `data/` that names the secret through it. On Unix, a file symlink.
#[cfg(unix)]
fn link_out_of(data: &std::path::Path, outside: &std::path::Path) -> &'static str {
    std::os::unix::fs::symlink(outside.join("secret.txt"), data.join("link.txt"))
        .expect("a symlink");
    "link.txt"
}

/// On Windows an unprivileged process may make no symlink (that needs Developer Mode or
/// administrator rights), but anyone may make a directory junction (`mklink /J`), which leads out of
/// the pack just as well.
#[cfg(windows)]
fn link_out_of(data: &std::path::Path, outside: &std::path::Path) -> &'static str {
    let made = std::process::Command::new("cmd")
        .arg("/C")
        .arg("mklink")
        .arg("/J")
        .arg(data.join("linked"))
        .arg(outside)
        .output()
        .expect("cmd runs");
    assert!(made.status.success(), "a directory junction: {made:?}");
    "linked/secret.txt"
}

/// The bytes reach the owner's seed and its fact, so a changed file is drift at resume (IB-8's
/// attachment half, through `check_configuration`); the same bytes are not.
#[test]
fn a_changed_attachment_is_drift_and_an_unchanged_one_is_not() {
    let scratch = ScratchRoot::new("attachment-drift");
    std::fs::create_dir_all(scratch.0.join("data")).expect("writable");
    let pack = |rows: &str| {
        std::fs::write(scratch.0.join("data/rows.csv"), rows).expect("writable");
        let mut configured = tabled(&scratch.0, "rows.csv");
        configured.attached = read_attachments(&scratch.0, &configured).expect("read");
        seeded_pack(vec![configured])
    };
    let facts = pack("1,2,3\n").assemble().expect("assembles").facts;
    assert_eq!(
        facts[1],
        emission(&Configured {
            step: 5,
            table: Some("1,2,3\n".to_owned())
        }),
        "the owner states what it read"
    );
    let save = saved(&facts);
    pack("1,2,3\n")
        .check_configuration(&save)
        .expect("the same bytes");
    match pack("1,2,4\n").check_configuration(&save) {
        Err(PackError::ConfigurationDrift { system, .. }) => assert_eq!(system, Probe::ID),
        other => panic!("expected ConfigurationDrift, got {other:?}"),
    }
}
