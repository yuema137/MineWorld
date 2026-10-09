//! One controller per seat, over a real socket (step-12 §16.4 SB-2, SB-7).
//!
//! `PROTOCOL.md` §4.2's join rules decide who drives a seat: a free seat goes to the first join, a
//! seat another connection holds is refused unless the join takes it over, a resume must match, and
//! an in-server controller yields to a person without a flag. The table that applies them is on the
//! world thread, so races between connections are decided there and never in a session.
//!
//! The in-server controller tested here is written in this file against the public seam
//! (`HostedController`): the server names no controller crate, so a test controller is exactly as
//! privileged as a real one — which is to say not at all (`ARC-42`).

mod support;

use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::sync::mpsc as std_mpsc;
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use mineworld_contracts::{ActionRequest, EntityId, WorldTime};
use mineworld_server::{
    ClosingReason, HostConfig, HostError, HostedAnswer, HostedController, HostedWorld, RefusalCode,
    ServerFrame, TookOver, WireObservation, WorldHost, app,
};
use serde_json::{Value, json};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use support::{ALICE, BOB, brisk, build, join_frame, key};

/// How long a test waits for something that should take milliseconds.
const PATIENCE: Duration = Duration::from_secs(5);

async fn start_with(
    config: HostConfig,
    assemble: impl FnOnce() -> Result<HostedWorld, HostError> + Send + 'static,
) -> SocketAddr {
    let host = WorldHost::spawn(config, assemble)
        .await
        .expect("the world is assembled and its thread starts");
    let (listener, address) = app::bind("127.0.0.1:0".parse().expect("a literal address"))
        .await
        .expect("an ephemeral port");
    tokio::spawn(async move {
        let _ = app::serve(listener, host, support::admission()).await;
    });
    address
}

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

    async fn send(&mut self, frame: &Value) {
        self.socket
            .send(Message::Text(frame.to_string().into()))
            .await
            .expect("the frame is sent");
    }

    /// The next frame, or `None` once the server has closed the connection.
    async fn next(&mut self) -> Option<ServerFrame> {
        loop {
            let next = tokio::time::timeout(PATIENCE, self.socket.next())
                .await
                .expect("something arrives within the patience of this test");
            match next {
                None | Some(Err(_) | Ok(Message::Close(_))) => return None,
                Some(Ok(Message::Text(text))) => {
                    return Some(serde_json::from_str(&text).expect("a server frame"));
                }
                Some(Ok(_)) => {}
            }
        }
    }

    /// The next frame that is not an observation.
    async fn answer(&mut self) -> ServerFrame {
        loop {
            let frame = self.next().await.expect("the connection is open");
            if !matches!(frame, ServerFrame::Observation { .. }) {
                return frame;
            }
        }
    }

    /// Reads a `closing` with this reason, then insists that the socket is closed.
    async fn closed_because(&mut self, expected: ClosingReason) {
        match self.answer().await {
            ServerFrame::Closing { reason, .. } => assert_eq!(reason, expected),
            other => panic!("a closing was expected, and the server said {other:?}"),
        }
        while let Some(frame) = self.next().await {
            assert!(
                matches!(frame, ServerFrame::Observation { .. }),
                "nothing but a closed socket follows a closing: {frame:?}"
            );
        }
    }
}

/// What a join came to: the welcome's control fields, or the refusal's code.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Joined {
    Welcomed {
        observer: EntityId,
        took_over: TookOver,
    },
    Refused(RefusalCode),
}

async fn joined(client: &mut Client) -> Joined {
    match client.answer().await {
        ServerFrame::Welcome {
            observer,
            took_over,
            ..
        } => Joined::Welcomed {
            observer,
            took_over,
        },
        ServerFrame::Refused { code, .. } => Joined::Refused(code),
        other => panic!("a welcome or a refusal was expected, and the server said {other:?}"),
    }
}

// ---------------------------------------------------------------------------------------------
// SB-2 — one controller per seat.
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn three_connections_racing_for_one_free_seat_seat_exactly_one() {
    let address = start_with(brisk(), build).await;
    let mut racers = Vec::new();
    for _ in 0..3 {
        racers.push(Client::connect(address).await);
    }
    let forged = {
        let mut join = join_frame(ALICE, "forger");
        join["resume"] = json!("00112233445566778899aabbccddeeff");
        join
    };
    let joins = [join_frame(ALICE, "one"), join_frame(ALICE, "two"), forged];
    // Every join is on the wire before any answer is read: the world thread decides the race.
    for (client, join) in racers.iter_mut().zip(&joins) {
        client.send(join).await;
    }
    let mut answers = Vec::new();
    for client in &mut racers {
        answers.push(joined(client).await);
    }

    let welcomed: Vec<&Joined> = answers
        .iter()
        .filter(|answer| matches!(answer, Joined::Welcomed { .. }))
        .collect();
    assert_eq!(welcomed.len(), 1, "exactly one welcome: {answers:?}");
    assert_eq!(
        answers[2],
        Joined::Refused(RefusalCode::InvalidResume),
        "a forged resume matches nothing, whoever won"
    );
    assert!(
        answers[..2].contains(&Joined::Refused(RefusalCode::SeatOccupied)),
        "the plain join that lost is told the seat is occupied: {answers:?}"
    );
}

#[tokio::test]
async fn a_connected_seat_is_refused_without_take_over_and_taken_with_it() {
    let address = start_with(brisk(), build).await;
    let mut holder = Client::connect(address).await;
    holder.send(&join_frame(ALICE, "holder")).await;
    let Joined::Welcomed {
        observer,
        took_over,
    } = joined(&mut holder).await
    else {
        panic!("a free seat is granted");
    };
    assert_eq!(took_over, TookOver::None);

    let mut fourth = Client::connect(address).await;
    fourth.send(&join_frame(ALICE, "fourth")).await;
    assert_eq!(
        joined(&mut fourth).await,
        Joined::Refused(RefusalCode::SeatOccupied)
    );

    // Still in the handshake after a refusal: the same connection asks again, taking the seat.
    let mut take = join_frame(ALICE, "fourth");
    take["take_over"] = json!(true);
    fourth.send(&take).await;
    assert_eq!(
        joined(&mut fourth).await,
        Joined::Welcomed {
            observer,
            took_over: TookOver::Connection,
        },
        "the same Person, taken from another connection"
    );
    holder.closed_because(ClosingReason::TakenOver).await;
}

#[tokio::test]
async fn a_resume_presented_while_the_old_socket_still_lives_supersedes_it() {
    let address = start_with(brisk(), build).await;
    let mut old = Client::connect(address).await;
    old.send(&join_frame(BOB, "old")).await;
    let ServerFrame::Welcome {
        resume, observer, ..
    } = old.answer().await
    else {
        panic!("a free seat is granted");
    };
    let resume = resume.expect("a resume secret");

    let mut new = Client::connect(address).await;
    let mut join = join_frame(BOB, "new");
    join["resume"] = json!(resume.reveal());
    new.send(&join).await;
    assert_eq!(
        joined(&mut new).await,
        Joined::Welcomed {
            observer,
            took_over: TookOver::Held,
        }
    );
    old.closed_because(ClosingReason::Superseded).await;
}

// ---------------------------------------------------------------------------------------------
// SB-7 — a hosted controller has no privilege.
// ---------------------------------------------------------------------------------------------

/// What the test controller reports back to the test: each request's answer, in order.
type Answers = std_mpsc::Sender<HostedAnswer>;

/// Drives alice: first it asks the world to act as somebody else, then as herself, then never again.
struct Impostor {
    asked: u8,
    answers: Answers,
}

impl HostedController for Impostor {
    fn next_consult(&self, after: WorldTime) -> WorldTime {
        WorldTime::from_seconds(after.seconds() + 1)
    }

    fn decide(&mut self, observation: &WireObservation) -> Option<ActionRequest> {
        let me = observation.observer();
        let someone_else = observation
            .entities()
            .iter()
            .map(mineworld_contracts::PerceivedEntity::id)
            .find(|entity| *entity != me)?;
        self.asked += 1;
        let request = match self.asked {
            // Carol speaking, as alice's controller would like her to: refused, never dispatched.
            1 => support::speak_request(someone_else, me, "forged"),
            2 => support::speak_request(me, someone_else, "own"),
            _ => return None,
        };
        // Built the way a client builds one, then decoded as the session decodes it.
        let wire: ActionRequest<mineworld_server::WirePayload> =
            serde_json::from_value(request).expect("a wire request");
        Some(mineworld_server::protocol::into_kernel_request(&wire).expect("a request"))
    }

    fn answered(&mut self, answer: &HostedAnswer) {
        let _ = self.answers.send(answer.clone());
    }
}

#[tokio::test]
async fn a_hosted_request_meets_the_actor_check_and_the_sessions_allocator() {
    let (answers, received) = std_mpsc::channel();
    let config = HostConfig {
        // A second of world time every few milliseconds, so the controller is due on every tick.
        time_scale: std::num::NonZeroU32::new(600).expect("non-zero"),
        ..brisk()
    };
    let address = start_with(config, move || {
        Ok(build()?.hosting(key(ALICE), move |_bound| {
            Box::new(Impostor {
                asked: 0,
                answers: answers.clone(),
            })
        }))
    })
    .await;

    // Bob's player acts throughout, so the session allocator and the hosted one are interleaved.
    let mut bob = Client::connect(address).await;
    bob.send(&join_frame(BOB, "bob")).await;
    let ServerFrame::Welcome {
        observer: bob_id, ..
    } = bob.answer().await
    else {
        panic!("bob's seat is free");
    };

    let mut session_ids = BTreeSet::new();
    let mut heard = Vec::new();
    let mut hosted = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut token = 0;
    while Instant::now() < deadline && hosted.len() < 2 {
        token += 1;
        bob.send(&json!({ "t": "submit", "token": format!("b{token}"),
                          "request": support::whisper_request(bob_id, "bob") }))
            .await;
        loop {
            match bob.next().await.expect("open") {
                ServerFrame::Result { action_id, .. } => {
                    session_ids.insert(action_id);
                    break;
                }
                ServerFrame::Observation { observation, .. } => heard.push(observation),
                other => panic!("unexpected {other:?}"),
            }
        }
        while let Ok(answer) = received.try_recv() {
            hosted.push(answer);
        }
    }
    // A few more observations, so a dispatched forgery would have had time to be perceived.
    for _ in 0..10 {
        if let Some(ServerFrame::Observation { observation, .. }) = bob.next().await {
            heard.push(observation);
        }
    }

    assert_eq!(hosted.len(), 2, "the controller asked twice: {hosted:?}");
    assert_eq!(
        hosted[0],
        HostedAnswer::Refused(RefusalCode::ActorNotObserver),
        "acting as somebody else is refused before the world considers it"
    );
    let HostedAnswer::Answered(own) = &hosted[1] else {
        panic!("a request as its own Person is answered: {:?}", hosted[1]);
    };
    assert!(
        matches!(
            own.result(),
            mineworld_contracts::ActionResult::Accepted { .. }
        ),
        "{own:?}"
    );

    // One allocator: the hosted request's identity sits inside the sessions' sequence, never
    // beside it, and every identity issued is consecutive.
    // The refused request was given no identity at all.
    let mut every: Vec<u64> = session_ids.iter().map(|id| id.raw()).collect();
    every.push(own.action_id().raw());
    every.sort_unstable();
    every.dedup();
    assert_eq!(
        every.len(),
        session_ids.len() + 1,
        "no identity issued twice"
    );
    assert!(
        every.windows(2).all(|pair| pair[1] == pair[0] + 1),
        "identities from one allocator, consecutive: {every:?}"
    );

    // Nothing was dispatched as carol: no observation anybody was shown carries the forgery.
    let shown = serde_json::to_string(&heard).expect("observations encode");
    assert!(!shown.contains("forged"), "a refused request left a fact");
}
