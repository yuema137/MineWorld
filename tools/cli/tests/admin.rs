//! The admin surface, run for real (step-12 §18.5 DA-1, DA-3 … DA-6, DA-8's binary half, DA-9; SD-D13).
//!
//! ```text
//! DA-1  no --admin-token: every /admin route is 404, and the server says there is no admin surface
//! DA-3  social-cafe --save --admin-token: every route twice; the revision never moves, and the save
//!       holds the genesis facts and nothing else
//! DA-4  market-town --town --save: sessions and seats listed; a kick closes {kicked}, the seat goes
//!       back to the town with no hold, the kicked resume is dead
//! DA-5  release a connected seat, a held seat, a hosted seat, an unknown seat
//! DA-6  pause: announced, the town stands still, a submit is refused, a hold still ends; resume
//!       continues from the frozen instant and the town acts again
//! DA-9  the token and the nicknames go nowhere they should not
//! ```
//!
//! The real binary and real sockets (`support`); every server is ended with `Child::kill`, which is
//! portable, except SD-D13's graceful-stop check, which is Unix-only until S13's Windows helper lands
//! (step-12 §18.14, QW-3).

mod fixture;
mod support;

use std::time::{Duration, Instant};

use mineworld_server::{ClosingReason, RefusalCode, ServerFrame, TookOver};
use serde_json::{Value, json};
use support::{
    Client, HttpAnswer, INVITE, MARKET_PACK, PACK, SaveDir, Server, admin, file_containing,
    join_frame, run_command, talk,
};

/// The admin token these servers are started with.
const TOKEN: &str = "cli-admin-token-9b3f";

async fn get(server: &Server, path: &str) -> HttpAnswer {
    admin(server.address, "GET", path, Some(TOKEN), "").await
}

async fn post(server: &Server, path: &str, body: &str) -> HttpAnswer {
    admin(server.address, "POST", path, Some(TOKEN), body).await
}

/// A join with this nickname.
fn join_as(seat: &str, nickname: &str) -> Value {
    let mut join = join_frame(seat);
    join["nickname"] = json!(nickname);
    join
}

/// The welcome's session, observer and resume secret.
fn welcomed(frame: &ServerFrame) -> (String, String, String, TookOver) {
    match frame {
        ServerFrame::Welcome {
            session,
            observer,
            resume,
            took_over,
            ..
        } => (
            session.to_string(),
            observer.to_string(),
            resume.as_ref().expect("a resume").reveal().to_owned(),
            *took_over,
        ),
        other => panic!("a welcome was expected, and the server said {other:?}"),
    }
}

async fn closed_because(client: &mut Client, expected: ClosingReason) {
    match client.answer().await {
        ServerFrame::Closing { reason, .. } => assert_eq!(reason, expected),
        other => panic!("a closing was expected, and the server said {other:?}"),
    }
}

/// The state of one seat on `GET /admin/seats`.
async fn seat_state(server: &Server, seat: &str) -> Value {
    let seats = get(server, "/admin/seats").await.body;
    seats["seats"]
        .as_array()
        .expect("a list")
        .iter()
        .find(|entry| entry["seat"] == json!(seat))
        .cloned()
        .unwrap_or_else(|| panic!("{seat} is not listed: {seats}"))
}

/// The genesis fact count `validate` states, and the fact count `inspect` reads from a save.
fn genesis_and_saved(pack: &str, save: &str) -> (String, String) {
    let (validated, said) = run_command(&["validate", pack]);
    assert!(validated.success(), "{said}");
    let genesis = said
        .lines()
        .find_map(|line| {
            line.split_once(" genesis fact(s)")
                .map(|(count, _)| count.trim().to_owned())
        })
        .expect("validate counts the genesis facts");
    let (inspected, report) = run_command(&["inspect", save]);
    assert!(inspected.success(), "{report}");
    let saved = report
        .lines()
        .find_map(|line| {
            line.strip_prefix("facts ")
                .map(|count| count.trim().to_owned())
        })
        .unwrap_or_else(|| panic!("inspect counts the facts: {report}"));
    (genesis, saved)
}

// ---------------------------------------------------------------------------------------------
// DA-1 — absent without a token.
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn without_an_admin_token_there_is_no_admin_surface() {
    let (server, output) = Server::start_with_env(&["server", PACK], &[]).await;
    output
        .line_starting("[mineworld] no admin surface (no --admin-token)")
        .await;
    for (method, path) in [
        ("GET", "/admin/sessions"),
        ("GET", "/admin/seats"),
        ("POST", "/admin/sessions/1/kick"),
        ("POST", "/admin/seats/visitor/release"),
        ("GET", "/admin/clock"),
        ("POST", "/admin/clock"),
    ] {
        let answer = admin(
            server.address,
            method,
            path,
            Some(TOKEN),
            r#"{"paused": true}"#,
        )
        .await;
        assert_eq!(answer.status, 404, "{method} {path}: {:?}", answer.body);
    }
}

// ---------------------------------------------------------------------------------------------
// DA-3 (+ DA-8's binary half) — no admin route changes world state.
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn no_admin_route_changes_world_state() {
    // Nobody acts and no routine boundary falls in the first hour after genesis, so only an admin
    // route could move the revision here.
    fixture::assert_quiet(0, 3_600, "admin.rs");
    let save = SaveDir::new("admin-host-state");
    let mut server = Server::start(&[
        "server",
        PACK,
        "--save",
        save.path(),
        "--admin-token",
        TOKEN,
    ])
    .await;
    let mut idle = Client::connect(server.address).await;
    let (session, _, _, _) = welcomed(&idle.join_as(join_as("visitor", "Idle")).await);
    let before = server.status().await;

    for round in 0..2 {
        assert_eq!(get(&server, "/admin/sessions").await.status, 200);
        assert_eq!(get(&server, "/admin/seats").await.status, 200);
        assert_eq!(get(&server, "/admin/clock").await.status, 200);
        let paused = post(&server, "/admin/clock", r#"{"paused": true}"#).await;
        assert_eq!(paused.body["paused"], json!(true));
        let resumed = post(&server, "/admin/clock", r#"{"paused": false}"#).await;
        assert_eq!(resumed.body["paused"], json!(false));
        let released = post(&server, "/admin/seats/wanderer/release", "").await;
        assert_eq!(released.body["released"], json!(false), "a free seat");
        let kicked = post(&server, &format!("/admin/sessions/{session}/kick"), "").await;
        if round == 0 {
            assert_eq!(kicked.status, 200, "{:?}", kicked.body);
            closed_because(&mut idle, ClosingReason::Kicked).await;
        } else {
            assert_eq!(kicked.status, 404, "kicked once");
        }
    }
    // DA-8's binary half: no body names an instant, and a live scale change is TW-c's.
    for (body, status) in [
        (r#"{"at": 999999}"#, 400),
        (r#"{"paused": true, "at": 5}"#, 400),
        (r#"{"time_scale": 12}"#, 409),
        ("{}", 400),
    ] {
        assert_eq!(
            post(&server, "/admin/clock", body).await.status,
            status,
            "{body}"
        );
    }

    let after = server.status().await;
    assert_eq!(after["revision"], before["revision"], "{before} → {after}");
    assert_eq!(after["paused"], json!(false));
    assert_eq!(after["time_scale"], before["time_scale"]);
    assert!(after["at"].as_i64() >= before["at"].as_i64());

    server.kill();
    let (genesis, saved) = genesis_and_saved(PACK, save.path());
    assert_eq!(
        saved, genesis,
        "the save holds the genesis facts and nothing else"
    );
}

// ---------------------------------------------------------------------------------------------
// DA-4 and DA-5 — kick and release (CP-D).
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn a_kick_or_a_release_gives_the_seat_back_to_the_town_and_kills_the_resume() {
    let save = SaveDir::new("admin-kick");
    let server = Server::start(&[
        "server",
        MARKET_PACK,
        "--town",
        "--save",
        save.path(),
        "--admin-token",
        TOKEN,
    ])
    .await;
    let mut avery = Client::connect(server.address).await;
    let (a_session, a_observer, _, _) = welcomed(&avery.join_as(join_as("visitor", "Avery")).await);
    let mut bea = Client::connect(server.address).await;
    let (b_session, b_observer, b_resume, took_over) =
        welcomed(&bea.join_as(join_as("alice", "Bea")).await);
    assert_eq!(took_over, TookOver::Hosted);

    // ── DA-4: who is here, and on which seat. ──────────────────────────────────────────────────
    let sessions = get(&server, "/admin/sessions").await.body;
    let listed: Vec<(String, String, String, String)> = sessions["sessions"]
        .as_array()
        .expect("a list")
        .iter()
        .map(|entry| {
            let text = |field: &str| entry[field].as_str().expect("a string").to_owned();
            (
                text("session"),
                text("nickname"),
                text("seat"),
                text("observer"),
            )
        })
        .collect();
    assert_eq!(
        listed,
        vec![
            (
                a_session.clone(),
                "Avery".into(),
                "visitor".into(),
                a_observer
            ),
            (b_session.clone(), "Bea".into(), "alice".into(), b_observer),
        ]
    );
    let seats = get(&server, "/admin/seats").await.body;
    for entry in seats["seats"].as_array().expect("a list") {
        let expected = match entry["seat"].as_str() {
            Some("visitor") => {
                json!({ "seat": "visitor", "state": "connected", "session": a_session })
            }
            Some("alice") => json!({ "seat": "alice", "state": "connected", "session": b_session }),
            _ => json!({ "seat": entry["seat"], "state": "hosted" }),
        };
        assert_eq!(entry, &expected);
    }

    let kicked = post(&server, &format!("/admin/sessions/{b_session}/kick"), "").await;
    assert_eq!(
        kicked.body,
        json!({ "session": b_session, "seat": "alice", "state": "hosted" })
    );
    closed_because(&mut bea, ClosingReason::Kicked).await;
    assert_eq!(
        seat_state(&server, "alice").await["state"],
        json!("hosted"),
        "no hold: alice is the town's again at once"
    );
    let mut back = Client::connect(server.address).await;
    let mut resume = join_frame("alice");
    resume["resume"] = json!(b_resume);
    match back.join_as(resume).await {
        ServerFrame::Refused { code, .. } => assert_eq!(code, RefusalCode::InvalidResume),
        other => panic!("a kicked binding's resume was answered {other:?}"),
    }
    let after = get(&server, "/admin/sessions").await.body;
    assert!(
        !after.to_string().contains("\"Bea\""),
        "the kicked session is no longer listed: {after}"
    );
    let unknown = post(&server, "/admin/sessions/987654/kick", "").await;
    assert_eq!(
        (unknown.status, unknown.body["error"].clone()),
        (404, json!("unknown_session"))
    );

    // ── DA-5: release a connected seat, a held seat, a hosted seat, and no seat. ─────────────────
    let released = post(&server, "/admin/seats/visitor/release", "").await;
    assert_eq!(
        released.body,
        json!({ "seat": "visitor", "released": true, "state": "hosted" })
    );
    closed_because(&mut avery, ClosingReason::Kicked).await;

    let mut cleo = Client::connect(server.address).await;
    let (_, _, c_resume, _) = welcomed(&cleo.join_as(join_as("wanderer", "Cleo")).await);
    cleo.disconnect().await;
    let deadline = Instant::now() + support::PATIENCE;
    while seat_state(&server, "wanderer").await["state"] != json!("held") {
        assert!(Instant::now() < deadline, "wanderer was never held");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let released = post(&server, "/admin/seats/wanderer/release", "").await;
    assert_eq!(
        released.body,
        json!({ "seat": "wanderer", "released": true, "state": "hosted" })
    );
    let mut cleo_again = Client::connect(server.address).await;
    let mut resume = join_frame("wanderer");
    resume["resume"] = json!(c_resume);
    match cleo_again.join_as(resume).await {
        ServerFrame::Refused { code, .. } => assert_eq!(code, RefusalCode::InvalidResume),
        other => panic!("a released hold's resume was answered {other:?}"),
    }

    let hosted = post(&server, "/admin/seats/bob/release", "").await;
    assert_eq!(
        hosted.body,
        json!({ "seat": "bob", "released": false, "state": "hosted" })
    );
    assert_eq!(seat_state(&server, "bob").await["state"], json!("hosted"));
    let unknown = post(&server, "/admin/seats/nobody/release", "").await;
    assert_eq!(
        (unknown.status, unknown.body["error"].clone()),
        (404, json!("unknown_seat"))
    );
}

// ---------------------------------------------------------------------------------------------
// DA-6 — pause and resume (S19 §7.2).
// ---------------------------------------------------------------------------------------------

/// Reads frames until a `clock` arrives, and returns its `paused` and how long that took.
async fn next_clock(client: &mut Client) -> (bool, Duration) {
    let asked = Instant::now();
    loop {
        match client.frame().await {
            ServerFrame::Clock { paused, .. } => return (paused, asked.elapsed()),
            ServerFrame::Observation { .. } => {}
            other => panic!("a clock frame was expected, and the server said {other:?}"),
        }
    }
}

#[tokio::test]
async fn a_paused_town_stands_still_and_resumes_where_it_stopped() {
    let save = SaveDir::new("admin-pause");
    let server = Server::start(&[
        "server",
        MARKET_PACK,
        "--town",
        "--hold",
        "3",
        "--save",
        save.path(),
        "--admin-token",
        TOKEN,
    ])
    .await;
    let mut one = Client::connect(server.address).await;
    let (_, one_observer, _, _) = welcomed(&one.join_as(join_as("visitor", "One")).await);
    let mut two = Client::connect(server.address).await;
    two.join_as(join_as("wanderer", "Two")).await;
    let mut dropping = Client::connect(server.address).await;
    dropping.join_as(join_as("ivan", "Drop")).await;
    // Every welcome is followed by the clock as it stands: running.
    for client in [&mut one, &mut two] {
        assert!(!next_clock(client).await.0, "the clock runs at the welcome");
    }

    let paused = post(&server, "/admin/clock", r#"{"paused": true}"#).await;
    assert_eq!(
        (paused.status, paused.body["paused"].clone()),
        (200, json!(true))
    );
    for client in [&mut one, &mut two] {
        let (now_paused, took) = next_clock(client).await;
        assert!(now_paused);
        assert!(
            took <= Duration::from_secs(1),
            "the clock frame took {took:?}"
        );
    }
    dropping.disconnect().await;

    let frozen = server.status().await;
    assert_eq!(frozen["paused"], json!(true));
    for _ in 0..2 {
        tokio::time::sleep(Duration::from_secs(2)).await;
        let still = server.status().await;
        assert_eq!(still["at"], frozen["at"], "a paused clock stands still");
        assert_eq!(
            still["revision"], frozen["revision"],
            "a paused town does not act"
        );
    }

    let me = mineworld_contracts::EntityId::from_raw(one_observer.parse().expect("an id"));
    one.send(json!({ "t": "submit", "token": "p1", "request": talk(me, me, "hello?") }))
        .await;
    match one.answer().await {
        ServerFrame::Refused { code, token, .. } => {
            assert_eq!(code, RefusalCode::Paused);
            assert_eq!(
                token.map(|token| token.as_str().to_owned()),
                Some("p1".into())
            );
        }
        other => panic!("a submit while paused was answered {other:?}"),
    }
    assert_eq!(server.status().await["revision"], frozen["revision"]);
    // The hold (3 wall seconds) ended during the pause: holds are wall time.
    assert_eq!(seat_state(&server, "ivan").await["state"], json!("hosted"));

    let resumed_at = Instant::now();
    let resumed = post(&server, "/admin/clock", r#"{"paused": false}"#).await;
    assert_eq!(resumed.body["paused"], json!(false));
    for client in [&mut one, &mut two] {
        assert!(!next_clock(client).await.0, "the clock runs again");
    }
    let first = server.status().await;
    let frozen_at = frozen["at"].as_i64().expect("an instant");
    let at = first["at"].as_i64().expect("an instant");
    let bound = i64::try_from(resumed_at.elapsed().as_secs()).expect("small") + 1;
    let scale = first["time_scale"].as_i64().expect("a scale");
    assert!(
        at >= frozen_at && at <= frozen_at + bound * scale,
        "after resume {at} is not within [{frozen_at}, {frozen_at} + {bound} × {scale}]"
    );
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if server.status().await["revision"] != frozen["revision"] {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "the town did not act within 30 s of resuming"
        );
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

// ---------------------------------------------------------------------------------------------
// DA-9 — secrets and names.
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn the_admin_token_and_the_nicknames_go_nowhere_else() {
    const NICKNAME: &str = "Zephyrine";
    let save = SaveDir::new("admin-secrets");
    let (mut server, output) = Server::start_with_env(
        &[
            "server",
            PACK,
            "--save",
            save.path(),
            "--admin-token",
            TOKEN,
        ],
        &[],
    )
    .await;
    let line = output.line_starting("[mineworld] admin surface: ").await;
    assert!(line.ends_with("/admin (bearer token as given)"), "{line}");
    let mut named = Client::connect(server.address).await;
    let welcome = named.join_as(join_as("visitor", NICKNAME)).await;
    assert!(
        matches!(&welcome, ServerFrame::Welcome { nickname, .. } if nickname.as_str() == NICKNAME)
    );
    let mut other = Client::connect(server.address).await;
    other.join_as(join_as("wanderer", "Other")).await;
    assert!(
        get(&server, "/admin/sessions")
            .await
            .body
            .to_string()
            .contains(NICKNAME),
        "the admin surface names the player"
    );
    assert!(
        !server.status().await.to_string().contains(NICKNAME),
        "/status names nobody"
    );
    let paused = post(&server, "/admin/clock", r#"{"paused": true}"#).await;
    assert_eq!(paused.status, 200);
    for _ in 0..10 {
        let frame = serde_json::to_string(&other.frame().await).expect("a frame");
        assert!(
            !frame.contains(NICKNAME),
            "another client was told a nickname: {frame}"
        );
        assert!(
            !frame.contains(TOKEN),
            "a client was sent the token: {frame}"
        );
    }
    server.kill();
    tokio::time::sleep(Duration::from_millis(200)).await;
    for written in output.stdout().iter().chain(&output.stderr()) {
        assert!(
            !written.contains(TOKEN),
            "the admin token was printed: {written}"
        );
        assert!(
            !written.contains(NICKNAME),
            "a nickname was logged: {written}"
        );
    }
    let directory = std::path::Path::new(save.path());
    assert_eq!(
        file_containing(directory, TOKEN.as_bytes()),
        None,
        "the token is in the save"
    );
    assert_eq!(
        file_containing(directory, NICKNAME.as_bytes()),
        None,
        "a nickname is in the save"
    );

    // The environment variable behaves as the flag, and is not echoed either.
    const FROM_ENV: &str = "env-admin-token-41c7";
    let (server, output) =
        Server::start_with_env(&["server", PACK], &[("MINEWORLD_ADMIN_TOKEN", FROM_ENV)]).await;
    output.line_starting("[mineworld] admin surface: ").await;
    let clock = admin(server.address, "GET", "/admin/clock", Some(FROM_ENV), "").await;
    assert_eq!(clock.status, 200);
    let wrong = admin(server.address, "GET", "/admin/clock", Some(INVITE), "").await;
    assert_eq!(wrong.status, 401, "the invite is not the admin token");
    for written in output.stdout().iter().chain(&output.stderr()) {
        assert!(
            !written.contains(FROM_ENV),
            "the admin token was printed: {written}"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// SD-D13 — a graceful stop takes the same path with an admin surface mounted (Unix; §18.14, QW-3).
// ---------------------------------------------------------------------------------------------

#[cfg(unix)]
#[tokio::test]
async fn an_interrupt_stops_an_administered_server_gracefully() {
    let save = SaveDir::new("admin-stop");
    let (mut server, output) = Server::start_with_env(
        &[
            "server",
            PACK,
            "--save",
            save.path(),
            "--admin-token",
            TOKEN,
        ],
        &[],
    )
    .await;
    assert_eq!(
        post(&server, "/admin/clock", r#"{"paused": true}"#)
            .await
            .status,
        200
    );
    let ended = server.interrupt();
    assert!(ended.success(), "a graceful stop exits cleanly: {ended:?}");
    tokio::time::sleep(Duration::from_millis(200)).await;
    let stdout = output.stdout();
    assert!(
        stdout.iter().any(|line| line == "[mineworld] stopping"),
        "{stdout:?}"
    );
    assert!(
        stdout.iter().any(|line| line.starts_with("[world] ticks ")),
        "the statistics were printed: {stdout:?}"
    );
}
