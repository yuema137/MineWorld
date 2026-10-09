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
//! 3  the pack's id is its directory's name, and a stated `mineworld:` range admits this framework
//! 4  every system it enables exists here, and none twice
//! 4b its requirements resolve in this build and the pack roots, and every pack in its composition
//!    carries a licence the policy allows (ARC-54, ARC-55)
//! 4c every configure: key is an enabled, configurable system of this build, listed once and not
//!    reserved; its file exists and its owner's type decodes it; every file in configure/ is listed;
//!    every system a configuration requires is enabled (ARC-61)
//! 5  every authoring key is declared once, across places, population, items and organizations
//! 6  every declared key has its file, and every file in people/, places/, items/ and
//!    organizations/ is declared
//! 7  every person's place exists, and every seat is one of the people
//! 8  content that needs a capability has it enabled
//! 9  every passage joins two distinct declared places, each pair once, with `movement` enabled
//! 10 every section's owner is enabled, its file may carry it, and every entity it names is
//!    declared, of the type its owner needs (ARC-31) — what a section says was already checked as it
//!    was read, by its owner's own type; then every entity a configuration names, the same way
//! ```
//!
//! The order is deliberate: each check assumes the previous one passed, so an author fixes one thing
//! at a time rather than reading a report about a file that could not be found *and* a key it
//! contains.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use mineworld_contracts::{EntityKey, EntityType, SystemId};
use mineworld_packages::{Compatibility, Composition, License, PackRoots, Version};
use serde::de::DeserializeOwned;

use crate::catalog::{AVAILABLE, Capability, LOCATION_OWNER, PASSAGE_OWNER};
use crate::configure;
use crate::content::ContentFile;
use crate::error::{ContentKind, Declared, PackError};
use crate::format::{
    AuthoredItem, AuthoredOrganization, AuthoredPerson, AuthoredPlace, FoundConfiguration,
    FoundSection, SectionState, WorldManifest,
};

/// The file every World Pack has.
pub const MANIFEST: &str = "world.yaml";

/// A World Pack's package fields, as `world.yaml` states them (`DECISIONS.md` `ARC-53`): each typed and
/// checked when present, all optional to the loader. `mineworld packs validate` requires them, through
/// `mineworld_packages::Identity::of_world`.
#[derive(Debug, Clone, Default)]
pub struct PackageFields {
    /// `world.version`.
    pub version: Option<Version>,
    /// `world.license`.
    pub license: Option<License>,
    /// `mineworld:`, already found to admit this framework.
    pub mineworld: Option<Compatibility>,
}

/// The extension a content file has. Anything else in a content directory is not content and is
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
    package: PackageFields,
    composition: Composition,
    systems: Vec<Capability>,
    places: BTreeMap<EntityKey, AuthoredPlace>,
    people: BTreeMap<EntityKey, AuthoredPerson>,
    items: BTreeMap<EntityKey, AuthoredItem>,
    organizations: BTreeMap<EntityKey, AuthoredOrganization>,
    seats: BTreeSet<EntityKey>,
    configuration: Vec<FoundConfiguration>,
}

impl WorldPack {
    /// Reads the pack in `root` and validates it, or says what is wrong and where — resolving its
    /// requirements in this build alone, with no pack root. A world without `requires:` reads exactly
    /// as it did before requirements existed.
    pub fn read(root: impl Into<PathBuf>) -> Result<Self, PackError> {
        Self::read_with(root, &PackRoots::none())
    }

    /// As [`WorldPack::read`], resolving the world's requirements in this build and in `roots`
    /// (`DECISIONS.md` `ARC-54`).
    pub fn read_with(root: impl Into<PathBuf>, roots: &PackRoots) -> Result<Self, PackError> {
        let root = root.into();
        if !root.is_dir() {
            return Err(PackError::NotAPackDirectory { path: root });
        }

        let manifest_path = root.join(MANIFEST);
        let manifest: WorldManifest = parse(&manifest_path, "world")?;

        check_pack_id(&root, &manifest.world.id)?;
        if let Some(range) = &manifest.mineworld {
            range
                .require_framework()
                .map_err(|refusal| PackError::FrameworkNotSupported {
                    path: manifest_path.clone(),
                    refusal: Box::new(refusal),
                })?;
        }
        let systems = resolve_systems(&manifest.systems)?;
        let composition = crate::requirements::resolve(&manifest_path, &manifest, &systems, roots)?;
        let configuration = configure::read(&root, &manifest.configure, &systems)?;
        check_keys_are_declared_once(&manifest)?;

        let places = read_content(&root, &manifest.places, ContentKind::Place, |text| {
            serde_saphyr::with_deserializer_from_str(text, |file| {
                ContentFile::new(ContentKind::Place, &systems).place(file)
            })
        })?;
        let people = read_content(&root, &manifest.population, ContentKind::Person, |text| {
            serde_saphyr::with_deserializer_from_str(text, |file| {
                ContentFile::new(ContentKind::Person, &systems).person(file)
            })
        })?;
        let items = read_content(&root, &manifest.items, ContentKind::Item, |text| {
            serde_saphyr::with_deserializer_from_str(text, |file| {
                ContentFile::new(ContentKind::Item, &systems).item(file)
            })
        })?;
        let organizations = read_content(
            &root,
            &manifest.organizations,
            ContentKind::Organization,
            |text| {
                serde_saphyr::with_deserializer_from_str(text, |file| {
                    ContentFile::new(ContentKind::Organization, &systems).organization(file)
                })
            },
        )?;
        for (kind, declared, list) in [
            (ContentKind::Place, &manifest.places, Declared::Places),
            (
                ContentKind::Person,
                &manifest.population,
                Declared::Population,
            ),
            (ContentKind::Item, &manifest.items, Declared::Items),
            (
                ContentKind::Organization,
                &manifest.organizations,
                Declared::Organizations,
            ),
        ] {
            check_nothing_undeclared(&root, kind, declared, list)?;
        }

        let seats = seats_of(&manifest, &people)?;
        check_locations(&root, &people, &places, &systems)?;
        check_passages(&root, &places, &systems)?;

        let pack = Self {
            root,
            id: manifest.world.id,
            name: manifest.world.name,
            package: PackageFields {
                version: manifest.world.version,
                license: manifest.world.license,
                mineworld: manifest.mineworld,
            },
            composition,
            systems,
            places,
            people,
            items,
            organizations,
            seats,
            configuration,
        };
        check_sections(&pack)?;
        configure::check_references(&pack)?;
        Ok(pack)
    }

    /// The world's resolved composition (`DECISIONS.md` `ARC-54`): each requirement with the pack that
    /// met it, each enabled system's pack — for `mineworld validate` and `mineworld packs` only. Nothing
    /// that builds a world reads it: it is not world state.
    pub fn composition(&self) -> &Composition {
        &self.composition
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

    /// The package fields `world.yaml` states (`DECISIONS.md` `ARC-53`), each already checked —
    /// for `mineworld packs` only. Nothing that builds a world reads them: they are not world state.
    pub fn package_fields(&self) -> &PackageFields {
        &self.package
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

    /// The item kinds, in key order (`ARC-36`).
    pub fn items(&self) -> &BTreeMap<EntityKey, AuthoredItem> {
        &self.items
    }

    /// The organizations, in key order.
    pub fn organizations(&self) -> &BTreeMap<EntityKey, AuthoredOrganization> {
        &self.organizations
    }

    /// The seats a client may occupy, in key order.
    pub fn seats(&self) -> &BTreeSet<EntityKey> {
        &self.seats
    }

    /// The System Packs this world configures, each decoded by its owner, in `configure:` order —
    /// which is seeding order (`DECISIONS.md` `ARC-61`).
    pub fn configuration(&self) -> &[FoundConfiguration] {
        &self.configuration
    }

    /// Every content file's sections, in the one order every per-file pass uses: items', then
    /// organizations', then places', then people's, each in key order (`ARC-36` item 7).
    ///
    /// The loader refuses sections in this order and seeds them in this order, so the first refusal
    /// and the genesis log are the same statement. What a person's or a place's section may name is
    /// seeded before it; a pack with no items or organizations yields exactly places then people.
    pub(crate) fn sectioned_files(
        &self,
    ) -> impl Iterator<Item = (ContentKind, &EntityKey, &Vec<FoundSection>)> {
        let items = self
            .items
            .iter()
            .map(|(key, item)| (ContentKind::Item, key, &item.sections));
        let organizations = self
            .organizations
            .iter()
            .map(|(key, organization)| (ContentKind::Organization, key, &organization.sections));
        let places = self
            .places
            .iter()
            .map(|(key, place)| (ContentKind::Place, key, &place.sections));
        let people = self
            .people
            .iter()
            .map(|(key, person)| (ContentKind::Person, key, &person.sections));
        items.chain(organizations).chain(places).chain(people)
    }

    /// A pack built in memory, for this crate's own tests of what no installed pack can show yet: a
    /// section on an item or organization file (F-22). Never read from disk and never checked.
    #[cfg(test)]
    pub(crate) fn in_memory(
        systems: Vec<Capability>,
        places: BTreeMap<EntityKey, AuthoredPlace>,
        people: BTreeMap<EntityKey, AuthoredPerson>,
        items: BTreeMap<EntityKey, AuthoredItem>,
        organizations: BTreeMap<EntityKey, AuthoredOrganization>,
    ) -> Self {
        Self {
            root: PathBuf::from("in-memory"),
            id: "in-memory".to_owned(),
            name: "In Memory".to_owned(),
            package: PackageFields::default(),
            composition: Composition {
                framework: mineworld_packages::framework_version(),
                mineworld: None,
                required: Vec::new(),
                systems: Vec::new(),
            },
            systems,
            places,
            people,
            items,
            organizations,
            seats: BTreeSet::new(),
            configuration: Vec::new(),
        }
    }

    /// The same in-memory pack, configured with probe configurations (`crate::configure`'s tests).
    #[cfg(test)]
    pub(crate) fn with_configuration(mut self, configuration: Vec<FoundConfiguration>) -> Self {
        self.configuration = configuration;
        self
    }

    /// Every declared key and the entity type it will be: one namespace across the four lists.
    pub(crate) fn declared_entities(&self) -> BTreeMap<&EntityKey, EntityType> {
        let places = self.places.keys().map(|key| (key, EntityType::Place));
        let people = self.people.keys().map(|key| (key, EntityType::Person));
        let items = self.items.keys().map(|key| (key, EntityType::Item));
        let organizations = self
            .organizations
            .keys()
            .map(|key| (key, EntityType::Organization));
        places
            .chain(people)
            .chain(items)
            .chain(organizations)
            .collect()
    }
}

/// Reads and parses one file, or reports the file that could not be read or did not parse.
///
/// The parser's report becomes the error's detail unaltered: it carries the line, the column and an
/// excerpt, and rewriting it in this crate's words would lose exactly the part an author needs.
pub(crate) fn parse<T: DeserializeOwned>(path: &Path, kind: &'static str) -> Result<T, PackError> {
    parse_with(path, kind, |text| serde_saphyr::from_str(text))
}

/// [`parse`], with the decoding supplied: a content file is decoded through a seed that knows which
/// sections this build's packs own ([`ContentFile`]), and a configuration file by its owner's type
/// (`crate::configure`). `serde-saphyr` is called from this module and that one only (`DEP-10`).
pub(crate) fn parse_with<T>(
    path: &Path,
    kind: &'static str,
    decode: impl FnOnce(&str) -> Result<T, serde_saphyr::Error>,
) -> Result<T, PackError> {
    if !path.exists() {
        return Err(PackError::FileMissing {
            path: path.to_path_buf(),
        });
    }
    let text = std::fs::read_to_string(path).map_err(|source| PackError::Unreadable {
        path: path.to_path_buf(),
        source,
    })?;
    decode(&text).map_err(|error| PackError::Malformed {
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

/// One key, one entity, across all four lists. A repeat is refused rather than merged, because a key
/// is how authored content refers to an entity and the kernel refuses the second entity claiming one.
fn check_keys_are_declared_once(manifest: &WorldManifest) -> Result<(), PackError> {
    let mut seen: BTreeMap<&EntityKey, Declared> = BTreeMap::new();
    let lists = [
        (&manifest.places, Declared::Places),
        (&manifest.population, Declared::Population),
        (&manifest.items, Declared::Items),
        (&manifest.organizations, Declared::Organizations),
    ];
    let entities = lists
        .into_iter()
        .flat_map(|(keys, list)| keys.iter().map(move |key| (key, list)));
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
    // population key is the normal case, not a clash.
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
fn read_content<T>(
    root: &Path,
    declared: &[EntityKey],
    kind: ContentKind,
    decode: impl Fn(&str) -> Result<T, serde_saphyr::Error>,
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
        content.insert(key.clone(), parse_with(&path, kind.describes(), &decode)?);
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

/// Every passage joins two distinct places this pack has, each pair is joined once, and the pack
/// enables the system that owns passages.
fn check_passages(
    root: &Path,
    places: &BTreeMap<EntityKey, AuthoredPlace>,
    systems: &[Capability],
) -> Result<(), PackError> {
    let mut joined = BTreeSet::new();
    for (key, place) in places {
        let path = content_path(root, ContentKind::Place, key.as_str());
        for passage in &place.passages {
            if !places.contains_key(&passage.to) {
                return Err(PackError::PassageToUnknownPlace {
                    place: key.clone(),
                    to: passage.to.clone(),
                    known: names(places.keys().cloned()),
                    path,
                });
            }
            if passage.to == *key {
                return Err(PackError::PassageToItself {
                    place: key.clone(),
                    path,
                });
            }
            let pair = if *key < passage.to {
                (key.clone(), passage.to.clone())
            } else {
                (passage.to.clone(), key.clone())
            };
            if !joined.insert(pair.clone()) {
                return Err(PackError::PassageStatedTwice {
                    first: pair.0,
                    second: pair.1,
                    path,
                });
            }
            if !systems.contains(&PASSAGE_OWNER) {
                return Err(PackError::ContentNeedsASystem {
                    subject: key.clone(),
                    content: "passage",
                    system: PASSAGE_OWNER.id(),
                    path,
                });
            }
        }
    }
    Ok(())
}

/// Every section is owned by an enabled pack, sits in a kind of file its owner lets carry it, and
/// names only entities this pack declares, of the type its owner needs (`ARC-31`).
///
/// What a section *says* was checked as it was read, by its owner's own type; this checks only what
/// every section shares. Files are checked in [`WorldPack::sectioned_files`]' order, so the first
/// refusal is the same on every machine. A reference is accepted iff its key is declared in the list of
/// the type it requires (`ARC-36` item 5).
pub(crate) fn check_sections(pack: &WorldPack) -> Result<(), PackError> {
    let root = &pack.root;
    let declared_entities = pack.declared_entities();
    for (kind, subject, sections) in pack.sectioned_files() {
        let path = content_path(root, kind, subject.as_str());
        for section in sections {
            let content = match &section.state {
                SectionState::Decoded(content) => content,
                SectionState::OwnerNotEnabled => {
                    return Err(PackError::ContentNeedsASystem {
                        subject: subject.clone(),
                        content: section.name.as_str(),
                        system: section.owner.id(),
                        path,
                    });
                }
                SectionState::NotCarriedHere => {
                    let carried_by = section
                        .owner
                        .section()
                        .map(|owner| {
                            owner
                                .carried_by
                                .iter()
                                .map(|kind| kind.describes())
                                .collect::<Vec<_>>()
                                .join(" and ")
                        })
                        .unwrap_or_default();
                    return Err(PackError::SectionNotCarriedHere {
                        subject: subject.clone(),
                        section: section.name,
                        kind,
                        carried_by,
                        path,
                    });
                }
            };
            for reference in content.references() {
                let declared = declared_entities.get(reference.key) == Some(&reference.entity_type);
                if !declared {
                    return Err(PackError::SectionNamesUnknownEntity {
                        subject: subject.clone(),
                        section: section.name,
                        key: reference.key.clone(),
                        expected: reference.entity_type,
                        path,
                    });
                }
            }
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
