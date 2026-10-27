//! The depiction file's optional `stations` section (`MODULE_SPEC.md` §6.1.2; `DECISIONS.md` `ARC-179`;
//! S33-e §5.2): a station's fixtures — a departures board, a clock, a ticket window — placed in the places
//! the station's building depicts, and its platform with the one track beside it, drawn by S29-e's shared
//! station pieces (`StationPieces`, `StationTrack`). The building itself is a `buildings` shell.
//!
//! A fixture shows only what the world disclosed about the place it stands in (`ARC-108`): the file says
//! where a board hangs, never what it reads. A platform depicts a place: its footprint is that place's
//! floor, its `surface_m` where its people stand. The track and its trackbed lie beside the platform's
//! edge, off every place's floor; nothing here draws a train (step-29 §5.2). Its checks run inside the
//! file's stages (`depiction_check.rs`), so one file still gets one code in the order
//! `MALFORMED → … → ON_FLOOR`.

use serde::Deserialize;

use crate::depiction_deck::PlaceFloor;
use crate::depiction_schema::{Colour, Key, PointMm, RectMm, Side};
use crate::far_field_schema::AssetName;

/// The most fixtures one station places.
pub const MAX_FIXTURES: usize = 16;
/// The most stations one file draws.
pub const MAX_STATIONS: usize = 4;

/// One station.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Station {
    /// Its name.
    pub name: Key,
    /// The pieces' materials: `StationPieces.style_of`'s name (`timber`, or the depot's `oak`).
    pub style: StationStyle,
    /// Boards, clocks and ticket windows.
    #[serde(default)]
    pub fixtures: Vec<Fixture>,
    /// The platform and its track.
    #[serde(default)]
    pub platform: Option<Platform>,
    /// Its citation.
    #[serde(default)]
    pub source: String,
}

/// A station's materials.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StationStyle {
    /// Painted boards and black iron (a village halt).
    Timber,
    /// Oak and bronze (a town's depot).
    Oak,
}

/// A fixture: one of S29-e's station pieces.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    /// Which piece.
    pub piece: Piece,
    /// The tags of the place it stands in, whose disclosed state it shows.
    #[serde(rename = "in")]
    pub in_place: Vec<Key>,
    /// Its centre on plan.
    pub at_mm: PointMm,
    /// Its centre's height above the floor (a window bay: its base, on the floor), metres.
    pub up_m: f64,
    /// The way its face looks, degrees clockwise from north.
    pub facing_deg: f64,
    /// A board's width and height; a clock's diameter twice; a window bay's width and depth; metres.
    pub size_m: [f64; 2],
    /// A post under it, this wide (a board or a clock standing free), metres.
    #[serde(default)]
    pub post_m: Option<f64>,
    /// A window bay's counter height, metres.
    #[serde(default)]
    pub counter_m: Option<f64>,
    /// A window bay's screen top, metres.
    #[serde(default)]
    pub screen_m: Option<f64>,
    /// A window bay's grilles.
    #[serde(default)]
    pub grilles: Option<u32>,
}

/// A station piece.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Piece {
    /// A departures board.
    Board,
    /// A wall clock.
    Clock,
    /// A ticket counter with its screen and shuttered grilles.
    WindowBay,
}

/// A platform beside one track.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Platform {
    /// The tags of the place whose floor it is.
    pub depicts: Vec<Key>,
    /// The side its edge (and the track) is on.
    pub edge: Side,
    /// The walking surface, metres above the street's surface.
    pub surface_m: f64,
    /// The surface's height above the rail head, metres.
    pub height_m: f64,
    /// The edge's stone coping, millimetres.
    pub coping_mm: u32,
    /// The detectable warning strip inboard of the coping, millimetres.
    pub strip_mm: u32,
    /// A fence with a closed gate across each end, this tall, metres.
    pub end_fence_m: f64,
    /// A roof cantilevered over the platform from its back.
    #[serde(default)]
    pub canopy: Option<Canopy>,
    /// The track.
    pub track: Track,
}

/// A canopy from the platform's back edge toward its edge.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Canopy {
    /// From and to, along the platform's length, plan millimetres on that axis.
    pub along_mm: [i64; 2],
    /// How far it reaches out, metres.
    pub depth_m: f64,
    /// Its outer (low) edge above the surface, metres.
    pub eaves_m: f64,
    /// Its rise toward the back, degrees.
    pub pitch_deg: f64,
    /// A texture slug of the client's library.
    pub material: AssetName,
    /// An optional tint.
    #[serde(default)]
    pub tint: Option<Colour>,
}

/// The track along the platform's edge.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Track {
    /// Between the rails' inside faces, millimetres.
    pub gauge_mm: u32,
    /// From the platform's edge to the track's centreline, millimetres.
    pub near_mm: u32,
    /// A tie's length and width, millimetres.
    pub tie_mm: [u32; 2],
    /// Tie centres, millimetres.
    pub tie_pitch_mm: u32,
    /// A rail's width and height, millimetres.
    pub rail_mm: [u32; 2],
    /// The end of the platform the line stops at, with a buffer stop; absent, it runs on at both.
    #[serde(default)]
    pub buffer: Option<Side>,
    /// How far the line runs on past the platform's other end(s) before its cutting, metres.
    pub beyond_m: f64,
    /// The cutting it leaves through, metres long (0: none).
    pub cutting_m: f64,
    /// The cutting's banks, metres above the trackbed.
    pub bank_m: f64,
}

/// The ballast's shoulder beyond the tie ends, millimetres (`StationTrack.SHOULDER`).
pub const SHOULDER_MM: f64 = 450.0;

impl Track {
    /// The trackbed's width from the platform's edge, millimetres: the near half, the tie's far half
    /// and the ballast's shoulder.
    #[must_use]
    pub fn reach_mm(&self) -> f64 {
        f64::from(self.near_mm) + f64::from(self.tie_mm[0]) / 2.0 + SHOULDER_MM
    }
}

// --- the checks, one per stage -------------------------------------------------------------------

/// Shape: a window bay states its counter, screen and grilles and no post; a board or a clock states
/// none of the three.
pub(crate) fn shape(stations: &[Station]) -> Result<(), String> {
    for (i, s) in stations.iter().enumerate() {
        for (j, f) in s.fixtures.iter().enumerate() {
            let at = format!("stations[{i}] \"{}\".fixtures[{j}]", s.name.as_str());
            let bay = (
                f.counter_m.is_some(),
                f.screen_m.is_some(),
                f.grilles.is_some(),
            );
            match f.piece {
                Piece::WindowBay if bay != (true, true, true) || f.post_m.is_some() => {
                    return Err(format!(
                        "{at}: a window bay states counter_m, screen_m and grilles, and no post_m"
                    ));
                }
                Piece::Board | Piece::Clock if bay != (false, false, false) => {
                    return Err(format!(
                        "{at}: counter_m, screen_m and grilles are a window bay's, not a {}'s",
                        if f.piece == Piece::Board {
                            "board"
                        } else {
                            "clock"
                        }
                    ));
                }
                _ => {}
            }
        }
    }
    Ok(())
}

/// Every station's citation, for the citations stage.
pub(crate) fn citations(stations: &[Station]) -> Vec<(String, &String)> {
    stations
        .iter()
        .enumerate()
        .map(|(i, s)| (format!("stations[{i}] \"{}\"", s.name.as_str()), &s.source))
        .collect()
}

/// The first value outside its range, or `None`.
pub(crate) fn ranges(stations: &[Station], heights: (f64, f64)) -> Option<String> {
    let mut out: Option<String> = None;
    let mut within = |at: String, value: f64, (low, high): (f64, f64)| {
        if out.is_none() && !(value.is_finite() && (low..=high).contains(&value)) {
            out = Some(format!("{at}: {value} is outside {low}…{high}"));
        }
    };
    within(
        "stations (count)".to_owned(),
        stations.len() as f64,
        (0.0, MAX_STATIONS as f64),
    );
    for (i, s) in stations.iter().enumerate() {
        let at = |field: &str| format!("stations[{i}] \"{}\".{field}", s.name.as_str());
        within(
            at("fixtures (count)"),
            s.fixtures.len() as f64,
            (0.0, MAX_FIXTURES as f64),
        );
        for (j, f) in s.fixtures.iter().enumerate() {
            let fat = |field: &str| at(&format!("fixtures[{j}].{field}"));
            within(fat("up_m"), f.up_m, (0.0, 6.0));
            within(fat("facing_deg"), f.facing_deg, (-360.0, 360.0));
            within(fat("size_m[0]"), f.size_m[0], (0.1, 10.0));
            within(fat("size_m[1]"), f.size_m[1], (0.1, 10.0));
            if let Some(p) = f.post_m {
                within(fat("post_m"), p, (0.05, 0.5));
            }
            if let (Some(c), Some(t), Some(g)) = (f.counter_m, f.screen_m, f.grilles) {
                within(fat("counter_m"), c, (0.6, 1.5));
                within(fat("screen_m"), t, (c + 1.0, 4.0));
                within(fat("grilles"), f64::from(g), (1.0, 8.0));
            }
        }
        if let Some(p) = &s.platform {
            let pat = |field: &str| at(&format!("platform.{field}"));
            within(pat("surface_m"), p.surface_m, heights);
            within(pat("height_m"), p.height_m, (0.0, 1.5));
            within(pat("coping_mm"), f64::from(p.coping_mm), (0.0, 1000.0));
            within(pat("strip_mm"), f64::from(p.strip_mm), (0.0, 1000.0));
            within(pat("end_fence_m"), p.end_fence_m, (0.6, 2.0));
            if let Some(c) = &p.canopy {
                within(pat("canopy.depth_m"), c.depth_m, (0.5, 5.0));
                within(pat("canopy.eaves_m"), c.eaves_m, (2.1, 6.0));
                within(pat("canopy.pitch_deg"), c.pitch_deg, (0.0, 45.0));
            }
            let t = &p.track;
            let tat = |field: &str| pat(&format!("track.{field}"));
            within(tat("gauge_mm"), f64::from(t.gauge_mm), (500.0, 2000.0));
            within(
                tat("near_mm"),
                f64::from(t.near_mm),
                (f64::from(t.gauge_mm) / 2.0, 5000.0),
            );
            within(
                tat("tie_mm[0]"),
                f64::from(t.tie_mm[0]),
                (f64::from(t.gauge_mm), 4000.0),
            );
            within(tat("tie_mm[1]"), f64::from(t.tie_mm[1]), (100.0, 400.0));
            within(
                tat("tie_pitch_mm"),
                f64::from(t.tie_pitch_mm),
                (300.0, 1500.0),
            );
            within(tat("rail_mm[0]"), f64::from(t.rail_mm[0]), (30.0, 150.0));
            within(tat("rail_mm[1]"), f64::from(t.rail_mm[1]), (50.0, 250.0));
            within(tat("beyond_m"), t.beyond_m, (0.0, 100.0));
            within(tat("cutting_m"), t.cutting_m, (0.0, 100.0));
            within(tat("bank_m"), t.bank_m, (0.5, 10.0));
        }
    }
    out
}

/// The first fixture or platform whose tags resolve to no one place, or `None`. `resolves` is the
/// file's rule (`depiction_check::resolve`): the place whose tags are exactly these, else the only one
/// carrying them all.
pub(crate) fn references(
    stations: &[Station],
    resolves: impl Fn(&[Key]) -> bool,
) -> Option<String> {
    let names = |tags: &[Key]| tags.iter().map(Key::as_str).collect::<Vec<_>>().join(", ");
    for (i, s) in stations.iter().enumerate() {
        let at = format!("stations[{i}] \"{}\"", s.name.as_str());
        for (j, f) in s.fixtures.iter().enumerate() {
            if !resolves(&f.in_place) {
                return Some(format!(
                    "{at}.fixtures[{j}].in: no one place is tagged exactly, or alone, {}",
                    names(&f.in_place)
                ));
            }
        }
        if let Some(p) = &s.platform
            && !resolves(&p.depicts)
        {
            return Some(format!(
                "{at}.platform.depicts: no one place is tagged exactly, or alone, {}",
                names(&p.depicts)
            ));
        }
    }
    None
}

/// Floors: each fixture inside its place's footprint; the platform on its place's footprint within
/// `tolerance` mm, its canopy along it; the trackbed beside the edge on no place's floor (`floors` are
/// the places' floors shrunk by the tolerance).
pub(crate) fn floors(
    stations: &[Station],
    place_of: impl Fn(&[Key]) -> Option<RectMm>,
    floors: &[PlaceFloor],
) -> Option<String> {
    for (i, s) in stations.iter().enumerate() {
        let at = format!("stations[{i}] \"{}\"", s.name.as_str());
        for (j, f) in s.fixtures.iter().enumerate() {
            let Some(r) = place_of(&f.in_place) else {
                continue;
            };
            let [x, y] = f.at_mm;
            if x < r[0] || x > r[0] + r[2] || y < r[1] || y > r[1] + r[3] {
                return Some(format!(
                    "{at}.fixtures[{j}].at_mm: the fixture stands outside its place's footprint"
                ));
            }
        }
        let Some(p) = &s.platform else {
            continue;
        };
        let Some(r) = place_of(&p.depicts) else {
            continue;
        };
        if let Some(c) = &p.canopy {
            let (lo, hi) = match p.edge {
                Side::E | Side::W => (r[1], r[1] + r[3]),
                Side::N | Side::S => (r[0], r[0] + r[2]),
            };
            let [a, b] = c.along_mm;
            if a >= b || a < lo || b > hi {
                return Some(format!(
                    "{at}.platform.canopy.along_mm: [{a}, {b}] is not an interval within the platform's \
                     length ({lo}…{hi})"
                ));
            }
        }
        let bed = trackbed(r, p);
        if let Some((tags, _)) = floors
            .iter()
            .find(|(_, (x0, y0, x1, y1))| bed.0 < *x1 && bed.2 > *x0 && bed.1 < *y1 && bed.3 > *y0)
        {
            return Some(format!(
                "{at}.platform.track: the trackbed beside the edge overlaps the place tagged {tags}"
            ));
        }
    }
    None
}

/// The trackbed beside the platform `r`'s edge, along the platform's length, as (x0, y0, x1, y1) mm.
fn trackbed(r: RectMm, p: &Platform) -> (f64, f64, f64, f64) {
    let (x0, y0) = (r[0] as f64, r[1] as f64);
    let (x1, y1) = (x0 + r[2] as f64, y0 + r[3] as f64);
    let reach = p.track.reach_mm();
    match p.edge {
        Side::E => (x1, y0, x1 + reach, y1),
        Side::W => (x0 - reach, y0, x0, y1),
        Side::N => (x0, y1, x1, y1 + reach),
        Side::S => (x0, y0 - reach, x1, y0),
    }
}
