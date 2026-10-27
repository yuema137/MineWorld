//! step-12 SB-6 (CP-B4) — **the hosted town lives**, on the world thread, within its tick budget.
//!
//! ```text
//! mineworld server worlds/market-town --town --cadence 5 --time-scale 12 --save DIR
//! two clients play two seats; every other seat is driven by the paced rule controller in-server
//! 120 wall seconds, then SIGINT: the operator's shutdown lines are the probe (step-12 SD-B11)
//! ```
//!
//! What is asserted is what an operator reads: every seat the town drives made at least three
//! accepted strides (since step-11 12n-2 a walk's strides are `walk-step`s: the paced controller
//! asks `walk-to` and then steps once a wall second), the world reported no fault, and the tick —
//! consults, the journal's fsync per request, and the sweep — fits half the 100 ms cadence (`ARC-42`,
//! I-11): its CPU p99 and its wall p50 at most 50 ms each. The wall p99 and maximum are printed beside
//! them and do not gate: a stall off-CPU the server cannot control must not decide the bound (operator
//! rulings on D-SB12, 2026-10-08, and on F-tick-1, 2026-10-15; step-12 E-SB9; `support/ticks.rs`).

mod support;

use std::time::{Duration, Instant};

use mineworld_server::ServerFrame;
use serde_json::json;
use support::ticks::TickStats;
use support::{Client, INVITE, MARKET_PACK, SaveDir, Server};

/// How long the town runs: 120 wall seconds, 24 consults of each seat at a cadence of 5 wall seconds.
const RUN: Duration = Duration::from_secs(120);

#[tokio::test]
async fn the_hosted_town_lives_within_its_tick_budget() {
    let save = SaveDir::new("hosted-town");
    let (mut server, output) = Server::start_captured(
        &[
            "server",
            MARKET_PACK,
            "--town",
            "--cadence",
            "5",
            // The scale measured, stated rather than taken from market-town's `hosting:` (TW-c C6).
            "--time-scale",
            "12",
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
    let ticks = TickStats::parse(&output.line_starting("[world] ticks ").await);
    // Printed whether or not it passes, so every run's numbers can be read (`--nocapture`).
    println!("CP-B4 tick {}", ticks.summary());
    ticks.assert_within_budget();

    let mut driven = 0;
    for seat in roster
        .iter()
        .filter(|seat| !["visitor", "wanderer"].contains(&seat.as_str()))
    {
        let line = output
            .line_starting(&format!("[world] hosted {seat}: "))
            .await;
        println!("{line}");
        // Driven means the host's controller acted for the seat and the world accepted it: strides by
        // day, or `sleep` at night since S35 N-a, so the count is every accepted request, not walk-steps.
        // The town is hosted from genesis (00:00) for 24 world minutes, so a seat that starts at home
        // goes to sleep at once and, rightly, asks nothing more until its alarm: since `ARC-330` R4 a
        // passer-by's push no longer wakes it. One accepted `sleep` is therefore driven too.
        let accepted: u64 = line
            .split_once(" consults, ")
            .and_then(|(_, rest)| rest.split_once(" accepted"))
            .and_then(|(number, _)| number.trim().parse().ok())
            .unwrap_or(0);
        let slept = line.contains(" accepted (sleep ");
        assert!(
            accepted >= 3 || slept,
            "{seat} had {accepted} accepted request(s) and did not go to sleep: {line}"
        );
        driven += 1;
    }
    assert_eq!(
        driven,
        roster.len() - 2,
        "every seat nobody plays was driven"
    );
}
