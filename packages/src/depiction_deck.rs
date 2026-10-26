//! The depiction file's optional `decks` section (`MODULE_SPEC.md` §6.1.2; `DECISIONS.md` `ARC-178`;
//! S33-e §5.1.1): a walked floor raised on piles over water or low ground — a pier, a jetty, a
//! boardwalk, a platform — with its optional approach ramp, rails, curb, open shelter and moored craft.
//!
//! A deck depicts a place: its footprint is that place's floor (within 50 mm), and its surface is where
//! that place's people stand. Furniture on it stays the world's (`ARC-307`); the section has no field
//! for a bench. Its checks run inside the file's stages (`depiction_check.rs`), so one file still gets
//! one code in the order `MALFORMED → … → ON_FLOOR`.

use serde::Deserialize;

use crate::depiction_schema::{Axis, Colour, Key, PointMm, RectMm, Roof, RoofForm, Side};
use crate::far_field_schema::AssetName;

/// The most posts a shelter has, and craft a deck moors.
pub const MAX_SHELTER_POSTS: usize = 16;
/// The most craft one deck moors.
pub const MAX_MOORED: usize = 16;
/// A place's floor shrunk by the floor tolerance: its tags (joined) and (x0, y0, x1, y1) in millimetres.
pub(crate) type PlaceFloor = (String, (f64, f64, f64, f64));

/// The steepest approach ramp: 1:12 (2010 ADA Standards §405.2, "not steeper than 1:12").
pub const STEEPEST_APPROACH: f64 = 1.0 / 12.0;

/// One deck.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Deck {
    /// Its name.
    pub name: Key,
    /// The tags of the place whose floor it is.
    pub depicts: Vec<Key>,
    /// Its footprint: the depicted place's floor.
    pub footprint_mm: RectMm,
    /// The walking surface, metres above the street's surface.
    pub surface_m: f64,
    /// Boards and stringers, metres.
    pub thickness_m: f64,
    /// The boards.
    pub boards: Boards,
    /// A ramp inside the footprint from one edge to the surface.
    #[serde(default)]
    pub approach: Option<Approach>,
    /// The piles.
    pub piles: Piles,
    /// Rails on some sides.
    #[serde(default)]
    pub rails: Option<DeckRails>,
    /// An edge curb along every railed side.
    #[serde(default)]
    pub curb: Option<Curb>,
    /// An open shelter on posts.
    #[serde(default)]
    pub shelter: Option<Shelter>,
    /// Static craft moored beside it.
    #[serde(default)]
    pub moored: Vec<Moored>,
    /// Its citation.
    #[serde(default)]
    pub source: String,
}

/// The deck's boards.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Boards {
    /// A texture slug of the client's library.
    pub material: AssetName,
    /// An optional tint.
    #[serde(default)]
    pub tint: Option<Colour>,
    /// The axis the boards run along.
    pub across: Axis,
    /// A board's width, millimetres.
    pub width_mm: u32,
    /// The gap between boards, millimetres.
    pub gap_mm: u32,
}

/// A ramp from the footprint's `side` edge, at `from_m`, to the surface over `length_m`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Approach {
    /// The edge it starts at.
    pub side: Side,
    /// Its run, metres.
    pub length_m: f64,
    /// Its height at that edge, metres above the street's surface.
    pub from_m: f64,
}

/// The piles: `bents` pairs of round piles, evenly spaced along the deck's length.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Piles {
    /// How many pairs.
    pub bents: u32,
    /// A pile's diameter, metres.
    pub diameter_m: f64,
    /// How far the end bents and the piles stand in from the footprint's edges, metres.
    pub inset_m: f64,
    /// Where the piles end below, metres above the street's surface.
    pub to_m: f64,
    /// X-bracing between each pair.
    pub bracing: bool,
    /// A texture slug of the client's library.
    pub material: AssetName,
}

/// Round rails on some of the deck's sides.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeckRails {
    /// The railed sides.
    pub sides: Vec<Side>,
    /// The top rail's height above the surface, metres.
    pub height_m: f64,
    /// A rail's diameter, metres.
    pub rail_m: f64,
    /// A post's width, metres.
    pub post_m: f64,
    /// Post spacing, metres.
    pub posts_m: f64,
    /// A texture slug of the client's library.
    pub material: AssetName,
}

/// An edge curb.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Curb {
    /// Its height above the surface, metres.
    pub height_m: f64,
    /// Its width, metres.
    pub width_m: f64,
}

/// An open shelter: posts and a roof over their bounding box.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shelter {
    /// The posts' centres.
    pub posts_mm: Vec<PointMm>,
    /// A post's width, metres.
    pub post_m: f64,
    /// The eaves' height above the surface, metres.
    pub eaves_m: f64,
    /// The roof, as a building's.
    pub roof: Roof,
    /// A lantern hung on one post.
    #[serde(default)]
    pub lantern: Option<Lantern>,
}

/// A lantern on a shelter's post.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lantern {
    /// The post's index in `posts_mm`.
    pub post: usize,
    /// Its height above the surface, metres.
    pub up_m: f64,
}

/// A static craft.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Moored {
    /// What it is.
    pub kind: CraftKind,
    /// Its centre.
    pub at_mm: PointMm,
    /// Its heading, degrees clockwise from north.
    pub yaw_deg: f64,
    /// Its length, metres.
    pub length_m: f64,
    /// Its beam, metres.
    pub beam_m: f64,
}

/// A kind of craft.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CraftKind {
    /// A small rowing boat.
    RowingBoat,
}

// --- the checks, one per stage -------------------------------------------------------------------

/// Shape: a mono roof needs `falls`; a lantern names a post.
pub(crate) fn shape(decks: &[Deck]) -> Result<(), String> {
    for (i, d) in decks.iter().enumerate() {
        if let Some(s) = &d.shelter {
            if (s.roof.form == RoofForm::Mono) != s.roof.falls.is_some() {
                return Err(format!(
                    "decks[{i}].shelter.roof: falls is stated for a mono roof, and only for it"
                ));
            }
            if let Some(l) = &s.lantern
                && l.post >= s.posts_mm.len()
            {
                return Err(format!(
                    "decks[{i}].shelter.lantern.post: {} names no post (there are {})",
                    l.post,
                    s.posts_mm.len()
                ));
            }
        }
    }
    Ok(())
}

/// Every deck's citation, for the citations stage.
pub(crate) fn citations(decks: &[Deck]) -> Vec<(String, &String)> {
    decks
        .iter()
        .enumerate()
        .map(|(i, d)| (format!("decks[{i}] \"{}\"", d.name.as_str()), &d.source))
        .collect()
}

/// The first value outside its range, or `None`.
pub(crate) fn ranges(decks: &[Deck], heights: (f64, f64), extent_mm: i64) -> Option<String> {
    let mut out: Option<String> = None;
    let mut within = |at: String, value: f64, (low, high): (f64, f64)| {
        if out.is_none() && !(value.is_finite() && (low..=high).contains(&value)) {
            out = Some(format!("{at}: {value} is outside {low}…{high}"));
        }
    };
    for (i, d) in decks.iter().enumerate() {
        let at = |field: &str| format!("decks[{i}] \"{}\".{field}", d.name.as_str());
        let [_, _, w, h] = d.footprint_mm;
        within(at("footprint_mm"), w as f64, (1.0, extent_mm as f64));
        within(at("footprint_mm"), h as f64, (1.0, extent_mm as f64));
        within(at("surface_m"), d.surface_m, heights);
        within(at("thickness_m"), d.thickness_m, (0.05, 1.0));
        within(
            at("boards.width_mm"),
            f64::from(d.boards.width_mm),
            (50.0, 400.0),
        );
        within(at("boards.gap_mm"), f64::from(d.boards.gap_mm), (0.0, 20.0));
        if let Some(a) = &d.approach {
            let depth = match a.side {
                Side::N | Side::S => h,
                Side::E | Side::W => w,
            } as f64
                / 1000.0;
            within(at("approach.length_m"), a.length_m, (0.3, depth));
            within(at("approach.from_m"), a.from_m, heights);
            let slope = (a.from_m - d.surface_m).abs() / a.length_m.max(1e-6);
            // 1:12 itself is allowed: a millionth of slack for the division's rounding
            within(
                at("approach (rise / run)"),
                slope,
                (0.0, STEEPEST_APPROACH + 1e-6),
            );
        }
        let p = &d.piles;
        within(at("piles.bents"), f64::from(p.bents), (2.0, 64.0));
        within(at("piles.diameter_m"), p.diameter_m, (0.1, 1.0));
        within(at("piles.inset_m"), p.inset_m, (0.0, 5.0));
        within(
            at("piles.to_m"),
            p.to_m,
            (heights.0, d.surface_m - d.thickness_m),
        );
        if let Some(r) = &d.rails {
            within(at("rails.height_m"), r.height_m, (0.3, 1.5));
            within(at("rails.rail_m"), r.rail_m, (0.02, 0.2));
            within(at("rails.post_m"), r.post_m, (0.03, 0.3));
            within(at("rails.posts_m"), r.posts_m, (0.5, 5.0));
        }
        if let Some(c) = &d.curb {
            within(at("curb.height_m"), c.height_m, (0.0, 0.3));
            within(at("curb.width_m"), c.width_m, (0.02, 0.5));
        }
        if let Some(s) = &d.shelter {
            within(
                at("shelter.posts_mm (count)"),
                s.posts_mm.len() as f64,
                (1.0, MAX_SHELTER_POSTS as f64),
            );
            within(at("shelter.post_m"), s.post_m, (0.05, 1.0));
            within(at("shelter.eaves_m"), s.eaves_m, (1.8, 6.0));
            within(at("shelter.roof.pitch_deg"), s.roof.pitch_deg, (0.0, 60.0));
            within(at("shelter.roof.overhang_m"), s.roof.overhang_m, (0.0, 2.0));
            if let Some(l) = &s.lantern {
                within(at("shelter.lantern.up_m"), l.up_m, (0.5, s.eaves_m));
            }
        }
        within(
            at("moored (count)"),
            d.moored.len() as f64,
            (0.0, MAX_MOORED as f64),
        );
        for (j, m) in d.moored.iter().enumerate() {
            within(
                at(&format!("moored[{j}].length_m")),
                m.length_m,
                (2.0, 12.0),
            );
            within(at(&format!("moored[{j}].beam_m")), m.beam_m, (0.8, 4.0));
        }
    }
    out
}

/// The first `depicts` naming no place, or `None`.
pub(crate) fn references(decks: &[Deck], depicted: impl Fn(&[Key]) -> bool) -> Option<String> {
    decks.iter().enumerate().find_map(|(i, d)| {
        (!depicted(&d.depicts)).then(|| {
            format!(
                "decks[{i}] \"{}\".depicts: no place carries the tags {}",
                d.name.as_str(),
                d.depicts
                    .iter()
                    .map(Key::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
    })
}

/// Floors: the deck on its place's footprint within `tolerance` mm; every shelter post inside it; no
/// craft on any place's floor (`floors` are the places' floors shrunk by the tolerance).
pub(crate) fn floors(
    decks: &[Deck],
    place_of: impl Fn(&[Key]) -> Option<RectMm>,
    floors: &[PlaceFloor],
    tolerance: i64,
) -> Option<String> {
    for (i, d) in decks.iter().enumerate() {
        let at = format!("decks[{i}] \"{}\"", d.name.as_str());
        let f = d.footprint_mm;
        if let Some(place) = place_of(&d.depicts) {
            let off = f
                .iter()
                .zip(place)
                .map(|(a, b)| (a - b).abs())
                .max()
                .unwrap_or(0);
            if off > tolerance {
                return Some(format!(
                    "{at}: the deck stands {off} mm off its place's footprint (at most {tolerance})"
                ));
            }
        }
        if let Some(s) = &d.shelter {
            let half = (s.post_m * 500.0).round() as i64;
            for (j, p) in s.posts_mm.iter().enumerate() {
                let inside = p[0] - half >= f[0]
                    && p[0] + half <= f[0] + f[2]
                    && p[1] - half >= f[1]
                    && p[1] + half <= f[1] + f[3];
                if !inside {
                    return Some(format!(
                        "{at}.shelter.posts_mm[{j}]: the post stands off the deck"
                    ));
                }
            }
        }
        for (j, m) in d.moored.iter().enumerate() {
            let outline = hull(m);
            if let Some((tags, _)) = floors.iter().find(|(_, (x0, y0, x1, y1))| {
                // a craft's hull, by its oriented box's corners, edge midpoints and centre, on a floor
                outline
                    .iter()
                    .any(|&(px, py)| px > *x0 && px < *x1 && py > *y0 && py < *y1)
            }) {
                return Some(format!(
                    "{at}.moored[{j}] overlaps the place tagged {tags} by more than {tolerance} mm"
                ));
            }
        }
    }
    None
}

/// A craft's hull outline: its oriented box's corners and edge midpoints, plan millimetres.
fn hull(m: &Moored) -> Vec<(f64, f64)> {
    let yaw = m.yaw_deg.to_radians();
    // heading `yaw` clockwise from north: along = (sin, cos), across = (cos, −sin)
    let (along, across) = ((yaw.sin(), yaw.cos()), (yaw.cos(), -yaw.sin()));
    let (l, b) = (m.length_m * 500.0, m.beam_m * 500.0);
    let (x, y) = (m.at_mm[0] as f64, m.at_mm[1] as f64);
    let mut out = Vec::new();
    for u in [-1.0, 0.0, 1.0] {
        for v in [-1.0, 0.0, 1.0] {
            out.push((
                x + along.0 * l * u + across.0 * b * v,
                y + along.1 * l * u + across.1 * b * v,
            ));
        }
    }
    out
}
