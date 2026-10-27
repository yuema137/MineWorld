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

    /// S31-b2 W1-attr: the copy with `attributes` enabled too, a `configure/attributes.yaml` naming a
    /// roles-only population profile, `roles` (the profile's `roles:` entries) and `authored`
    /// (person key, text appended to its file).
    fn with_attributes(
        name: &str,
        configuration: &str,
        sections: &[(&str, &str)],
        roles: &str,
        authored: &[(&str, &str)],
    ) -> Self {
        let mut all: Vec<(&str, &str)> = sections.to_vec();
        all.extend_from_slice(authored);
        let town = Self::with(name, configuration, &all);
        let world = town.world();
        let manifest = world.join("world.yaml");
        let text = std::fs::read_to_string(&manifest).expect("reads");
        let text = replaced_once(
            &text,
            "  - appearance\n  - notices\n",
            "  - appearance\n  - notices\n  - attributes\n",
        );
        let text = replaced_once(
            &text,
            "\nconfigure:\n  - notices\n",
            "\nconfigure:\n  - notices\n  - attributes\n",
        );
        std::fs::write(&manifest, text).expect("writes");
        std::fs::write(
            world.join("configure/attributes.yaml"),
            "format: 1\nprofile: notices-town\nseed: s31b2-fixture\n",
        )
        .expect("writes");
        std::fs::create_dir_all(world.join("populations")).expect("writable");
        std::fs::write(
            world.join("populations/notices-town.yaml"),
            format!("format: 1\nprofile: notices-town\nversion: 1\nroles:\n{roles}"),
        )
        .expect("writes");
        town
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

// ---- S31-b2 (`pr-s31b2-notices-attributes.md` §7): notices as attributes, and the topics record ----

/// W1-attr's roles: bob (tag `regular`) follows `prices.ingredients` by role.
const ROLES: &str =
    "  - { when: { tags: [regular] }, add: { notices.subscriptions: [prices.ingredients] } }\n";

/// W1-attr's authored value: carol follows both topics, written in `attributes:`.
const CAROL_AUTHORED: &str =
    "attributes: { notices.subscriptions: [news.daily, prices.ingredients] }\n";

/// W1-attr: the fixture with `attributes` enabled, no section, bob by role and carol authored.
fn attribute_town(name: &str) -> Town {
    Town::with_attributes(
        name,
        CONFIGURATION,
        &[],
        ROLES,
        &[("carol", CAROL_AUTHORED)],
    )
}

/// W2: the topics renamed (`prices.ingredients` → `market.daily`) and a third topic, published by dev.
fn w2_configuration() -> String {
    CONFIGURATION.replacen("prices.ingredients", "market.daily", 1)
        + "  weather.warnings:\n    body: text\n    publishers: [dev]\n    \
           valid_for: { min: \"1h\", max: \"1d\" }\n    per_day: 3\n"
}

/// A `subscribe` request as a client builds one.
fn subscribe(actor: EntityId, topic: &str) -> Value {
    json!({
        "actor": actor,
        "action_type": "subscribe",
        "target": null,
        "payload": { "action_type": "subscribe", "payload": { "topic": topic } },
        "actor_location": null,
    })
}

/// The record of component type `kind` an observation discloses about `subject`, as JSON.
fn record_about(observation: &WireObservation, subject: EntityId, kind: &str) -> Option<Value> {
    observation
        .entity(subject)?
        .components()
        .iter()
        .find(|record| record.component_type().as_str() == kind)
        .map(|record| record.payload().clone())
}

/// How many records of component type `kind` an observation carries, about anybody.
fn records_of(observation: &WireObservation, kind: &str) -> usize {
    observation
        .entities()
        .iter()
        .flat_map(|entity| entity.components())
        .filter(|record| record.component_type().as_str() == kind)
        .count()
}

/// AC-b2-3, AC-b2-4 through `mineworld validate`: W1-attr validates; W1-attr with `defaults.by_tag` or
/// a `notices:` section is refused `subscriptions-are-attributes`; W1 (no `attributes`) with both still
/// validates; in W2-attr a value from W1's topics is refused `UnknownTopic`, naming the vocabulary and
/// the nearest topic — the vocabulary is the world's own topics.
#[test]
fn notices_attributes_by_validate() {
    let (ok, printed) = attribute_town("notices-attr-valid").validate();
    assert!(ok, "W1-attr validates: {printed}");

    let with_defaults =
        format!("{CONFIGURATION}defaults:\n  by_tag: {{ regular: [news.daily] }}\n");
    let by_tag = Town::with_attributes(
        "notices-attr-by-tag",
        &with_defaults,
        &[],
        ROLES,
        &[("carol", CAROL_AUTHORED)],
    );
    let (ok, printed) = by_tag.validate();
    assert!(!ok, "defaults.by_tag beside attributes is refused");
    assert!(
        printed.contains("subscriptions-are-attributes"),
        "{printed}"
    );

    let section = Town::with_attributes(
        "notices-attr-section",
        CONFIGURATION,
        &[("dev", "notices: { subscribe: [news.daily] }\n")],
        ROLES,
        &[],
    );
    let (ok, printed) = section.validate();
    assert!(!ok, "a notices: section beside attributes is refused");
    assert!(
        printed.contains("subscriptions-are-attributes"),
        "{printed}"
    );

    let (ok, printed) = Town::with(
        "notices-plain-both",
        &with_defaults,
        &[("bob", BOTH), ("carol", BOTH)],
    )
    .validate();
    assert!(ok, "without attributes, both sources load: {printed}");

    let stale = Town::with_attributes(
        "notices-attr-w2",
        &w2_configuration(),
        &[],
        "  - { when: { tags: [regular] }, add: { notices.subscriptions: [market.daily] } }\n",
        &[(
            "carol",
            "attributes: { notices.subscriptions: [prices.ingredients] }\n",
        )],
    );
    let (ok, printed) = stale.validate();
    assert!(!ok, "W1's topic is not in W2's vocabulary");
    assert!(
        printed.contains("UnknownTopic") || printed.contains("notices.topics"),
        "{printed}"
    );
    assert!(
        printed.contains("market.daily"),
        "the nearest topic: {printed}"
    );
}

/// AC-b2-5, AC-b2-6 and AC-b2-7 on W1-attr through the server: bob's own `subscriptions` record carries
/// the rendered attribute; he is told nothing about carol's; his `topics` record lists both topics and
/// flips `subscribed` after a runtime `subscribe`, which also changes the projected attribute; killed
/// and restarted, the same; `mineworld replay` agrees.
#[tokio::test]
async fn notices_attributes_disclosed_and_resumed() {
    let town = attribute_town("notices-attr-run");
    let save = SaveDir::new("notices-attr-run-save");
    let world = town.world_str();
    let command = ["server", world.as_str(), "--save", save.path()];

    let mut first = Server::start(&command).await;
    let mut window = Client::connect(first.address).await;
    let (carol, _) = window.join("carol").await;
    drop(window);
    let mut window = Client::connect(first.address).await;
    let (bob, _) = window.join("bob").await;
    let seen = window.observation().await;
    let own = record_about(&seen, bob, "subscriptions").expect("bob's own record");
    assert_eq!(
        own["topics"],
        json!(["prices.ingredients"]),
        "by role: {own}"
    );
    let rendered = own["attributes"].to_string();
    assert!(
        rendered.contains("follows"),
        "the rendered phrase: {rendered}"
    );
    assert!(
        record_about(&seen, carol, "subscriptions").is_none()
            && record_about(&seen, carol, "topics").is_none(),
        "nothing about carol"
    );
    assert_eq!(
        records_of(&seen, "topics"),
        1,
        "one topics record: bob's own"
    );
    let topics = record_about(&seen, bob, "topics").expect("bob's topics record");
    assert_eq!(topics["topics"][0]["topic"], json!("news.daily"));
    assert_eq!(topics["topics"][0]["subscribed"], json!(false));
    assert_eq!(topics["topics"][1]["subscribed"], json!(true));

    let (_, result) = window.submit(subscribe(bob, "news.daily")).await;
    assert!(
        matches!(result, ActionResult::Accepted { .. }),
        "{result:?}"
    );
    let after = window
        .observation_where("bob follows news.daily", |observation| {
            record_about(observation, bob, "topics")
                .is_some_and(|topics| topics["topics"][0]["subscribed"] == json!(true))
        })
        .await;
    let own = record_about(&after, bob, "subscriptions").expect("bob's own record");
    assert!(
        own["attributes"].to_string().contains("news.daily"),
        "the projection follows the subscription: {own}"
    );
    let died = first.kill();
    assert!(died.killed(), "the server died of SIGKILL: {died:?}");
    drop(window);

    let second = Server::start(&command).await;
    let mut window = Client::connect(second.address).await;
    window.join("bob").await;
    let seen = window.observation().await;
    let own = record_about(&seen, bob, "subscriptions").expect("bob's record after the kill");
    assert_eq!(own["topics"], json!(["news.daily", "prices.ingredients"]));
    assert!(
        own["attributes"].to_string().contains("news.daily"),
        "{own}"
    );
    drop(window);
    drop(second);

    let (status, printed) = run_command(&["replay", &world, "--save", save.path()]);
    assert!(
        status.success(),
        "mineworld replay verifies the save: {printed}"
    );
}

/// AC-b2-5's W1 half and AC-b2-6 on W1 and W2: without `attributes`, the record has no `attributes`
/// field; every person's own `topics` record lists the world's topics in id order with its own
/// `may_publish` and `subscribed`, a non-subscriber's too, and nobody's about another person.
#[tokio::test]
async fn topics_record_without_attributes() {
    for (name, configuration, expected) in [
        (
            "notices-topics-w1",
            CONFIGURATION.to_owned(),
            vec!["news.daily", "prices.ingredients"],
        ),
        (
            "notices-topics-w2",
            w2_configuration(),
            vec!["market.daily", "news.daily", "weather.warnings"],
        ),
    ] {
        let town = Town::with(
            name,
            &configuration,
            &[("bob", "notices: { subscribe: [news.daily] }\n")],
        );
        let save = SaveDir::new(&format!("{name}-save"));
        let world = town.world_str();
        let server = Server::start(&["server", &world, "--save", save.path()]).await;
        for seat in ["hana", "bob", "dev"] {
            let mut window = Client::connect(server.address).await;
            let (me, _) = window.join(seat).await;
            let seen = window.observation().await;
            assert_eq!(records_of(&seen, "topics"), 1, "{seat}: its own only");
            let topics = record_about(&seen, me, "topics").expect("its topics record");
            let lines = topics["topics"].as_array().expect("lines");
            let names: Vec<&str> = lines
                .iter()
                .filter_map(|line| line["topic"].as_str())
                .collect();
            assert_eq!(names, expected, "{seat} in {name}");
            for line in lines {
                let topic = line["topic"].as_str().unwrap_or_default();
                let publisher = if topic == "weather.warnings" {
                    "dev"
                } else {
                    "hana"
                };
                assert_eq!(
                    line["may_publish"],
                    json!(seat == publisher),
                    "{seat} {topic}"
                );
                assert_eq!(
                    line["subscribed"],
                    json!(seat == "bob" && topic == "news.daily"),
                    "{seat} {topic}"
                );
            }
            if seat == "bob" {
                let own = record_about(&seen, me, "subscriptions").expect("bob's record");
                assert!(
                    own.get("attributes").is_none(),
                    "no attributes field: {own}"
                );
            }
            drop(window);
        }
        drop(server);
    }
}
