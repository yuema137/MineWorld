//! The per-seat request budget over a real socket (S27-a, `ARC-97`; `PROTOCOL.md` §8): AA-4 and AA-5
//! of `pr-s27a-seat-grants-budget.md` §8.
//!
//! ```text
//! a flood        every submit beyond the seat's bucket is refused rate_limited, retry_after_ms ≥ 1,
//!                before an ActionId is allocated: the answered requests' ActionIds have no gap
//! keyed by seat  leaving and joining again, or being taken over, does not refill the bucket
//! ```
//!
//! The bucket's arithmetic (burst, refill, exact wait) and the 2D trace replay (AA-6) are pinned with
//! injected instants in `src/seats/budget_tests.rs`; here, the wire and the allocation order.

mod support;

use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use mineworld_contracts::{ActionId, EntityId};
use mineworld_server::{HostConfig, RefusalCode, RequestBudget, ServerFrame, WorldHost, app};
use serde_json::{Value, json};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use support::{ALICE, brisk, build, join_frame, whisper_request};

/// How long a test waits for something that should take milliseconds.
const PATIENCE: Duration = Duration::from_secs(10);

/// A server with this host configuration, admitting the test invite.
async fn start(config: HostConfig) -> SocketAddr {
    let host = WorldHost::spawn(config, build)
        .await
        .expect("the world starts");
    let (listener, address) = app::bind("127.0.0.1:0".parse().expect("an address"))
        .await
        .expect("an ephemeral port");
    tokio::spawn(async move {
        let _ = app::serve(listener, host, support::admission()).await;
    });
    address
}

/// The default budget, `R = 10`, `B = 20`, with the brisk stream.
fn defaulted() -> HostConfig {
    let config = brisk();
    assert_eq!(config.request_budget, Some(RequestBudget::DEFAULT));
    config
}

/// One request a second, a burst of five: a bucket that does not refill while a test reconnects.
fn slow() -> HostConfig {
    HostConfig {
        request_budget: Some(RequestBudget {
            per_second: NonZeroU32::MIN,
            burst: NonZeroU32::new(5).expect("non-zero"),
        }),
        ..brisk()
    }
}

/// What a submit came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Answer {
    Result(ActionId),
    RateLimited { retry_after_ms: u64 },
}

struct Client {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    observer: Option<EntityId>,
    submitted: u32,
}

impl Client {
    async fn connect(address: SocketAddr) -> Self {
        let (socket, _) = tokio_tungstenite::connect_async(format!("ws://{address}/ws"))
            .await
            .expect("the server upgrades the connection");
        Self {
            socket,
            observer: None,
            submitted: 0,
        }
    }

    async fn send(&mut self, frame: &Value) {
        self.socket
            .send(Message::Text(frame.to_string().into()))
            .await
            .expect("the frame is sent");
    }

    /// The next frame that is not part of the stream.
    async fn answer(&mut self) -> ServerFrame {
        loop {
            let next = tokio::time::timeout(PATIENCE, self.socket.next())
                .await
                .expect("an answer within the patience of this test")
                .expect("the connection is open")
                .expect("a frame");
            if let Message::Text(text) = next {
                let frame: ServerFrame = serde_json::from_str(&text).expect("a server frame");
                if !matches!(
                    frame,
                    ServerFrame::Observation { .. } | ServerFrame::Clock { .. }
                ) {
                    return frame;
                }
            }
        }
    }

    async fn join(&mut self, frame: Value) {
        self.send(&frame).await;
        match self.answer().await {
            ServerFrame::Welcome { observer, .. } => self.observer = Some(observer),
            other => panic!("a welcome was expected, and the server said {other:?}"),
        }
    }

    /// Sends `n` whispers without waiting, then reads their `n` answers in order.
    async fn flood(&mut self, n: u32) -> Vec<Answer> {
        let me = self.observer.expect("seated");
        for _ in 0..n {
            self.submitted += 1;
            let token = format!("t{}", self.submitted);
            self.send(&json!({ "t": "submit", "token": token,
                               "request": whisper_request(me, "hush") }))
                .await;
        }
        let mut answers = Vec::new();
        while answers.len() < n as usize {
            match self.answer().await {
                ServerFrame::Result { action_id, .. } => answers.push(Answer::Result(action_id)),
                ServerFrame::Refused {
                    code: RefusalCode::RateLimited,
                    retry_after_ms,
                    ..
                } => answers.push(Answer::RateLimited {
                    retry_after_ms: retry_after_ms.expect("a rate_limited refusal says when"),
                }),
                other => panic!("a result or rate_limited was expected: {other:?}"),
            }
        }
        answers
    }
}

fn limited(answers: &[Answer]) -> usize {
    answers
        .iter()
        .filter(|answer| matches!(answer, Answer::RateLimited { .. }))
        .count()
}

/// AA-4 and M-a-4's oracle: 50 requests at once. The burst and whatever refilled while they were
/// answered go through; every other is `rate_limited` with `retry_after_ms ≥ 1`; and because a
/// refused request is refused before allocation, the answered ones' `ActionId`s are consecutive.
#[tokio::test]
async fn a_refused_request_allocates_nothing() {
    let address = start(defaulted()).await;
    let mut client = Client::connect(address).await;
    client.join(join_frame(ALICE, "flooder")).await;
    let answers = client.flood(50).await;

    let ids: Vec<u64> = answers
        .iter()
        .filter_map(|answer| match answer {
            Answer::Result(id) => Some(id.raw()),
            Answer::RateLimited { .. } => None,
        })
        .collect();
    assert!(
        (20..50).contains(&ids.len()),
        "the burst of 20, and at most what refilled, are answered: {answers:?}"
    );
    assert_eq!(limited(&answers), 50 - ids.len());
    assert!(
        ids.windows(2).all(|pair| pair[1] == pair[0] + 1),
        "a refused request took no ActionId: {ids:?}"
    );
    for answer in &answers {
        if let Answer::RateLimited { retry_after_ms } = answer {
            assert!(
                (1..=100).contains(retry_after_ms),
                "one token at 10 a second is at most 100 ms away: {retry_after_ms}"
            );
        }
    }

    // Waiting what the server said admits the next one.
    let Some(Answer::RateLimited { retry_after_ms }) = answers.last().copied() else {
        panic!("the last of 50 was refused: {answers:?}");
    };
    tokio::time::sleep(Duration::from_millis(retry_after_ms)).await;
    assert!(matches!(client.flood(1).await[..], [Answer::Result(_)]));
}

/// AA-5 and M-a-3's oracle: the bucket is the seat's. A connection drains alice's, leaves; a new one
/// joins alice and is refused at once; a third takes alice over and is refused at once. (`R` is one a
/// second, so nothing refills while the test reconnects.)
#[tokio::test]
async fn reconnecting_does_not_refill_the_budget() {
    let address = start(slow()).await;
    let mut first = Client::connect(address).await;
    first.join(join_frame(ALICE, "first")).await;
    let drained = first.flood(6).await;
    assert_eq!(
        limited(&drained),
        1,
        "the burst of 5, then refused: {drained:?}"
    );
    first.send(&json!({ "t": "leave" })).await;
    assert!(
        matches!(first.answer().await, ServerFrame::Closing { .. }),
        "the seat is given back before the next connection asks for it"
    );

    let mut second = Client::connect(address).await;
    second.join(join_frame(ALICE, "second")).await;
    assert_eq!(
        limited(&second.flood(1).await),
        1,
        "a fresh join of the seat is not a fresh bucket"
    );

    let mut third = Client::connect(address).await;
    let mut taking = join_frame(ALICE, "third");
    taking["take_over"] = json!(true);
    third.join(taking).await;
    assert_eq!(
        limited(&third.flood(1).await),
        1,
        "a takeover is not a fresh bucket"
    );
}
