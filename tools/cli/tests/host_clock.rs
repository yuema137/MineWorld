//! The host clock, through the real binary (`pr-tw-c-host-clock.md` §8: AC-TWc-4, AC-TWc-5, AC-TWc-6's
//! binary rows; ARC-69). Real processes, real sockets, the portable graceful stop (`Server::interrupt`),
//! the portable kill (`Server::kill`).
//!
//! ```text
//! AC-TWc-4  pacing is recorded, not fact: every clock route, twice → revision unchanged; the save's
//!           facts and journal are exactly those of a run without the calls
//! AC-TWc-5  CP-TW-c: pause stands still and refuses submits; rescale to 24× and to 6× advances at the
//!           scale in force; a graceful restart without a flag resumes from the save at 6×; a flag wins; a
//!           fresh world takes world.yaml's 12×
//! AC-TWc-6  the fast-forward rule at start and live: refused without --solo, refused by name off loopback,
//!           accepted with --solo; a restart of a --solo world never keeps its fast-forward scale
//! ```

mod support;

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use mineworld_server::{RefusalCode, ServerFrame};
use serde_json::json;
use support::{
    Client, HttpAnswer, INVITE, MARKET_PACK, PACK, SaveDir, Server, admin, run_command, talk,
};

const TOKEN: &str = "host-clock-admin-token-4c1e";

async fn post(server: &Server, body: &str) -> HttpAnswer {
    admin(server.address, "POST", "/admin/clock", Some(TOKEN), body).await
}

/// `mineworld <arguments>` run to completion, with its standard error kept: the refusals name the flag or the
/// address they refuse, and the test checks that they do.
fn refused(arguments: &[&str]) -> (bool, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_mineworld"))
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("the mineworld binary runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// AC-TWc-6 at start: a scale above 24 needs `--solo`, and `--solo` needs a loopback address. Each refusal
/// happens before anything listens, exits non-zero, and names what it refuses.
#[test]
fn the_fast_forward_rule_is_refused_at_start_by_name() {
    let (ok, stderr) = refused(&[
        "server",
        PACK,
        "--time-scale",
        "25",
        "--listen",
        "127.0.0.1:0",
    ]);
    assert!(!ok, "a fast-forward without --solo is refused");
    assert!(
        stderr.contains("--time-scale 25") && stderr.contains("--solo"),
        "the refusal names the flag and what would allow it: {stderr}"
    );

    let (ok, stderr) = refused(&["server", PACK, "--solo", "--listen", "0.0.0.0:0"]);
    assert!(!ok, "--solo off loopback is refused");
    assert!(
        stderr.contains("--solo") && stderr.contains("0.0.0.0:0"),
        "the refusal names the flag and the address: {stderr}"
    );
}

/// AC-TWc-6 live, and AC-TWc-4: on social-cafe with a save, every clock route is answered as the rule says,
/// the revision never moves, and the save holds exactly the rows a run without the calls holds.
#[tokio::test]
async fn pacing_changes_are_recorded_and_never_facts() {
    let with_calls = SaveDir::new("host-clock-calls");
    let without = SaveDir::new("host-clock-plain");
    let server = Server::start(&[
        "server",
        PACK,
        "--save",
        with_calls.path(),
        "--admin-token",
        TOKEN,
    ])
    .await;
    let before = server.status().await;

    // Every route, twice where the second is a repeat that must change nothing.
    for body in [
        r#"{"paused": true}"#,
        r#"{"paused": true}"#,
        r#"{"paused": false}"#,
        r#"{"paused": false}"#,
        r#"{"time_scale": 24}"#,
        r#"{"time_scale": 24}"#,
        r#"{"time_scale": 12}"#,
        r#"{"time_scale": 12, "paused": true}"#,
        r#"{"time_scale": 12, "paused": false}"#,
    ] {
        let answer = post(&server, body).await;
        assert_eq!(answer.status, 200, "{body}: {:?}", answer.body);
    }
    // Above a day in 24 hours is refused without --solo, and that changes nothing.
    let refusal = post(&server, r#"{"time_scale": 25}"#).await;
    assert_eq!(
        (refusal.status, refusal.body["error"].clone()),
        (409, json!("fast_forward_solo_only"))
    );
    let after = server.status().await;
    assert_eq!(
        after["revision"], before["revision"],
        "no pacing change is a revision"
    );
    assert_eq!(after["time_scale"], json!(12));
    assert_eq!(after["paused"], json!(false));
    drop(server);

    // The same world, with no clock calls at all, for comparison.
    let plain = Server::start(&[
        "server",
        PACK,
        "--save",
        without.path(),
        "--admin-token",
        TOKEN,
    ])
    .await;
    drop(plain);
    // The journal's table exists only for the calls' save (`ARC-69` item 6), and nothing else differs.
    let lines = |directory: &str| -> Vec<String> {
        let (_, output) = run_command(&["inspect", directory]);
        output
            .lines()
            .filter(|line| {
                line.starts_with("journal")
                    || line.starts_with("facts")
                    || line.starts_with("snapshots")
            })
            .map(str::to_owned)
            .collect()
    };
    assert_eq!(
        lines(with_calls.path()),
        lines(without.path()),
        "the calls' save holds exactly the rows of a run without them"
    );
}

/// AC-TWc-6 live: a fast-forward is accepted only on a `--solo` world, and a `--solo` world's restart resumes
/// at its last day-length scale, not at its fast-forward (`ARC-69` item 8).
#[tokio::test]
async fn a_solo_world_may_fast_forward_and_its_restart_never_keeps_the_fast_forward() {
    let save = SaveDir::new("host-clock-solo");
    let mut server = Server::start(&[
        "server",
        PACK,
        "--solo",
        "--time-scale",
        "12",
        "--save",
        save.path(),
        "--admin-token",
        TOKEN,
    ])
    .await;
    let fast = post(&server, r#"{"time_scale": 3600}"#).await;
    assert_eq!(
        (fast.status, fast.body["time_scale"].clone()),
        (200, json!(3600))
    );
    let too_fast = post(&server, r#"{"time_scale": 3601}"#).await;
    assert_eq!(
        too_fast.status, 400,
        "above one world hour per wall second is malformed"
    );
    // Killed, not stopped: the journal's last record is the one written before the answer, so this is the
    // harshest restart (no `Stopped` record).
    server.kill();
    drop(server);

    let (restarted, captured) = Server::start_captured(
        &[
            "server",
            PACK,
            "--solo",
            "--save",
            save.path(),
            "--admin-token",
            TOKEN,
            "--invite",
            INVITE,
        ],
        None,
    )
    .await;
    let line = captured.line_starting("[world] time scale").await;
    assert_eq!(
        line, "[world] time scale 12 (from the save)",
        "the last day-length scale, not the 3600 the save was last at"
    );
    assert_eq!(restarted.status().await["time_scale"], json!(12));
}

/// AC-TWc-5 (CP-TW-c), through the binary: a market town with a save and a flag of 12×. Pause stands still and
/// refuses a submit; resume and rescale to 24× advances `at` by 240 ± 24 in 10 wall seconds and both clients are
/// told; rescale to 6× advances 60 ± 6 in 10 wall seconds; a graceful restart without a flag resumes from the
/// save at 6×; a flag wins; a fresh save of market-town takes world.yaml's 12×.
#[tokio::test]
async fn the_checkpoint_pause_rescale_and_restart_hold_through_the_binary() {
    let save = SaveDir::new("host-clock-cp");
    let mut server = Server::start(&[
        "server",
        MARKET_PACK,
        "--town",
        "--save",
        save.path(),
        "--admin-token",
        TOKEN,
        "--time-scale",
        "12",
    ])
    .await;
    let mut client = Client::connect(server.address).await;
    let (me, _) = client.join("visitor").await;
    let mut other = Client::connect(server.address).await;
    other.join("wanderer").await;

    // Pause: three readings over ten wall seconds are equal, and a submit is refused `paused`.
    let paused = post(&server, r#"{"paused": true}"#).await;
    assert_eq!(paused.status, 200);
    let frozen = server.status().await["at"].as_i64().expect("an instant");
    for _ in 0..2 {
        tokio::time::sleep(Duration::from_secs(5)).await;
        assert_eq!(
            server.status().await["at"].as_i64(),
            Some(frozen),
            "a paused clock stands still"
        );
    }
    client
        .send(json!({ "t": "submit", "token": "cp1", "request": talk(me, me, "still?") }))
        .await;
    match client.answer().await {
        ServerFrame::Refused { code, .. } => assert_eq!(code, RefusalCode::Paused),
        other => panic!("a submit while paused was answered {other:?}"),
    }

    // Resume and rescale to 24×: 240 ± 24 world seconds in 10 wall seconds, and both clients are told.
    let rescaled = post(&server, r#"{"time_scale": 24, "paused": false}"#).await;
    assert_eq!(
        (rescaled.status, rescaled.body["time_scale"].clone()),
        (200, json!(24))
    );
    for watcher in [&mut client, &mut other] {
        let told = until_clock_scale(watcher, 24).await;
        assert!(
            told < Duration::from_secs(1),
            "both clients are told within one wall second: {told:?}"
        );
    }
    let from = server.status().await["at"].as_i64().expect("an instant");
    let started = Instant::now();
    tokio::time::sleep(Duration::from_secs(10)).await;
    let to = server.status().await["at"].as_i64().expect("an instant");
    let wall = started.elapsed().as_secs_f64();
    let expected = 24.0 * wall;
    assert!(
        ((to - from) as f64 - expected).abs() <= 24.0,
        "240 ± 24 world seconds in ten wall seconds: {} in {wall:.2} s",
        to - from
    );

    // Rescale to 6×: 60 ± 6 in ten wall seconds.
    let slowed = post(&server, r#"{"time_scale": 6}"#).await;
    assert_eq!(slowed.status, 200);
    let from = server.status().await["at"].as_i64().expect("an instant");
    let started = Instant::now();
    tokio::time::sleep(Duration::from_secs(10)).await;
    let to = server.status().await["at"].as_i64().expect("an instant");
    let wall = started.elapsed().as_secs_f64();
    assert!(
        ((to - from) as f64 - 6.0 * wall).abs() <= 6.0,
        "60 ± 6 world seconds in ten wall seconds: {} in {wall:.2} s",
        to - from
    );
    let last_read = to;

    // A graceful stop, then a restart without a flag: the startup line names the save, and `at` does not run
    // backwards across the restart.
    drop(client);
    drop(other);
    let status = server.interrupt();
    assert!(
        status.success(),
        "the graceful stop exits cleanly: {status}"
    );
    drop(server);

    let (resumed, captured) = Server::start_captured(
        &[
            "server",
            MARKET_PACK,
            "--town",
            "--save",
            save.path(),
            "--admin-token",
            TOKEN,
            "--invite",
            INVITE,
        ],
        None,
    )
    .await;
    assert_eq!(
        captured.line_starting("[world] time scale").await,
        "[world] time scale 6 (from the save)"
    );
    let first = resumed.status().await["at"].as_i64().expect("an instant");
    assert!(
        first >= last_read,
        "the restart resumes no earlier than the last reading: {first} < {last_read}"
    );
    assert!(
        first <= last_read + 6 * 30,
        "and not far ahead of it: {first}"
    );
    drop(resumed);

    // A flag wins over the save; a fresh world takes world.yaml's default.
    let (flagged, captured) = Server::start_captured(
        &[
            "server",
            MARKET_PACK,
            "--town",
            "--save",
            save.path(),
            "--admin-token",
            TOKEN,
            "--time-scale",
            "12",
            "--invite",
            INVITE,
        ],
        None,
    )
    .await;
    assert_eq!(
        captured.line_starting("[world] time scale").await,
        "[world] time scale 12 (from --time-scale)"
    );
    drop(flagged);

    let fresh = SaveDir::new("host-clock-fresh");
    let (world_default, captured) = Server::start_captured(
        &[
            "server",
            MARKET_PACK,
            "--save",
            fresh.path(),
            "--admin-token",
            TOKEN,
            "--invite",
            INVITE,
        ],
        None,
    )
    .await;
    assert_eq!(
        captured.line_starting("[world] time scale").await,
        "[world] time scale 12 (from world.yaml)"
    );
    drop(world_default);
}

/// Reads frames until a `clock` says `want`, and returns how long that took. A clock frame is the newest state
/// at the moment the session sends it (`PROTOCOL.md` §5.9): one sent before the change, still in flight, is
/// superseded and skipped here.
async fn until_clock_scale(client: &mut Client, want: u32) -> Duration {
    let asked = Instant::now();
    loop {
        match client.frame().await {
            ServerFrame::Clock { time_scale, .. } if time_scale == want => return asked.elapsed(),
            ServerFrame::Clock { .. } | ServerFrame::Observation { .. } => {}
            other => panic!("a clock frame was expected, and the server said {other:?}"),
        }
    }
}

/// `POST /admin/stop` (ARC-268, AC-WC-13's server half): a wrong token is refused and stops nothing; the
/// right one is answered 202, the process exits 0 by itself with a client still connected, the save's host
/// journal ends with `Stopped`, and a restart resumes no earlier than the last reading.
#[tokio::test]
async fn the_stop_route_takes_the_graceful_path() {
    let save = SaveDir::new("host-clock-stop");
    let arguments = [
        "server",
        PACK,
        "--solo",
        "--save",
        save.path(),
        "--admin-token",
        TOKEN,
    ];
    let mut server = Server::start(&arguments).await;
    let mut client = Client::connect(server.address).await;
    client.join("visitor").await;

    let wrong = admin(
        server.address,
        "POST",
        "/admin/stop",
        Some("not-the-token"),
        "{}",
    )
    .await;
    assert_eq!(wrong.status, 401, "a wrong token is refused");
    let odd = admin(
        server.address,
        "POST",
        "/admin/stop",
        Some(TOKEN),
        r#"{"at": 5}"#,
    )
    .await;
    assert_eq!(
        odd.status, 400,
        "a field the route does not define is malformed"
    );
    assert!(
        server.status().await["at"].is_i64(),
        "neither refusal stopped the world"
    );

    let last_read = server.status().await["at"].as_i64().expect("an instant");
    let asked = admin(server.address, "POST", "/admin/stop", Some(TOKEN), "{}").await;
    assert_eq!(
        (asked.status, asked.body["stopping"].clone()),
        (202, json!(true))
    );
    let status = server
        .exited_within(Duration::from_secs(20))
        .await
        .expect("the server stops by itself after /admin/stop");
    assert!(status.success(), "the graceful stop exits 0: {status}");
    drop(client);
    drop(server);

    let backend = mineworld_persistence::SqliteBackend::open(
        std::path::Path::new(save.path()),
        mineworld_persistence::Durability::PowerLoss,
    )
    .expect("the save opens");
    let rows =
        mineworld_persistence::PersistenceBackend::host_records(&backend).expect("a journal");
    let last: mineworld_persistence::HostRecord =
        serde_json::from_slice(&rows.last().expect("a record").record).expect("a host record");
    assert_eq!(last.change, mineworld_persistence::PacingChange::Stopped);
    drop(backend);

    let resumed = Server::start(&arguments).await;
    let first = resumed.status().await["at"].as_i64().expect("an instant");
    assert!(
        first >= last_read,
        "the restart resumes no earlier: {first} < {last_read}"
    );
}
