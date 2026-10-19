//! S30-c (`pr-s30c-attributes-pack.md` §8): the `attributes` pack through the real binary — the commands
//! on the fixture world, and the checkpoint through `mineworld server`: Alice's own observation lists her
//! non-hidden attributes exactly, Bob's lists her perceivable ones only, a perceived-as quantity as words.

mod headless;
mod support;

use headless::{mineworld, stderr, stdout};
use mineworld_attributes_model::Traits;
use mineworld_contracts::{Component, EntityId, EntityType};
use mineworld_naming::DisplayName;
use mineworld_server::WireObservation;
use serde_json::Value;
use support::{Client, Server};

/// The fixture world (§5.13).
const TOWN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/attributes-town"
);

#[test]
fn command_catalogue_lists_every_declaration_of_the_fixture() {
    let output = mineworld(&["attributes", "catalogue", TOWN, "--json"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let document: Value = serde_json::from_str(&stdout(&output)).expect("one JSON document");
    assert_eq!(document["attributes"].as_array().map(Vec::len), Some(93));
    assert_eq!(document["namespaces"]["attributes-town"], 93);
}

#[test]
fn command_show_prints_each_value_with_its_origin() {
    let output = mineworld(&["attributes", "show", TOWN, "--person", "alice", "--json"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let document: Value = serde_json::from_str(&stdout(&output)).expect("one JSON document");
    let alice = &document["people"][0];
    assert_eq!(alice["person"], "alice");
    let stature = alice["values"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|value| value["attribute"] == "attributes-town.stature")
        .expect("stature");
    assert_eq!(stature["value"], serde_json::json!({ "quantity": "1.660" }));
    assert_eq!(stature["origin"], "authored");
}

#[test]
fn command_check_is_clean_on_the_fixture() {
    let output = mineworld(&["attributes", "check", TOWN]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(
        stdout(&output).contains("93 attribute(s); clean"),
        "{}",
        stdout(&output)
    );
}

/// The `traits` entries an observation discloses about `subject`, as (attribute, value, text).
fn traits_of(
    observation: &WireObservation,
    subject: EntityId,
) -> Vec<(String, Option<Value>, Option<String>)> {
    let Some(record) = observation.entity(subject).and_then(|entity| {
        entity
            .components()
            .iter()
            .find(|record| *record.component_type() == Traits::COMPONENT_TYPE)
    }) else {
        return Vec::new();
    };
    record.payload()["entries"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|entry| {
            (
                entry["attribute"].as_str().unwrap_or_default().to_owned(),
                entry.get("value").cloned(),
                entry.get("text").and_then(Value::as_str).map(str::to_owned),
            )
        })
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

/// AC-c-1: the checkpoint's two views of Alice.
#[tokio::test]
async fn alices_own_view_and_bobs_view_of_her_are_filtered_for_the_observer() {
    let server = Server::start(&["server", TOWN]).await;
    let mut alice = Client::connect(server.address).await;
    alice.join("alice").await;
    let own = alice.observation().await;
    let attributes: Vec<String> = traits_of(&own, own.observer())
        .into_iter()
        .map(|(attribute, ..)| attribute)
        .collect();
    for expected in [
        "stature",
        "hair-colour",
        "chronotype",
        "glasses",
        "languages",
        "motto",
        "birth",
        "age-years",
        "age-band",
        "religion",
    ] {
        assert!(
            attributes.contains(&format!("attributes-town.{expected}")),
            "{expected} in {attributes:?}"
        );
    }
    assert!(
        !attributes
            .iter()
            .any(|attribute| attribute.ends_with(".secret")),
        "hidden: {attributes:?}"
    );

    let mut bob = Client::connect(server.address).await;
    bob.join("bob").await;
    let seen = bob.observation().await;
    let of_alice = traits_of(&seen, person_named(&seen, "Alice Moreau"));
    let names: Vec<&str> = of_alice
        .iter()
        .map(|(attribute, ..)| attribute.as_str())
        .collect();
    assert_eq!(
        names,
        [
            "attributes-town.age-band",
            "attributes-town.glasses",
            "attributes-town.hair-colour",
            "attributes-town.stature"
        ]
    );
    let stature = &of_alice[3];
    assert_eq!(
        (stature.1.as_ref(), stature.2.as_deref()),
        (None, Some("of average height"))
    );
}
