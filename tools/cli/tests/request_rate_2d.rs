//! S27-a C0 — **how often the real 2D client submits while a person walks** (step-27 §7.2,
//! `pr-s27a-seat-grants-budget.md` §9 C0, AA-12; ruled QS27a-2).
//!
//! Operator machine with Godot 4.7; not in CI. The tests are `#[ignore]`d and run with
//!
//! ```text
//! cargo test -p mineworld-cli --test request_rate_2d -- --ignored --nocapture --test-threads=1
//! MINEWORLD_WRITE_TRACE=1 cargo test -p mineworld-cli --test request_rate_2d -- --ignored --nocapture --test-threads=1
//! ```
//!
//! ```text
//! mineworld server worlds/market-town --save <scratch>     the real binary, the real pack
//! walking   seat visitor, --drive=street --strides=40       out of the starting place, onto the
//!                                                           street, then 40 strides along it: the
//!                                                           click path, one 1.9 m stride a request
//! keys      seat carol,   --drive=home --exit=keys          out of her home with the arrow keys: the
//!                                                           key path, a move every 0.5 m or 20°
//! ```
//!
//! Every answer the client receives is printed as `RESULT {token, result, t_ms}` and every request it
//! sends as `REQUEST {token, request}` (`clients/2d/scripts/harness/drive.gd`). The answers' `t_ms` are
//! bucketed into wall seconds from the first answer; the per-second counts and their p50, p99 and max
//! are printed. With `MINEWORLD_WRITE_TRACE=1` the walking run's requests, each at its answer's offset
//! from the first answer, are written to `server/tests/traces/walker-2d.json` — the frozen evidence the
//! server's budget test replays (AA-6). The client holds one request in flight, so an answer's instant
//! bounds the next request's from below and the trace's spacing is the client's own.

mod godot2d;
mod headless;
mod support;

use std::collections::BTreeMap;
use std::process::Command;

use godot2d::{Drive, MARKET_TOWN, tagged};
use serde_json::{Value, json};
use support::{SaveDir, Server};

/// Where the trace is written, beside the server's other test fixtures.
const TRACE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../server/tests/traces/walker-2d.json"
);

/// The nearest-rank percentile of `sorted` (ascending).
fn percentile(sorted: &[u64], p: f64) -> u64 {
    let rank = ((p / 100.0) * sorted.len() as f64).ceil() as usize;
    sorted[rank.clamp(1, sorted.len()) - 1]
}

/// One scripted run against a fresh market-town: every answer with its offset from the first, in
/// order, and the requests by token.
async fn run(seat: &str, arguments: &[&str]) -> (Vec<(u64, Value)>, BTreeMap<String, Value>) {
    let save = SaveDir::new("request-rate-2d");
    let server = Server::start(&["server", MARKET_TOWN, "--save", save.path()]).await;
    let (status, lines) = Drive::start(server.address, seat, arguments).finish().await;
    assert!(status.success(), "the client ran to its end");
    let requests = tagged(&lines, "REQUEST ")
        .into_iter()
        .map(|line| {
            (
                line["token"].as_str().expect("a token").to_owned(),
                line["request"].clone(),
            )
        })
        .collect();
    let results = tagged(&lines, "RESULT ");
    assert!(!results.is_empty(), "the client was answered");
    let first = results[0]["t_ms"].as_u64().expect("t_ms");
    let answers = results
        .into_iter()
        .map(|r| (r["t_ms"].as_u64().expect("t_ms") - first, r))
        .collect();
    (answers, requests)
}

/// Prints the per-second answer counts of one run and their p50, p99 and max.
fn report(what: &str, answers: &[(u64, Value)]) {
    let seconds = (answers.last().expect("one").0 / 1000 + 1) as usize;
    let mut counts = vec![0_u64; seconds];
    for (offset, _) in answers {
        counts[(*offset / 1000) as usize] += 1;
    }
    let mut sorted = counts.clone();
    sorted.sort_unstable();
    let accepted = answers
        .iter()
        .filter(|(_, r)| r["result"].get("accepted").is_some())
        .count();
    println!(
        "MEASURED {what}: answers {} (accepted {accepted}) over {seconds} wall s; per-second counts \
         {counts:?}",
        answers.len()
    );
    println!(
        "MEASURED {what}: per-second submits p50 {} p99 {} max {}",
        percentile(&sorted, 50.0),
        percentile(&sorted, 99.0),
        sorted.last().expect("one")
    );
}

#[tokio::test]
#[ignore = "operator machine with Godot 4.7; not in CI"]
async fn the_2d_walker_submits_a_few_requests_a_second() {
    let (answers, requests) =
        run("visitor", &["--drive=street", "--strides=40", "--hold=30"]).await;
    assert!(answers.len() >= 20, "a walk of many strides: {answers:?}");
    report("walking", &answers);

    if std::env::var("MINEWORLD_WRITE_TRACE").as_deref() == Ok("1") {
        let source = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("git runs");
        let source = String::from_utf8(source.stdout).expect("UTF-8");
        let trace: Vec<Value> = answers
            .iter()
            .map(|(offset, result)| {
                let token = result["token"].as_str().expect("a token");
                json!({ "t_ms": offset, "request": requests[token] })
            })
            .collect();
        let document = json!({ "source": source.trim(), "world": "market-town", "seat": "visitor",
            "scenario": "--drive=street --strides=40", "requests": trace });
        std::fs::create_dir_all(std::path::Path::new(TRACE).parent().expect("a parent"))
            .expect("the traces directory");
        std::fs::write(
            TRACE,
            serde_json::to_string_pretty(&document).expect("JSON") + "\n",
        )
        .expect("the trace is written");
        println!("MEASURED trace written to {TRACE}");
    }
}

#[tokio::test]
#[ignore = "operator machine with Godot 4.7; not in CI"]
async fn the_2d_walker_with_the_keys_submits_a_few_requests_a_second() {
    let (answers, _) = run("carol", &["--drive=home", "--exit=keys"]).await;
    report("keys", &answers);
}
