//! Two clients agree on a contested seat (S33-f AC-f-4; `ARC-145`): the 2D and the 3D client, each
//! through its own wiring of the shared sit decision (`clients/shared/sitting/sit_rule.gd`), are fed the
//! same recorded observations of Lakeside's bakery, two people still at one chair, and print who they
//! draw seated where. The two lists must be equal, and equal to the fixture's expectation: the lower
//! entity id takes the seat; a person still but 0.45 m away, and one walking past, are not seated.
//!
//! It starts Godot, so it is `#[ignore]`d — reported as ignored, never as passed, without it:
//!
//! ```text
//! cargo test -p mineworld-cli --test client_sit_rule -- --ignored
//! ```

mod godot2d;
mod headless;
mod support;

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

const REPOSITORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const FIXTURE: &str = "tests/fixtures/observations/sitting/contested-long-table.json";
const CLIENTS: [(&str, &str); 2] = [
    ("clients/2d", "scripts/harness/seat_check.gd"),
    ("clients/3d-spike", "scripts/checks/seat_check.gd"),
];

/// The `SEATED <at> <id> <point>` lines one client prints.
fn output(godot: &str, project: &str, script: &str, fixture: &Path, count: usize) -> Vec<String> {
    let project = Path::new(REPOSITORY).join(project);
    // Reuse an already bootstrapped class-name cache; the script itself is parsed on every run.
    if !project
        .join(".godot/global_script_class_cache.cfg")
        .exists()
    {
        let imported = Command::new(godot)
            .args(["--headless", "--path"])
            .arg(&project)
            .arg("--import")
            .status()
            .expect("godot runs");
        assert!(imported.success(), "{} imports", project.display());
    }
    let output = Command::new(godot)
        .args(["--headless", "--path"])
        .arg(&project)
        .args(["--script", script, "--"])
        .arg(format!("--fixture={}", fixture.display()))
        .output()
        .expect("godot runs");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        output.status.success()
            && !String::from_utf8_lossy(&output.stderr).contains("SCRIPT ERROR")
            && stdout.contains(&format!("seat_check: {count} frames")),
        "INCONCLUSIVE: {} printed no summary line:\n{stdout}\n{}",
        project.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    stdout.lines().map(str::to_owned).collect()
}

#[test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_sit_rule -- --ignored"]
fn both_clients_draw_the_same_person_on_a_contested_seat() {
    let godot = godot2d::godot();
    let fixture_path = Path::new(REPOSITORY).join(FIXTURE);
    let [two_drawn, three_drawn] =
        CLIENTS.map(|(project, script)| output(&godot, project, script, &fixture_path, 4));
    let decisions = |lines: &[String]| {
        lines
            .iter()
            .filter(|l| l.starts_with("SEATED "))
            .cloned()
            .collect::<Vec<_>>()
    };
    let (two, three) = (decisions(&two_drawn), decisions(&three_drawn));
    for at in 100..=103 {
        let rows: Vec<_> = two_drawn
            .iter()
            .filter(|l| l.starts_with(&format!("DRAWN {at} ")))
            .collect();
        assert_eq!(rows.len(), 5, "every observed person is drawn");
        for id in ["1", "7", "12", "30", "41"] {
            let expected = if id == "7" && at >= 102 {
                format!("DRAWN {at} 7 seated long-table.3 3 1090 3000")
            } else {
                format!("DRAWN {at} {id} standing")
            };
            assert!(rows.contains(&&expected), "missing {expected}: {rows:?}");
        }
    }
    println!("2D: {two:?}\n3D: {three:?}");
    assert_eq!(two, three, "AC-f-4: the 2D and the 3D decisions are equal");

    let fixture: Value = serde_json::from_str(
        &std::fs::read_to_string(Path::new(REPOSITORY).join(FIXTURE)).expect("the fixture reads"),
    )
    .expect("the fixture is JSON");
    let last = fixture["frames"]
        .as_array()
        .and_then(|frames| frames.last())
        .and_then(|frame| frame["at"].as_i64())
        .expect("a last frame");
    let finally: BTreeMap<String, String> = two
        .iter()
        .filter_map(|line| {
            let mut parts = line.split(' ').skip(1);
            let at: i64 = parts.next()?.parse().ok()?;
            let id = parts.next()?.to_owned();
            let point = parts.next()?.to_owned();
            (at == last).then_some((id, point))
        })
        .collect();
    let expected: BTreeMap<String, String> = fixture["expected"]
        .as_object()
        .expect("expected")
        .iter()
        .map(|(id, point)| (id.clone(), point.as_str().expect("a key").to_owned()))
        .collect();
    assert_eq!(
        finally, expected,
        "the lower id takes the contested seat; nobody else is drawn seated"
    );
    assert!(
        !two.iter()
            .any(|line| line.starts_with("SEATED 100 ") || line.starts_with("SEATED 101 ")),
        "nobody is drawn seated before standing still for 2 s of world time: {two:?}"
    );
}

/// Recorded posture frames exercise the real drawing path; the oracle reads disclosure and
/// measured drawing anchors independently of the depiction values. Generated renaming/mirroring
/// preserves the decisions and transforms the independent targets.
#[test]
#[ignore = "needs Godot; run at the S34-e milestone"]
fn disclosed_posture_and_seeded_variants_draw_exactly() {
    let godot = godot2d::godot();
    for world in ["social-cafe", "market-town"] {
        let path = Path::new(REPOSITORY).join(format!(
            "tests/fixtures/observations/sitting/posture-{world}.json"
        ));
        let mut fixture: Value = serde_json::from_str(
            &std::fs::read_to_string(&path).expect("recorded posture fixture"),
        )
        .unwrap();
        let evidence: Value =
            serde_json::from_str(include_str!("fixtures/seat-surfaces.json")).unwrap();
        fixture["drawing_targets"] = Value::Array(
            evidence["targets"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|t| t["world"] == world)
                .cloned()
                .collect(),
        );
        check_posture(&godot, &path, &fixture);
        // Generated counterfactual: draw every approved Café point through both actual placement paths.
        let mut every = fixture.clone();
        let first = fixture["frames"][0].clone();
        let mut all_frames = Vec::new();
        for (i, target) in fixture["drawing_targets"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            let mut frame = first.clone();
            frame["at"] = serde_json::json!(200 + i);
            let observer = frame["observer"].clone();
            for person in frame["entities"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .filter(|e| e["id"] == observer)
            {
                let place = person["location"]["place"]["entity"].clone();
                let components = person["components"].as_array_mut().unwrap();
                components.retain(|c| c["component_type"] != "posture-state");
                components.push(
                    serde_json::json!({"component_type": "posture-state", "payload": {
                        "seated": {"place": place, "point": target["point"], "since": 200}
                    }}),
                );
            }
            all_frames.push(frame);
        }
        every["frames"] = Value::Array(all_frames);
        let all_scratch = mineworld_test_support::scratch!(empty "s34b-all-cafe");
        let all_path = all_scratch.join("all.json");
        std::fs::write(&all_path, serde_json::to_vec(&every).unwrap()).unwrap();
        check_posture(&godot, &all_path, &every);
        let still = standing_on_point(&fixture);
        let scratch_still = mineworld_test_support::scratch!(empty "s34e-still-posture");
        let still_path = scratch_still.join("still.json");
        std::fs::write(&still_path, serde_json::to_vec(&still).unwrap()).unwrap();
        check_posture(&godot, &still_path, &still);
        let mut variant = every.clone();
        let mut seed = 0x34e_u64;
        let mut names = BTreeMap::new();
        // Seeded bijection: all repeated keys/tags share one renamed spelling, without world vocabulary.
        fn renamed(old: &str, seed: &mut u64, names: &mut BTreeMap<String, String>) -> String {
            names
                .entry(old.to_owned())
                .or_insert_with(|| {
                    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    format!("v{:016x}", seed)
                })
                .clone()
        }
        fn transform(v: &mut Value, seed: &mut u64, names: &mut BTreeMap<String, String>) {
            match v {
                Value::Object(fields) => {
                    for (key, value) in fields {
                        if (key == "key" || key == "point") && value.is_string() {
                            *value = Value::String(renamed(value.as_str().unwrap(), seed, names));
                        } else if key == "tags" {
                            for tag in value.as_array_mut().unwrap() {
                                *tag = Value::String(renamed(tag.as_str().unwrap(), seed, names));
                            }
                        } else if (key == "x" || key == "east") && value.is_number() {
                            *value = serde_json::json!(-value.as_f64().unwrap());
                        } else if (key == "facing" || key == "yaw") && value.is_number() {
                            *value =
                                serde_json::json!((360000.0 - value.as_f64().unwrap()) % 360000.0);
                        } else {
                            transform(value, seed, names);
                        }
                    }
                }
                Value::Array(values) => {
                    for value in values {
                        transform(value, seed, names);
                    }
                }
                _ => {}
            }
        }
        transform(&mut variant["frames"], &mut seed, &mut names);
        let scratch = mineworld_test_support::scratch!(empty "s34e-renamed-posture");
        let generated = scratch.join("variant.json");
        let mut depictions: Value = serde_json::from_str(
            &std::fs::read_to_string(Path::new(REPOSITORY).join(format!(
                "presentation/mineworld-default/3D/sit_points/{world}.yaml"
            )))
            .unwrap(),
        )
        .unwrap();
        transform(&mut depictions["points"], &mut seed, &mut names);
        for t in variant["drawing_targets"].as_array_mut().unwrap() {
            t["point"] = Value::String(names[t["point"].as_str().unwrap()].clone());
            t["drawing_anchor_place_m"][0] =
                serde_json::json!(-t["drawing_anchor_place_m"][0].as_f64().unwrap());
        }
        let pack = scratch.join("pack");
        std::fs::create_dir_all(pack.join("sit_points")).unwrap();
        std::fs::write(
            pack.join(format!("sit_points/{world}.yaml")),
            serde_json::to_vec(&depictions).unwrap(),
        )
        .unwrap();
        variant["depiction_pack"] = serde_json::json!(pack);
        std::fs::write(&generated, serde_json::to_vec(&variant).unwrap()).unwrap();
        check_posture(&godot, &generated, &variant);
        let still = standing_on_point(&variant);
        std::fs::write(&generated, serde_json::to_vec(&still).unwrap()).unwrap();
        check_posture(&godot, &generated, &still);
    }
}

fn component<'a>(entity: &'a Value, kind: &str) -> Option<&'a Value> {
    entity["components"]
        .as_array()?
        .iter()
        .find(|c| c["component_type"] == kind)
        .map(|c| &c["payload"])
}

fn check_posture(godot: &str, path: &Path, fixture: &Value) {
    let frames = fixture["frames"].as_array().unwrap();
    let [two, three] =
        CLIENTS.map(|(project, script)| output(godot, project, script, path, frames.len()));
    let decisions = |lines: &[String]| {
        lines
            .iter()
            .filter(|l| l.starts_with("SEATED "))
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(decisions(&two), decisions(&three));
    let mut expected = Vec::new();
    let mut expected_three = Vec::new();
    for frame in frames {
        let at = frame["at"].as_f64().unwrap() as i64;
        let entities = frame["entities"].as_array().unwrap();
        let place = entities
            .iter()
            .find(|e| e["id"] == frame["self_location"]["place"]["entity"])
            .unwrap();
        let points = component(place, "sit-points").unwrap().as_array().unwrap();
        for person in entities.iter().filter(|e| e["entity_type"] == "person") {
            let id = person["id"].as_str().unwrap();
            if let Some(point) =
                component(person, "posture-state").and_then(|s| s["seated"]["point"].as_str())
            {
                let p = points.iter().find(|p| p["key"] == point).unwrap();
                let target = fixture
                    .get("drawing_targets")
                    .and_then(Value::as_array)
                    .and_then(|targets| targets.iter().find(|t| t["point"] == point));
                let (x, y) = if let Some(t) = target {
                    let x =
                        (t["drawing_anchor_place_m"][0].as_f64().unwrap() * 1000.0).round() as i64;
                    let y =
                        (t["drawing_anchor_place_m"][1].as_f64().unwrap() * 1000.0).round() as i64;
                    let height = (t["seat_height_m"].as_f64().unwrap() * 1000000.0).round() as i64;
                    expected_three
                        .push(format!("DRAWN3 {at} {id} seated {point} {x} {y} {height}"));
                    (x, y)
                } else {
                    expected_three.push(format!("DRAWN3 {at} {id} standing"));
                    (
                        p["at"]["x"].as_f64().unwrap().round() as i64,
                        p["at"]["y"].as_f64().unwrap().round() as i64,
                    )
                };
                // Isometric screen-down is southeast (yaw 135°); five atlas views fold left/right.
                let yaw = p["facing"].as_f64().unwrap() / 1000.0;
                let angle = (yaw - 135.0 + 180.0).rem_euclid(360.0) - 180.0;
                let view = (angle.abs() / 45.0).round() as i64;
                expected.push(format!("DRAWN {at} {id} seated {point} {view} {x} {y}"));
            } else {
                expected.push(format!("DRAWN {at} {id} standing"));
                expected_three.push(format!("DRAWN3 {at} {id} standing"));
            }
        }
    }
    let mut actual_three: Vec<_> = three
        .into_iter()
        .filter(|l| l.starts_with("DRAWN3 "))
        .collect();
    actual_three.sort();
    expected_three.sort();
    assert_eq!(
        actual_three, expected_three,
        "3D figure-node coordinates and seat-height input"
    );
    let mut actual: Vec<_> = two
        .into_iter()
        .filter(|l| l.starts_with("DRAWN "))
        .collect();
    actual.sort();
    expected.sort();
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(&expected) {
        let a: Vec<_> = actual.split_whitespace().collect();
        let e: Vec<_> = expected.split_whitespace().collect();
        if e.get(3) == Some(&"seated") {
            assert_eq!(&a[..6], &e[..6], "drawing identity/posture/view");
            for axis in [6, 7] {
                let delta = a[axis].parse::<i64>().unwrap() - e[axis].parse::<i64>().unwrap();
                // The real 2D town conversion truncates to integer mm; the independent engine
                // target is rounded. One mm covers this observation quantization (below B's 10 mm).
                assert!(delta.abs() <= 1, "drawn coordinate: {actual} != {expected}");
            }
        } else {
            assert_eq!(actual, expected, "standing bystander");
        }
    }
}

/// A labelled counterfactual, never presented as a recorded frame: stand on a disclosed point for 5 s.
fn standing_on_point(fixture: &Value) -> Value {
    let mut f = fixture["frames"][0].clone();
    let point = f["entities"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|e| component(e, "sit-points"))
        .unwrap()[0]["at"]
        .clone();
    let observer = f["observer"].clone();
    for e in f["entities"].as_array_mut().unwrap() {
        if let Some(components) = e.get_mut("components").and_then(Value::as_array_mut) {
            components.retain(|c| c["component_type"] != "posture-state");
        }
        if e["id"] == observer {
            e["location"]["local"] = point.clone();
        }
    }
    f["self_location"]["local"] = point;
    f["at"] = serde_json::json!(100);
    let mut later = f.clone();
    later["at"] = serde_json::json!(105);
    let mut out = fixture.clone();
    out["frames"] = serde_json::json!([f, later]);
    out
}

#[test]
#[ignore = "needs Godot; posture disclosure must suppress legacy stillness inference"]
fn disclosure_suppresses_lakeside_fallback_even_after_five_seconds() {
    let mut fixture: Value = serde_json::from_str(
        &std::fs::read_to_string(Path::new(REPOSITORY).join(FIXTURE)).unwrap(),
    )
    .unwrap();
    let depictions: Value = serde_json::from_str(
        &std::fs::read_to_string(
            Path::new(REPOSITORY)
                .join("presentation/mineworld-default/3D/sit_points/lakeside.yaml"),
        )
        .unwrap(),
    )
    .unwrap();
    let points: Vec<_> = depictions["points"].as_array().unwrap().iter()
        .filter(|p| p["tags"].as_array().unwrap().iter().any(|t| t == "bakery"))
        .map(|p| serde_json::json!({"key": p["point"], "at": p["fallback"]["at"], "facing": p["fallback"]["facing"]})).collect();
    for frame in fixture["frames"].as_array_mut().unwrap() {
        for entity in frame["entities"].as_array_mut().unwrap() {
            if entity["entity_type"] == "place" {
                entity["components"] =
                    serde_json::json!([{"component_type": "sit-points", "payload": points}]);
            }
        }
    }
    let still = standing_on_point(&fixture);
    let scratch = mineworld_test_support::scratch!(empty "s34e-disclosure-over-fallback");
    let path = scratch.join("counterfactual.json");
    std::fs::write(&path, serde_json::to_vec(&still).unwrap()).unwrap();
    check_posture(&godot2d::godot(), &path, &still);
}
