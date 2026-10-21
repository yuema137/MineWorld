//! `AC-15` with the **real** clients (S14 PR 16e §4.2): one `mineworld server <world> --agent alice
//! --save DIR`, the real 2D client on one seat and the real 3D client on another, alive at once, and
//! the four identity lines of `docs/MVP.md` §9.1 read off what each client printed and checked against
//! the server's own oracles — `GET /status`, the save's fact log, and the save's head after the server
//! is gone. "Both windows looked alike" is never evidence (`ac15_one_alice.rs`, which proves the same
//! with Rust connections and runs in `core`, explains why).
//!
//! ```text
//! EVIDENCE 1  same world instance        2D IDENTITY_WORLD.instance = 3D's = GET /status instance
//! EVIDENCE 2  same Alice EntityId        both clients' people map the save's Alice id to one name
//! EVIDENCE 3  same event sequence        each client's EventIds are a sub-sequence of the save's
//! EVIDENCE 4  same persisted revision    each client's last revision ≤ the save's head
//! STORY       2D talks to Alice; then 3D does; her answer to 3D carries what 2D said
//! ```
//!
//! Every test starts Godot, so every test is `#[ignore]`d and runs at the milestone:
//!
//! ```text
//! cargo test -p mineworld-cli --test ac15_clients -- --ignored --test-threads=1
//! ```

mod godot2d;
mod godot3d;
mod headless;
mod support;

use std::path::Path;
use std::time::Duration;

use godot2d::worlds::{hosted, id, ids, start};
use godot2d::{MARKET_TOWN, World};
use godot3d::{identity, world_key};
use headless::Tables;
use mineworld_contracts::EventEnvelope;
use mineworld_persistence::format;
use serde_json::{Value, json};
use support::{SaveDir, run_command};

const SOCIAL_CAFE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/social-cafe");

/// The seat the agent drives (`--agent alice`) and the two seats the clients take.
const AGENT: &str = "alice";
const SEAT_2D: &str = "visitor";
const SEAT_3D: &str = "wanderer";
/// What each window says to Alice. Distinct, so her answer to the 3D window can only carry the 2D
/// window's words if one Alice heard both.
const FROM_2D: &str = "hello Alice, this is the 2D window";
const FROM_3D: &str = "hello Alice, this is the 3D window";
/// How long each client waits after speaking for Alice's answer.
const ANSWER_S: u64 = 8;

fn owned(arguments: &[String]) -> Vec<&str> {
    arguments.iter().map(String::as_str).collect()
}

/// Walk up to Alice and speak `words` to her (13b's selectors, both clients).
fn talk_steps(alice: &str, words: &str) -> Vec<Value> {
    vec![
        json!({ "open": alice }),
        json!({ "choose": { "walk": true } }),
        json!({ "open": alice }),
        json!({ "choose": { "action_type": "talk", "target": alice }, "input": words }),
        json!({ "sleep": ANSWER_S }),
    ]
}

/// The save's facts in log order: EventId, type, payload.
fn facts(save: &SaveDir) -> Vec<(String, String, Value)> {
    Tables::read(Path::new(save.path()))
        .facts
        .iter()
        .map(|(_, bytes)| {
            let fact: EventEnvelope = format::decode(bytes, "fact").expect("a fact");
            let payload: Value =
                serde_json::from_slice(fact.payload().payload()).expect("a JSON payload");
            (
                fact.id().raw().to_string(),
                fact.event_type().as_str().to_owned(),
                payload,
            )
        })
        .collect()
}

/// EVIDENCE 1: the two clients and `/status` name one instance. `Err` names the instances.
fn same_instance(two: &Value, three: &Value, status: &Value) -> Result<String, String> {
    let (a, b) = (&two["instance"], &three["instance"]);
    if a.as_str().is_none_or(str::is_empty) || a != b {
        return Err(format!(
            "the 2D client shows instance {a}, the 3D client {b}"
        ));
    }
    if *a != status["instance"] {
        return Err(format!(
            "the clients show instance {a}, the world reports {}",
            status["instance"]
        ));
    }
    Ok(a.as_str().unwrap_or("").to_owned())
}

/// Whether `seen` occurs, in its order, within `log`.
fn subsequence(seen: &[String], log: &[String]) -> bool {
    let mut rest = log.iter();
    seen.iter().all(|e| rest.any(|l| l == e))
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|a| a.iter().map(id).collect())
        .unwrap_or_default()
}

/// One world, both real clients, the agent: evidence 1–4 and the story (AC-e-4).
async fn one_alice(pack: &str, tag: &str) {
    let people = ids(pack);
    let (alice, visitor, wanderer) = (
        people[AGENT].clone(),
        people[SEAT_2D].clone(),
        people[SEAT_3D].clone(),
    );
    let save = SaveDir::new(&format!("ac15-clients-{tag}"));
    let world = World::start(&owned(&hosted(pack, &save)), None).await;
    let key = world_key(pack);
    let sync = mineworld_test_support::scratch!(empty "ac15-clients-sync");
    let sync_arg = format!("--sync={}", sync.to_str().expect("UTF-8"));

    // The 2D window speaks first, then stays seated until the 3D window is done.
    let mut steps_2d = talk_steps(&alice, FROM_2D);
    steps_2d.extend([
        json!({ "mark": "said" }),
        json!({ "after": "3d-done" }),
        json!({ "report": true }),
    ]);
    let (drive, _dir) = start(
        world.address,
        SEAT_2D,
        &format!("ac15-clients-{tag}-2d"),
        &Value::Array(steps_2d),
        &[sync_arg.as_str(), key.as_str()],
    );
    let said = sync.join("said");
    drive
        .until("the 2D window has spoken", |_| said.exists())
        .await;

    // The 3D window walks up to the same Alice and speaks, while the 2D window is still seated.
    let mut steps_3d = talk_steps(&alice, FROM_3D);
    steps_3d.push(json!({ "report": true }));
    let address = world.address;
    let name = format!("ac15-clients-{tag}-3d");
    let key_3d = key.clone();
    let three_d = tokio::task::spawn_blocking(move || {
        godot3d::play(
            address,
            SEAT_3D,
            &name,
            &Value::Array(steps_3d),
            &[key_3d.as_str()],
        )
    });
    let mut both_seated = json!(null);
    while !three_d.is_finished() {
        let status = world.status().await;
        if status["clients"] == json!(2) {
            both_seated = status;
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    let (status_3d, lines_3d) = three_d.await.expect("the 3D run ends");
    assert!(
        status_3d.success() && godot3d::passed(&lines_3d),
        "{tag}: the 3D probe passes"
    );
    std::fs::write(sync.join("3d-done"), "1").expect("the mark is written");
    let (status_2d, lines_2d) = drive.finish().await;
    assert!(
        status_2d.success() && godot2d::passed(&lines_2d),
        "{tag}: the 2D drive passes:\n{}",
        lines_2d.join("\n")
    );
    let status = world.status().await;
    drop(world);
    assert_eq!(
        both_seated["clients"],
        json!(2),
        "{tag}: both clients were seated at once while the agent drove Alice"
    );

    let (two, three) = (identity(&lines_2d), identity(&lines_3d));
    // EVIDENCE 1.
    let instance = same_instance(&two, &three, &status)
        .unwrap_or_else(|why| panic!("{tag}: EVIDENCE 1: {why}"));
    // EVIDENCE 2: the save's Alice, by one disclosed name, in both clients.
    let named = two["people"][&alice].clone();
    assert!(
        named.as_str().is_some_and(|n| !n.is_empty()),
        "{tag}: EVIDENCE 2: the 2D client knows Alice ({alice}) by a name: {two}"
    );
    assert_eq!(
        three["people"][&alice], named,
        "{tag}: EVIDENCE 2: the 3D client knows the same Alice by the same name"
    );
    // EVIDENCE 3: one event sequence.
    let log = facts(&save);
    let order: Vec<String> = log.iter().map(|(e, _, _)| e.clone()).collect();
    let (seen_2d, seen_3d) = (strings(&two["events_seen"]), strings(&three["events_seen"]));
    assert!(
        !seen_2d.is_empty() && !seen_3d.is_empty(),
        "{tag}: EVIDENCE 3: both clients learned facts"
    );
    for (who, seen) in [("2D", &seen_2d), ("3D", &seen_3d)] {
        assert!(
            subsequence(seen, &order),
            "{tag}: EVIDENCE 3: the {who} client's facts are a sub-sequence of the save's: {seen:?}"
        );
    }
    // EVIDENCE 4: one persisted revision.
    let head = status["revision"]
        .as_u64()
        .expect("a saved world's revision");
    for (who, record) in [("2D", &two), ("3D", &three)] {
        let revision = record["revision"].as_f64().map(|r| r as u64);
        assert!(
            revision.is_some_and(|r| r <= head),
            "{tag}: EVIDENCE 4: the {who} client's revision {revision:?} is one the world reached ({head})"
        );
    }
    let (replayed, printed) = run_command(&["replay", pack, "--save", save.path()]);
    assert!(replayed.success(), "{tag}: the save verifies: {printed}");
    // STORY: Alice's answer to the 3D window carries what the 2D window said, and the 3D client was
    // told it.
    let answer = log
        .iter()
        .rev()
        .find(|(_, t, p)| {
            t == "spoke" && id(&p["speaker"]) == alice && id(&p["listener"]) == wanderer
        })
        .unwrap_or_else(|| panic!("{tag}: Alice never answered the 3D window"));
    let words = answer.2["utterance"].to_string();
    assert!(
        words.contains(FROM_2D),
        "{tag}: STORY: she told the 3D window what the 2D window said: {words}"
    );
    assert!(
        seen_3d.contains(&answer.0),
        "{tag}: STORY: the 3D client learned that answer ({})",
        answer.0
    );
    assert!(
        log.iter().any(|(_, t, p)| t == "spoke"
            && id(&p["speaker"]) == visitor
            && p["utterance"].to_string().contains(FROM_2D)),
        "{tag}: the save holds the 2D window's words, spoken by the visitor"
    );

    println!("AC-15 with the real clients, {tag}:");
    println!("  same world instance        {instance} (2D, 3D, GET /status)");
    println!("  same Alice EntityId        {alice} = {named}");
    println!(
        "  one event sequence         2D {} facts, 3D {} facts, both in the save's order",
        seen_2d.len(),
        seen_3d.len()
    );
    println!(
        "  one persisted revision     2D {}, 3D {}, head {head}",
        two["revision"], three["revision"]
    );
    println!("  Alice told the 3D window   {words}");
}

#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test ac15_clients -- --ignored"]
async fn one_alice_with_the_real_clients_social_cafe() {
    one_alice(SOCIAL_CAFE, "social-cafe").await;
}

#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test ac15_clients -- --ignored"]
async fn one_alice_with_the_real_clients_market_town() {
    one_alice(MARKET_TOWN, "market-town").await;
}

/// AC-e-5: two servers, one client each. Evidence 1 fails, naming the two instances.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test ac15_clients -- --ignored"]
async fn two_servers_are_told_apart() {
    let (first_save, second_save) = (SaveDir::new("ac15-two-1"), SaveDir::new("ac15-two-2"));
    let first = World::start(&owned(&hosted(SOCIAL_CAFE, &first_save)), None).await;
    let second = World::start(&owned(&hosted(SOCIAL_CAFE, &second_save)), None).await;
    let key = world_key(SOCIAL_CAFE);
    let report = json!([{ "report": true }]);
    let (_, lines_2d) = godot2d::worlds::play(
        first.address,
        SEAT_2D,
        "ac15-two-2d",
        &report,
        &[key.as_str()],
    )
    .await;
    let (_, lines_3d) = godot3d::play(
        second.address,
        SEAT_3D,
        "ac15-two-3d",
        &report,
        &[key.as_str()],
    );
    let status = first.status().await;
    let (two, three) = (identity(&lines_2d), identity(&lines_3d));
    let why = same_instance(&two, &three, &status).expect_err("two servers are two worlds");
    let (a, b) = (
        two["instance"].as_str().expect("an instance"),
        three["instance"].as_str().expect("an instance"),
    );
    assert!(
        a != b && why.contains(a) && why.contains(b),
        "AC-e-5: the failure names both instances: {why}"
    );
    println!("AC-e-5 counterfactual: {why}");
}
