//! A 3D Presentation Pack's place shells (`places/shells.yaml`; S14 16f-b, `docs/decisions/ARC-52.md`;
//! `clients/3d-spike/PRESENTATION.md` §6).
//!
//! A place the server discloses and no depiction draws is drawn plainly from its disclosure: its floor,
//! walls outside the floor with an opening at each doorway, and a box per shell solid. This optional
//! file says how those plain shells look, by the place's tags. It never says where a wall is: that is
//! the server's `place-shape`.
//!
//! ```json
//! {
//!   "format": 1,
//!   "materials": { "plaster_grey": { "colour": [0.62, 0.62, 0.60], "roughness": 0.85 } },
//!   "default": { "wall": { "material": "plaster_grey", "height_m": 2.7 } },
//!   "tags": { "street": { "wall": null } }
//! }
//! ```
//!
//! The order of the checks, which the client's reader keeps when it falls back to its plain grey:
//!
//! ```text
//! SHELLS_MALFORMED → SHELLS_OUT_OF_RANGE → SHELLS_REFERENCE
//! ```

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use serde::{Deserialize, Deserializer};

use crate::error::PackageError;

/// The file, relative to a Presentation Pack's folder.
pub const SHELLS_FILE: &str = "places/shells.yaml";

/// Every way a shells file is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellsCode {
    /// Not parseable, an unknown key, a wrong type, or a format other than 1.
    Malformed,
    /// A number outside its range.
    OutOfRange,
    /// A material that the file's `materials` table does not declare.
    Reference,
}

impl ShellsCode {
    /// Every code, in the order the checks run.
    pub const ALL: [Self; 3] = [Self::Malformed, Self::OutOfRange, Self::Reference];

    /// The code as users, logs and tests see it.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Malformed => "SHELLS_MALFORMED",
            Self::OutOfRange => "SHELLS_OUT_OF_RANGE",
            Self::Reference => "SHELLS_REFERENCE",
        }
    }
}

impl fmt::Display for ShellsCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A shells file as written.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shells {
    /// Always 1.
    pub format: u32,
    /// The materials the styles name, by slug.
    #[serde(default)]
    pub materials: BTreeMap<String, Material>,
    /// The style of a place no tag binds.
    #[serde(default)]
    pub default: Style,
    /// Tag → the parts of the default it changes.
    #[serde(default)]
    pub tags: BTreeMap<String, Style>,
}

/// One material: a flat colour, or a texture of the client's library tinted by it.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Material {
    /// Linear red, green, blue, each 0 to 2 (a tint above 1 brightens a texture).
    pub colour: [f64; 3],
    /// 0 to 1.
    #[serde(default)]
    pub roughness: Option<f64>,
    /// A texture slug of the client's library; absent draws the flat colour.
    #[serde(default)]
    pub texture: Option<String>,
    /// How many metres one repeat of the texture covers, above 0 and at most 100.
    #[serde(default)]
    pub metres: Option<f64>,
}

/// The parts of a shell. In `default`, an absent part is the client's plain grey. In a tag's style,
/// an absent part keeps the default's, and `"wall": null` draws no walls.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Style {
    /// The walls; `Some(None)` is `null`: no walls.
    #[serde(default, deserialize_with = "present")]
    pub wall: Option<Option<Wall>>,
    /// The floor.
    #[serde(default)]
    pub floor: Option<Surface>,
    /// The drawn size of a doorway's opening.
    #[serde(default)]
    pub opening: Option<Opening>,
    /// The shell solids (a disclosed solid that names no furnishing).
    #[serde(default)]
    pub solid: Option<Surface>,
}

/// The walls of a shell.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wall {
    /// A material slug of `materials`.
    #[serde(default)]
    pub material: Option<String>,
    /// The drawn height, above 0 and at most 20 m.
    #[serde(default)]
    pub height_m: Option<f64>,
}

/// A floor or a solid.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Surface {
    /// A material slug of `materials`.
    #[serde(default)]
    pub material: Option<String>,
}

/// A doorway's drawn opening: a drawing size, never a rule (the server decides who passes).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Opening {
    /// Above 0 and at most 10 m.
    #[serde(default)]
    pub width_m: Option<f64>,
    /// Above 0 and at most 10 m.
    #[serde(default)]
    pub height_m: Option<f64>,
}

/// A field that is present, even as `null`, is `Some`; an absent one is `None` (by `default`).
fn present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

/// A refusal's code and detail, before the file's path is attached.
type Refusal = (ShellsCode, String);

/// Parses and validates shells `text`.
///
/// # Errors
///
/// The first refusal, by its code, with a detail naming the field.
pub fn parse(text: &str) -> Result<Shells, (ShellsCode, String)> {
    let shells: Shells =
        serde_saphyr::from_str(text).map_err(|error| (ShellsCode::Malformed, error.to_string()))?;
    if shells.format != 1 {
        return Err((
            ShellsCode::Malformed,
            format!("format: {} is not 1", shells.format),
        ));
    }
    check_ranges(&shells)?;
    check_references(&shells)?;
    Ok(shells)
}

/// Reads and validates the shells file at `path`.
///
/// # Errors
///
/// [`PackageError::Unreadable`] when the file cannot be read; [`PackageError::Shells`] with the first
/// refusal's code otherwise.
pub fn read(path: &Path) -> Result<Shells, PackageError> {
    let text = std::fs::read_to_string(path).map_err(|error| PackageError::Unreadable {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })?;
    parse(&text).map_err(|(code, detail)| PackageError::Shells {
        path: path.to_path_buf(),
        code,
        detail,
    })
}

/// Checks a Presentation Pack's `places/shells.yaml` when it has one; a pack without it is valid (its
/// shells are the client's plain grey).
///
/// # Errors
///
/// As [`read`].
pub fn check_pack(pack: &Path) -> Result<(), PackageError> {
    let path = pack.join(SHELLS_FILE);
    if !path.is_file() {
        return Ok(());
    }
    read(&path).map(|_| ())
}

fn within(field: &str, value: f64, low_exclusive: f64, high: f64) -> Result<(), Refusal> {
    if value.is_finite() && value > low_exclusive && value <= high {
        return Ok(());
    }
    Err((
        ShellsCode::OutOfRange,
        format!("{field}: {value} is outside ({low_exclusive}, {high}]"),
    ))
}

fn check_ranges(shells: &Shells) -> Result<(), Refusal> {
    for (slug, material) in &shells.materials {
        for (i, channel) in material.colour.iter().enumerate() {
            if !(channel.is_finite() && (0.0..=2.0).contains(channel)) {
                return Err((
                    ShellsCode::OutOfRange,
                    format!("materials.{slug}.colour[{i}]: {channel} is outside [0, 2]"),
                ));
            }
        }
        if let Some(roughness) = material.roughness
            && !(roughness.is_finite() && (0.0..=1.0).contains(&roughness))
        {
            return Err((
                ShellsCode::OutOfRange,
                format!("materials.{slug}.roughness: {roughness} is outside [0, 1]"),
            ));
        }
        if let Some(metres) = material.metres {
            within(&format!("materials.{slug}.metres"), metres, 0.0, 100.0)?;
        }
    }
    for (name, style) in styles(shells) {
        if let Some(Some(wall)) = &style.wall
            && let Some(height) = wall.height_m
        {
            within(&format!("{name}.wall.height_m"), height, 0.0, 20.0)?;
        }
        if let Some(opening) = &style.opening {
            if let Some(width) = opening.width_m {
                within(&format!("{name}.opening.width_m"), width, 0.0, 10.0)?;
            }
            if let Some(height) = opening.height_m {
                within(&format!("{name}.opening.height_m"), height, 0.0, 10.0)?;
            }
        }
    }
    Ok(())
}

fn check_references(shells: &Shells) -> Result<(), Refusal> {
    for (name, style) in styles(shells) {
        let named = [
            (
                "wall",
                style
                    .wall
                    .as_ref()
                    .and_then(|w| w.as_ref())
                    .and_then(|w| w.material.as_ref()),
            ),
            (
                "floor",
                style.floor.as_ref().and_then(|s| s.material.as_ref()),
            ),
            (
                "solid",
                style.solid.as_ref().and_then(|s| s.material.as_ref()),
            ),
        ];
        for (part, material) in named {
            if let Some(slug) = material
                && !shells.materials.contains_key(slug)
            {
                return Err((
                    ShellsCode::Reference,
                    format!("{name}.{part}.material: '{slug}' is not declared under materials"),
                ));
            }
        }
    }
    Ok(())
}

/// The default style and every tag's, each with the name a detail gives it.
fn styles(shells: &Shells) -> Vec<(String, &Style)> {
    let mut out = vec![("default".to_owned(), &shells.default)];
    for (tag, style) in &shells.tags {
        out.push((format!("tags.{tag}"), style));
    }
    out
}
