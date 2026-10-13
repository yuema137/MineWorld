//! step-12 SB-6 (CP-B4) — **the hosted town lives**, on the world thread, within its tick budget.
//!
//! ```text
//! mineworld server worlds/market-town --town --pace 5 --save DIR
//! two clients play two seats; every other seat is driven by the paced rule controller in-server
//! 120 wall seconds, then SIGINT: the operator's shutdown lines are the probe (step-12 SD-B11)
//! ```
//!
//! What is asserted is what an operator reads: every seat the town drives made at least three
//! accepted strides (since step-11 12n-2 a walk's strides are `walk-step`s: the paced controller
//! asks `walk-to` and then steps once a wall second), the world reported no fault, and the p99 tick — consults, the journal's
//! fsync per request, and the sweep — took at most 50 ms, half the 100 ms cadence (`ARC-42`, I-11).
//! The maximum is printed beside it and does not gate: one off-CPU stall the server cannot control
//! must not decide the bound (operator ruling on D-SB12, 2026-10-08; step-12 E-SB9).

mod support;

use std::time::{Duration, Instant};

use mineworld_server::ServerFrame;
use serde_json::json;
use support::{Client, INVITE, MARKET_PACK, SaveDir, Server};

/// How long the town runs: 120 wall seconds, 24 consults of each seat at a pace of 5.
const RUN: Duration = Duration::from_secs(120);
/// Half the observation cadence (`HostConfig::default`'s 100 ms), for the p99 tick.
const TICK_BUDGET_MS: f64 = 50.0;

#[tokio::test]
async fn the_hosted_town_lives_within_its_tick_budget() {
    let save = SaveDir::new("hosted-town");
    let (mut server, output) = Server::start_captured(
        &[
            "server",
            MARKET_PACK,
            "--town",
            "--pace",
            "5",
            "--invite",
            INVITE,
            "--save",
            save.path(),
        ],
        None,
    )
    .await;

    let mut players = Vec::new();
    for seat in ["visitor", "wanderer"] {
        let mut client = Client::connect(server.address).await;
        let welcome = client.join_as(support::join_frame(seat)).await;
        assert!(
            matches!(&welcome, ServerFrame::Welcome { .. }),
            "{seat}: {welcome:?}"
        );
        players.push(client);
    }
    let roster: Vec<String> = server.status().await["seats"]
        .as_array()
        .expect("the seats")
        .iter()
        .map(|seat| seat.as_str().expect("a key").to_owned())
        .collect();

    // Both players keep reading, as clients do, while the town runs.
    let ends = Instant::now() + RUN;
    while Instant::now() < ends {
        for player in &mut players {
            player.observation().await;
        }
    }
    let status = server.status().await;
    assert_eq!(status["faults"], json!(0), "{status}");
    assert_eq!(
        status["clients"],
        json!(2),
        "two connections; the town is not a client"
    );

    let ended = server.interrupt();
    assert!(ended.success(), "Ctrl-C is a clean stop: {ended:?}");
    let ticks = output.line_starting("[world] ticks ").await;
    let field = |name: &str| -> f64 {
        ticks
            .split(", ")
            .find_map(|part| part.strip_prefix(name))
            .and_then(|rest| rest.strip_suffix(" ms"))
            .and_then(|number| number.parse().ok())
            .unwrap_or_else(|| panic!("{name}… ms in {ticks:?}"))
    };
    let (p50, p99, longest) = (field("p50 "), field("p99 "), field("longest tick "));
    // Printed whether or not it passes, so every run's numbers can be read (`--nocapture`).
    println!("CP-B4 tick p50 {p50} ms, p99 {p99} ms, max {longest} ms ({ticks})");
    assert!(
        p99 <= TICK_BUDGET_MS,
        "the p99 tick took {p99} ms, over {TICK_BUDGET_MS} ms (p50 {p50} ms, max {longest} ms)"
    );

    let mut driven = 0;
    for seat in roster
        .iter()
        .filter(|seat| !["visitor", "wanderer"].contains(&seat.as_str()))
    {
        let line = output
            .line_starting(&format!("[world] hosted {seat}: "))
            .await;
        println!("{line}");
        let moves: u64 = line
            .split_once("walk-step ")
            .and_then(|(_, rest)| {
                rest.split(|c: char| !c.is_ascii_digit())
                    .next()
                    .and_then(|number| number.parse().ok())
            })
            .unwrap_or(0);
        assert!(moves >= 3, "{seat} made {moves} accepted stride(s): {line}");
        driven += 1;
    }
    assert_eq!(
        driven,
        roster.len() - 2,
        "every seat nobody plays was driven"
    );
}
