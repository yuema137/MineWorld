//! The renderer-integration spike's demo server.
//!
//! One authoritative world (`world.rs`) and one WebSocket endpoint. It does not use
//! `mineworld-kernel`: the kernel is being written elsewhere and this spike exists to test the
//! *contracts*, which are merged and stable.
//!
//! ```text
//! server  ── Observation ──►  client        every 100 ms
//! server  ◄── ActionIntent ──  client        whatever the player did
//! server  ── ActionResult ──►  client        the server's answer, and only the server's
//! ```
//!
//! # There is no wire encoder here any more
//!
//! This server used to carry `wire.rs`: a hand-written structural mirror of every contract shape
//! that rendered 64-bit ids as decimal strings at the protocol boundary, because `DD-15` assigned
//! the problem there. `FINDINGS.md` F2 measured what that encoder could not reach — a
//! `ComponentRecord` or `EventRecord` payload, which is where a real System Pack puts its ids —
//! and `mineworld-contracts` now encodes the four opaque identities itself, keyed on
//! `Serializer::is_human_readable()`. So the frames below are the contract's own `serde_json`
//! output, unmodified, and the ids in them are already decimal strings wherever they appear,
//! payloads included. The deletion of `wire.rs` is the observable result of that fix.
//!
//! Every request the server accepts is written to `spike/evidence/intents.jsonl` in the contract's
//! own canonical JSON, tagged by which client sent it. That file is the `AC-13` parity evidence: the
//! two `talk` requests are compared field by field in `parity.json`.
//!
//! # A client submits a request; this server allocates the identity
//!
//! The clients used to send a whole `ActionIntent`, `action_id` and all, which `FINDINGS.md` F4
//! recorded as a defect: `ids.rs` says allocation belongs to the world, and two clients with local
//! counters collide on their first action. They now send an `ActionRequest` — no identity, no clock —
//! and `SpikeWorld::allocate` turns it into the intent this server dispatches.

mod vocabulary;
mod world;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::any;
use futures_util::{SinkExt, StreamExt};
use mineworld_contracts::ActionRequest;
use serde_json::{Value, json};
use tokio::sync::Mutex;

use world::SpikeWorld;

/// Shared state: the world, and the intents each client has submitted.
struct Shared {
    world: SpikeWorld,
    /// The last `talk` request each client tag submitted, in canonical contract JSON.
    talks: std::collections::BTreeMap<String, Value>,
}

type App = Arc<Mutex<Shared>>;

fn evidence_dir() -> PathBuf {
    std::env::var("SPIKE_EVIDENCE").map_or_else(|_| PathBuf::from("../evidence"), PathBuf::from)
}

#[tokio::main]
async fn main() {
    if std::env::args().any(|argument| argument == "--dump") {
        let world = SpikeWorld::new();
        let observation = world.observation();
        println!(
            "{}",
            serde_json::to_string_pretty(&observation).expect("an observation serializes")
        );
        return;
    }
    let state: App = Arc::new(Mutex::new(Shared {
        world: SpikeWorld::new(),
        talks: std::collections::BTreeMap::new(),
    }));

    let app = Router::new()
        .route("/ws", any(upgrade))
        .with_state(Arc::clone(&state));

    let address: SocketAddr = "127.0.0.1:7878".parse().expect("a literal address");
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("port 7878 is free");
    println!("[server] listening on ws://{address}/ws");

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .expect("the server runs");
}

async fn upgrade(ws: WebSocketUpgrade, State(state): State<App>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| session(socket, state))
}

/// One client's whole conversation with the world.
async fn session(socket: WebSocket, state: App) {
    {
        let mut shared = state.lock().await;
        shared.world.reset_player();
        println!("[server] client connected; player reset to the door");
    }

    let (mut outgoing, mut incoming) = socket.split();
    let mut ticker = tokio::time::interval(Duration::from_millis(100));

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let frame = {
                    let mut shared = state.lock().await;
                    shared.world.tick();
                    observation_frame(&shared.world)
                };
                if outgoing.send(Message::Text(frame.into())).await.is_err() {
                    break;
                }
            }
            received = incoming.next() => {
                let Some(Ok(message)) = received else { break };
                let Message::Text(text) = message else { continue };
                let Some(reply) = handle(&state, text.as_str()).await else { continue };
                if outgoing.send(Message::Text(reply.into())).await.is_err() {
                    break;
                }
            }
        }
    }
    println!("[server] client disconnected");
}

/// Serializes the current observation — once, because there is only one encoding now.
///
/// What goes on the wire is exactly what the contract's own `serde` produces. Every `EntityId`,
/// `EventId`, `ActionId` and `ProcessId` in it is a decimal string, including the ones inside a
/// component or event payload, which is the half a protocol-level encoder could not reach.
fn observation_frame(world: &SpikeWorld) -> String {
    let observation = world.observation();
    let value = serde_json::to_value(&observation).expect("an observation serializes");
    json!({ "t": "observation", "observation": value }).to_string()
}

/// Handles one client frame, returning the reply to send if there is one.
async fn handle(state: &App, text: &str) -> Option<String> {
    let Ok(frame) = serde_json::from_str::<Value>(text) else {
        return Some(json!({"t": "error", "detail": "unparseable frame"}).to_string());
    };
    match frame.get("t").and_then(Value::as_str) {
        Some("note") => {
            println!(
                "[client] {}",
                frame.get("text").and_then(Value::as_str).unwrap_or("")
            );
            None
        }
        Some("request") => Some(handle_request(state, &frame).await),
        _ => Some(json!({"t": "error", "detail": "unknown frame type"}).to_string()),
    }
}

async fn handle_request(state: &App, frame: &Value) -> String {
    let tag = frame
        .get("client")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_owned();
    let Some(sent) = frame.get("request") else {
        return json!({"t": "error", "detail": "request frame carries no request"}).to_string();
    };

    // The contract reads its own encoding, and is the only thing that decides whether the frame is
    // well formed. What arrives has no `action_id` and no `issued_at`: there is nothing here for a
    // client to have invented.
    let request = match serde_json::from_value::<ActionRequest<Value>>(sent.clone()) {
        Ok(request) => request,
        Err(error) => {
            println!("[server] rejected a malformed request from {tag}: {error}");
            return json!({"t": "error", "detail": error.to_string()}).to_string();
        }
    };

    // Canonical form: what the contract itself says this *request* is. This — not the intent the
    // world builds from it — is what the AC-13 parity comparison is made of, because it is what each
    // client actually asked for.
    let canonical = serde_json::to_value(&request).expect("a request serializes");
    let action_type = request.action_type().to_string();

    let (action_id, result) = {
        let mut shared = state.lock().await;
        let intent = shared.world.allocate(request);
        let action_id = intent.action_id();
        let result = shared.world.resolve(&intent);
        if action_type == "talk" {
            shared.talks.insert(tag.clone(), canonical.clone());
            let talks = shared.talks.clone();
            record_talk(&tag, &canonical);
            compare(&talks);
        }
        (action_id, result)
    };
    println!("[server] {tag} -> {action_type} as {action_id}: {result:?}");

    let encoded = serde_json::to_value(&result).expect("a result serializes");
    // `ActionResult` deliberately carries no `ActionId`, so a protocol that wants its client to be
    // able to recognize the later facts its own request caused puts the allocated id in its own
    // frame. This is that frame doing it.
    json!({
        "t": "result",
        "action_id": action_id.to_string(),
        "result": encoded,
        "canonical_request": canonical,
    })
    .to_string()
}

/// Appends one accepted-or-refused `talk` request to the evidence log.
fn record_talk(tag: &str, canonical: &Value) {
    use std::io::Write as _;
    let directory = evidence_dir();
    let _ = std::fs::create_dir_all(&directory);
    let line = json!({"client": tag, "canonical_intent": canonical}).to_string();
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(directory.join("intents.jsonl"))
    {
        let _ = writeln!(file, "{line}");
    }
}

/// The `AC-13` comparison, written the moment both clients have spoken.
///
/// Two verdicts, because they may differ and the difference is the point. `identical_whole` compares
/// the submitted requests field for field. `identical_semantic_core` compares only what the world is
/// being asked for — who, what, of whom, with what payload.
///
/// Since the clients submit `ActionRequest`s, two of the three fields that used to make whole
/// equality impossible are simply gone: no client names an `action_id` or an `issued_at` any more.
/// What remains is `actor_location`, which the contract *intends* to differ — a 3D client reports the
/// position it walked to, a 2D client that models no position reports none.
fn compare(talks: &std::collections::BTreeMap<String, Value>) {
    let (Some(two), Some(three)) = (talks.get("2d"), talks.get("3d")) else {
        return;
    };
    let core = |value: &Value| {
        json!({
            "actor": value.get("actor"),
            "action_type": value.get("action_type"),
            "target": value.get("target"),
            "payload": value.get("payload"),
        })
    };
    let whole = two == three;
    let semantic = core(two) == core(three);
    let differing: Vec<&str> = [
        "actor",
        "action_type",
        "target",
        "payload",
        "actor_location",
    ]
    .into_iter()
    .filter(|key| two.get(*key) != three.get(*key))
    .collect();

    let verdict = json!({
        "identical_whole_intent": whole,
        "identical_semantic_core": semantic,
        "fields_that_differ": differing,
        "semantic_core_2d": core(two),
        "semantic_core_3d": core(three),
        "whole_request_2d": two,
        "whole_request_3d": three,
    });
    let directory = evidence_dir();
    let _ = std::fs::create_dir_all(&directory);
    let _ = std::fs::write(
        directory.join("parity.json"),
        serde_json::to_string_pretty(&verdict).expect("the verdict serializes"),
    );
    println!(
        "[server] AC-13 parity: whole={whole} semantic_core={semantic} differing={differing:?}"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use mineworld_contracts::{ActionId, EntityId, WorldTime};

    /// The exact frame a Godot client now builds, byte for byte, taken from `client-2d/main.gd`'s
    /// `submit`. Written by hand here rather than produced by the contract, so that a change in the
    /// contract's shape shows up as a failure rather than as agreement with itself.
    const FROM_THE_2D_CLIENT: &str = r#"{
        "actor": "101",
        "action_type": "talk",
        "target": "9007199254740995",
        "payload": {"action_type": "talk", "payload": {"topic": "greeting"}},
        "actor_location": null
    }"#;

    /// The half of `FINDINGS.md` F4 that can be checked without Godot: the frame a client sends
    /// carries no identity, the contract accepts it, and the *world* supplies the `ActionId` and the
    /// instant.
    #[test]
    fn a_client_frame_with_no_identity_becomes_an_intent_the_world_identified() {
        let request: ActionRequest<Value> =
            serde_json::from_str(FROM_THE_2D_CLIENT).expect("the client's frame is well formed");
        assert_eq!(request.actor(), EntityId::from_raw(101));
        assert_eq!(
            request.target(),
            Some(EntityId::from_raw(9_007_199_254_740_995)),
            "the 64-bit target survived the client's decimal string"
        );

        let mut world = world::SpikeWorld::new();
        let intent = world.allocate(request);

        // The world's first allocation, and the world's clock — neither of which appeared in the
        // frame above.
        assert_eq!(
            intent.action_id(),
            ActionId::from_raw(9_007_199_254_741_001)
        );
        assert_eq!(intent.issued_at(), WorldTime::from_seconds(32_400));
        assert_eq!(intent.action_type().as_str(), "talk");
    }

    /// Two clients submitting the identical frame cannot collide, which is the defect F4 measured:
    /// the spike's clients used local counters, so their ids were both invented and different, and
    /// two clients that happened to pick the same number would have been indistinguishable.
    #[test]
    fn two_clients_sending_the_same_frame_receive_two_identities() {
        let mut world = world::SpikeWorld::new();
        let first = world.allocate(
            serde_json::from_str(FROM_THE_2D_CLIENT).expect("the client's frame is well formed"),
        );
        let second = world.allocate(
            serde_json::from_str(FROM_THE_2D_CLIENT).expect("the client's frame is well formed"),
        );

        assert_ne!(first.action_id(), second.action_id());
        assert_eq!(
            second.action_id(),
            ActionId::from_raw(9_007_199_254_741_002)
        );
        // The same request of the world, twice. Only the identity differs, and the world chose both.
        assert_eq!(first.actor(), second.actor());
        assert_eq!(first.target(), second.target());
        assert_eq!(first.payload(), second.payload());
    }
}
