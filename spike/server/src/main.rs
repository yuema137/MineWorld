//! The renderer-integration spike's demo server.
//!
//! One authoritative world (`world.rs`), one WebSocket endpoint, and the `DD-15` wire encoding
//! (`wire.rs`). It does not use `mineworld-kernel`: the kernel is being written elsewhere and
//! this spike exists to test the *contracts*, which are merged and stable.
//!
//! ```text
//! server  ── Observation ──►  client        every 100 ms, in two encodings
//! server  ◄── ActionIntent ──  client        whatever the player did
//! server  ── ActionResult ──►  client        the server's answer, and only the server's
//! ```
//!
//! Every intent the server accepts is written to `spike/evidence/intents.jsonl` in the
//! contract's own canonical JSON, tagged by which client sent it. That file is the `AC-13`
//! parity evidence: the two `talk` intents are compared field by field in `parity.json`.

mod vocabulary;
mod wire;
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
use mineworld_contracts::ActionIntent;
use serde_json::{Value, json};
use tokio::sync::Mutex;

use wire::Direction;
use world::SpikeWorld;

/// Shared state: the world, and the intents each client has submitted.
struct Shared {
    world: SpikeWorld,
    /// The last `talk` intent each client tag submitted, in canonical contract JSON.
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
        let naive = serde_json::to_value(&observation).expect("an observation serializes");
        let mut encoded = naive.clone();
        wire::observation(Direction::Encode, &mut encoded);
        println!("{}", serde_json::to_string_pretty(&json!({"naive": naive, "wire": encoded})).expect("pretty"));
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

/// Serializes the current observation in both encodings.
///
/// `naive` is what `serde_json` produces from the contract type directly: 64-bit ids as JSON
/// numbers. `wire` is the same value after `DD-15`'s encoding. Sending both in one frame is what
/// lets a client compare them and report, from inside Godot, whether the decision was needed.
fn observation_frame(world: &SpikeWorld) -> String {
    let observation = world.observation();
    let naive = serde_json::to_value(&observation).expect("an observation serializes");
    let mut encoded = naive.clone();
    wire::observation(Direction::Encode, &mut encoded);
    json!({ "t": "observation", "naive": naive, "wire": encoded }).to_string()
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
        Some("intent") => Some(handle_intent(state, &frame).await),
        _ => Some(json!({"t": "error", "detail": "unknown frame type"}).to_string()),
    }
}

async fn handle_intent(state: &App, frame: &Value) -> String {
    let tag = frame
        .get("client")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_owned();
    let Some(sent) = frame.get("intent") else {
        return json!({"t": "error", "detail": "intent frame carries no intent"}).to_string();
    };

    // Undo the wire encoding, then let the *contract* decide whether the frame is well formed.
    let mut decoded = sent.clone();
    wire::action_intent(Direction::Decode, &mut decoded);
    let intent = match serde_json::from_value::<ActionIntent<Value>>(decoded.clone()) {
        Ok(intent) => intent,
        Err(error) => {
            println!("[server] rejected a malformed intent from {tag}: {error}");
            return json!({"t": "error", "detail": error.to_string()}).to_string();
        }
    };

    // Canonical form: what the contract itself says this intent is. This — not the wire form —
    // is what the AC-13 parity comparison is made of.
    let canonical = serde_json::to_value(&intent).expect("an intent serializes");
    let action_type = intent.action_type().to_string();

    let result = {
        let mut shared = state.lock().await;
        let result = shared.world.resolve(&intent);
        if action_type == "talk" {
            shared.talks.insert(tag.clone(), canonical.clone());
            let talks = shared.talks.clone();
            record_talk(&tag, &canonical);
            compare(&talks);
        }
        result
    };
    println!("[server] {tag} -> {action_type}: {result:?}");

    let mut encoded = serde_json::to_value(&result).expect("a result serializes");
    wire::action_result(Direction::Encode, &mut encoded);
    json!({"t": "result", "result": encoded, "canonical_intent": canonical}).to_string()
}

/// Appends one accepted-or-refused `talk` intent to the evidence log.
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
/// Two verdicts, because they differ and the difference is the point. `identical_whole` compares
/// the intents field for field. `identical_semantic_core` compares only what the world is being
/// asked for — who, what, of whom, with what payload — and leaves out the three fields that
/// cannot be equal: the `action_id` each client had to invent, the `issued_at` each read off a
/// different observation, and the `actor_location` a 2D client has no business reporting.
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
    let differing: Vec<&str> = ["action_id", "actor", "action_type", "target", "payload", "issued_at", "actor_location"]
        .into_iter()
        .filter(|key| two.get(*key) != three.get(*key))
        .collect();

    let verdict = json!({
        "identical_whole_intent": whole,
        "identical_semantic_core": semantic,
        "fields_that_differ": differing,
        "semantic_core_2d": core(two),
        "semantic_core_3d": core(three),
        "whole_intent_2d": two,
        "whole_intent_3d": three,
    });
    let directory = evidence_dir();
    let _ = std::fs::create_dir_all(&directory);
    let _ = std::fs::write(
        directory.join("parity.json"),
        serde_json::to_string_pretty(&verdict).expect("the verdict serializes"),
    );
    println!("[server] AC-13 parity: whole={whole} semantic_core={semantic} differing={differing:?}");
}
