//! The single-player advance through the binary (`ARC-374`, pr-nightlife N2-C2): the sleep pack's stop adapter
//! reaches the server by the composition every host uses — `tools/cli` names no pack — and an advance's
//! identity is durable across a kill and a restart.
//!
//! Market Town without `--town`, so every seat is free and Carol stays in the apartments, where she may sleep.
//! Without hosted seats the night is the sleep's own Process: the advance runs to the alarm and ends `woke`.

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

/// Polls the advance's status until it has ended, reading the sleeper's frames meanwhile as a client does.
/// A client that stops reading holds the night back rather than being released (N2-I13), so this one reads.
async fn until_ended(server: &Server, sleeper: &mut Client) -> Value {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    loop {
        while tokio::time::timeout(std::time::Duration::from_millis(20), sleeper.text())
            .await
            .is_ok()
        {}
        let status = admin(
            server.address,
            "GET",
            &format!("/admin/advance/{OPERATION}"),
            Some(TOKEN),
            "",
        )
        .await;
        assert_eq!(status.status, 200, "{:?}", status.body);
        if status.body["state"] == json!("ended") {
            return status.body;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "still running: {:?}",
            status.body
        );
    }
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
/// build lacks is `unknown_adapter`; asleep, the advance runs the hour to the alarm in moments, `woke`;
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
    let started = std::time::Instant::now();
    let admitted = advance(&server, OPERATION, "sleep").await;
    assert_eq!(admitted.status, 200, "{:?}", admitted.body);
    let expected = json!({
        "operation": OPERATION,
        "seat": "carol",
        "until": "sleep",
        "state": "ended",
        "outcome": "woke",
    });
    assert_eq!(until_ended(&server, &mut carol).await, expected);
    assert!(
        started.elapsed() < std::time::Duration::from_secs(10),
        "an hour at 12x is five wall minutes; the advance took {:?}",
        started.elapsed()
    );
    let after = server.status().await;
    let slept = after["at"].as_i64().expect("at") - before["at"].as_i64().expect("at");
    assert!(
        (3_600..3_600 + 60).contains(&slept),
        "the clock runs on from the alarm, an hour later: {slept} s"
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

/// Seconds of world time at 23:00 on the first day of a world that begins at midnight's instant 0.
const ELEVEN_PM: i64 = 23 * 3_600;
/// 07:00 the next morning.
const NEXT_SEVEN: i64 = 24 * 3_600 + 7 * 3_600;

/// The residential place a passage from where the observer stands leads to, read from what it perceives.
fn residential_passage(observation: &Value) -> Option<EntityId> {
    let entities = observation["entities"].as_array()?;
    entities
        .iter()
        .flat_map(|entity| entity["components"].as_array().into_iter().flatten())
        .filter(|component| component["component_type"] == json!("passages"))
        .flat_map(|component| {
            component["payload"]["leads_to"]
                .as_array()
                .into_iter()
                .flatten()
        })
        .find(|passage| {
            passage["to_tags"]
                .as_array()
                .is_some_and(|tags| tags.contains(&json!("residential")))
        })
        .and_then(|passage| serde_json::from_value(passage["to"]["entity"].clone()).ok())
}

fn act(actor: EntityId, action: &str, payload: Value) -> Value {
    json!({
        "actor": actor,
        "action_type": action,
        "target": null,
        "payload": { "action_type": action, "payload": payload },
        "actor_location": null,
    })
}

/// N2-C3's acceptance (F14): in a `--solo` Market Town with every other Person hosted, Carol sleeps from about
/// 23:00 until 07:00 by the sleep pack's own offer, and the host runs the night — every resident's consult and
/// stride, every Process — in at most 60 wall seconds (forty wall minutes at 12×), ending `woke` no later than
/// the alarm, with the scale unchanged after. The visited-instant count is the server's own line.
#[tokio::test]
async fn a_market_town_night_runs_to_morning_within_a_minute() {
    let save = SaveDir::new("advance-night");
    let (server, captured) = Server::start_captured(
        &[
            "server",
            MARKET_PACK,
            "--town",
            "--solo",
            "--save",
            save.path(),
            "--admin-token",
            TOKEN,
            "--invite",
            support::INVITE,
            "--time-scale",
            "3600",
        ],
        None,
    )
    .await;
    // Fast-forward the day (a solo world may), then the ordinary 12x for the night.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while server.status().await["at"].as_i64().expect("at") < ELEVEN_PM {
        assert!(std::time::Instant::now() < deadline, "the day did not pass");
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let slowed = admin(
        server.address,
        "POST",
        "/admin/clock",
        Some(TOKEN),
        r#"{"time_scale": 12}"#,
    )
    .await;
    assert_eq!(slowed.status, 200, "{:?}", slowed.body);

    let mut carol = Client::connect(server.address).await;
    let (me, _) = carol.join("carol").await;
    // Awake, whatever her controller decided before she was taken over, then to bed by the offer.
    let _ = carol.submit(act(me, "wake", json!({}))).await;
    let seen = carol.observation().await;
    if let Some(home) = residential_passage(&serde_json::to_value(&seen).expect("json"))
        && seen
            .self_location()
            .map(|location| location.place().entity_id())
            != Some(home)
    {
        carol.walk_there(me, home, None).await;
    }
    let observation = serde_json::to_value(
        carol
            .observation_where("carol at home", |seen| {
                seen.affordances().iter().any(|offer| {
                    offer.action_type().as_str() == "sleep-until" && offer.is_available()
                })
            })
            .await,
    )
    .expect("json");
    let offer = observation["affordances"]
        .as_array()
        .and_then(|offers| {
            offers
                .iter()
                .find(|offer| offer["action_type"] == json!("sleep-until"))
        })
        .cloned()
        .expect("sleep-until is offered");
    let at = offer["payload"]["at"]
        .as_i64()
        .unwrap_or_else(|| panic!("carol is home and the offer is complete: {offer}"));
    assert_eq!(at, NEXT_SEVEN, "the world's morning");
    carol
        .submit_accepted(act(me, "sleep-until", json!({ "at": at })))
        .await;

    let before = server.status().await;
    let started = std::time::Instant::now();
    let admitted = advance(&server, OPERATION, "sleep").await;
    assert_eq!(admitted.status, 200, "{:?}", admitted.body);
    let ended = until_ended(&server, &mut carol).await;
    let took = started.elapsed();
    let after = server.status().await;
    let line = captured.line_starting("[world] advance").await;
    eprintln!(
        "night {} → {} in {took:?} wall: {line}",
        before["at"], after["at"]
    );
    assert_eq!(ended["outcome"], json!("woke"), "{ended}");
    let reached = after["at"].as_i64().expect("at");
    assert!(
        reached <= NEXT_SEVEN + 60,
        "never past the alarm: {reached}"
    );
    assert!(
        took <= std::time::Duration::from_secs(60),
        "a night at 12x is forty wall minutes; the advance must take at most one: {took:?} ({line})"
    );
    assert_eq!(after["time_scale"], json!(12), "no rescale");
    assert_eq!(after["paused"], json!(false));
    drop(carol);
}
