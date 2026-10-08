//! In-crate tests of `configure` with a probe configuration (`DECISIONS.md` `ARC-61`).
//!
//! No pack this build installs is configurable, so the checks after "is this key a configurable
//! system" are exercised here with a probe `PackConfiguration`, labelled with a real capability as the
//! sections' in-crate tests label theirs (step-10 F-22).

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use mineworld_authoring::{
    AuthoredConfiguration, DecodeConfiguration, PackConfiguration, Reference, Seeding,
};
use mineworld_contracts::{EntityKey, EntityType, EventTypeId, Rejection, SystemId};
use mineworld_kernel::{Emission, SystemIdentity};
use serde::Deserialize;
use serde::de::DeserializeSeed;

use super::{check_references, check_requires, read_files};
use crate::catalog::Capability;
use crate::error::PackError;
use crate::format::{AuthoredPerson, AuthoredPlace, FoundConfiguration};
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
    const FACTS: &'static [EventTypeId] = &[];

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

    /// Refuses a step of 100 at genesis: a refusal only the assembled world's owner gives.
    fn seed(_: &Seeding<'_, '_>, settings: &Settings) -> Result<Vec<Emission>, Rejection> {
        if settings.step.0 == 100 {
            return Err(Rejection::System {
                code: mineworld_contracts::RejectionCode::from_static("probe-step-max"),
                detail: None,
            });
        }
        Ok(Vec::new())
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
