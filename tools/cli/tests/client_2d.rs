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
use mineworld_movement::{WalkEnded, WalkStarted};
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
    let entered = &evidence(&lines, "entered")[0];
    assert!(
        entered["tags"]
            .as_array()
            .is_some_and(|tags| tags.iter().any(|t| t == "cafe")),
        "the doorway the drive chose from the disclosure led into the café: {entered}"
    );
    let visited = places(&lines);
    assert_eq!(visited.len(), 3, "apartments, street, café: {visited:?}");
    // R-PK-1: on the street, before going in, the café's doorway already reads its name.
    let from_street = tagged(&lines, "SHOWN ")
        .into_iter()
        .filter(|shown| shown["place"] == visited[1].as_str())
        .collect::<Vec<_>>();
    assert!(
        from_street.iter().any(|shown| {
            shown["doorway_labels"].as_array().is_some_and(|labels| {
                labels.iter().any(|label| {
                    label["to"] == visited[2].as_str() && label["text"] == "door to café"
                })
            })
        }),
        "from the street the café's doorway reads 'door to café': {from_street:?}"
    );
    let observer = evidence(&lines, "welcome")[0]["observer"]
        .as_str()
        .expect("observer")
        .to_owned();
    world.kill();

    let facts = facts(&save);
    let mut entered = Vec::new();
    let mut arrivals = 0;
    // Since c163fec7 the drive walks by the server's route: one `walk-to` per leg, then paced
    // `walk-step`s. A `walk-to` records the walk's start, and a `walk-step` either a stride (an
    // arrival) or, at most once per walk, the walk's end alone. So every accepted request is in the
    // save, as the facts its result named, and the arrivals are the strides.
    let by_id: std::collections::BTreeMap<String, &EventEnvelope> =
        facts.iter().map(|f| (f.id().to_string(), f)).collect();
    let kinds: std::collections::BTreeMap<String, String> = tagged(&lines, "REQUEST ")
        .iter()
        .map(|r| {
            (
                r["token"].as_str().expect("token").to_owned(),
                r["request"]["action_type"]
                    .as_str()
                    .expect("type")
                    .to_owned(),
            )
        })
        .collect();
    let (mut walks, mut steps, mut lone_ends) = (0, 0, 0);
    for result in &results {
        let ids: Vec<&str> = result["result"]["accepted"]["events"]
            .as_array()
            .expect("events")
            .iter()
            .map(|id| id.as_str().expect("an event id"))
            .collect();
        // Each fact a result names was caused by the request, or by another fact of the same
        // result (movement's `walk-ended` follows the walker's own arrival).
        let named: Vec<&EventEnvelope> = ids
            .iter()
            .map(|id| {
                let fact = by_id
                    .get(*id)
                    .unwrap_or_else(|| panic!("event {id} is in the save"));
                let caused = match fact.caused_by() {
                    Causation::Action(_) => true,
                    Causation::Event(by) => ids.contains(&by.to_string().as_str()),
                    _ => false,
                };
                assert!(caused, "event {id} was caused by the request: {result}");
                *fact
            })
            .collect();
        let has = |kind| named.iter().any(|f| *f.event_type() == kind);
        match kinds[result["token"].as_str().expect("token")].as_str() {
            "walk-to" => {
                walks += 1;
                assert!(
                    has(WalkStarted::EVENT_TYPE),
                    "a walk-to starts a walk: {result}"
                );
            }
            "walk-step" => {
                steps += 1;
                if !has(Arrived::EVENT_TYPE) {
                    assert!(
                        has(WalkEnded::EVENT_TYPE),
                        "a step strides or ends: {result}"
                    );
                    lone_ends += 1;
                }
            }
            other => panic!("the walk drive asks only walk-to and walk-step: {other}"),
        }
    }
    assert!(walks > 0 && steps > 0, "{walks} walks, {steps} steps");
    assert!(
        lone_ends <= walks,
        "a walk ends without a stride at most once: {lone_ends} of {walks} walks"
    );
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
    assert_eq!(
        arrivals,
        steps - lone_ends,
        "one recorded arrival per accepted stride"
    );
}

/// F-9 and F-10, where `./mineworld-2d` starts: carol at home, played through the real input path.
/// Clicking past the room's far wall must not change the drawn floor; a click on the door, or walking
/// onto it with the keys, must take her out to the place the doorway leads to — confirmed by the
/// save's own `person-entered-place`, not by the client.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d -- --ignored"]
async fn leaves_home_by_its_door_and_the_room_stays_put() {
    for exit in ["click", "keys"] {
        let save = SaveDir::new(&format!("2d-home-{exit}"));
        let mut world = World::start(&args(&hosted(&save)), None).await;
        let (status, lines) = Drive::start(
            world.address,
            "carol",
            &["--drive=home", &format!("--exit={exit}")],
        )
        .finish()
        .await;
        assert!(
            status.success() && passed(&lines),
            "{exit}: the drive passes"
        );
        let observer = evidence(&lines, "welcome")[0]["observer"]
            .as_str()
            .expect("observer")
            .to_owned();
        let visited = places(&lines);
        world.kill();
        let mut entered = Vec::new();
        for fact in &facts(&save) {
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
            }
        }
        assert_eq!(visited.len(), 2, "{exit}: home, then out: {visited:?}");
        assert_eq!(
            entered,
            visited[1..].to_vec(),
            "{exit}: the save records the one crossing"
        );
    }
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
    // `--hold 0`: since S11-B a dropped connection's seat is held for its resume (30 s by default),
    // and this test's "somebody else" must be able to take carol the moment the client dies — as
    // before S11-B, which is what AC-W3 states (step-13 §15.12 E-13).
    let mut arguments = hosted(&save);
    arguments.extend(["--hold".to_owned(), "0".to_owned()]);
    let world = World::start(&args(&arguments), None).await;
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
    // Since the café has bodies (step-11 §19) the table between the wanderer and Alice and the counter
    // in front of her are solids: the wanderer walks to the counter's customer side next to her
    // (`ac15_one_alice.rs` ALSO_NEXT_TO_ALICE, 1.3 m east of her and in front of the counter), the
    // world routing the walk round the table (`walk-to`, ARC-75) and the wanderer pacing it.
    let next_to_alice = (to.x().value() + 1_300, 6_200);
    wanderer.walk_there(me, place, Some(next_to_alice)).await;
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
    // Counted before the kill, not after it: the client sees the connection close while the kill is
    // still being reaped, and on Windows its `STATE reconnecting` line was already read by then (nightly
    // 39012891928), so a count taken after the kill skipped the very line this test looks for.
    let killed_at = drive.lines().len();
    world.kill();
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
/// The walk under way goes on as the world's own: the client keeps pacing it with `walk-step`s, which
/// carry no position, and asks for no point of its own.
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
        log.submitted
            .iter()
            .filter(|(when, _)| *when >= at)
            .all(|(_, request)| request["action_type"] == "walk-step"),
        "no move answered the teleport: {:?}",
        log.submitted
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

/// A stub run of `--drive=strides --strides=3` from the stub's (3 000, 2 000): a walk 5.7 m east, to
/// (8 700, 2 000). Returns every request the stub received, with when, and where each left the body.
async fn three_strides(script: StubScript) -> (Vec<(Duration, Value)>, Vec<(i64, i64)>) {
    let (address, log) = stub(script).await;
    let (status, lines) = Drive::start(address, "carol", &["--drive=strides", "--strides=3"])
        .finish()
        .await;
    assert!(
        status.success() && passed(&lines),
        "the run passes: {lines:?}"
    );
    let log = log.lock().expect("log");
    let first = log
        .submitted
        .first()
        .map(|(when, _)| *when)
        .expect("a request");
    let requests = log
        .submitted
        .iter()
        .map(|(when, request)| (when.duration_since(first), request.clone()))
        .collect();
    (requests, log.placed.clone())
}

/// `ADOPTION.md` §4.2, step-11 SD-N12: going somewhere is one `walk-to` naming the destination, then
/// payload-less `walk-step`s, one per wall second, until the walk is no longer disclosed. The client
/// splits nothing and names no point but the destination.
fn one_walk_paced(requests: &[(Duration, Value)], destination: (i64, i64), steps: usize) {
    let types: Vec<&str> = requests
        .iter()
        .map(|(_, r)| r["action_type"].as_str().unwrap_or(""))
        .collect();
    let mut want = vec!["walk-to"];
    want.extend(std::iter::repeat_n("walk-step", steps));
    assert_eq!(types, want, "one walk-to, then the steps: {requests:?}");
    let to = &requests[0].1["payload"]["payload"]["to"]["place"]["local"];
    let asked = (to["x"].as_i64().expect("x"), to["y"].as_i64().expect("y"));
    assert!(
        (asked.0 - destination.0).abs() <= 1 && (asked.1 - destination.1).abs() <= 1,
        "the walk asks for {asked:?}, the destination is {destination:?}"
    );
    for pair in requests[1..].windows(2) {
        assert!(
            pair[1].0 - pair[0].0 >= Duration::from_millis(900),
            "one walk-step per wall second: {:?} then {:?}",
            pair[0].0,
            pair[1].0
        );
    }
}

/// An unobstructed walk: 5.7 m in WALK_STRIDE (1.34 m) steps is five steps, and the body ends at the
/// destination.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d -- --ignored"]
async fn a_walk_is_one_walk_to_paced_by_walk_steps() {
    let (requests, placed) = three_strides(StubScript::default()).await;
    one_walk_paced(&requests, (8_700, 2_000), 5);
    assert_eq!(placed.last(), Some(&(8_700, 2_000)), "arrived: {placed:?}");
}

/// The world stops the first step halfway (a body in the way): the client asks nothing of its own
/// about it. The walk is the world's, and the client paces it until the world says it is over.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d -- --ignored"]
async fn a_step_stopped_short_is_the_worlds_to_go_on_from() {
    let (requests, placed) = three_strides(StubScript {
        resolve: Some((2, godot2d::Resolve::Along(0.5))),
        ..StubScript::default()
    })
    .await;
    assert_eq!(
        placed[1],
        (3_670, 2_000),
        "the stub stopped the step halfway"
    );
    one_walk_paced(&requests, (8_700, 2_000), 5);
    assert_eq!(placed.last(), Some(&(8_700, 2_000)), "arrived: {placed:?}");
}

/// AC-W7: walking the world offers as unavailable is still asked for when the player chooses it —
/// the answer is the world's to give.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d -- --ignored"]
async fn an_unavailable_walk_is_still_asked_for() {
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
    assert!(!log.submitted.is_empty(), "the walk was submitted");
    let request: &Value = &log.submitted[0].1;
    assert_eq!(request["action_type"], "walk-to");
}
