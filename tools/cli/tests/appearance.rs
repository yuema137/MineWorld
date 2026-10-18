//! AS26a-1, -2 and -3 (`pr-s26a-appearance.md` §8), through the real binary: a look reaches a client
//! over the wire, is absent from a world without the pack, and is the same after a kill and a restart
//! of the same save.
//!
//! The oracles are the authored keys and the disclosed names, never entity ids.

mod support;

use std::collections::BTreeMap;

use mineworld_appearance::Look;
use mineworld_contracts::{Component, EntityId, EntityType};
use mineworld_naming::DisplayName;
use mineworld_server::WireObservation;
use support::{Client, SaveDir, Server};

const BODIES_YARD: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/bodies-yard");

/// The look an observation discloses about `subject`, decoded with appearance's own type.
fn look_of(observation: &WireObservation, subject: EntityId) -> Option<String> {
    let record = observation
        .entity(subject)?
        .components()
        .iter()
        .find(|record| *record.component_type() == Look::COMPONENT_TYPE)?;
    let look: Look = serde_json::from_value(record.payload_for::<Look>().ok()?.clone()).ok()?;
    Some(look.look().as_str().to_owned())
}

/// The display name an observation discloses about `subject`, decoded with naming's own type.
fn name_of(observation: &WireObservation, subject: EntityId) -> Option<String> {
    let record = observation
        .entity(subject)?
        .components()
        .iter()
        .find(|record| *record.component_type() == DisplayName::COMPONENT_TYPE)?;
    let name: DisplayName =
        serde_json::from_value(record.payload_for::<DisplayName>().ok()?.clone()).ok()?;
    Some(name.name().as_str().to_owned())
}

/// Every perceived person, by the name the observation discloses, with the look it discloses.
fn looks_by_name(observation: &WireObservation) -> BTreeMap<String, Option<String>> {
    observation
        .entities()
        .iter()
        .filter(|entity| matches!(entity.entity_type(), EntityType::Person))
        .map(|entity| {
            let name = name_of(observation, entity.id())
                .expect("every perceived person discloses a display name");
            (name, look_of(observation, entity.id()))
        })
        .collect()
}

#[tokio::test]
async fn every_person_the_visitor_perceives_is_disclosed_with_their_look() {
    let server = Server::start(&["server", support::PACK]).await;
    let mut visitor = Client::connect(server.address).await;
    visitor.join("visitor").await;

    let first = visitor.observation().await;

    let expected: BTreeMap<String, Option<String>> = [
        ("Alice Moreau", "alice-moreau"),
        ("Bob Achterberg", "bob-achterberg"),
        ("Vera Lindgren", "vera-lindgren"),
        ("Wes Calloway", "wes-calloway"),
    ]
    .into_iter()
    .map(|(name, look)| (name.to_owned(), Some(look.to_owned())))
    .collect();
    assert_eq!(
        looks_by_name(&first),
        expected,
        "the four people the visitor perceives, each disclosed with the look its file authors"
    );
}

#[tokio::test]
async fn a_world_without_appearance_discloses_no_look() {
    let server = Server::start(&["server", BODIES_YARD]).await;
    let mut ada = Client::connect(server.address).await;
    ada.join("ada").await;

    let seen = ada.observation().await;

    let persons = seen
        .entities()
        .iter()
        .filter(|entity| matches!(entity.entity_type(), EntityType::Person))
        .count();
    assert!(persons >= 2, "ada perceives at least two people: {persons}");
    for entity in seen.entities() {
        assert!(
            entity
                .components()
                .iter()
                .all(|record| *record.component_type() != Look::COMPONENT_TYPE),
            "{:?} carries no look in a world without appearance",
            entity.id()
        );
    }
}

#[tokio::test]
async fn a_look_is_the_same_after_the_server_is_killed_and_restarted() {
    let save = SaveDir::new("appearance-restart");
    let command = ["server", support::PACK, "--save", save.path()];

    let mut first = Server::start(&command).await;
    let mut before_window = Client::connect(first.address).await;
    before_window.join("visitor").await;
    let before = looks_by_name(&before_window.observation().await);
    let before_instance = first.status().await["instance"].clone();
    assert_eq!(before.len(), 4, "four people perceived before the kill");
    assert!(
        before.values().all(Option::is_some),
        "each is disclosed a look before the kill: {before:?}"
    );

    first.kill();
    let second = Server::start(&command).await;
    let mut after_window = Client::connect(second.address).await;
    after_window.join("visitor").await;
    let after = looks_by_name(&after_window.observation().await);

    assert_eq!(
        second.status().await["instance"],
        before_instance,
        "the restarted server is the same world instance"
    );
    assert_eq!(after, before, "and discloses the same looks");
}
