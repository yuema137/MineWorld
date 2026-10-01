//! PR 05b's acceptance, over real sockets: two WebSocket clients against one running world.
//!
//! Nothing here is mocked. The server is started on an ephemeral port, the clients are
//! `tokio-tungstenite` clients speaking the real protocol over TCP, `GET /status` is a hand-written
//! HTTP request on another socket, and the world is a real `World` with two real systems in it.
//!
//! The four claims, in the order the step document states them:
//!
//! ```text
//! A1  two clients connect at once, and each receives observations scoped to its own observer
//! A2  an intent from one produces an event that both are entitled to see
//! A3  killing one client leaves the world running and the other client unaffected
//! A4  a message that asserts state rather than requesting an action is rejected
//! ```
//!
//! plus the two that follow from the authority model: a client cannot act as somebody else, and
//! nothing streams to a connection that has not been given a seat.

mod support;

use std::net::SocketAddr;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use mineworld_contracts::{ActionResult, EntityId, EventId};
use mineworld_server::{
    PROTOCOL_VERSION, RefusalCode, ServerFrame, WireObservation, WorldHost, WorldSummary, app,
};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use support::{ALICE, BOB, CAROL, brisk, build};

/// How long a test waits for something that should take milliseconds.
const PATIENCE: Duration = Duration::from_secs(5);

/// How many observations a test reads before concluding that a fact is not going to arrive.
///
/// At the test cadence of 20 ms this is about a fifth of a second of stream, against a recent-event
/// window that holds 64 facts: an event the observer were entitled to would have appeared many times
/// over.
const ENOUGH_FRAMES: usize = 10;

/// Starts a server on an ephemeral port and returns the address it actually bound.
async fn start() -> SocketAddr {
    let host = WorldHost::spawn(brisk(), build)
        .await
        .expect("the world is assembled and its thread starts");
    let (listener, address) = app::bind("127.0.0.1:0".parse().expect("a literal address"))
        .await
        .expect("an ephemeral port");
    tokio::spawn(async move {
        let _ = app::serve(listener, host).await;
    });
    address
}

/// One WebSocket client, as a Godot client will be: it sends JSON text and reads JSON text.
struct Client {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl Client {
    async fn connect(address: SocketAddr) -> Self {
        let (socket, _) = tokio_tungstenite::connect_async(format!("ws://{address}/ws"))
            .await
            .expect("the server upgrades the connection");
        Self { socket }
    }

    async fn send(&mut self, text: String) {
        self.socket
            .send(Message::Text(text.into()))
            .await
            .expect("the frame is sent");
    }

    /// The next frame of any kind.
    async fn frame(&mut self) -> ServerFrame {
        let message = tokio::time::timeout(PATIENCE, self.socket.next())
            .await
            .expect("a frame arrives within the patience of this test")
            .expect("the connection is open")
            .expect("the frame is well formed");
        let Message::Text(text) = message else {
            panic!("this protocol carries text frames, and the server sent {message:?}");
        };
        serde_json::from_str(&text).expect("the server's frame decodes as a server frame")
    }

    /// Joins a seat and returns the observer the *server* chose for it.
    async fn join(&mut self, seat: &str) -> (EntityId, WorldSummary) {
        self.send(json!({ "t": "join", "seat": seat }).to_string())
            .await;
        match self.frame().await {
            ServerFrame::Welcome {
                protocol,
                observer,
                world,
                ..
            } => {
                assert_eq!(protocol, PROTOCOL_VERSION);
                (observer, world)
            }
            other => panic!("a join is answered with a welcome, and this was {other:?}"),
        }
    }

    /// The next observation, skipping anything else.
    async fn observation(&mut self) -> (u64, WireObservation) {
        loop {
            if let ServerFrame::Observation { seq, observation } = self.frame().await {
                return (seq, observation);
            }
        }
    }

    /// The next frame that is not an observation: an answer or a refusal.
    async fn answer(&mut self) -> ServerFrame {
        loop {
            let frame = self.frame().await;
            if !matches!(frame, ServerFrame::Observation { .. }) {
                return frame;
            }
        }
    }

    /// Submits a request and returns the events it caused, insisting that it was accepted.
    async fn submit_accepted(&mut self, token: &str, request: Value) -> Vec<EventId> {
        self.send(json!({ "t": "submit", "token": token, "request": request }).to_string())
            .await;
        match self.answer().await {
            ServerFrame::Result {
                token: echoed,
                action_id,
                result: ActionResult::Accepted { events },
            } => {
                assert_eq!(
                    echoed.as_str(),
                    token,
                    "the client's own token comes back on the answer"
                );
                assert!(
                    action_id.raw() >= 1,
                    "the server allocated the identity of this request"
                );
                events
            }
            other => panic!("the request should have been accepted, and the server said {other:?}"),
        }
    }

    /// Submits something the server should refuse, and returns why.
    async fn submit_refused(&mut self, text: String) -> RefusalCode {
        self.send(text).await;
        match self.answer().await {
            ServerFrame::Refused { code, .. } => code,
            other => panic!("this frame should have been refused, and the server said {other:?}"),
        }
    }

    /// Reads several observations, for a claim about what does *not* arrive.
    async fn observations(&mut self, count: usize) -> Vec<WireObservation> {
        let mut seen = Vec::with_capacity(count);
        for _ in 0..count {
            seen.push(self.observation().await.1);
        }
        seen
    }
}

/// `GET` one path, with a hand-written request on its own socket.
///
/// Deliberately not an HTTP client library: the claim is that the control plane answers a real HTTP
/// request, and thirty lines of socket is a smaller thing to trust than another dependency.
async fn get(address: SocketAddr, path: &str) -> Value {
    let mut stream = TcpStream::connect(address)
        .await
        .expect("the control plane accepts a connection");
    let request = format!("GET {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .await
        .expect("the request is written");
    let mut response = String::new();
    tokio::time::timeout(PATIENCE, stream.read_to_string(&mut response))
        .await
        .expect("the control plane answers")
        .expect("the response is text");
    let body = response
        .split_once("\r\n\r\n")
        .expect("an HTTP response has a body")
        .1;
    serde_json::from_str(body).expect("the body is JSON")
}

// ---------------------------------------------------------------------------------------------
// A1 — two clients at once, each scoped to its own observer.
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn two_clients_connect_at_once_and_each_receives_its_own_observers_observation() {
    let address = start().await;

    let mut two_d = Client::connect(address).await;
    let mut three_d = Client::connect(address).await;
    let (alice, world) = two_d.join(ALICE).await;
    let (bob, other_world) = three_d.join(BOB).await;

    assert_ne!(alice, bob, "two seats are two observers");
    assert_eq!(
        world.seats.len(),
        2,
        "the world states which seats it offers, and a client chooses among those"
    );
    // Two observers, one world, and the world says so by identity rather than by resemblance:
    // `MVP.md` §9.1's first evidence line, without which "the same world" is only an assumption
    // about what somebody typed into two address bars.
    assert_eq!(
        world.instance, other_world.instance,
        "both clients were told the identity of the same running world"
    );

    let (first_seq, seen_by_alice) = two_d.observation().await;
    let (_, seen_by_bob) = three_d.observation().await;

    assert_eq!(first_seq, 1, "a connection's frames are numbered from one");
    assert_eq!(seen_by_alice.observer(), alice);
    assert_eq!(seen_by_bob.observer(), bob);

    let alice_sees = support::perceived_ids(&seen_by_alice);
    let bob_sees = support::perceived_ids(&seen_by_bob);
    assert!(
        alice_sees.contains(&alice) && !alice_sees.contains(&bob),
        "the 2D client is shown alice's room and not bob's: {alice_sees:?}"
    );
    assert!(
        bob_sees.contains(&bob) && !bob_sees.contains(&alice),
        "the 3D client is shown bob's room and not alice's: {bob_sees:?}"
    );
    assert_ne!(
        alice_sees, bob_sees,
        "an observation is computed per observer, not filtered from one world frame"
    );

    // What the server said each client may attempt, which neither client computed.
    assert_eq!(seen_by_alice.affordances().len(), 1);
    assert_eq!(
        seen_by_alice.affordances()[0].target(),
        alice_sees.iter().copied().find(|id| *id != alice),
        "the affordance names the other person in alice's room"
    );

    let status = get(address, "/status").await;
    assert_eq!(status["clients"], json!(2), "both clients are connected");
    assert_eq!(status["entities"], json!(4));
    assert_eq!(
        get(address, "/health").await,
        json!({ "status": "ok", "protocol": PROTOCOL_VERSION })
    );
}

// ---------------------------------------------------------------------------------------------
// A2 — one client's intent, a fact both are entitled to.
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn an_intent_from_one_client_produces_an_event_both_clients_are_entitled_to_see() {
    let address = start().await;
    let mut two_d = Client::connect(address).await;
    let mut three_d = Client::connect(address).await;
    let (alice, _) = two_d.join(ALICE).await;
    let (bob, _) = three_d.join(BOB).await;

    let events = two_d
        .submit_accepted(
            "2d-1",
            support::speak_request(alice, bob, "hello from the cafe"),
        )
        .await;
    assert_eq!(events.len(), 1, "one request, one recorded fact");
    let spoken = events[0];

    let by_alice = two_d
        .observations(ENOUGH_FRAMES)
        .await
        .into_iter()
        .any(|observation| support::carries_event(&observation, spoken));
    let by_bob = three_d
        .observations(ENOUGH_FRAMES)
        .await
        .into_iter()
        .any(|observation| support::carries_event(&observation, spoken));

    assert!(
        by_alice,
        "the client that acted learns of the fact it caused"
    );
    assert!(
        by_bob,
        "and so does the other client, because the emitting system declared the fact public"
    );
}

/// The other half of the same claim, and the one that makes it mean something: a fact only one
/// observer is entitled to reaches only that client.
#[tokio::test]
async fn a_fact_only_one_observer_is_entitled_to_reaches_only_that_client() {
    let address = start().await;
    let mut two_d = Client::connect(address).await;
    let mut three_d = Client::connect(address).await;
    let (alice, _) = two_d.join(ALICE).await;
    let (_, _) = three_d.join(BOB).await;

    let events = two_d
        .submit_accepted("2d-1", support::whisper_request(alice, "to myself"))
        .await;
    let whispered = events[0];

    let by_alice = two_d
        .observations(ENOUGH_FRAMES)
        .await
        .into_iter()
        .any(|observation| support::carries_event(&observation, whispered));
    let by_bob = three_d
        .observations(ENOUGH_FRAMES)
        .await
        .into_iter()
        .any(|observation| support::carries_event(&observation, whispered));

    assert!(by_alice, "the participant learns of it");
    assert!(
        !by_bob,
        "and the other client never does: entitlement is decided per observer, not broadcast"
    );
}

/// An `EntityId` inside an *event payload* reaches a client as a decimal string, which is the
/// position `spike/FINDINGS.md` F2 measured as unreachable by a protocol-level encoder. This server
/// has no encoder: the contract does it.
#[tokio::test]
async fn an_identity_inside_an_event_payload_reaches_a_client_as_a_decimal_string() {
    let address = start().await;
    let mut two_d = Client::connect(address).await;
    let mut three_d = Client::connect(address).await;
    let (alice, _) = two_d.join(ALICE).await;
    let (bob, _) = three_d.join(BOB).await;

    two_d
        .submit_accepted("2d-1", support::speak_request(alice, bob, "hello"))
        .await;

    let payload = two_d
        .observations(ENOUGH_FRAMES)
        .await
        .into_iter()
        .find_map(|observation| {
            observation
                .events()
                .iter()
                .find(|event| event.envelope().event_type().as_str() == "spoke")
                .map(|event| event.envelope().payload().payload().clone())
        })
        .expect("the fact reaches the client that caused it");

    assert_eq!(
        payload["to"],
        json!(bob.raw().to_string()),
        "an id inside a payload is a string, so a client with only doubles cannot corrupt it"
    );
}

// ---------------------------------------------------------------------------------------------
// A3 — killing one client.
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn killing_one_client_leaves_the_world_running_and_the_other_client_unaffected() {
    let address = start().await;
    let mut two_d = Client::connect(address).await;
    let mut three_d = Client::connect(address).await;
    let (_, _) = two_d.join(ALICE).await;
    let (bob, _) = three_d.join(BOB).await;

    three_d.observation().await;
    // Killed rather than closed: the socket goes away with no close frame, which is what a crashed
    // or force-quit client does.
    drop(two_d);

    let (before, _) = three_d.observation().await;
    let (after, _) = three_d.observation().await;
    assert!(
        after > before,
        "the surviving client's stream keeps advancing: {before} then {after}"
    );

    let dave = support::perceived_ids(&three_d.observation().await.1)
        .into_iter()
        .find(|entity| *entity != bob)
        .expect("dave is in the street with bob");
    let events = three_d
        .submit_accepted(
            "3d-1",
            support::speak_request(bob, dave, "did you see that"),
        )
        .await;
    assert_eq!(events.len(), 1, "the world still dispatches");

    let status = get(address, "/status").await;
    assert_eq!(
        status["clients"],
        json!(1),
        "the dead connection stopped counting, and the live one did not"
    );
    assert_eq!(
        status["entities"],
        json!(4),
        "the world is the same world it was"
    );
}

// ---------------------------------------------------------------------------------------------
// A4 — a message that asserts state.
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn a_message_that_asserts_state_is_refused_and_changes_nothing() {
    let address = start().await;
    let mut two_d = Client::connect(address).await;
    let (alice, _) = two_d.join(ALICE).await;

    let before = room_of(&two_d.observation().await.1, alice);

    // `NETWORKING.md` §2, verbatim in spirit: "my money is now 5000", addressed to a world that has
    // no frame for being told anything.
    let code = two_d
        .submit_refused(
            json!({ "t": "set_state", "entity": alice, "room": "palace", "money": 5000 })
                .to_string(),
        )
        .await;
    assert_eq!(code, RefusalCode::UnknownFrame);

    assert_eq!(
        two_d
            .submit_refused("my money is now 5000".to_owned())
            .await,
        RefusalCode::MalformedFrame,
        "and the same assertion in prose is not a frame at all"
    );

    let after = room_of(&two_d.observation().await.1, alice);
    assert_eq!(
        before, after,
        "the assertion changed nothing: alice is where the world put her"
    );
    assert_eq!(before, json!("cafe"));
}

/// Where an observation says this entity is, read out of the component the world exposed.
fn room_of(observation: &WireObservation, entity: EntityId) -> Value {
    observation
        .entity(entity)
        .expect("the observer perceives itself")
        .components()
        .iter()
        .find(|record| record.component_type().as_str() == "room")
        .expect("the room component is exposed")
        .payload()["name"]
        .clone()
}

// ---------------------------------------------------------------------------------------------
// The authority model's other two edges.
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn a_client_cannot_ask_the_world_to_act_as_somebody_else() {
    let address = start().await;
    let mut two_d = Client::connect(address).await;
    let mut three_d = Client::connect(address).await;
    let (alice, _) = two_d.join(ALICE).await;
    let (bob, _) = three_d.join(BOB).await;

    let code = three_d
        .submit_refused(
            json!({
                "t": "submit",
                "token": "3d-1",
                "request": support::speak_request(alice, bob, "I am Alice"),
            })
            .to_string(),
        )
        .await;

    assert_eq!(code, RefusalCode::ActorNotObserver);
    let nothing_happened = two_d
        .observations(ENOUGH_FRAMES)
        .await
        .into_iter()
        .all(|observation| observation.events().is_empty());
    assert!(
        nothing_happened,
        "a refused frame produced no fact in anybody's world"
    );
}

#[tokio::test]
async fn nothing_streams_until_a_seat_is_granted_and_a_seat_is_held_for_the_connection() {
    let address = start().await;
    let mut client = Client::connect(address).await;

    assert_eq!(
        client
            .submit_refused(
                json!({ "t": "submit", "token": "c1",
                        "request": support::whisper_request(EntityId::from_raw(1), "hello") })
                .to_string()
            )
            .await,
        RefusalCode::NotJoined,
        "a request before a seat has no observer to act as"
    );

    client
        .send(json!({ "t": "join", "seat": CAROL }).to_string())
        .await;
    match client.answer().await {
        ServerFrame::Refused { code, .. } => assert_eq!(
            code,
            RefusalCode::UnknownSeat,
            "carol is a person in this world, and not a seat in it"
        ),
        other => panic!("an unknown seat is refused, and the server said {other:?}"),
    }

    let (alice, _) = client.join(ALICE).await;
    client
        .send(json!({ "t": "join", "seat": BOB }).to_string())
        .await;
    match client.answer().await {
        ServerFrame::Refused { code, .. } => assert_eq!(
            code,
            RefusalCode::AlreadyJoined,
            "a connection cannot change which observer it is"
        ),
        other => panic!("a second join is refused, and the server said {other:?}"),
    }
    assert_eq!(
        client.observation().await.1.observer(),
        alice,
        "and it is still the observer it was granted"
    );
}

// ---------------------------------------------------------------------------------------------
// S4 — work a system defers reaches clients at its instant, through the real host loop.
// ---------------------------------------------------------------------------------------------

/// A request whose system defers a fact is answered with nothing *now*; the fact is held in the
/// world's own schedule, fired when the host's clock reaches its instant, and reaches every client
/// entitled to it — carrying the request as its cause. Until S4 this server could only count such a
/// deferral as `deferrals_unscheduled`; the counter now stays at zero because nothing is dropped.
#[tokio::test]
async fn a_fact_a_system_defers_reaches_the_clients_at_its_instant() {
    let address = start().await;
    let mut two_d = Client::connect(address).await;
    let mut three_d = Client::connect(address).await;
    let (alice, _) = two_d.join(ALICE).await;
    let _ = three_d.join(BOB).await;

    let before: WorldSummary =
        serde_json::from_value(get(address, "/status").await).expect("a status answer");

    two_d
        .send(
            json!({ "t": "submit", "token": "remind-1",
                    "request": support::remind_request(alice, "the kettle", 1) })
            .to_string(),
        )
        .await;
    let request = match two_d.answer().await {
        ServerFrame::Result {
            action_id,
            result: ActionResult::Accepted { events },
            ..
        } => {
            assert!(
                events.is_empty(),
                "nothing happens at the request's own instant"
            );
            action_id
        }
        other => panic!("the reminder should have been accepted, and the server said {other:?}"),
    };

    // The host's clock counts whole seconds, so the reminder is due within about two seconds of
    // wall time. Read Bob's stream until it arrives, within a bound that is generous against that.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(4);
    let reminder = loop {
        assert!(
            tokio::time::Instant::now() < deadline,
            "the deferred fact never reached the other client"
        );
        let (_, observation) = three_d.observation().await;
        if let Some(perceived) = observation.events().iter().find(|perceived| {
            let envelope = perceived.envelope();
            envelope.event_type().as_str() == "spoke"
                && envelope.payload().payload()["words"] == json!("the kettle")
        }) {
            break perceived.envelope().clone();
        }
    };

    assert_eq!(
        *reminder.caused_by(),
        mineworld_contracts::Causation::Action(request),
        "the fact names the request that deferred it"
    );
    assert!(
        reminder.at().seconds() > before.at.seconds(),
        "it happened at a later instant than the world was at before the request: {} vs {}",
        reminder.at(),
        before.at
    );

    let after: WorldSummary =
        serde_json::from_value(get(address, "/status").await).expect("a status answer");
    assert_eq!(
        after.deferrals_unscheduled, 0,
        "nothing was left unscheduled"
    );
    assert_eq!(after.faults, 0);
}
