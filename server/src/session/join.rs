//! One `join`: `PROTOCOL.md` §4.1's checks, in its order, and the backfill a seated join is owed.
//!
//! Nothing here touches the socket; the handshake in `session.rs` sends what this decides.

use tokio::time::Instant;

use super::Connection;
use crate::admission::{Nickname, OfferedInvite, OfferedResume, UNAUTHORIZED_DELAY};
use crate::host::Seated;
use crate::protocol::{
    ClosingReason, PROTOCOL_VERSION, PerceivedJoin, Refusal, RefusalCode, ServerFrame,
    read_backfill,
};
use crate::seats::{Departure, JoinRequest};

/// What one `join` came to.
pub(super) enum Joining {
    /// The connection has its seat, and the `perceived` backfill it is owed.
    Seated(Box<Seated>, Nickname, Vec<ServerFrame>),
    /// Refused; the connection stays in the handshake and may join again.
    Refused(Refusal),
    /// Refused, and the connection ends: a mismatched protocol, or a wrong invite.
    Closed(Refusal, ClosingReason),
}

/// A join's credentials and options, before any of them is trusted.
pub(super) struct Offered {
    pub(super) protocol: u32,
    pub(super) invite: OfferedInvite,
    pub(super) nickname: String,
    pub(super) resume: Option<OfferedResume>,
    pub(super) take_over: bool,
    pub(super) perceived: Option<PerceivedJoin>,
}

/// `PROTOCOL.md` §4.1's checks, in its order. The first that fails decides the answer.
pub(super) async fn join(
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
    match read_backfill(seated.perceived(), seated.observer()).await {
        Ok(frames) => Joining::Seated(Box::new(seated), nickname, frames),
        Err(refusal) => {
            connection
                .host
                .leave(seated.subscription(), Departure::Left);
            Joining::Refused(refusal)
        }
    }
}
