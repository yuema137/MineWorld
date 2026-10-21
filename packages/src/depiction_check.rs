//! The depiction file's rules beyond its types (`MODULE_SPEC.md` §6.1.2), in order: shape → scope →
//! citations → ranges → polygons → references → doors → floors. The first fault is the refusal, named
//! with the field's path.

use crate::depiction::DepictionCode;
use crate::depiction_schema::{
    Area, Building, Depiction, EdgeKind, Height, OpeningState, PointMm, RectMm, RoofForm,
    ScatterArea, ScatterKind,
};

/// A refusal's code and detail, before the file's path is attached.
type Refusal = (DepictionCode, String);

fn refuse<T>(code: DepictionCode, detail: impl Into<String>) -> Result<T, Refusal> {
    Err((code, detail.into()))
}

/// How far something may reach onto a place's floor, or a shell stand off its place's footprint,
/// millimetres (I-d-4's 50 mm).
const FLOOR_TOLERANCE_MM: i64 = 50;
/// Heights, metres above the street's surface.
const HEIGHT_M: (f64, f64) = (-20.0, 60.0);
/// The largest extent of anything, millimetres.
const EXTENT_MM: i64 = 400_000;
/// The caps.
const MAX_LAYERS: usize = 4;
const MAX_SPECIES: usize = 16;
const MAX_TREES_PER_STAND: f64 = 4000.0;
const MAX_BUILDINGS: usize = 64;
const MAX_REEDS: u32 = 40;
const MAX_SCATTER: u32 = 4000;
const MAX_GI: usize = 4;
const GI_SUBDIV: [u32; 3] = [64, 128, 256];
const MAX_VERTICES: usize = 64;
/// A ramp's two lines are parallel within this, degrees.
const RAMP_PARALLEL_DEG: f64 = 1.0;

/// Parses and validates depiction `text` from a file whose stem is `stem`.
pub(crate) fn parse(
    text: &str,
    stem: &str,
    waters: Option<&[String]>,
) -> Result<Depiction, Refusal> {
    let file: Depiction = serde_saphyr::from_str(text)
        .map_err(|error| (DepictionCode::Malformed, error.to_string()))?;
    check_shape(&file)?;
    if file.scope.world.as_str() != stem {
        return refuse(
            DepictionCode::ScopeMismatch,
            format!(
                "scope.world {} belongs in {}.yaml, not {stem}.yaml",
                file.scope.world.as_str(),
                file.scope.world.as_str()
            ),
        );
    }
    check_citations(&file)?;
    check_ranges(&file)?;
    check_polygons(&file)?;
    check_references(&file, waters)?;
    check_doors(&file)?;
    check_floors(&file)?;
    Ok(file)
}

// --- shape ---------------------------------------------------------------------------------------

fn malformed<T>(detail: impl Into<String>) -> Result<T, Refusal> {
    refuse(DepictionCode::Malformed, detail)
}

fn check_shape(file: &Depiction) -> Result<(), Refusal> {
    if file.format != 1 {
        return malformed(format!("format {} is not 1, the only format", file.format));
    }
    if let Some(terrain) = &file.terrain {
        for (i, region) in terrain.regions.iter().enumerate() {
            check_height(&region.height, &format!("terrain.regions[{i}].height"))?;
        }
    }
    for (i, edge) in file.edges.iter().enumerate() {
        let at = format!("edges[{i}] \"{}\"", edge.name.as_str());
        let drop = matches!(edge.kind, EdgeKind::Quay | EdgeKind::Revetment);
        if drop != edge.drop_m.is_some() {
            return malformed(format!(
                "{at}: drop_m is stated for a quay or a revetment, and only for them"
            ));
        }
        if (edge.kind == EdgeKind::Rail) != edge.posts_m.is_some() {
            return malformed(format!(
                "{at}: posts_m is stated for a rail, and only for a rail"
            ));
        }
        line(&edge.line_mm, &format!("{at}.line_mm"))?;
    }
    for (i, path) in file.paths.iter().enumerate() {
        line(&path.line_mm, &format!("paths[{i}].line_mm"))?;
    }
    for (i, building) in file.buildings.iter().enumerate() {
        let at = format!("buildings[{i}] \"{}\"", building.name.as_str());
        if (building.roof.form == RoofForm::Mono) != building.roof.falls.is_some() {
            return malformed(format!(
                "{at}.roof: falls is stated for a mono roof, and only for it"
            ));
        }
        for (j, opening) in building.openings.iter().enumerate() {
            if opening.at_doorway == opening.offset_m.is_some() {
                return malformed(format!(
                    "{at}.openings[{j}]: exactly one of offset_m and at_doorway: true"
                ));
            }
        }
    }
    if let Some(trees) = &file.trees {
        for (i, stand) in trees.stands.iter().enumerate() {
            check_area(&stand.area, &format!("trees.stands[{i}].area"))?;
        }
        for (i, wall) in trees.air_walls.iter().enumerate() {
            match (&wall.ring, &wall.line_mm) {
                (Some(_), None) => {}
                (None, Some(points)) => line(points, &format!("trees.air_walls[{i}].line_mm"))?,
                _ => {
                    return malformed(format!(
                        "trees.air_walls[{i}]: exactly one of ring and line_mm"
                    ));
                }
            }
        }
    }
    for (i, scatter) in file.scatter.iter().enumerate() {
        let at = format!("scatter[{i}] \"{}\".area", scatter.name.as_str());
        match (
            &scatter.area.polygon_mm,
            &scatter.area.along_mm,
            scatter.area.width_m,
        ) {
            (Some(_), None, None) => {}
            (None, Some(points), Some(_)) => line(points, &at)?,
            _ => {
                return malformed(format!(
                    "{at}: exactly one of polygon_mm, or along_mm with width_m"
                ));
            }
        }
    }
    Ok(())
}

fn check_height(height: &Height, at: &str) -> Result<(), Refusal> {
    match (height.flat_m, &height.ramp) {
        (Some(_), None) => {
            if height.crown_m.is_some() != height.crown_along.is_some() {
                return malformed(format!("{at}: crown_m and crown_along are stated together"));
            }
            Ok(())
        }
        (None, Some(_)) if height.crown_m.is_none() && height.crown_along.is_none() => Ok(()),
        _ => malformed(format!(
            "{at}: exactly one of flat_m (with an optional crown) and ramp"
        )),
    }
}

fn check_area(area: &Area, at: &str) -> Result<(), Refusal> {
    match (&area.ring, &area.polygon_mm) {
        (Some(_), None) | (None, Some(_)) => Ok(()),
        _ => malformed(format!("{at}: exactly one of ring and polygon_mm")),
    }
}

fn line(points: &[PointMm], at: &str) -> Result<(), Refusal> {
    if points.len() < 2 {
        return malformed(format!("{at}: a line has at least two points"));
    }
    Ok(())
}

// --- citations -----------------------------------------------------------------------------------

fn check_citations(file: &Depiction) -> Result<(), Refusal> {
    let mut cited: Vec<(String, &String)> = Vec::new();
    if let Some(terrain) = &file.terrain {
        for (i, r) in terrain.regions.iter().enumerate() {
            cited.push((
                format!("terrain.regions[{i}] \"{}\"", r.name.as_str()),
                &r.source,
            ));
        }
        for (i, p) in terrain.profiles.iter().enumerate() {
            cited.push((
                format!("terrain.profiles[{i}] \"{}\"", p.name.as_str()),
                &p.source,
            ));
        }
        if let Some(bed) = &terrain.bed {
            cited.push(("terrain.bed".to_owned(), &bed.source));
        }
    }
    if let Some(water) = &file.water {
        cited.push(("water".to_owned(), &water.source));
    }
    for (i, e) in file.edges.iter().enumerate() {
        cited.push((format!("edges[{i}] \"{}\"", e.name.as_str()), &e.source));
    }
    for (i, b) in file.buildings.iter().enumerate() {
        cited.push((format!("buildings[{i}] \"{}\"", b.name.as_str()), &b.source));
    }
    if let Some(trees) = &file.trees {
        cited.push(("trees".to_owned(), &trees.source));
        for (i, s) in trees.species.iter().enumerate() {
            cited.push((
                format!("trees.species[{i}] \"{}\"", s.name.as_str()),
                &s.source,
            ));
        }
    }
    for (i, s) in file.scatter.iter().enumerate() {
        cited.push((format!("scatter[{i}] \"{}\"", s.name.as_str()), &s.source));
    }
    match cited
        .into_iter()
        .find(|(_, source)| source.trim().is_empty())
    {
        Some((at, _)) => refuse(
            DepictionCode::Uncited,
            format!("{at}: source is empty; cite it, or write \"approximate: <who replaces it>\""),
        ),
        None => Ok(()),
    }
}

// --- ranges --------------------------------------------------------------------------------------

/// Collects the first value outside its range.
struct Ranges(Option<String>);

impl Ranges {
    fn within(&mut self, at: impl FnOnce() -> String, value: f64, (low, high): (f64, f64)) {
        if self.0.is_none() && !(value.is_finite() && (low..=high).contains(&value)) {
            self.0 = Some(format!("{}: {value} is outside {low}…{high}", at()));
        }
    }

    fn positive(&mut self, at: impl FnOnce() -> String, value: f64, high: f64) {
        if self.0.is_none() && !(value.is_finite() && value > 0.0 && value <= high) {
            self.0 = Some(format!("{}: {value} is outside (0, {high}]", at()));
        }
    }

    fn cap(&mut self, at: &str, count: usize, cap: usize) {
        if self.0.is_none() && count > cap {
            self.0 = Some(format!("{at}: {count} is more than the cap of {cap}"));
        }
    }

    fn rect(&mut self, at: impl FnOnce() -> String, rect: RectMm) {
        let fits = |v: i64| (1..=EXTENT_MM).contains(&v);
        if self.0.is_none() && !(fits(rect[2]) && fits(rect[3])) {
            self.0 = Some(format!(
                "{}: a width and height of {} × {} mm, each outside 1…{EXTENT_MM}",
                at(),
                rect[2],
                rect[3]
            ));
        }
    }
}

fn check_ranges(file: &Depiction) -> Result<(), Refusal> {
    let mut r = Ranges(None);
    r.rect(
        || "street.footprint_mm".to_owned(),
        file.street.footprint_mm,
    );
    for (i, p) in file.places.iter().enumerate() {
        r.rect(|| format!("places[{i}].footprint_mm"), p.footprint_mm);
    }
    r.within(|| "spawn.up_m".to_owned(), file.spawn.up_m, HEIGHT_M);
    if let Some(t) = &file.terrain {
        r.rect(|| "terrain.extent_mm".to_owned(), t.extent_mm);
        r.within(|| "terrain.cell_m".to_owned(), t.cell_m, (0.5, 4.0));
        r.within(|| "terrain.blend_m".to_owned(), t.blend_m, (0.0, 50.0));
        r.cap("terrain.layers", t.layers.len(), MAX_LAYERS);
        for (i, layer) in t.layers.iter().enumerate() {
            r.positive(
                || format!("terrain.layers[{i}].repeat_m"),
                layer.repeat_m,
                100.0,
            );
        }
        for (i, region) in t.regions.iter().enumerate() {
            let h = &region.height;
            let at = || format!("terrain.regions[{i}].height");
            if let Some(flat) = h.flat_m {
                r.within(at, flat + h.crown_m.unwrap_or(0.0), HEIGHT_M);
            }
            if let Some(ramp) = &h.ramp {
                r.within(at, ramp.from_m, HEIGHT_M);
                r.within(at, ramp.to_m, HEIGHT_M);
            }
        }
        for (i, profile) in t.profiles.iter().enumerate() {
            r.within(
                || format!("terrain.profiles[{i}].bearing_deg"),
                profile.bearing_deg,
                (0.0, 360.0),
            );
            for (j, s) in profile.samples.iter().enumerate() {
                let at = || format!("terrain.profiles[{i}].samples[{j}]");
                r.within(at, s.height_m, HEIGHT_M);
                r.within(at, s.distance_m, (0.0, 400.0));
            }
        }
        if let Some(bed) = &t.bed {
            r.within(
                || "terrain.bed.depth_m".to_owned(),
                bed.depth_m,
                (0.0, 20.0),
            );
            r.positive(|| "terrain.bed.run_m".to_owned(), bed.run_m, 400.0);
        }
    }
    if let Some(w) = &file.water {
        r.within(|| "water.level_m".to_owned(), w.level_m, HEIGHT_M);
        r.positive(|| "water.extent_m".to_owned(), w.extent_m, 400.0);
        if let Some(fade) = w.shader.depth_fade_m {
            r.within(|| "water.shader.depth_fade_m".to_owned(), fade, (0.0, 20.0));
        }
        if let Some(foam) = w.shader.shore_foam {
            r.within(|| "water.shader.shore_foam".to_owned(), foam, (0.0, 1.0));
        }
        for (i, l) in w.lilies.iter().enumerate() {
            r.within(|| format!("water.lilies[{i}].cover"), l.cover, (0.0, 1.0));
        }
    }
    for (i, e) in file.edges.iter().enumerate() {
        let at = || format!("edges[{i}] \"{}\"", e.name.as_str());
        r.within(at, e.height_m, (0.0, 20.0));
        r.positive(at, e.width_m, 20.0);
        if let Some(drop) = e.drop_m {
            r.within(at, drop, (0.0, 20.0));
        }
        if let Some(posts) = e.posts_m {
            r.positive(at, posts, 20.0);
        }
    }
    for (i, p) in file.paths.iter().enumerate() {
        r.positive(|| format!("paths[{i}].width_m"), p.width_m, 20.0);
    }
    r.cap("buildings", file.buildings.len(), MAX_BUILDINGS);
    for (i, b) in file.buildings.iter().enumerate() {
        building_ranges(&mut r, i, b);
    }
    if let Some(trees) = &file.trees {
        r.cap("trees.species", trees.species.len(), MAX_SPECIES);
        for (i, s) in trees.species.iter().enumerate() {
            let at = || format!("trees.species[{i}] \"{}\".height_m", s.name.as_str());
            r.within(at, s.height_m.min, (0.3, 100.0));
            r.within(at, s.height_m.max, (s.height_m.min, 100.0));
        }
        for (i, stand) in trees.stands.iter().enumerate() {
            let at = || format!("trees.stands[{i}] \"{}\"", stand.name.as_str());
            r.within(at, stand.spacing_m, (1.0, 100.0));
            r.within(at, stand.density.from, (0.0, 1.0));
            r.within(at, stand.density.to, (0.0, 1.0));
            r.within(at, stand.density.rise_m[0], (0.0, 400.0));
            r.within(
                at,
                stand.density.rise_m[1],
                (stand.density.rise_m[0], 400.0),
            );
            for m in &stand.mix {
                r.within(at, m.weight, (0.0, 1.0e6));
            }
            if let Some(ring) = &stand.area.ring {
                r.within(at, ring.inner_m, (0.0, 400.0));
                r.within(at, ring.outer_m, (ring.inner_m, 400.0));
            }
            let trees_m2 = area_m2(&stand.area) / (stand.spacing_m * stand.spacing_m);
            if r.0.is_none() && trees_m2 > MAX_TREES_PER_STAND {
                r.0 = Some(format!(
                    "{}: about {trees_m2:.0} trees, more than the cap of {MAX_TREES_PER_STAND}",
                    at()
                ));
            }
        }
        for (i, single) in trees.singles.iter().enumerate() {
            r.within(
                || format!("trees.singles[{i}].height_m"),
                single.height_m,
                (0.3, 100.0),
            );
        }
        r.positive(|| "trees.lod_m".to_owned(), trees.lod_m, 400.0);
        r.within(
            || "trees.keep_clear.paths_m".to_owned(),
            trees.keep_clear.paths_m,
            (0.0, 50.0),
        );
        for (i, wall) in trees.air_walls.iter().enumerate() {
            if let Some(ring) = &wall.ring {
                r.positive(
                    || format!("trees.air_walls[{i}].ring.radius_m"),
                    ring.radius_m,
                    400.0,
                );
            }
        }
    }
    for (i, s) in file.scatter.iter().enumerate() {
        let at = || format!("scatter[{i}] \"{}\"", s.name.as_str());
        let cap = if s.kind == ScatterKind::Reeds {
            MAX_REEDS
        } else {
            MAX_SCATTER
        };
        r.within(at, f64::from(s.count), (1.0, f64::from(cap)));
        r.positive(at, s.size_m.min, 20.0);
        r.within(at, s.size_m.max, (s.size_m.min, 20.0));
        if let Some(width) = s.area.width_m {
            r.positive(at, width, 50.0);
        }
        if let Some(range) = s.range_m {
            r.positive(at, range, 400.0);
        }
    }
    if let Some(light) = &file.light {
        r.cap("light.gi", light.gi.len(), MAX_GI);
        for (i, gi) in light.gi.iter().enumerate() {
            r.rect(|| format!("light.gi[{i}].box_mm"), gi.box_mm);
            r.positive(|| format!("light.gi[{i}].height_m"), gi.height_m, 60.0);
            if r.0.is_none() && !GI_SUBDIV.contains(&gi.subdiv) {
                r.0 = Some(format!(
                    "light.gi[{i}].subdiv: {} is not 64, 128 or 256",
                    gi.subdiv
                ));
            }
        }
        for (i, probe) in light.probes.iter().enumerate() {
            r.rect(|| format!("light.probes[{i}].box_mm"), probe.box_mm);
            r.positive(
                || format!("light.probes[{i}].height_m"),
                probe.height_m,
                60.0,
            );
            r.within(
                || format!("light.probes[{i}].centre_up_m"),
                probe.centre_up_m,
                HEIGHT_M,
            );
        }
    }
    let v = &file.viewpoints;
    for (i, vp) in v.farfield.iter().enumerate() {
        r.within(
            || format!("viewpoints.farfield[{i}].eye_m"),
            vp.eye_m,
            HEIGHT_M,
        );
    }
    for (name, views) in [("farfield_shots", &v.farfield_shots), ("perf", &v.perf)] {
        for (i, view) in views.iter().enumerate() {
            r.within(
                || format!("viewpoints.{name}[{i}].eye_m"),
                view.eye_m,
                HEIGHT_M,
            );
            r.within(
                || format!("viewpoints.{name}[{i}].pitch_deg"),
                view.pitch_deg,
                (-90.0, 90.0),
            );
        }
    }
    match r.0 {
        Some(detail) => refuse(DepictionCode::OutOfRange, detail),
        None => Ok(()),
    }
}

fn building_ranges(r: &mut Ranges, i: usize, b: &Building) {
    let at = || format!("buildings[{i}] \"{}\"", b.name.as_str());
    r.within(at, f64::from(b.storeys), (1.0, 4.0));
    r.within(at, b.storey_m, (2.4, 4.0));
    r.within(at, b.plinth_m, (0.0, 2.0));
    r.within(at, b.roof.pitch_deg, (0.0, 60.0));
    r.within(at, b.roof.overhang_m, (0.0, 2.0));
    for o in &b.openings {
        r.within(at, f64::from(o.storey), (0.0, f64::from(b.storeys) - 1.0));
        r.positive(at, o.width_m, 10.0);
        r.positive(at, o.height_m, b.storey_m);
        if let Some(sill) = o.sill_m {
            r.within(at, sill, (0.0, b.storey_m));
        }
        if let Some(offset) = o.offset_m {
            r.within(at, offset, (0.0, 400.0));
        }
    }
    if let Some(c) = &b.chimney {
        r.positive(at, c.size_m[0], 5.0);
        r.positive(at, c.size_m[1], 5.0);
        r.within(at, c.above_ridge_m, (0.0, 10.0));
    }
}

// --- polygons ------------------------------------------------------------------------------------

fn check_polygons(file: &Depiction) -> Result<(), Refusal> {
    let mut polygons: Vec<(String, &Vec<PointMm>)> = Vec::new();
    if let Some(t) = &file.terrain {
        for (i, region) in t.regions.iter().enumerate() {
            polygons.push((
                format!("terrain.regions[{i}].polygon_mm"),
                &region.polygon_mm,
            ));
            if let Some(ramp) = &region.height.ramp {
                check_ramp(
                    ramp.from_mm,
                    ramp.to_mm,
                    &format!("terrain.regions[{i}].height.ramp"),
                )?;
            }
        }
    }
    if let Some(w) = &file.water {
        polygons.push(("water.polygon_mm".to_owned(), &w.polygon_mm));
        for (i, l) in w.lilies.iter().enumerate() {
            polygons.push((format!("water.lilies[{i}].polygon_mm"), &l.polygon_mm));
        }
    }
    for (i, b) in file.buildings.iter().enumerate() {
        polygons.push((format!("buildings[{i}].footprint_mm"), &b.footprint_mm));
    }
    if let Some(trees) = &file.trees {
        for (i, stand) in trees.stands.iter().enumerate() {
            if let Some(polygon) = &stand.area.polygon_mm {
                polygons.push((format!("trees.stands[{i}].area.polygon_mm"), polygon));
            }
        }
    }
    for (i, s) in file.scatter.iter().enumerate() {
        if let Some(polygon) = &s.area.polygon_mm {
            polygons.push((format!("scatter[{i}].area.polygon_mm"), polygon));
        }
    }
    for (at, points) in polygons {
        if points.len() < 3 || points.len() > MAX_VERTICES {
            return refuse(
                DepictionCode::Polygon,
                format!("{at}: {} vertices, outside 3…{MAX_VERTICES}", points.len()),
            );
        }
        if !simple(points) {
            return refuse(
                DepictionCode::Polygon,
                format!("{at}: the polygon is not simple"),
            );
        }
    }
    Ok(())
}

fn check_ramp(from: [PointMm; 2], to: [PointMm; 2], at: &str) -> Result<(), Refusal> {
    let direction = |l: [PointMm; 2]| {
        let (dx, dy) = ((l[1][0] - l[0][0]) as f64, (l[1][1] - l[0][1]) as f64);
        (dx != 0.0 || dy != 0.0).then_some(dy.atan2(dx))
    };
    let (Some(a), Some(b)) = (direction(from), direction(to)) else {
        return refuse(
            DepictionCode::Polygon,
            format!("{at}: a line's two points coincide"),
        );
    };
    let mut diff = (a - b).to_degrees().rem_euclid(180.0);
    if diff > 90.0 {
        diff = 180.0 - diff;
    }
    if diff > RAMP_PARALLEL_DEG {
        return refuse(
            DepictionCode::Polygon,
            format!("{at}: the lines differ by {diff:.1}°, more than {RAMP_PARALLEL_DEG}°"),
        );
    }
    Ok(())
}

fn cross(o: PointMm, a: PointMm, b: PointMm) -> i128 {
    i128::from(a[0] - o[0]) * i128::from(b[1] - o[1])
        - i128::from(a[1] - o[1]) * i128::from(b[0] - o[0])
}

fn on_segment(p: PointMm, a: PointMm, b: PointMm) -> bool {
    (a[0].min(b[0])..=a[0].max(b[0])).contains(&p[0])
        && (a[1].min(b[1])..=a[1].max(b[1])).contains(&p[1])
}

/// Whether segments a–b and c–d meet, touching included.
fn segments_meet(a: PointMm, b: PointMm, c: PointMm, d: PointMm) -> bool {
    let (d1, d2) = (cross(c, d, a), cross(c, d, b));
    let (d3, d4) = (cross(a, b, c), cross(a, b, d));
    if ((d1 > 0 && d2 < 0) || (d1 < 0 && d2 > 0)) && ((d3 > 0 && d4 < 0) || (d3 < 0 && d4 > 0)) {
        return true;
    }
    (d1 == 0 && on_segment(a, c, d))
        || (d2 == 0 && on_segment(b, c, d))
        || (d3 == 0 && on_segment(c, a, b))
        || (d4 == 0 && on_segment(d, a, b))
}

/// Whether a closed polygon is simple: no repeated vertex, no two non-adjacent edges meeting, and a
/// non-zero area.
fn simple(points: &[PointMm]) -> bool {
    let n = points.len();
    for (i, p) in points.iter().enumerate() {
        if points[i + 1..].contains(p) {
            return false;
        }
    }
    for i in 0..n {
        let (a, b) = (points[i], points[(i + 1) % n]);
        for j in (i + 1)..n {
            if j == i + 1 || (i == 0 && j == n - 1) {
                continue;
            }
            let (c, d) = (points[j], points[(j + 1) % n]);
            if segments_meet(a, b, c, d) {
                return false;
            }
        }
    }
    shoelace_mm2(points) != 0
}

fn shoelace_mm2(points: &[PointMm]) -> i128 {
    let n = points.len();
    (0..n)
        .map(|i| {
            let (a, b) = (points[i], points[(i + 1) % n]);
            i128::from(a[0]) * i128::from(b[1]) - i128::from(b[0]) * i128::from(a[1])
        })
        .sum::<i128>()
        / 2
}

fn area_m2(area: &Area) -> f64 {
    if let Some(ring) = &area.ring {
        return std::f64::consts::PI * (ring.outer_m.powi(2) - ring.inner_m.powi(2));
    }
    area.polygon_mm
        .as_deref()
        .map_or(0.0, |p| (shoelace_mm2(p).unsigned_abs() as f64) / 1.0e6)
}

// --- references ----------------------------------------------------------------------------------

fn check_references(file: &Depiction, waters: Option<&[String]>) -> Result<(), Refusal> {
    let missing = |detail: String| refuse(DepictionCode::Reference, detail);
    let layers: Vec<&str> = file
        .terrain
        .iter()
        .flat_map(|t| t.layers.iter().map(|l| l.name.as_str()))
        .collect();
    if let Some(t) = &file.terrain {
        let mut named = vec![("terrain.default_layer".to_owned(), t.default_layer.as_str())];
        for (i, region) in t.regions.iter().enumerate() {
            named.push((format!("terrain.regions[{i}].layer"), region.layer.as_str()));
        }
        if let Some((at, name)) = named.into_iter().find(|(_, n)| !layers.contains(n)) {
            return missing(format!("{at}: no terrain layer is named {name}"));
        }
    }
    for (i, path) in file.paths.iter().enumerate() {
        if !layers.contains(&path.layer.as_str()) {
            return missing(format!(
                "paths[{i}].layer: no terrain layer is named {}",
                path.layer.as_str()
            ));
        }
    }
    if let (Some(water), Some(waters)) = (&file.water, waters) {
        let name = water.far_field_water.as_str();
        if !waters.iter().any(|w| w == name) {
            return missing(format!(
                "water.far_field_water: the world's far field has no water named {name}"
            ));
        }
    }
    let tagged = |tag: &str| {
        file.places
            .iter()
            .any(|p| p.tags.iter().any(|t| t.as_str() == tag))
    };
    for (i, edge) in file.edges.iter().enumerate() {
        if let Some(tag) = edge.open_at.iter().find(|t| !tagged(t.as_str())) {
            return missing(format!(
                "edges[{i}].open_at: no place is tagged {}",
                tag.as_str()
            ));
        }
    }
    for (i, b) in file.buildings.iter().enumerate() {
        if let Some(tags) = &b.depicts
            && depicted(file, tags).is_none()
        {
            return missing(format!(
                "buildings[{i}] \"{}\".depicts: no place carries the tags {}",
                b.name.as_str(),
                joined(tags)
            ));
        }
    }
    if let Some(trees) = &file.trees {
        let species = |name: &str| trees.species.iter().any(|s| s.name.as_str() == name);
        for (i, stand) in trees.stands.iter().enumerate() {
            if let Some(m) = stand.mix.iter().find(|m| !species(m.species.as_str())) {
                return missing(format!(
                    "trees.stands[{i}].mix: no species is named {}",
                    m.species.as_str()
                ));
            }
        }
        for (i, single) in trees.singles.iter().enumerate() {
            if !species(single.species.as_str()) {
                return missing(format!(
                    "trees.singles[{i}]: no species is named {}",
                    single.species.as_str()
                ));
            }
        }
    }
    let street_or_place =
        |tag: &str| file.street.tags.iter().any(|t| t.as_str() == tag) || tagged(tag);
    for (i, step) in file.viewpoints.probe_route.iter().enumerate() {
        if let Some(tag) = step.iter().find(|t| !street_or_place(t.as_str())) {
            return missing(format!(
                "viewpoints.probe_route[{i}]: neither the street nor any place is tagged {}",
                tag.as_str()
            ));
        }
    }
    for name in &file.viewpoints.perf_breakdown {
        if !file.viewpoints.perf.iter().any(|v| &v.name == name) {
            return missing(format!(
                "viewpoints.perf_breakdown: no perf view is named {name}"
            ));
        }
    }
    Ok(())
}

fn joined(tags: &[crate::depiction_schema::Key]) -> String {
    tags.iter()
        .map(|t| t.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// The first place carrying every tag of `tags`.
fn depicted<'a>(
    file: &'a Depiction,
    tags: &[crate::depiction_schema::Key],
) -> Option<&'a crate::depiction_schema::PlaceBinding> {
    file.places
        .iter()
        .find(|p| tags.iter().all(|t| p.tags.iter().any(|pt| pt == t)))
}

// --- doors ---------------------------------------------------------------------------------------

fn check_doors(file: &Depiction) -> Result<(), Refusal> {
    for (i, b) in file.buildings.iter().enumerate() {
        for (j, o) in b.openings.iter().enumerate() {
            let at = format!("buildings[{i}] \"{}\".openings[{j}]", b.name.as_str());
            if o.at_doorway && b.depicts.is_none() {
                return refuse(
                    DepictionCode::Door,
                    format!("{at}: at_doorway on a building that depicts no place"),
                );
            }
            if o.state == OpeningState::Open && !(o.at_doorway && b.depicts.is_some()) {
                return refuse(
                    DepictionCode::Door,
                    format!("{at}: only a door at the doorway of a depicted place may be open"),
                );
            }
        }
    }
    Ok(())
}

// --- floors --------------------------------------------------------------------------------------

/// A place's floor shrunk by the tolerance, as (x0, y0, x1, y1) in millimetres; `None` when nothing is
/// left.
fn inner(rect: RectMm) -> Option<(f64, f64, f64, f64)> {
    let t = FLOOR_TOLERANCE_MM;
    let (x0, y0, x1, y1) = (
        rect[0] + t,
        rect[1] + t,
        rect[0] + rect[2] - t,
        rect[1] + rect[3] - t,
    );
    (x1 > x0 && y1 > y0).then_some((x0 as f64, y0 as f64, x1 as f64, y1 as f64))
}

fn check_floors(file: &Depiction) -> Result<(), Refusal> {
    let places: Vec<(String, (f64, f64, f64, f64))> = file
        .places
        .iter()
        .filter_map(|p| inner(p.footprint_mm).map(|r| (joined(&p.tags), r)))
        .collect();
    let on_floor = |at: String, overlaps: &dyn Fn((f64, f64, f64, f64)) -> bool| match places
        .iter()
        .find(|(_, r)| overlaps(*r))
    {
        Some((tags, _)) => refuse(
            DepictionCode::OnFloor,
            format!("{at} overlaps the place tagged {tags} by more than {FLOOR_TOLERANCE_MM} mm"),
        ),
        None => Ok(()),
    };
    if let Some(trees) = &file.trees {
        for (i, stand) in trees.stands.iter().enumerate() {
            let at = format!("trees.stands[{i}] \"{}\"", stand.name.as_str());
            match (&stand.area.ring, &stand.area.polygon_mm) {
                (Some(ring), _) => on_floor(at, &|r| ring_meets(ring, r))?,
                (None, Some(polygon)) => on_floor(at, &|r| polygon_meets(polygon, r))?,
                (None, None) => {}
            }
        }
    }
    for (i, s) in file.scatter.iter().enumerate() {
        let at = format!("scatter[{i}] \"{}\"", s.name.as_str());
        on_floor(at, &|r| scatter_meets(&s.area, r))?;
    }
    for (i, b) in file.buildings.iter().enumerate() {
        let at = format!("buildings[{i}] \"{}\"", b.name.as_str());
        match &b.depicts {
            None => on_floor(at, &|r| polygon_meets(&b.footprint_mm, r))?,
            Some(tags) => {
                if let Some(place) = depicted(file, tags) {
                    shell_on_footprint(&at, &b.footprint_mm, place.footprint_mm)?;
                }
            }
        }
    }
    Ok(())
}

fn shell_on_footprint(at: &str, footprint: &[PointMm], rect: RectMm) -> Result<(), Refusal> {
    let xs = footprint.iter().map(|p| p[0]);
    let ys = footprint.iter().map(|p| p[1]);
    let bounds = [
        xs.clone().min().unwrap_or(0),
        ys.clone().min().unwrap_or(0),
        xs.max().unwrap_or(0),
        ys.max().unwrap_or(0),
    ];
    let place = [rect[0], rect[1], rect[0] + rect[2], rect[1] + rect[3]];
    let off = bounds
        .iter()
        .zip(place)
        .map(|(a, b)| (a - b).abs())
        .max()
        .unwrap_or(0);
    if off > FLOOR_TOLERANCE_MM {
        return refuse(
            DepictionCode::OnFloor,
            format!(
                "{at}: the shell stands {off} mm off its place's footprint (at most {FLOOR_TOLERANCE_MM})"
            ),
        );
    }
    Ok(())
}

fn ring_meets(
    ring: &crate::depiction_schema::Ring,
    (x0, y0, x1, y1): (f64, f64, f64, f64),
) -> bool {
    let (cx, cy) = (ring.centre_mm[0] as f64, ring.centre_mm[1] as f64);
    let nx = cx.clamp(x0, x1) - cx;
    let ny = cy.clamp(y0, y1) - cy;
    let near = nx.hypot(ny);
    let far = (cx - x0)
        .abs()
        .max((cx - x1).abs())
        .hypot((cy - y0).abs().max((cy - y1).abs()));
    near < ring.outer_m * 1000.0 && far > ring.inner_m * 1000.0
}

/// Whether segment a–b passes through the open rectangle (Liang–Barsky clipping).
fn segment_enters(a: PointMm, b: PointMm, (x0, y0, x1, y1): (f64, f64, f64, f64)) -> bool {
    let (ax, ay) = (a[0] as f64, a[1] as f64);
    let (dx, dy) = (b[0] as f64 - ax, b[1] as f64 - ay);
    let (mut t0, mut t1) = (0.0_f64, 1.0_f64);
    for (p, q) in [(-dx, ax - x0), (dx, x1 - ax), (-dy, ay - y0), (dy, y1 - ay)] {
        if p == 0.0 {
            if q <= 0.0 {
                return false;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                t0 = t0.max(t);
            } else {
                t1 = t1.min(t);
            }
        }
    }
    if t1 <= t0 {
        return false;
    }
    let tm = (t0 + t1) / 2.0;
    let (mx, my) = (ax + dx * tm, ay + dy * tm);
    mx > x0 && mx < x1 && my > y0 && my < y1
}

fn inside(points: &[PointMm], (px, py): (f64, f64)) -> bool {
    let n = points.len();
    let mut odd = false;
    for i in 0..n {
        let (a, b) = (points[i], points[(i + 1) % n]);
        let (ax, ay, bx, by) = (a[0] as f64, a[1] as f64, b[0] as f64, b[1] as f64);
        if (ay > py) != (by > py) && px < ax + (py - ay) * (bx - ax) / (by - ay) {
            odd = !odd;
        }
    }
    odd
}

fn polygon_meets(points: &[PointMm], r: (f64, f64, f64, f64)) -> bool {
    let n = points.len();
    (0..n).any(|i| segment_enters(points[i], points[(i + 1) % n], r))
        || inside(points, ((r.0 + r.2) / 2.0, (r.1 + r.3) / 2.0))
}

fn distance_to_segment((px, py): (f64, f64), a: PointMm, b: PointMm) -> f64 {
    let (ax, ay) = (a[0] as f64, a[1] as f64);
    let (dx, dy) = (b[0] as f64 - ax, b[1] as f64 - ay);
    let length2 = dx * dx + dy * dy;
    let t = if length2 == 0.0 {
        0.0
    } else {
        (((px - ax) * dx + (py - ay) * dy) / length2).clamp(0.0, 1.0)
    };
    (px - ax - t * dx).hypot(py - ay - t * dy)
}

fn scatter_meets(area: &ScatterArea, r: (f64, f64, f64, f64)) -> bool {
    match (&area.polygon_mm, &area.along_mm, area.width_m) {
        (Some(polygon), _, _) => polygon_meets(polygon, r),
        (None, Some(line), Some(width)) => {
            let half = width * 500.0;
            let (x0, y0, x1, y1) = r;
            let corners = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)];
            line.windows(2).any(|w| {
                if segment_enters(w[0], w[1], r) {
                    return true;
                }
                let to_rect = |p: PointMm| {
                    let (px, py) = (p[0] as f64, p[1] as f64);
                    (px.clamp(x0, x1) - px).hypot(py.clamp(y0, y1) - py)
                };
                let nearest = to_rect(w[0]).min(to_rect(w[1])).min(
                    corners
                        .iter()
                        .map(|c| distance_to_segment(*c, w[0], w[1]))
                        .fold(f64::MAX, f64::min),
                );
                nearest < half
            })
        }
        _ => false,
    }
}
