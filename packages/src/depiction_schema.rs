//! The depiction file's types (`MODULE_SPEC.md` §6.1.2, format 1; `ARC-177`): each a named type, no
//! maps, no untyped values. Unknown keys are refused everywhere, so a field the format does not have
//! (a table, a bench, a door's passability) cannot be written (`ARC-307`).
//!
//! Plan coordinates are the street frame's integer millimetres (`[x_mm, y_mm]`, x east, y north);
//! heights are metres above the street's surface. The conditional shapes serde cannot express (a
//! height that is a flat or a ramp, an area that is a ring or a polygon) are checked by
//! `depiction_check.rs` as `DEPICTION_MALFORMED`.

use serde::Deserialize;

pub use crate::far_field_schema::{AssetName, Colour, Key};

/// A plan point, millimetres.
pub type PointMm = [i64; 2];

/// A plan rectangle, millimetres: `[x, y, width, height]`, (x, y) its south-west corner.
pub type RectMm = [i64; 4];

/// One depiction file (`MODULE_SPEC.md` §6.1.2, format 1).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Depiction {
    /// Always 1.
    pub format: u32,
    /// The world this file depicts; it equals the file's stem.
    pub scope: DepictionScope,
    /// Where plan (0, 0) stands in the scene.
    pub frame: Frame,
    /// The street every place opens onto.
    pub street: Street,
    /// Every other drawn place, by its tags.
    pub places: Vec<PlaceBinding>,
    /// The offline spawn.
    pub spawn: Spawn,
    /// The ground (absent: flat ground at the street's surface).
    #[serde(default)]
    pub terrain: Option<Terrain>,
    /// The near water (absent: none).
    #[serde(default)]
    pub water: Option<NearWater>,
    /// Kerbs, quays, revetments, rails, low walls and gates.
    #[serde(default)]
    pub edges: Vec<Edge>,
    /// Trails painted into the ground.
    #[serde(default)]
    pub paths: Vec<PathLine>,
    /// Building shells.
    #[serde(default)]
    pub buildings: Vec<Building>,
    /// Species, stands and single trees (absent: none).
    #[serde(default)]
    pub trees: Option<Trees>,
    /// Boulders, reeds, grass and wildflower clumps.
    #[serde(default)]
    pub scatter: Vec<Scatter>,
    /// GI volumes and reflection probes (absent: none).
    #[serde(default)]
    pub light: Option<Light>,
    /// Place art from the pack's `assets/places/`, each placed by an anchor (16f-d1; absent: none).
    #[serde(default)]
    pub models: Vec<Model>,
    /// The checks' and captures' views.
    pub viewpoints: Viewpoints,
}

/// What a depiction file is for.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DepictionScope {
    /// A World Pack, by its directory name.
    pub world: Key,
}

/// Where plan (0, 0) stands.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    /// The scene position of plan (0, 0), metres.
    pub origin_m: [f64; 3],
}

/// The street's binding.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Street {
    /// The tags a disclosed place must carry to be drawn as this street.
    pub tags: Vec<Key>,
    /// Its footprint.
    pub footprint_mm: RectMm,
}

/// A drawn place's binding: by tags, never by a place key.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlaceBinding {
    /// The tags a disclosed place must carry to be drawn as this one.
    pub tags: Vec<Key>,
    /// The label drawn over its door.
    pub label: String,
    /// Its footprint.
    pub footprint_mm: RectMm,
    /// The street-side point of its door.
    pub doorway_mm: PointMm,
}

/// The offline spawn.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spawn {
    /// Where.
    pub at_mm: PointMm,
    /// How far above the street's surface, metres.
    pub up_m: f64,
    /// Facing, degrees.
    pub yaw_deg: f64,
}

/// The ground.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Terrain {
    /// The rasterised extent.
    pub extent_mm: RectMm,
    /// The grid's cell, metres.
    pub cell_m: f64,
    /// How far a region blends into its surroundings, metres.
    pub blend_m: f64,
    /// The seed of any noise.
    pub seed: u64,
    /// The splat layers (at most four).
    pub layers: Vec<TerrainLayer>,
    /// The layer outside every region and path.
    pub default_layer: Key,
    /// Regions with their own heights.
    pub regions: Vec<Region>,
    /// Height profiles outside every region.
    #[serde(default)]
    pub profiles: Vec<Profile>,
    /// The lake bed under the near water.
    #[serde(default)]
    pub bed: Option<Bed>,
}

/// A splat layer.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainLayer {
    /// Its name.
    pub name: Key,
    /// A texture slug of the client's library.
    pub material: AssetName,
    /// The texture's repeat, metres.
    pub repeat_m: f64,
    /// An optional tint.
    #[serde(default)]
    pub tint: Option<Colour>,
}

/// A region of the ground.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Region {
    /// Its name.
    pub name: Key,
    /// Its outline.
    pub polygon_mm: Vec<PointMm>,
    /// Its height.
    pub height: Height,
    /// Its splat layer.
    pub layer: Key,
    /// Its citation.
    #[serde(default)]
    pub source: String,
}

/// A height: exactly one of `flat_m` (with an optional crown) or `ramp`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Height {
    /// A flat height, metres.
    #[serde(default)]
    pub flat_m: Option<f64>,
    /// A crown above the flat height, metres.
    #[serde(default)]
    pub crown_m: Option<f64>,
    /// The crown's axis.
    #[serde(default)]
    pub crown_along: Option<Axis>,
    /// A ramp.
    #[serde(default)]
    pub ramp: Option<Ramp>,
}

/// A plan axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Axis {
    /// East–west.
    X,
    /// North–south.
    Y,
}

/// A height linear in the distance between two parallel lines.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ramp {
    /// The first line.
    pub from_mm: [PointMm; 2],
    /// The height on it, metres.
    pub from_m: f64,
    /// The second line.
    pub to_mm: [PointMm; 2],
    /// The height on it, metres.
    pub to_m: f64,
}

/// A height profile along a bearing.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    /// Its name.
    pub name: Key,
    /// The bearing, degrees clockwise from true north.
    pub bearing_deg: f64,
    /// The samples.
    pub samples: Vec<ProfileSample>,
    /// Its citation.
    #[serde(default)]
    pub source: String,
}

/// One profile sample.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileSample {
    /// Distance from the frame origin, metres.
    pub distance_m: f64,
    /// Height, metres.
    pub height_m: f64,
}

/// The bed under the near water.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bed {
    /// Depth below the water at `run_m` from the water line, metres.
    pub depth_m: f64,
    /// The run, metres.
    pub run_m: f64,
    /// Its citation.
    #[serde(default)]
    pub source: String,
}

/// The near water.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NearWater {
    /// Its surface, metres.
    pub level_m: f64,
    /// The entry of the same world's far field whose colours it takes.
    pub far_field_water: Key,
    /// How far out it runs, metres.
    pub extent_m: f64,
    /// Its outline.
    pub polygon_mm: Vec<PointMm>,
    /// The shader's uniforms; an absent one keeps the shader's default.
    #[serde(default)]
    pub shader: WaterShader,
    /// Lily pads.
    #[serde(default)]
    pub lilies: Vec<Lilies>,
    /// Its citation.
    #[serde(default)]
    pub source: String,
}

/// The water shader's uniforms.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WaterShader {
    /// The depth over which shallow blends to deep, metres.
    #[serde(default)]
    pub depth_fade_m: Option<f64>,
    /// The shore band's strength, 0…1.
    #[serde(default)]
    pub shore_foam: Option<f64>,
}

/// A lily-pad patch.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lilies {
    /// Its outline.
    pub polygon_mm: Vec<PointMm>,
    /// The covered fraction, 0…1.
    pub cover: f64,
    /// The mask's seed.
    pub seed: u64,
}

/// A line-drawn edge.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    /// Its name.
    pub name: Key,
    /// What it is.
    pub kind: EdgeKind,
    /// Its line.
    pub line_mm: Vec<PointMm>,
    /// Its height, metres.
    pub height_m: f64,
    /// Its width, metres.
    pub width_m: f64,
    /// A texture slug of the client's library.
    pub material: AssetName,
    /// An optional tint.
    #[serde(default)]
    pub tint: Option<Colour>,
    /// Tags of `places` at whose doorway it leaves a gap.
    #[serde(default)]
    pub open_at: Vec<Key>,
    /// Quay and revetment only: the fall to the water, metres.
    #[serde(default)]
    pub drop_m: Option<f64>,
    /// Rail only: the post spacing, metres.
    #[serde(default)]
    pub posts_m: Option<f64>,
    /// Its citation.
    #[serde(default)]
    pub source: String,
}

/// What an edge is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    /// A kerb.
    Kerb,
    /// A quay wall.
    Quay,
    /// A revetment.
    Revetment,
    /// A rail.
    Rail,
    /// A low garden wall.
    Wall,
    /// A gate.
    Gate,
}

/// A trail painted into the ground.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathLine {
    /// Its name.
    pub name: Key,
    /// Its centre line.
    pub line_mm: Vec<PointMm>,
    /// Its width, metres.
    pub width_m: f64,
    /// The terrain layer it is painted with.
    pub layer: Key,
}

/// A building shell.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Building {
    /// Its name.
    pub name: Key,
    /// The tags of the place whose walls it draws; `null` for a shell nobody enters.
    #[serde(default)]
    pub depicts: Option<Vec<Key>>,
    /// Its footprint.
    pub footprint_mm: Vec<PointMm>,
    /// Its storeys.
    pub storeys: u32,
    /// A storey's height, metres.
    pub storey_m: f64,
    /// The plinth's height, metres.
    pub plinth_m: f64,
    /// The walls.
    pub walls: Surface,
    /// The roof.
    pub roof: Roof,
    /// Doors and windows.
    #[serde(default)]
    pub openings: Vec<Opening>,
    /// A chimney.
    #[serde(default)]
    pub chimney: Option<Chimney>,
    /// Its citation.
    #[serde(default)]
    pub source: String,
}

/// A wall surface.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Surface {
    /// A texture slug of the client's library.
    pub material: AssetName,
    /// An optional tint.
    #[serde(default)]
    pub tint: Option<Colour>,
}

/// A roof.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Roof {
    /// Its form.
    pub form: RoofForm,
    /// The ridge's axis (gable, hip).
    #[serde(default)]
    pub ridge: Option<Axis>,
    /// Which way a mono roof falls.
    #[serde(default)]
    pub falls: Option<Side>,
    /// Its pitch, degrees.
    pub pitch_deg: f64,
    /// The eaves' overhang, metres.
    #[serde(default)]
    pub overhang_m: f64,
    /// A texture slug of the client's library.
    pub material: AssetName,
    /// An optional tint.
    #[serde(default)]
    pub tint: Option<Colour>,
}

/// A roof's form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoofForm {
    /// Two slopes and two gables.
    Gable,
    /// Four slopes.
    Hip,
    /// One slope (`falls` names its low side).
    Mono,
    /// Flat.
    Flat,
}

/// A side of a footprint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    /// North.
    N,
    /// South.
    S,
    /// East.
    E,
    /// West.
    W,
}

/// A door or window.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Opening {
    /// The wall it is in.
    pub side: Side,
    /// What it is.
    pub kind: OpeningKind,
    /// The storey, 0 the ground's.
    pub storey: u32,
    /// From the side's left end, metres (or `at_doorway`).
    #[serde(default)]
    pub offset_m: Option<f64>,
    /// On the depicted place's doorway.
    #[serde(default)]
    pub at_doorway: bool,
    /// Width, metres.
    pub width_m: f64,
    /// Height, metres.
    pub height_m: f64,
    /// Sill above the storey's floor, metres.
    #[serde(default)]
    pub sill_m: Option<f64>,
    /// How it is drawn.
    pub state: OpeningState,
}

/// What an opening is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpeningKind {
    /// A door.
    Door,
    /// A double door.
    DoubleDoor,
    /// A window.
    Window,
    /// A shop window.
    ShopWindow,
}

/// How an opening is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpeningState {
    /// A closed leaf.
    Closed,
    /// Open (only a door of a depicted place, at its doorway).
    Open,
    /// A dark interior card.
    Dark,
}

/// A chimney.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Chimney {
    /// Where.
    pub at_mm: PointMm,
    /// Its plan size, metres.
    pub size_m: [f64; 2],
    /// How far above the ridge, metres.
    pub above_ridge_m: f64,
}

/// The woods.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trees {
    /// The species table.
    pub species: Vec<Species>,
    /// Stands.
    #[serde(default)]
    pub stands: Vec<Stand>,
    /// Single planted trees.
    #[serde(default)]
    pub singles: Vec<SingleTree>,
    /// What trunks keep clear of.
    pub keep_clear: KeepClear,
    /// The distance between the two LOD bands, metres.
    pub lod_m: f64,
    /// Invisible walls behind the first rank.
    #[serde(default)]
    pub air_walls: Vec<AirWall>,
    /// Its citation.
    #[serde(default)]
    pub source: String,
}

/// A species.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Species {
    /// Its name in this file.
    pub name: Key,
    /// Its Latin name.
    pub latin: String,
    /// The client's builder form (checked by the client).
    pub form: AssetName,
    /// The client's card (checked by the client).
    pub card: AssetName,
    /// Its drawn heights, metres.
    pub height_m: RangeM,
    /// Its citation.
    #[serde(default)]
    pub source: String,
}

/// A closed range, metres.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RangeM {
    /// The least.
    pub min: f64,
    /// The most.
    pub max: f64,
}

/// A stand of trees.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stand {
    /// Its name.
    pub name: Key,
    /// Where it grows.
    pub area: Area,
    /// The Poisson-disc spacing, metres.
    pub spacing_m: f64,
    /// How dense, rising from the ring's centre.
    pub density: Density,
    /// The species mix.
    pub mix: Vec<Mix>,
    /// The placement's seed.
    pub seed: u64,
}

/// An area: exactly one of `ring` or `polygon_mm`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Area {
    /// An annulus.
    #[serde(default)]
    pub ring: Option<Ring>,
    /// A polygon.
    #[serde(default)]
    pub polygon_mm: Option<Vec<PointMm>>,
}

/// An annulus.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ring {
    /// Its centre.
    pub centre_mm: PointMm,
    /// Its inner radius, metres.
    pub inner_m: f64,
    /// Its outer radius, metres.
    pub outer_m: f64,
}

/// A stand's density.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Density {
    /// The density at the first distance, 0…1.
    pub from: f64,
    /// The density at the second, 0…1.
    pub to: f64,
    /// The two distances from the ring's centre, metres.
    pub rise_m: [f64; 2],
}

/// One species in a stand.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mix {
    /// The species, by its name in `species`.
    pub species: Key,
    /// Its weight.
    pub weight: f64,
    /// Ground it prefers.
    #[serde(default)]
    pub prefer: Option<Prefer>,
}

/// Ground a species prefers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Prefer {
    /// Low ground near water.
    Low,
    /// Rises.
    Rise,
}

/// A single planted tree.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SingleTree {
    /// The species, by its name in `species`.
    pub species: Key,
    /// Where.
    pub at_mm: PointMm,
    /// Its height, metres.
    pub height_m: f64,
}

/// What trunks keep clear of.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeepClear {
    /// Every `places` footprint.
    pub places: bool,
    /// The distance from every path, metres.
    pub paths_m: f64,
}

/// An invisible wall: exactly one of `ring` or `line_mm`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AirWall {
    /// A circle.
    #[serde(default)]
    pub ring: Option<AirRing>,
    /// A line.
    #[serde(default)]
    pub line_mm: Option<Vec<PointMm>>,
}

/// A circular air wall.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AirRing {
    /// Its centre.
    pub centre_mm: PointMm,
    /// Its radius, metres.
    pub radius_m: f64,
}

/// Scattered small things.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scatter {
    /// Its name.
    pub name: Key,
    /// What is scattered.
    pub kind: ScatterKind,
    /// Where: exactly one of `polygon_mm` or `along_mm` with `width_m`.
    pub area: ScatterArea,
    /// How many.
    pub count: u32,
    /// Their sizes, metres.
    pub size_m: RangeM,
    /// The placement's seed.
    pub seed: u64,
    /// Visibility range, metres.
    #[serde(default)]
    pub range_m: Option<f64>,
    /// Its citation.
    #[serde(default)]
    pub source: String,
}

/// What is scattered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScatterKind {
    /// Granite boulders.
    Boulder,
    /// Reed clumps.
    Reeds,
    /// Grass clumps.
    Grass,
    /// Wildflower clumps.
    Wildflower,
}

/// A scatter's area.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScatterArea {
    /// A polygon.
    #[serde(default)]
    pub polygon_mm: Option<Vec<PointMm>>,
    /// A strip's centre line.
    #[serde(default)]
    pub along_mm: Option<Vec<PointMm>>,
    /// The strip's width, metres.
    #[serde(default)]
    pub width_m: Option<f64>,
}

/// One place asset placed in the scene (16f-d1; `PACKAGE_FORMAT.md` §3). The asset's frame is placed by
/// translation, and by a turn about the vertical for `at_mm`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    /// The asset: a folder of the pack's `assets/places/`.
    pub asset: Key,
    /// The tags of the place it depicts, or `null` for a piece nobody enters.
    #[serde(default)]
    pub depicts: Option<Vec<Key>>,
    /// Where it stands.
    pub anchor: Anchor,
}

/// Where a model stands: exactly one of an entrance put on the depicted place's doorway
/// (`entrance` with `at_doorway: true`), or the asset's origin at a plan point (`at_mm`, optional
/// `yaw_deg`).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Anchor {
    /// The asset's entrance that lands on the doorway.
    #[serde(default)]
    pub entrance: Option<AssetName>,
    /// Put the entrance on the depicted place's `doorway_mm`.
    #[serde(default)]
    pub at_doorway: bool,
    /// Where the asset's origin stands.
    #[serde(default)]
    pub at_mm: Option<PointMm>,
    /// The asset's turn about the vertical, degrees (with `at_mm`).
    #[serde(default)]
    pub yaw_deg: Option<f64>,
}

/// GI volumes and reflection probes.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Light {
    /// `VoxelGI` volumes (at most four).
    #[serde(default)]
    pub gi: Vec<GiVolume>,
    /// Reflection probes.
    #[serde(default)]
    pub probes: Vec<Probe>,
}

/// A GI volume.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GiVolume {
    /// Its plan box.
    pub box_mm: RectMm,
    /// Its height, metres.
    pub height_m: f64,
    /// Its subdivision: 64, 128 or 256.
    pub subdiv: u32,
}

/// A reflection probe.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Probe {
    /// Its plan box.
    pub box_mm: RectMm,
    /// Its height, metres.
    pub height_m: f64,
    /// Its centre above the street's surface, metres.
    pub centre_up_m: f64,
    /// When it updates.
    pub update: ProbeUpdate,
}

/// When a probe updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeUpdate {
    /// Once.
    Once,
}

/// The checks' and captures' views.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Viewpoints {
    /// The far-field check's viewpoints.
    pub farfield: Vec<FarFieldViewpoint>,
    /// The far-field regression views.
    #[serde(default)]
    pub farfield_shots: Vec<View>,
    /// The perf views.
    #[serde(default)]
    pub perf: Vec<View>,
    /// The perf breakdown's views, by name.
    #[serde(default)]
    pub perf_breakdown: Vec<String>,
    /// The link probe's walk, by tags.
    #[serde(default)]
    pub probe_route: Vec<Vec<Key>>,
}

/// A far-field viewpoint.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FarFieldViewpoint {
    /// Its id.
    pub id: String,
    /// Where.
    pub at_mm: PointMm,
    /// The eye above the street's surface, metres.
    pub eye_m: f64,
    /// Its roles.
    #[serde(default)]
    pub roles: Vec<ViewpointRole>,
}

/// A far-field viewpoint's role (RL-e0b §6.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewpointRole {
    /// Drawn with the far field's protected near band.
    Protected,
    /// Judged by the check.
    Judged,
    /// A high view.
    High,
}

/// A named view.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct View {
    /// Its name.
    pub name: String,
    /// Where.
    pub at_mm: PointMm,
    /// The eye above the street's surface, metres.
    pub eye_m: f64,
    /// Facing, degrees.
    pub yaw_deg: f64,
    /// Looking up, degrees.
    pub pitch_deg: f64,
}
