//! step-12 SB-6 (CP-B4) — **the hosted town lives**, on the world thread, within its tick budget.
//!
//! ```text
//! mineworld server worlds/market-town --town --pace 5 --save DIR
//! two clients play two seats; every other seat is driven by the paced rule controller in-server
//! 120 wall seconds, then SIGINT: the operator's shutdown lines are the probe (step-12 SD-B11)
//! ```
//!
//! What is asserted is what an operator reads: every seat the town drives made at least three
//! accepted `move`s, the world reported no fault, and the longest tick — consults, the journal's
//! fsync per request, and the sweep — took at most 50 ms, half the 100 ms cadence (`ARC-42`, I-11).

mod support;

use std::time::{Duration, Instant};

use mineworld_server::ServerFrame;
use serde_json::json;
use support::{Client, INVITE, MARKET_PACK, SaveDir, Server};

/// How long the town runs: 120 wall seconds, 24 consults of each seat at a pace of 5.
const RUN: Duration = Duration::from_secs(120);
/// Half the observation cadence (`HostConfig::default`'s 100 ms).
const TICK_BUDGET_MS: u64 = 50;

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
    println!("{ticks}");
    let longest: u64 = ticks
        .rsplit_once("longest tick ")
        .and_then(|(_, rest)| rest.strip_suffix(" ms"))
        .and_then(|number| number.parse().ok())
        .unwrap_or_else(|| panic!("a longest tick in {ticks:?}"));
    assert!(
        longest <= TICK_BUDGET_MS,
        "the longest tick took {longest} ms, over {TICK_BUDGET_MS} ms"
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
            .split_once("move ")
            .and_then(|(_, rest)| {
                rest.split(|c: char| !c.is_ascii_digit())
                    .next()
                    .and_then(|number| number.parse().ok())
            })
            .unwrap_or(0);
        assert!(moves >= 3, "{seat} made {moves} accepted move(s): {line}");
        driven += 1;
    }
    assert_eq!(
        driven,
        roster.len() - 2,
        "every seat nobody plays was driven"
    );
}
