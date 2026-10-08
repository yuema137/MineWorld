//! `pack.yaml`: the package identity of a data pack that has no other carrier (`ARC-53`,
//! `PACKAGE_FORMAT.md` §5.0), and the one content check E-a makes of a Presentation Pack.

use std::path::Path;

use serde::Deserialize;

use crate::error::PackageError;
use crate::identity::{Compatibility, Identity, License, PackId, PackType, Version};

/// The file a data pack states its identity in.
pub const PACK_FILE: &str = "pack.yaml";

/// A Presentation Pack's style manifest (`ART_DIRECTION.md` §12) — not a package manifest.
pub const STYLE_FILE: &str = "manifest.yaml";

/// `pack.yaml`, exactly: any other field is refused, `dependencies` included until a world's
/// requirements are resolved, so no field is accepted that nothing checks.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PackFile {
    id: PackId,
    #[serde(rename = "type")]
    kind: PackType,
    version: Version,
    mineworld: Compatibility,
    license: License,
    authors: Vec<String>,
    #[serde(default)]
    repository: Option<String>,
}

/// What `mineworld packs` needs of a style manifest; the rest of its schema is `ART_DIRECTION.md`'s,
/// so other fields are not this crate's to refuse.
#[derive(Debug, Deserialize)]
struct StyleManifest {
    id: String,
    dimension: Vec<String>,
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
/// empty author list; [`PackageError::NotCarriedHere`] for a type this file does not identify.
pub fn read_pack_file(dir: &Path) -> Result<Identity, PackageError> {
    let path = dir.join(PACK_FILE);
    let file: PackFile = serde_saphyr::from_str(&read(&path)?).map_err(|e| malformed(&path, e))?;
    let not_here = |why| PackageError::NotCarriedHere {
        path: path.clone(),
        kind: file.kind.to_string(),
        why,
    };
    match file.kind {
        PackType::PresentationPack => {}
        PackType::EntityPack => return Err(not_here("Entity Packs are read from S16's PR E-d")),
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
