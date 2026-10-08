//! The 2D reference client (`clients/2d`, step-13 S12 PR 13a), run for real against a real or a stub
//! server: AC-W1 … AC-W7 and AC-W10 of step-13 §14.2.
//!
//! Every test here starts Godot, so every test is `#[ignore]`d: `cargo test` reports them as ignored,
//! never as passed, on a machine without Godot (a test not run is not a pass). Run them with
//!
//! ```text
//! cargo test -p mineworld-cli --test client_2d -- --ignored --test-threads=1
//! ```
//!
//! Oracles are independent of the client: the save's fact log, another connection's observation,
//! `/status`, and what the stub knows it said. The client's own `[PASS]` lines are required as well,
//! but never alone.

mod godot2d;
mod headless;
mod support;

use std::time::Duration;

use godot2d::{Drive, MARKET_TOWN, StubScript, World, evidence, passed, places, stub, tagged};
use headless::Tables;
use mineworld_contracts::{ActionRequest, ActionResult, Causation, Event, EventEnvelope};
use mineworld_persistence::format;
use mineworld_presence::{Arrived, PersonEnteredPlace};
use mineworld_server::{WirePayload, differing_fields};
use serde_json::Value;
use support::{Client, SaveDir, stride, talk};

/// `--agent alice`, the save, and nothing else: a hosted market-town as the operator runs it.
fn hosted(save: &SaveDir) -> Vec<String> {
    vec![
        "server".into(),
        MARKET_TOWN.into(),
        "--agent".into(),
        "alice".into(),
        "--save".into(),
        save.path().into(),
    ]
}

fn args(owned: &[String]) -> Vec<&str> {
    owned.iter().map(String::as_str).collect()
}

/// A save's facts, decoded.
fn facts(save: &SaveDir) -> Vec<EventEnvelope> {
    Tables::read(std::path::Path::new(save.path()))
        .facts
        .iter()
        .map(|(_, bytes)| format::decode(bytes, "fact").expect("a fact"))
        .collect()
}

/// AC-W1 and AC-W2: out of the apartments, along the street, into the café, every stride accepted,
/// the observer's place changing twice — confirmed by the save's own facts, not by the client.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d -- --ignored"]
async fn walks_from_the_apartments_into_the_cafe() {
    let save = SaveDir::new("2d-walk");
    let mut world = World::start(&args(&hosted(&save)), None).await;
    let (status, lines) = Drive::start(world.address, "carol", &["--drive=walk"])
        .finish()
        .await;
    assert!(status.success() && passed(&lines), "the drive passes");

    let results = tagged(&lines, "RESULT ");
    let accepted = results
        .iter()
        .filter(|r| r["result"].get("accepted").is_some())
        .count();
    assert!(
        accepted > 0 && accepted == results.len(),
        "every move accepted: {accepted} of {}",
        results.len()
    );
    let visited = places(&lines);
    assert_eq!(visited.len(), 3, "apartments, street, café: {visited:?}");
    let observer = evidence(&lines, "welcome")[0]["observer"]
        .as_str()
        .expect("observer")
        .to_owned();
    world.kill();

    let facts = facts(&save);
    let mut entered = Vec::new();
    let mut arrivals = 0;
    for fact in &facts {
        if *fact.event_type() == PersonEnteredPlace::EVENT_TYPE {
            let e: PersonEnteredPlace = serde_json::from_slice(
                fact.payload()
                    .payload_for::<PersonEnteredPlace>()
                    .expect("presence's fact"),
            )
            .expect("decodes");
            if e.person().entity_id().to_string() == observer {
                entered.push(e.place().entity_id().to_string());
            }
        } else if *fact.event_type() == Arrived::EVENT_TYPE {
            let a: Arrived = serde_json::from_slice(
                fact.payload()
                    .payload_for::<Arrived>()
                    .expect("presence's fact"),
            )
            .expect("decodes");
            // Only arrivals someone asked for: a genesis placement is not a move.
            if a.person().entity_id().to_string() == observer
                && matches!(fact.caused_by(), Causation::Action(_))
            {
                arrivals += 1;
            }
        }
    }
    assert_eq!(
        entered,
        visited[1..].to_vec(),
        "the save records the street, then the café"
    );
    assert_eq!(arrivals, accepted, "one recorded arrival per accepted move");
}

/// AC-W10: the same walk asks the world for exactly the same things whatever draws it; and a click
/// asks for the point clicked, in both projections.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d -- --ignored"]
async fn the_requests_do_not_depend_on_the_presentation() {
    let mut transcripts: Vec<(String, Vec<ActionRequest<WirePayload>>)> = Vec::new();
    for presentation in [
        "--variant=town",
        "--variant=full",
        "--variant=people",
        "--variant=procedural",
        "--presentation=none",
    ] {
        let save = SaveDir::new(&format!("2d-i5-{}", presentation.replace(['-', '='], "")));
        let world = World::start(&args(&hosted(&save)), None).await;
        let (status, lines) = Drive::start(world.address, "carol", &["--drive=walk", presentation])
            .finish()
            .await;
        assert!(
            status.success() && passed(&lines),
            "{presentation}: the drive passes"
        );
        let requests = tagged(&lines, "REQUEST ")
            .into_iter()
            .map(|r| serde_json::from_value(r["request"].clone()).expect("a legal ActionRequest"))
            .collect();
        transcripts.push((presentation.to_owned(), requests));
    }
    let (reference_name, reference) = &transcripts[0];
    for (name, requests) in &transcripts[1..] {
        assert_eq!(
            requests.len(),
            reference.len(),
            "{name} sent as many requests as {reference_name}"
        );
        for (i, (a, b)) in reference.iter().zip(requests).enumerate() {
            assert!(
                differing_fields(a, b).is_empty(),
                "{name} request {i} differs: {:?}",
                differing_fields(a, b)
            );
        }
    }
    for presentation in ["--variant=town", "--presentation=none"] {
        let save = SaveDir::new("2d-click");
        let world = World::start(&args(&hosted(&save)), None).await;
        let (status, lines) =
            Drive::start(world.address, "carol", &["--drive=click", presentation])
                .finish()
                .await;
        assert!(
            status.success() && passed(&lines),
            "{presentation}: a click asks for the clicked point"
        );
    }
}

/// AC-W3, `AC-3` end to end: the client is killed mid-walk; the world goes on and carol is moved by
/// somebody else; the relaunched client is told the same world, a later revision, and draws carol
/// where the world now has her.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d -- --ignored"]
async fn killed_mid_walk_and_relaunched() {
    let save = SaveDir::new("2d-relaunch");
    let world = World::start(&args(&hosted(&save)), None).await;
    let mut drive = Drive::start(world.address, "carol", &["--drive=walk"]);
    drive
        .until("an accepted stride on the street", |lines| {
            let street_at = lines
                .iter()
                .position(|l| l.starts_with("PLACE ") && l.contains("street"));
            street_at.is_some_and(|at| {
                lines[at..]
                    .iter()
                    .any(|l| l.starts_with("RESULT ") && l.contains("accepted"))
            })
        })
        .await;
    let instance = evidence(&drive.lines(), "welcome")[0]["instance"].clone();
    let killed = drive.kill();
    assert!(!killed.success(), "the client died, not exited");
    let before = world.status().await;

    // The world goes on: the wanderer walks up to Alice and speaks; her agent answers.
    let mut wanderer = Client::connect(world.address).await;
    let (me, _) = wanderer.join("wanderer").await;
    let seen = wanderer.observation().await;
    let alice = support::tagged(&seen, "barista").expect("alice is in the café");
    let place = seen.self_location().expect("placed").place().entity_id();
    let to = seen
        .entity(alice)
        .and_then(|e| e.location())
        .and_then(|l| l.local())
        .expect("alice's position");
    let from = seen
        .self_location()
        .and_then(|l| l.local())
        .expect("own position");
    let (fx, fy) = (from.x().value(), from.y().value());
    let (tx, ty) = (to.x().value() - 1000, to.y().value() - 1000);
    wanderer
        .walk_accepted(support::walk(me, place, (fx, fy), (tx, ty)))
        .await;
    wanderer
        .submit_accepted(talk(me, alice, "Busy morning?"))
        .await;

    // And somebody moves carol: a connection to her seat, one stride, then gone.
    let mut other = Client::connect(world.address).await;
    let (carol, _) = other.join("carol").await;
    let her = other.observation().await;
    let at = her.self_location().expect("placed");
    let local = at.local().expect("a position");
    let moved_to = (local.x().value() + 700, local.y().value() - 900);
    let (_, result) = other
        .submit(stride(
            carol,
            at.place().entity_id(),
            moved_to.0,
            moved_to.1,
        ))
        .await;
    assert!(
        matches!(result, ActionResult::Accepted { .. }),
        "the stride is accepted: {result:?}"
    );
    other.disconnect().await;
    wanderer.disconnect().await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let after = world.status().await;
    assert!(
        after["revision"].as_u64() > before["revision"].as_u64(),
        "the revision advanced while the client was dead"
    );
    assert!(
        after["at"].as_i64() >= before["at"].as_i64(),
        "the clock did not go back"
    );

    let (status, lines) = Drive::start(world.address, "carol", &["--drive=seated"])
        .finish()
        .await;
    assert!(
        status.success() && passed(&lines),
        "the relaunched client passes its own checks"
    );
    let welcome = &evidence(&lines, "welcome")[0];
    assert_eq!(welcome["instance"], instance, "the same world");
    assert!(
        welcome["revision"].as_f64() >= after["revision"].as_f64(),
        "told a revision no earlier than the world's after the kill: {welcome} vs {after}"
    );
    // Placed from its first observation: a client that started from anything it remembered would
    // have to be corrected by the reconciliation rule, and that correction is visible.
    assert!(
        evidence(&lines, "reconciled").is_empty(),
        "drawn where the world said from the first frame"
    );
    let me_now = &evidence(&lines, "self")[0];
    assert_eq!(
        me_now["drawn_local"]["x"].as_i64(),
        Some(i64::from(moved_to.0)),
        "drawn where the world put carol"
    );
    assert_eq!(
        me_now["drawn_local"]["y"].as_i64(),
        Some(i64::from(moved_to.1)),
        "drawn where the world put carol"
    );
}

/// AC-W4: the server dies under a walking client and comes back on the same save and port — the
/// client re-joins on its own, is told the same world, keeps what it learned, and replays nothing;
/// then a different world on that port, and the client forgets the old layout.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d -- --ignored"]
async fn reconnects_to_a_restarted_world_and_to_a_replaced_one() {
    let save = SaveDir::new("2d-reconnect");
    let mut world = World::start(&args(&hosted(&save)), None).await;
    let drive = Drive::start(
        world.address,
        "carol",
        &["--drive=street", "--strides=20", "--hold=40"],
    );
    let mut known_before = 0;
    drive
        .until("two accepted strides along the street", |lines| {
            tagged(lines, "EVIDENCE ")
                .iter()
                .any(|e| e.get("self").is_some())
                && lines.iter().filter(|l| l.starts_with("RESULT ")).count() >= 6
        })
        .await;
    if let Some(selfie) = evidence(&drive.lines(), "self").first() {
        known_before = selfie["passages"].as_u64().unwrap_or(0);
    }
    world.kill();
    let killed_at = drive.lines().len();
    tokio::time::sleep(Duration::from_secs(2)).await;
    world.restart().await;
    let (status, lines) = drive.finish().await;
    assert!(
        status.success() && passed(&lines),
        "the drive passes after reconnecting"
    );
    let after_kill = &lines[killed_at..];
    let rejoined = after_kill
        .iter()
        .position(|l| l.starts_with("STATE seated"))
        .expect("seated again");
    assert!(
        after_kill
            .iter()
            .any(|l| l.starts_with("STATE reconnecting")),
        "it saw the drop"
    );
    // The walk in progress was abandoned and nothing is replayed: this scenario asks for nothing
    // after the drop, so any request there is a replay.
    assert!(rejoined > 0, "the drop came before the new seat");
    assert!(
        !after_kill.iter().any(|l| l.starts_with("REQUEST ")),
        "nothing was submitted after the drop"
    );
    let welcomes = evidence(&lines, "welcome");
    assert_eq!(welcomes.len(), 2, "welcomed twice");
    assert_eq!(
        welcomes[0]["instance"], welcomes[1]["instance"],
        "the same world"
    );
    let last = evidence(&lines, "self")
        .last()
        .cloned()
        .expect("a final report");
    assert!(
        known_before >= 6,
        "the street's doorways were learned: {known_before}"
    );
    assert_eq!(
        last["passages"].as_u64(),
        Some(known_before),
        "the layout was kept"
    );

    // A different world on the same address: a fresh save.
    let fresh = SaveDir::new("2d-reconnect-other");
    let address = world.address;
    world.kill();
    let drive = Drive::start(address, "carol", &["--drive=idle", "--hold=4"]);
    let _other = World::start(&args(&hosted(&fresh)), Some(address)).await;
    drive
        .until("seated", |lines| {
            lines.iter().any(|l| l.starts_with("STATE seated"))
        })
        .await;
    let (status, lines) = drive.finish().await;
    assert!(status.success(), "the idle run ends cleanly");
    let last = evidence(&lines, "self").last().cloned().expect("a report");
    assert_eq!(
        last["passages"].as_u64(),
        Some(1),
        "a fresh world: carol is home, one doorway known"
    );
}

/// AC-W5: a stride the world refuses ends the walk, and the body is drawn where the world says.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d -- --ignored"]
async fn a_refused_stride_ends_the_walk() {
    let (address, log) = stub(StubScript {
        reject: Some(3),
        ..StubScript::default()
    })
    .await;
    let (status, lines) = Drive::start(address, "carol", &["--drive=strides", "--strides=6"])
        .finish()
        .await;
    assert!(status.success(), "the run ends cleanly");
    let log = log.lock().expect("log");
    assert_eq!(log.submitted.len(), 3, "no stride after the refused one");
    let selfie = evidence(&lines, "self").last().cloned().expect("a report");
    assert_eq!(
        selfie["drawn_local"]["x"].as_i64(),
        Some(log.position.0),
        "drawn where the world says"
    );
    assert_eq!(
        selfie["drawn_local"]["y"].as_i64(),
        Some(log.position.1),
        "drawn where the world says"
    );
    let reconciled = evidence(&lines, "reconciled");
    assert!(
        reconciled.iter().any(|r| r["frames"].as_u64() <= Some(1)),
        "followed within one frame: {reconciled:?}"
    );
}

/// AC-W6: an observation that moves the player is followed at once, and never answered by a move.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d -- --ignored"]
async fn a_teleport_is_followed_not_argued_with() {
    let (address, log) = stub(StubScript {
        teleport_after: Some(2),
        ..StubScript::default()
    })
    .await;
    let (status, lines) = Drive::start(address, "carol", &["--drive=strides", "--strides=2"])
        .finish()
        .await;
    assert!(status.success() && passed(&lines), "the run passes");
    let log = log.lock().expect("log");
    let (at, to) = log.teleported.expect("the stub teleported the player");
    assert!(
        log.submitted.iter().all(|(when, _)| *when < at),
        "no move answered the teleport"
    );
    let reconciled = evidence(&lines, "reconciled");
    let followed = reconciled
        .iter()
        .find(|r| r["local"]["y"].as_i64() == Some(to.1))
        .expect("followed the teleport");
    assert!(
        followed["frames"].as_u64() <= Some(1),
        "within one frame: {followed}"
    );
}

/// AC-W7: a `move` the world offers as unavailable is still asked for when the player chooses it —
/// the answer is the world's to give.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d -- --ignored"]
async fn an_unavailable_move_is_still_asked_for() {
    let (address, log) = stub(StubScript {
        move_unavailable: true,
        ..StubScript::default()
    })
    .await;
    let (status, _lines) = Drive::start(address, "carol", &["--drive=strides", "--strides=1"])
        .finish()
        .await;
    assert!(status.success(), "the run ends cleanly");
    let log = log.lock().expect("log");
    assert_eq!(log.submitted.len(), 1, "the stride was submitted");
    let request: &Value = &log.submitted[0].1;
    assert_eq!(request["action_type"], "move");
}
