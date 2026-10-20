//! Running the real 3D client (`clients/3d-spike`, the connected slice) from a test: its market probe
//! (`--slice-market`, `scripts/slice/slice_probe_market.gd`) played headless against a real world, and
//! what it printed (step-15 §22, PR 16d).
//!
//! The probe prints one fact per line in 13b's keys — `MENU`, `PANELS`, `REQUEST`, `RESULT`, `TOAST`,
//! `STEP`, `STEP_DONE` — beside the raw disclosure each was built from, and `SCREEN`, `EVIDENCE`,
//! `[FAIL]` and `market complete: PASS|FAIL`. A test reads those and checks them against an oracle of
//! its own: the raw frame the probe printed, or the save's fact log.
//!
//! Godot is spawned directly (`godot` or `$GODOT`), never through the bash launcher, so these tests run
//! wherever Godot and cargo run (step-15 D-16d-12).

#![allow(dead_code)]

use std::io::{BufRead, BufReader};
use std::net::SocketAddr;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex, Once};
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::support::INVITE;

/// The 3D client's Godot project.
pub const PROJECT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../clients/3d-spike");
/// How long one scripted 3D run may take before the test calls it hung: the slice builds its scene
/// and bakes its light before the first step.
pub const RUN_LIMIT: Duration = Duration::from_secs(480);

/// Builds the project's class and import cache once per test binary.
pub fn imported() -> String {
    static IMPORT: Once = Once::new();
    let binary = crate::godot2d::godot();
    IMPORT.call_once(|| {
        let status = Command::new(&binary)
            .args(["--headless", "--path", PROJECT, "--import"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("godot runs");
        assert!(status.success(), "the 3D project imports");
    });
    binary
}

/// Plays `steps` (13b's selector format) with the market probe as `seat` against `server`, headless,
/// with `extra` client arguments; returns how the run ended and every line it printed, standard error
/// included (a script error is printed there).
pub fn play(
    server: SocketAddr,
    seat: &str,
    name: &str,
    steps: &Value,
    extra: &[&str],
) -> (ExitStatus, Vec<String>) {
    let dir = mineworld_test_support::scratch!(empty name);
    let path = dir.join("steps.json");
    std::fs::write(&path, serde_json::to_string_pretty(steps).expect("JSON"))
        .expect("steps written");
    let binary = imported();
    let mut child = Command::new(binary)
        .args([
            "--headless",
            "--path",
            PROJECT,
            "res://scenes/slice.tscn",
            "--",
        ])
        .arg("--slice-market")
        .arg(format!("--server={server}"))
        .arg(format!("--seat={seat}"))
        // the depiction the client draws the joined world with (S33-c, ARC-144): market-town's
        .arg("--world-key=market-town")
        .arg(format!("--invite={INVITE}"))
        .arg("--nickname=client-3d-test")
        .arg(format!("--steps={}", path.to_str().expect("a UTF-8 path")))
        .args(extra)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("godot runs");
    let lines = Arc::new(Mutex::new(Vec::new()));
    let mut readers = Vec::new();
    for (stream, tag) in [
        (
            Box::new(child.stdout.take().expect("piped")) as Box<dyn std::io::Read + Send>,
            "",
        ),
        (
            Box::new(child.stderr.take().expect("piped")) as Box<dyn std::io::Read + Send>,
            "[stderr] ",
        ),
    ] {
        let sink = Arc::clone(&lines);
        readers.push(std::thread::spawn(move || {
            for line in BufReader::new(stream).lines().map_while(Result::ok) {
                eprintln!("[3d] {tag}{line}");
                sink.lock().expect("lines").push(format!("{tag}{line}"));
            }
        }));
    }
    let deadline = Instant::now() + RUN_LIMIT;
    let status = loop {
        if let Some(status) = child.try_wait().expect("waitable") {
            break status;
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "the 3D client hung; it printed:\n{}",
                lines.lock().expect("lines").join("\n")
            );
        }
        std::thread::sleep(Duration::from_millis(100));
    };
    for reader in readers {
        reader.join().expect("a reader ends");
    }
    let printed = lines.lock().expect("lines").clone();
    drop(dir);
    (status, printed)
}

/// Whether the probe ran every step and nothing failed: its own verdict, and no script error.
pub fn passed(lines: &[String]) -> bool {
    lines.iter().any(|line| line == "market complete: PASS")
        && !lines
            .iter()
            .any(|line| line.contains("[FAIL]") || line.contains("SCRIPT ERROR"))
}

/// The `MENU` lines about `subject`, in order.
pub fn menus(lines: &[String], subject: &str) -> Vec<Value> {
    crate::godot2d::tagged(lines, "MENU ")
        .into_iter()
        .filter(|menu| menu["subject"] == subject)
        .collect()
}

/// The `EVIDENCE` value under `key`, the first one printed.
pub fn evidence(lines: &[String], key: &str) -> Value {
    crate::godot2d::evidence(lines, key)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no EVIDENCE {key}"))
}

/// A parsed JSON value with every whole-number double written as an integer, as it was on the wire
/// (Godot reads every JSON number as a double, `ADOPTION.md` §3.2).
pub fn whole(value: &Value) -> Value {
    match value {
        Value::Number(n) => n
            .as_f64()
            .filter(|f| f.fract() == 0.0 && f.abs() < 9.0e15 && n.as_i64().is_none())
            .map_or_else(
                || value.clone(),
                // A whole double below 2^53 is an exact integer.
                #[allow(clippy::cast_possible_truncation)]
                |f| Value::from(f as i64),
            ),
        Value::Array(items) => Value::Array(items.iter().map(whole).collect()),
        Value::Object(fields) => {
            Value::Object(fields.iter().map(|(k, v)| (k.clone(), whole(v))).collect())
        }
        _ => value.clone(),
    }
}
