//! A Presentation Pack's far field (`MODULE_SPEC.md` §6.1.1; `DECISIONS.md` `ARC-102`).
//!
//! However small the world, every presented view closes on a far field. A Presentation Pack carries
//! `far_field/<world-key>.yaml` for every world it presents; this module reads one, refuses it by a
//! named code ([`FarFieldCode`]) when it is wrong, and checks that a pack carries the file a world
//! needs. The clients' shared GDScript module (`clients/shared/environment/far_field.gd`) applies the
//! same rules in the same order; a cross-check runs both over one corpus of planted-bad files.
//!
//! The order of the checks, which both validators keep so one file gets one code:
//!
//! ```text
//! MALFORMED (or SLOT / SLOT_TOWN, during parsing) → SCOPE_MISMATCH → SECTION_MISSING → SECTION_FOREIGN
//! → UNCITED → OUT_OF_RANGE → INSIDE_BOUNDARY → PROFILE → GAP
//! ```

use std::fmt;
use std::path::{Path, PathBuf};

// The types live in `far_field_schema.rs` and the rules in `far_field_check.rs`, siblings of this
// file (the crate's sources stay one flat directory, which `tests/structure.rs` reads).
pub use crate::far_field_schema::{
    AssetName, Beyond, Boundary, BoundaryKind, Colour, CoursePoint, FarField, Foreground, Haze,
    HeightRange, Horizon, Key, Layer, MassingSector, ProceduralSky, ProfilePoint, Scope, Sector,
    Silhouette, Sky, SkyKind, Skyline, Slot, SlotName, Slots, Surface, Surround, SurroundBase,
    Water, WaterColour, WaterKind,
};

use crate::error::PackageError;

/// The directory of a Presentation Pack that holds its far fields.
pub const FAR_FIELD_DIR: &str = "far_field";

/// The extension of a far-field file.
pub const FAR_FIELD_EXTENSION: &str = "yaml";

/// The stem prefix of a region's far-field file (`region-<region-key>.yaml`).
pub const REGION_PREFIX: &str = "region-";

/// The step at which sector coverage is checked, in degrees (`FAR_FIELD_GAP` names a gap to 0.1°).
const STEP_TENTHS: u32 = 3600;

/// Marks a slot refusal inside the parser's message, so it is reported as `FAR_FIELD_SLOT` rather
/// than `FAR_FIELD_MALFORMED`.
pub(crate) const SLOT_MARK: &str = "far-field slot: ";

/// Marks a town-slot refusal inside the parser's message (`FAR_FIELD_SLOT_TOWN`). It does not contain
/// [`SLOT_MARK`], so neither mark can be mistaken for the other.
pub(crate) const SLOT_TOWN_MARK: &str = "far-field town slot: ";

/// Every way a far field is refused. One list, in this order, shared with the clients' GDScript
/// module (`MineWorldFarField.CODES`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FarFieldCode {
    /// The pack has no far field for a presented world, or no `far_field/` at all.
    Missing,
    /// A client was started without a world key it can resolve (client-only).
    WorldUnknown,
    /// Not parseable, an unknown key, a wrong type or enum value.
    Malformed,
    /// `scope` disagrees with the file name.
    ScopeMismatch,
    /// A section the pack's dimension requires is absent.
    SectionMissing,
    /// A section of another dimension is present.
    SectionForeign,
    /// The sectors leave part of the circle open (3D), or the surround has no extent (2D).
    Gap,
    /// A far-field distance lies at or inside `boundary.reach_m`.
    InsideBoundary,
    /// Profile points out of their sector, not increasing along it, or not at its edges.
    Profile,
    /// A number outside its range.
    OutOfRange,
    /// A citation is empty.
    Uncited,
    /// A slot key other than `station`/`landmark`, a missing one, or an unknown state; or (validate) a
    /// filled slot in a world that is not a town.
    Slot,
    /// A town's slots disagree with the town: exactly one slot filled (file); or, against the world's
    /// `settlement:`, a slot not filled or naming another place (validate).
    SlotTown,
}

impl FarFieldCode {
    /// Every code, in the shared order.
    pub const ALL: [Self; 13] = [
        Self::Missing,
        Self::WorldUnknown,
        Self::Malformed,
        Self::ScopeMismatch,
        Self::SectionMissing,
        Self::SectionForeign,
        Self::Gap,
        Self::InsideBoundary,
        Self::Profile,
        Self::OutOfRange,
        Self::Uncited,
        Self::Slot,
        Self::SlotTown,
    ];

    /// The code as users, logs and tests see it.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "FAR_FIELD_MISSING",
            Self::WorldUnknown => "FAR_FIELD_WORLD_UNKNOWN",
            Self::Malformed => "FAR_FIELD_MALFORMED",
            Self::ScopeMismatch => "FAR_FIELD_SCOPE_MISMATCH",
            Self::SectionMissing => "FAR_FIELD_SECTION_MISSING",
            Self::SectionForeign => "FAR_FIELD_SECTION_FOREIGN",
            Self::Gap => "FAR_FIELD_GAP",
            Self::InsideBoundary => "FAR_FIELD_INSIDE_BOUNDARY",
            Self::Profile => "FAR_FIELD_PROFILE",
            Self::OutOfRange => "FAR_FIELD_OUT_OF_RANGE",
            Self::Uncited => "FAR_FIELD_UNCITED",
            Self::Slot => "FAR_FIELD_SLOT",
            Self::SlotTown => "FAR_FIELD_SLOT_TOWN",
        }
    }
}

impl fmt::Display for FarFieldCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The dimension a Presentation Pack presents, from its style manifest's `dimension` list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dimension {
    /// `2d`: the surround is required.
    TwoD,
    /// `3d`: datum, sky, foreground, silhouettes and skyline are required.
    ThreeD,
}

impl Dimension {
    /// The dimension a style manifest's entry names, if it names one.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "2d" => Some(Self::TwoD),
            "3d" => Some(Self::ThreeD),
            _ => None,
        }
    }
}
/// Reads and validates the far field at `path` for a pack presenting `dimensions`.
///
/// # Errors
///
/// [`PackageError::Unreadable`] when the file cannot be read; [`PackageError::FarField`] with the
/// first refusal's code otherwise.
pub fn read(path: &Path, dimensions: &[Dimension]) -> Result<FarField, PackageError> {
    let text = std::fs::read_to_string(path).map_err(|error| PackageError::Unreadable {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })?;
    let stem = path
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default();
    crate::far_field_check::parse(&text, stem, dimensions).map_err(|(code, detail)| {
        PackageError::FarField {
            path: path.to_path_buf(),
            code,
            detail,
        }
    })
}

/// The first maximal run of bearings no sector covers, checked at every 0.1° from 0.0 to 359.9, as
/// `(from, to)`; a run through north starts before 360 and ends after 0. `None` when the circle is
/// closed.
#[must_use]
pub fn uncovered(sectors: &[Sector]) -> Option<(f64, f64)> {
    let open = |tenth: u32| {
        let bearing = f64::from(tenth % STEP_TENTHS) / 10.0;
        !sectors.iter().any(|sector| sector.contains(bearing))
    };
    let first = (0..STEP_TENTHS).find(|tenth| open(*tenth))?;
    let run = (first..STEP_TENTHS)
        .take_while(|tenth| open(*tenth))
        .count();
    let end = first + u32::try_from(run).unwrap_or(STEP_TENTHS);
    if first == 0 && end == STEP_TENTHS {
        return Some((0.0, 360.0));
    }
    // A run that starts at 0 may have begun before north: extend it backward.
    let start = if first == 0 {
        let behind = (0..STEP_TENTHS)
            .rev()
            .take_while(|tenth| open(*tenth))
            .count();
        (STEP_TENTHS - u32::try_from(behind).unwrap_or(0)) % STEP_TENTHS
    } else {
        first
    };
    Some((f64::from(start) / 10.0, f64::from(end) / 10.0))
}

/// Checks every far-field file of the pack in `dir`, presenting `dimensions`; returns their stems
/// in name order.
///
/// # Errors
///
/// [`PackageError::FarFieldMissing`] when the pack has no `far_field/` directory or it holds no
/// file; [`PackageError::Unreadable`] when it cannot be listed; the first file's refusal otherwise.
pub fn check_pack(dir: &Path, dimensions: &[Dimension]) -> Result<Vec<String>, PackageError> {
    let fields = dir.join(FAR_FIELD_DIR);
    let missing = || PackageError::FarFieldMissing {
        pack: dir.display().to_string(),
        scope: NO_DIRECTORY.to_owned(),
    };
    if !fields.is_dir() {
        return Err(missing());
    }
    let unreadable = |error: std::io::Error| PackageError::Unreadable {
        path: fields.clone(),
        reason: error.to_string(),
    };
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(&fields).map_err(unreadable)? {
        let path = entry.map_err(unreadable)?.path();
        let extension = path.extension().and_then(std::ffi::OsStr::to_str);
        if extension == Some(FAR_FIELD_EXTENSION) && path.is_file() {
            files.push(path);
        }
    }
    if files.is_empty() {
        return Err(missing());
    }
    files.sort();
    let mut keys = Vec::new();
    for path in files {
        read(&path, dimensions)?;
        if let Some(stem) = path.file_stem().and_then(std::ffi::OsStr::to_str) {
            keys.push(stem.to_owned());
        }
    }
    Ok(keys)
}

/// The `scope` a [`PackageError::FarFieldMissing`] names when the pack has no `far_field/` at all.
pub const NO_DIRECTORY: &str = "no far_field/";

/// Checks that the Presentation Pack `pack` in `dir` carries `far_field/<world>.yaml`. Only the
/// file's presence: what it says is `mineworld packs validate`'s and the client's to check.
///
/// # Errors
///
/// [`PackageError::FarFieldMissing`] naming the pack and the world.
pub fn require_for_world(dir: &Path, pack: &str, world: &str) -> Result<(), PackageError> {
    let file = dir
        .join(FAR_FIELD_DIR)
        .join(format!("{world}.{FAR_FIELD_EXTENSION}"));
    if file.is_file() {
        Ok(())
    } else {
        Err(PackageError::FarFieldMissing {
            pack: pack.to_owned(),
            scope: world.to_owned(),
        })
    }
}

/// The message of a [`PackageError::FarFieldMissing`].
pub(crate) fn missing_message(pack: &str, scope: &str) -> String {
    if scope == NO_DIRECTORY {
        format!("FAR_FIELD_MISSING: the Presentation Pack {pack} has {NO_DIRECTORY} directory")
    } else {
        format!(
            "FAR_FIELD_MISSING: the Presentation Pack {pack} has no far field for the world {scope} \
             ({FAR_FIELD_DIR}/{scope}.{FAR_FIELD_EXTENSION})"
        )
    }
}
