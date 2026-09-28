//! `mineworld server worlds/social-cafe` — the acceptance this PR is judged on, run for real.
//!
//! Nothing here is mocked and nothing is in-process. The test starts the **actual binary** cargo
//! built, pointed at the **actual World Pack** in the repository, on an ephemeral port; then it
//! connects a real WebSocket client, occupies the seat the pack offers, and reads what the server
//! sends. That is the whole of A4:
//!
//! ```text
//! mineworld server worlds/social-cafe starts
//! a client connects to it
//! and what it perceives is the world the YAML describes
//! ```
//!
//! The last line is what makes this more than a smoke test: the observation has to name Alice, at the
//! position `people/alice.yaml` authored, with `talk` offered against her and refused for distance —
//! all decided by the server, from a file on disk, through the whole stack.

use std::net::SocketAddr;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use mineworld_server::{ServerFrame, WireObservation};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;

/// How long the test waits for the server to come up, and for a frame to arrive.
const PATIENCE: Duration = Duration::from_secs(20);

/// The pack the command is pointed at: the repository's own.
const PACK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/social-cafe");

/// The real binary, hosting the real pack, killed when the test ends.
struct Server {
    process: Child,
    address: SocketAddr,
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

impl Server {
    /// Starts `mineworld server <pack> --listen 127.0.0.1:<port>` and waits until it answers.
    ///
    /// A port is chosen by binding one and letting it go, rather than by hard-coding: two test
    /// binaries may run at once, and a fixed port makes that a flake nobody can reproduce.
    async fn start(arguments: &[&str]) -> Self {
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
}

/// An address nothing is listening on: bound, read back, released.
async fn free_port() -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("an ephemeral port");
    listener.local_addr().expect("the port it bound")
}

/// One HTTP GET, hand-written, because the CLI's tests need no HTTP client of their own.
async fn get(address: SocketAddr, path: &str) -> Option<String> {
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
fn body(response: &str) -> Value {
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

#[tokio::test]
async fn the_server_starts_from_the_pack_and_says_what_it_is_hosting() {
    let server = Server::start(&["server", PACK]).await;

    let status = body(
        &get(server.address, "/status")
            .await
            .expect("the server answers /status"),
    );

    assert_eq!(
        status["entities"], 4,
        "the world the pack describes: one place and three people — {status}",
    );
    let systems: Vec<&str> = status["systems"]
        .as_array()
        .expect("a list of systems")
        .iter()
        .map(|system| system["system"].as_str().expect("a name"))
        .collect();
    assert_eq!(
        systems,
        ["presence", "conversation"],
        "in the order world.yaml states, which is the order they reduce in",
    );
    assert_eq!(
        status["seats"].as_array().map(Vec::len),
        Some(1),
        "the one seat the pack offers: {status}",
    );
}

#[tokio::test]
async fn a_client_connects_to_the_hosted_pack_and_perceives_the_world_the_yaml_describes() {
    let server = Server::start(&["server", PACK]).await;

    let (mut socket, _) = tokio_tungstenite::connect_async(format!("ws://{}/ws", server.address))
        .await
        .expect("the server upgrades the connection");

    // The seat the pack offers, by the authoring key `world.yaml` names — a client never names an
    // entity, and this is the whole of why the roster is the world's.
    socket
        .send(Message::Text(
            json!({ "t": "join", "seat": "visitor" }).to_string().into(),
        ))
        .await
        .expect("the frame is sent");

    let welcome = next_frame(&mut socket).await;
    let ServerFrame::Welcome {
        seat,
        observer,
        world,
        ..
    } = welcome
    else {
        panic!("the first frame is a welcome: {welcome:?}");
    };
    assert_eq!(seat.as_str(), "visitor");
    assert_eq!(
        observer.raw(),
        4,
        "the visitor is the fourth entity the pack allocates — deterministically, every time",
    );
    assert_eq!(world.entities, 4);

    // And then the world itself, as this observer perceives it. The next frame, not an awaited
    // search: a seated connection is swept on the host's own cadence and nothing else is sent to it.
    let observation: WireObservation = match next_frame(&mut socket).await {
        ServerFrame::Observation { observation, .. } => observation,
        other => panic!("a seated connection is sent observations: {other:?}"),
    };

    let perceived: Vec<u64> = observation
        .entities()
        .iter()
        .map(|entity| entity.id().raw())
        .collect();
    assert_eq!(
        perceived,
        [1, 2, 3, 4],
        "the café and everybody in it, by the ids the pack resolved its keys to",
    );

    let alice = observation
        .entity(mineworld_contracts::EntityId::from_raw(2))
        .expect("alice is perceived");
    let position = alice
        .location()
        .and_then(|location| location.local())
        .expect("the position people/alice.yaml authored");
    assert_eq!(
        (position.x().value(), position.y().value()),
        (1200, 2400),
        "millimetres, exactly as the file wrote them, through the whole stack",
    );

    let talk: Vec<(bool, Option<u64>)> = observation
        .affordances()
        .iter()
        .filter(|affordance| affordance.action_type().as_str() == "talk")
        .map(|affordance| {
            (
                affordance.is_available(),
                affordance.target().map(|t| t.raw()),
            )
        })
        .collect();
    assert_eq!(
        talk.len(),
        2,
        "talk offered against the two other people: {talk:?}",
    );
    assert!(
        talk.iter().all(|(available, _)| !available),
        "and refused for both, because the pack puts the visitor at the door — the server decided \
         that, not the client: {talk:?}",
    );
}

#[tokio::test]
async fn a_seat_the_pack_does_not_offer_is_refused() {
    let server = Server::start(&["server", PACK]).await;

    let (mut socket, _) = tokio_tungstenite::connect_async(format!("ws://{}/ws", server.address))
        .await
        .expect("the server upgrades the connection");
    socket
        .send(Message::Text(
            json!({ "t": "join", "seat": "alice" }).to_string().into(),
        ))
        .await
        .expect("the frame is sent");

    // Alice exists in this world and is not a seat: `world.yaml` offers `visitor` and nothing else, so
    // a client cannot connect *as* her. The roster is the world's, not the client's.
    let frame = next_frame(&mut socket).await;
    assert!(
        matches!(frame, ServerFrame::Refused { .. }),
        "got: {frame:?}",
    );
}

/// The next frame the server sends, decoded.
async fn next_frame(
    socket: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) -> ServerFrame {
    let message = tokio::time::timeout(PATIENCE, socket.next())
        .await
        .expect("a frame arrives")
        .expect("the socket is open")
        .expect("a readable frame");
    let text = match message {
        Message::Text(text) => text,
        other => panic!("the protocol is JSON text: {other:?}"),
    };
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("a server frame: {error} in {text}"))
}
