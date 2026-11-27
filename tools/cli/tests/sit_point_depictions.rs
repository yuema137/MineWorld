//! S34-B drawing-coordinate oracle: measured visual seats, never furnishing origins.
//! Targets/triangles were measured and independently reviewed before authoring the displacements.
use serde_json::Value;
use std::path::Path;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const EVIDENCE: &str = include_str!("fixtures/seat-surfaces.json");
fn read(path: &str) -> Value {
    serde_saphyr::from_str(&std::fs::read_to_string(Path::new(ROOT).join(path)).unwrap()).unwrap()
}
fn number(v: &Value) -> f64 {
    v.as_f64().expect("finite coordinate")
}
fn near(actual: f64, expected: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= 0.010,
        "{label}: {actual} != {expected} (10 mm)"
    );
}
// Vertical ray against the selected original triangle, including curved surfaces and gaps.
fn height(triangle: &Value, x: f64, z: f64) -> Option<f64> {
    let a = &triangle[0];
    let b = &triangle[1];
    let c = &triangle[2];
    let bx = number(&b[0]) - number(&a[0]);
    let bz = number(&b[2]) - number(&a[2]);
    let cx = number(&c[0]) - number(&a[0]);
    let cz = number(&c[2]) - number(&a[2]);
    let det = bx * cz - bz * cx;
    if det.abs() < 1e-12 {
        return None;
    }
    let dx = x - number(&a[0]);
    let dz = z - number(&a[2]);
    let u = (dx * cz - dz * cx) / det;
    let v = (bx * dz - bz * dx) / det;
    (u >= -1e-8 && v >= -1e-8 && u + v <= 1.0 + 1e-8).then(|| {
        number(&a[1]) + u * (number(&b[1]) - number(&a[1])) + v * (number(&c[1]) - number(&a[1]))
    })
}

#[test]
fn all_cafe_depictions_match_independently_measured_drawing_anchors() {
    let evidence: Value = serde_json::from_str(EVIDENCE).unwrap();
    let bindings = read("presentation/mineworld-default/3D/furniture/furnishings.yaml");
    for world in ["social-cafe", "market-town"] {
        let source = read(&format!("worlds/{world}/places/cafe.yaml"));
        let depiction = read(&format!(
            "presentation/mineworld-default/3D/sit_points/{world}.yaml"
        ));
        assert_eq!(depiction["format"], 2);
        assert_eq!(depiction["scope"]["world"], world);
        let points = depiction["points"].as_array().unwrap();
        assert_eq!(points.len(), 9);
        for target in evidence["targets"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|t| t["world"] == world)
        {
            let matches: Vec<_> = points
                .iter()
                .filter(|p| p["point"] == target["point"])
                .collect();
            assert_eq!(matches.len(), 1);
            let dep = matches[0];
            assert_eq!(dep["prop"], target["prop"]);
            assert!(dep.get("seat_offset_m").is_none());
            let tags = dep["tags"].as_array().unwrap();
            assert!(!tags.is_empty());
            assert!(
                tags.iter()
                    .all(|t| source["tags"].as_array().unwrap().contains(t))
            );
            let point = source["sit-points"]
                .as_array()
                .unwrap()
                .iter()
                .find(|p| p["key"] == target["point"])
                .unwrap();
            let x =
                number(&point["at"]["x"]) / 1000.0 + number(&dep["seat_displacement_m"]["east"]);
            let north =
                number(&point["at"]["y"]) / 1000.0 + number(&dep["seat_displacement_m"]["north"]);
            near(
                x,
                number(&target["drawing_anchor_place_m"][0]),
                "east drawing anchor",
            );
            near(
                north,
                number(&target["drawing_anchor_place_m"][1]),
                "north drawing anchor",
            );
            near(
                number(&dep["seat_height_m"]),
                number(&target["seat_height_m"]),
                "seat height",
            );
            let prop = source["body"]["furnishings"]
                .as_array()
                .unwrap()
                .iter()
                .find(|p| p["key"] == target["prop"])
                .unwrap();
            let binding = &bindings["kinds"][prop["kind"].as_str().unwrap()];
            assert_eq!(binding["model"], target["model"]);
            let yaw =
                ((number(&binding["front_md"]) - number(&prop["facing"])) / 1000.0).to_radians();
            let dx = x - number(&prop["at"]["x"]) / 1000.0;
            let dz = -(north - number(&prop["at"]["y"]) / 1000.0);
            let local_x = yaw.cos() * dx - yaw.sin() * dz;
            let local_z = yaw.sin() * dx + yaw.cos() * dz;
            let surface = &evidence["surfaces"][target["model"].as_str().unwrap()];
            let measured = surface["triangles_m"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|t| height(t, local_x, local_z))
                .reduce(f64::max)
                .expect("drawing anchor intersects selected visual seat triangles");
            near(
                number(&dep["seat_height_m"]),
                measured,
                "independent selected surface height",
            );
        }
    }
}

#[test]
fn lakeside_calibrated_scalar_rows_retain_the_frozen_coordinates() {
    let evidence: Value = serde_json::from_str(EVIDENCE).unwrap();
    let depiction = read("presentation/mineworld-default/3D/sit_points/lakeside.yaml");
    assert_eq!(depiction["format"], 1);
    let points = depiction["points"].as_array().unwrap();
    assert_eq!(points.len(), 29);
    for row in evidence["legacy"]["rows"].as_array().unwrap() {
        let dep = points
            .iter()
            .find(|p| p["point"] == row["point"] && p["tags"] == row["tags"])
            .unwrap();
        assert!(dep.get("seat_displacement_m").is_none());
        let expected = &row["fallback_expected"];
        let at = &dep["fallback"]["at"];
        assert_eq!(dep["prop"], expected["prop"]);
        assert_eq!(dep["seat_height_m"], expected["seat_height_m"]);
        let yaw = (number(&dep["fallback"]["facing"]) / 1000.0).to_radians();
        let offset = number(&dep["seat_offset_m"]) * 1000.0;
        for (actual, want) in [
            (
                number(&at["x"]) - yaw.sin() * offset,
                number(&expected["pelvis"][0]),
            ),
            (
                number(&at["y"]) - yaw.cos() * offset,
                number(&expected["pelvis"][1]),
            ),
        ] {
            // Independent f64 calculation versus frozen engine Vector2. Exact engine parity is in sit_pose_check.
            assert!(
                (actual - want).abs() < 0.002,
                "legacy calibration changed: {actual} != {want}"
            );
        }
    }
    let surfaces = evidence["legacy_surfaces"].as_array().unwrap();
    assert_eq!(surfaces.len(), 29);
    for row in surfaces {
        assert_eq!(row["sample_over_candidate_seat"], true);
        near(
            number(&row["authored_height_m"]),
            number(&row["surface_height_m"]),
            "legacy visual seat height",
        );
    }
}
