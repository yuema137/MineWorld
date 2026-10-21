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
#[allow(dead_code)]
mod headless;
#[allow(dead_code)]
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
fn decisions(godot: &str, project: &str, script: &str) -> Vec<String> {
    let project = Path::new(REPOSITORY).join(project);
    let imported = Command::new(godot)
        .args(["--headless", "--path"])
        .arg(&project)
        .arg("--import")
        .status()
        .expect("godot runs");
    assert!(imported.success(), "{} imports", project.display());
    let output = Command::new(godot)
        .args(["--headless", "--path"])
        .arg(&project)
        .args(["--script", script])
        .output()
        .expect("godot runs");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        stdout.contains("seat_check: 4 frames"),
        "INCONCLUSIVE: {} printed no summary line:\n{stdout}\n{}",
        project.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    stdout
        .lines()
        .filter(|line| line.starts_with("SEATED "))
        .map(str::to_owned)
        .collect()
}

#[test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_sit_rule -- --ignored"]
fn both_clients_draw_the_same_person_on_a_contested_seat() {
    let godot = godot2d::godot();
    let [two, three] = CLIENTS.map(|(project, script)| decisions(&godot, project, script));
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
            (at == last).then(|| (parts.next()?.to_owned(), parts.next()?.to_owned()))
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
        !two.iter().any(|line| line.starts_with("SEATED 100 ") || line.starts_with("SEATED 101 ")),
        "nobody is drawn seated before standing still for 2 s of world time: {two:?}"
    );
}
