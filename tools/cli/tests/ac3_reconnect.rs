//! `AC-3` — a player whose connection drops gets the same Person back (step-12 SB-3, SB-9; CP-B2).
//!
//! ```text
//! mineworld server worlds/market-town --town --save DIR --hold 10
//! a client plays `visitor`, and its socket dies without `leave`     the seat is held, undriven
//! the town goes on living: the persisted revision advances           nobody drives the visitor
//! the client comes back with its resume, inside the hold            took_over "held", same Person
//! a second drop, and the hold runs out                              the town's controller has it
//! ```
//!
//! The resume secret is a credential: it is in the holder's own welcome and nowhere else — not on
//! stdout, not on stderr, not in any byte of the save (I-5).

mod support;

use std::time::{Duration, Instant};

use mineworld_server::{RefusalCode, ServerFrame, TookOver};
use serde_json::json;
use support::{Client, INVITE, MARKET_PACK, SaveDir, Server, file_containing, join_frame};

/// The server's hold in this test, in wall seconds.
const HOLD: u64 = 10;

/// What a welcome said about control, and the secret it carried.
fn welcomed(frame: &ServerFrame) -> (mineworld_contracts::EntityId, TookOver, String) {
    match frame {
        ServerFrame::Welcome {
            observer,
            took_over,
            resume,
            ..
        } => (
            *observer,
            *took_over,
            resume
                .as_ref()
                .expect("a resume secret")
                .reveal()
                .to_owned(),
        ),
        other => panic!("a welcome was expected, and the server said {other:?}"),
    }
}

fn resuming(seat: &str, secret: &str) -> serde_json::Value {
    let mut join = join_frame(seat);
    join["resume"] = json!(secret);
    join
}

#[tokio::test]
async fn a_dropped_player_resumes_the_same_person_inside_the_hold_and_not_after() {
    let save = SaveDir::new("ac3-reconnect");
    let hold = HOLD.to_string();
    let (mut server, output) = Server::start_captured(
        &[
            "server",
            MARKET_PACK,
            "--town",
            "--hold",
            &hold,
            "--invite",
            INVITE,
            "--save",
            save.path(),
        ],
        None,
    )
    .await;
    let mut secrets = Vec::new();

    // ── Playing the visitor. ─────────────────────────────────────────────────────────────────
    let mut player = Client::connect(server.address).await;
    let (visitor, took_over, secret) = welcomed(&player.join_as(join_frame("visitor")).await);
    assert_eq!(
        took_over,
        TookOver::Hosted,
        "the town was driving the visitor"
    );
    secrets.push(secret.clone());
    assert_eq!(
        server.status().await["clients"],
        json!(1),
        "one connection; the town's controllers are not clients (SB-9)"
    );
    let before = player.observation().await;
    let standing = *before
        .self_location()
        .expect("the visitor knows where it is");

    // ── The drop: no `leave`, the socket just ends. ──────────────────────────────────────────
    player.disconnect().await;
    let dropped_at = server.status().await["revision"]
        .as_u64()
        .expect("persisted");
    // The town lives while nobody plays the visitor.
    let deadline = Instant::now() + Duration::from_secs(HOLD - 3);
    let mut during = dropped_at;
    while Instant::now() < deadline && during == dropped_at {
        tokio::time::sleep(Duration::from_millis(200)).await;
        during = server.status().await["revision"]
            .as_u64()
            .expect("persisted");
    }
    assert!(
        during > dropped_at,
        "the town acted during the hold: revision {dropped_at} → {during}"
    );

    // ── Back inside the hold, with the resume. ───────────────────────────────────────────────
    let mut player = Client::connect(server.address).await;
    let (again, took_over, secret) = welcomed(&player.join_as(resuming("visitor", &secret)).await);
    secrets.push(secret.clone());
    assert_eq!(
        took_over,
        TookOver::Held,
        "the seat was held for this player"
    );
    assert_eq!(again, visitor, "the same Person");
    let after = player.observation().await;
    assert!(
        after.at() > before.at(),
        "the world went on: {} then {}",
        before.at(),
        after.at()
    );
    assert_eq!(
        after.self_location(),
        Some(&standing),
        "nobody drove the visitor while it was held — not even the town"
    );

    // ── A second drop, and the hold runs out. ────────────────────────────────────────────────
    player.disconnect().await;
    tokio::time::sleep(Duration::from_secs(HOLD + 1)).await;
    let mut late = Client::connect(server.address).await;
    late.send(resuming("visitor", &secret)).await;
    assert!(
        matches!(
            late.answer().await,
            ServerFrame::Refused {
                code: RefusalCode::InvalidResume,
                ..
            }
        ),
        "a resume presented after the hold matches nothing"
    );
    let (_, took_over, secret) = welcomed(&late.join_as(join_frame("visitor")).await);
    secrets.push(secret);
    assert_eq!(
        took_over,
        TookOver::Hosted,
        "the hold ended and the seat went back to the town's controller"
    );

    // ── SB-9: the secrets are nowhere but in their own welcomes. ─────────────────────────────
    server.kill();
    tokio::time::sleep(Duration::from_millis(200)).await;
    for secret in &secrets {
        for line in output.stdout().iter().chain(&output.stderr()) {
            assert!(
                !line.contains(secret.as_str()),
                "a resume secret was printed: {line}"
            );
        }
        assert_eq!(
            file_containing(std::path::Path::new(save.path()), secret.as_bytes()),
            None,
            "a resume secret is in the save"
        );
    }
}
