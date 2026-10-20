//! S31-b C6 — public notices through the real binary (`pr-s31b-notices.md` §8, AC-b-6 … AC-b-11;
//! `docs/decisions/ARC-115.md`).
//!
//! A scratch copy of `worlds/social-cafe` enables `notices`, declares the fixture's two topics and maps
//! the fixture's four people onto four of its seats:
//!
//! ```text
//! pia → hana   publishes news.daily and prices.ingredients; subscribes to nothing
//! sam → bob    subscribes to both
//! sue → carol  subscribes to both
//! nat → dev    subscribes to nothing
//! ```
//!
//! ```text
//! audience_through_perceived     AC-b-8   a protocol client on hana's seat publishes; `mineworld
//!                                         perceived` lists the notice for hana, bob and carol, not dev
//! kill_and_resume                AC-b-9   SIGKILL; the same command again: bob's board shows the notice;
//!                                         the next notice is numbered 2; `mineworld replay` reproduces
//! genesis_refusals_by_validate   AC-b-7   an undeclared topic in a section, a bound in the
//!                                         configuration, a non-person publisher: `mineworld validate`
//!                                         fails naming each
//! no_random_subscription         AC-b-6   30 days headless, seed 7, every seat on its bundled
//!                                         controller: no subscribed, unsubscribed or notice-published
//!                                         (long: CI only under R-LOAD-1)
//! removable                      AC-b-11  the copy with `notices` removed again validates and runs
//! ```
//!
//! The audience oracle is presence's export (`mineworld perceived`), never a value `notices` reports
//! about itself.

mod headless;
mod support;

use std::path::{Path, PathBuf};

use headless::{Scratch, fresh, mineworld, stderr, stdout};
use mineworld_contracts::{ActionResult, EntityId};
use mineworld_server::WireObservation;
use serde_json::{Value, json};
use support::{Client, SaveDir, Server, run_command};

/// The fixture's configuration (§8), with the fixture's publisher mapped to `hana`.
const CONFIGURATION: &str = "\
topics:
  news.daily:
    body: text
    publishers: [hana]
    valid_for: { min: \"1h\", max: \"7d\" }
    per_day: 2
  prices.ingredients:
    body: prices
    publishers: [hana]
    valid_for: { min: \"1h\", max: \"48h\" }
    per_day: 4
";

/// The subscribers' sections: bob (sam) and carol (sue) subscribe to both topics.
const SUBSCRIBED: [&str; 2] = ["bob", "carol"];
const BOTH: &str = "notices: { subscribe: [news.daily, prices.ingredients] }\n";

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("writable");
    for entry in std::fs::read_dir(from).expect("readable") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copied");
        }
    }
}

/// `text` with `old` replaced by `new`, where `old` occurs exactly once.
fn replaced_once(text: &str, old: &str, new: &str) -> String {
    assert_eq!(text.matches(old).count(), 1, "exactly one {old:?}");
    text.replacen(old, new, 1)
}

/// A scratch social-cafe with `notices` enabled and configured, and the subscribers' sections.
struct Town {
    scratch: Scratch,
}

impl Town {
    fn new(name: &str) -> Self {
        Self::with(name, CONFIGURATION, &[("bob", BOTH), ("carol", BOTH)])
    }

    /// The copy with `configuration` and `sections` (person key, text appended to its file).
    fn with(name: &str, configuration: &str, sections: &[(&str, &str)]) -> Self {
        let scratch = fresh(name);
        let world = scratch.join("social-cafe");
        copy_dir(Path::new(headless::PACK), &world);
        let manifest = world.join("world.yaml");
        let original = std::fs::read_to_string(&manifest).expect("reads");
        let enabled = replaced_once(
            &original,
            "  - appearance\n",
            "  - appearance\n  - notices\n",
        );
        // The pack may already configure other systems: join its `configure:` list, or start one.
        let configured = if enabled.contains("\nconfigure:\n") {
            replaced_once(&enabled, "\nconfigure:\n", "\nconfigure:\n  - notices\n")
        } else {
            format!("{enabled}\nconfigure:\n  - notices\n")
        };
        std::fs::write(&manifest, configured).expect("writes");
        std::fs::create_dir_all(world.join("configure")).expect("writable");
        std::fs::write(world.join("configure/notices.yaml"), configuration).expect("writes");
        for (person, text) in sections {
            let file = world.join(format!("people/{person}.yaml"));
            let original = std::fs::read_to_string(&file).expect("reads");
            std::fs::write(&file, format!("{original}{text}")).expect("writes");
        }
        Self { scratch }
    }

    fn world(&self) -> PathBuf {
        self.scratch.join("social-cafe")
    }

    fn world_str(&self) -> String {
        self.world().to_str().expect("a UTF-8 path").to_owned()
    }

    /// `mineworld validate` on the copy: success, and what it printed on either stream.
    fn validate(&self) -> (bool, String) {
        let output = mineworld(&["validate", &self.world_str()]);
        (
            output.status.success(),
            format!("{}{}", stdout(&output), stderr(&output)),
        )
    }
}

/// A `publish-notice` request as a client builds one: no identity, no instant.
fn publish(actor: EntityId, topic: &str, text: &str, valid_for: u32) -> Value {
    json!({
        "actor": actor,
        "action_type": "publish-notice",
        "target": null,
        "payload": { "action_type": "publish-notice", "payload": {
            "topic": topic, "body": { "text": text }, "valid_for": valid_for,
        } },
        "actor_location": null,
    })
}

/// How many `notice-published` facts `mineworld perceived` lists for `person`.
fn notices_perceived(world: &str, save: &str, person: &str) -> usize {
    let (status, printed) = run_command(&["perceived", world, "--save", save, "--person", person]);
    assert!(status.success(), "perceived --person {person}: {printed}");
    printed
        .lines()
        .filter(|line| line.split_whitespace().nth(2) == Some("notice-published"))
        .count()
}

/// The `notice-board` record an observation discloses about its own observer, as JSON.
fn own_board(observation: &WireObservation) -> Option<Value> {
    let me = observation.entity(observation.observer())?;
    me.components()
        .iter()
        .find(|record| record.component_type().as_str() == "notice-board")
        .map(|record| record.payload().clone())
}

#[tokio::test]
async fn audience_through_perceived() {
    let town = Town::new("notices-audience");
    let save = SaveDir::new("notices-audience-save");
    let world = town.world_str();
    let mut server = Server::start(&["server", &world, "--save", save.path()]).await;
    let mut window = Client::connect(server.address).await;
    let (hana, _) = window.join("hana").await;
    let (_, result) = window
        .submit(publish(
            hana,
            "news.daily",
            "The market opens late tomorrow.",
            3_600,
        ))
        .await;
    assert!(
        matches!(result, ActionResult::Accepted { .. }),
        "{result:?}"
    );
    let died = server.kill();
    assert!(died.killed(), "the server died of SIGKILL: {died:?}");
    drop(window);

    for (person, expected) in [("hana", 1), ("bob", 1), ("carol", 1), ("dev", 0)] {
        assert_eq!(
            notices_perceived(&world, save.path(), person),
            expected,
            "{person} perceives {expected} notice(s)"
        );
    }
}

#[tokio::test]
async fn kill_and_resume() {
    let town = Town::new("notices-resume");
    let save = SaveDir::new("notices-resume-save");
    let world = town.world_str();
    let command = ["server", world.as_str(), "--save", save.path()];

    let mut first = Server::start(&command).await;
    let mut window = Client::connect(first.address).await;
    let (hana, _) = window.join("hana").await;
    let (_, result) = window
        .submit(publish(hana, "news.daily", "before the crash", 7 * 86_400))
        .await;
    assert!(
        matches!(result, ActionResult::Accepted { .. }),
        "{result:?}"
    );
    let died = first.kill();
    assert!(died.killed(), "the server died of SIGKILL: {died:?}");
    drop(window);

    let second = Server::start(&command).await;
    let mut window = Client::connect(second.address).await;
    window.join("bob").await;
    let seen = window.observation().await;
    let board = own_board(&seen).expect("bob's board survived the kill");
    assert_eq!(
        board["notices"].as_array().map(Vec::len),
        Some(1),
        "{board}"
    );
    assert_eq!(board["notices"][0]["id"], json!(1), "{board}");
    drop(window);

    let mut window = Client::connect(second.address).await;
    let (hana_again, _) = window.join("hana").await;
    assert_eq!(hana_again, hana, "the seat resolves to the same Person");
    let (_, result) = window
        .submit(publish(hana, "news.daily", "after the crash", 7 * 86_400))
        .await;
    assert!(
        matches!(result, ActionResult::Accepted { .. }),
        "{result:?}"
    );
    let board = window
        .observation_where("hana's board shows the second notice", |observation| {
            own_board(observation).is_some_and(|board| board["notices"][0]["id"] == json!(2))
        })
        .await;
    assert_eq!(
        own_board(&board).expect("a board")["notices"]
            .as_array()
            .map(Vec::len),
        Some(1),
        "the second notice replaced the first on the board, numbered first + 1"
    );
    drop(window);
    drop(second);

    let (status, printed) = run_command(&["replay", &world, "--save", save.path()]);
    assert!(
        status.success(),
        "mineworld replay verifies the save: {printed}"
    );
}

#[test]
fn genesis_refusals_by_validate() {
    let (ok, printed) = Town::new("notices-valid").validate();
    assert!(ok, "the fixture copy validates: {printed}");

    let undeclared = Town::with(
        "notices-undeclared",
        CONFIGURATION,
        &[("bob", "notices: { subscribe: [news.weekly] }\n")],
    );
    let (ok, printed) = undeclared.validate();
    assert!(!ok, "an undeclared topic is refused");
    assert!(printed.contains("unknown-topic"), "{printed}");

    let bound = Town::with(
        "notices-bound",
        &CONFIGURATION.replacen("per_day: 2", "per_day: 0", 1),
        &[],
    );
    let (ok, printed) = bound.validate();
    assert!(!ok, "per_day 0 is refused");
    assert!(
        printed.contains("per_day is 1 to 96") && printed.contains("notices.yaml"),
        "{printed}"
    );

    let place = Town::with(
        "notices-place-publisher",
        &CONFIGURATION.replacen("publishers: [hana]", "publishers: [cafe]", 1),
        &[],
    );
    let (ok, printed) = place.validate();
    assert!(!ok, "a place as publisher is refused");
    assert!(printed.contains("cafe"), "{printed}");
}

/// AC-b-6. Long (30 days): run in CI only (R-LOAD-1).
#[test]
fn no_random_subscription() {
    let town = Town::new("notices-quiet");
    let save = town.scratch.join("quiet-save");
    let output = mineworld(&[
        "run",
        &town.world_str(),
        "--headless",
        "--seed",
        "7",
        "--days",
        "30",
        "--save",
        save.to_str().expect("a UTF-8 path"),
    ]);
    assert!(output.status.success(), "run: {}", stderr(&output));
    let tables = headless::Tables::read(&save);
    let types: Vec<String> = tables
        .facts
        .iter()
        .map(|(_, bytes)| {
            let fact: Value = serde_json::from_slice(bytes).expect("a fact is JSON");
            fact["event_type"].as_str().unwrap_or_default().to_owned()
        })
        .collect();
    assert!(
        types.iter().any(|kind| kind == "notices-configured"),
        "the run used the notices copy"
    );
    for forbidden in ["subscribed", "unsubscribed", "notice-published"] {
        assert!(
            !types.iter().any(|kind| kind == forbidden),
            "no seat {forbidden} at random: every offer of the pack is incomplete (Q-S31a-3)"
        );
    }
}

#[test]
fn removable() {
    let town = Town::new("notices-removable");
    let world = town.world();
    // Remove `notices` again: its line in `systems:`, its configuration, and the sections.
    // The manifest returns to the bundled pack's text, byte for byte.
    std::fs::copy(
        Path::new(headless::PACK).join("world.yaml"),
        world.join("world.yaml"),
    )
    .expect("restored");
    std::fs::remove_file(world.join("configure/notices.yaml")).expect("removes");
    for person in SUBSCRIBED {
        let file = world.join(format!("people/{person}.yaml"));
        let text = std::fs::read_to_string(&file).expect("reads");
        std::fs::write(&file, replaced_once(&text, BOTH, "")).expect("writes");
    }
    let (ok, printed) = town.validate();
    assert!(ok, "the world without notices validates: {printed}");
    let output = mineworld(&[
        "run",
        &town.world_str(),
        "--headless",
        "--seed",
        "7",
        "--days",
        "1",
    ]);
    assert!(output.status.success(), "and runs: {}", stderr(&output));
}
