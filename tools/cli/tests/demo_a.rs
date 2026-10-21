//! Demo A's proof (S12 PR 13f, `ARC-346`): the real 2D client plays `docs/MVP.md` §7.1 on two worlds,
//! market-town and social-cafe, and reports what it draws so the parity test (S14 PR 16e,
//! `shown_parity.rs`) can trust it.
//!
//! Every test here starts Godot, so every test is `#[ignore]`d: `cargo test` reports them as ignored,
//! never as passed, on a machine without Godot (a test not run is not a pass). Run them with
//!
//! ```text
//! cargo test -p mineworld-cli --test demo_a -- --ignored --test-threads=1
//! ```
//!
//! Two kinds of test, each on both worlds: the report (`*_report_is_what_is_drawn`, AC-f-4/5/6) and the
//! Demo A story (`*_demo_a_story`, §4.1 steps 1–14, AC-f-1/2/3/6).
//!
//! No test names a world's person, place or item: every choice is read from what the world disclosed
//! (the client's own `SHOWN` report of the drawn scene, the observation's offers), and every assertion's
//! oracle is the save, `GET /status`, or what the world listed — never the client's `[PASS]` alone.

mod godot2d;
mod headless;
mod support;

use std::collections::BTreeSet;
use std::time::Duration;

use godot2d::worlds::{facts, id, ids, panel, start};
use godot2d::{Drive, MARKET_TOWN, World, evidence, passed, places, tagged};
use serde_json::{Value, json};
use support::SaveDir;

const SOCIAL_CAFE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/social-cafe");
/// The seat the client takes: both worlds offer it (the `AC-15` player seats).
const SEAT: &str = "visitor";

fn owned(arguments: &[String]) -> Vec<&str> {
    arguments.iter().map(String::as_str).collect()
}

/// A doorway as `SHOWN` lists it: `(from, to, here, there)`.
fn doorway(d: &Value) -> (String, String, String, String) {
    let s = |k: &str| d[k].to_string();
    (s("from"), s("to"), s("here"), s("there"))
}

/// The steps of one report: walk into the place tagged `cafe` (a disclosed category both worlds have;
/// `bodies.gd` derives every waypoint from the passages), wait until the drawn objects are what the
/// newest observation lists, then report.
fn report_steps() -> Value {
    json!([{ "walk_into": "cafe" }, { "objects": {} }, { "report": true }, { "open": "self" }])
}

/// One run of the report on `world` with `extra` flags: its exit, lines, and `GET /status` read while
/// the client was still seated.
async fn report(world: &World, name: &str, extra: &[&str]) -> (bool, Vec<String>, Value) {
    let (drive, _dir) = start(world.address, SEAT, name, &report_steps(), extra);
    drive
        .until("its report", |lines| {
            lines.iter().any(|l| l.starts_with("MENU "))
                || lines.iter().any(|l| l.starts_with("drive complete"))
        })
        .await;
    let status = world.status().await;
    let (exit, lines) = drive.finish().await;
    (exit.success() && passed(&lines), lines, status)
}

/// AC-f-4, AC-f-5, AC-f-6 on one world (PR 13f §5).
async fn the_report_is_what_is_drawn(pack: &str, tag: &str) {
    let genesis: BTreeSet<String> = ids(pack).into_values().collect();
    let save = SaveDir::new(&format!("demo-a-report-{tag}"));
    // `--hold 0`: each run below takes the same seat the run before it just left (S11-B holds a
    // dropped seat for its resume, 30 s by default).
    let arguments = ["server", pack, "--save", save.path(), "--hold", "0"].map(str::to_owned);
    let world = World::start(&owned(&arguments), None).await;

    let (ok, lines, status) = report(&world, &format!("demo-a-{tag}-1"), &[]).await;
    assert!(ok, "{tag}: the drive passes:\n{}", lines.join("\n"));
    let shown = tagged(&lines, "SHOWN ").pop().expect("a SHOWN report");

    // AC-f-4: every doorway once; the drawn doorways of the place stood in are listed.
    let doorways: Vec<_> = shown["doorways"]
        .as_array()
        .expect("doorways")
        .iter()
        .map(doorway)
        .collect();
    let distinct: BTreeSet<_> = doorways.iter().cloned().collect();
    assert_eq!(
        distinct.len(),
        doorways.len(),
        "{tag}: each doorway once: {doorways:?}"
    );
    let here = shown["place"].to_string();
    let out = doorways
        .iter()
        .find(|d| d.0 == here)
        .unwrap_or_else(|| panic!("{tag}: a doorway of {here} is drawn: {doorways:?}"))
        .clone();

    // AC-f-5: the objects drawn are the objects the newest observation lists, by id.
    let objects = evidence(&lines, "objects")
        .pop()
        .expect("an objects report");
    let listed: BTreeSet<String> = objects["listed"]
        .as_array()
        .expect("listed")
        .iter()
        .map(|v| v.as_str().expect("an id").to_owned())
        .collect();
    let drawn: BTreeSet<String> = shown["objects"]
        .as_array()
        .expect("objects")
        .iter()
        .map(|o| o["id"].as_str().expect("an id").to_owned())
        .collect();
    assert_eq!(drawn, listed, "{tag}: SHOWN.objects is the listing");
    // ... and the observer's own offers are the frame's, in its order, each with its verdict.
    let observer = tagged(&lines, "MENU ")
        .into_iter()
        .find(|m| m["self"] == true)
        .expect("the observer's own menu");
    let offered: Vec<(Value, Value, bool)> = observer["offered"]
        .as_array()
        .expect("offered")
        .iter()
        .map(|a| {
            (
                a["action_type"].clone(),
                a["target"].clone(),
                a["available"] == true,
            )
        })
        .collect();
    let mine: Vec<(Value, Value, bool)> = shown["affordances"]
        .as_array()
        .expect("affordances")
        .iter()
        .filter(|a| a["subject"] == observer["subject"])
        .map(|a| {
            (
                a["action_type"].clone(),
                a["target"].clone(),
                a["available"] == true,
            )
        })
        .collect();
    assert_eq!(
        mine, offered,
        "{tag}: SHOWN.affordances lists my offers as offered"
    );
    assert!(!mine.is_empty(), "{tag}: the observer is offered something");
    for a in shown["affordances"].as_array().expect("affordances") {
        if a["subject"] != observer["subject"] {
            assert_eq!(
                a["target"], a["subject"],
                "{tag}: a person's menu is about them: {a}"
            );
        }
    }

    // AC-f-6: the instance is the running world's, the revision no later than its head, every person
    // one the world was made with.
    let identity = tagged(&lines, "IDENTITY_WORLD ")
        .pop()
        .expect("an IDENTITY_WORLD line");
    assert_eq!(
        identity["instance"], status["instance"],
        "{tag}: one world: {status}"
    );
    assert!(
        identity["revision"]
            .as_f64()
            .is_some_and(|r| r <= status["revision"].as_f64().unwrap_or(f64::NEG_INFINITY)),
        "{tag}: a revision the world reached: {identity} vs {status}"
    );
    let people = identity["people"].as_object().expect("people");
    assert!(!people.is_empty(), "{tag}: people are listed");
    for (id, name) in people {
        assert!(genesis.contains(id), "{tag}: {id} is a person of the world");
        assert!(name.is_string(), "{tag}: {id}'s disclosed name");
    }

    // AC-f-4, the mutation: the same run with that doorway dropped lists every other doorway, not it.
    let from = out.0.trim_matches('"').to_owned();
    let to = out.1.trim_matches('"').to_owned();
    let flag = format!("--drop-doorway={from},{to}");
    let (ok, dropped, _) = report(&world, &format!("demo-a-{tag}-2"), &[&flag]).await;
    assert!(
        ok,
        "{tag}: the drive passes with {flag}:\n{}",
        dropped.join("\n")
    );
    let after: BTreeSet<_> = tagged(&dropped, "SHOWN ").pop().expect("a SHOWN report")["doorways"]
        .as_array()
        .expect("doorways")
        .iter()
        .map(doorway)
        .collect();
    let expected: BTreeSet<_> = distinct
        .iter()
        .filter(|d| !(d.0 == out.0 && d.1 == out.1))
        .cloned()
        .collect();
    assert_eq!(after, expected, "{tag}: exactly {from} -> {to} is gone");

    // A doorway nobody disclosed fails the drive by name.
    let (ok, lines, _) = report(&world, &format!("demo-a-{tag}-3"), &["--drop-doorway=0,0"]).await;
    assert!(!ok, "{tag}: an unknown doorway fails the drive");
    assert!(
        lines
            .iter()
            .any(|l| l.contains("[FAIL]") && l.contains("drop-doorway") && l.contains("0 -> 0")),
        "{tag}: the failure names it"
    );
}

#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test demo_a -- --ignored"]
async fn market_town_report_is_what_is_drawn() {
    the_report_is_what_is_drawn(MARKET_TOWN, "market-town").await;
}

#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test demo_a -- --ignored"]
async fn social_cafe_report_is_what_is_drawn() {
    the_report_is_what_is_drawn(SOCIAL_CAFE, "social-cafe").await;
}

// ── The Demo A story (PR 13f §4.1, steps 1–14) ────────────────────────────────────────────────────

/// What A says, and the activity A proposes (a slug the `group-activity` pack validates, never a
/// world's word).
const LINE: &str = "Hello! Busy today?";
const KIND: &str = "coffee";
/// Step 9: how far B must draw A moving — one person's radius (realistic default, 250 mm).
const WALKED_MM: f64 = 250.0;
/// Step 12: how far a relaunched client may draw its body from where the world put it — 13d's
/// reconciliation bound (`client_2d_bodies.rs`), wider than the 0.11 m town drift (F-13e-1).
const RECONCILE_MM: f64 = 150.0;

/// Step 7: the item offer each world's menu holds (AC-f-2), and how its consequence is read.
#[derive(Clone, Copy)]
enum Use {
    /// A purchase from the place's listing: the save records the listed price paid.
    Buy,
    /// A kick of the nearest loose object: the save records it moved, by A.
    Kick,
}

impl Use {
    fn action_type(self) -> &'static str {
        match self {
            Use::Buy => "buy",
            Use::Kick => "kick",
        }
    }

    fn steps(self) -> Vec<Value> {
        match self {
            Use::Buy => vec![
                json!({ "open": "self" }),
                json!({ "choose": { "action_type": "buy" } }),
            ],
            Use::Kick => vec![
                json!({ "objects": {} }),
                json!({ "walk_near": "nearest" }),
                json!({ "open_object": "nearest" }),
                json!({ "choose": { "action_type": "kick" } }),
                json!({ "objects": { "moved": true } }),
            ],
        }
    }
}

/// The World Pack's directory name, which the client is told for its far field.
fn world_key(pack: &str) -> String {
    format!(
        "--world-key={}",
        std::path::Path::new(pack)
            .file_name()
            .and_then(|n| n.to_str())
            .expect("a world directory")
    )
}

/// The answer to the request sent with `token`.
fn result_of(lines: &[String], token: &Value) -> Value {
    tagged(lines, "RESULT ")
        .into_iter()
        .find(|r| r["token"] == *token)
        .unwrap_or_else(|| panic!("request {token} was never answered"))
}

/// The entity the first typed reference of a request's payload names (the item bought, the object
/// kicked).
fn about(request: &Value) -> String {
    request["payload"]["payload"]
        .as_object()
        .and_then(|fields| fields.values().find(|v| v.get("entity").is_some()))
        .map(id)
        .unwrap_or_default()
}

/// Where the client's `SHOWN` reports drew `person`: `(place, x, y)` in that place's frame.
fn drawn_at(lines: &[String], person: &str) -> Vec<(String, f64, f64)> {
    tagged(lines, "SHOWN ")
        .iter()
        .filter_map(|s| {
            let p = s["people"].as_array()?.iter().find(|p| p["id"] == person)?;
            Some((
                s["place"].to_string(),
                p["local"]["x"].as_f64()?,
                p["local"]["y"].as_f64()?,
            ))
        })
        .collect()
}

/// The farthest apart two reports drew `person` in one place, in millimetres.
fn farthest(lines: &[String], person: &str) -> f64 {
    let at = drawn_at(lines, person);
    let mut most = 0.0_f64;
    for (i, a) in at.iter().enumerate() {
        for b in &at[i + 1..] {
            if a.0 == b.0 {
                most = most.max((a.1 - b.1).hypot(a.2 - b.2));
            }
        }
    }
    most
}

/// The head revision `mineworld inspect` reads from a stopped world's save (Q13f-3).
fn inspected_head(save: &SaveDir) -> u64 {
    let (status, report) = support::run_command(&["inspect", save.path()]);
    assert!(status.success(), "the save inspects: {report}");
    report
        .lines()
        .find_map(|line| {
            let rest = line.trim_start().strip_prefix("head")?;
            rest.split_whitespace()
                .skip_while(|w| *w != "revision")
                .nth(1)?
                .parse()
                .ok()
        })
        .unwrap_or_else(|| panic!("no head revision in: {report}"))
}

/// The disclosed name of `person` in the newest `IDENTITY_WORLD` that lists them, or "".
fn named(lines: &[String], person: &str) -> String {
    tagged(lines, "IDENTITY_WORLD ")
        .iter()
        .rev()
        .find_map(|i| i["people"][person].as_str().map(str::to_owned))
        .unwrap_or_default()
}

/// Whether a client drew `person` with the participation marker of `KIND`.
fn marked(lines: &[String], person: &str) -> bool {
    tagged(lines, "SHOWN ").iter().any(|s| {
        s["people"].as_array().is_some_and(|people| {
            people
                .iter()
                .any(|p| p["id"] == person && p["activity"] == KIND)
        })
    })
}

/// The Demo A story on one world (§4.1, AC-f-1/2/3/6). Seats come from `GET /status`: the first two
/// for the players A and B. Asked before the story, the world's first observation for A names A's
/// place (and its first tag, which A walks back into) and the seat Persons there; the first of them
/// becomes a reactive agent A talks to (it answers and stays where it is), and the next seat is the
/// takeover. Every person, object and offer is chosen from what the world disclosed; every oracle is
/// the save, `/status` or another client.
#[allow(clippy::too_many_lines)]
async fn demo_a_story(pack: &str, tag: &str, item: Use) {
    let genesis = ids(pack);
    let persons: BTreeSet<String> = genesis.values().cloned().collect();
    let save = SaveDir::new(&format!("demo-a-story-{tag}"));
    // The seats the world offers, in its order, from a world started only to be asked.
    let mut probe = World::start(&["server", pack], None).await;
    let seats: Vec<String> = probe.status().await["seats"]
        .as_array()
        .expect("seats")
        .iter()
        .map(|s| s.as_str().expect("a seat").to_owned())
        .collect();
    assert!(
        seats.len() >= 4,
        "{tag}: two player seats, an agent's and one the town drives: {seats:?}"
    );
    let (seat_a, seat_b) = (seats[0].clone(), seats[1].clone());
    let mut asker = support::Client::connect(probe.address).await;
    asker.join(&seat_a).await;
    let first = serde_json::to_value(asker.observation().await).expect("an observation");
    asker.disconnect().await;
    probe.kill();
    let home = id(&first["self_location"]["place"]);
    let entities = first["entities"].as_array().cloned().unwrap_or_default();
    let home_tag = entities
        .iter()
        .find(|e| id(&e["id"]) == home)
        .and_then(|e| e["tags"].as_array()?.first()?.as_str().map(str::to_owned))
        .unwrap_or_else(|| panic!("{tag}: A's place {home} discloses a tag"));
    let seat_of: std::collections::BTreeMap<String, String> = seats
        .iter()
        .filter_map(|s| genesis.get(s).map(|person| (person.clone(), s.clone())))
        .collect();
    let seat_agent = entities
        .iter()
        .filter(|e| e["entity_type"] == "person" && id(&e["location"]["place"]) == home)
        .filter_map(|e| seat_of.get(&id(&e["id"])))
        .find(|s| **s != seat_a && **s != seat_b)
        .cloned()
        .unwrap_or_else(|| panic!("{tag}: a seat Person shares A's first place {home}"));
    let seat_c = seats
        .iter()
        .find(|s| ![&seat_a, &seat_b, &seat_agent].contains(s))
        .expect("a seat for the takeover")
        .clone();
    // Step 1. `--town`: an agent drives every seat no client holds (R-S10-1). `--hold 0`: a seat
    // whose client dies goes back to the town at once, so the relaunched client takes it back.
    let arguments = [
        "server",
        pack,
        "--town",
        "--save",
        save.path(),
        "--hold",
        "0",
        "--agent",
        seat_agent.as_str(),
    ]
    .map(str::to_owned);
    let mut world = World::start(&owned(&arguments), None).await;
    let key = world_key(pack);
    let status = world.status().await;
    let instance = status["instance"].clone();
    // The Person the reactive agent drives: A talks to them (it answers talk, R-S10-1).
    let agent = genesis[&seat_agent].clone();
    let sync = mineworld_test_support::scratch!(empty format!("demo-a-sync-{tag}"));
    let sync_arg = format!("--sync={}", sync.to_str().expect("UTF-8"));
    let part = json!({ "await": { "component": "participation", "of": "self", "present": true } });

    // Step 2: B joins its seat and waits in the café-tagged place for A.
    let b_steps = json!([
        { "walk_into": home_tag.clone() }, { "report": true }, { "mark": "b-in" },
        { "after": "talked" }, { "sleep": 3 }, { "report": true },
        { "after": "invited" }, { "await": { "offered": { "action_type": "accept-invitation" } } },
        { "open": { "offered": "accept-invitation" } },
        { "choose": { "action_type": "accept-invitation" } },
        part.clone(), { "report": true }, { "mark": "accepted" },
        { "after": "a-done" }, { "report": true }
    ]);
    let (b, _b) = start(
        world.address,
        &seat_b,
        &format!("demo-a-{tag}-b"),
        &b_steps,
        &[&sync_arg, &key],
    );
    b.until("B seated", |lines| {
        lines.iter().any(|l| l.starts_with("SEAT "))
    })
    .await;
    let b_id = tagged(&b.lines(), "SEAT ")[0]["observer"]
        .as_str()
        .expect("B's observer")
        .to_owned();

    // Steps 3–8 for A: out of its place and back in, talk to someone the town drives, use an item,
    // invite B.
    let mut a_steps = vec![
        json!({ "walk_into": "street" }),
        json!({ "report": true }),
        json!({ "walk_into": home_tag.clone() }),
        json!({ "report": true }),
        json!({ "after": "b-in" }),
        json!({ "open": { "offered": "talk", "among": [agent.clone()] } }),
        json!({ "choose": { "walk": true } }),
        json!({ "open": "last" }),
        json!({ "choose": { "action_type": "talk" }, "input": LINE }),
        json!({ "sleep": 6 }),
        json!({ "report": true }),
        json!({ "mark": "talked" }),
    ];
    a_steps.extend(item.steps());
    a_steps.extend([
        json!({ "report": true }),
        json!({ "open": b_id.clone() }),
        json!({ "choose": { "walk": true } }),
        json!({ "open": b_id.clone() }),
        json!({ "choose": { "action_type": "invite" }, "input": KIND }),
        json!({ "mark": "invited" }),
        json!({ "after": "accepted" }),
        part,
        json!({ "report": true }),
        json!({ "mark": "a-done" }),
        json!({ "sleep": 2 }),
    ]);
    let (a, _a) = start(
        world.address,
        &seat_a,
        &format!("demo-a-{tag}-a"),
        &Value::Array(a_steps),
        &[&sync_arg, &key],
    );
    let (a_exit, a_lines) = a.finish().await;
    let (b_exit, b_lines) = b.finish().await;
    assert!(
        a_exit.success() && passed(&a_lines),
        "{tag}: A's drive passes:\n{}",
        a_lines.join("\n")
    );
    assert!(
        b_exit.success() && passed(&b_lines),
        "{tag}: B's drive passes:\n{}",
        b_lines.join("\n")
    );
    let a_id = evidence(&a_lines, "welcome")[0]["observer"]
        .as_str()
        .expect("A's observer")
        .to_owned();

    // Step 3: A stood in at least two places.
    let visited: BTreeSet<String> = places(&a_lines).into_iter().collect();
    assert!(
        visited.len() >= 2,
        "{tag}: A walked into another place: {visited:?}"
    );
    // Step 4: A drew at least two people, each labelled with the name the world disclosed.
    let mut crowd = 0;
    for shown in tagged(&a_lines, "SHOWN ") {
        let people = shown["people"].as_array().cloned().unwrap_or_default();
        crowd = crowd.max(people.len());
        for p in people {
            let who = p["id"].as_str().unwrap_or_default().to_owned();
            let name = named(&a_lines, &who);
            if !name.is_empty() {
                assert_eq!(
                    p["label"],
                    name.as_str(),
                    "{tag}: {who} labelled by the disclosed name"
                );
            }
        }
    }
    assert!(crowd >= 2, "{tag}: A drew at least two people: {crowd}");
    // Step 5: the talk was accepted (the save's `spoke` facts are read below).
    let chosen = tagged(&a_lines, "CHOSE ")
        .into_iter()
        .find(|c| c["offered"] == "talk")
        .expect("A chose someone to talk to");
    let x = chosen["subject"].as_str().expect("an id").to_owned();
    let talk = tagged(&a_lines, "REQUEST ")
        .into_iter()
        .rev()
        .find(|r| r["request"]["action_type"] == "talk")
        .expect("A talked");
    let answer = result_of(&a_lines, &talk["token"]);
    assert!(
        answer["result"].get("accepted").is_some(),
        "{tag}: the talk is accepted: {answer}"
    );
    // Step 7: the item offer was sent and accepted.
    let used = tagged(&a_lines, "REQUEST ")
        .into_iter()
        .find(|r| r["request"]["action_type"] == item.action_type())
        .unwrap_or_else(|| panic!("{tag}: A sent a {}", item.action_type()));
    let used_answer = result_of(&a_lines, &used["token"]);
    assert!(
        used_answer["result"].get("accepted").is_some(),
        "{tag}: the {} is accepted: {used_answer}",
        item.action_type()
    );
    let thing = about(&used["request"]);
    // Step 8: both clients drew both players with the participation marker.
    for (who, lines) in [("A", &a_lines), ("B", &b_lines)] {
        assert!(
            marked(lines, &a_id) && marked(lines, &b_id),
            "{tag}: {who} drew the marker on A and B"
        );
    }
    // Step 9: B saw A walk, and showed A's line as a bubble over A.
    assert!(
        farthest(&b_lines, &a_id) >= WALKED_MM,
        "{tag}: B drew A walking"
    );
    assert!(
        tagged(&b_lines, "BUBBLE ")
            .iter()
            .any(|r| r["who"] == a_id.as_str() && r["text"] == LINE),
        "{tag}: B showed A's line over A"
    );

    // Step 11: a third client takes over a seat the town drives, and is told so by name.
    let pick = format!("--pick-seat={seat_c}");
    let (c_exit, c_lines) = Drive::start_seatless(world.address, &["--drive=join", &pick, &key])
        .finish()
        .await;
    assert!(
        c_exit.success() && passed(&c_lines),
        "{tag}: the takeover drive passes"
    );
    let taken = &evidence(&c_lines, "welcome")[0];
    assert_eq!(taken["took_over"], "hosted", "{tag}: {taken}");
    assert_eq!(
        taken["observer"],
        genesis[&seat_c].as_str(),
        "{tag}: the seat's Person"
    );
    let taken_name = tagged(&c_lines, "SEAT ")[0]["name"]
        .as_str()
        .expect("a name")
        .to_owned();
    assert!(
        tagged(&c_lines, "TOAST ")
            .iter()
            .any(|t| t["kind"] == "seat"
                && t["took_over"] == "hosted"
                && t["text"]
                    .as_str()
                    .is_some_and(|text| text.contains(&taken_name))),
        "{tag}: told by name"
    );

    // Step 12: A is SIGKILLed mid-walk; the world goes on; A comes back, drawn where the world put it.
    let mut a2 = Drive::start(
        world.address,
        &seat_a,
        &["--drive=street", "--strides=20", "--hold=40", &key],
    );
    a2.until("an accepted stride", |lines| {
        lines
            .iter()
            .any(|l| l.starts_with("RESULT ") && l.contains("accepted"))
    })
    .await;
    assert!(!a2.kill().success(), "{tag}: A died, not exited");
    let before = world.status().await;
    tokio::time::sleep(Duration::from_secs(5)).await;
    let after = world.status().await;
    assert!(
        after["revision"].as_u64() > before["revision"].as_u64(),
        "{tag}: the world went on while A was dead: {before} → {after}"
    );
    let a3 = Drive::start(world.address, &seat_a, &["--drive=idle", "--hold=60", &key]);
    let b2 = Drive::start(world.address, &seat_b, &["--drive=idle", "--hold=60", &key]);
    a3.until("A seated", |lines| {
        lines.iter().any(|l| l.starts_with("SEAT "))
    })
    .await;
    b2.until("B seated", |lines| {
        lines.iter().any(|l| l.starts_with("SEAT "))
    })
    .await;
    let back = evidence(&a3.lines(), "welcome")[0].clone();
    assert_eq!(
        back["instance"], instance,
        "{tag}: A is back in the same world"
    );
    assert!(
        back["revision"]
            .as_f64()
            .is_some_and(|r| r >= after["revision"].as_f64().unwrap_or(f64::INFINITY)),
        "{tag}: A is told a revision no earlier than the world's: {back} vs {after}"
    );

    // Step 13: the server is SIGKILLed and restarted on the same save; both clients come back.
    let marks = (a3.lines().len(), b2.lines().len());
    world.kill();
    let kept = facts(&save);
    let head = inspected_head(&save);
    tokio::time::sleep(Duration::from_secs(2)).await;
    world.restart().await;
    a3.until("A seated again", |lines| {
        lines[marks.0..]
            .iter()
            .any(|l| l.starts_with("STATE seated"))
    })
    .await;
    b2.until("B seated again", |lines| {
        lines[marks.1..]
            .iter()
            .any(|l| l.starts_with("STATE seated"))
    })
    .await;
    let (a3_exit, a3_lines) = a3.finish().await;
    let (b2_exit, b2_lines) = b2.finish().await;
    let status_after = world.status().await;
    world.kill();
    assert!(a3_exit.success() && passed(&a3_lines), "{tag}: A passes");
    assert!(b2_exit.success() && passed(&b2_lines), "{tag}: B passes");
    let all = facts(&save);
    assert!(
        all.len() >= kept.len() && all[..kept.len()] == kept[..],
        "{tag}: the facts before the kill are a prefix of the facts after"
    );
    for (who, lines, mark) in [("A", &a3_lines, marks.0), ("B", &b2_lines, marks.1)] {
        let again = evidence(&lines[mark..], "welcome");
        let welcome = again
            .first()
            .unwrap_or_else(|| panic!("{tag}: {who} is welcomed again"));
        assert_eq!(
            welcome["instance"], instance,
            "{tag}: {who}: the same instance"
        );
        assert!(
            welcome["revision"]
                .as_f64()
                .is_some_and(|r| r >= head as f64),
            "{tag}: {who}: told a revision ≥ the save's head {head}: {welcome}"
        );
    }
    // Step 12's last clause: the relaunched A is drawn where the world put it.
    let me = evidence(&a3_lines, "self").pop().expect("A reports itself");
    let axis = |v: &Value, k: &str| v[k].as_f64().unwrap_or(f64::NAN);
    let gap = (axis(&me["drawn_local"], "x") - axis(&me["local"], "x"))
        .hypot(axis(&me["drawn_local"], "y") - axis(&me["local"], "y"));
    assert!(
        gap <= RECONCILE_MM,
        "{tag}: A is drawn {gap} mm from where the world has it"
    );

    // Step 14: A's IDENTITY_WORLD names the running world and the world's own people.
    let identity = tagged(&a3_lines, "IDENTITY_WORLD ")
        .pop()
        .expect("an IDENTITY_WORLD line");
    assert_eq!(identity["instance"], status_after["instance"], "{tag}");
    assert!(
        identity["revision"].as_f64().is_some_and(|r| r
            <= status_after["revision"]
                .as_f64()
                .unwrap_or(f64::NEG_INFINITY)),
        "{tag}: {identity} vs {status_after}"
    );
    for listed in identity["people"].as_object().expect("people").keys() {
        assert!(
            persons.contains(listed),
            "{tag}: {listed} is a person of the world"
        );
    }

    // The save, for steps 5–8: A's line to X and X's answer, in A's history; the acquaintance in A's
    // panel; the item's consequence; the invitation accepted.
    let spoke: Vec<&Value> = all
        .iter()
        .filter(|(t, _)| t == "spoke")
        .map(|(_, p)| p)
        .collect();
    let said = spoke
        .iter()
        .position(|p| {
            id(&p["speaker"]) == a_id && id(&p["listener"]) == x && p["utterance"] == LINE
        })
        .unwrap_or_else(|| panic!("{tag}: the save holds A's line to {x}"));
    let reply = spoke[said + 1..]
        .iter()
        .find(|p| id(&p["speaker"]) == x && id(&p["listener"]) == a_id)
        .and_then(|p| p["utterance"].as_str())
        .unwrap_or_else(|| panic!("{tag}: {x} answers A"))
        .to_owned();
    assert!(
        tagged(&a_lines, "PANELS ").iter().any(|p| {
            panel(p, "conversation-history").is_some_and(|h| {
                h["rows"].as_array().is_some_and(|rows| {
                    rows.iter()
                        .any(|r| r.as_str().is_some_and(|r| r.contains(&reply)))
                })
            })
        }),
        "{tag}: X's answer reached A's history: {reply}"
    );
    let x_name = named(&a_lines, &x);
    assert!(
        all.iter().any(|(t, p)| t == "became-acquainted"
            && [id(&p["person"]), id(&p["counterpart"])].contains(&a_id)
            && [id(&p["person"]), id(&p["counterpart"])].contains(&x)),
        "{tag}: the save says A and {x} became acquainted"
    );
    assert!(
        tagged(&a_lines, "PANELS ").iter().any(|p| {
            panel(p, "acquaintances").is_some_and(|a| {
                a["rows"]
                    .as_array()
                    .is_some_and(|rows| rows.iter().any(|r| r == x_name.as_str()))
            })
        }),
        "{tag}: A's acquaintances panel lists {x_name}"
    );
    // Step 10: somebody no client drove walked on their own while the story ran (the town's agents).
    let players = [
        a_id.as_str(),
        b_id.as_str(),
        taken["observer"].as_str().unwrap_or(""),
    ];
    assert!(
        all.iter()
            .any(|(t, p)| t == "walk-started" && !players.contains(&id(&p["person"]).as_str())),
        "{tag}: an agent-driven person walked on their own"
    );
    match item {
        Use::Buy => {
            let price = tagged(&a_lines, "PANELS ")
                .iter()
                .find_map(|p| {
                    panel(p, "shop")?["payload"]["listed"]
                        .as_array()?
                        .iter()
                        .find(|l| id(&l["item"]) == thing)?["price"]
                        .as_f64()
                })
                .unwrap_or_else(|| panic!("{tag}: the listing names {thing}'s price"))
                as u64;
            assert!(
                all.iter().any(|(t, p)| t == "money-transferred"
                    && id(&p["from"]) == a_id
                    && p["amount"].as_u64() == Some(price)),
                "{tag}: A paid the listed price {price} for {thing}"
            );
        }
        Use::Kick => {
            assert!(
                all.iter().any(|(t, p)| t == "object-moved"
                    && id(&p["object"]) == thing
                    && id(&p["by"]) == a_id
                    && p["from"] != p["to"]),
                "{tag}: the save records {thing} moved by A"
            );
        }
    }
    assert!(
        all.iter().any(|(t, p)| t == "invitation-accepted"
            && id(&p["inviter"]) == a_id
            && id(&p["invitee"]) == b_id
            && p["kind"] == KIND),
        "{tag}: the save records B accepting A's invitation"
    );
}

#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test demo_a -- --ignored"]
async fn market_town_demo_a_story() {
    demo_a_story(MARKET_TOWN, "market-town", Use::Buy).await;
}

#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test demo_a -- --ignored"]
async fn social_cafe_demo_a_story() {
    demo_a_story(SOCIAL_CAFE, "social-cafe", Use::Kick).await;
}
