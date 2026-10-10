//! The real binary, the real World Pack, real sockets: the harness every test in this directory uses.
//!
//! Nothing here is mocked and nothing is in-process. `mineworld` is started as the process cargo
//! built, pointed at `worlds/social-cafe` on disk, on an ephemeral port; clients are real WebSocket
//! connections speaking the frames in `server/PROTOCOL.md`. That is deliberate and it is what makes
//! these tests acceptance tests rather than integration tests: what they exercise is the command an
//! operator types.

// Two test binaries share this module and each uses a part of it.
#![allow(dead_code)]

use std::io::{BufRead, BufReader, Read};
use std::net::SocketAddr;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use mineworld_contracts::{ActionId, ActionResult, EntityId, EventId, PerceivedEntity};
use mineworld_conversation::ConversationHistory;
use mineworld_server::{ServerFrame, WireObservation, WorldRevision, WorldSummary};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;

/// How long a test waits for the server to come up, and for a frame to arrive.
pub const PATIENCE: Duration = Duration::from_secs(20);

/// The invite every test server is started with, and every test client joins with.
pub const INVITE: &str = "cli-test-invite-3f9c0a1b";

/// A revision-2 join frame (`PROTOCOL.md` §2) for a seat, with the test invite.
pub fn join_frame(seat: &str) -> Value {
    json!({ "t": "join", "protocol": 2, "invite": INVITE, "nickname": format!("{seat}-player"),
            "seat": seat })
}

/// The pack every test is pointed at: the repository's own.
pub const PACK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/social-cafe");

/// The twelve-person town, for the tests in which in-server controllers drive a whole town.
pub const MARKET_PACK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/market-town");

/// The real binary, hosting the real pack, killed when the test ends.
pub struct Server {
    process: Child,
    pub address: SocketAddr,
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

impl Server {
    /// Starts `mineworld <arguments> --listen 127.0.0.1:<port>` and waits until it answers.
    ///
    /// A port is chosen by binding one and letting it go, rather than by hard-coding: two test
    /// binaries may run at once, and a fixed port makes that a flake nobody can reproduce.
    ///
    /// Every server is started with [`INVITE`] unless the arguments name an invite themselves, so a
    /// test's [`Client::join`] is admitted (`PROTOCOL.md` §4.1).
    pub async fn start(arguments: &[&str]) -> Self {
        let invite: &[&str] = if arguments.contains(&"--invite") {
            &[]
        } else {
            &["--invite", INVITE]
        };
        let configure = |command: &mut Command| {
            command
                .args(arguments)
                .args(invite)
                .env_remove("MINEWORLD_INVITE")
                .env_remove("MINEWORLD_ADMIN_TOKEN");
        };
        Self::launch(configure, false).await.0
    }

    /// Like [`Server::start`] (the test invite unless the arguments name one), with these extra
    /// environment variables, keeping everything the server prints in memory (step-12 DA-9).
    pub async fn start_with_env(arguments: &[&str], env: &[(&str, &str)]) -> (Self, Captured) {
        let invite: &[&str] = if arguments.contains(&"--invite") {
            &[]
        } else {
            &["--invite", INVITE]
        };
        let configure = |command: &mut Command| {
            command
                .args(arguments)
                .args(invite)
                .env_remove("MINEWORLD_INVITE")
                .env_remove("MINEWORLD_ADMIN_TOKEN")
                .envs(env.iter().copied());
        };
        let (server, captured) = Self::launch(configure, true).await;
        (server, captured.expect("captured output"))
    }

    /// Starts the binary on a free port and waits until it answers.
    ///
    /// A port chosen by binding and releasing it can be taken by another process before the server
    /// binds it — another test binary, or another worktree's tests on the same machine. A server that
    /// exits before it answers is therefore started again on another port, a bounded number of times;
    /// one that is running but silent fails the test.
    async fn launch(configure: impl Fn(&mut Command), capture: bool) -> (Self, Option<Captured>) {
        const ATTEMPTS: usize = 3;
        for _ in 0..ATTEMPTS {
            let address = free_port().await;
            let mut command = Command::new(env!("CARGO_BIN_EXE_mineworld"));
            configure(&mut command);
            command.args(["--listen", &address.to_string()]);
            if capture {
                command.stdout(Stdio::piped()).stderr(Stdio::piped());
            } else {
                command.stdout(Stdio::null()).stderr(Stdio::inherit());
            }
            let mut process = command.spawn().expect("the mineworld binary runs");
            let captured = capture.then(|| Captured {
                stdout: drain(process.stdout.take().expect("piped stdout")),
                stderr: drain(process.stderr.take().expect("piped stderr")),
            });
            let mut server = Self { process, address };
            if server.answers().await {
                return (server, captured);
            }
        }
        panic!("the server exited before answering /health, {ATTEMPTS} times");
    }

    /// Polls `GET /health` until the server answers (`true`) or exits (`false`), or fails the test.
    async fn answers(&mut self) -> bool {
        let deadline = Instant::now() + PATIENCE;
        while Instant::now() < deadline {
            if get(self.address, "/health").await.is_some() {
                return true;
            }
            if self.process.try_wait().ok().flatten().is_some() {
                return false;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        panic!("the server did not answer /health within {PATIENCE:?}");
    }

    /// What the world says it is, answered by the world itself.
    pub async fn status(&self) -> Value {
        body(
            &get(self.address, "/status")
                .await
                .expect("the server answers /status"),
        )
    }

    /// Starts `mineworld <arguments> --listen 127.0.0.1:<port>` with exactly these environment
    /// variables for the invite (`MINEWORLD_INVITE` is otherwise removed) and **no** invite of the
    /// harness's own, keeping everything it writes to stdout and stderr in memory — for the tests
    /// that read what the server prints (step-12 §15.4 SA-3, SA-5). Nothing is written to disk.
    pub async fn start_captured(arguments: &[&str], invite_env: Option<&str>) -> (Self, Captured) {
        let configure = |command: &mut Command| {
            command
                .args(arguments)
                .env_remove("MINEWORLD_INVITE")
                .env_remove("MINEWORLD_ADMIN_TOKEN");
            if let Some(invite) = invite_env {
                command.env("MINEWORLD_INVITE", invite);
            }
        };
        let (server, captured) = Self::launch(configure, true).await;
        (server, captured.expect("captured output"))
    }

    /// Kills the process with `SIGKILL` — no shutdown, no checkpoint, no flush — and returns how it
    /// ended, so that a test can show the death was real.
    pub fn kill(&mut self) -> std::process::ExitStatus {
        self.process.kill().expect("SIGKILL is delivered");
        self.process.wait().expect("the process is reaped")
    }

    /// Stops the process with `SIGINT`, as an operator's Ctrl-C does — the graceful stop that prints
    /// the shutdown statistics (step-12 SD-B11) — and returns how it ended.
    pub fn interrupt(&mut self) -> std::process::ExitStatus {
        // The shell's builtin `kill`, not a `kill` binary: a minimal CI image has a shell and may have
        // no procps (and this crate takes no libc dependency to signal a child).
        let sent = Command::new("sh")
            .args(["-c", &format!("kill -INT {}", self.process.id())])
            .status()
            .expect("sh runs");
        assert!(sent.success(), "SIGINT is delivered");
        self.process.wait().expect("the process is reaped")
    }
}

/// The first file under `directory` whose bytes contain `needle`, if any — for the claim that a
/// secret is in no byte of a save.
pub fn file_containing(directory: &std::path::Path, needle: &[u8]) -> Option<std::path::PathBuf> {
    for entry in std::fs::read_dir(directory).expect("the directory exists") {
        let path = entry.expect("an entry").path();
        if path.is_dir() {
            if let Some(found) = file_containing(&path, needle) {
                return Some(found);
            }
        } else if std::fs::read(&path)
            .expect("readable")
            .windows(needle.len())
            .any(|window| window == needle)
        {
            return Some(path);
        }
    }
    None
}

/// What a captured server has written so far, one string per line, kept in memory.
pub struct Captured {
    stdout: Arc<Mutex<Vec<String>>>,
    stderr: Arc<Mutex<Vec<String>>>,
}

impl Captured {
    /// Every stdout line so far.
    pub fn stdout(&self) -> Vec<String> {
        self.stdout
            .lock()
            .expect("the reader thread is sound")
            .clone()
    }

    /// Every stderr line so far.
    pub fn stderr(&self) -> Vec<String> {
        self.stderr
            .lock()
            .expect("the reader thread is sound")
            .clone()
    }

    /// Waits until a stdout line begins with `prefix`, and returns it.
    pub async fn line_starting(&self, prefix: &str) -> String {
        let deadline = Instant::now() + PATIENCE;
        while Instant::now() < deadline {
            if let Some(line) = self
                .stdout()
                .into_iter()
                .find(|line| line.starts_with(prefix))
            {
                return line;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        panic!(
            "no stdout line began with {prefix:?} within {PATIENCE:?}: {:?}",
            self.stdout()
        );
    }
}

/// Reads a pipe on a thread of its own until it closes, so the child never blocks on a full pipe.
fn drain(pipe: impl Read + Send + 'static) -> Arc<Mutex<Vec<String>>> {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&lines);
    std::thread::spawn(move || {
        for line in BufReader::new(pipe).lines().map_while(Result::ok) {
            sink.lock().expect("the test is sound").push(line);
        }
    });
    lines
}

/// A save path of its own that does not exist yet, removed when dropped (scratch, DEP-29).
pub struct SaveDir {
    scratch: mineworld_test_support::Scratch,
}

impl SaveDir {
    pub fn new(name: &str) -> Self {
        Self {
            scratch: mineworld_test_support::scratch!(name),
        }
    }

    pub fn path(&self) -> &str {
        self.scratch.to_str().expect("a UTF-8 temporary path")
    }
}

/// Runs `mineworld <arguments>` to completion and returns its exit status and standard output.
pub fn run_command(arguments: &[&str]) -> (std::process::ExitStatus, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_mineworld"))
        .args(arguments)
        .stderr(Stdio::inherit())
        .output()
        .expect("the mineworld binary runs");
    (
        output.status,
        String::from_utf8(output.stdout).expect("UTF-8 output"),
    )
}

/// An address nothing is listening on: bound, read back, released.
pub async fn free_port() -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("an ephemeral port");
    listener.local_addr().expect("the port it bound")
}

/// One HTTP GET, hand-written, because these tests need no HTTP client of their own.
pub async fn get(address: SocketAddr, path: &str) -> Option<String> {
    let mut socket = TcpStream::connect(address).await.ok()?;
    socket
        .write_all(
            format!("GET {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n")
                .as_bytes(),
        )
        .await
        .ok()?;
    let mut response = String::new();
    socket.read_to_string(&mut response).await.ok()?;
    response.contains(" 200 ").then_some(response)
}

/// One HTTP answer from the admin surface: its status and its JSON body (`Null` if none).
pub struct HttpAnswer {
    pub status: u16,
    pub body: Value,
}

/// One HTTP request to `path`, with `Authorization: Bearer <token>` when a token is given and a JSON
/// body for a `POST` — the admin surface's request helper (step-12 §19.2).
pub async fn admin(
    address: SocketAddr,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: &str,
) -> HttpAnswer {
    let mut socket = TcpStream::connect(address).await.expect("connects");
    let mut request =
        format!("{method} {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n");
    if let Some(token) = token {
        request.push_str(&format!("Authorization: Bearer {token}\r\n"));
    }
    if method == "POST" {
        request.push_str(&format!(
            "Content-Type: application/json\r\nContent-Length: {}\r\n",
            body.len()
        ));
    }
    request.push_str("\r\n");
    if method == "POST" {
        request.push_str(body);
    }
    socket
        .write_all(request.as_bytes())
        .await
        .expect("the request is sent");
    let mut response = String::new();
    socket
        .read_to_string(&mut response)
        .await
        .expect("the response is read");
    let status = response
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .unwrap_or_else(|| panic!("an HTTP status line: {response}"));
    let text = response.split_once("\r\n\r\n").map_or("", |(_, body)| body);
    HttpAnswer {
        status,
        body: serde_json::from_str(text.trim()).unwrap_or(Value::Null),
    }
}

/// Whether a frame belongs to the stream a seated connection is sent unasked — an observation, or a
/// `clock` (`PROTOCOL.md` §5.9) — rather than answering something it said.
pub fn streamed(frame: &ServerFrame) -> bool {
    matches!(
        frame,
        ServerFrame::Observation { .. } | ServerFrame::Clock { .. }
    )
}

/// The body of an HTTP response, as JSON.
pub fn body(response: &str) -> Value {
    let body = response
        .split_once("\r\n\r\n")
        .map_or(response, |(_, body)| body);
    // A chunked response arrives as `<len>\r\n<json>\r\n0\r\n\r\n`; the JSON is the longest line that
    // parses, which is enough for a test and needs no HTTP client.
    body.lines()
        .filter_map(|line| serde_json::from_str::<Value>(line.trim()).ok())
        .next()
        .unwrap_or_else(|| panic!("a JSON body: {body}"))
}

type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// One connected client, and the little it is allowed to say.
pub struct Client {
    socket: Socket,
    /// What this connection's observer is, once it has a seat.
    pub observer: Option<EntityId>,
    submitted: u32,
    /// The whole observation this connection holds and its `seq`, which a `delta` applies to
    /// (`PROTOCOL.md` §5.3).
    held: Option<(u64, WireObservation)>,
}

impl Client {
    /// Opens a connection. Nothing streams yet: a connection has to ask for a seat first.
    pub async fn connect(address: SocketAddr) -> Self {
        let (socket, _) = tokio_tungstenite::connect_async(format!("ws://{address}/ws"))
            .await
            .expect("the server upgrades the connection");
        Self {
            socket,
            observer: None,
            submitted: 0,
            held: None,
        }
    }

    /// Occupies a seat and returns the observer the server resolved it to, with what it said about
    /// the world.
    pub async fn join(&mut self, seat: &str) -> (EntityId, WorldSummary) {
        self.send(join_frame(seat)).await;
        match self.frame().await {
            ServerFrame::Welcome {
                observer, world, ..
            } => {
                self.observer = Some(observer);
                (observer, world)
            }
            other => panic!("a join is answered with a welcome, and this was {other:?}"),
        }
    }

    /// The next observation, skipping anything else.
    pub async fn observation(&mut self) -> WireObservation {
        self.perceived().await.1
    }

    /// The next observation with the persisted revision its frame named (`PROTOCOL.md` §5).
    pub async fn perceived(&mut self) -> (Option<WorldRevision>, WireObservation) {
        loop {
            if let ServerFrame::Observation {
                revision,
                observation,
                ..
            } = self.frame().await
            {
                return (revision, observation);
            }
        }
    }

    /// Submits one request and returns the identity the server allocated and what the world answered.
    ///
    /// The token is this client's own and is checked on the way back, because pairing an answer with
    /// a request is what it is for.
    pub async fn submit(&mut self, request: Value) -> (ActionId, ActionResult) {
        self.submitted += 1;
        let token = format!("t{}", self.submitted);
        self.send(json!({ "t": "submit", "token": token, "request": request }))
            .await;
        loop {
            match self.frame().await {
                ServerFrame::Result {
                    token: echoed,
                    action_id,
                    result,
                } => {
                    assert_eq!(echoed.as_str(), token, "an answer carries its own token");
                    return (action_id, result);
                }
                frame if streamed(&frame) => continue,
                other => panic!("a submission is answered or refused: {other:?}"),
            }
        }
    }

    /// Submits a request the protocol should refuse, and returns why.
    ///
    /// A refusal is not a rejection: a rejection is the world's considered answer to a well-formed
    /// request, and it arrives inside a result (`PROTOCOL.md` §5).
    pub async fn submit_refused(&mut self, request: Value) -> mineworld_server::RefusalCode {
        self.submitted += 1;
        let token = format!("t{}", self.submitted);
        self.send(json!({ "t": "submit", "token": token, "request": request }))
            .await;
        loop {
            match self.frame().await {
                ServerFrame::Refused { code, .. } => return code,
                frame if streamed(&frame) => continue,
                other => panic!("expected this frame to be refused: {other:?}"),
            }
        }
    }

    /// Submits a request and expects it to be accepted, returning the facts it caused.
    pub async fn submit_accepted(&mut self, request: Value) -> (ActionId, Vec<EventId>) {
        let (action_id, result) = self.submit(request).await;
        match result {
            ActionResult::Accepted { events } => (action_id, events),
            other => panic!("expected the world to accept this request: {other:?}"),
        }
    }

    /// Submits each stride of a [`walk`] in order, requiring each to be accepted, and returns every
    /// answer — one per stride, so a caller can count the requests a walk took.
    pub async fn walk_accepted(&mut self, strides: Vec<Value>) -> Vec<(ActionId, Vec<EventId>)> {
        let mut answers = Vec::with_capacity(strides.len());
        for stride in strides {
            answers.push(self.submit_accepted(stride).await);
        }
        answers
    }

    /// Waits until this observer's observation satisfies `ready`, or fails the test.
    ///
    /// Polling the stream rather than sleeping: the world is swept on its own cadence, and what a
    /// test is waiting for is a *state*, not a duration.
    pub async fn observation_where(
        &mut self,
        what: &str,
        mut ready: impl FnMut(&WireObservation) -> bool,
    ) -> WireObservation {
        let deadline = Instant::now() + PATIENCE;
        let mut last = None;
        while Instant::now() < deadline {
            let observation = self.observation().await;
            if ready(&observation) {
                return observation;
            }
            last = Some(observation);
        }
        panic!("no observation with {what} arrived within {PATIENCE:?}; the last one was {last:?}");
    }

    /// Sends one frame.
    pub async fn send(&mut self, frame: Value) {
        self.socket
            .send(Message::Text(frame.to_string().into()))
            .await
            .expect("the frame is sent");
    }

    /// The next frame the server sends, decoded — with a `delta` applied to the observation this
    /// client holds and handed on as the whole observation it describes, as every client does
    /// (`PROTOCOL.md` §5.3). A delta whose base is not the frame held fails the test.
    pub async fn frame(&mut self) -> ServerFrame {
        let text = self.text().await;
        let frame: ServerFrame = serde_json::from_str(&text)
            .unwrap_or_else(|error| panic!("a server frame: {error} in {text}"));
        match frame {
            ServerFrame::Observation {
                seq,
                revision,
                acted_through,
                observation,
            } => {
                self.held = Some((seq, observation.clone()));
                ServerFrame::Observation {
                    seq,
                    revision,
                    acted_through,
                    observation,
                }
            }
            ServerFrame::Delta {
                seq,
                base,
                revision,
                acted_through,
                delta,
            } => {
                let (held_seq, held) = self.held.as_ref().expect("a delta follows an observation");
                assert_eq!(base, *held_seq, "a delta applies to the frame held");
                let observation =
                    mineworld_server::protocol::delta::apply(held, &delta).expect("it applies");
                self.held = Some((seq, observation.clone()));
                ServerFrame::Observation {
                    seq,
                    revision,
                    acted_through,
                    observation,
                }
            }
            other => other,
        }
    }

    /// The next frame the server sends, as the text it sent — for a test that counts bytes.
    pub async fn text(&mut self) -> String {
        let message = tokio::time::timeout(PATIENCE, self.socket.next())
            .await
            .expect("a frame arrives")
            .expect("the socket is open")
            .expect("a readable frame");
        match message {
            Message::Text(text) => text.to_string(),
            other => panic!("the protocol is JSON text: {other:?}"),
        }
    }

    /// The next frame that is not part of the stream (an observation or a clock).
    pub async fn answer(&mut self) -> ServerFrame {
        loop {
            let frame = self.frame().await;
            if !streamed(&frame) {
                return frame;
            }
        }
    }

    /// Joins with a frame of the test's own making (a `resume`, a `take_over`) and returns the
    /// server's answer to it: a welcome or a refusal. A welcome records the observer.
    pub async fn join_as(&mut self, frame: Value) -> ServerFrame {
        self.send(frame).await;
        let answer = self.answer().await;
        if let ServerFrame::Welcome { observer, .. } = &answer {
            self.observer = Some(*observer);
        }
        answer
    }

    /// Closes the connection, as a killed client does.
    pub async fn disconnect(mut self) {
        let _ = self.socket.close(None).await;
    }
}

// ---------------------------------------------------------------------------------------------
// Reading an observation, the way a client does.
// ---------------------------------------------------------------------------------------------

/// The entity in this observation whose tags include `tag`, if the observation lists one.
///
/// How a client finds Alice: by what the world says about her, never by a hard-coded identity. The
/// pack gives her the `barista` tag, and tags are what reach a client through an observation
/// (`worlds/social-cafe/people/alice.yaml`).
pub fn tagged(observation: &WireObservation, tag: &str) -> Option<EntityId> {
    observation
        .entities()
        .iter()
        .find(|entity| entity.tags().iter().any(|carried| carried.as_str() == tag))
        .map(PerceivedEntity::id)
}

/// Whether the server says this observer may `talk` to that person right now.
pub fn may_talk_to(observation: &WireObservation, target: EntityId) -> bool {
    observation.affordances().iter().any(|affordance| {
        affordance.action_type().as_str() == "talk"
            && affordance.target() == Some(target)
            && affordance.is_available()
    })
}

/// What this observer has been told, as the observation disclosed it.
///
/// Decoded with the conversation pack's own component type rather than by reading JSON by hand, so a
/// change to the component's shape is a compile error here rather than a test that keeps passing
/// while asserting nothing.
pub fn own_history(observation: &WireObservation) -> Option<ConversationHistory> {
    let me = observation.entity(observation.observer())?;
    let record = me
        .components()
        .iter()
        .find(|record| record.component_type().as_str() == "conversation-history")?;
    serde_json::from_value(record.payload().clone()).ok()
}

/// A `talk` request, exactly as a client builds one: no identity, no instant, and the action type in
/// both of the places the contract requires it.
pub fn talk(actor: EntityId, target: EntityId, said: &str) -> Value {
    json!({
        "actor": actor,
        "action_type": "talk",
        "target": target,
        "payload": { "action_type": "talk", "payload": { "utterance": said } },
        "actor_location": null,
    })
}

/// One `move` request to a position in `place`: the protocol-level form of a stride.
///
/// Millimetres as integers, because the contract refuses a float where an `i32` is declared — the
/// loud failure `spike/FINDINGS.md` F9 measured.
pub fn stride(actor: EntityId, place: EntityId, x: i32, y: i32) -> Value {
    json!({
        "actor": actor,
        "action_type": "move",
        "target": null,
        "payload": { "action_type": "move", "payload": { "to": {
            "place": { "entity": place.to_string(), "entity_type": "place" },
            "local": { "x": x, "y": y, "z": 0 },
            "facing": null,
        } } },
        "actor_location": null,
    })
}

/// The longest stride a client sends, in millimetres: the 2 m `server/PROTOCOL.md` §6.2 publishes.
/// A literal here, not the movement pack's constant, so a change to the server's bound is caught by
/// these tests rather than followed by them (`ARC-23` rule 2).
pub const STRIDE_MM: i64 = 2_000;

/// A straight walk inside `place` from `from` to `to`, as the `move` requests a client sends: the
/// fewest equal strides, each at most [`STRIDE_MM`], ending exactly at `to`.
///
/// Integer arithmetic only. The waypoint after `k` of `n` strides is `from + (to − from)·k / n`,
/// rounded toward zero; `n` grows until every stride, measured by its squared length, fits — so
/// rounding can never make one stride a millimetre too long.
pub fn walk(actor: EntityId, place: EntityId, from: (i32, i32), to: (i32, i32)) -> Vec<Value> {
    let (dx, dy) = (i64::from(to.0 - from.0), i64::from(to.1 - from.1));
    let point = |k: i64, n: i64| {
        let x = i64::from(from.0) + dx * k / n;
        let y = i64::from(from.1) + dy * k / n;
        (x, y)
    };
    let fits = |n: i64| {
        (1..=n).all(|k| {
            let (a, b) = (point(k - 1, n), point(k, n));
            let (sx, sy) = (b.0 - a.0, b.1 - a.1);
            sx * sx + sy * sy <= STRIDE_MM * STRIDE_MM
        })
    };
    let n = (1..)
        .find(|n| fits(*n))
        .expect("some number of strides fits");
    (1..=n)
        .map(|k| {
            let (x, y) = point(k, n);
            stride(
                actor,
                place,
                i32::try_from(x).expect("on the map"),
                i32::try_from(y).expect("on the map"),
            )
        })
        .collect()
}
