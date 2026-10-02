//! The real binary, the real World Pack, real sockets: the harness every test in this directory uses.
//!
//! Nothing here is mocked and nothing is in-process. `mineworld` is started as the process cargo
//! built, pointed at `worlds/social-cafe` on disk, on an ephemeral port; clients are real WebSocket
//! connections speaking the frames in `server/PROTOCOL.md`. That is deliberate and it is what makes
//! these tests acceptance tests rather than integration tests: what they exercise is the command an
//! operator types.

// Two test binaries share this module and each uses a part of it.
#![allow(dead_code)]

use std::net::SocketAddr;
use std::process::{Child, Command, Stdio};
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

/// The pack every test is pointed at: the repository's own.
pub const PACK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/social-cafe");

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
    pub async fn start(arguments: &[&str]) -> Self {
        let address = free_port().await;
        let listen = address.to_string();
        let process = Command::new(env!("CARGO_BIN_EXE_mineworld"))
            .args(arguments)
            .args(["--listen", &listen])
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("the mineworld binary runs");

        let server = Self { process, address };
        server.wait_until_healthy().await;
        server
    }

    /// Polls `GET /health` until the server answers, or fails the test with what it knows.
    async fn wait_until_healthy(&self) {
        let deadline = Instant::now() + PATIENCE;
        while Instant::now() < deadline {
            if get(self.address, "/health").await.is_some() {
                return;
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

    /// Kills the process with `SIGKILL` — no shutdown, no checkpoint, no flush — and returns how it
    /// ended, so that a test can show the death was real.
    pub fn kill(&mut self) -> std::process::ExitStatus {
        self.process.kill().expect("SIGKILL is delivered");
        self.process.wait().expect("the process is reaped")
    }
}

/// A save directory of its own under the system temporary directory, empty when made and removed
/// when dropped.
pub struct SaveDir {
    path: std::path::PathBuf,
}

impl SaveDir {
    pub fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("mineworld-cli-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        Self { path }
    }

    pub fn path(&self) -> &str {
        self.path.to_str().expect("a UTF-8 temporary path")
    }
}

impl Drop for SaveDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
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
        }
    }

    /// Occupies a seat and returns the observer the server resolved it to, with what it said about
    /// the world.
    pub async fn join(&mut self, seat: &str) -> (EntityId, WorldSummary) {
        self.send(json!({ "t": "join", "seat": seat })).await;
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
                ServerFrame::Observation { .. } => continue,
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
                ServerFrame::Observation { .. } => continue,
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

    /// The next frame the server sends, decoded.
    pub async fn frame(&mut self) -> ServerFrame {
        let message = tokio::time::timeout(PATIENCE, self.socket.next())
            .await
            .expect("a frame arrives")
            .expect("the socket is open")
            .expect("a readable frame");
        let text = match message {
            Message::Text(text) => text,
            other => panic!("the protocol is JSON text: {other:?}"),
        };
        serde_json::from_str(&text)
            .unwrap_or_else(|error| panic!("a server frame: {error} in {text}"))
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

/// An `arrive` request: the protocol-level form of walking somewhere.
///
/// Millimetres as integers, because the contract refuses a float where an `i32` is declared — the
/// loud failure `spike/FINDINGS.md` F9 measured.
pub fn arrive(actor: EntityId, place: EntityId, x: i32, y: i32) -> Value {
    json!({
        "actor": actor,
        "action_type": "arrive",
        "target": null,
        "payload": { "action_type": "arrive", "payload": { "location": {
            "place": { "entity": place.to_string(), "entity_type": "place" },
            "local": { "x": x, "y": y, "z": 0 },
            "facing": null,
        } } },
        "actor_location": null,
    })
}
