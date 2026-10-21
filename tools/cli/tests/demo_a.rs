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
//! No test names a world's person, place or item: every choice is read from what the world disclosed
//! (the client's own `SHOWN` report of the drawn scene, the observation's offers), and every assertion's
//! oracle is the save, `GET /status`, or what the world listed — never the client's `[PASS]` alone.

mod godot2d;
mod headless;
mod support;

use std::collections::BTreeSet;

use godot2d::worlds::{ids, start};
use godot2d::{MARKET_TOWN, World, evidence, passed, tagged};
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
    assert_eq!(distinct.len(), doorways.len(), "{tag}: each doorway once: {doorways:?}");
    let here = shown["place"].to_string();
    let out = doorways
        .iter()
        .find(|d| d.0 == here)
        .unwrap_or_else(|| panic!("{tag}: a doorway of {here} is drawn: {doorways:?}"))
        .clone();

    // AC-f-5: the objects drawn are the objects the newest observation lists, by id.
    let objects = evidence(&lines, "objects").pop().expect("an objects report");
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
        .map(|a| (a["action_type"].clone(), a["target"].clone(), a["available"] == true))
        .collect();
    let mine: Vec<(Value, Value, bool)> = shown["affordances"]
        .as_array()
        .expect("affordances")
        .iter()
        .filter(|a| a["subject"] == observer["subject"])
        .map(|a| (a["action_type"].clone(), a["target"].clone(), a["available"] == true))
        .collect();
    assert_eq!(mine, offered, "{tag}: SHOWN.affordances lists my offers as offered");
    assert!(!mine.is_empty(), "{tag}: the observer is offered something");
    for a in shown["affordances"].as_array().expect("affordances") {
        if a["subject"] != observer["subject"] {
            assert_eq!(a["target"], a["subject"], "{tag}: a person's menu is about them: {a}");
        }
    }

    // AC-f-6: the instance is the running world's, the revision no later than its head, every person
    // one the world was made with.
    let identity = tagged(&lines, "IDENTITY_WORLD ").pop().expect("an IDENTITY_WORLD line");
    assert_eq!(identity["instance"], status["instance"], "{tag}: one world: {status}");
    assert!(
        identity["revision"].is_u64()
            && identity["revision"].as_u64() <= status["revision"].as_u64(),
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
    assert!(ok, "{tag}: the drive passes with {flag}:\n{}", dropped.join("\n"));
    let after: BTreeSet<_> = tagged(&dropped, "SHOWN ")
        .pop()
        .expect("a SHOWN report")["doorways"]
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
