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

use std::collections::BTreeSet;

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
    if world == LAKESIDE {
        // Milestone E's composition is found only through its pack roots (ARC-54).
        for root in ["entities", "presentation/mineworld-default"] {
            out.extend([
                "--packs".to_owned(),
                format!("{}/../../{root}", env!("CARGO_MANIFEST_DIR")),
            ]);
        }
    }
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
        assert!(
            status.success() && passed(&lines),
            "{world}: the drive passes"
        );
        let welcomed = welcomes(&lines);
        assert_eq!(welcomed.len(), 2, "{world}: welcomed twice: {welcomed:?}");
        assert_eq!(
            welcomed[0]["observer"], welcomed[1]["observer"],
            "{world}: the same Person"
        );
        assert_eq!(
            welcomed[1]["took_over"], "held",
            "{world}: {:?}",
            welcomed[1]
        );
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
        let (status, lines) =
            Drive::start_seatless(server.address, &["--drive=join", "--pick=0", &key])
                .finish()
                .await;
        server.kill();
        assert!(
            status.success() && passed(&lines),
            "{world}: the drive passes"
        );
        let shown = screen_where(&lines, |s| {
            s["seats"].as_array().is_some_and(|a| !a.is_empty())
        })
        .expect("the screen listed seats");
        assert_eq!(
            shown["seats"], listed,
            "{world}: the seats are /status's, in order"
        );
        let join = &tagged(&lines, "JOIN ")[0];
        assert_eq!(join["seat"], listed[0], "{world}: entry 0 was asked for");
        assert_eq!(join["take_over"], false);
        let seat = &tagged(&lines, "SEAT ")[0];
        assert!(
            t_ms(seat) - t_ms(join) <= 10_000,
            "{world}: seated within 10 s: {seat}"
        );
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
    let answer = screen_where(&lines, |s| {
        s["answer"] == "unauthorized" && s["visible"] == true
    })
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
        let a = Drive::start(
            server.address,
            "visitor",
            &["--drive=idle", "--hold=40", &key],
        );
        a.until("A seated", |lines| {
            lines.iter().any(|l| l.starts_with("SEAT "))
        })
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

        let occupied = screen_where(&b_lines, |s| {
            s["answer"] == "seat_occupied" && s["visible"] == true
        })
        .expect("B is told the seat is occupied");
        assert_eq!(occupied["take_over"], true, "{world}: Take over offered");
        let joins = tagged(&b_lines, "JOIN ");
        assert_eq!(
            joins
                .iter()
                .map(|j| j["take_over"].clone())
                .collect::<Vec<_>>(),
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
            screen_where(&a_lines, |s| s["visible"] == true
                && s["answer"] == "taken_over")
            .is_some(),
            "{world}: A is back at its join screen"
        );
        let a_joins = tagged(&a_lines, "JOIN ");
        assert_eq!(
            a_joins.len(),
            1,
            "{world}: A never joins again: {a_joins:?}"
        );
        // A held its run for 40 s after it was seated; the takeover came within B's first ~20 s,
        // so A watched for at least 10 s after it ended and sent nothing.
        let a_seated = t_ms(&tagged(&a_lines, "SEAT ")[0]);
        assert!(
            t_ms(&ended[0]) - a_seated <= 30_000,
            "{world}: the takeover left 10 s to watch"
        );
    }
}

/// AC-13e-4 — with `--town`, joining a seat the town drives is welcomed `took_over: "hosted"`, the
/// player is told by the Person's displayed name, and the observer is that seat's Person.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_many -- --ignored"]
async fn taking_over_a_town_person() {
    for (world, seat) in [(SOCIAL_CAFE, "bob"), (LAKESIDE, "bruno")] {
        // The Person the World Pack gave this seat at genesis; lakeside's pack is read only through
        // its pack roots, so there the SEAT line's observer is compared with the welcome's alone.
        let person = (world != LAKESIDE).then(|| godot2d::worlds::ids(world)[seat].clone());
        let save = SaveDir::new("2d-many-hosted");
        let mut server = World::start(&args(&hosted(world, &save, &["--town"])), None).await;
        let key = world_key(world);
        let pick = format!("--pick-seat={seat}");
        let (status, lines) = Drive::start_seatless(server.address, &["--drive=join", &pick, &key])
            .finish()
            .await;
        server.kill();
        assert!(
            status.success() && passed(&lines),
            "{world}: the drive passes"
        );
        let welcome = &welcomes(&lines)[0];
        assert_eq!(welcome["took_over"], "hosted", "{world}: {welcome}");
        if let Some(person) = &person {
            assert_eq!(
                welcome["observer"],
                person.as_str(),
                "{world}: the seat's Person"
            );
        }
        let seated = &tagged(&lines, "SEAT ")[0];
        assert_eq!(
            seated["observer"], welcome["observer"],
            "{world}: seated as welcomed"
        );
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

/// The ids of the facts of `event_type` in `seat`'s perceived export of a stopped world's save
/// (`mineworld perceived … --json`, the server's own judgement, `PROTOCOL.md` §5.8), recorded at a
/// world time within `[from, to]`.
fn perceived_ids(
    world: &str,
    save: &SaveDir,
    seat: &str,
    event_type: &str,
    from: u64,
    to: u64,
) -> BTreeSet<String> {
    let (status, out) = support::run_command(&[
        "perceived",
        world,
        "--save",
        save.path(),
        "--person",
        seat,
        "--json",
    ]);
    assert!(status.success(), "mineworld perceived runs");
    out.lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("a PerceivedEvent"))
        .filter(|fact| fact["event_type"] == event_type)
        .filter(|fact| fact["at"].as_u64().is_some_and(|at| at >= from && at <= to))
        .map(|fact| {
            fact["id"]
                .as_str()
                .map_or_else(|| fact["id"].to_string(), str::to_owned)
        })
        .collect()
}

/// The ids on the client's `tag` lines (BUBBLE, NOTICE, SKIPPED) that satisfy `keep`.
fn ids_of(lines: &[String], tag: &str, keep: impl Fn(&Value) -> bool) -> Vec<String> {
    tagged(lines, tag)
        .into_iter()
        .filter(|r| keep(r))
        .map(|r| r["id"].as_str().expect("an id").to_owned())
        .collect()
}

/// AC-13e-6 — A (the visitor) walks up to Bob's player B and says a line: B shows it as a bubble over
/// A, with exactly that text, within 2 s of A's accepted result. Over B's whole run, the `spoke` facts
/// B bubbled are the `spoke` facts its perceived export holds (bar those whose speaker B had not
/// drawn, which it reports as skipped), none twice.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_many -- --ignored"]
async fn bubbles_are_the_facts_heard() {
    const LINE: &str = "hello from A";
    for world in [MARKET_TOWN, SOCIAL_CAFE] {
        let town = godot2d::worlds::ids(world);
        let (visitor, bob) = (town["visitor"].clone(), town["bob"].clone());
        let save = SaveDir::new("2d-many-bubbles");
        let mut server = World::start(&args(&hosted(world, &save, &["--town"])), None).await;
        let key = world_key(world);
        let b = Drive::start(
            server.address,
            "bob",
            &["--drive=listen", "--hold=70", &key],
        );
        b.until("B seated", |lines| {
            lines.iter().any(|l| l.starts_with("SEAT "))
        })
        .await;
        let steps = serde_json::json!([
            { "open": bob }, { "choose": { "walk": true } },
            { "open": bob }, { "choose": { "action_type": "talk" }, "input": LINE },
            { "sleep": 4 }
        ]);
        let (a_status, a_lines) =
            godot2d::worlds::play(server.address, "visitor", "2d-many-a", &steps, &[&key]).await;
        let (b_status, b_lines) = b.finish().await;
        let dropped = server.status().await["events_dropped"]
            .as_u64()
            .unwrap_or(0);
        server.kill();
        assert!(a_status.success() && passed(&a_lines), "{world}: A passes");
        assert!(b_status.success() && passed(&b_lines), "{world}: B passes");

        let talk = tagged(&a_lines, "REQUEST ")
            .into_iter()
            .find(|r| r["request"]["action_type"] == "talk")
            .expect("A talked");
        let answer = tagged(&a_lines, "RESULT ")
            .into_iter()
            .find(|r| r["token"] == talk["token"])
            .expect("answered");
        assert!(
            answer["result"].get("accepted").is_some(),
            "{world}: {answer}"
        );
        let bubble = tagged(&b_lines, "BUBBLE ")
            .into_iter()
            .find(|r| r["who"] == visitor.as_str() && r["text"] == LINE)
            .unwrap_or_else(|| panic!("{world}: B shows A's line over A"));
        let after = bubble["unix_ms"].as_i64().expect("unix_ms")
            - answer["unix_ms"].as_i64().expect("unix_ms");
        assert!(
            after <= 2_000,
            "{world}: within 2 s of the accepted result: {after} ms"
        );

        let bubbled = ids_of(&b_lines, "BUBBLE ", |_| true);
        let unique: BTreeSet<String> = bubbled.iter().cloned().collect();
        assert_eq!(unique.len(), bubbled.len(), "{world}: no fact twice");
        if dropped > 0 {
            eprintln!("{world}: INCONCLUSIVE set check: /status events_dropped = {dropped}");
            continue;
        }
        let span = &evidence(&b_lines, "listened")[0];
        let heard = perceived_ids(
            world,
            &save,
            "bob",
            "spoke",
            span["first_at"].as_u64().expect("first_at"),
            span["last_at"].as_u64().expect("last_at"),
        );
        let undrawn: BTreeSet<String> = ids_of(&b_lines, "SKIPPED ", |r| {
            r["event_type"] == "spoke" && r["why"] == "undrawn"
        })
        .into_iter()
        .collect();
        let shown_or_skipped: BTreeSet<String> = unique.union(&undrawn).cloned().collect();
        assert!(
            unique.is_subset(&heard),
            "{world}: every bubble is a fact B perceived: {unique:?} vs {heard:?}"
        );
        assert!(
            heard.is_subset(&shown_or_skipped),
            "{world}: every spoke B perceived is bubbled or its speaker undrawn: {heard:?} vs {shown_or_skipped:?}"
        );
    }
}

/// AC-13e-7 — a group activity starts (A invites, C accepts, as 13b did): each of them, being where
/// it started, shows one notice for it, worded from `ui.event.group-activity-started` and naming both
/// members — as many as its perceived export holds, each once.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_many -- --ignored"]
async fn an_activity_starting_is_noticed_once() {
    for world in [SOCIAL_CAFE, MARKET_TOWN] {
        let town = godot2d::worlds::ids(world);
        let (visitor, bob) = (town["visitor"].clone(), town["bob"].clone());
        let save = SaveDir::new("2d-many-notice");
        let mut server = World::start(&args(&hosted(world, &save, &[])), None).await;
        let key = world_key(world);
        let sync = mineworld_test_support::scratch!(empty "2d-many-notice-sync");
        let sync_arg = format!("--sync={}", sync.to_str().expect("UTF-8"));
        let a_steps = serde_json::json!([
            { "open": bob }, { "choose": { "walk": true } },
            { "open": bob }, { "choose": { "action_type": "invite" }, "input": "coffee" },
            { "mark": "invited" }, { "after": "accepted" }, { "sleep": 3 }, { "report": true }
        ]);
        let c_steps = serde_json::json!([
            { "after": "invited" },
            { "await": { "offered": { "action_type": "accept-invitation", "target": visitor } } },
            { "open": visitor }, { "choose": { "action_type": "accept-invitation" } },
            { "mark": "accepted" }, { "sleep": 3 }, { "report": true }
        ]);
        let (a, _a) = godot2d::worlds::start(
            server.address,
            "visitor",
            "2d-many-na",
            &a_steps,
            &[&sync_arg, &key],
        );
        let (c, _c) = godot2d::worlds::start(
            server.address,
            "bob",
            "2d-many-nc",
            &c_steps,
            &[&sync_arg, &key],
        );
        let (a_status, a_lines) = a.finish().await;
        let (c_status, c_lines) = c.finish().await;
        server.kill();
        assert!(a_status.success() && passed(&a_lines), "{world}: A passes");
        assert!(c_status.success() && passed(&c_lines), "{world}: C passes");

        for (seat, lines) in [("visitor", &a_lines), ("bob", &c_lines)] {
            let started = perceived_ids(world, &save, seat, "group-activity-started", 0, u64::MAX);
            assert!(!started.is_empty(), "{world}: {seat} perceived the start");
            let of_started: Vec<Value> = tagged(lines, "NOTICE ")
                .into_iter()
                .filter(|n| n["event_type"] == "group-activity-started")
                .collect();
            let ids: BTreeSet<String> = of_started
                .iter()
                .map(|n| n["id"].as_str().expect("id").to_owned())
                .collect();
            assert_eq!(ids.len(), of_started.len(), "{world}: {seat}: once each");
            assert_eq!(ids, started, "{world}: {seat}: the starts it perceived");
            let names: Vec<String> = [&visitor, &bob]
                .iter()
                .map(|id| label_in(lines, id))
                .collect();
            for notice in &of_started {
                let text = notice["text"].as_str().expect("text");
                for name in &names {
                    assert!(text.contains(name.as_str()), "{world}: {text} names {name}");
                }
            }
        }
    }
}

/// The label the client drew for a person, from its `SHOWN` reports.
fn label_in(lines: &[String], person: &str) -> String {
    tagged(lines, "SHOWN ")
        .iter()
        .flat_map(|s| s["people"].as_array().cloned().unwrap_or_default())
        .find(|p| p["id"] == person)
        .and_then(|p| p["label"].as_str().map(str::to_owned))
        .unwrap_or_else(|| panic!("{person} was never shown"))
}

/// AC-13e-8 — a living town: hosted with `--town` at the world's own time scale, at least one person
/// this player does not drive is drawn at least 1 m from where it was first drawn within 120 s, and
/// at the end every person is drawn where the server says (AC-W2, the drive's own check).
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_many -- --ignored"]
async fn the_town_lives_without_us() {
    for world in [MARKET_TOWN, SOCIAL_CAFE] {
        let save = SaveDir::new("2d-many-town");
        let mut server = World::start(&args(&hosted(world, &save, &["--town"])), None).await;
        let key = world_key(world);
        let (status, lines) = Drive::start(
            server.address,
            "visitor",
            &["--drive=town", "--hold=120", &key],
        )
        .finish()
        .await;
        server.kill();
        assert!(
            status.success() && passed(&lines),
            "{world}: the drive passes"
        );
        let me = welcomes(&lines)[0]["observer"]
            .as_str()
            .expect("observer")
            .to_owned();
        let mut first: std::collections::BTreeMap<String, (i64, i64)> = Default::default();
        let mut moved = 0_i64;
        for shown in tagged(&lines, "SHOWN ") {
            for person in shown["people"].as_array().cloned().unwrap_or_default() {
                let id = person["id"].as_str().unwrap_or("").to_owned();
                let (Some(x), Some(y)) =
                    (person["local"]["x"].as_i64(), person["local"]["y"].as_i64())
                else {
                    continue;
                };
                if id == me {
                    continue;
                }
                let start = *first.entry(id).or_insert((x, y));
                let d = ((x - start.0) as f64).hypot((y - start.1) as f64) as i64;
                moved = moved.max(d);
            }
        }
        assert!(
            moved >= 1_000,
            "{world}: somebody walked at least 1 m on their own: {moved} mm"
        );
    }
}
