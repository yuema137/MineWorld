//! Many players, a living town (step-13 S12 PR 13e): real 2D clients against a real hosted server,
//! AC-13e-1 … AC-13e-8 of `.structured-coding/plans/mvp0/pr-13e-many-players-2d.md` §6.
//!
//! Every test starts Godot, so every test is `#[ignore]`d: `cargo test` reports them as ignored,
//! never as passed, on a machine without Godot (a test not run is not a pass). Run them with
//!
//! ```text
//! cargo test -p mineworld-cli --test client_2d_many -- --ignored --test-threads=1
//! ```
//!
//! Oracles are the server's: `/status`, its welcome and closing codes as the client printed them, the
//! identity another client's observation listed, and `mineworld perceived` — never the client's own
//! verdict alone. Every criterion runs on a world besides market-town (lane charter).

mod godot2d;
mod headless;
mod support;

use godot2d::{Drive, MARKET_TOWN, World, evidence, passed, tagged};
use serde_json::Value;
use support::SaveDir;

const SOCIAL_CAFE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/social-cafe");
const LAKESIDE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/lakeside");

/// `mineworld server <world> --save <save>` plus `extra`, as owned arguments.
fn hosted(world: &str, save: &SaveDir, extra: &[&str]) -> Vec<String> {
    let mut out: Vec<String> = ["server", world, "--save", save.path()]
        .map(str::to_owned)
        .to_vec();
    out.extend(extra.iter().map(|a| (*a).to_owned()));
    out
}

fn args(owned: &[String]) -> Vec<&str> {
    owned.iter().map(String::as_str).collect()
}

/// The World Pack's directory name, which the client is told for its far field.
fn world_key(world: &str) -> String {
    format!(
        "--world-key={}",
        std::path::Path::new(world)
            .file_name()
            .and_then(|n| n.to_str())
            .expect("a world directory")
    )
}

/// The welcome evidence lines, in order.
fn welcomes(lines: &[String]) -> Vec<Value> {
    evidence(lines, "welcome")
}

/// AC-13e-5 — the client's socket closes under it mid-session: it is welcomed again `held`, as the
/// same observer, within the server's default 30 s hold, and no `seat_occupied` is ever answered.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_many -- --ignored"]
async fn a_blink_resumes_the_same_seat() {
    for (world, seat) in [(MARKET_TOWN, "visitor"), (SOCIAL_CAFE, "wanderer")] {
        let save = SaveDir::new("2d-many-blink");
        let mut server = World::start(&args(&hosted(world, &save, &[])), None).await;
        let key = world_key(world);
        let (status, lines) = Drive::start(server.address, seat, &["--drive=blink", &key])
            .finish()
            .await;
        server.kill();
        assert!(status.success() && passed(&lines), "{world}: the drive passes");
        let welcomed = welcomes(&lines);
        assert_eq!(welcomed.len(), 2, "{world}: welcomed twice: {welcomed:?}");
        assert_eq!(welcomed[0]["observer"], welcomed[1]["observer"], "{world}: the same Person");
        assert_eq!(welcomed[1]["took_over"], "held", "{world}: {:?}", welcomed[1]);
        assert!(
            !tagged(&lines, "REFUSED ")
                .iter()
                .any(|r| r["code"] == "seat_occupied"),
            "{world}: never seat_occupied"
        );
        let back = &evidence(&lines, "blink_back")[0];
        assert!(
            back["after_ms"].as_u64().expect("after_ms") <= 30_000 + 5_000,
            "{world}: within the hold: {back}"
        );
    }
}

/// The `t_ms` of a tagged record.
fn t_ms(record: &Value) -> u64 {
    record["t_ms"].as_u64().expect("t_ms")
}

/// The first `JOIN_SCREEN` record that satisfies `what`.
fn screen_where(lines: &[String], what: impl Fn(&Value) -> bool) -> Option<Value> {
    tagged(lines, "JOIN_SCREEN ").into_iter().find(|s| what(s))
}

/// AC-13e-1 — started with no `--seat`, the join screen lists exactly `/status`'s seats, in order;
/// choosing the first and pressing Join seats the client, its observer drawn within 10 s.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_many -- --ignored"]
async fn joins_from_the_screen() {
    for world in [MARKET_TOWN, LAKESIDE] {
        let save = SaveDir::new("2d-many-join");
        let mut server = World::start(&args(&hosted(world, &save, &[])), None).await;
        let listed = server.status().await["seats"].clone();
        let key = world_key(world);
        let (status, lines) = Drive::start_seatless(server.address, &["--drive=join", "--pick=0", &key])
            .finish()
            .await;
        server.kill();
        assert!(status.success() && passed(&lines), "{world}: the drive passes");
        let shown = screen_where(&lines, |s| s["seats"].as_array().is_some_and(|a| !a.is_empty()))
            .expect("the screen listed seats");
        assert_eq!(shown["seats"], listed, "{world}: the seats are /status's, in order");
        let join = &tagged(&lines, "JOIN ")[0];
        assert_eq!(join["seat"], listed[0], "{world}: entry 0 was asked for");
        assert_eq!(join["take_over"], false);
        let seat = &tagged(&lines, "SEAT ")[0];
        assert!(t_ms(seat) - t_ms(join) <= 10_000, "{world}: seated within 10 s: {seat}");
        assert_eq!(evidence(&lines, "drawn")[0]["seated"], true, "{world}");
        assert!(
            evidence(&lines, "drawn")[0]["people"].as_u64().unwrap_or(0) >= 1,
            "{world}: the observer is drawn"
        );
    }
}

/// AC-13e-2 (and AD-13e-6) — a wrong invite: the screen answers `unauthorized` within 3 s; nothing
/// was drawn and nothing requested; neither invite appears in anything the client printed.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_many -- --ignored"]
async fn a_wrong_invite_draws_nothing() {
    const WRONG: &str = "not-the-invite-7d1e";
    let save = SaveDir::new("2d-many-wrong");
    let mut server = World::start(&args(&hosted(SOCIAL_CAFE, &save, &[])), None).await;
    let key = world_key(SOCIAL_CAFE);
    let wrong = format!("--invite-override={WRONG}");
    let (status, lines) = Drive::start_seatless(server.address, &["--drive=join", &wrong, &key])
        .finish()
        .await;
    server.kill();
    assert!(status.success(), "the drive ends");
    let join = &tagged(&lines, "JOIN ")[0];
    let answer = screen_where(&lines, |s| s["answer"] == "unauthorized" && s["visible"] == true)
        .expect("the screen answers unauthorized");
    assert!(t_ms(&answer) - t_ms(join) <= 3_000, "within 3 s: {answer}");
    let drawn = &evidence(&lines, "drawn")[0];
    assert_eq!(drawn["people"], 0, "{drawn}");
    assert_eq!(drawn["places"], 0, "{drawn}");
    assert!(tagged(&lines, "REQUEST ").is_empty(), "nothing requested");
    assert!(tagged(&lines, "SEAT ").is_empty(), "never seated");
    for secret in [support::INVITE, WRONG] {
        assert!(
            !lines.iter().any(|line| line.contains(secret)),
            "an invite was printed"
        );
    }
}

/// AC-13e-3 — B asks for the seat A holds: refused `seat_occupied`, offered Take over; on purpose B
/// takes it (`took_over: "connection"`, told so), A is told `taken_over`, returns to its join screen
/// and sends no join afterwards.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_many -- --ignored"]
async fn an_occupied_seat_is_refused_then_taken_on_purpose() {
    for world in [MARKET_TOWN, SOCIAL_CAFE] {
        let save = SaveDir::new("2d-many-occupied");
        let mut server = World::start(&args(&hosted(world, &save, &[])), None).await;
        let key = world_key(world);
        let a = Drive::start(server.address, "visitor", &["--drive=idle", "--hold=40", &key]);
        a.until("A seated", |lines| lines.iter().any(|l| l.starts_with("SEAT ")))
            .await;
        let b = Drive::start_seatless(
            server.address,
            &[
                "--drive=join",
                "--pick-seat=visitor",
                "--take-over",
                "--take-over-after=5",
                "--hold=5",
                &key,
            ],
        );
        let (b_status, b_lines) = b.finish().await;
        let (a_status, a_lines) = a.finish().await;
        server.kill();
        assert!(b_status.success() && passed(&b_lines), "{world}: B passes");
        assert!(a_status.success(), "{world}: A ends");

        let occupied = screen_where(&b_lines, |s| s["answer"] == "seat_occupied" && s["visible"] == true)
            .expect("B is told the seat is occupied");
        assert_eq!(occupied["take_over"], true, "{world}: Take over offered");
        let joins = tagged(&b_lines, "JOIN ");
        assert_eq!(
            joins.iter().map(|j| j["take_over"].clone()).collect::<Vec<_>>(),
            [Value::Bool(false), Value::Bool(true)],
            "{world}: a join, then a takeover on purpose"
        );
        let b_welcome = welcomes(&b_lines);
        assert_eq!(b_welcome.len(), 1, "{world}");
        assert_eq!(b_welcome[0]["took_over"], "connection", "{world}");
        assert!(
            tagged(&b_lines, "TOAST ")
                .iter()
                .any(|t| t["kind"] == "seat" && t["took_over"] == "connection"),
            "{world}: B is told it took the Person over"
        );

        let ended = tagged(&a_lines, "ENDED ");
        assert_eq!(ended.len(), 1, "{world}: A ended once: {ended:?}");
        assert_eq!(ended[0]["code"], "taken_over", "{world}");
        assert!(
            tagged(&a_lines, "TOAST ")
                .iter()
                .any(|t| t["kind"] == "seat" && t["reason"] == "taken_over"),
            "{world}: A is told"
        );
        assert!(
            screen_where(&a_lines, |s| s["visible"] == true && s["answer"] == "taken_over").is_some(),
            "{world}: A is back at its join screen"
        );
        let a_joins = tagged(&a_lines, "JOIN ");
        assert_eq!(a_joins.len(), 1, "{world}: A never joins again: {a_joins:?}");
        // A held its run for 40 s after it was seated; the takeover came within B's first ~20 s,
        // so A watched for at least 10 s after it ended and sent nothing.
        let a_seated = t_ms(&tagged(&a_lines, "SEAT ")[0]);
        assert!(t_ms(&ended[0]) - a_seated <= 30_000, "{world}: the takeover left 10 s to watch");
    }
}

/// AC-13e-4 — with `--town`, joining a seat the town drives is welcomed `took_over: "hosted"`, the
/// player is told by the Person's displayed name, and the observer is that seat's Person.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_many -- --ignored"]
async fn taking_over_a_town_person() {
    for (world, seat) in [(SOCIAL_CAFE, "bob"), (LAKESIDE, "bruno")] {
        let person = godot2d::worlds::ids(world)[seat].clone();
        let save = SaveDir::new("2d-many-hosted");
        let mut server = World::start(&args(&hosted(world, &save, &["--town"])), None).await;
        let key = world_key(world);
        let pick = format!("--pick-seat={seat}");
        let (status, lines) = Drive::start_seatless(server.address, &["--drive=join", &pick, &key])
            .finish()
            .await;
        server.kill();
        assert!(status.success() && passed(&lines), "{world}: the drive passes");
        let welcome = &welcomes(&lines)[0];
        assert_eq!(welcome["took_over"], "hosted", "{world}: {welcome}");
        assert_eq!(welcome["observer"], person.as_str(), "{world}: the seat's Person");
        let seated = &tagged(&lines, "SEAT ")[0];
        let name = seated["name"].as_str().expect("a name").to_owned();
        assert!(!name.is_empty(), "{world}: named");
        assert!(
            tagged(&lines, "TOAST ").iter().any(|t| t["kind"] == "seat"
                && t["took_over"] == "hosted"
                && t["text"].as_str().is_some_and(|text| text.contains(&name))),
            "{world}: told by name"
        );
    }
}
