//! A Presentation Pack's depiction files (`MODULE_SPEC.md` §6.1.2; `DECISIONS.md` `ARC-177`).
//!
//! A 3D Presentation Pack may carry `depictions/<world-key>.yaml`: a world's 3D look as data — where
//! the ground rises, where the water lies, the house shells, the woods — read by the 3D client's
//! generic builders. The server never reads it. It holds no furniture (`ARC-307`) and no rule.
//!
//! This module reads one file and refuses it by a named code ([`DepictionCode`]) when it is wrong.
//! The order of the checks, so one file gets one code:
//!
//! ```text
//! MALFORMED → SCOPE_MISMATCH → UNCITED → OUT_OF_RANGE → POLYGON → REFERENCE → DOOR → ON_FLOOR
//! ```
//!
//! The 3D client's reader (`SliceDepictionFile`) refuses by the same names for what it checks —
//! `MALFORMED`, `SCOPE_MISMATCH`, and `REFERENCE` for the names only the client knows (texture slugs,
//! tree forms and cards) — and trusts a validated pack for the geometry codes.

use std::fmt;
use std::path::{Path, PathBuf};

pub use crate::depiction_schema::*;
use crate::error::PackageError;
use crate::far_field::{self, Dimension, FAR_FIELD_DIR, FAR_FIELD_EXTENSION};

/// The directory of a 3D Presentation Pack that holds its depiction files. Optional.
pub const DEPICTIONS_DIR: &str = "depictions";

/// The extension of a depiction file.
pub const DEPICTION_EXTENSION: &str = "yaml";

/// Every way a depiction file is refused, in the order the checks run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepictionCode {
    /// Not the JSON subset; a missing required key; an unknown key; a wrong type or enum value; a
    /// conditional shape broken (a height both flat and a ramp).
    Malformed,
    /// `scope.world` is not the file's stem.
    ScopeMismatch,
    /// A physical block without a non-empty `source`.
    Uncited,
    /// A value outside its range, or a cap exceeded.
    OutOfRange,
    /// A polygon with fewer than 3 or more than 64 vertices, or not simple; a ramp's lines not
    /// parallel within 1°.
    Polygon,
    /// A name that names nothing in the file, or in the same pack's far field.
    Reference,
    /// An `open` door not at the doorway of a depicted place; `at_doorway` on a building that depicts
    /// nothing.
    Door,
    /// Something this file draws overlapping a place's floor by more than 50 mm, or a shell more than
    /// 50 mm off the footprint of the place it depicts.
    OnFloor,
}

impl DepictionCode {
    /// Every code, in the order the checks run.
    pub const ALL: [Self; 8] = [
        Self::Malformed,
        Self::ScopeMismatch,
        Self::Uncited,
        Self::OutOfRange,
        Self::Polygon,
        Self::Reference,
        Self::Door,
        Self::OnFloor,
    ];

    /// The code as users, logs and tests see it.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Malformed => "DEPICTION_MALFORMED",
            Self::ScopeMismatch => "DEPICTION_SCOPE_MISMATCH",
            Self::Uncited => "DEPICTION_UNCITED",
            Self::OutOfRange => "DEPICTION_OUT_OF_RANGE",
            Self::Polygon => "DEPICTION_POLYGON",
            Self::Reference => "DEPICTION_REFERENCE",
            Self::Door => "DEPICTION_DOOR",
            Self::OnFloor => "DEPICTION_ON_FLOOR",
        }
    }
}

impl fmt::Display for DepictionCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Reads and validates the depiction file at `path`. `waters` are the names of the water bodies of
/// the same world's far field, which `water.far_field_water` must name; `None` skips that one check
/// (a file read outside a pack).
///
/// # Errors
///
/// [`PackageError::Unreadable`] when the file cannot be read; [`PackageError::Depiction`] with the
/// first refusal's code otherwise.
pub fn read(path: &Path, waters: Option<&[String]>) -> Result<Depiction, PackageError> {
    let text = std::fs::read_to_string(path).map_err(|error| PackageError::Unreadable {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })?;
    let stem = path
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default();
    crate::depiction_check::parse(&text, stem, waters).map_err(|(code, detail)| {
        PackageError::Depiction {
            path: path.to_path_buf(),
            code,
            detail,
        }
    })
}

/// The water names of the 3D far field `far_field/<world>.yaml` beside `depictions/` in `dir`, or an
/// empty list when the pack has none for that world.
///
/// # Errors
///
/// The far field's own refusal.
fn waters_of(dir: &Path, world: &str) -> Result<Vec<String>, PackageError> {
    let file = dir
        .join(FAR_FIELD_DIR)
        .join(format!("{world}.{FAR_FIELD_EXTENSION}"));
    if !file.is_file() {
        return Ok(Vec::new());
    }
    let field = far_field::read(&file, &[Dimension::ThreeD])?;
    Ok(field
        .water
        .iter()
        .map(|water| water.name.as_str().to_owned())
        .collect())
}

/// Checks every depiction file of the pack in `dir`; returns their stems in name order. A pack
/// without `depictions/` has none, which is valid.
///
/// # Errors
///
/// [`PackageError::Unreadable`] when the directory cannot be listed; the first file's refusal
/// otherwise.
pub fn check_pack(dir: &Path) -> Result<Vec<String>, PackageError> {
    let depictions = dir.join(DEPICTIONS_DIR);
    if !depictions.is_dir() {
        return Ok(Vec::new());
    }
    let unreadable = |error: std::io::Error| PackageError::Unreadable {
        path: depictions.clone(),
        reason: error.to_string(),
    };
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(&depictions).map_err(unreadable)? {
        let path = entry.map_err(unreadable)?.path();
        let extension = path.extension().and_then(std::ffi::OsStr::to_str);
        if extension == Some(DEPICTION_EXTENSION) && path.is_file() {
            files.push(path);
        }
    }
    files.sort();
    let mut keys = Vec::new();
    for path in files {
        let stem = path
            .file_stem()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or_default()
            .to_owned();
        let waters = waters_of(dir, &stem)?;
        read(&path, Some(waters.as_slice()))?;
        keys.push(stem);
    }
    Ok(keys)
}
