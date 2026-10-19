//! The far field's types (`MODULE_SPEC.md` §6.1.1, format 1): each a named type, no maps, no untyped
//! values. Unknown keys are refused everywhere.

use std::fmt;

use serde::Deserialize;
use serde::de::{self, Deserializer, MapAccess, Visitor};

use crate::far_field::SLOT_MARK;

/// One far-field file (`MODULE_SPEC.md` §6.1.1, format 1).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FarField {
    /// Always 1.
    pub format: u32,
    /// The world or region this file is for; it equals the file's stem.
    pub scope: Scope,
    /// Where the reachable presentation ends.
    pub boundary: Boundary,
    /// Water in sight; may be empty.
    pub water: Vec<Water>,
    /// 3D: the presented ground's elevation above sea level, metres.
    #[serde(default)]
    pub datum_m: Option<f64>,
    /// 3D: the datum's citation.
    #[serde(default)]
    pub datum_source: Option<String>,
    /// 3D: the sky.
    #[serde(default)]
    pub sky: Option<Sky>,
    /// 3D: the ground from the boundary outward.
    #[serde(default)]
    pub foreground: Option<Foreground>,
    /// 3D: distant terrain bands.
    #[serde(default)]
    pub terrain_silhouettes: Option<Vec<Silhouette>>,
    /// 3D: the town's distant massing.
    #[serde(default)]
    pub skyline: Option<Skyline>,
    /// 2D: what is drawn past the land rectangle.
    #[serde(default)]
    pub surround: Option<Surround>,
}

/// What a far-field file is for.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Scope {
    /// A World Pack, by its directory name.
    World(Key),
    /// An S21 region.
    Region(Key),
}

/// Where the reachable presentation ends.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Boundary {
    /// How the boundary is made.
    pub kind: BoundaryKind,
    /// The radius containing every reachable point and every near construction, metres.
    pub reach_m: f64,
    /// What the ground just past the boundary is.
    pub beyond: Beyond,
}

/// How a boundary is made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryKind {
    /// An invisible wall.
    InvisibleWall,
    /// Built or landscape terminations.
    Natural,
}

/// What lies just past the boundary. No value names a walkable approach.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Beyond {
    /// A drop.
    Drop,
    /// Water.
    Water,
    /// Scrub.
    Scrub,
    /// Planting.
    Planting,
    /// Rough ground.
    RoughGround,
}

/// The sky (3D).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sky {
    /// Panorama or procedural.
    pub kind: SkyKind,
    /// The panorama asset, iff `kind` is `panorama`.
    #[serde(default)]
    pub panorama: Option<AssetName>,
    /// The procedural colours, iff `kind` is `procedural`.
    #[serde(default)]
    pub procedural: Option<ProceduralSky>,
    /// Meteorological optical range V, metres (Koschmieder).
    pub visibility_m: f64,
    /// Its citation.
    pub visibility_source: String,
}

/// How the sky is made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkyKind {
    /// A panorama image from the client's asset catalogue.
    Panorama,
    /// Two colours.
    Procedural,
}

/// A procedural sky's colours.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProceduralSky {
    /// The zenith colour.
    pub top: Colour,
    /// The horizon colour.
    pub horizon: Colour,
}

/// The ground from the boundary outward (3D).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Foreground {
    /// Its surface.
    pub surface: Surface,
    /// How far it falls below the datum, metres.
    pub drop_m: f64,
    /// Over what horizontal distance, metres.
    pub run_m: f64,
}

/// A foreground surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Surface {
    /// Scrub.
    Scrub,
    /// Dry grass.
    DryGrass,
    /// Grass.
    Grass,
    /// Rough ground.
    RoughGround,
}

/// One band of distant terrain (3D).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Silhouette {
    /// Its name.
    pub name: Key,
    /// Its layer: draw order and proxy distance.
    pub layer: Layer,
    /// Where it exists.
    pub sector: Sector,
    /// Its surface colour before haze.
    pub albedo: Colour,
    /// Its crest, from `sector.from_deg` to `sector.to_deg`.
    pub profile: Vec<ProfilePoint>,
    /// Its citation.
    pub source: String,
}

/// A silhouette's layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layer {
    /// Nearest.
    Near,
    /// Middle.
    Mid,
    /// Farthest.
    Far,
}

/// One point of a silhouette's crest.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfilePoint {
    /// True bearing, degrees.
    pub bearing_deg: f64,
    /// Crest height above sea level, metres.
    pub height_m: f64,
    /// Distance from the origin, metres.
    pub distance_m: f64,
}

/// A body of water in sight.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Water {
    /// Its name.
    pub name: Key,
    /// Its kind.
    pub kind: WaterKind,
    /// Where it lies.
    pub sector: Sector,
    /// Its nearest shoreline, metres.
    pub shore_m: f64,
    /// Its surface above sea level, metres.
    pub level_m: f64,
    /// Whether it runs to the horizon.
    pub horizon: Horizon,
    /// Its colours.
    pub colour: WaterColour,
    /// Its citation.
    pub source: String,
    /// A river's width, metres (river only).
    #[serde(default)]
    pub width_m: Option<f64>,
    /// A river's course (river only).
    #[serde(default)]
    pub course: Option<Vec<CoursePoint>>,
}

/// A kind of water.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaterKind {
    /// Open sea.
    Sea,
    /// A bay.
    Bay,
    /// A lake.
    Lake,
    /// A river.
    River,
}

/// Whether water runs to the horizon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Horizon {
    /// Water to the horizon with nothing behind it.
    Open,
    /// Something stands behind it.
    Closed,
}

/// Water's colours.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WaterColour {
    /// Deep water.
    pub deep: Colour,
    /// Shallow water.
    pub shallow: Colour,
}

/// One point of a river's course.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoursePoint {
    /// True bearing, degrees.
    pub bearing_deg: f64,
    /// Distance from the origin, metres.
    pub distance_m: f64,
}

/// The town's distant massing (3D).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Skyline {
    /// Where massing stands.
    pub sectors: Vec<MassingSector>,
    /// The massing's deterministic layout.
    pub seed: u32,
    /// The named slots.
    pub slots: Slots,
}

/// Massing in one sector.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MassingSector {
    /// Where.
    pub sector: Sector,
    /// Nearest radius, metres.
    pub from_m: f64,
    /// Farthest radius, metres.
    pub to_m: f64,
    /// Building heights above their ground, metres.
    pub height_m: HeightRange,
    /// Share of plan area covered by buildings.
    pub density: f64,
    /// Its citation.
    pub source: String,
}

/// A range of heights, metres.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeightRange {
    /// Lowest.
    pub min: f64,
    /// Highest.
    pub max: f64,
}

/// The two named slots every 3D far field has.
#[derive(Debug, Clone, PartialEq)]
pub struct Slots {
    /// Where the station will stand.
    pub station: Slot,
    /// Where the landmark will stand.
    pub landmark: Slot,
}

/// A named slot's state.
#[derive(Debug, Clone, PartialEq)]
pub enum Slot {
    /// Not placed yet.
    Unplaced,
    /// Reserved at a place, its footprint kept clear of massing.
    Reserved {
        /// True bearing, degrees.
        bearing_deg: f64,
        /// Distance from the origin, metres.
        distance_m: f64,
        /// Footprint, metres.
        footprint_m: f64,
        /// Its citation.
        source: String,
    },
}

/// What is drawn past the land rectangle (2D).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Surround {
    /// How far beyond the land rectangle, plan metres.
    pub extent_m: f64,
    /// What fills it where no water is declared.
    pub base: SurroundBase,
    /// The haze toward the edge.
    pub haze: Haze,
}

/// What fills a surround.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurroundBase {
    /// Land.
    Land,
    /// Water.
    Water,
}

/// A surround's haze.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Haze {
    /// The colour it blends toward.
    pub colour: Colour,
    /// Where it starts beyond the land rectangle, metres.
    pub from_m: f64,
    /// Where it is complete, metres.
    pub to_m: f64,
}

/// A clockwise sector of true bearings, `[from_deg, to_deg)`, which may wrap through 0.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sector {
    /// Where it starts.
    pub from_deg: f64,
    /// Where it ends.
    pub to_deg: f64,
}

impl Sector {
    /// Its clockwise span, degrees.
    #[must_use]
    pub fn span(&self) -> f64 {
        clockwise(self.from_deg, self.to_deg)
    }

    /// Whether `bearing` lies in `[from_deg, to_deg)`.
    #[must_use]
    pub fn contains(&self, bearing: f64) -> bool {
        clockwise(self.from_deg, bearing) < self.span()
    }
}

/// The clockwise angle from `a` to `b`, in `[0, 360)`.
pub(crate) fn clockwise(a: f64, b: f64) -> f64 {
    (b - a).rem_euclid(360.0)
}

/// Defines a string newtype that deserializes only when its rule holds.
macro_rules! checked_text {
    ($(#[$doc:meta])* $name:ident, $rule:expr, $what:literal) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name(String);

        impl $name {
            /// The text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let text = String::deserialize(deserializer)?;
                let rule: fn(&str) -> bool = $rule;
                if rule(&text) {
                    Ok(Self(text))
                } else {
                    Err(de::Error::custom(format!("'{text}' is not {}", $what)))
                }
            }
        }
    };
}

checked_text!(
    /// A key: 1–48 of `a-z`, `0-9`, `-`.
    Key,
    |t| (1..=48).contains(&t.len())
        && t.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'),
    "a key (1-48 of a-z, 0-9, -)"
);
checked_text!(
    /// An asset name: 1–64 of `a-z`, `0-9`, `_`, `-`.
    AssetName,
    |t| (1..=64).contains(&t.len())
        && t.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_'),
    "an asset name (1-64 of a-z, 0-9, _, -)"
);
checked_text!(
    /// A colour, `#rrggbb`.
    Colour,
    |t| t.len() == 7 && t.starts_with('#') && t.bytes().skip(1).all(|b| b.is_ascii_hexdigit()),
    "a colour (#rrggbb)"
);

impl<'de> Deserialize<'de> for Slots {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SlotsVisitor;
        impl<'de> Visitor<'de> for SlotsVisitor {
            type Value = Slots;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("the slots station and landmark")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Slots, A::Error> {
                let (mut station, mut landmark) = (None, None);
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "station" => station = Some(map.next_value::<SlotFields>()?),
                        "landmark" => landmark = Some(map.next_value::<SlotFields>()?),
                        other => {
                            return Err(de::Error::custom(format!(
                                "{SLOT_MARK}'{other}' is not a slot (station, landmark)"
                            )));
                        }
                    }
                }
                let missing = |name| de::Error::custom(format!("{SLOT_MARK}{name} is missing"));
                Ok(Slots {
                    station: station.ok_or_else(|| missing("station"))?.slot()?,
                    landmark: landmark.ok_or_else(|| missing("landmark"))?.slot()?,
                })
            }
        }
        deserializer.deserialize_map(SlotsVisitor)
    }
}

/// A slot as written, before its state decides which fields it needs.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SlotFields {
    state: String,
    #[serde(default)]
    bearing_deg: Option<f64>,
    #[serde(default)]
    distance_m: Option<f64>,
    #[serde(default)]
    footprint_m: Option<f64>,
    #[serde(default)]
    source: Option<String>,
}

impl SlotFields {
    fn slot<E: de::Error>(self) -> Result<Slot, E> {
        let fields = (
            self.bearing_deg,
            self.distance_m,
            self.footprint_m,
            self.source,
        );
        match (self.state.as_str(), fields) {
            ("unplaced", (None, None, None, None)) => Ok(Slot::Unplaced),
            ("unplaced", _) => Err(E::custom(format!(
                "{SLOT_MARK}an unplaced slot states no position"
            ))),
            (
                "reserved",
                (Some(bearing_deg), Some(distance_m), Some(footprint_m), Some(source)),
            ) => Ok(Slot::Reserved {
                bearing_deg,
                distance_m,
                footprint_m,
                source,
            }),
            ("reserved", _) => Err(E::custom(format!(
                "{SLOT_MARK}a reserved slot states bearing_deg, distance_m, footprint_m and source"
            ))),
            (other, _) => Err(E::custom(format!(
                "{SLOT_MARK}'{other}' is not a slot state (unplaced, reserved)"
            ))),
        }
    }
}
