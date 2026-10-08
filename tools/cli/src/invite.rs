//! The server's invite: which one, and the one line that tells a player how to join.
//!
//! Every client presents the invite in its `join`, on loopback as on a LAN — there is no open mode
//! (`server/PROTOCOL.md` §4.1, step-12 QS11-6). The operator gives one with `--invite` or the
//! environment variable `MINEWORLD_INVITE` (clap resolves them, the flag first); when neither is
//! given, one is generated and printed once, as a join line a person can paste and a launcher can
//! parse:
//!
//! ```text
//! [mineworld] invite <token> — join with: <address> seat=<seat> invite=<token>
//! ```
//!
//! A launcher reads the token after `invite ` on the line that begins `[mineworld] invite`
//! (step-12 §11.4). An invite the operator gave is never printed — the operator has it, and a log
//! file should not — so that case prints a line beginning `[mineworld] join with:` instead, which no
//! launcher can mistake for a token line. Nothing else in this binary prints the invite.

use std::net::SocketAddr;

use mineworld_contracts::EntityKey;
use mineworld_server::InviteToken;

/// Where the invite came from, which decides whether it may be printed.
#[derive(Debug)]
pub enum Invite {
    /// Generated here; printed once on the join line.
    Generated(InviteToken),
    /// Given by the operator; never printed.
    Given(InviteToken),
}

impl Invite {
    /// The operator's invite if there is one, otherwise a fresh one.
    ///
    /// A given invite that is not a legal one stops the server with a message that does not repeat
    /// it: a near miss of a secret is still a secret.
    pub fn resolve(given: Option<String>) -> Result<Self, String> {
        match given {
            Some(text) => InviteToken::given(text).map(Self::Given).map_err(|error| {
                format!("[mineworld] --invite / MINEWORLD_INVITE is not usable: {error}")
            }),
            None => InviteToken::generate()
                .map(Self::Generated)
                .map_err(|error| format!("[mineworld] {error}")),
        }
    }

    /// The invite itself, for the server's admission.
    pub fn token(&self) -> &InviteToken {
        match self {
            Self::Generated(token) | Self::Given(token) => token,
        }
    }

    /// The line a person pastes to join, naming one seat a player can take.
    pub fn join_line(&self, address: SocketAddr, seat: &EntityKey) -> String {
        match self {
            Self::Generated(token) => {
                let token = token.reveal();
                format!(
                    "[mineworld] invite {token} — join with: {address} seat={seat} invite={token}"
                )
            }
            Self::Given(_) => {
                format!("[mineworld] join with: {address} seat={seat} invite=<the invite you gave>")
            }
        }
    }
}

/// The seat a join line suggests: the first one no in-server controller drives, or the first one if
/// every seat is driven.
pub fn suggested_seat<'a>(seats: &'a [EntityKey], driven: &[EntityKey]) -> Option<&'a EntityKey> {
    seats
        .iter()
        .find(|seat| !driven.contains(seat))
        .or_else(|| seats.first())
}
