//! Protocol revision 2's handshake over a real socket (step-12 §15.4 SA-1, SA-2, SA-5, SA-6).
//!
//! `PROTOCOL.md` §4.1 fixes the order of a join's checks — protocol, invite, nickname, resume, seat —
//! and what each failure costs the connection: a mismatched protocol or a wrong invite ends it, the
//! others leave it in the handshake. Every bound here is a literal from that specification: 500 ms,
//! 32 scalar values, the closing reasons. Nothing is mocked: a real world on its own thread, a real
//! router on an ephemeral port, a `tokio-tungstenite` client over TCP.

mod support;

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use mineworld_server::{
    ClosingReason, RefusalCode, ResumeSecret, ServerFrame, SessionId, TookOver, WorldHost,
    WorldSummary, app,
};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use support::{ALICE, BOB, INVITE, brisk, build, join_frame};

/// How long a test waits for something that should take milliseconds.
const PATIENCE: Duration = Duration::from_secs(5);

/// `PROTOCOL.md` §4.1: a wrong invite is answered no sooner than this after the join.
const UNAUTHORIZED_NOT_BEFORE: Duration = Duration::from_millis(500);

async fn start() -> SocketAddr {
    let host = WorldHost::spawn(brisk(), build)
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

    /// The next text frame, as text, or `None` once the server has closed the connection.
    async fn text(&mut self) -> Option<String> {
        loop {
            let next = tokio::time::timeout(PATIENCE, self.socket.next())
                .await
                .expect("something arrives within the patience of this test");
            match next {
                None | Some(Err(_)) | Some(Ok(Message::Close(_))) => return None,
                Some(Ok(Message::Text(text))) => return Some(text.to_string()),
                Some(Ok(_)) => {}
            }
        }
    }

    /// The next frame, which must exist.
    async fn frame(&mut self) -> ServerFrame {
        let text = self.text().await.expect("the connection is open");
        serde_json::from_str(&text).expect("a server frame")
    }

    /// The next frame that is not part of the stream (an observation or a clock).
    async fn answer(&mut self) -> ServerFrame {
        loop {
            let frame = self.frame().await;
            if !matches!(
                frame,
                ServerFrame::Observation { .. } | ServerFrame::Clock { .. }
            ) {
                return frame;
            }
        }
    }

    async fn refused(&mut self) -> RefusalCode {
        match self.answer().await {
            ServerFrame::Refused { code, .. } => code,
            other => panic!("a refusal was expected, and the server said {other:?}"),
        }
    }

    /// Reads a `closing` with this reason, then insists that the socket is closed.
    async fn closed_because(&mut self, expected: ClosingReason) {
        match self.answer().await {
            ServerFrame::Closing { reason, .. } => assert_eq!(reason, expected),
            other => panic!("a closing was expected, and the server said {other:?}"),
        }
        assert!(
            self.text().await.is_none(),
            "nothing follows a closing: the server closes the socket"
        );
    }

    async fn welcome(&mut self) -> Welcomed {
        match self.answer().await {
            ServerFrame::Welcome {
                protocol,
                nickname,
                session,
                resume,
                hold_seconds,
                took_over,
                world,
                ..
            } => Welcomed {
                protocol,
                nickname: nickname.as_str().to_owned(),
                session,
                resume,
                hold_seconds,
                took_over,
                world,
            },
            other => panic!("a welcome was expected, and the server said {other:?}"),
        }
    }
}

struct Welcomed {
    protocol: u32,
    nickname: String,
    session: SessionId,
    resume: Option<ResumeSecret>,
    hold_seconds: u32,
    took_over: TookOver,
    world: WorldSummary,
}

async fn status(address: SocketAddr) -> Value {
    let mut stream = TcpStream::connect(address).await.expect("a connection");
    let request = format!("GET /status HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).await.expect("written");
    let mut response = String::new();
    tokio::time::timeout(PATIENCE, stream.read_to_string(&mut response))
        .await
        .expect("answered")
        .expect("text");
    let body = response.split_once("\r\n\r\n").expect("a body").1;
    serde_json::from_str(body).expect("JSON")
}

// ---------------------------------------------------------------------------------------------
// SA-1 — the protocol is checked first.
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn a_join_of_another_revision_is_told_so_and_closed_even_with_the_right_invite() {
    let address = start().await;
    for join in [
        json!({ "t": "join", "protocol": 1, "invite": INVITE, "nickname": "Yue", "seat": ALICE }),
        // Revision 1's own join, verbatim: it has no protocol field at all.
        json!({ "t": "join", "seat": BOB }),
    ] {
        let mut client = Client::connect(address).await;
        client.send(&join).await;
        assert_eq!(
            client.refused().await,
            RefusalCode::ProtocolMismatch,
            "{join}"
        );
        client.closed_because(ClosingReason::ProtocolMismatch).await;
    }
}

// ---------------------------------------------------------------------------------------------
// SA-2 — one guess, answered late, and the connection ends.
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn a_wrong_or_missing_invite_is_refused_no_sooner_than_half_a_second_and_closed() {
    let address = start().await;
    let mut last_changed = INVITE.to_owned();
    last_changed.pop();
    last_changed.push('0');
    let joins = [
        json!({ "t": "join", "protocol": 2, "nickname": "Yue", "seat": ALICE }),
        json!({ "t": "join", "protocol": 2, "invite": last_changed, "nickname": "Yue",
                "seat": ALICE }),
        json!({ "t": "join", "protocol": 2, "invite": format!("{INVITE} "), "nickname": "Yue",
                "seat": ALICE }),
    ];
    for join in joins {
        let mut client = Client::connect(address).await;
        let sent = Instant::now();
        client.send(&join).await;
        assert_eq!(client.refused().await, RefusalCode::Unauthorized, "{join}");
        let waited = sent.elapsed();
        assert!(
            waited >= UNAUTHORIZED_NOT_BEFORE,
            "a wrong invite was answered after {waited:?}, sooner than {UNAUTHORIZED_NOT_BEFORE:?}"
        );
        client.closed_because(ClosingReason::Unauthorized).await;
    }
}

// ---------------------------------------------------------------------------------------------
// SA-5 — nicknames: checked, trimmed, and shown to nobody but their own connection.
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn an_invalid_nickname_is_refused_and_the_connection_may_try_again() {
    let address = start().await;
    let mut client = Client::connect(address).await;
    for nickname in ["   ".to_owned(), "é".repeat(33), "bell\u{7}".to_owned()] {
        client.send(&join_frame(ALICE, &nickname)).await;
        assert_eq!(
            client.refused().await,
            RefusalCode::InvalidNickname,
            "{nickname:?}"
        );
    }
    let longest = "é".repeat(32);
    client.send(&join_frame(ALICE, &longest)).await;
    assert_eq!(
        client.welcome().await.nickname,
        longest,
        "32 scalar values is a nickname, however many bytes they take"
    );
}

#[tokio::test]
async fn a_nickname_reaches_its_own_connection_trimmed_and_nobody_else() {
    const SECRET: &str = "Zephyrine-7";
    let address = start().await;
    let mut player = Client::connect(address).await;
    player.send(&join_frame(ALICE, "  Zephyrine-7 ")).await;
    assert_eq!(player.welcome().await.nickname, SECRET);

    let mut other = Client::connect(address).await;
    other.send(&join_frame(BOB, "Bob's player")).await;
    let welcome = other.text().await.expect("a welcome");
    let ServerFrame::Welcome { observer, .. } = serde_json::from_str(&welcome).expect("a frame")
    else {
        panic!("a join is answered with a welcome: {welcome}");
    };
    let mut heard = vec![welcome];
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        heard.push(other.text().await.expect("the stream runs"));
    }
    other
        .send(&json!({ "t": "set_state", "nickname": SECRET }))
        .await;
    other
        .send(&json!({ "t": "submit", "token": "w1",
                        "request": support::whisper_request(observer, "hello") }))
        .await;
    for _ in 0..40 {
        heard.push(other.text().await.expect("the stream runs"));
    }
    heard.push(status(address).await.to_string());

    for text in &heard {
        assert!(
            !text.contains(SECRET),
            "another connection was shown a nickname: {text}"
        );
    }
    assert!(
        heard.iter().any(|text| text.contains("\"t\":\"result\"")),
        "the other connection's own answer was among what it read"
    );
}

// ---------------------------------------------------------------------------------------------
// SA-6 — the welcome of revision 2, a resume that matches no hold, and leave (as S11-B made them).
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn a_welcome_says_who_this_connection_is_and_that_nothing_was_taken_over() {
    let address = start().await;
    let mut first = Client::connect(address).await;
    first.send(&join_frame(ALICE, "first")).await;
    let one = first.welcome().await;
    let mut second = Client::connect(address).await;
    second.send(&join_frame(BOB, "second")).await;
    let two = second.welcome().await;

    for welcomed in [&one, &two] {
        assert_eq!(welcomed.protocol, 2);
        let resume = welcomed.resume.as_ref().expect("a resume secret (S11-B)");
        assert_eq!(resume.reveal().len(), 32, "128 bits in hexadecimal");
        assert_eq!(welcomed.hold_seconds, 30, "the server's default hold");
        assert_eq!(welcomed.took_over, TookOver::None, "both seats were free");
        assert_eq!(welcomed.world.protocol, 2);
        assert_eq!(welcomed.world.time_scale, 1);
    }
    assert_ne!(one.session, two.session, "two connections are two sessions");
    assert_ne!(one.resume, two.resume, "and two secrets");
}

#[tokio::test]
async fn a_resume_that_matches_no_hold_is_refused_and_the_connection_joins_without_it() {
    let address = start().await;
    let mut client = Client::connect(address).await;
    let mut join = join_frame(ALICE, "Yue");
    join["resume"] = json!("00112233445566778899aabbccddeeff");
    client.send(&join).await;
    assert_eq!(client.refused().await, RefusalCode::InvalidResume);
    client.send(&join_frame(ALICE, "Yue")).await;
    assert_eq!(client.welcome().await.protocol, 2);
}

#[tokio::test]
async fn leave_gives_the_seat_up_at_once_and_ends_the_connection() {
    let address = start().await;
    let mut staying = Client::connect(address).await;
    staying.send(&join_frame(BOB, "staying")).await;
    let _ = staying.welcome().await;
    let mut leaving = Client::connect(address).await;
    leaving.send(&join_frame(ALICE, "leaving")).await;
    let _ = leaving.welcome().await;
    assert_eq!(status(address).await["clients"], json!(2));

    leaving.send(&json!({ "t": "leave" })).await;
    leaving.closed_because(ClosingReason::Left).await;
    assert_eq!(
        status(address).await["clients"],
        json!(1),
        "the seat was released when the client left, not at some later sweep"
    );

    let mut never_joined = Client::connect(address).await;
    never_joined.send(&json!({ "t": "leave" })).await;
    never_joined.closed_because(ClosingReason::Left).await;
}
