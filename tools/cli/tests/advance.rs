//! The single-player advance through the binary (`ARC-374`, pr-nightlife N2-C2): the sleep pack's stop adapter
//! reaches the server by the composition every host uses — `tools/cli` names no pack — and an advance's
//! identity is durable across a kill and a restart.
//!
//! Market Town without `--town`, so every seat is free and Carol stays in the apartments, where she may sleep.
//! In this build an admitted advance is recorded and ends at once `stopped { not_yet_executable }`.

mod support;

use mineworld_contracts::EntityId;
use serde_json::{Value, json};
use support::{Client, HttpAnswer, MARKET_PACK, SaveDir, Server, admin};

const TOKEN: &str = "advance-admin-token-7e21";
const OPERATION: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f0";

fn sleep_for(actor: EntityId, minutes: u32) -> Value {
    json!({
        "actor": actor,
        "action_type": "sleep",
        "target": null,
        "payload": { "action_type": "sleep", "payload": { "for": minutes } },
        "actor_location": null,
    })
}

async fn advance(server: &Server, operation: &str, until: &str) -> HttpAnswer {
    let body = format!(r#"{{"operation":"{operation}","seat":"carol","until":"{until}"}}"#);
    admin(server.address, "POST", "/admin/advance", Some(TOKEN), &body).await
}

fn arguments(save: &SaveDir, solo: bool) -> Vec<&str> {
    let mut arguments = vec![
        "server",
        MARKET_PACK,
        "--save",
        save.path(),
        "--admin-token",
        TOKEN,
    ];
    if solo {
        arguments.push("--solo");
    }
    arguments
}

/// The real adapter is found by name (`not_asleep`, not `unknown_adapter`, before Carol sleeps); a name the
/// build lacks is `unknown_adapter`; asleep, the advance is admitted, recorded and ended without moving time;
/// a repeat answers the record, another payload under the same id is refused; and after a kill the status by
/// id is the same.
#[tokio::test]
async fn an_advance_finds_the_sleep_adapter_and_its_record_survives_a_kill() {
    let save = SaveDir::new("advance-solo");
    let mut server = Server::start(&arguments(&save, true)).await;
    let mut carol = Client::connect(server.address).await;
    let (me, _) = carol.join("carol").await;

    let unknown = advance(&server, OPERATION, "nap").await;
    assert_eq!(
        (unknown.status, unknown.body["error"].clone()),
        (409, json!("unknown_adapter"))
    );
    let awake = advance(&server, OPERATION, "sleep").await;
    assert_eq!(
        (awake.status, awake.body["error"].clone()),
        (409, json!("not_asleep")),
        "the sleep pack's adapter answered: {:?}",
        awake.body
    );

    carol.submit_accepted(sleep_for(me, 60)).await;
    let before = server.status().await;
    let admitted = advance(&server, OPERATION, "sleep").await;
    assert_eq!(admitted.status, 200, "{:?}", admitted.body);
    let expected = json!({
        "operation": OPERATION,
        "seat": "carol",
        "until": "sleep",
        "state": "ended",
        "outcome": "stopped",
        "reason": "not_yet_executable",
    });
    assert_eq!(admitted.body, expected);
    let after = server.status().await;
    assert_eq!(
        after["revision"], before["revision"],
        "no time moved, nothing was submitted"
    );

    let repeated = advance(&server, OPERATION, "sleep").await;
    assert_eq!((repeated.status, repeated.body), (200, expected.clone()));
    let conflict = advance(&server, OPERATION, "nap").await;
    assert_eq!(conflict.body["error"], json!("operation_conflict"));

    server.kill();
    drop(carol);
    drop(server);
    let restarted = Server::start(&arguments(&save, true)).await;
    let status = admin(
        restarted.address,
        "GET",
        &format!("/admin/advance/{OPERATION}"),
        Some(TOKEN),
        "",
    )
    .await;
    assert_eq!((status.status, status.body), (200, expected));
}

/// A shared world never advances: the same sleeper on a host started without `--solo` is refused `not_solo`
/// and nothing is recorded.
#[tokio::test]
async fn a_shared_world_never_advances() {
    let save = SaveDir::new("advance-shared");
    let server = Server::start(&arguments(&save, false)).await;
    let mut carol = Client::connect(server.address).await;
    let (me, _) = carol.join("carol").await;
    carol.submit_accepted(sleep_for(me, 60)).await;
    let refused = advance(&server, OPERATION, "sleep").await;
    assert_eq!(
        (refused.status, refused.body["error"].clone()),
        (409, json!("not_solo"))
    );
    let status = admin(
        server.address,
        "GET",
        &format!("/admin/advance/{OPERATION}"),
        Some(TOKEN),
        "",
    )
    .await;
    assert_eq!(status.status, 404, "nothing was recorded");
}
