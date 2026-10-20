//! The far field's rules beyond its types, in the order both validators keep (`MODULE_SPEC.md`
//! §6.1.1): shape → scope → sections → citations → ranges → boundary → profiles → coverage.

use crate::far_field::{
    Dimension, FarFieldCode, REGION_PREFIX, SLOT_MARK, SLOT_TOWN_MARK, uncovered,
};
use crate::far_field_schema::{
    FarField, Horizon, Scope, Sector, SkyKind, Slot, WaterKind, clockwise,
};

/// A refusal's code and detail, before the file's path is attached.
type Refusal = (FarFieldCode, String);

fn refuse<T>(code: FarFieldCode, detail: impl Into<String>) -> Result<T, Refusal> {
    Err((code, detail.into()))
}

/// Parses and validates far-field `text` from a file whose stem is `stem`.
pub(crate) fn parse(text: &str, stem: &str, dimensions: &[Dimension]) -> Result<FarField, Refusal> {
    let field: FarField = serde_saphyr::from_str(text).map_err(|error| {
        let message = error.to_string();
        if let Some(at) = message.find(SLOT_TOWN_MARK) {
            return (
                FarFieldCode::SlotTown,
                message[at + SLOT_TOWN_MARK.len()..].to_owned(),
            );
        }
        match message.find(SLOT_MARK) {
            Some(at) => (
                FarFieldCode::Slot,
                message[at + SLOT_MARK.len()..].to_owned(),
            ),
            None => (FarFieldCode::Malformed, message),
        }
    })?;
    check_shape(&field)?;
    check_scope(&field.scope, stem)?;
    check_sections(&field, dimensions)?;
    check_citations(&field)?;
    check_ranges(&field)?;
    check_boundary(&field)?;
    check_profiles(&field)?;
    check_coverage(&field)?;
    Ok(field)
}

/// The conditional fields serde cannot express: the format, the sky's kind, a river's course.
fn check_shape(field: &FarField) -> Result<(), Refusal> {
    use FarFieldCode::Malformed;
    if field.format != 1 {
        return refuse(
            Malformed,
            format!("format {} is not 1, the only format", field.format),
        );
    }
    if let Some(sky) = &field.sky {
        match (sky.kind, &sky.panorama, &sky.procedural) {
            (SkyKind::Panorama, Some(_), None) | (SkyKind::Procedural, None, Some(_)) => {}
            _ => {
                return refuse(
                    Malformed,
                    "sky: a panorama sky states `panorama` only, a procedural sky `procedural` only",
                );
            }
        }
    }
    for water in &field.water {
        let river = water.kind == WaterKind::River;
        let stated = (water.width_m.is_some(), water.course.is_some());
        if (river && stated != (true, true)) || (!river && stated != (false, false)) {
            return refuse(
                Malformed,
                format!(
                    "water {}: width_m and course are stated for a river, and only for a river",
                    water.name.as_str()
                ),
            );
        }
        if water.course.as_ref().is_some_and(|c| c.len() < 2) {
            return refuse(
                Malformed,
                format!(
                    "water {}: a course has at least two points",
                    water.name.as_str()
                ),
            );
        }
    }
    Ok(())
}

fn check_scope(scope: &Scope, stem: &str) -> Result<(), Refusal> {
    let expected = match scope {
        Scope::World(key) => key.as_str().to_owned(),
        Scope::Region(key) => format!("{REGION_PREFIX}{}", key.as_str()),
    };
    if expected == stem {
        Ok(())
    } else {
        refuse(
            FarFieldCode::ScopeMismatch,
            format!("scope {scope:?} belongs in {expected}.yaml, not {stem}.yaml"),
        )
    }
}

fn check_sections(field: &FarField, dimensions: &[Dimension]) -> Result<(), Refusal> {
    let three = [
        ("datum_m", field.datum_m.is_some()),
        ("datum_source", field.datum_source.is_some()),
        ("sky", field.sky.is_some()),
        ("foreground", field.foreground.is_some()),
        (
            "terrain_silhouettes",
            field
                .terrain_silhouettes
                .as_ref()
                .is_some_and(|s| !s.is_empty()),
        ),
        (
            "skyline",
            field
                .skyline
                .as_ref()
                .is_some_and(|s| !s.sectors.is_empty()),
        ),
    ];
    let two = [("surround", field.surround.is_some())];
    let has = |d| dimensions.contains(&d);
    for (dimension, sections) in [(Dimension::ThreeD, &three[..]), (Dimension::TwoD, &two[..])] {
        if has(dimension)
            && let Some((name, _)) = sections.iter().find(|(_, present)| !present)
        {
            return refuse(
                FarFieldCode::SectionMissing,
                format!("{name} is required of a {} pack", word(dimension)),
            );
        }
    }
    let three_stated = [
        ("datum_m", field.datum_m.is_some()),
        ("datum_source", field.datum_source.is_some()),
        ("sky", field.sky.is_some()),
        ("foreground", field.foreground.is_some()),
        ("terrain_silhouettes", field.terrain_silhouettes.is_some()),
        ("skyline", field.skyline.is_some()),
    ];
    for (dimension, sections) in [
        (Dimension::ThreeD, &three_stated[..]),
        (Dimension::TwoD, &two[..]),
    ] {
        if !has(dimension)
            && let Some((name, _)) = sections.iter().find(|(_, present)| *present)
        {
            return refuse(
                FarFieldCode::SectionForeign,
                format!("{name} belongs to a {} pack", word(dimension)),
            );
        }
    }
    Ok(())
}

const fn word(dimension: Dimension) -> &'static str {
    match dimension {
        Dimension::TwoD => "2d",
        Dimension::ThreeD => "3d",
    }
}

fn check_citations(field: &FarField) -> Result<(), Refusal> {
    let mut sources: Vec<(String, &str)> = Vec::new();
    if let Some(text) = &field.datum_source {
        sources.push(("datum_source".to_owned(), text));
    }
    if let Some(sky) = &field.sky {
        sources.push(("sky.visibility_source".to_owned(), &sky.visibility_source));
    }
    for band in field.terrain_silhouettes.iter().flatten() {
        sources.push((format!("silhouette {}", band.name.as_str()), &band.source));
    }
    for water in &field.water {
        sources.push((format!("water {}", water.name.as_str()), &water.source));
    }
    if let Some(skyline) = &field.skyline {
        for (index, sector) in skyline.sectors.iter().enumerate() {
            sources.push((format!("skyline sector {index}"), &sector.source));
        }
        for (name, slot) in [
            ("station", &skyline.slots.station),
            ("landmark", &skyline.slots.landmark),
        ] {
            if let Slot::Reserved { source, .. } | Slot::Filled { source, .. } = slot {
                sources.push((format!("slot {name}"), source));
            }
        }
    }
    match sources.iter().find(|(_, text)| text.trim().is_empty()) {
        Some((what, _)) => refuse(
            FarFieldCode::Uncited,
            format!("{what}: the source is empty"),
        ),
        None => Ok(()),
    }
}

/// Every number against its range, in the order of the file.
fn check_ranges(field: &FarField) -> Result<(), Refusal> {
    let mut ranges = Ranges::default();
    ranges.above("boundary.reach_m", field.boundary.reach_m, 0.0);
    for water in &field.water {
        let name = water.name.as_str();
        ranges.sector(&format!("water {name}"), water.sector);
        ranges.within(
            &format!("water {name} level_m"),
            water.level_m,
            -430.0,
            5000.0,
        );
        if let Some(width) = water.width_m {
            ranges.within(&format!("water {name} width_m"), width, 1.0, 5000.0);
        }
        for point in water.course.iter().flatten() {
            ranges.bearing(&format!("water {name} course"), point.bearing_deg);
        }
    }
    if let Some(datum) = field.datum_m {
        ranges.within("datum_m", datum, -430.0, 5000.0);
    }
    if let Some(sky) = &field.sky {
        ranges.within("sky.visibility_m", sky.visibility_m, 1000.0, 300_000.0);
    }
    if let Some(ground) = &field.foreground {
        ranges.within("foreground.drop_m", ground.drop_m, 0.0, 500.0);
        ranges.within("foreground.run_m", ground.run_m, 10.0, 2000.0);
    }
    for band in field.terrain_silhouettes.iter().flatten() {
        let name = format!("silhouette {}", band.name.as_str());
        ranges.sector(&name, band.sector);
        for point in &band.profile {
            ranges.bearing(&name, point.bearing_deg);
            ranges.within(&format!("{name} height_m"), point.height_m, -430.0, 9000.0);
        }
    }
    if let Some(skyline) = &field.skyline {
        for (index, sector) in skyline.sectors.iter().enumerate() {
            let name = format!("skyline sector {index}");
            ranges.sector(&name, sector.sector);
            if sector.to_m <= sector.from_m {
                ranges.fail(format!(
                    "{name}: to_m {} is not beyond from_m {}",
                    sector.to_m, sector.from_m
                ));
            }
            let h = sector.height_m;
            let ordered = h.min > 0.0 && h.min <= h.max && h.max <= 400.0;
            if !ordered {
                ranges.fail(format!(
                    "{name}: height_m {}–{} is not 0 < min ≤ max ≤ 400",
                    h.min, h.max
                ));
            }
            let density = sector.density > 0.0 && sector.density <= 1.0;
            if !density {
                ranges.fail(format!(
                    "{name}: density {} is not in (0, 1]",
                    sector.density
                ));
            }
        }
        for (name, slot) in [
            ("station", &skyline.slots.station),
            ("landmark", &skyline.slots.landmark),
        ] {
            if let Slot::Reserved {
                bearing_deg,
                footprint_m,
                ..
            }
            | Slot::Filled {
                bearing_deg,
                footprint_m,
                ..
            } = slot
            {
                ranges.bearing(&format!("slot {name}"), *bearing_deg);
                ranges.above(&format!("slot {name} footprint_m"), *footprint_m, 0.0);
            }
            if let Slot::Filled { distance_m, .. } = slot {
                ranges.above(&format!("slot {name} distance_m"), *distance_m, 0.0);
            }
        }
    }
    if let Some(surround) = &field.surround {
        let haze = &surround.haze;
        let ordered = haze.from_m >= 0.0 && haze.to_m > haze.from_m;
        if !ordered {
            ranges.fail(format!(
                "surround.haze: from_m {} and to_m {} are not 0 ≤ from_m < to_m",
                haze.from_m, haze.to_m
            ));
        }
    }
    match ranges.first {
        Some(detail) => refuse(FarFieldCode::OutOfRange, detail),
        None => Ok(()),
    }
}

/// The first out-of-range number found.
#[derive(Default)]
struct Ranges {
    first: Option<String>,
}

impl Ranges {
    fn fail(&mut self, detail: String) {
        self.first.get_or_insert(detail);
    }

    fn within(&mut self, what: &str, value: f64, low: f64, high: f64) {
        if !(low..=high).contains(&value) {
            self.fail(format!("{what} {value} is not in [{low}, {high}]"));
        }
    }

    fn above(&mut self, what: &str, value: f64, low: f64) {
        if value.is_nan() || value <= low {
            self.fail(format!("{what} {value} is not > {low}"));
        }
    }

    fn bearing(&mut self, what: &str, value: f64) {
        if !(0.0..360.0).contains(&value) {
            self.fail(format!("{what}: bearing {value} is not in [0, 360)"));
        }
    }

    fn sector(&mut self, what: &str, sector: Sector) {
        self.bearing(what, sector.from_deg);
        self.bearing(what, sector.to_deg);
        if sector.from_deg == sector.to_deg {
            self.fail(format!(
                "{what}: a sector from {0} to {0} is empty; use two sectors for a full ring",
                sector.from_deg
            ));
        }
    }
}

fn check_boundary(field: &FarField) -> Result<(), Refusal> {
    let reach = field.boundary.reach_m;
    let mut distances: Vec<(String, f64)> = Vec::new();
    for band in field.terrain_silhouettes.iter().flatten() {
        for point in &band.profile {
            distances.push((
                format!("silhouette {}", band.name.as_str()),
                point.distance_m,
            ));
        }
    }
    for water in &field.water {
        distances.push((
            format!("water {} shore_m", water.name.as_str()),
            water.shore_m,
        ));
    }
    if let Some(skyline) = &field.skyline {
        for (index, sector) in skyline.sectors.iter().enumerate() {
            distances.push((format!("skyline sector {index} from_m"), sector.from_m));
        }
        for (name, slot) in [
            ("station", &skyline.slots.station),
            ("landmark", &skyline.slots.landmark),
        ] {
            // A filled slot's building is a reachable Place: the one entry allowed inside the boundary.
            if let Slot::Reserved { distance_m, .. } = slot {
                distances.push((format!("slot {name} distance_m"), *distance_m));
            }
        }
    }
    match distances.iter().find(|(_, distance)| *distance <= reach) {
        Some((what, distance)) => refuse(
            FarFieldCode::InsideBoundary,
            format!("{what} {distance} is not beyond boundary.reach_m {reach}"),
        ),
        None => Ok(()),
    }
}

fn check_profiles(field: &FarField) -> Result<(), Refusal> {
    for band in field.terrain_silhouettes.iter().flatten() {
        let name = band.name.as_str();
        let fail = |why: &str| refuse(FarFieldCode::Profile, format!("silhouette {name}: {why}"));
        let span = band.sector.span();
        if band.profile.len() < 2 {
            return fail("a profile has at least two points");
        }
        let offsets: Vec<f64> = band
            .profile
            .iter()
            .map(|p| clockwise(band.sector.from_deg, p.bearing_deg))
            .collect();
        if offsets.iter().any(|offset| *offset > span) {
            return fail("a profile point lies outside the sector");
        }
        if offsets.windows(2).any(|pair| pair[1] <= pair[0]) {
            return fail("bearings do not increase strictly along the sector");
        }
        let last = offsets[offsets.len() - 1];
        if offsets[0] != 0.0 || last != span {
            return fail("the profile does not start at from_deg and end at to_deg");
        }
    }
    Ok(())
}

fn check_coverage(field: &FarField) -> Result<(), Refusal> {
    if let Some(surround) = &field.surround
        && (surround.extent_m.is_nan() || surround.extent_m <= 0.0)
    {
        return refuse(
            FarFieldCode::Gap,
            format!("surround.extent_m {} is not positive", surround.extent_m),
        );
    }
    if field.terrain_silhouettes.is_none() && field.skyline.is_none() {
        return Ok(());
    }
    let sectors: Vec<Sector> = field
        .terrain_silhouettes
        .iter()
        .flatten()
        .map(|band| band.sector)
        .chain(
            field
                .skyline
                .iter()
                .flat_map(|s| s.sectors.iter().map(|m| m.sector)),
        )
        .chain(
            field
                .water
                .iter()
                .filter(|water| water.horizon == Horizon::Open)
                .map(|water| water.sector),
        )
        .collect();
    match uncovered(&sectors) {
        Some((from, to)) => refuse(
            FarFieldCode::Gap,
            format!("the far field leaves [{from:.1}, {to:.1}) open"),
        ),
        None => Ok(()),
    }
}
