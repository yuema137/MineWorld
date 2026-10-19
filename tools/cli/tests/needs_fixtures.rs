//! Recorded observations of needs, sleep and lateness (`pr-s35-ne-presentation.md` §5; S35 N-e).
//!
//! Each fixture is one observation as a seat received it from the real `mineworld` binary, written by an
//! ignored recorder. Recording is run by hand, once, and the files are committed:
//!
//! ```text
//! MINEWORLD_WRITE_FIXTURES=1 cargo test -p mineworld-cli --test needs_fixtures -- --ignored --test-threads=1
//! ```
//!
//! ```text
//! market-town-night.json  Market Town with the packs, `--town`; a resident asleep at 02:30 on day 1 with
//!                         an alarm, their needs, and their agenda's next part
//! commute-late.json       commute-town with the packs; Ben's seat held from genesis and never moved; his
//!                         shift at the bakery ends at 12:00 and he never came: `late-for-shift`, absent
//! commute-sleeper.json    commute-town with the packs and sleep permitted at the bakery's class; Ana sleeps
//!                         there for 60 minutes, and Ben's first observation with her in the room shows it
//! ```
//!
//! The worlds are read as copies (`fixture/needs.rs` switches the packs on; the reference world is not
//! edited). Nothing here names a need or a word: the recorders only ask a seat for what it was told.

mod headless;
mod support;

use headless::Scratch;

#[path = "fixture/needs.rs"]
mod needs_fixture;

use std::path::Path;
use std::time::{Duration, Instant};

use mineworld_contracts::{ActionResult, EntityId};
use mineworld_worldpack::WorldPack;
use serde_json::{Value, json};
use support::{Client, Server};

const MARKET: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/market-town");
const COMMUTE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worlds/commute-town");
const DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/observations/needs"
);

/// 02:30 of day 1 in world seconds (day 1 begins at 0, the genesis instant).
const NIGHT: i64 = 2 * 3_600 + 30 * 60;

/// Fast-forward needs `--solo` (loopback only); the recorders run at 300 world seconds per wall second.
const SCALE: &str = "300";

/// A fixture the recorder has not reached yet waits at most this long, in wall seconds.
const WAIT: Duration = Duration::from_secs(900);

/// The `payload` of `entity`'s component `kind` in an observation, if disclosed to the observer.
fn component<'a>(observation: &'a Value, entity: &str, kind: &str) -> Option<&'a Value> {
    observation["entities"]
        .as_array()?
        .iter()
        .find(|perceived| perceived["id"].as_str() == Some(entity))?["components"]
        .as_array()?
        .iter()
        .find(|record| record["component_type"].as_str() == Some(kind))
        .map(|record| &record["payload"])
}

/// The place a doorway of the observer's surroundings leads to, when that place's tags name `tag`: the
/// `passages` a seat is told about the place it stands in (`to_tags` lists the destination's tags).
fn leads_to(observation: &Value, tag: &str) -> Option<EntityId> {
    observation["entities"]
        .as_array()?
        .iter()
        .flat_map(|perceived| perceived["components"].as_array().into_iter().flatten())
        .filter(|record| record["component_type"].as_str() == Some("passages"))
        .flat_map(|record| {
            record["payload"]["leads_to"]
                .as_array()
                .into_iter()
                .flatten()
        })
        .find(|passage| {
            passage["to_tags"]
                .as_array()
                .is_some_and(|tags| tags.iter().any(|carried| carried.as_str() == Some(tag)))
        })
        .and_then(|passage| passage["to"]["entity"].as_str()?.parse::<u64>().ok())
        .map(EntityId::from_raw)
}

fn observer_of(observation: &Value) -> String {
    observation["observer"]
        .as_str()
        .expect("an observation names its observer")
        .to_owned()
}

fn to_value(observation: impl serde::Serialize) -> Value {
    serde_json::to_value(observation).expect("an observation is JSON")
}

/// Whether an observation tells its observer their needs, their sleep (with its alarm) and the next part
/// of their day. No seat asleep at 02:30 in Market Town's run was a worker whose next part was punctual
/// (the punctual next part is recorded in `commute-late.json` instead), so this asks for the part only.
fn own_needs_asleep_and_next(observation: &Value) -> bool {
    let me = observer_of(observation);
    component(observation, &me, "needs").is_some_and(|needs| needs["words"].is_object())
        && component(observation, &me, "asleep").is_some_and(|asleep| asleep["alarm"].is_i64())
        && component(observation, &me, "agenda").is_some_and(|agenda| agenda["next"].is_object())
}

/// Writes one fixture. Refuses to run without `MINEWORLD_WRITE_FIXTURES`, so an ordinary test run never
/// rewrites the committed files.
fn record(name: &str, observation: &Value) {
    assert!(
        std::env::var_os("MINEWORLD_WRITE_FIXTURES").is_some(),
        "{name}: set MINEWORLD_WRITE_FIXTURES=1 to record a fixture"
    );
    std::fs::create_dir_all(DIR).expect("the fixtures directory");
    let path = Path::new(DIR).join(format!("{name}.json"));
    let text = serde_json::to_string_pretty(observation).expect("JSON");
    std::fs::write(&path, text + "\n").expect("the fixture writes");
    println!("recorded {}", path.display());
}

/// Waits until the world's clock reads `at` (`/status`'s `at`), in wall seconds.
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

/// `sleep { for: minutes }` from `actor`, as an agent or a player asks (the same offer a rule seat takes).
fn sleep_for(actor: EntityId, minutes: u32) -> Value {
    json!({
        "actor": actor,
        "action_type": "sleep",
        "target": null,
        "payload": { "action_type": "sleep", "payload": { "for": minutes } },
        "actor_location": null,
    })
}

/// The copy of a world with the packs, as a path string. The copy is removed when the returned guard is
/// dropped, so the caller keeps the guard for as long as the server reads the copy.
fn copy_of(name: &str, world: &str) -> (Scratch, String) {
    let copy = needs_fixture::with_needs(name, Path::new(world), 6);
    let text = copy.to_str().expect("a printable path").to_owned();
    (copy, text)
}

#[tokio::test]
#[ignore = "records the needs fixtures; run by hand with MINEWORLD_WRITE_FIXTURES=1"]
async fn record_market_town_night() {
    let (_scratch, copy) = copy_of("needs-fixture-market-night", MARKET);
    let pack = WorldPack::read(&copy).expect("the copy reads");
    let arguments = [
        "server",
        copy.as_str(),
        "--town",
        "--solo",
        "--time-scale",
        SCALE,
        "--cadence",
        "1",
        "--hold",
        "0",
    ];
    let server = Server::start(&arguments).await;
    wait_until(&server, NIGHT).await;
    for seat in pack.seats().iter().map(|seat| seat.as_str()) {
        let mut client = Client::connect(server.address).await;
        client.join(seat).await;
        let observation = to_value(client.observation().await);
        if own_needs_asleep_and_next(&observation) {
            record("market-town-night", &observation);
            return;
        }
        client.disconnect().await;
    }
    panic!("no seat was asleep at 02:30 with its needs, sleep and agenda in view");
}

#[tokio::test]
#[ignore = "records the needs fixtures; run by hand with MINEWORLD_WRITE_FIXTURES=1"]
async fn record_commute_late() {
    let (_scratch, copy) = copy_of("needs-fixture-commute-late", COMMUTE);
    let arguments = [
        "server",
        copy.as_str(),
        "--solo",
        "--time-scale",
        SCALE,
        "--cadence",
        "1",
        "--hold",
        "0",
    ];
    let server = Server::start(&arguments).await;
    let mut client = Client::connect(server.address).await;
    client.join("ben").await;
    let deadline = Instant::now() + WAIT;
    loop {
        let observation = to_value(client.observation().await);
        let me = observer_of(&observation);
        let late = observation["events"].as_array().is_some_and(|events| {
            events.iter().any(|event| {
                event["event_type"].as_str() == Some("late-for-shift")
                    && event["payload"]["payload"]["employee"]["entity"].as_str()
                        == Some(me.as_str())
            })
        });
        if late {
            let hunger = component(&observation, &me, "needs")
                .and_then(|needs| needs["words"]["hunger"].as_str().map(str::to_owned));
            assert!(
                matches!(hunger.as_deref(), Some("hungry" | "famished")),
                "the lateness is recorded while hungry or worse: {hunger:?}"
            );
            assert!(
                component(&observation, &me, "agenda")
                    .is_some_and(|agenda| agenda["next"].is_object()),
                "the agenda's next part is in view"
            );
            record("commute-late", &observation);
            return;
        }
        assert!(Instant::now() < deadline, "no late-for-shift arrived");
    }
}

#[tokio::test]
#[ignore = "records the needs fixtures; run by hand with MINEWORLD_WRITE_FIXTURES=1"]
async fn record_commute_sleeper() {
    let (_scratch, copy) = copy_of("needs-fixture-commute-sleeper", COMMUTE);
    // The bakery's floor is a class of its own, and sleep is permitted there (configure/classes.yaml and
    // configure/sleep.yaml, which the test writes into its copy; the reference world is not edited).
    let manifest = Path::new(&copy).join("world.yaml");
    let text = std::fs::read_to_string(&manifest).expect("the manifest reads");
    let text = text.replacen(
        "configure:\n  - employment\n",
        "configure:\n  - employment\n  - classes\n  - sleep\n",
        1,
    );
    assert!(text.contains("  - classes\n"), "the manifest names classes");
    std::fs::write(&manifest, text).expect("the manifest writes");
    std::fs::create_dir_all(Path::new(&copy).join("configure")).expect("the configure directory");
    std::fs::write(
        Path::new(&copy).join("configure/classes.yaml"),
        "- { class: bakery-floor, of: place, tag: bakery }\n",
    )
    .expect("classes write");
    std::fs::write(
        Path::new(&copy).join("configure/sleep.yaml"),
        "rules:\n  - { action: sleep, place: bakery-floor, effect: permit }\n",
    )
    .expect("sleep writes");
    let arguments = [
        "server",
        copy.as_str(),
        "--solo",
        "--time-scale",
        SCALE,
        "--cadence",
        "1",
        "--hold",
        "0",
    ];
    let server = Server::start(&arguments).await;

    // Ana (seat P) walks from home to the bakery through the street, then sleeps there for an hour.
    let mut ana = Client::connect(server.address).await;
    let (ana_id, _) = ana.join("ana").await;
    let home = to_value(ana.observation().await);
    let street = leads_to(&home, "street").expect("home's doorway leads to the street");
    ana.walk_there(ana_id, street, None).await;
    let on_street = to_value(ana.observation().await);
    let bakery = leads_to(&on_street, "bakery").expect("the street's doorway leads to the bakery");
    ana.walk_there(ana_id, bakery, None).await;
    let (_, answer) = ana.submit(sleep_for(ana_id, 60)).await;
    assert!(
        matches!(answer, ActionResult::Accepted { .. }),
        "Ana's sleep at the bakery is accepted: {answer:?}"
    );
    let sleeper = ana_id.to_string();

    // Ben (seat Q) walks in and sees her there, asleep.
    let mut ben = Client::connect(server.address).await;
    let (ben_id, _) = ben.join("ben").await;
    let ben_home = to_value(ben.observation().await);
    let ben_street = leads_to(&ben_home, "street").expect("home's doorway leads to the street");
    ben.walk_there(ben_id, ben_street, None).await;
    let ben_on_street = to_value(ben.observation().await);
    let ben_bakery =
        leads_to(&ben_on_street, "bakery").expect("the street's doorway leads to the bakery");
    ben.walk_there(ben_id, ben_bakery, None).await;
    let observation = to_value(
        ben.observation_where("Ana asleep in the bakery", |observation| {
            let value = to_value(observation);
            component(&value, &sleeper, "asleep").is_some()
        })
        .await,
    );
    assert!(
        component(&observation, &sleeper, "asleep")
            .is_some_and(|asleep| asleep["alarm"].is_null() || asleep.get("alarm").is_none()),
        "Ben is told Ana is asleep, and not when she wakes"
    );
    assert!(
        component(&observation, &observer_of(&observation), "needs")
            .is_some_and(|needs| needs["words"].is_object()),
        "Ben's own needs are in view"
    );
    record("commute-sleeper", &observation);
}
