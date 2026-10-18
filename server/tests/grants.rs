//! Seat grants over a real socket (S27-a, `ARC-97`; `PROTOCOL.md` §§4.1, 4.2): CP-a's grant half —
//! AA-1, AA-2, AA-3 and AA-9 of `pr-s27a-seat-grants-budget.md` §8.
//!
//! ```text
//! the invite         any seat, any takeover (as before)
//! a grant            only the seats it names (check 4a, not_granted); a takeover of a connected or
//!                    held seat only if it carries take-over (check 6, not_granted)
//! ```
//!
//! A `not_granted` displaces nobody: both checks return before the seat table binds anything, which
//! is what `a_grant_refused_a_seat_displaces_nobody` pins (mutation M-a-1, the check moved after
//! control, sends the player there `closing taken_over`).

mod support;

use std::net::SocketAddr;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use mineworld_contracts::EntityId;
use mineworld_server::{
    Admission, ClosingReason, RefusalCode, SeatGrant, ServerFrame, TookOver, WorldHost, app,
};
use serde_json::{Value, json};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use support::{ALICE, BOB, INVITE, brisk, build, grant, join_presenting, speak_request};

/// How long a test waits for something that should take milliseconds.
const PATIENCE: Duration = Duration::from_secs(5);
/// How long a connection that must not be displaced is watched for a `closing`.
const WATCH: Duration = Duration::from_millis(400);

/// A server admitting the test invite and these grants.
async fn start(grants: &[SeatGrant]) -> SocketAddr {
    let admission: Admission = support::admission()
        .with_grants(grants.iter().cloned())
        .expect("distinct credentials");
    let host = WorldHost::spawn(brisk(), build)
        .await
        .expect("the world starts");
    let (listener, address) = app::bind("127.0.0.1:0".parse().expect("an address"))
        .await
        .expect("an ephemeral port");
    tokio::spawn(async move {
        let _ = app::serve(listener, host, admission).await;
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

    async fn next_within(&mut self, patience: Duration) -> Option<Option<ServerFrame>> {
        loop {
            let next = tokio::time::timeout(patience, self.socket.next())
                .await
                .ok()?;
            match next {
                None | Some(Err(_) | Ok(Message::Close(_))) => return Some(None),
                Some(Ok(Message::Text(text))) => {
                    return Some(Some(serde_json::from_str(&text).expect("a server frame")));
                }
                Some(Ok(_)) => {}
            }
        }
    }

    /// The next frame that is not part of the stream.
    async fn answer(&mut self) -> ServerFrame {
        loop {
            let frame = self
                .next_within(PATIENCE)
                .await
                .expect("an answer within the patience of this test")
                .expect("the connection is open");
            if !matches!(
                frame,
                ServerFrame::Observation { .. } | ServerFrame::Clock { .. }
            ) {
                return frame;
            }
        }
    }

    /// Asserts that for [`WATCH`] nothing but the stream arrives: no `closing`, no closed socket.
    async fn stays_seated(&mut self, why: &str) {
        let until = tokio::time::Instant::now() + WATCH;
        while let Some(left) = until.checked_duration_since(tokio::time::Instant::now()) {
            match self.next_within(left).await {
                None => return,
                Some(Some(ServerFrame::Observation { .. } | ServerFrame::Clock { .. })) => {}
                Some(other) => panic!("{why}: the seated player was sent {other:?}"),
            }
        }
    }

    async fn closed_because(&mut self, expected: ClosingReason) {
        match self.answer().await {
            ServerFrame::Closing { reason, .. } => assert_eq!(reason, expected),
            other => panic!("a closing was expected, and the server said {other:?}"),
        }
    }
}

/// What a join came to.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Joined {
    Welcomed {
        observer: EntityId,
        took_over: TookOver,
        resume: String,
    },
    Refused(RefusalCode),
}

async fn joined(client: &mut Client) -> Joined {
    match client.answer().await {
        ServerFrame::Welcome {
            observer,
            took_over,
            resume,
            ..
        } => Joined::Welcomed {
            observer,
            took_over,
            resume: resume.expect("a resume").reveal().to_owned(),
        },
        ServerFrame::Refused { code, .. } => Joined::Refused(code),
        other => panic!("a welcome or a refusal was expected, and the server said {other:?}"),
    }
}

fn took_over(joined: &Joined) -> Option<TookOver> {
    match joined {
        Joined::Welcomed { took_over, .. } => Some(*took_over),
        Joined::Refused(_) => None,
    }
}

fn taking(mut join: Value) -> Value {
    join["take_over"] = json!(true);
    join
}

/// AA-1: a grant for alice seats alice; asking it for bob is `not_granted`, and the same connection
/// then joins alice.
#[tokio::test]
async fn a_grant_admits_the_seats_it_names_and_no_other() {
    let alice = grant(&[ALICE], false);
    let address = start(std::slice::from_ref(&alice)).await;
    let token = alice.token().reveal();
    let mut client = Client::connect(address).await;
    client.send(&join_presenting(BOB, "friend", token)).await;
    assert_eq!(
        joined(&mut client).await,
        Joined::Refused(RefusalCode::NotGranted)
    );
    client.send(&join_presenting(ALICE, "friend", token)).await;
    assert_eq!(
        took_over(&joined(&mut client).await),
        Some(TookOver::None),
        "the connection stayed, and its grant's seat is free"
    );
}

/// M-a-1's oracle: a grant for alice asking to take over bob, whom an invite holder drives, is
/// refused before control — so bob's player is not sent `closing taken_over`.
#[tokio::test]
async fn a_grant_refused_a_seat_displaces_nobody() {
    let alice = grant(&[ALICE], true);
    let address = start(std::slice::from_ref(&alice)).await;
    let mut human = Client::connect(address).await;
    human.send(&join_presenting(BOB, "human", INVITE)).await;
    assert_eq!(took_over(&joined(&mut human).await), Some(TookOver::None));

    let mut other = Client::connect(address).await;
    other
        .send(&taking(join_presenting(
            BOB,
            "other",
            alice.token().reveal(),
        )))
        .await;
    assert_eq!(
        joined(&mut other).await,
        Joined::Refused(RefusalCode::NotGranted)
    );
    human
        .stays_seated("a grant refused the seat it asked for")
        .await;
}

/// AA-2 and M-a-2's oracle: without take-over a grant cannot displace a connected player; with it, it
/// can, and the player is sent `closing taken_over`.
#[tokio::test]
async fn a_grant_without_take_over_cannot_displace_a_player() {
    let plain = grant(&[ALICE], false);
    let mighty = grant(&[ALICE], true);
    let address = start(&[plain.clone(), mighty.clone()]).await;
    let mut human = Client::connect(address).await;
    human.send(&join_presenting(ALICE, "human", INVITE)).await;
    let Joined::Welcomed { observer, .. } = joined(&mut human).await else {
        panic!("a free seat");
    };

    let mut agent = Client::connect(address).await;
    agent
        .send(&taking(join_presenting(
            ALICE,
            "agent",
            plain.token().reveal(),
        )))
        .await;
    assert_eq!(
        joined(&mut agent).await,
        Joined::Refused(RefusalCode::NotGranted)
    );
    // Without the flag it is the ordinary answer, whatever the credential.
    agent
        .send(&join_presenting(ALICE, "agent", plain.token().reveal()))
        .await;
    assert_eq!(
        joined(&mut agent).await,
        Joined::Refused(RefusalCode::SeatOccupied)
    );
    human.stays_seated("a grant without take-over").await;

    let mut taker = Client::connect(address).await;
    taker
        .send(&taking(join_presenting(
            ALICE,
            "taker",
            mighty.token().reveal(),
        )))
        .await;
    match joined(&mut taker).await {
        Joined::Welcomed {
            observer: same,
            took_over,
            ..
        } => {
            assert_eq!(same, observer, "the same Person");
            assert_eq!(took_over, TookOver::Connection);
        }
        refused @ Joined::Refused(_) => panic!("a grant with take-over: {refused:?}"),
    }
    human.closed_because(ClosingReason::TakenOver).await;
}

/// AA-3: a dropped player resumes its own hold with its `resume`, whatever credential it joined
/// with; a grant without take-over cannot take a seat another player's drop left held.
#[tokio::test]
async fn a_hold_is_resumed_with_any_credential_and_not_taken_without_the_right() {
    let mine = grant(&[ALICE], false);
    let theirs = grant(&[ALICE], false);
    let address = start(&[mine.clone(), theirs.clone()]).await;

    let mut first = Client::connect(address).await;
    first
        .send(&join_presenting(ALICE, "first", mine.token().reveal()))
        .await;
    let Joined::Welcomed { resume, .. } = joined(&mut first).await else {
        panic!("a free seat");
    };
    drop(first); // no `leave`: the seat is held
    tokio::time::sleep(Duration::from_millis(200)).await;

    let mut intruder = Client::connect(address).await;
    intruder
        .send(&taking(join_presenting(
            ALICE,
            "intruder",
            theirs.token().reveal(),
        )))
        .await;
    assert_eq!(
        joined(&mut intruder).await,
        Joined::Refused(RefusalCode::NotGranted),
        "a held seat is not taken without the right"
    );

    let mut back = Client::connect(address).await;
    let mut rejoin = join_presenting(ALICE, "first", mine.token().reveal());
    rejoin["resume"] = json!(resume);
    back.send(&rejoin).await;
    assert_eq!(
        took_over(&joined(&mut back).await),
        Some(TookOver::Held),
        "its own hold, resumed with a grant"
    );
}

/// The answers one connection receives, in order, for a fixed sequence: a seat the grant does not
/// name, the seat it does, one request, and leaving.
async fn answers_for(address: SocketAddr, token: &str) -> Vec<String> {
    let mut client = Client::connect(address).await;
    let mut said = Vec::new();
    client.send(&join_presenting(BOB, "same", token)).await;
    let refused = joined(&mut client).await;
    said.push(format!("{refused:?}"));
    client.send(&join_presenting(ALICE, "same", token)).await;
    let Joined::Welcomed {
        observer,
        took_over,
        ..
    } = joined(&mut client).await
    else {
        panic!("the grant's seat");
    };
    said.push(format!("welcome {observer} {took_over:?}"));
    client
        .send(&json!({ "t": "submit", "token": "c1",
            "request": speak_request(observer, observer, "hello") }))
        .await;
    match client.answer().await {
        // The answer's kind and reason; an accepted answer's event identities are the world's next
        // ones and differ by construction.
        ServerFrame::Result { result, .. } => {
            let value = serde_json::to_value(&result).expect("serializes");
            said.push(if value.get("accepted").is_some() {
                "accepted".to_owned()
            } else {
                value.to_string()
            });
        }
        other => said.push(format!("{other:?}")),
    }
    client.send(&json!({ "t": "leave" })).await;
    client.closed_because(ClosingReason::Left).await;
    said.push("left".to_owned());
    said
}

/// AA-9 (AC-S27-6's shape): the same sequence presented with the same grant by two connections, one
/// after the other, is answered identically — the server applies the credential, not who holds it.
#[tokio::test]
async fn the_same_grant_is_answered_the_same_whoever_presents_it() {
    let alice = grant(&[ALICE], false);
    let address = start(std::slice::from_ref(&alice)).await;
    let first = answers_for(address, alice.token().reveal()).await;
    let second = answers_for(address, alice.token().reveal()).await;
    assert_eq!(first, second);
    assert_eq!(first[0], "Refused(NotGranted)", "{first:?}");
}

/// AA-7's library half: the server's `Access` refuses an admin token equal to the invite or to a
/// grant, because whoever held that credential would be the host. (A grant equal to the invite or to
/// another grant is refused by `Admission::with_grants`, in `admission.rs`'s unit tests: grant tokens
/// are generated, so only the library can be handed a colliding one.)
#[test]
fn an_admin_token_equal_to_any_player_credential_refuses_access() {
    use mineworld_server::{Access, AdminToken, AdmissionError};
    let visitor = grant(&[ALICE], false);
    let admission = support::admission()
        .with_grants([visitor.clone()])
        .expect("distinct");
    for colliding in [INVITE, visitor.token().reveal()] {
        let token = AdminToken::given(colliding).expect("a legal token");
        assert_eq!(
            Access::new(admission.clone(), Some(token))
                .map(|_| ())
                .expect_err("collides"),
            AdmissionError::AdminTokenNotDistinct
        );
    }
    let own = AdminToken::given("host-only-token-1").expect("a legal token");
    assert!(Access::new(admission, Some(own)).is_ok());
}
