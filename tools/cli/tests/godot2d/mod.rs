//! Running the real 2D client (`clients/2d`) from a test: the Godot process, what it printed, a world
//! that can be killed and started again on the same port, and a revision-1 stub server that lies.
//!
//! The client prints one fact per line (`clients/2d/scripts/harness/drive.gd`): `REQUEST`, `RESULT`,
//! `PLACE`, `STATE`, `EVIDENCE`, `SHOWN`, `[PASS]`/`[FAIL]` and `drive complete`. A test reads those
//! and checks them against an oracle of its own — the save's fact log, another client's observation,
//! or what the stub knows it said.

#![allow(dead_code)]

use std::io::{BufRead, BufReader};
use std::net::SocketAddr;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex, Once};
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;

use crate::support::{body, get};

/// The client's Godot project.
pub const PROJECT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../clients/2d");
/// The world the client is demonstrated in.
pub const MARKET_TOWN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/market-town");
/// How long one scripted client run may take before the test calls it hung.
pub const RUN_LIMIT: Duration = Duration::from_secs(150);

/// The Godot binary, from `GODOT` or `PATH`. These tests are `#[ignore]`d and run only with
/// `--ignored`; without Godot they fail loudly rather than pass (a test not run is not a pass).
pub fn godot() -> String {
    let binary = std::env::var("GODOT").unwrap_or_else(|_| "godot".to_owned());
    let found = Command::new(&binary)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success());
    assert!(
        found,
        "Godot 4.7 is needed on PATH (or GODOT=…) to run the 2D client"
    );
    binary
}

/// Builds the project's class cache once per test binary (`ADOPTION.md` §1).
pub fn imported() -> String {
    static IMPORT: Once = Once::new();
    let binary = godot();
    IMPORT.call_once(|| {
        let status = Command::new(&binary)
            .args(["--headless", "--path", PROJECT, "--import"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("godot runs");
        assert!(status.success(), "the 2D project imports");
        // A script that does not parse leaves Godot running an empty scene until the run limit;
        // parse the composition root (and through its preloads, every script) first, and say why.
        let parsed = Command::new(&binary)
            .args([
                "--headless",
                "--path",
                PROJECT,
                "--check-only",
                "--script",
                "res://scripts/app.gd",
            ])
            .output()
            .expect("godot runs");
        let said = String::from_utf8_lossy(&parsed.stderr).into_owned()
            + &String::from_utf8_lossy(&parsed.stdout);
        assert!(
            !said.contains("SCRIPT ERROR"),
            "the client's scripts do not parse:\n{said}"
        );
    });
    binary
}

/// One run of the scripted client.
pub struct Drive {
    child: Child,
    lines: Arc<Mutex<Vec<String>>>,
}

impl Drive {
    /// `godot --headless --path clients/2d -- --server=… --seat=… <arguments>`.
    pub fn start(server: SocketAddr, seat: &str, arguments: &[&str]) -> Self {
        let binary = imported();
        let mut child = Command::new(binary)
            .args(["--headless", "--path", PROJECT, "--"])
            .arg(format!("--server={server}"))
            .arg(format!("--seat={seat}"))
            .args(arguments)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("godot runs");
        let stdout = child.stdout.take().expect("piped");
        let lines = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&lines);
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                eprintln!("[2d] {line}");
                sink.lock().expect("lines").push(line);
            }
        });
        Self { child, lines }
    }

    /// Everything printed so far.
    pub fn lines(&self) -> Vec<String> {
        self.lines.lock().expect("lines").clone()
    }

    /// Waits until what was printed satisfies `ready`.
    pub async fn until(&self, what: &str, ready: impl Fn(&[String]) -> bool) {
        let deadline = Instant::now() + RUN_LIMIT;
        while Instant::now() < deadline {
            if ready(&self.lines()) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        panic!(
            "the client never printed {what}; it printed:\n{}",
            self.lines().join("\n")
        );
    }

    /// Waits for the run to end by itself, and returns how and what it printed.
    pub async fn finish(mut self) -> (ExitStatus, Vec<String>) {
        let deadline = Instant::now() + RUN_LIMIT;
        loop {
            if let Some(status) = self.child.try_wait().expect("the process can be waited on") {
                // Let the reader drain the pipe.
                tokio::time::sleep(Duration::from_millis(200)).await;
                return (status, self.lines());
            }
            if Instant::now() > deadline {
                let _ = self.child.kill();
                panic!("the client hung; it printed:\n{}", self.lines().join("\n"));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// `SIGKILL`: no goodbye, no close frame — the client simply stops existing.
    pub fn kill(&mut self) -> ExitStatus {
        self.child.kill().expect("SIGKILL is delivered");
        self.child.wait().expect("reaped")
    }
}

impl Drop for Drive {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The JSON after `prefix` on every line that starts with it.
pub fn tagged(lines: &[String], prefix: &str) -> Vec<Value> {
    lines
        .iter()
        .filter_map(|line| line.strip_prefix(prefix))
        .map(|rest| serde_json::from_str(rest).unwrap_or_else(|e| panic!("{prefix}{rest}: {e}")))
        .collect()
}

/// Every `EVIDENCE` object that has `key`, as its value under that key.
pub fn evidence(lines: &[String], key: &str) -> Vec<Value> {
    tagged(lines, "EVIDENCE ")
        .into_iter()
        .filter_map(|value| value.get(key).cloned())
        .collect()
}

/// The place ids of the `PLACE` lines, in order.
pub fn places(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .filter_map(|line| line.strip_prefix("PLACE "))
        .map(|rest| rest.split(' ').next().unwrap_or("").to_owned())
        .collect()
}

/// Whether the run's own assertions all passed.
pub fn passed(lines: &[String]) -> bool {
    lines.iter().any(|line| line == "drive complete: PASS")
        && !lines.iter().any(|line| line.contains("[FAIL]"))
}

/// A hosted world that a test may kill and start again on the same address.
pub struct World {
    child: Option<Child>,
    pub address: SocketAddr,
    arguments: Vec<String>,
}

impl World {
    /// `mineworld server <arguments> --listen <address>` (a free port when `address` is None).
    pub async fn start(arguments: &[&str], address: Option<SocketAddr>) -> Self {
        let address = match address {
            Some(address) => address,
            None => crate::support::free_port().await,
        };
        let mut world = Self {
            child: None,
            address,
            arguments: arguments.iter().map(|a| (*a).to_owned()).collect(),
        };
        world.restart().await;
        world
    }

    /// Starts the process again, with the same arguments and address.
    pub async fn restart(&mut self) {
        let child = Command::new(env!("CARGO_BIN_EXE_mineworld"))
            .args(&self.arguments)
            .args(["--listen", &self.address.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("the mineworld binary runs");
        self.child = Some(child);
        let deadline = Instant::now() + Duration::from_secs(30);
        while Instant::now() < deadline {
            if get(self.address, "/health").await.is_some() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        panic!("the world did not come up on {}", self.address);
    }

    /// `SIGKILL`, and returns how it ended.
    pub fn kill(&mut self) -> ExitStatus {
        let mut child = self.child.take().expect("running");
        child.kill().expect("SIGKILL is delivered");
        child.wait().expect("reaped")
    }

    /// What the world says it is (`GET /status`).
    pub async fn status(&self) -> Value {
        body(&get(self.address, "/status").await.expect("/status answers"))
    }
}

impl Drop for World {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// How the stub answers the client's `n`-th submitted request (1-based).
#[derive(Clone, Copy, Debug, Default)]
pub struct StubScript {
    /// Answer this request `rejected too_far_away` and leave the observer where they were.
    pub reject: Option<usize>,
    /// After answering this request, move the observer 5 m north, with no request asking for it.
    pub teleport_after: Option<usize>,
    /// Offer `move` as unavailable (`too_far_away`) in every observation.
    pub move_unavailable: bool,
}

/// What the stub saw and said.
#[derive(Debug, Default)]
pub struct StubLog {
    /// Every `submit` frame's request, in arrival order, with when it arrived.
    pub submitted: Vec<(Instant, Value)>,
    /// When the stub teleported the observer, and to where (millimetres).
    pub teleported: Option<(Instant, (i64, i64))>,
    /// The observer's position the stub last told the client.
    pub position: (i64, i64),
}

/// A revision-1 server that seats one client in a bare room and answers by script, so a test can
/// make the world say what a real one would only say by accident (step-13 §8.2, `ARC-47`).
pub async fn stub(script: StubScript) -> (SocketAddr, Arc<Mutex<StubLog>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("a port");
    let address = listener.local_addr().expect("bound");
    let log = Arc::new(Mutex::new(StubLog {
        position: (3000, 2000),
        ..StubLog::default()
    }));
    let shared = Arc::clone(&log);
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("the client connects");
        let mut socket = tokio_tungstenite::accept_async(stream)
            .await
            .expect("a WebSocket");
        let mut seq = 0_u64;
        let mut tick = tokio::time::interval(Duration::from_millis(100));
        let mut seated = false;
        loop {
            tokio::select! {
                _ = tick.tick() => {
                    if seated {
                        seq += 1;
                        let frame = stub_observation(seq, shared.lock().expect("log").position, script.move_unavailable);
                        if socket.send(Message::Text(frame.to_string().into())).await.is_err() {
                            return;
                        }
                    }
                }
                incoming = socket.next() => {
                    let Some(Ok(Message::Text(text))) = incoming else { return };
                    let frame: Value = serde_json::from_str(&text).expect("JSON");
                    match frame["t"].as_str() {
                        Some("join") => {
                            seated = true;
                            let welcome = json!({ "t": "welcome", "protocol": 1, "seat": frame["seat"], "observer": "9",
                                "world": { "protocol": 1, "instance": "5705b0000000000000000000000057ab", "at": 0,
                                    "entities": 2, "systems": [], "seats": [frame["seat"]], "clients": 1,
                                    "observations_dropped": 0, "deferrals_unscheduled": 0, "faults": 0, "revision": null } });
                            let _ = socket.send(Message::Text(welcome.to_string().into())).await;
                        }
                        Some("submit") => {
                            let request = frame["request"].clone();
                            let (n, result, teleport) = {
                                let mut log = shared.lock().expect("log");
                                log.submitted.push((Instant::now(), request.clone()));
                                let n = log.submitted.len();
                                let result = if script.reject == Some(n) {
                                    json!({ "rejected": "too_far_away" })
                                } else {
                                    let to = &request["payload"]["payload"]["to"]["local"];
                                    log.position = (to["x"].as_i64().expect("x"), to["y"].as_i64().expect("y"));
                                    json!({ "accepted": { "events": [n.to_string()] } })
                                };
                                (n, result, script.teleport_after == Some(n))
                            };
                            let answer = json!({ "t": "result", "token": frame["token"], "action_id": n.to_string(), "result": result });
                            let _ = socket.send(Message::Text(answer.to_string().into())).await;
                            if teleport {
                                let mut log = shared.lock().expect("log");
                                log.position.1 += 5000;
                                log.teleported = Some((Instant::now(), log.position));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    });
    (address, log)
}

fn stub_observation(seq: u64, position: (i64, i64), move_unavailable: bool) -> Value {
    let place = json!({ "entity": "1", "entity_type": "place" });
    let here = json!({ "place": place, "local": { "x": position.0, "y": position.1, "z": 0 }, "facing": null });
    let requirement = json!({ "place": "any", "within_range": null, "requires_line_of_access": false,
        "requires_target_available": false });
    let offer = if move_unavailable {
        json!({ "action_type": "move", "target": null, "available": false,
            "unavailable_reason": "too_far_away", "requirement": requirement })
    } else {
        json!({ "action_type": "move", "target": null, "available": true,
            "unavailable_reason": null, "requirement": requirement })
    };
    json!({ "t": "observation", "seq": seq, "revision": null, "observation": {
        "observer": "9", "at": 0, "self_location": here,
        "entities": [
            { "id": "1", "entity_type": "place", "location": null, "tags": ["room"], "components": [] },
            { "id": "9", "entity_type": "person", "location": here, "tags": [], "components": [
                { "component_type": "display-name", "entity": "9", "payload": { "name": "Stub Person" }, "schema_version": 1 } ] }
        ],
        "relations": [], "events": [], "affordances": [offer] } })
}
