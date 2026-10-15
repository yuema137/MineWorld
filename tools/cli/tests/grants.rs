//! `mineworld server --grant`, run for real (S27-a, `docs/DECISIONS.md` `ARC-97`; design AA-1, AA-7's CLI
//! half, AA-8).
//!
//! ```text
//! a malformed --grant      refuses to start, exit status ≠ 0, the message names the value (not secret)
//! a grant line             one per grant, in the invite line's grammar, in roster order of first seat
//! the grant admits         its seat; another seat is refused not_granted and the connection may join
//! the token goes nowhere   stdout (but its line), stderr, the save, /status, /admin/sessions
//! /admin/sessions          credential { kind: grant, seats, take_over } or { kind: invite }
//! ```
//!
//! The real binary and real sockets (`support`).

mod support;

use std::time::Duration;

use mineworld_server::{RefusalCode, ServerFrame};
use serde_json::{Value, json};
use support::{Client, PACK, SaveDir, Server, admin, file_containing, join_frame, stride};

/// The admin token these servers are started with.
const TOKEN: &str = "cli-grant-admin-token-5e2a";

/// The prefix of every grant line (`tools/cli/src/invite.rs`).
const GRANT_LINE: &str = "[mineworld] grant ";

/// A join presenting `credential` in the `invite` field, as a grant is presented.
fn join_with(seat: &str, credential: &str, nickname: &str) -> Value {
    let mut join = join_frame(seat);
    join["invite"] = json!(credential);
    join["nickname"] = json!(nickname);
    join
}

/// The token on a grant line: the word after the seat list (and `take-over`, when present).
fn token_of(line: &str) -> String {
    let words: Vec<&str> = line[GRANT_LINE.len()..].split_whitespace().collect();
    let token = if words.get(1) == Some(&"take-over") {
        words[2]
    } else {
        words[1]
    };
    token.to_owned()
}

/// Runs `mineworld server <arguments>` expecting it to refuse to start; returns everything it said.
fn refused_to_start(arguments: &[&str]) -> String {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_mineworld"))
        .arg("server")
        .arg(PACK)
        .args(arguments)
        .args(["--listen", "127.0.0.1:0"])
        .env_remove("MINEWORLD_INVITE")
        .env_remove("MINEWORLD_ADMIN_TOKEN")
        .output()
        .expect("the binary runs");
    let said = String::from_utf8_lossy(&output.stderr).into_owned()
        + &String::from_utf8_lossy(&output.stdout);
    assert!(
        !output.status.success(),
        "{arguments:?} should refuse to start, and it said: {said}"
    );
    said
}

#[test]
fn a_malformed_grant_refuses_to_start_and_names_the_value() {
    for value in [
        "visitor:watch",
        "visitor:take-over:take-over",
        "visitor++wanderer",
        "+visitor",
        "visitor+visitor",
        "nobody",
        "visitor+nobody:take-over",
    ] {
        let said = refused_to_start(&["--grant", value]);
        assert!(
            said.contains(value),
            "the refusal of --grant {value} names it: {said}"
        );
        assert!(
            !said.contains(GRANT_LINE),
            "a refused grant was issued anyway: {said}"
        );
    }
}

#[tokio::test]
async fn a_grant_is_printed_once_admits_its_seats_only_and_its_token_goes_nowhere_else() {
    const NICKNAME: &str = "Quillon-grant";
    let save = SaveDir::new("grant-secrets");
    let (mut server, output) = Server::start_with_env(
        &[
            "server",
            PACK,
            "--save",
            save.path(),
            "--admin-token",
            TOKEN,
            "--grant",
            "visitor",
            "--grant",
            "bob+alice:take-over",
        ],
        &[],
    )
    .await;

    // Both lines, in the invite's grammar, in the roster's (key) order of each grant's first seat —
    // bob before visitor, whatever order the flags were given in; a grant's seats as typed.
    output.line_starting("[mineworld] admin surface: ").await;
    let lines: Vec<String> = output
        .stdout()
        .into_iter()
        .filter(|line| line.starts_with(GRANT_LINE))
        .collect();
    assert_eq!(lines.len(), 2, "one line per grant: {lines:?}");
    let (pair, visitor) = (token_of(&lines[0]), token_of(&lines[1]));
    let address = server.address;
    assert_eq!(
        lines[1],
        format!(
            "[mineworld] grant visitor {visitor} — join with: {address} seat=visitor invite={visitor}"
        )
    );
    assert_eq!(
        lines[0],
        format!(
            "[mineworld] grant bob+alice take-over {pair} — join with: {address} seat=bob \
             invite={pair}"
        )
    );
    for token in [&visitor, &pair] {
        assert_eq!(token.len(), 32, "a generated grant is 128 bits in hex");
        assert!(token.bytes().all(|b| b.is_ascii_hexdigit()));
    }
    assert_ne!(visitor, pair, "every grant has its own token");

    // AA-1: the grant admits its seat; another seat is refused, and the connection stays.
    let mut granted = Client::connect(address).await;
    match granted
        .join_as(join_with("alice", &visitor, NICKNAME))
        .await
    {
        ServerFrame::Refused { code, .. } => assert_eq!(code, RefusalCode::NotGranted),
        other => panic!("a seat the grant does not name is not_granted: {other:?}"),
    }
    let welcome = granted
        .join_as(join_with("visitor", &visitor, NICKNAME))
        .await;
    assert!(
        matches!(welcome, ServerFrame::Welcome { .. }),
        "the same connection joins the seat the grant names: {welcome:?}"
    );
    // And acts: one stride where it stands.
    let me = granted.observer.expect("an observer");
    let here = granted.observation().await;
    let location = here.self_location().expect("a location");
    let local = location.local().expect("a position");
    granted
        .submit_accepted(stride(
            me,
            location.place().entity_id(),
            local.x().value(),
            local.y().value(),
        ))
        .await;

    // An invite session beside it, to compare what the admin surface says of each.
    let mut invited = Client::connect(address).await;
    invited.join("wanderer").await;

    let sessions = admin(address, "GET", "/admin/sessions", Some(TOKEN), "")
        .await
        .body;
    let credential_of = |seat: &str| -> Value {
        sessions["sessions"]
            .as_array()
            .expect("a list")
            .iter()
            .find(|session| session["seat"] == seat)
            .unwrap_or_else(|| panic!("a session on {seat}: {sessions}"))["credential"]
            .clone()
    };
    assert_eq!(
        credential_of("visitor"),
        json!({ "kind": "grant", "seats": ["visitor"], "take_over": false })
    );
    assert_eq!(credential_of("wanderer"), json!({ "kind": "invite" }));
    let status = server.status().await.to_string();

    server.kill();
    tokio::time::sleep(Duration::from_millis(200)).await;
    let (stdout, stderr) = (output.stdout(), output.stderr());
    for token in [&visitor, &pair] {
        assert_eq!(
            stdout
                .iter()
                .filter(|line| line.contains(token.as_str()))
                .count(),
            1,
            "a grant's token is on its own line and no other: {stdout:?}"
        );
        assert!(
            stderr.iter().all(|line| !line.contains(token.as_str())),
            "a grant's token reached stderr: {stderr:?}"
        );
        assert!(
            !status.contains(token.as_str()),
            "/status holds a grant token"
        );
        assert!(
            !sessions.to_string().contains(token.as_str()),
            "/admin/sessions holds a grant token"
        );
        assert_eq!(
            file_containing(std::path::Path::new(save.path()), token.as_bytes()),
            None,
            "a grant token is in the save"
        );
    }
    for line in stdout.iter().chain(&stderr) {
        assert!(!line.contains(NICKNAME), "a nickname was printed: {line}");
    }
}

// ---------------------------------------------------------------------------------------------
// --request-budget (S27-a C5; `PROTOCOL.md` §8).
// ---------------------------------------------------------------------------------------------

/// The startup summary line, which states the budget.
const SUMMARY: &str = "[mineworld] hold ";

#[test]
fn a_malformed_request_budget_refuses_to_start_and_names_the_value() {
    for value in [
        "10", "0/0", "0/20", "10/0", "10/5", "a/b", "-1/2", "+1/2", "10/20/30", "",
    ] {
        // One argument, `--request-budget=VALUE`, so that `-1/2` is a value and not a flag.
        let said = refused_to_start(&[&format!("--request-budget={value}")]);
        assert!(
            said.contains("--request-budget") && said.contains(&format!("'{value}'")),
            "the refusal of --request-budget {value:?} names it: {said}"
        );
    }
}

#[tokio::test]
async fn the_request_budget_is_announced_at_startup() {
    for (arguments, expected) in [
        (&[][..], "request budget 10/s burst 20 per seat"),
        (
            &["--request-budget", "3/6"][..],
            "request budget 3/s burst 6 per seat",
        ),
        (&["--request-budget", "0"][..], "no request budget"),
    ] {
        let server_arguments: Vec<&str> =
            ["server", PACK].iter().chain(arguments).copied().collect();
        let (mut server, output) = Server::start_with_env(&server_arguments, &[]).await;
        let line = output.line_starting(SUMMARY).await;
        assert!(line.contains(expected), "{arguments:?}: {line}");
        server.kill();
    }
}

/// AA-4 against the real binary and a save: 50 `move`s of 0 mm sent at once. Those beyond the bucket
/// are refused `rate_limited` with `retry_after_ms ≥ 1`, and the save's journal holds exactly the
/// answered ones (read with `mineworld inspect`, A-a-v3).
#[tokio::test]
async fn a_flood_is_refused_and_the_save_journals_only_the_answered_requests() {
    let save = SaveDir::new("request-budget");
    let mut server = Server::start(&["server", PACK, "--save", save.path()]).await;
    let mut client = Client::connect(server.address).await;
    let (me, _) = client.join("visitor").await;
    let here = client.observation().await;
    let location = here.self_location().expect("a location");
    let local = location.local().expect("a position");
    let step = stride(
        me,
        location.place().entity_id(),
        local.x().value(),
        local.y().value(),
    );
    for n in 0..50 {
        client
            .send(json!({ "t": "submit", "token": format!("f{n}"), "request": step }))
            .await;
    }
    let (mut answered, mut limited) = (0_u64, 0_u64);
    while answered + limited < 50 {
        match client.answer().await {
            ServerFrame::Result { .. } => answered += 1,
            ServerFrame::Refused {
                code: RefusalCode::RateLimited,
                retry_after_ms,
                ..
            } => {
                assert!(
                    retry_after_ms.is_some_and(|ms| ms >= 1),
                    "{retry_after_ms:?}"
                );
                limited += 1;
            }
            ServerFrame::Refused { code, .. } => panic!("refused {code:?}, not rate_limited"),
            _ => {}
        }
    }
    assert!(
        (20..50).contains(&answered),
        "the burst of 20, and at most what refilled, are answered: {answered} of 50"
    );
    server.kill();
    tokio::time::sleep(Duration::from_millis(200)).await;

    let (status, inspected) = support::run_command(&["inspect", save.path()]);
    assert!(status.success(), "inspect reads the save: {inspected}");
    let journaled: u64 = inspected
        .lines()
        .filter_map(|line| line.strip_prefix("journal    request move "))
        .map(|rest| {
            rest.rsplit(' ')
                .next()
                .and_then(|count| count.parse::<u64>().ok())
                .expect("a count")
        })
        .sum();
    assert_eq!(
        journaled, answered,
        "the journal holds the answered requests and none refused: {inspected}"
    );
}
