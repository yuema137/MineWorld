//! The data directories System Packs own (`DECISIONS.md` `ARC-121` item 1; `MODULE_SPEC.md` §§2, 3.1): read
//! step 4d.
//!
//! ```text
//! the rule      every directory of the World Pack's root and of each required Entity Pack's is a format
//!               directory, the first component of `decisions:`, or a directory an installed pack owns —
//!               enabled by the world, or refused naming the owner (DataNeedsASystem); a name beginning
//!               with `.` is not content and is ignored; anything else is UnknownDirectory, listing every
//!               directory this build knows
//! collect       for each enabled owner, in `systems:` order: files the enabled packs ship, in `systems:`
//!               order; then each required Entity Pack, in id order; then the world — within each source,
//!               directories in the owner's order and files by name, byte order; a subdirectory inside a
//!               data directory is UnknownDirectory
//! decode        each file by its owner's own type, straight from the YAML stream, so a refusal keeps its
//!               line and column (DEP-10)
//! context       the owner builds one from every file and its own configuration; an owner with no files
//!               still gets one, built from none
//! ```
//!
//! The loader never learns what a file means: it lists, reads and hands over, then holds the context
//! erased. No file is read for an owner the world does not enable.
//!
//! Everything below is written against [`DataOwnerOps`], implemented for the build's [`Capability`] and,
//! in this crate's tests, for probe owners — so order, refusals and contexts are tested without
//! installing a pack (step-10 QS-24's precedent).

use std::any::Any;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use mineworld_authoring::{
    AuthoredConfiguration, DataDirectory, DataOrigin, DataRefusal, DataSource, OwnerContext,
    ShippedData,
};
use mineworld_contracts::SystemId;
use serde::de::Deserializer;

use crate::catalog::{AVAILABLE, Capability};
use crate::configure;
use crate::content::Contexts;
use crate::entities;
use crate::error::{ContentKind, PackError};
use crate::format::{DecisionsFile, FoundConfiguration, WorldManifest};
use crate::read::parse_with;
use mineworld_packages::Composition;

/// The extension a data file has; any other file in a data directory is not content.
const DATA_EXTENSION: &str = "yaml";

/// The kind a malformed data file is reported as.
const DATA_KIND: &str = "data";

/// What the loader asks of a data owner: the seam of [`mineworld_sdk::SystemPack`]'s data items, with
/// one implementation per owner type.
pub(crate) trait DataOwnerOps: Copy + Ord {
    /// The owner's id.
    fn id(self) -> SystemId;

    /// The directories it owns, in its order; empty for a pack that owns none.
    fn directories(self) -> &'static [DataDirectory];

    /// Decodes one of its files.
    fn decode_file<'de, D: Deserializer<'de>>(
        self,
        directory: DataDirectory,
        deserializer: D,
    ) -> Result<Box<dyn Any + Send + Sync>, D::Error>;

    /// Builds its context.
    fn context(
        self,
        files: Vec<(DataOrigin, Box<dyn Any + Send + Sync>)>,
        configuration: Option<&dyn AuthoredConfiguration>,
    ) -> Result<Arc<dyn OwnerContext>, DataRefusal>;

    /// The files its pack ships.
    fn ships(self) -> &'static [ShippedData];
}

impl DataOwnerOps for Capability {
    fn id(self) -> SystemId {
        Self::id(self)
    }

    fn directories(self) -> &'static [DataDirectory] {
        self.data().map_or(&[], |data| data.directories)
    }

    fn decode_file<'de, D: Deserializer<'de>>(
        self,
        directory: DataDirectory,
        deserializer: D,
    ) -> Result<Box<dyn Any + Send + Sync>, D::Error> {
        self.decode_data_file(directory, deserializer)
    }

    fn context(
        self,
        files: Vec<(DataOrigin, Box<dyn Any + Send + Sync>)>,
        configuration: Option<&dyn AuthoredConfiguration>,
    ) -> Result<Arc<dyn OwnerContext>, DataRefusal> {
        self.data_context(files, configuration)
    }

    fn ships(self) -> &'static [ShippedData] {
        self.shipped()
    }
}

/// One pack whose directories are read: the world, or a required Entity Pack.
#[derive(Debug, Clone)]
pub(crate) struct DataRoot<'a> {
    /// The pack's id, which a data file's origin names.
    pub(crate) pack: &'a str,
    /// What kind of pack it is.
    pub(crate) source: DataSource,
    /// Its directory.
    pub(crate) dir: &'a Path,
    /// The directories its format gives a meaning: a World Pack's six, an Entity Pack's `items`.
    pub(crate) format: &'a [&'a str],
    /// Any other directory it may hold: a World Pack's `decisions:` directory.
    pub(crate) admitted: Vec<String>,
}

/// The installed owner of the directory written `name`, and that directory.
fn owner_of<O: DataOwnerOps>(installed: &[O], name: &str) -> Option<(O, DataDirectory)> {
    installed.iter().find_map(|owner| {
        owner
            .directories()
            .iter()
            .find(|directory| directory.as_str() == name)
            .map(|directory| (*owner, *directory))
    })
}

/// Every directory an installed owner owns.
fn owned<O: DataOwnerOps>(installed: &[O]) -> impl Iterator<Item = &'static str> + '_ {
    installed.iter().flat_map(|owner| {
        owner
            .directories()
            .iter()
            .map(|directory| directory.as_str())
    })
}

/// Names, sorted and once each, for a refusal to list; "none" for none.
fn listed<'n>(names: impl IntoIterator<Item = &'n str>) -> String {
    let mut names: Vec<&str> = names.into_iter().collect();
    names.sort_unstable();
    names.dedup();
    if names.is_empty() {
        "none".to_owned()
    } else {
        names.join(", ")
    }
}

/// Every directory name `root` may hold in this build, for a refusal to list.
fn known<O: DataOwnerOps>(root: &DataRoot<'_>, installed: &[O]) -> String {
    let mut names: Vec<&str> = root.format.to_vec();
    names.extend(root.admitted.iter().map(String::as_str));
    for name in owned(installed) {
        names.push(name);
    }
    listed(names)
}

/// The entries of `dir`, sorted by name (byte order), so a refusal is the same on every machine.
fn entries(dir: &Path) -> Result<Vec<(OsString, PathBuf)>, PackError> {
    let unreadable = |source| PackError::Unreadable {
        path: dir.to_path_buf(),
        source,
    };
    let mut listed = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(unreadable)? {
        let entry = entry.map_err(unreadable)?;
        listed.push((entry.file_name(), entry.path()));
    }
    Ok(ordered(listed))
}

/// A listing in name order. Its own function so the order is tested against a listing in any order
/// (`read_dir`'s is the file system's).
fn ordered(mut listed: Vec<(OsString, PathBuf)>) -> Vec<(OsString, PathBuf)> {
    listed.sort();
    listed
}

/// Whether a name is hidden from content: it begins with `.`.
fn is_dot(name: &OsString) -> bool {
    name.as_encoded_bytes().first() == Some(&b'.')
}

/// The directory rule for one pack root (F-S30b-3).
pub(crate) fn check_directories<O: DataOwnerOps>(
    root: &DataRoot<'_>,
    installed: &[O],
    enabled: &[O],
) -> Result<(), PackError> {
    for (name, path) in entries(root.dir)? {
        if !path.is_dir() || is_dot(&name) {
            continue;
        }
        let Some(name) = name.to_str() else {
            return Err(PackError::UnknownDirectory {
                path,
                known: known(root, installed),
            });
        };
        if root.format.contains(&name) || root.admitted.iter().any(|admitted| admitted == name) {
            continue;
        }
        match owner_of(installed, name) {
            Some((owner, _)) if enabled.contains(&owner) => {}
            Some((owner, _)) => {
                return Err(PackError::DataNeedsASystem {
                    directory: name.to_owned(),
                    system: owner.id(),
                    path,
                });
            }
            None => {
                return Err(PackError::UnknownDirectory {
                    path,
                    known: known(root, installed),
                });
            }
        }
    }
    Ok(())
}

/// One data file found, with its text, before it is decoded.
struct Found {
    origin: DataOrigin,
    /// The path a refusal names: the file on disk, or a shipped file's path and pack.
    shown: PathBuf,
    text: Option<&'static str>,
}

/// The shipped file `shipped` of the pack `pack`, as a refusal names it.
fn shown_shipped(pack: &SystemId, shipped: &ShippedData) -> PathBuf {
    PathBuf::from(format!(
        "{} (shipped by the '{}' system)",
        shipped.path(),
        pack.as_str()
    ))
}

/// Every shipped file of every enabled pack lies in a directory some installed pack owns: one that does
/// not is a defect of the build, refused rather than dropped.
fn check_shipped<O: DataOwnerOps>(installed: &[O], enabled: &[O]) -> Result<(), PackError> {
    for pack in enabled {
        for shipped in pack.ships() {
            if owner_of(installed, shipped.directory().as_str()).is_none() {
                return Err(PackError::UnknownDirectory {
                    path: shown_shipped(&pack.id(), shipped),
                    known: listed(owned(installed)),
                });
            }
        }
    }
    Ok(())
}

/// `owner`'s files, in collection order (I-b-7).
fn collect<O: DataOwnerOps>(
    owner: O,
    enabled: &[O],
    roots: &[DataRoot<'_>],
) -> Result<Vec<Found>, PackError> {
    let directories = owner.directories();
    let rank = |directory: DataDirectory| directories.iter().position(|own| *own == directory);
    let mut found = Vec::new();
    for pack in enabled {
        let mut shipped: Vec<(usize, &'static str, &ShippedData)> = pack
            .ships()
            .iter()
            .filter_map(|shipped| rank(shipped.directory()).map(|at| (at, shipped.file(), shipped)))
            .collect();
        shipped.sort_by(|a, b| (a.0, a.1.as_bytes()).cmp(&(b.0, b.1.as_bytes())));
        for (_, _, shipped) in shipped {
            found.push(Found {
                origin: DataOrigin {
                    pack: pack.id().as_str().to_owned(),
                    source: DataSource::SystemPack,
                    directory: shipped.directory(),
                    path: PathBuf::from(shipped.path()),
                },
                shown: shown_shipped(&pack.id(), shipped),
                text: Some(shipped.text()),
            });
        }
    }
    for root in roots {
        for directory in directories {
            let dir = root.dir.join(directory.as_str());
            if !dir.is_dir() {
                continue;
            }
            for (name, path) in entries(&dir)? {
                if is_dot(&name) {
                    continue;
                }
                if path.is_dir() {
                    return Err(PackError::UnknownDirectory {
                        path,
                        known: "none: a data directory holds files only".to_owned(),
                    });
                }
                if path.extension().and_then(std::ffi::OsStr::to_str) != Some(DATA_EXTENSION) {
                    continue;
                }
                found.push(Found {
                    origin: DataOrigin {
                        pack: root.pack.to_owned(),
                        source: root.source,
                        directory: *directory,
                        path: path.clone(),
                    },
                    shown: path,
                    text: None,
                });
            }
        }
    }
    Ok(found)
}

/// The directories a World Pack's format gives a meaning: the four content kinds', `configure/` and
/// `data/`. The same six `DataDirectory::FORMAT` keeps every pack from owning (a test holds them equal).
pub(crate) const WORLD_DIRECTORIES: [&str; 6] = [
    ContentKind::Person.directory(),
    ContentKind::Place.directory(),
    ContentKind::Item.directory(),
    ContentKind::Organization.directory(),
    configure::DIRECTORY,
    mineworld_authoring::ATTACHMENT_DIRECTORY,
];

/// Read step 4d for a World Pack (`ARC-121` item 1): the directory rule for each required Entity Pack
/// (in id order) and the world, then every enabled data owner's context from its files and its own
/// configuration.
pub(crate) fn read(
    root: &Path,
    manifest: &WorldManifest,
    composition: &Composition,
    systems: &[Capability],
    configuration: &[FoundConfiguration],
) -> Result<Contexts, PackError> {
    let mut roots: Vec<DataRoot<'_>> = entities::required_entity_packs(composition)
        .map(|(pack, dir)| DataRoot {
            pack,
            source: DataSource::EntityPack,
            dir,
            format: &[mineworld_packages::ENTITY_ITEMS],
            admitted: Vec::new(),
        })
        .collect();
    roots.push(DataRoot {
        pack: &manifest.world.id,
        source: DataSource::World,
        dir: root,
        format: &WORLD_DIRECTORIES,
        admitted: manifest
            .decisions
            .as_ref()
            .and_then(DecisionsFile::directory)
            .map(str::to_owned)
            .into_iter()
            .collect(),
    });
    contexts(&roots, &AVAILABLE, systems, |owner| {
        configuration
            .iter()
            .find(|found| found.owner == owner)
            .map(|found| &*found.configuration)
    })
}

/// The directory rule for every root, then every enabled owner's context, in `enabled` order. `roots`
/// is the required Entity Packs in id order, then the world (I-b-7).
pub(crate) fn contexts<'c, O: DataOwnerOps>(
    roots: &[DataRoot<'_>],
    installed: &[O],
    enabled: &[O],
    configuration_of: impl Fn(O) -> Option<&'c dyn AuthoredConfiguration>,
) -> Result<BTreeMap<O, Arc<dyn OwnerContext>>, PackError> {
    for root in roots {
        check_directories(root, installed, enabled)?;
    }
    check_shipped(installed, enabled)?;
    let mut contexts = BTreeMap::new();
    for owner in enabled {
        if owner.directories().is_empty() {
            continue;
        }
        let mut files = Vec::new();
        for found in collect(*owner, enabled, roots)? {
            let directory = found.origin.directory;
            let decode = |text: &str| {
                serde_saphyr::with_deserializer_from_str(text, |file| {
                    owner.decode_file(directory, file)
                })
            };
            let decoded = match found.text {
                Some(text) => decode(text).map_err(|error| PackError::Malformed {
                    path: found.shown.clone(),
                    kind: DATA_KIND,
                    detail: error.to_string(),
                })?,
                None => parse_with(&found.shown, DATA_KIND, decode)?,
            };
            files.push((found.origin, decoded));
        }
        let context = owner
            .context(files, configuration_of(*owner))
            .map_err(|refusal| PackError::DataRefused {
                system: owner.id(),
                file: refusal.file,
                reason: refusal.reason,
            })?;
        contexts.insert(*owner, context);
    }
    Ok(contexts)
}

/// `mineworld packs validate` of an Entity Pack's data (`ARC-71` point 9, `ARC-121`): the directory rule
/// with every installed owner admitted, and each file decoded by its owner — file by file, with no
/// context, since no world composes it.
pub(crate) fn validate_entity_data(pack: &str, dir: &Path) -> Result<(), PackError> {
    let root = DataRoot {
        pack,
        source: DataSource::EntityPack,
        dir,
        format: &[mineworld_packages::ENTITY_ITEMS],
        admitted: Vec::new(),
    };
    check_directories(&root, &AVAILABLE, &AVAILABLE)?;
    for owner in AVAILABLE {
        for found in collect(owner, &[], std::slice::from_ref(&root))? {
            let directory = found.origin.directory;
            parse_with(&found.shown, DATA_KIND, |text| {
                serde_saphyr::with_deserializer_from_str(text, |file| {
                    owner.decode_file(directory, file)
                })
            })?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
