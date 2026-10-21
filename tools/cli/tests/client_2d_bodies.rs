//! The 2D reference client among bodies (S12 PR 13d, ARC-342): loose objects drawn where the world
//! lists them, a kick and an aimed throw asked for from the map and drawn along the owner's path, a wall
//! that stops the player, and a shove from another seat — against real servers (AC-13d-1 … -3, -5 … -7).
//! The stub checks are in `client_2d_interact_stub.rs`.
//!
//! Worlds: W1 `worlds/bodies-yard`; W2 `worlds/social-cafe` (walked into); W3 a variant of W1 this file
//! writes into a scratch directory, every loose object moved 700 mm and every place renamed, so a client
//! that remembered W1 fails; and W1 without `bodies`. Every target the scripted player uses is derived
//! by the client from what it was shown (`harness/bodies.gd`); the bounds are the plan's (§6), fixed
//! here, never read from the code under test.
//!
//! Every test starts Godot, so every one is `#[ignore]`d, and runs at the milestone:
//!
//! ```text
//! cargo test -p mineworld-cli --test client_2d_bodies -- --ignored --test-threads=1
//! ```

mod godot2d;
mod headless;
mod support;

use std::path::{Path, PathBuf};
use std::time::Duration;

use godot2d::worlds::{copy_dir, ids, menus, start};
use godot2d::{World, evidence, passed, tagged};
use mineworld_contracts::ActionResult;
use serde_json::{Value, json};
use support::Client;

const BODIES_YARD: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/bodies-yard");
const SOCIAL_CAFE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/social-cafe");
/// The seat the 2D client takes in bodies-yard, and the one that shoves it.
const PLAYER: &str = "ada";
const SHOVER: &str = "cleo";

/// AC-13d-2, -3, -5, -6: drawn within this of where the world says (plan §6).
const AT_REST_MM: f64 = 1.0;
const RECONCILE_MM: f64 = 150.0;
/// A person's radius (`docs/` realistic default, 250 mm): the most a drawn body may lie inside a solid.
const PERSON_RADIUS_MM: f64 = 250.0;
/// How far the variant moves every loose object (plan §6, W3).
const SHIFT_MM: i64 = 700;

fn mm(value: &Value) -> (f64, f64) {
    let axis = |k: &str| value[k].as_f64().unwrap_or(f64::NAN);
    (axis("x"), axis("y"))
}

fn apart(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0).hypot(a.1 - b.1)
}

fn owned(arguments: &[String]) -> Vec<&str> {
    arguments.iter().map(String::as_str).collect()
}

fn hosted(world: &str) -> Vec<String> {
    vec!["server".to_owned(), world.to_owned()]
}

/// The client's requests, as sent.
fn requests(lines: &[String]) -> Vec<Value> {
    tagged(lines, "REQUEST ")
        .into_iter()
        .map(|r| r["request"].clone())
        .collect()
}

/// AC-13d-1: every report's drawn set is the listed set, and every drawn object is within 1 mm of its
/// disclosed `at` once at rest.
fn drawn_is_disclosed(lines: &[String]) {
    let reports = evidence(lines, "objects");
    assert!(!reports.is_empty(), "the objects were reported");
    for report in &reports {
        let mut listed: Vec<String> = report["listed"]
            .as_array()
            .expect("listed")
            .iter()
            .map(|v| v.as_str().expect("an id").to_owned())
            .collect();
        let drawn = report["drawn"].as_array().expect("drawn");
        let mut ids: Vec<String> = drawn
            .iter()
            .map(|d| d["id"].as_str().expect("an id").to_owned())
            .collect();
        listed.sort();
        ids.sort();
        assert_eq!(ids, listed, "drawn = listed: {report}");
        for d in drawn {
            assert!(
                apart(mm(&d["local"]), mm(&d["at"])) <= AT_REST_MM,
                "drawn where disclosed: {d}"
            );
        }
    }
}

/// The last motion drawn of `object`.
fn last_motion(lines: &[String], object: &str) -> Value {
    let reports = evidence(lines, "objects");
    let report = reports.last().expect("an objects report");
    report["motions"]
        .as_array()
        .expect("motions")
        .iter()
        .rev()
        .find(|m| m["id"] == object)
        .cloned()
        .unwrap_or_else(|| panic!("no motion of {object}: {report}"))
}

/// AC-13d-2's drawing half: a path was played point by point and the drawing rests on `to`.
fn played_along_its_path(lines: &[String], object: &str) {
    let motion = last_motion(lines, object);
    assert_eq!(
        motion["how"], "path",
        "the owner's path is played: {motion}"
    );
    assert_eq!(motion["ended"], true, "{motion}");
    let points = motion["points"].as_array().expect("points");
    let visited = motion["visited"].as_array().expect("visited");
    assert_eq!(
        visited.len(),
        points.len(),
        "one row per keyframe: {motion}"
    );
    for (seen, point) in visited.iter().zip(points) {
        assert!(
            apart(mm(seen), mm(point)) <= AT_REST_MM,
            "visited in order: {motion}"
        );
    }
}

/// The request of `action_type` sent, and the offer it came from (the subject's menu's `offered`).
fn sent_and_offered(lines: &[String], action_type: &str) -> (Value, Value) {
    let sent = requests(lines)
        .into_iter()
        .rev()
        .find(|r| r["action_type"] == action_type)
        .unwrap_or_else(|| panic!("no {action_type} was sent"));
    let object = sent["payload"]["payload"]["object"]["entity"]
        .as_str()
        .expect("an object reference")
        .to_owned();
    let offered = menus(lines, &object)
        .last()
        .and_then(|menu| {
            menu["offered"]
                .as_array()
                .and_then(|o| o.iter().find(|a| a["action_type"] == action_type).cloned())
        })
        .unwrap_or_else(|| panic!("the object {object}'s menu offered no {action_type}"));
    (sent, offered)
}

/// The steps of a kick: walk up to the nearest object, click it, choose the world's kick, wait for the
/// drawing to come to rest.
fn kick_steps() -> Vec<Value> {
    vec![
        json!({ "objects": {} }),
        json!({ "walk_near": "nearest" }),
        json!({ "open_object": "nearest" }),
        json!({ "choose": { "action_type": "kick" } }),
        json!({ "objects": { "moved": true } }),
    ]
}

/// The steps of an aimed throw: walk up to the nearest ball, click it, choose "… at…", click the
/// disclosed floor's centre, wait for the drawing to come to rest.
fn throw_steps() -> Vec<Value> {
    vec![
        json!({ "walk_near": "ball" }),
        json!({ "open_object": "ball" }),
        json!({ "choose": { "aim": true } }),
        json!({ "click_floor": "centre" }),
        json!({ "objects": { "moved": true } }),
    ]
}

/// AC-13d-2 on a real world: the kick sent is the offer, unchanged, and is drawn along its path.
fn kicked(lines: &[String]) {
    let (sent, offered) = sent_and_offered(lines, "kick");
    assert_eq!(sent["target"], offered["target"], "the offer's target");
    assert_eq!(
        sent["payload"]["payload"], offered["payload"],
        "the offer's payload"
    );
    let object = sent["payload"]["payload"]["object"]["entity"]
        .as_str()
        .expect("an id");
    played_along_its_path(lines, object);
}

/// AC-13d-3 on a real world: the throw carries the offer's object reference and `toward` the clicked
/// floor point in integer millimetres; the drawing rests where the world then lists the object.
fn thrown(lines: &[String]) {
    let (sent, offered) = sent_and_offered(lines, "throw");
    let payload = &sent["payload"]["payload"];
    assert_eq!(
        payload["object"], offered["payload"]["object"],
        "the reference, unchanged"
    );
    let clicked = evidence(lines, "clicked_floor")
        .pop()
        .expect("a floor click");
    let toward = &payload["toward"];
    assert!(
        toward["x"].is_i64() && toward["y"].is_i64(),
        "integer mm: {toward}"
    );
    assert!(
        apart(mm(toward), mm(&clicked["local"])) <= AT_REST_MM,
        "toward the clicked point: {toward} vs {clicked}"
    );
    let object = payload["object"]["entity"].as_str().expect("an id");
    let motion = last_motion(lines, object);
    assert_eq!(motion["ended"], true, "{motion}");
}

/// AC-13d-6: the held keys stopped at the solid; the drawn body is where the world put it.
fn stopped_by_the_wall(lines: &[String]) {
    let wall = evidence(lines, "wall").pop().expect("a wall report");
    let observed = mm(&wall["observed"]);
    let drawn = mm(&wall["drawn"]);
    assert!(
        apart(observed, drawn) <= RECONCILE_MM,
        "drawn within {RECONCILE_MM} mm of the observed position: {wall}"
    );
    let solid = &wall["solid"];
    let (lo, hi) = (mm(&solid["min"]), mm(&solid["max"]));
    let depth = (drawn.0 - lo.0)
        .min(hi.0 - drawn.0)
        .min(drawn.1 - lo.1)
        .min(hi.1 - drawn.1);
    assert!(
        depth <= PERSON_RADIUS_MM,
        "never deeper than a person's radius inside the solid: {depth} mm, {wall}"
    );
}

/// A shove from `seat` at the player: walk to where the player was last observed (the server routes
/// and stops the walker short of the body there, inside the shove's 1 000 mm — a walk to the person
/// stops 1 200 mm short), then shove, until the world accepts. `place` and `local` are the player's
/// observed position, from the client's last report. Returns how many tries it took.
async fn shove_the_player(
    address: std::net::SocketAddr,
    seat: &str,
    player: &str,
    place: &str,
    local: &Value,
) -> usize {
    let mut client = Client::connect(address).await;
    let (me, _) = client.join(seat).await;
    let request = |action: &str, target: Option<&str>, payload: Value| {
        json!({ "actor": me, "action_type": action, "target": target,
            "payload": { "action_type": action, "payload": payload }, "actor_location": null })
    };
    // Integer millimetres, as the contract declares them (the client's report is Godot's doubles).
    let (x, y) = mm(local);
    let mut walks = Vec::new();
    // The player's own point first; then points 700 mm off it, inside the shove's reach, in case a walk
    // to an occupied point is refused.
    for (dx, dy) in [
        (0.0, 0.0),
        (700.0, 0.0),
        (-700.0, 0.0),
        (0.0, 700.0),
        (0.0, -700.0),
    ] {
        let to = json!({ "place": { "entity": place, "entity_type": "place" },
            "local": { "x": (x + dx).round() as i64, "y": (y + dy).round() as i64, "z": 0 },
            "facing": null });
        let (_, walked) = client
            .submit(request("walk-to", None, json!({ "to": { "place": to } })))
            .await;
        let accepted = matches!(walked, ActionResult::Accepted { .. });
        walks.push(format!("{walked:?}"));
        if accepted {
            break;
        }
    }
    let mut last = String::new();
    for tries in 1..=40 {
        let (_, shoved) = client
            .submit(request("shove", Some(player), json!({})))
            .await;
        if matches!(shoved, ActionResult::Accepted { .. }) {
            return tries;
        }
        last = format!("{shoved:?}");
        client.submit(request("walk-step", None, json!({}))).await;
        tokio::time::sleep(Duration::from_millis(1_000)).await;
    }
    panic!("{seat} never shoved {player}: {last}; walks {walks:?}");
}

/// AC-13d-5: within 2 s of the shove the drawn player is within 150 mm of the observed position.
fn reconciled_after_the_shove(lines: &[String]) {
    let me = evidence(lines, "self").pop().expect("a self report");
    assert!(
        apart(mm(&me["local"]), mm(&me["drawn_local"])) <= RECONCILE_MM,
        "drawn where the world put me: {me}"
    );
}

/// W1 in full: objects, a kick, an aimed throw, a wall, a shove.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_bodies -- --ignored"]
async fn bodies_yard_in_2d() {
    let world = World::start(&owned(&hosted(BODIES_YARD)), None).await;
    let player = ids(BODIES_YARD)[PLAYER].clone();
    let sync = mineworld_test_support::scratch!(empty "2d-bodies-sync");
    let sync_arg = format!("--sync={}", sync.to_str().expect("UTF-8"));
    let mut steps = kick_steps();
    steps.extend(throw_steps());
    steps.extend([
        json!({ "hold_into_solid": 3 }),
        json!({ "mark": "ready" }),
        json!({ "after": "shoved" }),
        json!({ "sleep": 2 }),
        json!({ "self": true }),
        json!({ "objects": {} }),
    ]);
    let (drive, _dir) = start(
        world.address,
        PLAYER,
        "2d-bodies",
        &Value::Array(steps),
        &[&sync_arg],
    );
    let ready = sync.join("ready");
    drive
        .until("the player is ready to be shoved", |_| ready.exists())
        .await;
    let wall = evidence(&drive.lines(), "wall")
        .pop()
        .expect("the wall report precedes the mark");
    let place = wall["place"].as_str().expect("a place").to_owned();
    shove_the_player(world.address, SHOVER, &player, &place, &wall["observed"]).await;
    std::fs::write(sync.join("shoved"), "1").expect("the mark is written");
    let (status, lines) = drive.finish().await;
    assert!(
        status.success() && passed(&lines),
        "the drive passes:\n{}",
        lines.join("\n")
    );
    drawn_is_disclosed(&lines);
    kicked(&lines);
    thrown(&lines);
    stopped_by_the_wall(&lines);
    reconciled_after_the_shove(&lines);
}

/// W2: social-cafe's objects, walked in to, and an aimed throw there.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_bodies -- --ignored"]
async fn the_cafe_ball_in_2d() {
    let world = World::start(&owned(&hosted(SOCIAL_CAFE)), None).await;
    let mut steps = vec![json!({ "walk_into": "cafe" }), json!({ "objects": {} })];
    steps.extend(throw_steps());
    steps.push(json!({ "objects": {} }));
    let (drive, _dir) = start(
        world.address,
        "carol",
        "2d-bodies-cafe",
        &Value::Array(steps),
        &[],
    );
    let (status, lines) = drive.finish().await;
    assert!(
        status.success() && passed(&lines),
        "the drive passes:\n{}",
        lines.join("\n")
    );
    drawn_is_disclosed(&lines);
    thrown(&lines);
}

/// W3: the generated variant — a kick and a wall, nothing remembered from W1.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_bodies -- --ignored"]
async fn a_moved_and_renamed_yard_in_2d() {
    let scratch = mineworld_test_support::scratch!(empty "2d-bodies-variant");
    let variant = moved_and_renamed(&scratch);
    let path = variant.to_str().expect("UTF-8").to_owned();
    let world = World::start(&owned(&hosted(&path)), None).await;
    let mut steps = kick_steps();
    steps.extend([json!({ "hold_into_solid": 3 }), json!({ "objects": {} })]);
    let (drive, _dir) = start(
        world.address,
        PLAYER,
        "2d-bodies-w3",
        &Value::Array(steps),
        &[],
    );
    let (status, lines) = drive.finish().await;
    assert!(
        status.success() && passed(&lines),
        "the drive passes:\n{}",
        lines.join("\n")
    );
    drawn_is_disclosed(&lines);
    kicked(&lines);
    stopped_by_the_wall(&lines);
}

/// AC-13d-7: W1 without `bodies` — nothing drawn as an object, no object menu, no error line.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_bodies -- --ignored"]
async fn a_yard_without_bodies_in_2d() {
    let scratch = mineworld_test_support::scratch!(empty "2d-bodies-without");
    let without = without_bodies(&scratch);
    let path = without.to_str().expect("UTF-8").to_owned();
    let world = World::start(&owned(&hosted(&path)), None).await;
    let steps =
        json!([{ "objects": { "count": 0 } }, { "sleep": 2 }, { "objects": { "count": 0 } }]);
    let (drive, _dir) = start(world.address, PLAYER, "2d-bodies-without", &steps, &[]);
    let (status, lines) = drive.finish().await;
    assert!(
        status.success() && passed(&lines),
        "the drive passes:\n{}",
        lines.join("\n")
    );
    for report in evidence(&lines, "objects") {
        assert_eq!(report["drawn"], json!([]), "no object drawn: {report}");
    }
    let errors: Vec<&String> = lines
        .iter()
        .filter(|l| l.contains("SCRIPT ERROR") || l.contains("Parse Error"))
        .collect();
    assert!(errors.is_empty(), "no error line: {errors:?}");
}

/// The generated variant of the yard, built without Godot: it loads, and nothing of W1's is where it
/// was — every place renamed, every object moved — so the Godot run above proves nothing was remembered.
#[test]
fn the_variant_yard_moves_and_renames_everything() {
    let scratch = mineworld_test_support::scratch!(empty "2d-bodies-variant-check");
    let variant = moved_and_renamed(&scratch);
    let before = ids(BODIES_YARD);
    let after = ids(variant.to_str().expect("UTF-8"));
    for place in ["court", "hall"] {
        assert!(
            before.contains_key(place) && !after.contains_key(place),
            "{place} renamed"
        );
    }
    let mut moved = 0;
    for entry in std::fs::read_dir(Path::new(BODIES_YARD).join("items")).expect("items") {
        let entry = entry.expect("an entry");
        let x = at_x(&std::fs::read_to_string(entry.path()).expect("reads")).expect("an at");
        let renamed = entry
            .file_name()
            .to_string_lossy()
            .replace("court", "plaza")
            .replace("hall", "lobby");
        let there = std::fs::read_to_string(variant.join("items").join(&renamed)).expect("kept");
        let shift = (at_x(&there).expect("an at") - x).abs();
        assert_eq!(shift, SHIFT_MM, "{renamed} moved {SHIFT_MM} mm");
        moved += 1;
    }
    assert!(moved > 0, "the yard has loose objects");
}

/// W1 with every place renamed and every loose object moved 700 mm along x — east, or west where east
/// would leave its footprint off the floor — written into `scratch`; never into `worlds/`.
fn moved_and_renamed(scratch: &Path) -> PathBuf {
    let rename = |text: &str| text.replace("court", "plaza").replace("hall", "lobby");
    let copy = scratch.join("bodies-yard");
    copy_dir(Path::new(BODIES_YARD), &copy);
    let floor_max: Vec<(String, i64)> = ["court", "hall"]
        .iter()
        .map(|place| {
            let text = std::fs::read_to_string(copy.join(format!("places/{place}.yaml")))
                .expect("a place");
            let after = &text[text.find("max: { x: ").expect("a floor max") + 10..];
            let max: i64 = after[..after.find(',').expect("x,")]
                .trim()
                .parse()
                .expect("a number");
            (rename(place), max)
        })
        .collect();
    let mut files = Vec::new();
    for directory in ["", "places", "people", "items"] {
        for entry in std::fs::read_dir(copy.join(directory)).expect("a directory") {
            let path = entry.expect("an entry").path();
            if path.extension().is_some_and(|e| e == "yaml") {
                files.push(path);
            }
        }
    }
    for path in files {
        let mut text = rename(&std::fs::read_to_string(&path).expect("reads"));
        if path.parent().is_some_and(|p| p.ends_with("items")) {
            text = shifted(&text, &floor_max);
        }
        let name = rename(&path.file_name().expect("a name").to_string_lossy());
        std::fs::remove_file(&path).expect("removed");
        std::fs::write(path.with_file_name(name), text).expect("written");
    }
    copy
}

/// The `x` of an item's `at: { place: …, x: N, … }`, if it has one.
fn at_x(text: &str) -> Option<i64> {
    let line = text
        .lines()
        .find(|l| l.trim_start().starts_with("at: { place:"))?;
    let after = &line[line.find("x: ")? + 3..];
    after[..after.find(',')?].trim().parse().ok()
}

/// An item's text with its `at` moved SHIFT_MM along x, east unless that leaves the floor.
fn shifted(text: &str, floor_max: &[(String, i64)]) -> String {
    let Some(x) = at_x(text) else {
        return text.to_owned();
    };
    let line = text
        .lines()
        .find(|l| l.trim_start().starts_with("at: { place:"))
        .expect("an at line");
    let max = floor_max
        .iter()
        .find(|(place, _)| line.contains(&format!("place: {place},")))
        .map_or(i64::MAX, |(_, max)| *max);
    // 400 mm is the largest half-width a loose object may have (`bodies`' BodyShape).
    let moved = if x + SHIFT_MM + 400 <= max {
        x + SHIFT_MM
    } else {
        x - SHIFT_MM
    };
    text.replacen(
        line,
        &line.replacen(&format!("x: {x},"), &format!("x: {moved},"), 1),
        1,
    )
}

/// W1 without the `bodies` pack: out of `systems`, with its `body:` sections and its loose objects
/// (which only `bodies` gives a world), written into `scratch`.
fn without_bodies(scratch: &Path) -> PathBuf {
    let copy = scratch.join("bodies-yard");
    copy_dir(Path::new(BODIES_YARD), &copy);
    let world_file = copy.join("world.yaml");
    let declared = std::fs::read_to_string(&world_file).expect("world.yaml");
    let mut text = declared.replacen("\n  - bodies\n", "\n", 1);
    assert_ne!(text, declared, "bodies was declared");
    let start_items = text.find("\nitems:\n").expect("an items list");
    let end_items = start_items + text[start_items + 1..].find("\n\n").expect("its end") + 1;
    text.replace_range(start_items..end_items, "");
    std::fs::write(&world_file, text).expect("written");
    std::fs::remove_dir_all(copy.join("items")).expect("items removed");
    for place in ["court", "hall"] {
        let path = copy.join(format!("places/{place}.yaml"));
        let text = std::fs::read_to_string(&path).expect("a place");
        let cut = text.find("\nbody:").expect("a body section");
        std::fs::write(&path, &text[..=cut]).expect("written");
    }
    copy
}
