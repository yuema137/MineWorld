//! One client's whole conversation with the world.
//!
//! A session does four things and nothing else: it admits the connection, it seats it, it forwards
//! that observer's observations, and it carries submitted requests to the world and the answers
//! back. It holds no world state, evaluates no rule, and computes no perception.
//!
//! # The two phases, and why joining is a phase
//!
//! ```text
//! handshake   nothing streams yet; the only frame that gets anywhere is a join that passes
//!             PROTOCOL.md §4.1's checks, in order: protocol, invite, nickname, resume, seat
//! seated      observations flow, and requests are dispatched as this connection's observer
//! ```
//!
//! A connection acquires its observer exactly once, from the world's seat roster, and there is no
//! frame that changes it afterwards — a second join is refused. That is what makes `INV-13`
//! structural here: perception cannot be widened by asking, because nothing in the protocol asks
//! for perception at all. What a client can say is a seat it wants, a request it would like
//! resolved, and that it is leaving.
//!
//! # Where the invite and the nickname stop
//!
//! Both are checked here, in the connection's own task, before the world is asked for anything. The
//! world thread never receives either (`WorldHost::join` takes a seat only), so neither can reach a
//! journal, a fact or a save. The fixed delay before an `unauthorized` answer is slept here too: it
//! holds up this connection and nothing else.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{CloseFrame, Message, WebSocket, close_code};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::time::Instant;

use crate::admin::registry::{Joined, Registered, Registry};
use crate::admission::{Admission, Nickname, OfferedInvite, OfferedResume, UNAUTHORIZED_DELAY};
use crate::host::{Backfill, Seated, Streamed, Streams, SubscriptionId, WorldHost};
use crate::protocol::{
    ClientFrame, ClosingReason, PROTOCOL_VERSION, PerceivedJoin, Refusal, RefusalCode, ServerFrame,
    SessionId, backfill_frames, delta, into_kernel_request,
};
use crate::seats::{Departure, JoinRequest};

type Outgoing = SplitSink<WebSocket, Message>;
type Incoming = SplitStream<WebSocket>;

/// What arrived on the socket.
enum Received {
    /// A text frame, which is the only kind this protocol carries.
    Text(String),
    /// A frame that is not text and not a control frame.
    NotText,
    /// The connection ended.
    Gone,
}

/// How the seated phase ended.
enum Ending {
    /// The socket closed or failed; there is nobody to tell.
    Gone,
    /// The client sent `leave`.
    Left,
    /// The world stopped streaming.
    WorldStopped,
    /// The world unbound this connection from its seat: another connection took it over or
    /// superseded it.
    Released(ClosingReason),
}

/// What one `join` came to.
enum Joining {
    /// The connection has its seat, and the `perceived` backfill it is owed.
    Seated(Box<Seated>, Nickname, Vec<ServerFrame>),
    /// Refused; the connection stays in the handshake and may join again.
    Refused(Refusal),
    /// Refused, and the connection ends: a mismatched protocol, or a wrong invite.
    Closed(Refusal, ClosingReason),
}

/// What a session is given besides its socket: the world, who may join it, and its own identity.
pub(crate) struct Connection {
    /// The hosted world.
    pub(crate) host: WorldHost,
    /// The invite check.
    pub(crate) admission: std::sync::Arc<Admission>,
    /// Which connection this is.
    pub(crate) session: SessionId,
    /// Where a seated connection is listed for the admin surface.
    pub(crate) registry: Arc<Registry>,
}

/// Runs one connection to completion, then releases its subscription.
pub(crate) async fn run(socket: WebSocket, connection: Connection) {
    let (mut outgoing, mut incoming) = socket.split();
    let Some((mut seated, nickname, backfill)) =
        handshake(&mut outgoing, &mut incoming, &connection).await
    else {
        return;
    };

    let joined = Joined {
        nickname: nickname.clone(),
        seat: seated.seat().clone(),
        observer: seated.observer(),
        connected_at: unix_seconds(),
    };
    let welcome = ServerFrame::Welcome {
        protocol: PROTOCOL_VERSION,
        seat: seated.seat().clone(),
        observer: seated.observer(),
        nickname,
        session: connection.session,
        resume: Some(seated.resume().clone()),
        hold_seconds: seated.hold_seconds(),
        took_over: seated.took_over(),
        world: seated.world().clone(),
    };
    // The welcome, then the clock as it stood at the welcome (`PROTOCOL.md` §5.9), then the perceived
    // backfill (§5.8), then the stream.
    let clock = seated.clock_at_welcome().into_frame();
    let mut said = send(&mut outgoing, &welcome).await;
    for frame in std::iter::once(&clock).chain(&backfill) {
        if said.is_err() {
            break;
        }
        said = send(&mut outgoing, frame).await;
    }
    drop(backfill);
    let ending = if said.is_ok() {
        let listed = connection.registry.register(connection.session, joined);
        let ending = stream(
            &mut outgoing,
            &mut incoming,
            &connection.host,
            &mut seated,
            &listed,
        )
        .await;
        // Off the admin surface as soon as the stream ends — before the closing handshake.
        drop(listed);
        ending
    } else {
        Ending::Gone
    };
    // Explicit rather than left to the sweep that would reap a closed channel anyway: a client that
    // leaves or disconnects should stop counting as a connected client immediately — and the world
    // must learn how it ended, because a dropped socket's seat is held and a left one is not
    // (`PROTOCOL.md` §4.2). A released connection no longer holds a seat; there is nothing to say.
    let host = &connection.host;
    match ending {
        Ending::Gone => host.leave(seated.subscription(), Departure::Dropped),
        Ending::Left => {
            host.leave(seated.subscription(), Departure::Left);
            close(&mut outgoing, &mut incoming, ClosingReason::Left).await;
        }
        Ending::WorldStopped => {
            host.leave(seated.subscription(), Departure::Left);
            close(&mut outgoing, &mut incoming, ClosingReason::WorldStopped).await;
        }
        Ending::Released(ClosingReason::Lagged) => {
            // `PROTOCOL.md` §5.5: the refusal says what to do (rejoin with the cursor), then closing.
            let lagged = Refusal::new(RefusalCode::Lagged).detail(
                "the perceived stream fell too far behind; rejoin with resume and your cursor",
            );
            if send(&mut outgoing, &lagged.into_frame()).await.is_ok() {
                close(&mut outgoing, &mut incoming, ClosingReason::Lagged).await;
            }
        }
        Ending::Released(reason) => close(&mut outgoing, &mut incoming, reason).await,
    }
}

/// Reads frames until the connection has a seat, refusing everything else.
async fn handshake(
    outgoing: &mut Outgoing,
    incoming: &mut Incoming,
    connection: &Connection,
) -> Option<(Seated, Nickname, Vec<ServerFrame>)> {
    loop {
        let received = receive(incoming).await;
        let arrived = Instant::now();
        let refusal = match received {
            Received::Gone => return None,
            Received::NotText => not_text(),
            Received::Text(text) => match ClientFrame::decode(&text) {
                Err(refusal) => refusal,
                Ok(ClientFrame::Join {
                    protocol,
                    invite,
                    nickname,
                    seat,
                    resume,
                    take_over,
                    perceived,
                }) => {
                    let offered = Offered {
                        protocol,
                        invite,
                        nickname,
                        resume,
                        take_over,
                        perceived,
                    };
                    match join(connection, offered, seat, arrived).await {
                        Joining::Seated(seated, nickname, backfill) => {
                            return Some((*seated, nickname, backfill));
                        }
                        Joining::Refused(refusal) => refusal,
                        Joining::Closed(refusal, reason) => {
                            if send(outgoing, &refusal.into_frame()).await.is_ok() {
                                close(outgoing, incoming, reason).await;
                            }
                            return None;
                        }
                    }
                }
                Ok(ClientFrame::Submit { token, .. }) => Refusal::new(RefusalCode::NotJoined)
                    .about(Some(token))
                    .detail("join a seat before submitting a request"),
                Ok(ClientFrame::Leave {}) => {
                    // Nothing to release; a client that asks to go is let go.
                    close(outgoing, incoming, ClosingReason::Left).await;
                    return None;
                }
            },
        };
        send(outgoing, &refusal.into_frame()).await.ok()?;
    }
}

/// A join's credentials and options, before any of them is trusted.
struct Offered {
    protocol: u32,
    invite: OfferedInvite,
    nickname: String,
    resume: Option<OfferedResume>,
    take_over: bool,
    perceived: Option<PerceivedJoin>,
}

/// `PROTOCOL.md` §4.1's checks, in its order. The first that fails decides the answer.
async fn join(
    connection: &Connection,
    offered: Offered,
    seat: mineworld_contracts::EntityKey,
    arrived: Instant,
) -> Joining {
    if offered.protocol != PROTOCOL_VERSION {
        return Joining::Closed(
            Refusal::new(RefusalCode::ProtocolMismatch).detail(format!(
                "this server speaks protocol {PROTOCOL_VERSION}, and the join speaks {}",
                offered.protocol
            )),
            ClosingReason::ProtocolMismatch,
        );
    }
    if connection.admission.admit(&offered.invite).is_err() {
        // Measured from the frame's arrival, so the answer's timing says nothing about the check.
        tokio::time::sleep_until(arrived + UNAUTHORIZED_DELAY).await;
        return Joining::Closed(
            Refusal::new(RefusalCode::Unauthorized).detail("that is not this server's invite"),
            ClosingReason::Unauthorized,
        );
    }
    let nickname = match Nickname::new(&offered.nickname) {
        Ok(nickname) => nickname,
        Err(error) => {
            return Joining::Refused(Refusal::new(RefusalCode::InvalidNickname).detailed(error));
        }
    };
    // The seat and who may have it are the world thread's to decide (`PROTOCOL.md` §4.2): the
    // resume and the takeover flag travel with the seat, and nothing here holds a binding.
    let request = JoinRequest {
        seat,
        take_over: offered.take_over,
        resume: offered.resume,
        session: connection.session,
    };
    let seated = match connection
        .host
        .join_perceiving(request, offered.perceived)
        .await
    {
        Ok(seated) => seated,
        Err(refusal) => return Joining::Refused(refusal),
    };
    // The backfill is read before the welcome, off the world's thread: a history that cannot be read
    // now is answered `cursor_unavailable` with nothing granted, and the connection may join again.
    match backfill(&seated).await {
        Ok(frames) => Joining::Seated(Box::new(seated), nickname, frames),
        Err(refusal) => {
            connection
                .host
                .leave(seated.subscription(), Departure::Left);
            Joining::Refused(refusal)
        }
    }
}

/// The `perceived` frames a joining connection is owed before its live stream (`PROTOCOL.md`
/// §5.8): the facts in `(since, head]` its observer learned, in frames of at most
/// [`BACKFILL_FRAME`] events, the last one's `through` the head — sent even when empty.
async fn backfill(seated: &Seated) -> Result<Vec<ServerFrame>, Refusal> {
    let Some(Backfill {
        history,
        since,
        through,
    }) = seated.perceived().and_then(|start| start.backfill.clone())
    else {
        return Ok(Vec::new());
    };
    let observer = seated.observer();
    let read = tokio::task::spawn_blocking(move || {
        history
            .perceived(observer, since, through)
            .map(|facts| backfill_frames(&facts, through))
    })
    .await;
    match read {
        Ok(Ok(frames)) => Ok(frames),
        Ok(Err(unavailable)) => {
            Err(Refusal::new(RefusalCode::CursorUnavailable).detailed(unavailable))
        }
        Err(error) => Err(Refusal::new(RefusalCode::CursorUnavailable).detailed(error)),
    }
}

/// The seated phase: observations out, requests in, until either end stops.
///
/// One task, so the two directions take turns rather than running concurrently. That is a
/// deliberate limit and not a coupling: the work between turns is a `serde_json` encode and a
/// message to the world thread, and the *world* never waits for any of it — it delivers
/// observations with `try_send` and answers requests without touching a client.
async fn stream(
    outgoing: &mut Outgoing,
    incoming: &mut Incoming,
    host: &WorldHost,
    seated: &mut Seated,
    listed: &Registered,
) -> Ending {
    let observer = seated.observer();
    let subscription = seated.subscription();
    let mut encoder = delta::Encoder::new(seated.keyframe_every());
    let Streams {
        observations,
        released,
        clock,
    } = seated.streams();

    loop {
        let frame = tokio::select! {
            // First, so that a connection unbound from its seat is told why rather than being
            // told the world stopped when its observation stream ends with the binding.
            biased;
            reason = &mut *released => {
                return reason.map_or(Ending::WorldStopped, Ending::Released);
            }
            observation = observations.recv() => {
                // `None` means the world has stopped, or unbound this connection — which the
                // `released` branch, polled first, has said if so. The connection ends with it.
                let Some(streamed) = observation else {
                    return released.try_recv().map_or(Ending::WorldStopped, Ending::Released);
                };
                match streamed {
                    Streamed::Facts { through, events } => ServerFrame::Perceived {
                        through,
                        events: events.iter().map(|event| (**event).clone()).collect(),
                    },
                    // Whole at keyframes, a delta otherwise; entities in id order (§§5.2, 5.3).
                    Streamed::Observation(perceived) => encoder.frame(
                        listed.next_seq(),
                        perceived.revision,
                        perceived.acted_through,
                        perceived.observation,
                    ),
                }
            }
            changed = clock.changed() => {
                // The world thread holds the sender for as long as it runs.
                if changed.is_err() {
                    return released.try_recv().map_or(Ending::WorldStopped, Ending::Released);
                }
                clock.borrow_and_update().into_frame()
            }
            received = receive(incoming) => match received {
                Received::Gone => return Ending::Gone,
                Received::NotText => not_text().into_frame(),
                Received::Text(text) => match ClientFrame::decode(&text) {
                    Err(refusal) => refusal.into_frame(),
                    Ok(ClientFrame::Join { .. }) => Refusal::new(RefusalCode::AlreadyJoined)
                        .detail("a connection holds one seat for its whole life")
                        .into_frame(),
                    Ok(ClientFrame::Submit { token, request }) => {
                        submit(host, (subscription, observer), token, &request).await
                    }
                    Ok(ClientFrame::Leave {}) => return Ending::Left,
                },
            },
        };
        if send(outgoing, &frame).await.is_err() {
            return Ending::Gone;
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
    (subscription, observer): (SubscriptionId, mineworld_contracts::EntityId),
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
    match host
        .submit_on(Some(subscription), observer, kernel_request)
        .await
    {
        Ok(submitted) => ServerFrame::Result {
            token,
            action_id: submitted.action_id(),
            result: submitted.result().clone(),
        },
        Err(refusal) => refusal.about(Some(token)).into_frame(),
    }
}

/// Reads the next frame that means anything, letting axum answer the control frames.
async fn receive(incoming: &mut Incoming) -> Received {
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

/// How long the server waits for a client to answer its close before dropping the connection.
const CLOSE_GRACE: Duration = Duration::from_secs(2);

/// Says why, then closes the connection (`PROTOCOL.md` §5.6).
///
/// The client is given the chance to close first: after `closing`, whatever it still sends is read
/// and ignored until it closes the connection itself or [`CLOSE_GRACE`] passes, and only then does
/// the server send its own close frame. Closing at once instead loses the frames just sent to a
/// client whose WebSocket discards what it has not yet read when a close arrives in the same read —
/// measured with Godot's `WebSocketPeer`, which reported a bare closed connection and never the
/// refusal and `closing` before it. A client that is already gone simply does not hear it.
async fn close(outgoing: &mut Outgoing, incoming: &mut Incoming, reason: ClosingReason) {
    let said = send(
        outgoing,
        &ServerFrame::Closing {
            reason,
            detail: None,
        },
    )
    .await;
    if said.is_ok() {
        let _ = tokio::time::timeout(CLOSE_GRACE, async {
            while !matches!(receive(incoming).await, Received::Gone) {}
        })
        .await;
        let goodbye = CloseFrame {
            code: close_code::NORMAL,
            reason: "".into(),
        };
        let _ = outgoing.send(Message::Close(Some(goodbye))).await;
    }
    let _ = outgoing.close().await;
}

/// Sends one frame, or reports that the connection is gone.
///
/// A frame that cannot be serialized is a defect in the server rather than in the client, so it is
/// reported and the connection continues: the alternative is a silent disconnect whose cause is
/// invisible at both ends.
async fn send(outgoing: &mut Outgoing, frame: &ServerFrame) -> Result<(), ConnectionGone> {
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

/// Wall-clock Unix seconds, for `connected_at` on the admin surface — host state, never a
/// `WorldTime`.
fn unix_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

/// The connection ended while the server was writing to it.
struct ConnectionGone;
