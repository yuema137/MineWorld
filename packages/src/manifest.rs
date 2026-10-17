//! `pack.yaml`: the package identity of a data pack that has no other carrier (`ARC-53`,
//! `PACKAGE_FORMAT.md` §5.0); the one content check E-a makes of a Presentation Pack; and an Entity
//! Pack's directory layout (`ARC-71`) — file names only: what an item file says is the World Pack
//! loader's to read.

use std::path::Path;

use serde::Deserialize;

use crate::error::PackageError;
use crate::identity::{Compatibility, Identity, License, PackId, PackType, Version};

/// The file a data pack states its identity in.
pub const PACK_FILE: &str = "pack.yaml";

/// A Presentation Pack's style manifest (`ART_DIRECTION.md` §12) — not a package manifest.
pub const STYLE_FILE: &str = "manifest.yaml";

/// The directory an Entity Pack's item kinds are in, one `<key>.yaml` per kind (`ARC-71`).
pub const ENTITY_ITEMS: &str = "items";

/// The extension of an Entity Pack's item file; any other file in [`ENTITY_ITEMS`] is not content.
pub const ENTITY_ITEM_EXTENSION: &str = "yaml";

/// The content directories of a World Pack that an Entity Pack may not carry in MVP-0.
const NOT_IN_AN_ENTITY_PACK: [&str; 3] = ["places", "people", "organizations"];

/// `pack.yaml`, exactly: any other field is refused, `dependencies` included until a world's
/// requirements are resolved, so no field is accepted that nothing checks.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(example = example_pack_file())]
struct PackFile {
    /// The pack's id; a world names it in `requires:`, and it must equal the pack directory's name.
    id: PackId,
    /// What kind of pack this file identifies: presentation-pack or entity-pack.
    #[serde(rename = "type")]
    kind: PackType,
    /// The pack's release version, semver.
    version: Version,
    /// The framework versions the pack works with, a semver range such as "^0.1".
    mineworld: Compatibility,
    /// The pack's licence, an SPDX expression; the world's licence policy must allow it.
    license: License,
    /// Who made the pack: at least one name.
    authors: Vec<String>,
    /// Where the pack's source lives, a URL.
    #[serde(default)]
    repository: Option<String>,
}

/// The example the generated reference prints for a `pack.yaml`.
fn example_pack_file() -> serde_json::Value {
    serde_json::json!({
        "id": "seaside-goods",
        "type": "entity-pack",
        "version": "0.1.0",
        "mineworld": "^0.1",
        "license": "MIT",
        "authors": ["Ana Lee"],
    })
}

/// What `mineworld packs` needs of a style manifest; the rest of its schema is `ART_DIRECTION.md`'s,
/// so other fields are not this crate's to refuse.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(example = serde_json::json!({ "id": "cosy-town", "dimension": ["2d", "3d"] }))]
struct StyleManifest {
    /// The style's id.
    id: String,
    /// The dimensions the style draws in, such as [3d]: at least one. The manifest's other keys are
    /// the art direction's (docs/ART_DIRECTION.md §12) and are not checked here.
    dimension: Vec<String>,
}

/// The JSON Schema of a data pack's `pack.yaml` (`ARC-90`), for the generated reference.
#[must_use]
pub fn pack_file_schema() -> schemars::Schema {
    schemars::SchemaGenerator::default().into_root_schema_for::<PackFile>()
}

/// The JSON Schema of the part of a Presentation Pack's style manifest this crate reads (`ARC-90`).
#[must_use]
pub fn style_manifest_schema() -> schemars::Schema {
    schemars::SchemaGenerator::default().into_root_schema_for::<StyleManifest>()
}

fn read(path: &Path) -> Result<String, PackageError> {
    std::fs::read_to_string(path).map_err(|error| PackageError::Unreadable {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })
}

fn malformed(path: &Path, error: impl std::fmt::Display) -> PackageError {
    PackageError::Malformed {
        path: path.to_path_buf(),
        message: error.to_string(),
    }
}

/// Reads and checks the `pack.yaml` in `dir`.
///
/// # Errors
///
/// [`PackageError::Malformed`] for a file that is not valid YAML, an unknown or missing field, or a
/// malformed value (with the parser's line and column); [`PackageError::NoAuthors`] inside it for an
/// empty author list; [`PackageError::NotCarriedHere`] for a type this file does not identify; for an
/// Entity Pack, [`check_entity_layout`]'s refusals.
pub fn read_pack_file(dir: &Path) -> Result<Identity, PackageError> {
    let path = dir.join(PACK_FILE);
    let file: PackFile = serde_saphyr::from_str(&read(&path)?).map_err(|e| malformed(&path, e))?;
    let not_here = |why| PackageError::NotCarriedHere {
        path: path.clone(),
        kind: file.kind.to_string(),
        why,
    };
    match file.kind {
        PackType::PresentationPack | PackType::EntityPack => {}
        PackType::WorldPack => {
            return Err(not_here("a World Pack is identified by its world.yaml"));
        }
        PackType::SystemPack | PackType::ControllerPack => {
            return Err(not_here(
                "a code pack is identified by its Cargo.toml, through package!()",
            ));
        }
    }
    if file.authors.is_empty() || file.authors.iter().any(|a| a.trim().is_empty()) {
        return Err(malformed(&path, PackageError::NoAuthors));
    }
    if file.kind == PackType::EntityPack {
        check_entity_layout(dir)?;
    }
    Ok(Identity {
        id: file.id,
        kind: file.kind,
        version: file.version,
        license: file.license,
        authors: file.authors,
        repository: file.repository,
        mineworld: Some(file.mineworld),
    })
}

/// Checks an Entity Pack's directory `dir` (`ARC-71` point 1, amended by `ARC-121`): it carries no
/// `places/`, `people/` or `organizations/`, and it declares something — a file in `items/`, or in any
/// other directory whose name does not begin with `.`, whose extension is `yaml`. Only names are read:
/// whether each name is a key, and what each file says, is the World Pack loader's.
///
/// Which directories besides `items/` are *known* is not decided here: they are the data directories of
/// installed System Packs (`ARC-121` item 1), and this crate does not know the installed packs. The loader
/// decides, when a world requires the pack, and `mineworld packs validate` against the build's installed
/// set; a data-only pack therefore passes this check and is judged there.
///
/// # Errors
///
/// [`PackageError::EntityPackCarries`] naming the first directory it may not have;
/// [`PackageError::EntityPackDeclaresNothing`] when no such directory holds a `.yaml` file;
/// [`PackageError::Unreadable`] when a directory cannot be listed.
pub fn check_entity_layout(dir: &Path) -> Result<(), PackageError> {
    for name in NOT_IN_AN_ENTITY_PACK {
        let path = dir.join(name);
        if path.exists() {
            return Err(PackageError::EntityPackCarries { path });
        }
    }
    let items = dir.join(ENTITY_ITEMS);
    if items.is_dir() && holds_a_yaml_file(&items)? {
        return Ok(());
    }
    let unreadable = |error: std::io::Error| PackageError::Unreadable {
        path: dir.to_path_buf(),
        reason: error.to_string(),
    };
    // Sorted, so the first unreadable directory is the same on every machine.
    let mut others = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(unreadable)? {
        let path = entry.map_err(unreadable)?.path();
        let name = path.file_name().and_then(std::ffi::OsStr::to_str);
        let other = name.is_some_and(|name| !name.starts_with('.') && name != ENTITY_ITEMS);
        if other && path.is_dir() {
            others.push(path);
        }
    }
    others.sort();
    for other in others {
        if holds_a_yaml_file(&other)? {
            return Ok(());
        }
    }
    Err(PackageError::EntityPackDeclaresNothing {
        dir: dir.to_path_buf(),
    })
}

/// Whether `directory` holds, directly, a file whose extension is `yaml`.
fn holds_a_yaml_file(directory: &Path) -> Result<bool, PackageError> {
    let unreadable = |error: std::io::Error| PackageError::Unreadable {
        path: directory.to_path_buf(),
        reason: error.to_string(),
    };
    for entry in std::fs::read_dir(directory).map_err(unreadable)? {
        let path = entry.map_err(unreadable)?.path();
        let extension = path.extension().and_then(std::ffi::OsStr::to_str);
        if extension == Some(ENTITY_ITEM_EXTENSION) && path.is_file() {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Checks a Presentation Pack's style manifest in `dir`: it parses, with a string `id` and a
/// non-empty `dimension` list.
///
/// # Errors
///
/// [`PackageError::Unreadable`] when it is absent; [`PackageError::Malformed`] otherwise.
pub fn check_style_manifest(dir: &Path) -> Result<(), PackageError> {
    let path = dir.join(STYLE_FILE);
    let style: StyleManifest =
        serde_saphyr::from_str(&read(&path)?).map_err(|e| malformed(&path, e))?;
    if style.id.trim().is_empty() {
        return Err(malformed(&path, "`id` is empty"));
    }
    if style.dimension.is_empty() {
        return Err(malformed(&path, "`dimension` lists no dimension"));
    }
    Ok(())
}
