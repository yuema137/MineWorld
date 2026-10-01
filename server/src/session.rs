//! One client's whole conversation with the world.
//!
//! A session does three things and nothing else: it seats the connection, it forwards that
//! observer's observations, and it carries submitted requests to the world and the answers back. It
//! holds no world state, evaluates no rule, and computes no perception.
//!
//! # The two phases, and why joining is a phase
//!
//! ```text
//! handshake   nothing streams yet; the only frame that gets anywhere is a join
//! seated      observations flow, and requests are dispatched as this connection's observer
//! ```
//!
//! A connection acquires its observer exactly once, from the world's seat roster, and there is no
//! frame that changes it afterwards — a second join is refused. That is what makes `INV-13`
//! structural here: perception cannot be widened by asking, because nothing in the protocol asks
//! for perception at all. What a client can say is a seat it wants and a request it would like
//! resolved.

use axum::extract::ws::{Message, WebSocket};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};

use crate::host::{Seated, WorldHost};
use crate::protocol::{
    ClientFrame, PROTOCOL_VERSION, Refusal, RefusalCode, ServerFrame, into_kernel_request,
};

/// What arrived on the socket.
enum Received {
    /// A text frame, which is the only kind this protocol carries.
    Text(String),
    /// A frame that is not text and not a control frame.
    NotText,
    /// The connection ended.
    Gone,
}

/// Runs one connection to completion, then releases its subscription.
pub(crate) async fn run(socket: WebSocket, host: WorldHost) {
    let (mut outgoing, mut incoming) = socket.split();
    let Some(mut seated) = handshake(&mut outgoing, &mut incoming, &host).await else {
        return;
    };

    let welcome = ServerFrame::Welcome {
        protocol: PROTOCOL_VERSION,
        seat: seated.seat().clone(),
        observer: seated.observer(),
        world: seated.world().clone(),
    };
    if send(&mut outgoing, &welcome).await.is_ok() {
        stream(&mut outgoing, &mut incoming, &host, &mut seated).await;
    }
    // Explicit rather than left to the sweep that would reap a closed channel anyway: a client that
    // disconnects should stop counting as a connected client immediately.
    host.leave(seated.subscription());
}

/// Reads frames until the connection has a seat, refusing everything else.
async fn handshake(
    outgoing: &mut SplitSink<WebSocket, Message>,
    incoming: &mut SplitStream<WebSocket>,
    host: &WorldHost,
) -> Option<Seated> {
    loop {
        let refusal = match receive(incoming).await {
            Received::Gone => return None,
            Received::NotText => not_text(),
            Received::Text(text) => match ClientFrame::decode(&text) {
                Err(refusal) => refusal,
                Ok(ClientFrame::Join { seat }) => match host.join(seat).await {
                    Ok(seated) => return Some(seated),
                    Err(refusal) => refusal,
                },
                Ok(ClientFrame::Submit { token, .. }) => Refusal::new(RefusalCode::NotJoined)
                    .about(Some(token))
                    .detail("join a seat before submitting a request"),
            },
        };
        send(outgoing, &refusal.into_frame()).await.ok()?;
    }
}

/// The seated phase: observations out, requests in, until either end stops.
///
/// One task, so the two directions take turns rather than running concurrently. That is a
/// deliberate limit and not a coupling: the work between turns is a `serde_json` encode and a
/// message to the world thread, and the *world* never waits for any of it — it delivers
/// observations with `try_send` and answers requests without touching a client.
async fn stream(
    outgoing: &mut SplitSink<WebSocket, Message>,
    incoming: &mut SplitStream<WebSocket>,
    host: &WorldHost,
    seated: &mut Seated,
) {
    let observer = seated.observer();
    let mut seq: u64 = 0;

    loop {
        tokio::select! {
            observation = seated.observations().recv() => {
                // `None` means the world has stopped. The connection ends with it: there is nothing
                // left to observe.
                let Some(perceived) = observation else { return };
                seq += 1;
                let frame = ServerFrame::Observation {
                    seq,
                    revision: perceived.revision,
                    observation: perceived.observation,
                };
                if send(outgoing, &frame).await.is_err() {
                    return;
                }
            }
            received = receive(incoming) => {
                let frame = match received {
                    Received::Gone => return,
                    Received::NotText => not_text().into_frame(),
                    Received::Text(text) => match ClientFrame::decode(&text) {
                        Err(refusal) => refusal.into_frame(),
                        Ok(ClientFrame::Join { .. }) => Refusal::new(RefusalCode::AlreadyJoined)
                            .detail("a connection holds one seat for its whole life")
                            .into_frame(),
                        Ok(ClientFrame::Submit { token, request }) => {
                            submit(host, observer, token, &request).await
                        }
                    },
                };
                if send(outgoing, &frame).await.is_err() {
                    return;
                }
            }
        }
    }
}

/// Carries one submitted request to the world and turns the answer into a frame.
///
/// The client's token goes onto whichever frame comes back, answer or refusal, because pairing a
/// reply with a request is what the token is for. The `action_id` on the answer is the identity the
/// *server* allocated, which is what lets a client recognize a later fact whose `Causation` names
/// its own request.
async fn submit(
    host: &WorldHost,
    observer: mineworld_contracts::EntityId,
    token: crate::protocol::CorrelationToken,
    request: &mineworld_contracts::ActionRequest<crate::protocol::WirePayload>,
) -> ServerFrame {
    let kernel_request = match into_kernel_request(request) {
        Ok(request) => request,
        Err(error) => {
            return Refusal::new(RefusalCode::MalformedFrame)
                .about(Some(token))
                .detailed(error)
                .into_frame();
        }
    };
    match host.submit(observer, kernel_request).await {
        Ok(submitted) => ServerFrame::Result {
            token,
            action_id: submitted.action_id(),
            result: submitted.result().clone(),
        },
        Err(refusal) => refusal.about(Some(token)).into_frame(),
    }
}

/// Reads the next frame that means anything, letting axum answer the control frames.
async fn receive(incoming: &mut SplitStream<WebSocket>) -> Received {
    loop {
        return match incoming.next().await {
            None | Some(Err(_)) | Some(Ok(Message::Close(_))) => Received::Gone,
            Some(Ok(Message::Text(text))) => Received::Text(text.to_string()),
            Some(Ok(Message::Ping(_) | Message::Pong(_))) => continue,
            Some(Ok(Message::Binary(_))) => Received::NotText,
        };
    }
}

fn not_text() -> Refusal {
    Refusal::new(RefusalCode::MalformedFrame)
        .detail("this protocol carries JSON text frames; a binary frame is not one")
}

/// Sends one frame, or reports that the connection is gone.
///
/// A frame that cannot be serialized is a defect in the server rather than in the client, so it is
/// reported and the connection continues: the alternative is a silent disconnect whose cause is
/// invisible at both ends.
async fn send(
    outgoing: &mut SplitSink<WebSocket, Message>,
    frame: &ServerFrame,
) -> Result<(), ConnectionGone> {
    let text = match serde_json::to_string(frame) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("[session] a server frame would not serialize: {error}");
            return Ok(());
        }
    };
    outgoing
        .send(Message::Text(text.into()))
        .await
        .map_err(|_| ConnectionGone)
}

/// The connection ended while the server was writing to it.
struct ConnectionGone;
