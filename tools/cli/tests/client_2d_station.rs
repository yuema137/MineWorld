//! The station in the 2D client (S29-f, `.structured-coding/plans/mvp0/pr-s29f-station-2d.md` §§7–8):
//! the depot is drawn for a station that is its town's landmark and never for any other, the hall is
//! drawn at the shape the world discloses, and the board, window and kiosk show only what was disclosed
//! (`ARC-108`, `ARC-196`).
//!
//! Every test starts Godot and a real server, so every test is `#[ignore]`d: reported as ignored, never
//! as passed, on a machine without Godot. Run at the milestone, one at a time:
//!
//! ```text
//! cargo test -p mineworld-cli --test client_2d_station -- --ignored --test-threads=1
//! ```
//!
//! The oracle is the drive's printed `EVIDENCE` lines and its summary line, never its exit status alone.

mod godot2d;
// `godot2d::worlds` reads saves through it.
mod headless;
mod support;

use std::path::{Path, PathBuf};

use godot2d::{Drive, World, evidence, passed};
use serde_json::Value;

const REPOSITORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const DEPOT: &str = "sib_building_station";
const HALT: &str = "sib_building_station_halt";

fn repository(relative: &str) -> String {
    Path::new(REPOSITORY)
        .join(relative)
        .to_str()
        .expect("UTF-8")
        .to_owned()
}

/// Runs `--drive=station` on `world` as `seat`; returns every line the client printed, after checking
/// the drive's own verdict and its station summary.
async fn drive(world: &str, seat: &str, key: &str) -> Vec<String> {
    let mut arguments = vec!["server".to_owned(), world.to_owned()];
    if key == "lakeside" {
        // Milestone E's composition is found only through its pack roots (ARC-54)
        arguments.extend([
            "--packs".to_owned(),
            repository("entities"),
            "--packs".to_owned(),
            repository("presentation/mineworld-default"),
        ]);
    }
    let borrowed: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let world_process = World::start(&borrowed, None).await;
    let world_key = format!("--world-key={key}");
    let (status, lines) = Drive::start(
        world_process.address,
        seat,
        &["--drive=station", &world_key],
    )
    .finish()
    .await;
    assert!(
        lines
            .iter()
            .any(|l| l == "all station checks pass" || l.ends_with("STATION CHECKS FAILED")),
        "INCONCLUSIVE: {key}: the station drive printed no summary line:\n{}",
        lines.join("\n")
    );
    assert!(
        status.success() && passed(&lines) && lines.iter().any(|l| l == "all station checks pass"),
        "{key}: the station drive did not pass:\n{}",
        lines.join("\n")
    );
    lines
}

fn one(lines: &[String], key: &str) -> Value {
    evidence(lines, key)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no EVIDENCE {key}:\n{}", lines.join("\n")))
}

/// The room report of the place carrying `tag`, as the drive printed it there.
fn room(lines: &[String], tag: &str, without: &[&str]) -> Value {
    evidence(lines, "station_room")
        .into_iter()
        .find(|r| {
            let tags: Vec<&str> = r["tags"]
                .as_array()
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            tags.contains(&tag) && !without.iter().any(|w| tags.contains(w))
        })
        .unwrap_or_else(|| panic!("no room report for {tag}:\n{}", lines.join("\n")))
}

fn word(report: &Value, label: &str) -> String {
    report["words"]
        .as_array()
        .and_then(|w| w.iter().find(|x| x["label"] == label))
        .and_then(|x| x["text"].as_str())
        .unwrap_or("")
        .to_owned()
}

/// What every station walk must show, whatever the world: board blank with "No service yet", the
/// window shut with "Closed", and no fixture opened by a disclosure that did not happen.
fn fixtures_shut(lines: &[String]) {
    let hall = room(lines, "station", &["ticket-office", "platform"]);
    assert_eq!(hall["state"]["board"]["state"], "none", "{hall}");
    assert_eq!(word(&hall, "board"), "No service yet", "{hall}");
    let tickets = room(lines, "ticket-office", &[]);
    assert_eq!(tickets["state"]["window"], "shut", "{tickets}");
    assert_eq!(word(&tickets, "window"), "Closed", "{tickets}");
    let platform = room(lines, "platform", &[]);
    assert_eq!(platform["wall_m"], 1.1, "{platform}");
}

/// R-1: Market Town draws the depot at the street's west end, the hall at its disclosed 51.8 × 16.8 m.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_station -- --ignored"]
async fn r1_market_town() {
    let lines = drive(&repository("worlds/market-town"), "carol", "market-town").await;
    let facade = one(&lines, "station_facade");
    assert_eq!(facade["sprite"], DEPOT, "{facade}");
    assert_eq!(
        facade["in_dir"],
        serde_json::json!([-1.0, 0.0]),
        "inward west: {facade}"
    );
    let hall = room(&lines, "station", &["ticket-office", "platform"]);
    assert_eq!(hall["shaped"], true, "{hall}");
    let size: Vec<f64> = hall["size_m"]
        .as_array()
        .expect("a size")
        .iter()
        .filter_map(Value::as_f64)
        .collect();
    let mut sorted = size.clone();
    sorted.sort_by(f64::total_cmp);
    assert!(
        (sorted[0] - 16.8).abs() <= 0.01 && (sorted[1] - 51.8).abs() <= 0.01,
        "{hall}"
    );
    // The kiosk is open exactly when the hall discloses a shop (S29-b).
    let disclosed_shop = hall["state"]["kiosk"] == "open";
    let kiosk_roles: Vec<&str> = hall["furnishings"]
        .as_array()
        .map(|a| a.iter().filter_map(|f| f["role"].as_str()).collect())
        .unwrap_or_default();
    let expected = if disclosed_shop {
        "furnishing.kiosk:open"
    } else {
        "furnishing.kiosk:shut"
    };
    assert!(kiosk_roles.contains(&expected), "{hall}");
    assert_eq!(one(&lines, "station_from_street")["kiosk"], "shut");
    fixtures_shut(&lines);
}

/// R-2: Social Café draws the same depot; nothing is sold in its hall, so the kiosk is shut.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_station -- --ignored"]
async fn r2_social_cafe() {
    let lines = drive(&repository("worlds/social-cafe"), "carol", "social-cafe").await;
    assert_eq!(one(&lines, "station_facade")["sprite"], DEPOT);
    let hall = room(&lines, "station", &["ticket-office", "platform"]);
    assert_eq!(hall["state"]["kiosk"], "shut", "{hall}");
    fixtures_shut(&lines);
}

/// R-3: Lakeside's station is not the village's landmark: never the depot; drawn as a near-side room.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_station -- --ignored"]
async fn r3_lakeside() {
    let lines = drive(&repository("worlds/lakeside"), "alba", "lakeside").await;
    let facade = one(&lines, "station_facade");
    assert_ne!(facade["sprite"], DEPOT, "{facade}");
    assert_eq!(facade["drawn_as"], "room", "{facade}");
    fixtures_shut(&lines);
}

/// A copy of Market Town, named as the pack, with each `(file, from, to)` replacement made exactly once.
fn variant(name: &str, edits: &[(&str, &str, &str)]) -> mineworld_test_support::Scratch {
    let copy = mineworld_test_support::scratch!(name).within("market-town");
    copy_dir(&PathBuf::from(repository("worlds/market-town")), &copy);
    for (file, from, to) in edits {
        let path = copy.join(file);
        let text = std::fs::read_to_string(&path).expect("reads");
        assert_eq!(text.matches(from).count(), 1, "{file} holds {from:?} once");
        std::fs::write(&path, text.replace(from, to)).expect("writes");
    }
    copy
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a directory");
    for entry in std::fs::read_dir(from).expect("lists") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("a copy");
        }
    }
}

/// R-4 (V1): the park is the town's landmark instead: the same station is drawn as the halt, by role,
/// and its hall is still drawn at the world's 51.8 m.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_station -- --ignored"]
async fn r4_landmark_elsewhere() {
    let copy = variant(
        "station-v1",
        &[
            ("world.yaml", "  landmark: station\n", "  landmark: park\n"),
            ("places/station.yaml", "  - landmark\n", ""),
            (
                "places/park.yaml",
                "  - outdoor\n",
                "  - outdoor\n  - landmark\n",
            ),
        ],
    );
    let lines = drive(copy.to_str().expect("UTF-8"), "carol", "market-town").await;
    assert_eq!(one(&lines, "station_facade")["sprite"], HALT);
    let hall = room(&lines, "station", &["ticket-office", "platform"]);
    assert_eq!(hall["shaped"], true, "{hall}");
    fixtures_shut(&lines);
}

/// R-5 (V3): every station place lists its tags in reverse: the same pictures, specs and names.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_station -- --ignored"]
async fn r5_tag_order() {
    let copy = variant(
        "station-v3",
        &[
            (
                "places/station.yaml",
                "tags:\n  - station\n  - public\n  - landmark\n",
                "tags:\n  - landmark\n  - public\n  - station\n",
            ),
            (
                "places/station-tickets.yaml",
                "tags:\n  - station\n  - ticket-office\n  - public\n",
                "tags:\n  - public\n  - ticket-office\n  - station\n",
            ),
            (
                "places/station-platform.yaml",
                "tags:\n  - station\n  - stop\n  - platform\n  - public\n  - outdoor\n",
                "tags:\n  - outdoor\n  - public\n  - platform\n  - stop\n  - station\n",
            ),
        ],
    );
    let lines = drive(copy.to_str().expect("UTF-8"), "carol", "market-town").await;
    assert_eq!(one(&lines, "station_facade")["sprite"], DEPOT);
    let hall = room(&lines, "station", &["ticket-office", "platform"]);
    let labels: Vec<String> = hall["doorway_labels"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|d| d["text"].as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    assert!(
        labels.iter().any(|l| l.contains("ticket office")),
        "{labels:?}"
    );
    assert!(labels.iter().any(|l| l.contains("platform")), "{labels:?}");
    fixtures_shut(&lines);
}
