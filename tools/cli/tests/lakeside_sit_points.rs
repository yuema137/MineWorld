//! Lakeside's sit points, written into its place files by S34-c2 (`pr-s34c-world-sit-points.md` §1
//! "In (S34-c2 …)" and its dated note Q-S34c3-7; `ARC-172`), and the 3D Presentation Pack's depictions of
//! them (S33-f, `pr-s33f-interiors-sit-points.md` §5.2; `ARC-145`).
//!
//! ```text
//! (AC-c-1) loads   shipped Lakeside, with `posture` enabled and its `sit-points:` sections, is read and
//!                  loaded by the real loader: posture's own checks (keys, spacing >= 510 mm) and S34-b's
//!                  load check refuse nothing
//! (I-c-1)  clear   every point stands on its place's floor, >= R + GAP (260 mm) from every floor edge
//!                  and every solid, as a doorway point does (SD-S34-3); and the pelvis the depiction
//!                  draws (`at` moved by `seat_offset_m` against the facing) is over its prop's solid
//! (I-f-3)  one     the world's points equal the design's table, value for value; every point has
//!                  exactly one depiction and every depiction one point (I-S34-9's drawn half for
//!                  Lakeside, R-S34f-f-1); the depiction file carries no position of its own (ARC-172:
//!                  the fallback copy is gone, so the two cannot drift)
//! ```
//!
//! The oracles are the world's own place files (`bodies/mod.rs`'s independent reading) and the design's
//! values restated here; never a value the client or the System computes.

mod bodies;
mod headless;
#[allow(dead_code)]
mod lakeside;

use std::collections::BTreeMap;
use std::path::Path;

use bodies::{GAP, R, rooms_of};
use lakeside::{lakeside, read_lf, read_world, repository};
use mineworld_contracts::WorldTime;
use serde_json::Value;

/// The presentation file under test.
const FILE: &str = "presentation/mineworld-default/3D/sit_points/lakeside.yaml";

/// The design's table (S33-f §5.2 as revised; S34-c2's ledger BC-c2-1 … BC-c2-3): (place key, sit-point
/// key, group, x, y, facing). One person per 600 mm of bench (SD-c-2, Q-S34c3-4: floor(length / 600)),
/// centred, 260 mm in front of the bench's box; the bakery's points 260 mm behind each chair's box.
#[rustfmt::skip]
const TABLE: [(&str, &str, &str, i32, i32, u32); 29] = [
    ("bakery", "long-table.1", "long-table.chair-1", 1660, 1800, 270_000),
    ("bakery", "long-table.2", "long-table.chair-2", 1660, 2400, 270_000),
    ("bakery", "long-table.3", "long-table.chair-3", 1660, 3000, 270_000),
    ("bakery", "long-table.4", "long-table.chair-4", 1660, 3600, 270_000),
    ("bakery", "long-table.5", "long-table.chair-5", 1660, 4200, 270_000),
    ("pier", "pier-bench.1", "pier-bench", 800, 1010, 0),
    ("pier", "pier-bench.2", "pier-bench", 1400, 1010, 0),
    ("pier", "pier-bench.3", "pier-bench", 2000, 1010, 0),
    ("pier", "pier-bench.4", "pier-bench", 2600, 1010, 0),
    ("pier", "pier-bench.5", "pier-bench", 3200, 1010, 0),
    ("square", "tree-bench-north.1", "tree-bench-north", 11400, 10260, 0),
    ("square", "tree-bench-north.2", "tree-bench-north", 12000, 10260, 0),
    ("square", "tree-bench-east.1", "tree-bench-east", 13260, 8400, 90_000),
    ("square", "tree-bench-east.2", "tree-bench-east", 13260, 9000, 90_000),
    ("square", "tree-bench-east.3", "tree-bench-east", 13260, 9600, 90_000),
    ("station", "wall-bench.1", "wall-bench", 1900, 710, 0),
    ("station", "wall-bench.2", "wall-bench", 2500, 710, 0),
    ("station", "wall-bench.3", "wall-bench", 3100, 710, 0),
    ("station", "wall-bench.4", "wall-bench", 3700, 710, 0),
    ("station", "wall-bench.5", "wall-bench", 4300, 710, 0),
    ("station", "wall-bench.6", "wall-bench", 4900, 710, 0),
    ("station", "wall-bench.7", "wall-bench", 5500, 710, 0),
    ("station", "wall-bench.8", "wall-bench", 6100, 710, 0),
    ("station-platform", "platform-bench-s.1", "platform-bench-s", -3240, -5600, 90_000),
    ("station-platform", "platform-bench-s.2", "platform-bench-s", -3240, -5000, 90_000),
    ("station-platform", "platform-bench-s.3", "platform-bench-s", -3240, -4400, 90_000),
    ("station-platform", "platform-bench-n.1", "platform-bench-n", -3240, 4400, 90_000),
    ("station-platform", "platform-bench-n.2", "platform-bench-n", -3240, 5000, 90_000),
    ("station-platform", "platform-bench-n.3", "platform-bench-n", -3240, 5600, 90_000),
];

/// One sit point as a place file writes it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct WorldPoint {
    place: String,
    key: String,
    group: String,
    x: i32,
    y: i32,
    facing: u32,
}

/// The value after `name:` in a one-line flow mapping, up to the next `,` or `}`.
fn field<'a>(line: &'a str, name: &str) -> &'a str {
    let rest = line
        .split(&format!(" {name}:"))
        .nth(1)
        .unwrap_or_else(|| panic!("`{name}:` in {line}"));
    rest.trim_start()
        .split([',', '}'])
        .next()
        .expect("a value")
        .trim()
}

/// Every place file's `sit-points:` entries, read as text, sorted by place and key.
fn world_points(world: &Path) -> Vec<WorldPoint> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(world.join("places")).expect("places/ lists") {
        let path = entry.expect("an entry").path();
        let place = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("a key")
            .to_owned();
        let text = read_lf(&path);
        for line in text
            .lines()
            .skip_while(|line| *line != "sit-points:")
            .skip(1)
            .take_while(|line| line.starts_with("  - {"))
        {
            out.push(WorldPoint {
                place: place.clone(),
                key: field(line, "key").to_owned(),
                group: field(line, "group").to_owned(),
                x: field(line, "x").parse().expect("x"),
                y: field(line, "y").parse().expect("y"),
                facing: field(line, "facing").parse().expect("facing"),
            });
        }
    }
    out.sort();
    out
}

/// One depiction as the presentation file states it.
struct Depiction {
    tag: String,
    /// The place tags the point's place must not carry (`unless`).
    unless: Vec<String>,
    key: String,
    group: String,
    prop: String,
    offset_mm: i32,
    /// Whether the entry still carries a position of its own (`fallback`), which ARC-172 removed.
    positioned: bool,
}

fn depictions() -> Vec<Depiction> {
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
            Depiction {
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
                positioned: p.get("fallback").is_some() || p.get("at").is_some(),
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

/// Each depiction by (place key, point key).
fn depictions_by_point(world: &Path) -> BTreeMap<(String, String), Depiction> {
    let mut out = BTreeMap::new();
    for d in depictions() {
        let place = place_of(world, &d.tag, &d.unless);
        let key = (place.clone(), d.key.clone());
        assert!(
            out.insert(key, d).is_none(),
            "one depiction per point of {place}"
        );
    }
    out
}

#[test]
fn shipped_lakeside_loads_under_posture_s_checks() {
    let world = lakeside();
    let points = world_points(&world);
    assert!(
        !points.is_empty(),
        "located first: Lakeside's place files carry sit points"
    );
    let loaded = read_world(&world).load(WorldTime::EPOCH);
    assert!(
        loaded.is_ok(),
        "Lakeside with posture and its sit points loads: {:?}",
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
    let drawn = depictions_by_point(&world);
    let clearance = f64::from(R + GAP);
    let mut faults = Vec::new();
    for p in world_points(&world) {
        let room = &rooms[&p.place];
        let (x0, y0, x1, y1) = room.floor;
        let edge = (p.x - x0).min(x1 - p.x).min(p.y - y0).min(y1 - p.y);
        if f64::from(edge) < clearance {
            faults.push(format!(
                "{}/{}: {edge} mm from the floor's edge",
                p.place, p.key
            ));
        }
        for (rect, _) in &room.solids {
            let d = distance_to(*rect, p.x, p.y);
            if d < clearance {
                faults.push(format!(
                    "{}/{}: {d:.0} mm from the solid {rect:?}",
                    p.place, p.key
                ));
            }
        }
        let Some(depiction) = drawn.get(&(p.place.clone(), p.key.clone())) else {
            faults.push(format!("{}/{}: no depiction", p.place, p.key));
            continue;
        };
        // the pelvis: `at` moved by the offset against the facing (yaw from +y toward +x)
        let yaw = f64::from(p.facing) / 1000.0_f64;
        let (fx, fy) = (yaw.to_radians().sin(), yaw.to_radians().cos());
        let px = f64::from(p.x) - fx * f64::from(depiction.offset_mm);
        let py = f64::from(p.y) - fy * f64::from(depiction.offset_mm);
        let names = solid_names(&world, &p.place);
        assert_eq!(
            names.len(),
            room.solids.len(),
            "{}: every solid read",
            p.place
        );
        let seat: Vec<_> = room
            .solids
            .iter()
            .zip(&names)
            .filter(|(_, name)| name.as_deref() == Some(depiction.prop.as_str()))
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
                "{}/{}: the pelvis ({px:.0}, {py:.0}) is over no solid of {} ({seat:?})",
                p.place, p.key, depiction.prop
            ));
        }
    }
    assert!(faults.is_empty(), "I-c-1:\n{}", faults.join("\n"));
}

#[test]
fn one_table_the_world_s_points_and_their_depictions() {
    let world = lakeside();
    let got = world_points(&world);
    let mut want: Vec<WorldPoint> = TABLE
        .iter()
        .map(|(place, key, group, x, y, facing)| WorldPoint {
            place: (*place).to_owned(),
            key: (*key).to_owned(),
            group: (*group).to_owned(),
            x: *x,
            y: *y,
            facing: *facing,
        })
        .collect();
    want.sort();
    assert_eq!(got, want, "the world's sit points are the design's table");

    let drawn = depictions_by_point(&world);
    let points: Vec<(String, String)> = got
        .iter()
        .map(|p| (p.place.clone(), p.key.clone()))
        .collect();
    let bound: Vec<(String, String)> = drawn.keys().cloned().collect();
    assert_eq!(
        bound, points,
        "every sit point has one depiction, every depiction one sit point"
    );
    for p in &got {
        let d = &drawn[&(p.place.clone(), p.key.clone())];
        assert_eq!(
            d.group, p.group,
            "{}/{}: the depiction's group",
            p.place, p.key
        );
        assert!(
            !d.positioned,
            "{}/{}: the depiction carries no position of its own (ARC-172)",
            p.place, p.key
        );
    }
}
