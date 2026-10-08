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

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use mineworld_authoring::AuthoredConfiguration;

use crate::catalog::{AVAILABLE, Capability};
use crate::error::PackError;
use crate::format::{ConfigurationKey, FoundConfiguration};
use crate::read::{MANIFEST, WorldPack, parse_with};

/// The directory a World Pack's configuration files live in.
pub const DIRECTORY: &str = "configure";

/// The extension a configuration file has. Anything else in `configure/` is left alone.
const EXTENSION: &str = "yaml";

/// Keys reserved for what a later build configures, and what each is for (QIA-1). No installed pack
/// may have one of these ids; a test holds it.
pub const RESERVED: [(&str, &str); 2] = [
    ("classes", "the World's Interaction List's entity classes"),
    ("packages", "the licence policy's override"),
];

/// Where a configuration file lives.
fn file(root: &Path, key: &str) -> PathBuf {
    root.join(DIRECTORY).join(format!("{key}.{EXTENSION}"))
}

/// Reads `configure:`: each key resolved, each file decoded by its owner's own type, every file in
/// `configure/` listed, every required system enabled — in that order, each check assuming the one
/// before it.
pub(crate) fn read(
    root: &Path,
    keys: &[ConfigurationKey],
    systems: &[Capability],
) -> Result<Vec<FoundConfiguration>, PackError> {
    let owners = resolve_keys(&root.join(MANIFEST), keys, systems)?;
    let found = read_files(root, &owners, |owner, text| {
        serde_saphyr::with_deserializer_from_str(text, |file| owner.decode_configuration(file))
    })?;
    check_nothing_undeclared(root, keys)?;
    check_requires(&found, systems)?;
    Ok(found)
}

/// Each key's capability, in `configure:` order: listed once, then resolved.
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
        .map(|key| resolve(manifest, key, systems))
        .collect()
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

/// The capability one key configures: not reserved, a system of this build, enabled, and
/// configurable.
fn resolve(
    manifest: &Path,
    key: &ConfigurationKey,
    systems: &[Capability],
) -> Result<Capability, PackError> {
    if let Some((_, reserved_for)) = RESERVED.iter().find(|(name, _)| *name == key.as_str()) {
        return Err(PackError::ConfigurationReserved {
            key: key.to_string(),
            reserved_for,
            path: manifest.to_path_buf(),
        });
    }
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

#[cfg(test)]
mod tests;
