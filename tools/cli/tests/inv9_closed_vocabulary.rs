//! `INV-9` over the whole of revision 2 — **a client requests; it never asserts** (step-12 §9.5 CP-E3,
//! §20.3 SE-2; `docs/NETWORKING.md` §2).
//!
//! ```text
//! mineworld server worlds/social-cafe --save DIR --admin-token T     (the first hour: nothing is due)
//! unknown_frame         set_state, move_to, give, and the host's own words — clock, pause, kick,
//!                       release — which are HTTP routes, never frames
//! actor_not_observer    a submit as another Person
//! malformed_frame       a join naming its observer; a leave naming a seat
//! let go                a leave before any join; the seat it might have meant is still its player's
//! a fresh start         a join after leave, on a new connection: the same Person, taken from nobody
//! 401                   every admin route without the token, and with a wrong one, no sooner than
//!                       500 ms after the request
//! nothing moved         the revision /status reports, and — after the process dies — the save holds
//!                       the genesis facts `validate` counts and nothing else
//! ```
//!
//! The revision-1 half of the table is S11-A's (`server_command.rs` SA-4) and the admin routes *with*
//! the token are S11-D's (`admin.rs` DA-3); this is the closed vocabulary end to end, on one persisted
//! world, through the real binary.

mod fixture;
mod support;

use std::time::{Duration, Instant};

use mineworld_server::{ClosingReason, RefusalCode, ServerFrame, TookOver};
use serde_json::{Value, json};
use support::{Client, PACK, SaveDir, Server, admin, join_frame, run_command, talk};

/// The admin token this server is started with.
const TOKEN: &str = "inv9-admin-token-5d21";

/// `PROTOCOL.md` §11's unauthorized answer comes no sooner than this after the request.
const UNAUTHORIZED_DELAY: Duration = Duration::from_millis(500);

/// Every admin route, with the body a `POST` carries.
const ROUTES: [(&str, &str, &str); 6] = [
    ("GET", "/admin/sessions", ""),
    ("GET", "/admin/seats", ""),
    ("POST", "/admin/sessions/1/kick", ""),
    ("POST", "/admin/seats/visitor/release", ""),
    ("GET", "/admin/clock", ""),
    ("POST", "/admin/clock", r#"{"paused": true}"#),
];

/// The server's answer to one frame: the refusal's code, or what it said instead.
async fn refused(client: &mut Client, frame: &Value) -> RefusalCode {
    client.send(frame.clone()).await;
    match client.answer().await {
        ServerFrame::Refused { code, .. } => code,
        other => panic!("{frame} must be refused, and the server said {other:?}"),
    }
}

async fn closed_because(client: &mut Client, expected: ClosingReason) {
    match client.answer().await {
        ServerFrame::Closing { reason, .. } => assert_eq!(reason, expected),
        other => panic!("a closing was expected, and the server said {other:?}"),
    }
}

/// The state of one seat on `GET /admin/seats`, asked with the token.
async fn seat_state(server: &Server, seat: &str) -> Value {
    let seats = admin(server.address, "GET", "/admin/seats", Some(TOKEN), "")
        .await
        .body;
    seats["seats"]
        .as_array()
        .expect("a list")
        .iter()
        .find(|entry| entry["seat"] == json!(seat))
        .map(|entry| entry["state"].clone())
        .unwrap_or_else(|| panic!("{seat} is not listed: {seats}"))
}

#[tokio::test(flavor = "multi_thread")]
async fn the_client_vocabulary_is_closed_and_nothing_it_asserts_moves_the_world() {
    // Nobody acts and no routine boundary falls in the first hour after genesis, so only something
    // this test sends could move the revision.
    fixture::assert_quiet(0, 3_600, "inv9_closed_vocabulary.rs");
    let save = SaveDir::new("inv9-closed-vocabulary");
    let mut server = Server::start(&[
        "server",
        PACK,
        "--save",
        save.path(),
        "--admin-token",
        TOKEN,
    ])
    .await;
    let mut player = Client::connect(server.address).await;
    let (visitor, _) = player.join("visitor").await;
    let mut other = Client::connect(server.address).await;
    let (wanderer, _) = other.join("wanderer").await;
    let before = server.status().await["revision"].clone();
    assert_eq!(before, json!(1), "a new world, at genesis");

    // ── Assertions of state, and the host's words, are no frame at all. ──────────────────────
    for asserted in [
        json!({ "t": "set_state", "entity": visitor, "money": 5000 }),
        json!({ "t": "move_to", "position": { "x": 1000, "y": 2000, "z": 0 } }),
        json!({ "t": "give", "item": "1", "to": visitor }),
        json!({ "t": "clock", "paused": true }),
        json!({ "t": "pause" }),
        json!({ "t": "kick", "session": "1" }),
        json!({ "t": "release", "seat": "wanderer" }),
    ] {
        assert_eq!(
            refused(&mut player, &asserted).await,
            RefusalCode::UnknownFrame,
            "{asserted}"
        );
    }
    assert_eq!(
        player
            .submit_refused(talk(wanderer, visitor, "I am the wanderer"))
            .await,
        RefusalCode::ActorNotObserver,
        "a request as another Person"
    );

    // ── A known frame carrying an assertion is malformed, and changes nothing. ───────────────
    assert_eq!(
        refused(&mut player, &json!({ "t": "leave", "seat": "wanderer" })).await,
        RefusalCode::MalformedFrame,
        "a leave naming somebody else's seat"
    );
    assert_eq!(
        seat_state(&server, "wanderer").await,
        json!("connected"),
        "and the wanderer's seat is still its player's"
    );
    let mut naming_itself = Client::connect(server.address).await;
    let mut join = join_frame("carol");
    join["observer"] = json!(visitor);
    assert_eq!(
        refused(&mut naming_itself, &join).await,
        RefusalCode::MalformedFrame,
        "a join that names an observer"
    );

    // ── leave before a join, and a join after leave. ─────────────────────────────────────────
    let mut stranger = Client::connect(server.address).await;
    stranger.send(json!({ "t": "leave" })).await;
    closed_because(&mut stranger, ClosingReason::Left).await;
    for seat in ["visitor", "wanderer"] {
        assert_eq!(
            seat_state(&server, seat).await,
            json!("connected"),
            "a leave from a connection with no seat released nobody's: {seat}"
        );
    }
    player.send(json!({ "t": "leave" })).await;
    closed_because(&mut player, ClosingReason::Left).await;
    let mut again = Client::connect(server.address).await;
    match again.join_as(join_frame("visitor")).await {
        ServerFrame::Welcome {
            observer,
            took_over,
            ..
        } => {
            assert_eq!(observer, visitor, "the same Person");
            assert_eq!(
                took_over,
                TookOver::None,
                "taken from nobody: the seat was let go"
            );
        }
        other => panic!("a join after leave is welcomed, and the server said {other:?}"),
    }

    // ── The admin surface without the token. ─────────────────────────────────────────────────
    for token in [None, Some("inv9-admin-token-5d22")] {
        for (method, path, body) in ROUTES {
            let asked = Instant::now();
            let answer = admin(server.address, method, path, token, body).await;
            let waited = asked.elapsed();
            assert_eq!(
                answer.status, 401,
                "{method} {path} with {token:?}: {:?}",
                answer.body
            );
            assert!(
                waited >= UNAUTHORIZED_DELAY,
                "{method} {path} with {token:?} answered after {waited:?}"
            );
        }
    }
    let status = server.status().await;
    assert_eq!(
        status["paused"],
        json!(false),
        "no unauthorized pause: {status}"
    );
    assert_eq!(status["clients"], json!(2), "nobody was kicked: {status}");

    // ── Nothing moved. ───────────────────────────────────────────────────────────────────────
    assert_eq!(
        status["revision"], before,
        "nothing was committed: {status}"
    );
    drop((again, other, naming_itself, stranger));
    server.kill();
    let (validated, said) = run_command(&["validate", PACK]);
    assert!(validated.success(), "{said}");
    let genesis = said
        .lines()
        .find_map(|line| {
            line.split_once(" genesis fact(s)")
                .map(|(count, _)| count.trim().to_owned())
        })
        .expect("validate counts the genesis facts");
    let (inspected, report) = run_command(&["inspect", save.path()]);
    assert!(inspected.success(), "{report}");
    assert!(
        report
            .lines()
            .any(|line| line == format!("facts      {genesis}")),
        "the save holds the {genesis} genesis facts and nothing else: {report}"
    );
}
