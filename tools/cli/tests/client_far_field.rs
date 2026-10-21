//! The far field is mandatory (RL-e0, `docs/DECISIONS.md` ARC-102, ARC-103): both reference clients
//! prove it by rendering, and the clients' far-field validator agrees with `mineworld_packages`.
//!
//! Every test here starts Godot, so every test is `#[ignore]`d — reported as ignored, never as passed,
//! on a machine without it (a test not run is not a pass). Where they run (R-FF-1, ruled 2026-10-15):
//! the nightly `clients` layer runs all three on Windows and the 2D check and the validators on Linux
//! and macOS (`scripts/ci_godot.py` SKIPS); ci.yml's per-PR `farfield` job runs all three on Linux when
//! a client or a Presentation Pack changes; locally on a Mac the 3D check takes 18 s:
//!
//! ```text
//! cargo test -p mineworld-cli --test client_far_field -- --ignored --test-threads=1
//! ```
//!
//! Verdicts are read from the lines the checks print (`slice_far_field_check.gd`,
//! `far_field_2d_check.gd`, `far_field_check.gd`), never from an exit status alone; a run that prints
//! no summary line is inconclusive and fails as such.

mod godot2d;
// `godot2d::worlds` reads saves through it.
mod headless;
mod support;

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use godot2d::{Drive, World, passed};
use mineworld_packages::far_field;
use mineworld_packages::{Dimension, PackageError};

const REPOSITORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const CLIENT_3D: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../clients/3d-spike");
const SHARED: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../clients/shared");
/// The rendered 3D check is 120 offscreen renders of the full slice; a software renderer on a hosted
/// runner takes minutes, not seconds (R-FF-1): with ruling (b) the hosted Linux runner (lavapipe) took
/// 41.7 min, its import included (PR CI run 38812488057), so the limit leaves it a margin.
const SLICE_LIMIT: Duration = Duration::from_secs(60 * 60);

fn repository(relative: &str) -> PathBuf {
    Path::new(REPOSITORY).join(relative)
}

/// Runs Godot with `arguments` until it exits or `limit` passes; returns whether it exited in time,
/// with success, and every line it printed.
fn godot(arguments: &[&str], limit: Duration) -> (Option<bool>, Vec<String>) {
    let mut child = Command::new(godot2d::godot())
        .args(arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("godot runs");
    let stdout = child.stdout.take().expect("piped");
    let lines = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&lines);
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            godot2d::echo("godot", &line);
            sink.lock().expect("lines").push(line);
        }
    });
    let deadline = Instant::now() + limit;
    let status = loop {
        if let Some(status) = child.try_wait().expect("the process can be waited on") {
            break Some(status.success());
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }
        std::thread::sleep(Duration::from_millis(200));
    };
    reader.join().expect("the reader ends with the process");
    let printed = lines.lock().expect("lines").clone();
    (status, printed)
}

/// A first import of the 3D project (every texture and model) took more than 10 minutes on the hosted
/// Windows runner (nightly run 38793596092); the other Godot tests import without a limit.
const IMPORT_LIMIT: Duration = Duration::from_secs(30 * 60);

fn import(project: &str) {
    let (done, _) = godot(&["--headless", "--path", project, "--import"], IMPORT_LIMIT);
    assert_eq!(done, Some(true), "{project} imports");
}

/// The verdict of a check that prints `all <name> checks pass` or `<n> <NAME> CHECKS FAILED`, with
/// every `FAIL <name> …` line: PASS only with the pass line and no failure line.
fn judge(lines: &[String], name: &str, expected_lines: usize, line_prefix: &str) {
    let failures: Vec<&String> = lines
        .iter()
        .filter(|l| {
            l.starts_with(&format!("FAIL {name} ")) || l.starts_with(&format!("FAIL {name}-"))
        })
        .collect();
    let passed = lines
        .iter()
        .any(|l| l == &format!("all {name} checks pass"));
    let failed = lines
        .iter()
        .any(|l| l.ends_with(&format!("{} CHECKS FAILED", name.to_uppercase())));
    assert!(
        passed || failed,
        "INCONCLUSIVE: the {name} check printed no summary line:\n{}",
        lines.join("\n")
    );
    assert!(
        failures.is_empty() && passed && !failed,
        "{name} failed:\n{}",
        failures
            .iter()
            .map(|l| l.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
    let verdicts = lines.iter().filter(|l| l.starts_with(line_prefix)).count();
    assert!(
        verdicts >= expected_lines,
        "{name}: {verdicts} verdict lines, expected at least {expected_lines}"
    );
}

/// AC-FF-5: offline, without GI, the slice closes every open sector at every fixed viewpoint —
/// FF-BELOW and FF-ABOVE (RL-e0 §6.3) from frames shown, hidden and of the sky alone — and the far
/// field is drawn right from every judged height: FF-ORDER, FF-CREST, FF-SEAM, FF-RELIEF, FF-RING and
/// FF-BAY from its ID renders (RL-e0b §6.3).
#[test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_far_field -- --ignored"]
fn three_d_far_field_closes_every_open_sector() {
    import(CLIENT_3D);
    let (done, lines) = godot(
        &[
            "--always-on-top",
            "--resolution",
            "1600x900",
            "--path",
            CLIENT_3D,
            "res://scenes/slice.tscn",
            "--",
            "--slice-farfield",
            "--gi=none",
            "--settings=none",
            "--aniso=off",
        ],
        SLICE_LIMIT,
    );
    assert!(
        done.is_some(),
        "INCONCLUSIVE: the slice did not finish in {SLICE_LIMIT:?}; it printed:\n{}",
        lines.join("\n")
    );
    // 7 viewpoints × 8 azimuths × (below, above): one line each, open-water sectors excepted; then
    // a crest and a seam line for each of the 3 judged viewpoints × 8 azimuths (RL-e0b §6.4)
    judge(&lines, "farfield", 7 * 8 + 3 * 8 * 2, "[farfield");
}

/// S33-d AC-d-1 (`DECISIONS.md` ARC-177): Lakeside, drawn from its depiction file — terrain, near lake,
/// village and woods — closes every open sector at its seven viewpoints (VP-L-1…7) and draws its far
/// field right from the three judged ones, by the same criteria as above.
#[test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_far_field -- --ignored"]
fn three_d_lakeside_far_field_closes_every_open_sector() {
    import(CLIENT_3D);
    let (done, lines) = godot(
        &[
            "--always-on-top",
            "--resolution",
            "1600x900",
            "--path",
            CLIENT_3D,
            "res://scenes/slice.tscn",
            "--",
            "--slice-farfield",
            "--world-key=lakeside",
            "--gi=none",
            "--settings=none",
            "--aniso=off",
        ],
        SLICE_LIMIT,
    );
    assert!(
        done.is_some(),
        "INCONCLUSIVE: the slice did not finish in {SLICE_LIMIT:?}; it printed:\n{}",
        lines.join("\n")
    );
    judge(&lines, "farfield", 7 * 8 + 3 * 8 * 2, "[farfield");
}

/// S33-c AC-c-5 (`DECISIONS.md` ARC-146): a world key the 3D client has no depiction for is refused by
/// name, exit 3, before anything is drawn; a join with no world key is refused by the far-field
/// loader, unchanged. Neither falls back to another world's depiction.
#[test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_far_field -- --ignored"]
fn a_world_without_a_depiction_is_refused_by_name() {
    import(CLIENT_3D);
    for (arguments, code, line) in [
        (
            vec!["--world-key=nowhere"],
            "PRESENTATION_WORLD_MISSING: { world: nowhere }",
            "3d-nowhere.pck",
        ),
        (
            vec!["--server=127.0.0.1:9", "--invite=none"],
            "FAR_FIELD_WORLD_UNKNOWN",
            "--world-key=",
        ),
    ] {
        let output = Command::new(godot2d::godot())
            .args([
                "--headless",
                "--path",
                CLIENT_3D,
                "res://scenes/slice.tscn",
                "--",
            ])
            .args(["--slice-drive", "--settings=none"])
            .args(&arguments)
            .output()
            .expect("godot runs");
        let printed = String::from_utf8_lossy(&output.stdout).into_owned();
        assert_eq!(
            output.status.code(),
            Some(3),
            "{arguments:?}: a scripted refusal exits 3; it printed:\n{printed}"
        );
        assert!(
            printed.contains(code) && printed.contains(line),
            "{arguments:?}: the refusal names {code}; it printed:\n{printed}"
        );
        assert!(
            !printed.contains("all drive checks pass"),
            "{arguments:?}: nothing is drawn after a refusal:\n{printed}"
        );
    }
}

/// AC-FF-6: in each world with a 2D far field, at every reachable extreme and every aspect the
/// settings allow, the clear colour is never drawn (two sentinels, RL-e0 §6.4).
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_far_field -- --ignored"]
async fn two_d_clear_colour_is_never_visible() {
    let path = |relative: &str| repository(relative).to_str().expect("UTF-8").to_owned();
    // [world, seat]: each world's own seat (Lakeside seats its villagers; bodies-yard starts gus in
    // the court, from where the hall is one doorway away)
    for (world, seat) in [
        ("market-town", "carol"),
        ("social-cafe", "carol"),
        ("lakeside", "alba"),
        ("bodies-yard", "gus"),
    ] {
        let mut arguments = vec!["server".to_owned(), path(&format!("worlds/{world}"))];
        if world == "lakeside" {
            // Milestone E's composition is found only through its pack roots (ARC-54)
            arguments.extend([
                "--packs".to_owned(),
                path("entities"),
                "--packs".to_owned(),
                path("presentation/mineworld-default"),
            ]);
        }
        let borrowed: Vec<&str> = arguments.iter().map(String::as_str).collect();
        let world_process = World::start(&borrowed, None).await;
        let key = format!("--world-key={world}");
        let (status, lines) = Drive::start_with(
            world_process.address,
            seat,
            &["--drive=farfield", &key],
            &["--always-on-top", "--resolution", "1600x900"],
        )
        .finish()
        .await;
        assert!(
            status.success() && passed(&lines),
            "{world}: the drive did not pass:\n{}",
            lines.join("\n")
        );
        // 8 extremes × 4 aspects
        judge(&lines, "farfield2d", 8 * 4, "[farfield2d] ");
        drop(world_process);
    }
}

/// The code a corpus file gets from `mineworld_packages`, or "ok".
fn rust_code(path: &Path, dimensions: &[Dimension]) -> String {
    match far_field::read(path, dimensions) {
        Ok(_) => "ok".to_owned(),
        Err(PackageError::FarField { code, .. }) => code.as_str().to_owned(),
        Err(other) => panic!("{}: {other}", path.display()),
    }
}

/// AC-FF-3: the GDScript validator and the Rust validator give the same code for every file of the
/// shared corpus, for an absent file, and for every shipped far field of both dimensions.
#[test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_far_field -- --ignored"]
fn the_two_validators_agree() {
    import(SHARED);
    let packs = repository("presentation/mineworld-default");
    let two = format!("--pack={}:2d", packs.join("2D").display());
    let three = format!("--pack={}:3d", packs.join("3D").display());
    let script = Path::new(SHARED).join("checks/far_field_check.gd");
    let (done, lines) = godot(
        &[
            "--headless",
            "--path",
            SHARED,
            "--script",
            script.to_str().expect("UTF-8"),
            "--",
            &two,
            &three,
        ],
        Duration::from_secs(300),
    );
    assert_eq!(done, Some(true), "the check ran:\n{}", lines.join("\n"));
    let printed: Vec<(String, String)> = lines
        .iter()
        .filter_map(|l| l.strip_prefix("FARFIELD "))
        .map(|rest| {
            let (name, code) = rest.rsplit_once(' ').expect("<name> <code>");
            (name.to_owned(), code.to_owned())
        })
        .collect();
    let corpus = repository("packages/tests/fixtures/far_field");
    let mut expected: Vec<(String, String)> = Vec::new();
    for entry in std::fs::read_dir(&corpus).expect("the corpus lists") {
        let path = entry.expect("an entry").path();
        let stem = path.file_stem().and_then(|s| s.to_str()).expect("a stem");
        let dimension = if stem.ends_with("-2d") {
            Dimension::TwoD
        } else {
            Dimension::ThreeD
        };
        expected.push((stem.to_owned(), rust_code(&path, &[dimension])));
    }
    let absent = far_field::require_for_world(&corpus.join(".."), "corpus", "missing")
        .expect_err("no corpus file is named missing");
    let PackageError::FarFieldMissing { .. } = absent else {
        panic!("not a missing far field: {absent}");
    };
    expected.push(("missing".to_owned(), "FAR_FIELD_MISSING".to_owned()));
    for (dir, word, dimension) in [
        ("2D", "2d", Dimension::TwoD),
        ("3D", "3d", Dimension::ThreeD),
    ] {
        for entry in std::fs::read_dir(packs.join(dir).join("far_field")).expect("far_field/") {
            let path = entry.expect("an entry").path();
            let stem = path.file_stem().and_then(|s| s.to_str()).expect("a stem");
            expected.push((format!("{word}/{stem}"), rust_code(&path, &[dimension])));
        }
    }
    expected.sort();
    let mut got = printed;
    got.sort();
    assert_eq!(
        got, expected,
        "the GDScript and Rust validators disagree (left: GDScript, right: Rust)"
    );
    assert!(
        expected.iter().filter(|(_, code)| code == "ok").count() >= 8,
        "the shipped far fields and the two valid examples are all read"
    );
}
