//! A world's configuration of its System Packs: `world.yaml`'s `configure:` and the files in
//! `configure/` (`docs/DECISIONS.md` `ARC-61`).
//!
//! ```text
//! read              each key resolved against the build and the world, each file decoded by its
//!                   owner's type, every file in configure/ listed, every system a configuration
//!                   requires enabled                                  (WorldPack::read, step 4c)
//! check_references  every entity a configuration names is declared, of the type its owner needs
//!                                                                     (after content, beside sections)
//! seed              each configuration's genesis facts, in configure: order, each in its owner's
//!                   vocabulary and of a type its owner declared      (load: after locations, before
//!                                                                     sections)
//! compare           a save's configuration facts against this pack's, in order: drift is refused at
//!                   resume                                            (every resuming host)
//! ```
//!
//! The loader never learns what a configuration means: it checks only what every configuration
//! shares, as `ARC-31` has it check only what every section shares.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use mineworld_authoring::{
    ATTACHMENT_MAX_BYTES, Attached, AuthoredConfiguration, ConfigurationContext,
    ConfigurationRefusal, EntityClasses, Seeding,
};
use mineworld_contracts::{
    EntityId, EntityKey, EventEnvelope, EventRecord, EventTypeId, SystemId, Visibility,
};
use mineworld_kernel::{Emission, WorldRead};
use mineworld_packages::LicencePolicy;

use crate::catalog::{AVAILABLE, Capability};
use crate::error::PackError;
use crate::format::{ConfigurationKey, FoundConfiguration};
use crate::read::{MANIFEST, WorldPack, parse_with};

/// The directory a World Pack's configuration files live in.
pub const DIRECTORY: &str = "configure";

/// The extension a configuration file has. Anything else in `configure/` is left alone.
const EXTENSION: &str = "yaml";

/// The keys of `configure:` that name no System Pack, each owned by a framework crate, with what it
/// is (`ARC-61` note). Never resolved against the installed set, never seeded on its own. No installed
/// pack may have one of these ids; a test holds it.
pub const FRAMEWORK: [(&str, &str); 2] = [
    (CLASSES, "the world's entity classes (ARC-64)"),
    (PACKAGES, "the world's licence policy (ARC-55)"),
];

/// The framework key of the world's entity classes.
const CLASSES: &str = "classes";

/// The framework key of the world's licence policy.
const PACKAGES: &str = "packages";

/// Where a configuration file lives.
fn file(root: &Path, key: &str) -> PathBuf {
    root.join(DIRECTORY).join(format!("{key}.{EXTENSION}"))
}

fn is_framework(key: &ConfigurationKey) -> bool {
    FRAMEWORK.iter().any(|(name, _)| *name == key.as_str())
}

/// A framework key's file, or the refusal that it is listed and absent.
fn framework_text(root: &Path, key: &'static str) -> Result<(PathBuf, String), PackError> {
    let path = file(root, key);
    if !path.exists() {
        return Err(PackError::ConfigurationFileMissing {
            system: SystemId::from_static(key),
            path,
        });
    }
    let text = std::fs::read_to_string(&path).map_err(|source| PackError::Unreadable {
        path: path.clone(),
        source,
    })?;
    Ok((path, text))
}

/// The world's licence policy (`ARC-55` note): `configure/packages.yaml` decoded straight into
/// `LicencePolicy` when `configure:` lists `packages`, otherwise the default. Read before the world's
/// requirements resolve; never seeded, never compared at resume.
pub(crate) fn licence_policy(
    root: &Path,
    keys: &[ConfigurationKey],
) -> Result<LicencePolicy, PackError> {
    if !keys.iter().any(|key| key.as_str() == PACKAGES) {
        return Ok(LicencePolicy::default());
    }
    let (path, text) = framework_text(root, PACKAGES)?;
    let invalid = |detail: String| PackError::LicencePolicyInvalid {
        path: path.clone(),
        detail,
    };
    let stated: StatedPolicy = serde_saphyr::from_str(&text).map_err(|e| invalid(e.to_string()))?;
    LicencePolicy::new(
        stated
            .allowed
            .iter()
            .map(|identifier| identifier.0.as_str()),
    )
    .map_err(|e| invalid(e.to_string()))
}

/// `configure/packages.yaml` as written: `LicencePolicy`'s own shape, decoded here identifier by
/// identifier so a refusal keeps the identifier's line and column (`LicencePolicy`'s own decoding checks
/// the list as a whole, after the decoder has left it).
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct StatedPolicy {
    allowed: Vec<Identifier>,
}

/// One SPDX licence identifier, checked by `LicencePolicy` itself as it decodes.
#[derive(serde::Deserialize)]
#[serde(try_from = "String")]
struct Identifier(String);

impl TryFrom<String> for Identifier {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        LicencePolicy::new([text.as_str()])
            .map(|_| Self(text))
            .map_err(|refusal| refusal.to_string())
    }
}

/// The world's entity classes (`ARC-64`): `configure/classes.yaml` when `configure:` lists `classes`,
/// otherwise none.
fn classes(root: &Path, keys: &[ConfigurationKey]) -> Result<EntityClasses, PackError> {
    if !keys.iter().any(|key| key.as_str() == CLASSES) {
        return Ok(EntityClasses::default());
    }
    let (path, text) = framework_text(root, CLASSES)?;
    serde_saphyr::from_str(&text).map_err(|error| PackError::ClassesInvalid {
        path,
        detail: error.to_string(),
    })
}

/// Reads `configure:`: each key listed once and resolved, the classes read, each System Pack's file
/// decoded by its owner's own type, every file in `configure/` listed, every required system enabled,
/// every attachment read, and each configuration checked against the classes — in that order, each
/// check assuming the one before it.
pub(crate) fn read(
    root: &Path,
    keys: &[ConfigurationKey],
    systems: &[Capability],
) -> Result<(EntityClasses, Vec<FoundConfiguration>), PackError> {
    let owners = resolve_keys(&root.join(MANIFEST), keys, systems)?;
    let classes = classes(root, keys)?;
    let mut found = read_files(root, &owners, |owner, text| {
        serde_saphyr::with_deserializer_from_str(text, |file| owner.decode_configuration(file))
    })?;
    check_nothing_undeclared(root, keys)?;
    check_requires(&found, systems)?;
    for configured in &mut found {
        configured.attached = read_attachments(root, configured)?;
    }
    for configured in &found {
        check(&classes, configured)?;
    }
    Ok((classes, found))
}

/// The System Pack each key configures, in `configure:` order: every key listed once, the framework
/// keys set aside, the rest resolved.
fn resolve_keys(
    manifest: &Path,
    keys: &[ConfigurationKey],
    systems: &[Capability],
) -> Result<Vec<Capability>, PackError> {
    let mut seen = BTreeSet::new();
    if let Some(twice) = keys.iter().find(|key| !seen.insert(*key)) {
        return Err(PackError::ConfigurationListedTwice {
            key: twice.to_string(),
            path: manifest.to_path_buf(),
        });
    }
    keys.iter()
        .filter(|key| !is_framework(key))
        .map(|key| resolve(manifest, key, systems))
        .collect()
}

/// The bytes of every `data/` file a configuration names: each present, inside the pack once its links
/// are followed, and within the size the loader reads (`ARC-61` note).
pub(crate) fn read_attachments(
    root: &Path,
    configured: &FoundConfiguration,
) -> Result<Attached, PackError> {
    let mut attached = Attached::none();
    let system = configured.configuration.owner();
    for attachment in configured.configuration.attachments() {
        let refused = |kind: fn(SystemId, String, PathBuf) -> PackError| {
            kind(
                system.clone(),
                attachment.to_string(),
                configured.path.clone(),
            )
        };
        let path = attachment.path_under(root);
        let unreadable = |source| PackError::Unreadable {
            path: path.clone(),
            source,
        };
        if !path.is_file() {
            return Err(refused(|system, attachment, path| {
                PackError::AttachmentMissing {
                    system,
                    attachment,
                    path,
                }
            }));
        }
        let real = path.canonicalize().map_err(unreadable)?;
        let pack = root.canonicalize().map_err(unreadable)?;
        if !real.starts_with(&pack) {
            return Err(refused(|system, attachment, path| {
                PackError::AttachmentOutside {
                    system,
                    attachment,
                    path,
                }
            }));
        }
        let bytes = std::fs::metadata(&real).map_err(unreadable)?.len();
        if bytes > ATTACHMENT_MAX_BYTES {
            return Err(PackError::AttachmentTooLarge {
                system,
                attachment: attachment.to_string(),
                bytes,
                max: ATTACHMENT_MAX_BYTES,
                path: configured.path.clone(),
            });
        }
        attached.insert(
            attachment.clone(),
            std::fs::read(&real).map_err(unreadable)?,
        );
    }
    Ok(attached)
}

/// A configuration's own refusal once the world's classes are known, named with its file.
fn check(classes: &EntityClasses, configured: &FoundConfiguration) -> Result<(), PackError> {
    let system = configured.configuration.owner();
    let path = configured.path.clone();
    configured
        .configuration
        .check(&ConfigurationContext::new(classes, &configured.attached))
        .map_err(|refusal| match refusal {
            ConfigurationRefusal::ClassUndefined { class, entry } => PackError::ClassUndefined {
                system,
                class: class.to_string(),
                entry: entry.to_string(),
                path,
            },
            ConfigurationRefusal::Ambiguous {
                first,
                second,
                field,
            } => PackError::AmbiguousEntries {
                system,
                first: first.to_string(),
                second: second.to_string(),
                field,
                path,
            },
        })
}

/// Each owner's file, decoded by `decode` — the owner's own type, through the YAML stream, so a
/// refusal keeps its line and column (`DEP-10`).
fn read_files(
    root: &Path,
    owners: &[Capability],
    decode: impl Fn(Capability, &str) -> Result<Arc<dyn AuthoredConfiguration>, serde_saphyr::Error>,
) -> Result<Vec<FoundConfiguration>, PackError> {
    let mut found = Vec::with_capacity(owners.len());
    for &owner in owners {
        let path = file(root, owner.id().as_str());
        if !path.exists() {
            return Err(PackError::ConfigurationFileMissing {
                system: owner.id(),
                path,
            });
        }
        let configuration = parse_with(&path, "configuration", |text| decode(owner, text))?;
        found.push(FoundConfiguration {
            owner,
            path,
            configuration,
            attached: Attached::none(),
        });
    }
    Ok(found)
}

/// Every system a configuration requires is enabled.
fn check_requires(found: &[FoundConfiguration], systems: &[Capability]) -> Result<(), PackError> {
    for configured in found {
        for requires in configured.configuration.requires() {
            if !systems.iter().any(|system| system.id() == requires) {
                return Err(PackError::ConfigurationRequiresSystem {
                    requires,
                    by: configured.owner.id(),
                    path: configured.path.clone(),
                });
            }
        }
    }
    Ok(())
}

/// The capability one key configures: a system of this build, enabled, and configurable.
fn resolve(
    manifest: &Path,
    key: &ConfigurationKey,
    systems: &[Capability],
) -> Result<Capability, PackError> {
    let Some(capability) = Capability::resolve(key.system()) else {
        let available: Vec<String> = AVAILABLE
            .into_iter()
            .map(|capability| format!("'{capability}'"))
            .collect();
        return Err(PackError::ConfigurationOfUnknownSystem {
            key: key.to_string(),
            available: available.join(", "),
            path: manifest.to_path_buf(),
        });
    };
    if !systems.contains(&capability) {
        return Err(PackError::ConfigurationOwnerNotEnabled {
            system: capability.id(),
            path: manifest.to_path_buf(),
        });
    }
    if capability.configuration().is_none() {
        return Err(PackError::NotConfigurable {
            system: capability.id(),
            path: manifest.to_path_buf(),
        });
    }
    Ok(capability)
}

/// Every `.yaml` file in `configure/` is listed. Collected and sorted, so the same file is named on
/// every machine (`read_dir` order is the filesystem's).
fn check_nothing_undeclared(root: &Path, keys: &[ConfigurationKey]) -> Result<(), PackError> {
    let directory = root.join(DIRECTORY);
    if !directory.is_dir() {
        return Ok(());
    }
    let unreadable = |source| PackError::Unreadable {
        path: directory.clone(),
        source,
    };
    let mut stems = BTreeSet::new();
    for entry in std::fs::read_dir(&directory).map_err(unreadable)? {
        let path = entry.map_err(unreadable)?.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some(EXTENSION) {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
            stems.insert(stem.to_owned());
        }
    }
    let listed: BTreeSet<&str> = keys.iter().map(ConfigurationKey::as_str).collect();
    match stems
        .into_iter()
        .find(|stem| !listed.contains(stem.as_str()))
    {
        Some(key) => Err(PackError::ConfigurationFileNotDeclared {
            path: file(root, &key),
            key,
        }),
        None => Ok(()),
    }
}

/// Every entity a configuration names is declared, of the type its owner needs. Run after content is
/// read and checked, beside the sections' own check.
pub(crate) fn check_references(pack: &WorldPack) -> Result<(), PackError> {
    let declared = pack.declared_entities();
    for configured in pack.configuration() {
        for reference in configured.configuration.references() {
            if declared.get(reference.key) != Some(&reference.entity_type) {
                return Err(PackError::ConfigurationNamesUnknownEntity {
                    system: configured.owner.id(),
                    key: reference.key.clone(),
                    expected: reference.entity_type,
                    path: configured.path.clone(),
                });
            }
        }
    }
    Ok(())
}

/// The genesis facts every configuration becomes, in `configure:` order (`ARC-61` item 6): each
/// refused if its owner refuses the value, if a fact is another pack's vocabulary, or if a fact's type
/// is not one its owner declared as a configuration fact.
pub(crate) fn seed(
    world: &WorldRead<'_>,
    ids: &BTreeMap<EntityKey, EntityId>,
    classes: &EntityClasses,
    configured: &[FoundConfiguration],
) -> Result<Vec<Emission>, PackError> {
    let mut facts = Vec::new();
    for found in configured {
        let configuration = &found.configuration;
        let system = configuration.owner();
        let emissions = configuration
            .seed(
                &Seeding::new(world, ids),
                &ConfigurationContext::new(classes, &found.attached),
            )
            .map_err(|reason| PackError::ConfigurationRefusedByOwner {
                system: system.clone(),
                reason: Box::new(reason),
                path: found.path.clone(),
            })?;
        for emission in &emissions {
            if *emission.owner() != system {
                return Err(PackError::ConfigurationStatedAnotherPacksFact {
                    system,
                    event_type: emission.event_type().clone(),
                    owner: emission.owner().clone(),
                    path: found.path.clone(),
                });
            }
            if !configuration.facts().contains(emission.event_type()) {
                return Err(PackError::ConfigurationStatedUndeclaredFact {
                    system,
                    event_type: emission.event_type().clone(),
                    path: found.path.clone(),
                });
            }
        }
        facts.extend(emissions);
    }
    Ok(facts)
}

/// The first difference between a save's configuration and a pack's (`ARC-61` item 7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drift {
    /// The system whose configuration fact differs, or is on one side only.
    pub system: SystemId,
    /// The save's fact at that position, or "nothing".
    pub saved: String,
    /// The pack's fact at that position, or "nothing".
    pub here: String,
}

/// One configuration fact, as compared: its event type, its record and its visibility.
fn describe(record: &EventRecord, visibility: &Visibility) -> String {
    format!(
        "{} {} {visibility:?}",
        record.event_type(),
        String::from_utf8_lossy(record.payload())
    )
}

/// Compares, in order, the configuration facts of a save's genesis with the ones a pack seeds: every
/// fact whose type `owners` names, by event type, record and visibility. Configuration added and
/// configuration removed are both drift. Pure.
///
/// # Errors
///
/// The first difference, naming the system the differing fact belongs to.
pub fn compare(
    saved: &[EventEnvelope],
    here: &[Emission],
    owners: &BTreeMap<EventTypeId, SystemId>,
) -> Result<(), Drift> {
    let mut saved = saved
        .iter()
        .filter(|fact| owners.contains_key(fact.event_type()))
        .map(|fact| (fact.payload(), fact.visibility()));
    let mut here = here
        .iter()
        .filter(|fact| owners.contains_key(fact.event_type()))
        .map(|fact| (fact.record(), fact.visibility()));
    loop {
        let (system, saved, here) = match (saved.next(), here.next()) {
            (None, None) => return Ok(()),
            (Some(a), Some(b)) if a == b => continue,
            (Some(a), b) => (owners[a.0.event_type()].clone(), Some(a), b),
            (None, Some(b)) => (owners[b.0.event_type()].clone(), None, Some(b)),
        };
        let shown = |fact: Option<(&EventRecord, &Visibility)>| {
            fact.map_or_else(
                || "nothing".to_owned(),
                |(record, visibility)| describe(record, visibility),
            )
        };
        return Err(Drift {
            system,
            saved: shown(saved),
            here: shown(here),
        });
    }
}

impl WorldPack {
    /// Refuses to resume or replay a save against this pack when its configuration differs from the
    /// save's (`ARC-61` item 7, QPL-12). Every resuming host calls this before it resumes or verifies.
    ///
    /// Assembles the world as genesis would — which seeds the configuration — and compares the
    /// configuration facts, in order, with the save's genesis facts (`saved_genesis`), over the event
    /// types every enabled pack declares as configuration facts.
    ///
    /// # Errors
    ///
    /// [`PackError::ConfigurationDrift`] naming the first differing system and both sides, or whatever
    /// assembling the world refuses.
    pub fn check_configuration(&self, saved_genesis: &[EventEnvelope]) -> Result<(), PackError> {
        // What every enabled pack declares, and what each configuration in this pack declares, under
        // its owner: in a build the two are one `PackConfiguration::FACTS` (`configures!()`), and the
        // second is also what seeding checks a fact against (D-9). The first alone sees a configuration
        // removed from the pack; the second lets a probe-labelled test drive this function.
        let declared = self.systems().iter().flat_map(|capability| {
            capability
                .configuration_facts()
                .iter()
                .map(move |fact| (fact.clone(), capability.id()))
        });
        let configured = self.configuration().iter().flat_map(|found| {
            let owner = found.configuration.owner();
            found
                .configuration
                .facts()
                .iter()
                .map(move |fact| (fact.clone(), owner.clone()))
        });
        let owners: BTreeMap<EventTypeId, SystemId> = declared.chain(configured).collect();
        let assembled = self.assemble()?;
        compare(saved_genesis, &assembled.facts, &owners).map_err(|drift| {
            PackError::ConfigurationDrift {
                system: drift.system,
                saved: drift.saved,
                here: drift.here,
            }
        })
    }
}

#[cfg(test)]
mod tests;
