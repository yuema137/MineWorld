//! S33-i — **an MCP agent plays Lakeside like a person, and a 3D client sees it** (`pr-s33i-agents.md` §5.2,
//! AC-i-4; `ARC-175`).
//!
//! ```text
//! mineworld server worlds/lakeside --grant nils --save DIR           the world, persisted
//! godot clients/3d-spike … --slice-observe --seat=elin               the real 3D client: out to the street,
//!                                                                    then stands there, logging what it draws
//! python clients/agent/tests/test_play_lakeside.py --server …        the scripted MCP agent as Nils
//! ```
//!
//! The 3D client walks Elin out of her door onto the depiction's street with its own `move`s, as a player
//! would, and stands there. Then the agent script (no model) walks Nils from the pier to the station and back.
//! An observer perceives only its own place (presence's rule), so the comparison covers exactly the
//! crossings of the observer's place: the save's `person-entered-place` facts for Nils that enter or leave
//! it, against the appearances and disappearances of his figure in the client's `--observe-log`. They must
//! be the same, in the same order, each within 2 s of world time; and while he is drawn in the street, his
//! figure must travel from one doorway to the other and back. Nothing here names a place: the observer's
//! place is the one its probe reports, and the people are the two seats the test plays.
//!
//! It needs Godot 4.7 and the Python workspace (`uv sync --locked`), so the pair is `#[ignore]`d and runs
//! in the nightly `agent-observer` layer (`scripts/ci_layer.py`) of every OS's `clients-<os>` job:
//!
//! ```text
//! cargo test -p mineworld-cli --test agent_with_3d_observer -- --ignored --test-threads=1
//! ```
//!
//! The comparison itself (and M-i-2, a log that drops one crossing) is checked without Godot.

mod godot2d;
mod godot3d;
mod headless;
mod support;

use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use godot2d::SLICE_3D_CI;
use godot2d::worlds::id;
use headless::Tables;
use mineworld_contracts::{EventEnvelope, WorldTime};
use mineworld_packages::PackRoots;
use mineworld_persistence::format;
use mineworld_worldpack::WorldPack;
use serde_json::Value;
use support::{INVITE, SaveDir, Server};

/// The seat the agent plays, and the seat the 3D client observes from.
const AGENT_SEAT: &str = "nils";
const OBSERVER_SEAT: &str = "elin";
/// The world, by its pack directory, and the depiction key the 3D client draws it with.
const WORLD: &str = "lakeside";
/// How far apart, in world seconds, a drawn crossing may be from the server's.
const TOLERANCE_S: i64 = 2;
/// The least distance, in mm, the figure must travel across the observer's place on each pass.
const TRAVEL_MM: f64 = 5_000.0;
/// How near, in mm, the inbound pass must start and end to where the outbound one ended and started.
const DOORWAY_MM: f64 = 3_000.0;
/// The ceiling for each process to reach its next line.
const PATIENCE: Duration = Duration::from_secs(480);

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The pack roots every command on the world names (its world.yaml): the Entity Packs and the
/// Presentation Packs it requires.
fn pack_roots() -> Vec<PathBuf> {
    vec![
        repository().join("entities"),
        repository().join("presentation").join("mineworld-default"),
    ]
}

/// The entity id the world's genesis gives the authored key `key`, as a payload carries it.
fn genesis_id(pack: &Path, key: &str) -> String {
    let roots = PackRoots::new(pack_roots(), None).expect("the roots exist");
    let loaded = WorldPack::read_with(pack, &roots)
        .expect("the World Pack reads")
        .load(WorldTime::EPOCH)
        .expect("it loads");
    loaded
        .ids()
        .iter()
        .find(|(k, _)| k.as_str() == key)
        .map(|(_, id)| id.raw().to_string())
        .unwrap_or_else(|| panic!("{key} is an authored key of the world"))
}

/// One crossing of the observer's place by the agent's person, at a world second.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Crossing {
    Enters(i64),
    Leaves(i64),
}

impl Crossing {
    const fn at(self) -> i64 {
        match self {
            Self::Enters(at) | Self::Leaves(at) => at,
        }
    }

    const fn enters(self) -> bool {
        matches!(self, Self::Enters(_))
    }
}

/// One line of the observe log in the observer's place: the clock, and where `person` is drawn (if at all).
#[derive(Clone, Debug, PartialEq)]
struct Drawn {
    at: i64,
    figure: Option<(f64, f64)>,
}

/// The save's crossings of `place` by `person`, in log order: each `person-entered-place` that enters or
/// leaves it.
fn server_crossings(save: &SaveDir, person: &str, place: &str) -> Vec<Crossing> {
    Tables::read(Path::new(save.path()))
        .facts
        .iter()
        .filter_map(|(_, bytes)| {
            let fact: EventEnvelope = format::decode(bytes, "fact").expect("a fact");
            if fact.event_type().as_str() != "person-entered-place" {
                return None;
            }
            let payload: Value =
                serde_json::from_slice(fact.payload().payload()).expect("a JSON payload");
            if id(&payload["person"]) != person {
                return None;
            }
            let at = fact.at().seconds();
            if id(&payload["place"]) == place {
                Some(Crossing::Enters(at))
            } else if id(&payload["from"]) == place {
                Some(Crossing::Leaves(at))
            } else {
                None
            }
        })
        .collect()
}

/// The observe log's lines in `place`, with `person`'s drawn position on each.
fn drawn_in(log: &str, person: &str, place: &str) -> Vec<Drawn> {
    log.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str::<Value>(line).expect("an observe-log line is JSON"))
        .filter(|line| line["place"] == place)
        .map(|line| Drawn {
            at: line["at"].as_i64().unwrap_or_else(|| {
                #[allow(clippy::cast_possible_truncation)]
                let at = line["at"].as_f64().expect("a clock") as i64;
                at
            }),
            figure: line["figures"]
                .as_array()
                .expect("figures")
                .iter()
                .find(|figure| id(&figure["id"]) == person)
                .map(|figure| {
                    (
                        figure["x"].as_f64().expect("x"),
                        figure["y"].as_f64().expect("y"),
                    )
                }),
        })
        .collect()
}

/// The figure's appearances and disappearances in the log, as crossings, and the positions it was drawn
/// at during each appearance.
fn drawn_crossings(drawn: &[Drawn]) -> (Vec<Crossing>, Vec<Vec<(f64, f64)>>) {
    let mut crossings = Vec::new();
    let mut passes: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut shown = false;
    for line in drawn {
        match (shown, line.figure) {
            (false, Some(at)) => {
                crossings.push(Crossing::Enters(line.at));
                passes.push(vec![at]);
                shown = true;
            }
            (true, Some(at)) => passes.last_mut().expect("a pass").push(at),
            (true, None) => {
                crossings.push(Crossing::Leaves(line.at));
                shown = false;
            }
            (false, None) => {}
        }
    }
    (crossings, passes)
}

fn distance(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0).hypot(a.1 - b.1)
}

/// AC-i-4's verdict: the drawn crossings are the server's, in order, each within [`TOLERANCE_S`]; and the
/// figure travels across the place and back. Each failure names its step.
fn compare(server: &[Crossing], drawn: &[Drawn]) -> Result<(), String> {
    let (shown, passes) = drawn_crossings(drawn);
    if server.len() != 4 {
        return Err(format!(
            "the server recorded {} crossings of the observer's place, not 4: {server:?}",
            server.len()
        ));
    }
    for (step, expected) in server.iter().enumerate() {
        let Some(seen) = shown.get(step) else {
            return Err(format!(
                "crossing {} ({expected:?}) was never drawn; drawn: {shown:?}",
                step + 1
            ));
        };
        if seen.enters() != expected.enters() || (seen.at() - expected.at()).abs() > TOLERANCE_S {
            return Err(format!(
                "crossing {}: drawn {seen:?}, the server recorded {expected:?} (±{TOLERANCE_S} s)",
                step + 1
            ));
        }
    }
    if shown.len() != server.len() {
        return Err(format!(
            "drawn {} crossings, the server recorded {}: {shown:?}",
            shown.len(),
            server.len()
        ));
    }
    let [outbound, inbound] = passes.as_slice() else {
        return Err(format!("{} passes drawn, not 2", passes.len()));
    };
    let (out_first, out_last) = (outbound[0], *outbound.last().expect("a position"));
    let (in_first, in_last) = (inbound[0], *inbound.last().expect("a position"));
    if distance(out_first, out_last) < TRAVEL_MM || distance(in_first, in_last) < TRAVEL_MM {
        return Err(format!(
            "the figure did not travel across the place: out {out_first:?} → {out_last:?}, \
             in {in_first:?} → {in_last:?}"
        ));
    }
    if distance(in_first, out_last) > DOORWAY_MM || distance(in_last, out_first) > DOORWAY_MM {
        return Err(format!(
            "the way back is not the way out: out {out_first:?} → {out_last:?}, \
             in {in_first:?} → {in_last:?}"
        ));
    }
    Ok(())
}

// ── The processes ───────────────────────────────────────────────────────────────────────────────

/// A child whose output is kept, line by line, and echoed under `tag`; killed when dropped.
struct Watched {
    child: Child,
    lines: Arc<Mutex<Vec<String>>>,
}

impl Watched {
    fn spawn(mut command: Command, tag: &'static str) -> Self {
        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| panic!("{tag} starts: {e}"));
        let lines = Arc::new(Mutex::new(Vec::new()));
        let out: Box<dyn Read + Send> = Box::new(child.stdout.take().expect("piped"));
        let err: Box<dyn Read + Send> = Box::new(child.stderr.take().expect("piped"));
        for (stream, prefix) in [(out, ""), (err, "[stderr] ")] {
            let sink = Arc::clone(&lines);
            std::thread::spawn(move || {
                for line in BufReader::new(stream).lines().map_while(Result::ok) {
                    eprintln!("[{tag}] {prefix}{line}");
                    sink.lock().expect("lines").push(format!("{prefix}{line}"));
                }
            });
        }
        Self { child, lines }
    }

    fn lines(&self) -> Vec<String> {
        self.lines.lock().expect("lines").clone()
    }

    /// Waits for a line that starts with `prefix`; returns it.
    async fn line_starting(&mut self, prefix: &str) -> String {
        let deadline = Instant::now() + PATIENCE;
        loop {
            if let Some(line) = self.lines().into_iter().find(|l| l.starts_with(prefix)) {
                return line;
            }
            if let Some(status) = self.child.try_wait().expect("waitable") {
                panic!(
                    "ended ({status}) before {prefix:?}:\n{}",
                    self.lines().join("\n")
                );
            }
            assert!(
                Instant::now() < deadline,
                "no {prefix:?} line:\n{}",
                self.lines().join("\n")
            );
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Waits for the process to end by itself.
    async fn finish(&mut self) -> (ExitStatus, Vec<String>) {
        let deadline = Instant::now() + PATIENCE;
        loop {
            if let Some(status) = self.child.try_wait().expect("waitable") {
                tokio::time::sleep(Duration::from_millis(200)).await;
                return (status, self.lines());
            }
            assert!(
                Instant::now() < deadline,
                "hung:\n{}",
                self.lines().join("\n")
            );
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

impl Drop for Watched {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The workspace's own interpreter (Milestone D's `interpreter`).
fn interpreter() -> PathBuf {
    let venv =
        std::env::var_os("VIRTUAL_ENV").map_or_else(|| repository().join(".venv"), PathBuf::from);
    let python = if cfg!(windows) {
        venv.join("Scripts").join("python.exe")
    } else {
        venv.join("bin").join("python")
    };
    assert!(
        python.is_file(),
        "{} is missing: run `uv sync --locked` at the repository root first",
        python.display()
    );
    python
}

#[tokio::test]
#[ignore = "needs Godot 4.7 and the Python workspace: cargo test -p mineworld-cli --test agent_with_3d_observer -- --ignored"]
async fn a_3d_client_sees_the_agents_walk_as_the_server_records_it() {
    let pack_dir = repository().join("worlds").join(WORLD);
    let pack = pack_dir.to_str().expect("UTF-8").to_owned();
    let save = SaveDir::new("agent-3d-observer");
    let mut arguments = vec!["server".to_owned(), pack.clone()];
    for root in pack_roots() {
        arguments.push("--packs".to_owned());
        arguments.push(root.to_str().expect("UTF-8").to_owned());
    }
    arguments.extend(["--grant", AGENT_SEAT, "--save", save.path()].map(str::to_owned));
    let borrowed: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let (server, captured) = Server::start_with_env(&borrowed, &[]).await;
    // `tools/cli/src/serve.rs`'s listening line: `http://{address} (ws://{address}/ws)`.
    let ws = format!("ws://{}/ws", server.address);
    let http = format!("http://{}", server.address);
    let grant_line = captured
        .line_starting(&format!("[mineworld] grant {AGENT_SEAT} "))
        .await;
    let grant = grant_line
        .split_whitespace()
        .nth(3)
        .expect("the grant line carries its token")
        .to_owned();
    let nils = genesis_id(&pack_dir, AGENT_SEAT);

    // The observer: out of Elin's door onto the street, then standing there.
    let scratch = mineworld_test_support::scratch!(empty "agent-3d-observer-log");
    let log = scratch.join("observe.jsonl");
    let done = scratch.join("done");
    let mut observer = {
        let mut command = Command::new(godot3d::imported());
        command
            .args([
                "--headless",
                "--path",
                godot3d::PROJECT,
                "res://scenes/slice.tscn",
                "--",
            ])
            .arg("--slice-observe")
            .arg(format!("--server={}", server.address))
            .arg(format!("--seat={OBSERVER_SEAT}"))
            .arg(format!("--world-key={WORLD}"))
            .arg(format!("--invite={INVITE}"))
            .arg("--nickname=observer-3d")
            .arg("--settings=none")
            .arg(format!("--observe-log={}", log.to_str().expect("UTF-8")))
            .arg(format!("--observe-until={}", done.to_str().expect("UTF-8")))
            .args(SLICE_3D_CI);
        Watched::spawn(command, "3d")
    };
    let ready = observer.line_starting("OBSERVE READY ").await;
    let street: Value =
        serde_json::from_str(ready.trim_start_matches("OBSERVE READY ")).expect("JSON");
    let street = id(&street["place"]);
    assert!(!street.is_empty(), "the observer stands somewhere: {ready}");

    // The agent: the scenario's script, as a program, with the seat's grant.
    let mut agent = {
        let mut command = Command::new(interpreter());
        command
            .arg(repository().join("clients/agent/tests/test_play_lakeside.py"))
            .args(["--server", &ws, "--http", &http])
            .current_dir(repository().join("clients/agent/tests"))
            .env("MINEWORLD_GRANT", &grant)
            .env("PYTHONUNBUFFERED", "1")
            // UTF-8 on the pipe on every OS: Windows' default code page cannot print every failure
            // message, and a byte that is not UTF-8 would end the reader's lines.
            .env("PYTHONUTF8", "1");
        Watched::spawn(command, "agent")
    };
    let (status, said) = agent.finish().await;
    assert!(
        status.success() && said.iter().any(|l| l == "LAKESIDE PASS"),
        "the agent's script passes:\n{}",
        said.join("\n")
    );

    // Let the last observation reach the observer, then let it go.
    tokio::time::sleep(Duration::from_secs(3)).await;
    std::fs::write(&done, b"done").expect("the stop file is written");
    let (status, printed) = observer.finish().await;
    assert!(
        status.success() && printed.iter().any(|l| l == "OBSERVE DONE"),
        "the observer ends cleanly"
    );
    assert!(
        !printed.iter().any(|l| l.contains("SCRIPT ERROR")),
        "no script error in the 3D client"
    );
    drop(server);

    let crossings = server_crossings(&save, &nils, &street);
    let drawn = drawn_in(
        &std::fs::read_to_string(&log).expect("the observe log reads"),
        &nils,
        &street,
    );
    let (shown, passes) = drawn_crossings(&drawn);
    eprintln!("AC-i-4 server {crossings:?}");
    eprintln!("AC-i-4 drawn  {shown:?}");
    eprintln!("AC-i-4 passes {passes:?}");
    if let Err(step) = compare(&crossings, &drawn) {
        panic!("AC-i-4: {step}\nserver: {crossings:?}\ndrawn: {drawn:?}");
    }
}

// ── The comparison, without Godot ─────────────────────────────────────────────────────────────

fn line(at: i64, figure: Option<(f64, f64)>) -> Drawn {
    Drawn { at, figure }
}

/// A walk out and back, drawn as the server recorded it: out at 10…18 s, back at 40…48 s.
fn a_faithful_log() -> (Vec<Crossing>, Vec<Drawn>) {
    let server = vec![
        Crossing::Enters(10),
        Crossing::Leaves(18),
        Crossing::Enters(40),
        Crossing::Leaves(48),
    ];
    let drawn = vec![
        line(9, None),
        line(10, Some((18_000.0, 400.0))),
        line(14, Some((24_000.0, 1_800.0))),
        line(17, Some((29_400.0, 3_000.0))),
        line(19, None),
        line(41, Some((29_400.0, 3_000.0))),
        line(45, Some((24_000.0, 1_800.0))),
        line(47, Some((18_200.0, 500.0))),
        line(48, None),
    ];
    (server, drawn)
}

#[test]
fn a_faithful_log_passes() {
    let (server, drawn) = a_faithful_log();
    assert_eq!(compare(&server, &drawn), Ok(()));
}

#[test]
fn a_log_that_drops_a_crossing_fails_naming_it() {
    // M-i-2: the figure is never drawn leaving on the way back.
    let (server, mut drawn) = a_faithful_log();
    drawn.pop();
    let failure = compare(&server, &drawn).expect_err("a dropped crossing fails");
    assert!(failure.contains("crossing 4"), "{failure}");
}

#[test]
fn a_late_crossing_fails_naming_it() {
    let (server, mut drawn) = a_faithful_log();
    drawn[4] = line(18 + TOLERANCE_S + 2, None);
    let failure = compare(&server, &drawn).expect_err("a late crossing fails");
    assert!(failure.contains("crossing 2"), "{failure}");
}

#[test]
fn a_figure_that_never_moves_fails() {
    let (server, mut drawn) = a_faithful_log();
    for entry in &mut drawn {
        if entry.figure.is_some() {
            entry.figure = Some((18_000.0, 400.0));
        }
    }
    let failure = compare(&server, &drawn).expect_err("a still figure fails");
    assert!(failure.contains("did not travel"), "{failure}");
}
