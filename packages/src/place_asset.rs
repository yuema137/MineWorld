//! A 3D Presentation Pack's place art (`assets/places/<asset-id>/`; S14 16f-d1; `PACKAGE_FORMAT.md` §3,
//! `clients/3d-spike/PRESENTATION.md` §8).
//!
//! A place asset is a glTF 2.0 binary (`.glb`) and the `asset.yaml` that says what it means: which of
//! its nodes are entrances (a drawn door's street-side point) and which are spaces (a room's volume),
//! under which licence, and from which library files. A depiction file places assets by their id
//! (its `models` section, `ARC-177`); the server never reads either.
//!
//! What glTF cannot carry (a world-triplanar material, a Label3D sign, a body, a library prop) is a
//! recipe in the pack's `materials/material_rules.yaml`, named by content: a glTF material `m_<hash>`,
//! a node `<kind>_<hash>_<n>`. The 3D client re-binds by those names.
//!
//! The order of the checks, so one asset gets one code:
//!
//! ```text
//! ASSET_MALFORMED → ASSET_UNLICENSED → ASSET_REFERENCE
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::PackageError;
use crate::far_field_schema::{AssetName, Key};

/// The directory of a 3D Presentation Pack that holds its place assets. Optional.
pub const PLACE_ASSETS_DIR: &str = "assets/places";

/// Each place asset's contract file.
pub const ASSET_FILE: &str = "asset.yaml";

/// The pack's recipes for what glTF cannot carry, relative to the pack.
pub const MATERIAL_RULES_FILE: &str = "materials/material_rules.yaml";

/// The only asset type a place asset has.
const PLACE_VISUAL: &str = "place_visual";

/// How many hex digits a recipe's name carries.
const HASH_DIGITS: usize = 12;

/// The node kinds that carry a recipe.
const RECIPE_KINDS: [&str; 3] = ["mesh", "light", "node"];

/// Every way a place asset is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetCode {
    /// Not parseable, an unknown key, a wrong type, a type other than `place_visual`, an id other than
    /// its folder's name, a unit other than metres; a model that is not glTF 2.0 binary; a rules file
    /// that is not format 1.
    Malformed,
    /// No licence, or a licence or a source's licence that is not an SPDX expression.
    Unlicensed,
    /// A name that names nothing: the model file; an entrance or space the model has no node for; a
    /// door's entrance or space the bindings do not list; a material or node recipe the rules lack.
    Reference,
}

impl AssetCode {
    /// Every code, in the order the checks run.
    pub const ALL: [Self; 3] = [Self::Malformed, Self::Unlicensed, Self::Reference];

    /// The code as users, logs and tests see it.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Malformed => "ASSET_MALFORMED",
            Self::Unlicensed => "ASSET_UNLICENSED",
            Self::Reference => "ASSET_REFERENCE",
        }
    }
}

impl fmt::Display for AssetCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One `asset.yaml` (`PACKAGE_FORMAT.md` §3; YAML's JSON subset when written by the export tool).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlaceAsset {
    /// The asset's id: its folder's name.
    pub id: Key,
    /// Always `place_visual`.
    #[serde(rename = "type")]
    pub kind: String,
    /// The files.
    pub format: AssetFormat,
    /// What the model's named nodes mean.
    pub semantic_bindings: SemanticBindings,
    /// Each entrance's door: what the client's door registry needs to bind a place to it.
    #[serde(default)]
    pub doors: Vec<AssetDoor>,
    /// The style families it belongs to.
    #[serde(default)]
    pub presentation: Option<AssetPresentation>,
    /// The model's unit.
    pub scale: AssetScale,
    /// Its licence, and the licence of every library file it references.
    pub license: AssetLicense,
    /// How it was made.
    #[serde(default)]
    pub source: Option<AssetProvenance>,
}

/// The files of an asset.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetFormat {
    /// The `.glb`, beside `asset.yaml`.
    pub model: String,
    /// The pack's rules file, relative to the pack, when the model names recipes.
    #[serde(default)]
    pub rules: Option<String>,
}

/// The model's named nodes.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticBindings {
    /// Nodes at a drawn door's street-side point, facing out.
    #[serde(default)]
    pub entrances: Vec<AssetName>,
    /// Where a system's activity is drawn (never whether it is allowed, `ENGINEERING_RULES.md` §19).
    #[serde(default)]
    pub interaction_anchors: Vec<AssetName>,
    /// Nodes at a room's centre whose scale is the room's size.
    #[serde(default)]
    pub spaces: Vec<AssetName>,
}

/// One entrance's door.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetDoor {
    /// The entrance.
    pub name: AssetName,
    /// The door's label.
    pub label: String,
    /// The space behind it, or `null` for a closed door.
    #[serde(default)]
    pub space: Option<AssetName>,
    /// From the sill to the room's inner face, metres.
    pub depth_m: f64,
    /// The room floor's height above the sill, metres.
    pub floor_m: f64,
}

/// The style families.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetPresentation {
    /// Style ids.
    pub style_family: Vec<String>,
}

/// The model's unit.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetScale {
    /// Always `meter`.
    pub unit: String,
}

/// An asset's licence.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetLicense {
    /// The licence of what the asset's author made, an SPDX expression.
    pub spdx: String,
    /// The library files it references, grouped by licence.
    #[serde(default)]
    pub sources: Vec<AssetSource>,
}

/// Library files under one licence.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetSource {
    /// What they are.
    pub what: String,
    /// Their licence, an SPDX expression.
    pub spdx: String,
    /// Where the licence is stated.
    pub url: String,
    /// The files, relative to the client's project.
    pub files: Vec<String>,
}

/// How the asset was made.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetProvenance {
    /// The tool that wrote it.
    pub tool: String,
    /// The authoring source the tool built it from.
    pub authoring: String,
}

/// A pack's place assets: each id and its entrances, for the depiction files' `models` (`ARC-177`).
#[derive(Debug, Clone, Default)]
pub struct PlaceAssets {
    entrances: BTreeMap<String, Vec<String>>,
}

impl PlaceAssets {
    /// Whether the pack has the asset.
    #[must_use]
    pub fn has(&self, id: &str) -> bool {
        self.entrances.contains_key(id)
    }

    /// The asset's entrances, or `None` without the asset.
    #[must_use]
    pub fn entrances(&self, id: &str) -> Option<&[String]> {
        self.entrances.get(id).map(Vec::as_slice)
    }

    /// The ids, in name order.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.entrances.keys().map(String::as_str)
    }
}

/// The rules file, as far as this validator reads it: the recipes' names. A recipe's body is an engine
/// object's settings, which only the client that re-binds it reads.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Rules {
    format: u32,
    source: String,
    materials: BTreeMap<String, serde::de::IgnoredAny>,
    nodes: BTreeMap<String, serde::de::IgnoredAny>,
}

/// The parts of a glTF document this validator reads.
#[derive(Debug, Deserialize)]
struct Gltf {
    asset: GltfAsset,
    #[serde(default)]
    nodes: Vec<Named>,
    #[serde(default)]
    materials: Vec<Named>,
}

#[derive(Debug, Deserialize)]
struct GltfAsset {
    version: String,
}

#[derive(Debug, Deserialize)]
struct Named {
    #[serde(default)]
    name: Option<String>,
}

/// A refusal's code and detail, before the file's path is attached.
type Refusal = (AssetCode, String);

/// Reads and validates the place asset in `dir` (`assets/places/<id>/`) of the pack at `pack`.
///
/// # Errors
///
/// [`PackageError::PlaceAsset`] with the first refusal's code.
pub fn read(dir: &Path, pack: &Path) -> Result<PlaceAsset, PackageError> {
    let file = dir.join(ASSET_FILE);
    let refused = |path: &Path, (code, detail): Refusal| PackageError::PlaceAsset {
        path: path.to_path_buf(),
        code,
        detail,
    };
    let text = std::fs::read_to_string(&file).map_err(|error| {
        refused(
            &file,
            (
                AssetCode::Malformed,
                format!("cannot read {ASSET_FILE}: {error}"),
            ),
        )
    })?;
    let asset: PlaceAsset = serde_saphyr::from_str(&text)
        .map_err(|error| refused(&file, (AssetCode::Malformed, error.to_string())))?;
    let folder = dir
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default();
    check_shape(&asset, folder).map_err(|r| refused(&file, r))?;
    check_licence(&asset).map_err(|r| refused(&file, r))?;
    let model = dir.join(&asset.format.model);
    let gltf = read_glb(&model).map_err(|r| refused(&model, r))?;
    check_bindings(&asset, &gltf).map_err(|r| refused(&file, r))?;
    check_recipes(&asset, &gltf, pack).map_err(|(path, r)| refused(&path, r))?;
    Ok(asset)
}

/// Checks every place asset of the pack in `pack`; returns their ids and entrances. A pack without
/// `assets/places/` has none, which is valid.
///
/// # Errors
///
/// [`PackageError::Unreadable`] when the directory cannot be listed; the first asset's refusal
/// otherwise.
pub fn check_pack(pack: &Path) -> Result<PlaceAssets, PackageError> {
    let root = pack.join(PLACE_ASSETS_DIR);
    let mut out = PlaceAssets::default();
    if !root.is_dir() {
        return Ok(out);
    }
    let unreadable = |error: std::io::Error| PackageError::Unreadable {
        path: root.clone(),
        reason: error.to_string(),
    };
    let mut dirs: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(&root).map_err(unreadable)? {
        let path = entry.map_err(unreadable)?.path();
        if path.is_dir() {
            dirs.push(path);
        }
    }
    dirs.sort();
    for dir in dirs {
        let asset = read(&dir, pack)?;
        let entrances = asset
            .semantic_bindings
            .entrances
            .iter()
            .map(|e| e.as_str().to_owned())
            .collect();
        out.entrances
            .insert(asset.id.as_str().to_owned(), entrances);
    }
    Ok(out)
}

fn malformed<T>(detail: impl Into<String>) -> Result<T, Refusal> {
    Err((AssetCode::Malformed, detail.into()))
}

fn check_shape(asset: &PlaceAsset, folder: &str) -> Result<(), Refusal> {
    if asset.id.as_str() != folder {
        return malformed(format!(
            "id {} is not its folder's name {folder}",
            asset.id.as_str()
        ));
    }
    if asset.kind != PLACE_VISUAL {
        return malformed(format!("type {} is not {PLACE_VISUAL}", asset.kind));
    }
    if asset.scale.unit != "meter" {
        return malformed(format!("scale.unit {} is not meter", asset.scale.unit));
    }
    let model = &asset.format.model;
    if model.contains('/') || model.contains('\\') || !model.ends_with(".glb") {
        return malformed(format!(
            "format.model {model} is not a .glb file name beside {ASSET_FILE}"
        ));
    }
    Ok(())
}

fn check_licence(asset: &PlaceAsset) -> Result<(), Refusal> {
    let spdx_ok = |expression: &str| {
        !expression.trim().is_empty() && spdx::Expression::parse(expression).is_ok()
    };
    if !spdx_ok(&asset.license.spdx) {
        return Err((
            AssetCode::Unlicensed,
            format!(
                "license.spdx \"{}\" is not an SPDX expression: an asset states its licence",
                asset.license.spdx
            ),
        ));
    }
    for (i, source) in asset.license.sources.iter().enumerate() {
        if !spdx_ok(&source.spdx) {
            return Err((
                AssetCode::Unlicensed,
                format!(
                    "license.sources[{i}].spdx \"{}\" is not an SPDX expression",
                    source.spdx
                ),
            ));
        }
    }
    Ok(())
}

/// The JSON chunk of a glTF 2.0 binary (`.glb`: a 12-byte header, then the JSON chunk).
fn read_glb(path: &Path) -> Result<Gltf, Refusal> {
    let bytes = std::fs::read(path).map_err(|error| {
        (
            AssetCode::Reference,
            format!("the model cannot be read: {error}"),
        )
    })?;
    let word = |at: usize| {
        bytes
            .get(at..at + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    };
    let (Some(magic), Some(version), Some(length), Some(kind)) =
        (word(0), word(4), word(12), word(16))
    else {
        return malformed("not a glTF binary: shorter than its header");
    };
    // "glTF" and "JSON", little-endian.
    if magic != 0x4654_6C67 || version != 2 || kind != 0x4E4F_534A {
        return malformed("not a glTF 2.0 binary with a JSON chunk first");
    }
    let json = usize::try_from(length)
        .ok()
        .and_then(|n| bytes.get(20..20 + n))
        .ok_or((
            AssetCode::Malformed,
            "the JSON chunk runs past the file".to_owned(),
        ))?;
    let gltf: Gltf = serde_json::from_slice(json)
        .map_err(|error| (AssetCode::Malformed, format!("the JSON chunk: {error}")))?;
    if gltf.asset.version != "2.0" {
        return malformed(format!("asset.version {} is not 2.0", gltf.asset.version));
    }
    Ok(gltf)
}

fn check_bindings(asset: &PlaceAsset, gltf: &Gltf) -> Result<(), Refusal> {
    let nodes: BTreeSet<&str> = gltf
        .nodes
        .iter()
        .filter_map(|n| n.name.as_deref())
        .collect();
    let b = &asset.semantic_bindings;
    for (list, names) in [
        ("entrances", &b.entrances),
        ("interaction_anchors", &b.interaction_anchors),
        ("spaces", &b.spaces),
    ] {
        if let Some(name) = names.iter().find(|n| !nodes.contains(n.as_str())) {
            return Err((
                AssetCode::Reference,
                format!(
                    "semantic_bindings.{list}: the model {} has no node named {}",
                    asset.format.model,
                    name.as_str()
                ),
            ));
        }
    }
    let listed = |names: &[AssetName], name: &AssetName| names.iter().any(|n| n == name);
    for (i, door) in asset.doors.iter().enumerate() {
        if !listed(&b.entrances, &door.name) {
            return Err((
                AssetCode::Reference,
                format!(
                    "doors[{i}].name: {} is not among semantic_bindings.entrances",
                    door.name.as_str()
                ),
            ));
        }
        if let Some(space) = &door.space
            && !listed(&b.spaces, space)
        {
            return Err((
                AssetCode::Reference,
                format!(
                    "doors[{i}].space: {} is not among semantic_bindings.spaces",
                    space.as_str()
                ),
            ));
        }
    }
    Ok(())
}

/// A recipe name's hash: `m_<hash>` for a material, `<kind>_<hash>_<n>` for a node; `None` for a name
/// that is not a recipe's.
fn recipe_hash(name: &str, material: bool) -> Option<&str> {
    let hex = |h: &str| h.len() == HASH_DIGITS && h.bytes().all(|b| b.is_ascii_hexdigit());
    if material {
        return name.strip_prefix("m_").filter(|h| hex(h));
    }
    let mut parts = name.splitn(3, '_');
    let (Some(kind), Some(hash), Some(n)) = (parts.next(), parts.next(), parts.next()) else {
        return None;
    };
    (RECIPE_KINDS.contains(&kind) && hex(hash) && n.bytes().all(|b| b.is_ascii_digit()))
        .then_some(hash)
}

fn check_recipes(asset: &PlaceAsset, gltf: &Gltf, pack: &Path) -> Result<(), (PathBuf, Refusal)> {
    let materials: Vec<&str> = gltf
        .materials
        .iter()
        .filter_map(|m| m.name.as_deref())
        .filter_map(|n| recipe_hash(n, true).map(|_| n))
        .collect();
    let nodes: Vec<(&str, &str)> = gltf
        .nodes
        .iter()
        .filter_map(|n| n.name.as_deref())
        .filter_map(|n| recipe_hash(n, false).map(|h| (n, h)))
        .collect();
    if materials.is_empty() && nodes.is_empty() {
        return Ok(());
    }
    let file = pack.join(MATERIAL_RULES_FILE);
    let at_asset = pack
        .join(PLACE_ASSETS_DIR)
        .join(asset.id.as_str())
        .join(ASSET_FILE);
    let Some(rules_path) = &asset.format.rules else {
        return Err((
            at_asset,
            (
                AssetCode::Reference,
                format!(
                    "the model names recipes, and format.rules does not name {MATERIAL_RULES_FILE}"
                ),
            ),
        ));
    };
    if rules_path != MATERIAL_RULES_FILE {
        return Err((
            at_asset,
            (
                AssetCode::Reference,
                format!("format.rules {rules_path} is not the pack's {MATERIAL_RULES_FILE}"),
            ),
        ));
    }
    let text = std::fs::read_to_string(&file).map_err(|error| {
        (
            file.clone(),
            (
                AssetCode::Reference,
                format!("cannot read the rules: {error}"),
            ),
        )
    })?;
    let rules: Rules = serde_json::from_str(&text)
        .map_err(|error| (file.clone(), (AssetCode::Malformed, error.to_string())))?;
    if rules.format != 1 || rules.source.is_empty() {
        return Err((
            file,
            (
                AssetCode::Malformed,
                format!("format {} is not 1, or the source is empty", rules.format),
            ),
        ));
    }
    if let Some(name) = materials
        .iter()
        .find(|n| !rules.materials.contains_key(**n))
    {
        return Err((
            at_asset,
            (
                AssetCode::Reference,
                format!("the model's material {name} has no recipe in {MATERIAL_RULES_FILE}"),
            ),
        ));
    }
    if let Some((name, _)) = nodes.iter().find(|(_, h)| !rules.nodes.contains_key(*h)) {
        return Err((
            at_asset,
            (
                AssetCode::Reference,
                format!("the model's node {name} has no recipe in {MATERIAL_RULES_FILE}"),
            ),
        ));
    }
    Ok(())
}
