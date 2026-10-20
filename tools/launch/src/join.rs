//! The one thing the launcher reads from the server: its join line (`server/PROTOCOL.md` §4.1).
//!
//! ```text
//! [mineworld] invite <token> — join with: <address> seat=<seat> invite=<token>
//! ```

/// Where a client joins, and the invite it presents. The invite is never written to the launcher's log.
#[derive(Debug, PartialEq, Eq)]
pub struct Join {
    pub address: String,
    pub invite: String,
}

const PREFIX: &str = "[mineworld] invite ";
const ADDRESS: &str = " join with: ";

/// The join line among `printed`, if the server has printed it. Only complete lines are read: a log read
/// while the server is writing may end in half a line, and half an address is not an address.
pub fn find(printed: &str) -> Option<Join> {
    let complete = printed.rfind('\n').map_or("", |end| &printed[..end]);
    complete.lines().find_map(parse)
}

/// One line read as a join line: the token after `invite ` up to the next space, and the address after
/// ` join with: ` up to the next space. Anything else is not one.
pub fn parse(line: &str) -> Option<Join> {
    let rest = line.strip_prefix(PREFIX)?;
    let invite = rest.split(' ').next().filter(|token| !token.is_empty())?;
    let (_, after) = rest.split_once(ADDRESS)?;
    let address = after
        .split(' ')
        .next()
        .filter(|address| !address.is_empty())?;
    Some(Join {
        address: address.to_owned(),
        invite: invite.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_token_and_the_address_are_read_from_a_generated_invites_line() {
        let printed = "[mineworld] Social Café (social-cafe) — 7 system(s), 11 seat(s)\n\
            [mineworld] listening on http://127.0.0.1:50805 (ws://127.0.0.1:50805/ws), protocol 2\n\
            [mineworld] invite 0f3a9c2b7d1e4f60a8b9c0d1e2f3a4b5 — join with: 127.0.0.1:50805 \
            seat=alice invite=0f3a9c2b7d1e4f60a8b9c0d1e2f3a4b5\n";
        assert_eq!(
            find(printed),
            Some(Join {
                address: "127.0.0.1:50805".to_owned(),
                invite: "0f3a9c2b7d1e4f60a8b9c0d1e2f3a4b5".to_owned(),
            })
        );
    }

    #[test]
    fn a_supplied_invites_line_and_a_partial_line_are_not_join_lines() {
        // With --invite or MINEWORLD_INVITE the server prints no token; the launcher never supplies one.
        assert_eq!(
            find("[mineworld] join with: 127.0.0.1:1 seat=alice invite=<the invite you gave>\n"),
            None
        );
        // A line still being written when the log is read: no address yet, or half of one.
        assert_eq!(find("[mineworld] invite 0f3a9c2b — join wi"), None);
        assert_eq!(
            find("[mineworld] invite 0f3a9c2b — join with: 127.0.0.1:50"),
            None
        );
    }
}
