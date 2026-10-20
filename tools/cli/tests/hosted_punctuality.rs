//! A hosted punctuality smoke with needs and sleep on two worlds (`pr-s35-ne-presentation.md` §6.2, AC-Ne-10;
//! S35 N-e). The worlds are Market Town and the test-only `commute-town`, each a copy with the packs
//! (`fixture/needs.rs`), so the reference world is not edited.
//!
//! ```text
//! AC-Ne-10   `server --town --solo --time-scale 300 --cadence 1 --save DIR`, from genesis until the clock
//!            passes the day's last first-shift start + 30 min (read from the copy's people: each job's
//!            `from:`, the latest of them); then the server is interrupted and the save is read back:
//!            every day-1 `shift-started` has `present: true`, and no `late-for-shift` exists at all
//! ```
//!
//! The thresholds are fixed here before any run: 30 minutes of margin, the default lateness policy (no
//! deduction, no complaint), and the one day. A FAIL is a milestone finding, never a retune. The test is
//! `#[ignore]`d for run time (two runs of about two and a half wall minutes each) and runs at the milestone:
//! `cargo test -p mineworld-cli --test hosted_punctuality -- --ignored --test-threads=1 --nocapture`.

mod headless;
mod social;

#[path = "fixture/needs.rs"]
mod needs_fixture;

mod support;

use std::path::Path;
use std::time::{Duration, Instant};

use headless::Tables;
use mineworld_contracts::EventEnvelope;
use serde_json::Value;
use support::{SaveDir, Server};

const MARKET: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/market-town");
const COMMUTE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worlds/commute-town");

/// The margin after the last first-shift start, in world seconds (fixed before any run).
const MARGIN: i64 = 30 * 60;
/// One day, in world seconds.
const DAY: i64 = 86_400;
/// World seconds per wall second (`--time-scale`; above 24 only with `--solo`).
const SCALE: &str = "300";
/// How long the recorder waits for the clock, in wall seconds.
const WAIT: Duration = Duration::from_secs(900);

/// The latest `job: from:` of any person in the copy: the day's last first-shift start, in seconds after
/// midnight. Read from the people's files as text, so no world's name is written here.
fn last_first_shift_start(copy: &Path) -> i64 {
    let mut latest = None;
    for entry in std::fs::read_dir(copy.join("people")).expect("people/ lists") {
        let text = std::fs::read_to_string(entry.expect("an entry").path())
            .expect("a person's file reads");
        let mut in_job = false;
        for line in text.lines() {
            if line.starts_with("job:") {
                in_job = true;
            } else if !line.starts_with(' ') && !line.trim().is_empty() {
                in_job = false;
            } else if in_job && line.trim_start().starts_with("from: \"") {
                let clock = line
                    .trim_start()
                    .trim_start_matches("from: \"")
                    .trim_end_matches('"');
                let (hours, minutes) = clock.split_once(':').expect("HH:MM");
                let seconds = hours.parse::<i64>().expect("hours") * 3_600
                    + minutes.parse::<i64>().expect("minutes") * 60;
                latest = Some(latest.map_or(seconds, |so_far: i64| so_far.max(seconds)));
            }
        }
    }
    latest.expect("the copy has at least one job with a first shift")
}

/// Waits until the world's clock (`/status`'s `at`) reads `at`.
async fn wait_until(server: &Server, at: i64) {
    let deadline = Instant::now() + WAIT;
    loop {
        if server.status().await["at"].as_i64().unwrap_or(0) >= at {
            return;
        }
        assert!(Instant::now() < deadline, "the world did not reach {at}");
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

fn payload(fact: &EventEnvelope) -> Value {
    serde_json::from_slice(fact.payload().payload()).unwrap_or(Value::Null)
}

/// Runs the copy of `world` hosted to the margin after its last first-shift start, and checks the save.
async fn run(name: &str, world: &str) {
    let (_scratch, copy) = {
        let scratch = needs_fixture::with_needs(name, Path::new(world), 6);
        let text = scratch.to_str().expect("a printable path").to_owned();
        (scratch, text)
    };
    let until = last_first_shift_start(Path::new(&copy)) + MARGIN;
    let save = SaveDir::new(&format!("{name}-save"));
    let arguments = [
        "server",
        copy.as_str(),
        "--town",
        "--solo",
        "--time-scale",
        SCALE,
        "--cadence",
        "1",
        "--save",
        save.path(),
    ];
    let mut server = Server::start(&arguments).await;
    wait_until(&server, until).await;
    server.interrupt();
    drop(server);
    let facts = social::facts_of(&Tables::read(Path::new(save.path())));
    let day_one: Vec<&EventEnvelope> = facts
        .iter()
        .filter(|fact| fact.at().seconds() < DAY)
        .filter(|fact| fact.event_type().as_str() == "shift-started")
        .collect();
    let late: Vec<&EventEnvelope> = facts
        .iter()
        .filter(|fact| fact.event_type().as_str() == "late-for-shift")
        .collect();
    let absent: Vec<Value> = day_one
        .iter()
        .map(|fact| payload(fact))
        .filter(|value| value["present"].as_bool() != Some(true))
        .collect();
    println!(
        "{name}: until {until} s, {} day-1 shifts, {} absent, {} late-for-shift",
        day_one.len(),
        absent.len(),
        late.len()
    );
    assert!(
        !day_one.is_empty(),
        "{name}: shifts ran on day 1 (a run that measures nothing fails)"
    );
    assert!(
        absent.is_empty(),
        "{name}: every day-1 shift is present: {absent:?}"
    );
    assert!(
        late.is_empty(),
        "{name}: nobody is late: {:?}",
        late.iter().map(|fact| payload(fact)).collect::<Vec<_>>()
    );
}

#[tokio::test]
#[ignore = "acceptance at the milestone (SPEED FIRST): AC-Ne-10 on Market Town and commute-town, hosted"]
async fn hosted_punctuality_with_needs_and_sleep_on_two_worlds() {
    run("needs-punctual-market", MARKET).await;
    run("needs-punctual-commute", COMMUTE).await;
}
