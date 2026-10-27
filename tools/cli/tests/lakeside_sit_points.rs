//! Lakeside's sit points, fixed once in presentation data (S33-f, `pr-s33f-interiors-sit-points.md`
//! §5.2, Q-S33f-1 (A); `ARC-145`), checked against the world they will be copied into (S34-c).
//!
//! ```text
//! (AC-f-1) loads   the table, written as `sit-points:` sections into a scratch copy of Lakeside with
//!                  `posture` enabled, is read and loaded by the real loader: posture's own checks
//!                  (keys, spacing >= 510 mm) refuse nothing
//! (I-f-2)  clear   every point stands on its place's floor, >= R + GAP (260 mm) from every floor edge
//!                  and every solid, as a doorway point does (SD-S34-3); and the pelvis the depiction
//!                  draws (`at` moved by `seat_offset_m` against the facing) is over its prop's solid
//! (AC-f-5) one     the depiction file's fallback `at` / `facing` equal §5.2's table, value for value
//! ```
//!
//! The oracles are the world's own place files (`bodies/mod.rs`'s independent reading) and §5.2's
//! values restated here; never a value the client or the System computes.

mod bodies;
mod headless;
#[allow(dead_code)]
mod lakeside;

use std::collections::BTreeMap;
use std::path::Path;

use bodies::{GAP, R, rooms_of};
use lakeside::{edit, lakeside, read_lf, read_world, repository, scratch_lakeside};
use mineworld_contracts::WorldTime;
use serde_json::Value;

/// The presentation file under test.
const FILE: &str = "presentation/mineworld-default/3D/sit_points/lakeside.yaml";

/// §5.2's table as revised (revision 2: the long table's points behind its chairs, Q-S34c-2):
/// (place tag, key, x, y, facing). The halt's 13 rows are S33-e2's, fixed on the world's merged benches
/// (`pr-s33e-pier-halt.md` §15): the waiting room's bench row along its south wall, (1 500, 0)–(6 500, 450),
/// faces north, so its seven points stand 260 mm north of it; the platform's two benches face east.
const TABLE: [(&str, &str, i32, i32, u32); 29] = [
    ("pier", "pier-bench.1", 800, 1010, 0),
    ("pier", "pier-bench.2", 1400, 1010, 0),
    ("pier", "pier-bench.3", 2000, 1010, 0),
    ("pier", "pier-bench.4", 2600, 1010, 0),
    ("pier", "pier-bench.5", 3200, 1010, 0),
    ("square", "tree-bench-north.1", 11200, 10260, 0),
    ("square", "tree-bench-north.2", 11800, 10260, 0),
    ("square", "tree-bench-north.3", 12400, 10260, 0),
    ("square", "tree-bench-east.1", 13260, 8400, 90_000),
    ("square", "tree-bench-east.2", 13260, 9000, 90_000),
    ("square", "tree-bench-east.3", 13260, 9600, 90_000),
    ("bakery", "long-table.1", 1660, 1800, 270_000),
    ("bakery", "long-table.2", 1660, 2400, 270_000),
    ("bakery", "long-table.3", 1660, 3000, 270_000),
    ("bakery", "long-table.4", 1660, 3600, 270_000),
    ("bakery", "long-table.5", 1660, 4200, 270_000),
    ("station", "wall-bench.1", 2200, 710, 0),
    ("station", "wall-bench.2", 2800, 710, 0),
    ("station", "wall-bench.3", 3400, 710, 0),
    ("station", "wall-bench.4", 4000, 710, 0),
    ("station", "wall-bench.5", 4600, 710, 0),
    ("station", "wall-bench.6", 5200, 710, 0),
    ("station", "wall-bench.7", 5800, 710, 0),
    ("platform", "platform-bench-s.1", -3240, -5600, 90_000),
    ("platform", "platform-bench-s.2", -3240, -5000, 90_000),
    ("platform", "platform-bench-s.3", -3240, -4400, 90_000),
    ("platform", "platform-bench-n.1", -3240, 4400, 90_000),
    ("platform", "platform-bench-n.2", -3240, 5000, 90_000),
    ("platform", "platform-bench-n.3", -3240, 5600, 90_000),
];

/// One point as the depiction file states it.
struct Point {
    tag: String,
    /// The place tags the point's place must not carry (`unless`).
    unless: Vec<String>,
    key: String,
    group: String,
    prop: String,
    offset_mm: i32,
    x: i32,
    y: i32,
    facing: u32,
}

fn points() -> Vec<Point> {
    let text = read_lf(&repository(FILE));
    let file: Value =
        serde_json::from_str(&text).expect("the depiction file is YAML's JSON subset");
    assert_eq!(file["format"], 1);
    assert_eq!(file["scope"]["world"], "lakeside");
    file["points"]
        .as_array()
        .expect("points")
        .iter()
        .map(|p| {
            let tags = p["tags"].as_array().expect("tags");
            assert_eq!(tags.len(), 1, "one tag names each point's place: {p}");
            let at = &p["fallback"]["at"];
            Point {
                tag: tags[0].as_str().expect("a tag").to_owned(),
                unless: p["unless"]
                    .as_array()
                    .map(|u| {
                        u.iter()
                            .map(|t| t.as_str().expect("a tag").to_owned())
                            .collect()
                    })
                    .unwrap_or_default(),
                key: p["point"].as_str().expect("a key").to_owned(),
                group: p["group"].as_str().expect("a group").to_owned(),
                prop: p["prop"].as_str().expect("a prop").to_owned(),
                offset_mm: (p["seat_offset_m"].as_f64().expect("an offset") * 1000.0).round()
                    as i32,
                x: i32::try_from(at["x"].as_i64().expect("x")).expect("x fits"),
                y: i32::try_from(at["y"].as_i64().expect("y")).expect("y fits"),
                facing: u32::try_from(p["fallback"]["facing"].as_u64().expect("facing"))
                    .expect("facing fits"),
            }
        })
        .collect()
}

/// The place key whose file lists `tag` under `tags:` and none of `unless`; exactly one.
fn place_of(world: &Path, tag: &str, unless: &[String]) -> String {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(world.join("places")).expect("places/ lists") {
        let path = entry.expect("an entry").path();
        let text = read_lf(&path);
        let tags: Vec<&str> = text
            .lines()
            .skip_while(|line| *line != "tags:")
            .skip(1)
            .take_while(|line| line.starts_with("  - "))
            .map(|line| line.trim_start_matches("  - ").trim())
            .collect();
        if tags.contains(&tag) && !unless.iter().any(|u| tags.contains(&u.as_str())) {
            found.push(
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .expect("a key")
                    .to_owned(),
            );
        }
    }
    assert_eq!(
        found.len(),
        1,
        "one place carries the tag {tag} and none of {unless:?}: {found:?}"
    );
    found.remove(0)
}

/// `sit-points:` sections, by place key, in the place-file form S34-c will write.
fn sections(world: &Path, points: &[Point]) -> BTreeMap<String, String> {
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    for p in points {
        let section = out
            .entry(place_of(world, &p.tag, &p.unless))
            .or_insert_with(|| "\nsit-points:\n".to_owned());
        section.push_str(&format!(
            "  - {{ key: {}, group: {}, at: {{ x: {}, y: {} }}, facing: {} }}\n",
            p.key, p.group, p.x, p.y, p.facing
        ));
    }
    out
}

/// A scratch copy of Lakeside with `posture` enabled and the table written into its place files.
fn with_posture(name: &str, points: &[Point]) -> headless::Scratch {
    let world = scratch_lakeside(name);
    edit(
        &world.join("world.yaml"),
        "  - appearance\n",
        "  - appearance\n  - posture\n",
    );
    for (place, section) in sections(&world, points) {
        let file = world.join("places").join(format!("{place}.yaml"));
        let mut text = read_lf(&file);
        text.push_str(&section);
        std::fs::write(&file, text).expect("writes");
    }
    world
}

#[test]
fn the_table_loads_under_posture_s_checks() {
    let points = points();
    let world = with_posture("sit-points", &points);
    let loaded = read_world(&world).load(WorldTime::EPOCH);
    assert!(
        loaded.is_ok(),
        "Lakeside with posture and the sit-point table loads: {:?}",
        loaded.err()
    );
}

/// The distance, mm, from (x, y) to the rectangle (x0, y0, x1, y1); 0 inside it.
fn distance_to(rect: (i32, i32, i32, i32), x: i32, y: i32) -> f64 {
    let dx = f64::from((rect.0 - x).max(0).max(x - rect.2));
    let dy = f64::from((rect.1 - y).max(0).max(y - rect.3));
    dx.hypot(dy)
}

/// The furnishing keys named by each solid of a place file, in the solids' order (`None`: a shell).
fn solid_names(world: &Path, place: &str) -> Vec<Option<String>> {
    let text = read_lf(&world.join("places").join(format!("{place}.yaml")));
    text.lines()
        .skip_while(|line| line.trim() != "solids:")
        .skip(1)
        .take_while(|line| line.starts_with(' '))
        .filter(|line| line.trim_start().starts_with("- {"))
        .map(|line| {
            line.split("furnishing:")
                .nth(1)
                .map(|rest| rest.trim().trim_end_matches('}').trim().to_owned())
        })
        .collect()
}

#[test]
fn every_point_stands_clear_and_its_pelvis_is_over_its_seat() {
    let world = lakeside();
    let rooms = rooms_of(&world);
    let clearance = f64::from(R + GAP);
    let mut faults = Vec::new();
    for p in points() {
        let place = place_of(&world, &p.tag, &p.unless);
        let room = &rooms[&place];
        let (x0, y0, x1, y1) = room.floor;
        let edge = (p.x - x0).min(x1 - p.x).min(p.y - y0).min(y1 - p.y);
        if f64::from(edge) < clearance {
            faults.push(format!(
                "{place}/{}: {edge} mm from the floor's edge",
                p.key
            ));
        }
        for (rect, _) in &room.solids {
            let d = distance_to(*rect, p.x, p.y);
            if d < clearance {
                faults.push(format!(
                    "{place}/{}: {d:.0} mm from the solid {rect:?}",
                    p.key
                ));
            }
        }
        // the pelvis: `at` moved by the offset against the facing (yaw from +y toward +x)
        let yaw = f64::from(p.facing) / 1000.0_f64;
        let (fx, fy) = (yaw.to_radians().sin(), yaw.to_radians().cos());
        let px = f64::from(p.x) - fx * f64::from(p.offset_mm);
        let py = f64::from(p.y) - fy * f64::from(p.offset_mm);
        let names = solid_names(&world, &place);
        assert_eq!(names.len(), room.solids.len(), "{place}: every solid read");
        let seat: Vec<_> = room
            .solids
            .iter()
            .zip(&names)
            .filter(|(_, name)| name.as_deref() == Some(p.prop.as_str()))
            .map(|((rect, _), _)| *rect)
            .collect();
        let over = seat.iter().any(|r| {
            px >= f64::from(r.0)
                && px <= f64::from(r.2)
                && py >= f64::from(r.1)
                && py <= f64::from(r.3)
        });
        if !over {
            faults.push(format!(
                "{place}/{}: the pelvis ({px:.0}, {py:.0}) is over no solid of {} ({seat:?})",
                p.key, p.prop
            ));
        }
    }
    assert!(faults.is_empty(), "I-f-2:\n{}", faults.join("\n"));
}

#[test]
fn the_fallback_positions_equal_the_design_table() {
    let got: Vec<(String, String, i32, i32, u32)> = points()
        .into_iter()
        .map(|p| (p.tag, p.key, p.x, p.y, p.facing))
        .collect();
    let want: Vec<(String, String, i32, i32, u32)> = TABLE
        .iter()
        .map(|(t, k, x, y, f)| ((*t).to_owned(), (*k).to_owned(), *x, *y, *f))
        .collect();
    assert_eq!(got, want, "AC-f-5: one table (I-f-3)");
}
