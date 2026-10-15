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
//!
//! A seat grant (`--grant SEATS[:take-over]`, `docs/DECISIONS.md` `ARC-97`) is a narrower credential
//! presented in the same `invite` field. Its token is never given — not by a flag, not by the
//! environment — only generated, and printed once in the same grammar:
//!
//! ```text
//! [mineworld] grant <seat>[+<seat>…][ take-over] <token> — join with: <address> seat=<seat> invite=<token>
//! ```

use std::net::SocketAddr;

use mineworld_contracts::EntityKey;
use mineworld_server::{InviteToken, SeatGrant};

use crate::listed;

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

/// One `--grant SEATS[:take-over]` as the operator typed it (`server/PROTOCOL.md` §4.1 "Seat grants",
/// `docs/DECISIONS.md` `ARC-97`): the seats in the order given, and whether it may take over a seat
/// another player holds. It holds no token — a grant's token is only ever generated, after the roster
/// is known — so the text itself is not secret and a refusal may repeat it.
#[derive(Debug, Clone)]
pub struct GrantFlag {
    given: String,
    seats: Vec<EntityKey>,
    take_over: bool,
}

/// The suffix that gives a grant the right to take a held or connected seat.
const TAKE_OVER: &str = ":take-over";

/// Parses one `--grant` value: `seat`, or `seat+seat+…`, optionally followed by `:take-over`. Any other
/// suffix, an empty seat, a seat that is not a legal key, or a seat named twice is refused; clap's
/// message names the value.
pub fn grant_flag(text: &str) -> Result<GrantFlag, String> {
    let (list, take_over) = match text.strip_suffix(TAKE_OVER) {
        Some(list) => (list, true),
        None => (text, false),
    };
    if let Some((_, suffix)) = list.split_once(':') {
        return Err(format!(
            "the only suffix a grant takes is ':take-over', and this one is ':{suffix}'"
        ));
    }
    let mut seats: Vec<EntityKey> = Vec::new();
    for name in list.split('+') {
        if name.is_empty() {
            return Err("a grant names seats joined by '+', and one of them is empty".to_owned());
        }
        let seat = EntityKey::new(name)
            .map_err(|error| format!("'{name}' is not a seat name: {error}"))?;
        if seats.contains(&seat) {
            return Err(format!("the grant names seat '{name}' twice"));
        }
        seats.push(seat);
    }
    Ok(GrantFlag {
        given: text.to_owned(),
        seats,
        take_over,
    })
}

/// A grant the server issued for one `--grant`, with the line that prints its token once.
#[derive(Debug)]
pub struct Issued {
    flag: GrantFlag,
    grant: SeatGrant,
}

/// The operator's grants checked against the world's roster, each given a freshly generated token, in
/// the roster order of each one's first seat (stable for grants that share one).
pub fn issue(flags: Vec<GrantFlag>, roster: &[EntityKey]) -> Result<Vec<Issued>, String> {
    let mut issued = Vec::with_capacity(flags.len());
    for flag in flags {
        if let Some(missing) = flag.seats.iter().find(|seat| !roster.contains(seat)) {
            return Err(format!(
                "[mineworld] --grant {}: this world offers no seat '{missing}'. It offers: {}",
                flag.given,
                listed(roster.iter()),
            ));
        }
        let grant = SeatGrant::generate(flag.seats.iter().cloned().collect(), flag.take_over)
            .map_err(|error| format!("[mineworld] --grant {}: {error}", flag.given))?;
        issued.push(Issued { flag, grant });
    }
    issued.sort_by_key(|issued| {
        roster
            .iter()
            .position(|seat| Some(seat) == issued.flag.seats.first())
    });
    Ok(issued)
}

impl Issued {
    /// The grant itself, for the server's admission.
    pub const fn grant(&self) -> &SeatGrant {
        &self.grant
    }

    /// The one line that prints this grant's token, in the invite line's grammar so a launcher reads
    /// it the same way: the token after `grant <seats>[ take-over] `, and a join line to paste.
    pub fn line(&self, address: SocketAddr) -> String {
        let seats = listed_with(&self.flag.seats, "+");
        let take_over = if self.flag.take_over {
            " take-over"
        } else {
            ""
        };
        let first = &self.flag.seats[0];
        let token = self.grant.token().reveal();
        format!(
            "[mineworld] grant {seats}{take_over} {token} — join with: {address} seat={first} \
             invite={token}"
        )
    }
}

fn listed_with(seats: &[EntityKey], separator: &str) -> String {
    seats
        .iter()
        .map(EntityKey::as_str)
        .collect::<Vec<_>>()
        .join(separator)
}

/// The seat a join line suggests: the first one no in-server controller drives, or the first one if
/// every seat is driven.
pub fn suggested_seat<'a>(seats: &'a [EntityKey], driven: &[EntityKey]) -> Option<&'a EntityKey> {
    seats
        .iter()
        .find(|seat| !driven.contains(seat))
        .or_else(|| seats.first())
}
