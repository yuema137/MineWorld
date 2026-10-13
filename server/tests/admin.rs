//! The admin surface over a real socket and real HTTP (step-12 §18.5 DA-2, DA-5's in-process half,
//! DA-8, DA-10's socket half).
//!
//! `PROTOCOL.md` §11: the routes exist only with an admin token, every permission failure is a `401`
//! no sooner than 500 ms after the request, no body can name an instant, and nothing an invite
//! holder sends on the socket changes time. The HTTP client is hand-written, one request per
//! connection, because the claim includes exactly which bytes a request carries.

mod support;

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use mineworld_server::{
    Access, AdminToken, ClosingReason, RefusalCode, ServerFrame, SessionId, WorldHost, app,
};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use support::{ALICE, BOB, INVITE, brisk, build, join_frame};

/// The admin token every server in this file is started with.
const TOKEN: &str = "admin-test-token-5d2e";

/// How long a test waits for something that should take milliseconds.
const PATIENCE: Duration = Duration::from_secs(5);

async fn start(admin: Option<&str>) -> SocketAddr {
    let host = WorldHost::spawn(brisk(), build)
        .await
        .expect("the world is assembled and its thread starts");
    let (listener, address) = app::bind("127.0.0.1:0".parse().expect("a literal address"))
        .await
        .expect("an ephemeral port");
    let access = Access {
        admission: support::admission(),
        admin: admin.map(|token| AdminToken::given(token).expect("a legal token")),
    };
    tokio::spawn(async move {
        let _ = app::serve(listener, host, access).await;
    });
    address
}

/// One HTTP answer: its status, its JSON body, and how long after sending it arrived.
struct Answer {
    status: u16,
    body: Value,
    took: Duration,
}

/// One HTTP/1.1 request with exactly these extra header lines, on a connection of its own.
async fn http(
    address: SocketAddr,
    method: &str,
    path: &str,
    headers: &[String],
    body: Option<&str>,
) -> Answer {
    let mut socket = TcpStream::connect(address).await.expect("connects");
    let mut request =
        format!("{method} {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n");
    for header in headers {
        request.push_str(header);
        request.push_str("\r\n");
    }
    let body = body.unwrap_or("");
    if method == "POST" {
        request.push_str(&format!(
            "Content-Type: application/json\r\nContent-Length: {}\r\n",
            body.len()
        ));
    }
    request.push_str("\r\n");
    request.push_str(body);
    let sent = Instant::now();
    socket
        .write_all(request.as_bytes())
        .await
        .expect("the request is sent");
    let mut response = String::new();
    socket
        .read_to_string(&mut response)
        .await
        .expect("the response is read");
    let took = sent.elapsed();
    let status = response
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .unwrap_or_else(|| panic!("an HTTP status line: {response}"));
    let text = response.split_once("\r\n\r\n").map_or("", |(_, body)| body);
    let body = serde_json::from_str(text.trim()).unwrap_or(Value::Null);
    Answer { status, body, took }
}

fn bearer(token: &str) -> Vec<String> {
    vec![format!("Authorization: Bearer {token}")]
}

async fn admin_get(address: SocketAddr, path: &str) -> Answer {
    http(address, "GET", path, &bearer(TOKEN), None).await
}

async fn admin_post(address: SocketAddr, path: &str, body: &str) -> Answer {
    http(address, "POST", path, &bearer(TOKEN), Some(body)).await
}

async fn status(address: SocketAddr) -> Value {
    http(address, "GET", "/status", &[], None).await.body
}

/// RFC 4648 base64, for the one `Basic` header a test sends.
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(char::from(ALPHABET[((n >> (18 - 6 * i)) & 63) as usize]));
            } else {
                out.push('=');
            }
        }
    }
    out
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

    /// Joins a seat; returns the session, after checking that the clock frame comes next.
    async fn join(&mut self, seat: &str, nickname: &str) -> (SessionId, Value) {
        self.send(&join_frame(seat, nickname)).await;
        let ServerFrame::Welcome { session, world, .. } =
            self.next().await.expect("the connection is open")
        else {
            panic!("a join is answered with a welcome");
        };
        // DA-10: a clock frame follows every welcome, before the first observation, and says
        // what the welcome's world said.
        match self.next().await.expect("the connection is open") {
            ServerFrame::Clock {
                at,
                time_scale,
                paused,
            } => {
                assert_eq!(
                    (at, time_scale, paused),
                    (world.at, world.time_scale, world.paused)
                );
            }
            other => panic!("the frame after a welcome is a clock, and this was {other:?}"),
        }
        (session, json!(world))
    }

    /// The next clock frame's `paused`, skipping observations.
    async fn clock(&mut self) -> bool {
        match self.answer().await {
            ServerFrame::Clock { paused, .. } => paused,
            other => panic!("a clock frame was expected, and the server said {other:?}"),
        }
    }

    /// Reads a `closing` with this reason, then insists that the socket closes.
    async fn closed_because(&mut self, expected: ClosingReason) {
        match self.answer().await {
            ServerFrame::Closing { reason, .. } => assert_eq!(reason, expected),
            other => panic!("a closing was expected, and the server said {other:?}"),
        }
        while let Some(frame) = self.next().await {
            assert!(
                matches!(frame, ServerFrame::Observation { .. }),
                "{frame:?}"
            );
        }
    }

    /// Nothing but observations for a while: no closing, no refusal, no clock.
    async fn only_observations_for(&mut self, quiet: Duration) {
        let until = Instant::now() + quiet;
        while Instant::now() < until {
            let frame = self.next().await.expect("still connected");
            assert!(
                matches!(frame, ServerFrame::Observation { .. }),
                "only observations were expected: {frame:?}"
            );
        }
    }
}

fn speak(actor: &Value, target: &Value) -> Value {
    json!({ "t": "submit", "token": "c4", "request": {
        "actor": actor, "action_type": "speak", "target": target,
        "payload": { "action_type": "speak", "payload": { "words": "hello" } },
        "actor_location": null } })
}

// ---------------------------------------------------------------------------------------------
// DA-2 — permission refusals.
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn every_permission_failure_is_unauthorized_after_half_a_second_and_changes_nothing() {
    let address = start(Some(TOKEN)).await;
    let mut alice = Client::connect(address).await;
    let (session, _) = alice.join(ALICE, "Avery").await;

    let mut last_changed = TOKEN.to_owned();
    last_changed.pop();
    last_changed.push('f');
    let refused: Vec<(&str, Vec<String>, &str)> = vec![
        ("no Authorization header", vec![], ""),
        ("the invite", bearer(INVITE), ""),
        (
            "the token with its last character changed",
            bearer(&last_changed),
            "",
        ),
        (
            // A literal trailing space is optional whitespace that HTTP strips before any server
            // sees it (RFC 9110 §5.5; step-12 D-SD4), so the case is the token, a space, and more.
            "the token followed by a space and more",
            vec![format!("Authorization: Bearer {TOKEN} x")],
            "",
        ),
        (
            "Basic credentials",
            vec![format!("Authorization: Basic {}", base64(TOKEN.as_bytes()))],
            "",
        ),
        (
            "the token in the query string",
            vec![],
            "?token=admin-test-token-5d2e",
        ),
        (
            "a lower-case scheme with a wrong token",
            vec![format!("Authorization: bearer {last_changed}")],
            "",
        ),
    ];
    let kick = format!("/admin/sessions/{session}/kick");
    for (case, headers, query) in &refused {
        for (path, body) in [("/admin/clock", r#"{"paused": true}"#), (kick.as_str(), "")] {
            let answer = http(
                address,
                "POST",
                &format!("{path}{query}"),
                headers,
                Some(body),
            )
            .await;
            assert_eq!(answer.status, 401, "{case} on {path}: {:?}", answer.body);
            assert_eq!(
                answer.body,
                json!({ "error": "unauthorized" }),
                "{case} on {path}"
            );
            assert!(
                answer.took >= Duration::from_millis(500),
                "{case} on {path} was answered after {:?}, under 500 ms",
                answer.took
            );
        }
    }

    let clock = admin_get(address, "/admin/clock").await;
    assert_eq!(clock.status, 200);
    assert_eq!(
        clock.body["paused"],
        json!(false),
        "no refused request paused the clock"
    );
    let seats = admin_get(address, "/admin/seats").await;
    assert_eq!(
        seats.body["seats"][0],
        json!({ "seat": "alice", "state": "connected", "session": session.to_string() }),
        "the targeted client is still seated"
    );
    alice
        .only_observations_for(Duration::from_millis(300))
        .await;
}

// ---------------------------------------------------------------------------------------------
// DA-8 — only the host changes time; the client vocabulary is closed.
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn an_invite_holder_cannot_change_time_and_no_body_names_an_instant() {
    let address = start(Some(TOKEN)).await;
    let mut alice = Client::connect(address).await;
    alice.join(ALICE, "Avery").await;
    for frame in [
        json!({ "t": "pause" }),
        json!({ "t": "clock", "paused": true }),
        json!({ "t": "kick", "session": "1" }),
        json!({ "t": "release", "seat": "alice" }),
    ] {
        alice.send(&frame).await;
        match alice.answer().await {
            ServerFrame::Refused { code, .. } => {
                assert_eq!(code, RefusalCode::UnknownFrame, "{frame}");
            }
            other => panic!("{frame} was answered {other:?}"),
        }
    }
    let mut bob = Client::connect(address).await;
    let mut join = join_frame(BOB, "Bea");
    join["admin_token"] = json!(TOKEN);
    bob.send(&join).await;
    match bob.answer().await {
        ServerFrame::Refused { code, .. } => assert_eq!(code, RefusalCode::MalformedFrame),
        other => panic!("a join carrying admin_token was answered {other:?}"),
    }

    let before = status(address).await;
    for (body, code, error) in [
        (r#"{"at": 999999}"#, 400, "malformed"),
        (r#"{"paused": true, "at": 5}"#, 400, "malformed"),
        (r#"{"time_scale": 12}"#, 409, "time_scale_fixed"),
        (
            r#"{"paused": true, "time_scale": 12}"#,
            409,
            "time_scale_fixed",
        ),
        ("{}", 400, "malformed"),
        ("not json", 400, "malformed"),
    ] {
        let answer = admin_post(address, "/admin/clock", body).await;
        assert_eq!(answer.status, code, "{body}: {:?}", answer.body);
        assert_eq!(answer.body["error"], json!(error), "{body}");
    }
    let after = status(address).await;
    assert_eq!(
        after["paused"],
        json!(false),
        "no refused body paused the clock"
    );
    assert_eq!(after["time_scale"], before["time_scale"]);
    assert_eq!(after["revision"], before["revision"]);
    tokio::time::sleep(Duration::from_millis(1_100)).await;
    let later = status(address).await;
    assert!(
        later["at"].as_i64() > before["at"].as_i64(),
        "the clock still runs: {later}"
    );
}

// ---------------------------------------------------------------------------------------------
// DA-10's socket half and the in-process pause.
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn a_pause_is_announced_refuses_submits_and_resumes_where_it_stopped() {
    let address = start(Some(TOKEN)).await;
    let mut alice = Client::connect(address).await;
    alice.join(ALICE, "Avery").await;
    let mut bob = Client::connect(address).await;
    bob.join(BOB, "Bea").await;

    let paused = admin_post(address, "/admin/clock", r#"{"paused": true}"#).await;
    assert_eq!(paused.status, 200);
    assert_eq!(paused.body["paused"], json!(true));
    assert!(alice.clock().await, "alice is told the clock stopped");
    assert!(bob.clock().await, "bob is told the clock stopped");
    let frozen = status(address).await;
    assert_eq!(frozen["paused"], json!(true));

    // Repeating the current state is 200 and announces nothing.
    let again = admin_post(address, "/admin/clock", r#"{"paused": true}"#).await;
    assert_eq!(again.status, 200);

    let alice_id =
        json!(admin_get(address, "/admin/sessions").await.body["sessions"][0]["observer"]);
    alice.send(&speak(&alice_id, &alice_id)).await;
    match alice.answer().await {
        ServerFrame::Refused { code, token, .. } => {
            assert_eq!(code, RefusalCode::Paused);
            assert_eq!(
                token.map(|token| token.as_str().to_owned()),
                Some("c4".to_owned())
            );
        }
        other => panic!("a submit while paused was answered {other:?}"),
    }
    tokio::time::sleep(Duration::from_millis(1_200)).await;
    assert_eq!(
        status(address).await["at"],
        frozen["at"],
        "a paused clock stands still"
    );

    let resumed = admin_post(address, "/admin/clock", r#"{"paused": false}"#).await;
    assert_eq!(resumed.body["paused"], json!(false));
    assert_eq!(
        resumed.body["at"], frozen["at"],
        "it resumes where it stopped"
    );
    assert!(!alice.clock().await, "alice is told the clock runs again");
    assert!(!bob.clock().await);
    let clock = admin_get(address, "/admin/clock").await;
    assert_eq!(clock.body["time_scale"], json!(1));
}

// ---------------------------------------------------------------------------------------------
// DA-5's in-process half: kick, release and the sessions list.
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn a_kick_and_a_release_close_the_connection_kicked_and_free_the_seat() {
    let address = start(Some(TOKEN)).await;
    let mut alice = Client::connect(address).await;
    let (alice_session, _) = alice.join(ALICE, "Avery").await;
    let mut bob = Client::connect(address).await;
    let (bob_session, _) = bob.join(BOB, "Bea").await;

    let listed = admin_get(address, "/admin/sessions").await.body["sessions"].clone();
    let names: Vec<(String, String, String)> = listed
        .as_array()
        .expect("a list")
        .iter()
        .map(|session| {
            (
                session["session"].as_str().expect("a string").to_owned(),
                session["nickname"].as_str().expect("a string").to_owned(),
                session["seat"].as_str().expect("a string").to_owned(),
            )
        })
        .collect();
    assert_eq!(
        names,
        vec![
            (
                alice_session.to_string(),
                "Avery".to_owned(),
                "alice".to_owned()
            ),
            (bob_session.to_string(), "Bea".to_owned(), "bob".to_owned()),
        ]
    );
    assert_eq!(listed[0]["observations_dropped"], json!(0));
    assert!(
        listed[0]["connected_at"]
            .as_u64()
            .is_some_and(|at| at > 1_700_000_000)
    );

    let kicked = admin_post(
        address,
        &format!("/admin/sessions/{alice_session}/kick"),
        "",
    )
    .await;
    assert_eq!(kicked.status, 200);
    assert_eq!(
        kicked.body,
        json!({ "session": alice_session.to_string(), "seat": "alice", "state": "free" })
    );
    alice.closed_because(ClosingReason::Kicked).await;
    let after = admin_get(address, "/admin/sessions").await.body;
    assert_eq!(
        after["sessions"].as_array().map(Vec::len),
        Some(1),
        "{after}"
    );

    let unknown = admin_post(address, "/admin/sessions/999/kick", "").await;
    assert_eq!(
        (unknown.status, unknown.body["error"].clone()),
        (404, json!("unknown_session"))
    );
    let malformed = admin_post(address, "/admin/sessions/seven/kick", "").await;
    assert_eq!(malformed.status, 400);

    let released = admin_post(address, "/admin/seats/bob/release", "").await;
    assert_eq!(
        released.body,
        json!({ "seat": "bob", "released": true, "state": "free" })
    );
    bob.closed_because(ClosingReason::Kicked).await;
    let again = admin_post(address, "/admin/seats/bob/release", "").await;
    assert_eq!(
        again.body,
        json!({ "seat": "bob", "released": false, "state": "free" })
    );
    let unknown = admin_post(address, "/admin/seats/otto/release", "").await;
    assert_eq!(
        (unknown.status, unknown.body["error"].clone()),
        (404, json!("unknown_seat"))
    );
}

#[tokio::test]
async fn without_a_token_there_is_no_admin_surface() {
    let address = start(None).await;
    for (method, path) in [
        ("GET", "/admin/sessions"),
        ("GET", "/admin/seats"),
        ("POST", "/admin/sessions/1/kick"),
        ("POST", "/admin/seats/alice/release"),
        ("GET", "/admin/clock"),
        ("POST", "/admin/clock"),
    ] {
        let answer = http(address, method, path, &bearer(TOKEN), Some("{}")).await;
        assert_eq!(answer.status, 404, "{method} {path}");
    }
}
