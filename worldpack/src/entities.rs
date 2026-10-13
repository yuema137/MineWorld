//! Entity Packs: item kinds a world requires rather than declares (`DECISIONS.md` `ARC-71`).
//!
//! `mineworld-packages` has already identified each one and checked its layout (item kinds only, at
//! least one). This module reads what the files say, with the item-file format and the section
//! decoding a world's own `items/` uses — so a kind means the same thing wherever it was written:
//!
//! ```text
//! read      every items/<key>.yaml, key = the file's stem (an EntityKey or refused), into a map by
//!           key — never in the file system's listing order (PD-q3) — decoded with the requiring
//!           world's enabled systems (read order step 6b)
//! confine   a section names only kinds of the same pack (self-contained, ARC-71 point 8)
//! merge     into the world's items, one key namespace across every source, a key stated by two
//!           sources refused naming both (ARC-71 point 5)
//! ```
//!
//! Where each kind came from is kept beside it ([`ItemSource`]), for provenance and for every refusal
//! that names a content file.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mineworld_contracts::{EntityKey, EntityType};
use mineworld_packages::{
    Composition, ENTITY_ITEM_EXTENSION, ENTITY_ITEMS, PackType, Source, read_pack_file,
};

use crate::catalog::{AVAILABLE, Capability};
use crate::content::ContentFile;
use crate::error::{ContentKind, Declared, PackError};
use crate::format::{AuthoredItem, SectionState, WorldManifest};
use crate::read::parse_with;

/// Where an item kind was declared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ItemSource {
    /// The world's own `items:` and `items/`.
    World,
    /// A required Entity Pack: its id and its directory.
    EntityPack {
        /// The pack's id, which becomes the kind's `source_pack`.
        id: String,
        /// The pack's directory, which holds the kind's file.
        dir: PathBuf,
    },
}

/// A world's item kinds composed with its Entity Packs', and where each came from.
pub(crate) type Composed = (
    BTreeMap<EntityKey, AuthoredItem>,
    BTreeMap<EntityKey, ItemSource>,
);

/// One Entity Pack's kinds, read.
struct EntityPackContent {
    id: String,
    dir: PathBuf,
    items: BTreeMap<EntityKey, AuthoredItem>,
}

impl EntityPackContent {
    /// Reads `dir`'s item files, decoding each with `systems`.
    fn read(id: &str, dir: &Path, systems: &[Capability]) -> Result<Self, PackError> {
        let directory = dir.join(ENTITY_ITEMS);
        let unreadable = |source| PackError::Unreadable {
            path: directory.clone(),
            source,
        };
        let mut files = BTreeMap::new();
        for entry in std::fs::read_dir(&directory).map_err(unreadable)? {
            let path = entry.map_err(unreadable)?.path();
            if path.extension().and_then(std::ffi::OsStr::to_str) != Some(ENTITY_ITEM_EXTENSION) {
                continue;
            }
            let stem = path
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_default();
            let key = EntityKey::new(&stem).map_err(|source| PackError::EntityPackKeyInvalid {
                pack: id.to_owned(),
                path: path.clone(),
                source,
            })?;
            files.insert(key, path);
        }
        let mut items = BTreeMap::new();
        for (key, path) in files {
            let item = parse_with(&path, ContentKind::Item.describes(), |text| {
                serde_saphyr::with_deserializer_from_str(text, |file| {
                    ContentFile::new(ContentKind::Item, systems).item(file)
                })
            })?;
            items.insert(key, item);
        }
        let pack = Self {
            id: id.to_owned(),
            dir: dir.to_path_buf(),
            items,
        };
        pack.check_self_contained()?;
        Ok(pack)
    }

    /// The file a kind of this pack was read from.
    fn file(&self, key: &EntityKey) -> PathBuf {
        item_file(&self.dir, key)
    }

    /// Every decoded section names only kinds this pack declares (`ARC-71` point 8).
    fn check_self_contained(&self) -> Result<(), PackError> {
        for (subject, item) in &self.items {
            for section in &item.sections {
                let SectionState::Decoded(content) = &section.state else {
                    continue;
                };
                for reference in content.references() {
                    let own = reference.entity_type == EntityType::Item
                        && self.items.contains_key(reference.key);
                    if !own {
                        return Err(PackError::EntityPackReachesOut {
                            pack: self.id.clone(),
                            subject: subject.clone(),
                            section: section.name,
                            key: reference.key.clone(),
                            path: self.file(subject),
                        });
                    }
                }
            }
        }
        Ok(())
    }
}

/// An Entity Pack's item file for `key` under `dir`.
pub(crate) fn item_file(dir: &Path, key: &EntityKey) -> PathBuf {
    dir.join(ContentKind::Item.directory())
        .join(format!("{key}.{ENTITY_ITEM_EXTENSION}"))
}

/// Read order step 6b: every required Entity Pack's kinds, in requirement (id) order, read with the
/// world's enabled `systems` and merged into the world's own `items`, one namespace across every source.
/// Returns the merged kinds and where each came from.
pub(crate) fn compose(
    manifest: &WorldManifest,
    composition: &Composition,
    systems: &[Capability],
    items: BTreeMap<EntityKey, AuthoredItem>,
) -> Result<Composed, PackError> {
    // Every key the world declares, with the list that declares it — `world.yaml`'s lists are already
    // one namespace (rule 1), checked before content was read.
    let mut sources: BTreeMap<EntityKey, String> = BTreeMap::new();
    for (keys, list) in [
        (&manifest.places, Declared::Places),
        (&manifest.population, Declared::Population),
        (&manifest.items, Declared::Items),
        (&manifest.organizations, Declared::Organizations),
    ] {
        for key in keys {
            sources.insert(key.clone(), format!("world.yaml's {list}"));
        }
    }
    let mut item_sources: BTreeMap<EntityKey, ItemSource> = items
        .keys()
        .map(|key| (key.clone(), ItemSource::World))
        .collect();
    let mut merged = items;
    for required in &composition.required {
        let (PackType::EntityPack, Source::Directory(dir)) =
            (required.identity.kind, &required.source)
        else {
            continue;
        };
        let pack = EntityPackContent::read(required.identity.id.as_str(), dir, systems)?;
        for (key, item) in pack.items.iter() {
            let here = format!("the Entity Pack {} ({})", pack.id, pack.file(key).display());
            if let Some(first) = sources.get(key) {
                return Err(PackError::KeyFromTwoSources {
                    key: key.clone(),
                    first: first.clone(),
                    second: here,
                });
            }
            sources.insert(key.clone(), here);
            item_sources.insert(
                key.clone(),
                ItemSource::EntityPack {
                    id: pack.id.clone(),
                    dir: pack.dir.clone(),
                },
            );
            merged.insert(key.clone(), item.clone());
        }
    }
    Ok((merged, item_sources))
}

/// `mineworld packs validate` of an Entity Pack (`ARC-71` point 9): its identity and layout (through
/// `mineworld-packages`), then every item file read against **this build's whole installed set** — each
/// section decoded by its owner; whether a world enables that owner is that world's question — refusing a
/// section the item file may not carry, and checking the pack is self-contained. Returns the pack's id and
/// its kinds, in key order. The identity's licence and framework range are the caller's to judge.
///
/// # Errors
///
/// The first [`PackError`]: a `pack.yaml` refusal (as [`PackError::Requirements`] naming `pack.yaml`),
/// an item file whose name is not a key, one that does not parse, a section not carried by item files,
/// or a section naming a key outside the pack.
pub fn validate_entity_pack(dir: &Path) -> Result<Vec<EntityKey>, PackError> {
    let identity = read_pack_file(dir).map_err(|refusal| PackError::Requirements {
        path: dir.join(mineworld_packages::PACK_FILE),
        refusal: Box::new(refusal),
    })?;
    let pack = EntityPackContent::read(identity.id.as_str(), dir, &AVAILABLE)?;
    for (subject, item) in &pack.items {
        for section in &item.sections {
            if matches!(section.state, SectionState::NotCarriedHere) {
                return Err(crate::read::not_carried_here(
                    subject,
                    section,
                    ContentKind::Item,
                    pack.file(subject),
                ));
            }
        }
    }
    Ok(pack.items.into_keys().collect())
}
