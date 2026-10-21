//! S30-f §5.11 (`pr-s30f-attribute-consumers.md`): one fixture world, one hand-written oracle. Through the
//! real `mineworld server`, the observations Bob and Alice receive, read by the model crate's typed reader
//! and composition (the decision model's surface and the reference every other surface is held to): Bob's
//! glance and inspect list of Alice, Alice's who-I-am, how-I-speak and own sheet, the absences
//! (X-S30-4), and the masked variant (AC-f-8). The Python, 2D and 3D surfaces are compared with the same
//! oracle by their own tests.

mod headless;
mod support;

use std::path::Path;

use headless::fresh;
use mineworld_attributes_model::Disclosed;
use mineworld_attributes_model::disclosed::{compose, inspect, sheet};
use mineworld_contracts::{Component, EntityId, EntityType};
use mineworld_naming::DisplayName;
use mineworld_server::WireObservation;
use serde_json::Value;
use support::{Client, Server};

const WORLD: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/attributes-consumers"
);
const MASKED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/attributes-consumers.masked.yaml"
);
const ORACLE: &str = include_str!("fixtures/attributes-consumers.expected.json");
const ENTITIES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../entities");

/// The budgets of §5.4.
const GLANCE: usize = 160;
const ABOUT: usize = 600;
const MANNER: usize = 160;

fn oracle() -> Value {
    serde_json::from_str(ORACLE).expect("the oracle is JSON")
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .expect("a list")
        .iter()
        .map(|item| item.as_str().expect("a string").to_owned())
        .collect()
}

/// The perceived person whose disclosed display name is `name`.
fn person_named(observation: &WireObservation, name: &str) -> EntityId {
    observation
        .entities()
        .iter()
        .filter(|entity| matches!(entity.entity_type(), EntityType::Person))
        .find(|entity| {
            entity.components().iter().any(|record| {
                *record.component_type() == DisplayName::COMPONENT_TYPE
                    && record.payload().get("name").and_then(Value::as_str) == Some(name)
            })
        })
        .map(|entity| entity.id())
        .unwrap_or_else(|| panic!("{name} is perceived"))
}

fn disclosed(observation: &WireObservation, subject: EntityId) -> Disclosed {
    Disclosed::of(observation.entity(subject).expect("perceived"))
}

/// Everything `observation` discloses about `subject`, as text, for the absence checks.
fn everything_about(observation: &WireObservation, subject: EntityId) -> String {
    serde_json::to_string(
        &observation
            .entity(subject)
            .map(|entity| entity.components().to_vec()),
    )
    .expect("encodes")
}

fn assert_absent(surface: &str, haystack: &str, absent: &Value) {
    for needle in strings(&absent["ids"])
        .into_iter()
        .chain(strings(&absent["texts"]))
    {
        assert!(
            !haystack.contains(&needle),
            "{surface}: '{needle}' must not appear"
        );
    }
}

async fn seated(server: &Server, seat: &str) -> WireObservation {
    let mut client = Client::connect(server.address).await;
    client.join(seat).await;
    client.observation().await
}

/// AC-f-1, AC-f-2, AC-f-3, AC-f-7, AC-f-12.
#[tokio::test]
async fn bob_and_alice_see_what_the_oracle_says_and_nothing_private() {
    let oracle = oracle();
    let server = Server::start(&["server", WORLD, "--packs", ENTITIES]).await;

    let bobs = seated(&server, "bob").await;
    let alice = person_named(&bobs, "Alice Moreau");
    let of_alice = disclosed(&bobs, alice);
    assert_eq!(
        compose(of_alice.entries(), Some("observed"), Some(GLANCE)),
        oracle["alice_seen_by_bob"]["glance"]
            .as_str()
            .expect("glance")
    );
    assert_eq!(
        inspect(of_alice.entries()),
        strings(&oracle["alice_seen_by_bob"]["inspect"])
    );
    let carol = person_named(&bobs, "Carol Okafor");
    assert_eq!(
        compose(
            disclosed(&bobs, carol).entries(),
            Some("observed"),
            Some(GLANCE)
        ),
        oracle["carol_seen_by_bob"]["glance"]
            .as_str()
            .expect("glance")
    );
    assert_absent(
        "Bob's observation of Alice",
        &everything_about(&bobs, alice),
        &oracle["absent_from_bob"],
    );

    let own = seated(&server, "alice").await;
    let me = disclosed(&own, own.observer());
    assert_eq!(
        compose(me.entries(), Some("prompt"), Some(ABOUT)),
        oracle["alice_own"]["about"].as_str().expect("about")
    );
    assert_eq!(
        compose(me.entries(), Some("speech"), Some(MANNER)),
        oracle["alice_own"]["manner"].as_str().expect("manner")
    );
    assert_eq!(sheet(me.entries()), strings(&oracle["alice_own"]["sheet"]));
    assert_absent(
        "Alice's own observation",
        &everything_about(&own, own.observer()),
        &oracle["absent_from_alice_own"],
    );
    let bob = person_named(&own, "Bob Achterberg");
    assert_eq!(
        compose(
            disclosed(&own, bob).entries(),
            Some("observed"),
            Some(GLANCE)
        ),
        oracle["bob_seen_by_alice"]["glance"]
            .as_str()
            .expect("glance")
    );
    assert_absent(
        "Alice's observation of Bob",
        &everything_about(&own, bob),
        &oracle["absent_from_alice"],
    );
}

/// A copy of `from` at `to`, every file.
fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a directory");
    for entry in std::fs::read_dir(from).expect("readable") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("copied");
        }
    }
}

/// AC-f-8: under a world mask without `core.hair-colour` (and the person files without it, since a
/// masked-out attribute cannot be stated), hair colour is on no surface and the rest is unchanged.
#[tokio::test]
async fn a_masked_out_attribute_is_on_no_surface() {
    let oracle = oracle();
    let root = fresh("s30f-masked");
    let world = root.join("attributes-consumers");
    copy_tree(Path::new(WORLD), &world);
    std::fs::copy(MASKED, world.join("configure/attributes.yaml")).expect("masked configuration");
    for person in ["alice", "bob"] {
        let path = world.join(format!("people/{person}.yaml"));
        let text = std::fs::read_to_string(&path).expect("readable");
        let kept: Vec<&str> = text
            .lines()
            .filter(|line| !line.trim_start().starts_with("hair-colour:"))
            .collect();
        std::fs::write(&path, kept.join("\n") + "\n").expect("written");
    }
    let world = world.to_str().expect("utf-8").to_owned();
    let server = Server::start(&["server", &world, "--packs", ENTITIES]).await;
    let bobs = seated(&server, "bob").await;
    let alice = person_named(&bobs, "Alice Moreau");
    assert_eq!(
        compose(
            disclosed(&bobs, alice).entries(),
            Some("observed"),
            Some(GLANCE)
        ),
        oracle["masked"]["alice_seen_by_bob"]["glance"]
            .as_str()
            .expect("glance")
    );
    let own = seated(&server, "alice").await;
    for (surface, observation, subject) in [
        ("Bob's observation of Alice", &bobs, alice),
        ("Alice's own observation", &own, own.observer()),
    ] {
        assert_absent(
            surface,
            &everything_about(observation, subject),
            &oracle["masked"]["absent_everywhere"],
        );
    }
}
