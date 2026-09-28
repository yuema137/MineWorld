//! Reading a pack off the disk, and refusing a bad one by name.
//!
//! Everything here is validation. Nothing in this module builds a [`World`](mineworld_kernel::World)
//! — that is [`crate::load`] — and the split is what makes `mineworld validate <world>` an honest
//! command rather than a partial load: a pack that reads is a pack whose files exist, parse,
//! cross-reference and ask only for capabilities this build has.
//!
//! # The order the checks run in
//!
//! ```text
//! 1  the directory exists, and world.yaml is in it
//! 2  world.yaml parses, with unknown fields refused
//! 3  the pack's id is its directory's name
//! 4  every system it enables exists here, and none twice
//! 5  every authoring key is declared once
//! 6  every declared key has its file, and every file is declared
//! 7  every person's place exists, and every seat is one of the people
//! 8  content that needs a capability has it enabled
//! ```
//!
//! The order is deliberate: each check assumes the previous one passed, so an author fixes one thing
//! at a time rather than reading a report about a file that could not be found *and* a key it
//! contains.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use mineworld_contracts::{EntityKey, SystemId};
use serde::de::DeserializeOwned;

use crate::catalog::{AVAILABLE, Capability, LOCATION_OWNER};
use crate::error::{ContentKind, Declared, PackError};
use crate::format::{AuthoredPerson, AuthoredPlace, WorldManifest};

/// The file every World Pack has.
pub const MANIFEST: &str = "world.yaml";

/// The extension a content file has. Anything else in `people/` or `places/` is not content and is
/// left alone — a README belongs in a pack as much as it does anywhere else.
const CONTENT_EXTENSION: &str = "yaml";

/// A World Pack that has been read and found consistent: the content, and nothing derived from it.
///
/// Holds no [`World`](mineworld_kernel::World) and allocates no identity. Two `WorldPack`s read from
/// one directory are equal in everything a load depends on, which is what makes loading the same pack
/// twice produce the same world (`AC-12`).
#[derive(Debug, Clone)]
pub struct WorldPack {
    root: PathBuf,
    id: String,
    name: String,
    systems: Vec<Capability>,
    places: BTreeMap<EntityKey, AuthoredPlace>,
    people: BTreeMap<EntityKey, AuthoredPerson>,
    seats: BTreeSet<EntityKey>,
}

impl WorldPack {
    /// Reads the pack in `root` and validates it, or says what is wrong and where.
    pub fn read(root: impl Into<PathBuf>) -> Result<Self, PackError> {
        let root = root.into();
        if !root.is_dir() {
            return Err(PackError::NotAPackDirectory { path: root });
        }

        let manifest_path = root.join(MANIFEST);
        let manifest: WorldManifest = parse(&manifest_path, "world")?;

        check_pack_id(&root, &manifest.world.id)?;
        let systems = resolve_systems(&manifest.systems)?;
        check_keys_are_declared_once(&manifest)?;

        let places = read_content(&root, &manifest.places, ContentKind::Place)?;
        let people = read_content(&root, &manifest.population, ContentKind::Person)?;
        check_nothing_undeclared(
            &root,
            ContentKind::Place,
            &manifest.places,
            Declared::Places,
        )?;
        check_nothing_undeclared(
            &root,
            ContentKind::Person,
            &manifest.population,
            Declared::Population,
        )?;

        let seats = seats_of(&manifest, &people)?;
        check_locations(&root, &people, &places, &systems)?;

        Ok(Self {
            root,
            id: manifest.world.id,
            name: manifest.world.name,
            systems,
            places,
            people,
            seats,
        })
    }

    /// The directory this pack was read from.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The pack's id, which is its directory's name.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// What the world is called, for a person.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The systems this world enables, in the order the pack states — which is installation order,
    /// and therefore reduction order.
    pub fn systems(&self) -> &[Capability] {
        &self.systems
    }

    /// The places, in key order.
    pub fn places(&self) -> &BTreeMap<EntityKey, AuthoredPlace> {
        &self.places
    }

    /// The people, in key order.
    pub fn people(&self) -> &BTreeMap<EntityKey, AuthoredPerson> {
        &self.people
    }

    /// The seats a client may occupy, in key order.
    pub fn seats(&self) -> &BTreeSet<EntityKey> {
        &self.seats
    }
}

/// Reads and parses one file, or reports the file that could not be read or did not parse.
///
/// The parser's report becomes the error's detail unaltered: it carries the line, the column and an
/// excerpt, and rewriting it in this crate's words would lose exactly the part an author needs.
fn parse<T: DeserializeOwned>(path: &Path, kind: &'static str) -> Result<T, PackError> {
    if !path.exists() {
        return Err(PackError::FileMissing {
            path: path.to_path_buf(),
        });
    }
    let text = std::fs::read_to_string(path).map_err(|source| PackError::Unreadable {
        path: path.to_path_buf(),
        source,
    })?;
    serde_saphyr::from_str(&text).map_err(|error| PackError::Malformed {
        path: path.to_path_buf(),
        kind,
        detail: error.to_string(),
    })
}

/// A pack's id is its directory's name, or it has two identities and therefore none.
fn check_pack_id(root: &Path, declared: &str) -> Result<(), PackError> {
    let canonical = root
        .canonicalize()
        .map_err(|source| PackError::Unreadable {
            path: root.to_path_buf(),
            source,
        })?;
    let directory = canonical
        .file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().into_owned());
    if directory == declared {
        return Ok(());
    }
    Err(PackError::PackIdIsNotItsDirectory {
        declared: declared.to_owned(),
        directory,
    })
}

/// Every enabled system exists in this build, and none is named twice.
fn resolve_systems(declared: &[SystemId]) -> Result<Vec<Capability>, PackError> {
    let mut resolved = Vec::with_capacity(declared.len());
    let mut seen = BTreeSet::new();
    for name in declared {
        let capability = Capability::resolve(name).ok_or_else(|| PackError::UnknownSystem {
            system: name.clone(),
            available: names(AVAILABLE.into_iter().map(Capability::id)),
        })?;
        if !seen.insert(capability) {
            return Err(PackError::SystemDeclaredTwice {
                system: name.clone(),
            });
        }
        resolved.push(capability);
    }
    Ok(resolved)
}

/// One key, one entity. A repeat is refused rather than merged, because a key is how authored content
/// refers to an entity and the kernel refuses the second entity claiming one.
fn check_keys_are_declared_once(manifest: &WorldManifest) -> Result<(), PackError> {
    let mut seen: BTreeMap<&EntityKey, Declared> = BTreeMap::new();
    let entities = manifest
        .places
        .iter()
        .map(|key| (key, Declared::Places))
        .chain(
            manifest
                .population
                .iter()
                .map(|key| (key, Declared::Population)),
        );
    for (key, list) in entities {
        if let Some(first) = seen.insert(key, list) {
            return Err(PackError::KeyDeclaredTwice {
                key: key.clone(),
                first,
                second: list,
            });
        }
    }

    // Seats are checked among themselves: a seat *is* one of the people, so a seat repeating a
    // population key is the normal case rather than a collision.
    let mut seats = BTreeSet::new();
    for seat in &manifest.seats {
        if !seats.insert(seat) {
            return Err(PackError::KeyDeclaredTwice {
                key: seat.clone(),
                first: Declared::Seats,
                second: Declared::Seats,
            });
        }
    }
    Ok(())
}

/// Reads the file each declared key names.
fn read_content<T: DeserializeOwned>(
    root: &Path,
    declared: &[EntityKey],
    kind: ContentKind,
) -> Result<BTreeMap<EntityKey, T>, PackError> {
    let mut content = BTreeMap::new();
    for key in declared {
        let path = content_path(root, kind, key.as_str());
        if !path.exists() {
            return Err(PackError::ContentFileMissing {
                key: key.clone(),
                kind,
                path,
            });
        }
        content.insert(key.clone(), parse(&path, kind.describes())?);
    }
    Ok(content)
}

/// Nothing in a content directory is left out of the manifest.
///
/// The commonest authoring mistake this format admits: write `people/carol.yaml`, forget to add
/// `carol` to `population`, and get a world with no Carol in it and nothing to explain why. Only
/// `.yaml` files are considered content; a README or a subdirectory is not.
fn check_nothing_undeclared(
    root: &Path,
    kind: ContentKind,
    declared: &[EntityKey],
    list: Declared,
) -> Result<(), PackError> {
    let directory = root.join(kind.directory());
    if !directory.is_dir() {
        return Ok(());
    }
    let entries = std::fs::read_dir(&directory).map_err(|source| PackError::Unreadable {
        path: directory.clone(),
        source,
    })?;

    // Collected and sorted, so that a pack with two undeclared files is refused for the same one on
    // every machine: `read_dir` order is the filesystem's, and a refusal that varied between runs
    // would be a refusal an author cannot reproduce.
    let mut found = BTreeSet::new();
    for entry in entries {
        let entry = entry.map_err(|source| PackError::Unreadable {
            path: directory.clone(),
            source,
        })?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some(CONTENT_EXTENSION) {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
            found.insert(stem.to_owned());
        }
    }

    let declared: BTreeSet<&str> = declared.iter().map(EntityKey::as_str).collect();
    for stem in &found {
        if !declared.contains(stem.as_str()) {
            return Err(PackError::ContentFileNotDeclared {
                key: stem.clone(),
                kind,
                list,
                path: content_path(root, kind, stem),
            });
        }
    }
    Ok(())
}

/// Every seat is one of this pack's people.
fn seats_of(
    manifest: &WorldManifest,
    people: &BTreeMap<EntityKey, AuthoredPerson>,
) -> Result<BTreeSet<EntityKey>, PackError> {
    let mut seats = BTreeSet::new();
    for seat in &manifest.seats {
        if !people.contains_key(seat) {
            return Err(PackError::SeatIsNotOneOfThePeople { seat: seat.clone() });
        }
        seats.insert(seat.clone());
    }
    Ok(seats)
}

/// Every authored location names a place this pack has, and asks for a capability it enabled.
fn check_locations(
    root: &Path,
    people: &BTreeMap<EntityKey, AuthoredPerson>,
    places: &BTreeMap<EntityKey, AuthoredPlace>,
    systems: &[Capability],
) -> Result<(), PackError> {
    for (key, person) in people {
        let Some(location) = &person.location else {
            continue;
        };
        let path = content_path(root, ContentKind::Person, key.as_str());
        if !places.contains_key(&location.place) {
            return Err(PackError::PersonInUnknownPlace {
                person: key.clone(),
                place: location.place.clone(),
                known: names(places.keys().cloned()),
                path,
            });
        }
        if !systems.contains(&LOCATION_OWNER) {
            return Err(PackError::ContentNeedsASystem {
                subject: key.clone(),
                content: "location",
                system: LOCATION_OWNER.id(),
                path,
            });
        }
    }
    Ok(())
}

/// Where a content file lives, from the key that names it.
fn content_path(root: &Path, kind: ContentKind, key: &str) -> PathBuf {
    root.join(kind.directory())
        .join(format!("{key}.{CONTENT_EXTENSION}"))
}

/// A list of names for a person to read, in the order given.
fn names(values: impl IntoIterator<Item = impl core::fmt::Display>) -> String {
    let listed: Vec<String> = values
        .into_iter()
        .map(|value| format!("'{value}'"))
        .collect();
    if listed.is_empty() {
        "none".to_owned()
    } else {
        listed.join(", ")
    }
}
