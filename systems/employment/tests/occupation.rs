//! S30-f §5.10 (`pr-s30f-attribute-consumers.md`; step-30 §9.4, the named exception): `employment`
//! owns `employment.occupation`, projected from an optional job `title:`. Where the world enables
//! `attributes`, the holder's `employment` payload carries the block; another observer is told nothing;
//! without `attributes`, or without a title, every byte is what it was.

mod support;

use std::path::PathBuf;

use mineworld_attributes::AttributesSystem;
use mineworld_attributes_model::declaration::AttributesFile;
use mineworld_attributes_model::facts::COMPOSED_SCHEMA;
use mineworld_attributes_model::{AttributesComposed, Catalogue, DataFile};
use mineworld_authoring::{AuthoredSection, DataDirectory, DataOrigin, DataSource, Seeding};
use mineworld_contracts::{Component, EntityId, Visibility};
use mineworld_employment::{Employment, EmploymentSystem};
use mineworld_kernel::{Emission, World};
use serde_json::{Value, json};
use support::{GENESIS, JOB, Town, compose, job, key, providers};

const OCCUPATION: &str = include_str!("../attributes/occupation.yaml");

/// Alice's job, titled.
const TITLED: &str = "{ employer: cafe-company, workplace: cafe, from: \"08:00\", until: \"12:00\", \
                      wage: 120, produces: { coffee: 4, croissant: 2 }, title: barista }";

/// The town begun with Alice titled and Bob untitled, and `attributes` installed with a catalogue of
/// the shipped occupation file when `with_attributes`.
fn town(with_attributes: bool) -> (World, EntityId, EntityId) {
    let mut world = compose(true);
    if with_attributes {
        world.install(AttributesSystem).expect("attributes");
    }
    let (mut world, keys, mut facts) = Town::assemble(world, false);
    {
        let read = world.read();
        let seeding = Seeding::new(&read, &keys);
        for (person, written) in [("alice", TITLED), ("bob", JOB)] {
            facts.extend(
                EmploymentSystem::seed(&seeding, keys[&key(person)], &job(written).expect("a job"))
                    .expect("seeded"),
            );
        }
    }
    if with_attributes {
        let origin = DataOrigin {
            pack: "employment".to_owned(),
            source: DataSource::SystemPack,
            directory: DataDirectory::from_static("attributes"),
            path: PathBuf::from("attributes/occupation.yaml"),
        };
        let decoded: AttributesFile = serde_saphyr::from_str(OCCUPATION).expect("decodes");
        let catalogue =
            Catalogue::build(&[(origin, DataFile::Attributes(decoded))]).expect("builds");
        let composed = AttributesComposed {
            schema: COMPOSED_SCHEMA,
            catalogue: catalogue.projection(),
            population: None,
            composition: None,
        };
        facts.insert(
            0,
            Emission::new::<AttributesComposed>(
                serde_json::to_vec(&composed).expect("encodes"),
                Visibility::SystemInternal,
            ),
        );
    }
    world.genesis(GENESIS, facts).expect("begins");
    (world, keys[&key("alice")], keys[&key("bob")])
}

/// The `employment` payload `observer` is told about `subject`, if any.
fn employment_seen(world: &World, observer: EntityId, subject: EntityId) -> Option<Value> {
    let seen = mineworld_presence::observe(world, observer, GENESIS, &providers());
    let entity = seen.entity(subject)?;
    entity
        .components()
        .iter()
        .find(|record| *record.component_type() == Employment::COMPONENT_TYPE)
        .map(|record| record.payload().clone())
}

#[test]
fn the_holder_is_told_her_occupation_and_another_observer_nothing() {
    let (world, alice, bob) = town(true);
    let own = employment_seen(&world, alice, alice).expect("Alice is told her own");
    assert_eq!(
        own["attributes"],
        json!({ "schema": 2, "entries": [ { "attribute": "employment.occupation", "kind": "text",
            "label": "Occupation", "facets": ["prompt"], "rank": 205,
            "value": { "text": "barista" }, "text": "works as barista" } ] })
    );
    assert_eq!(own["job"]["title"], "barista");
    assert_eq!(employment_seen(&world, bob, alice), None, "Bob is told nothing of it");
    let bobs = employment_seen(&world, bob, bob).expect("Bob is told his own");
    assert_eq!(bobs.get("attributes"), None, "an untitled job has no occupation");
}

/// M-f-9's test: without `attributes`, the payload has no `attributes` member, and an untitled job's
/// bytes have no `title`.
#[test]
fn without_attributes_or_a_title_the_bytes_are_what_they_were() {
    let (world, alice, bob) = town(false);
    let own = employment_seen(&world, alice, alice).expect("Alice is told her own");
    assert_eq!(own.get("attributes"), None, "{own}");
    assert_eq!(own["job"]["title"], "barista");
    let bobs = employment_seen(&world, bob, bob).expect("Bob is told his own");
    assert_eq!(bobs.get("attributes"), None);
    assert_eq!(bobs["job"].get("title"), None, "{bobs}");
    let plain = Town::begun(true);
    let alices = employment_seen(&plain.world, plain.id("alice"), plain.id("alice")).expect("own");
    let keys: Vec<&String> = alices["job"].as_object().expect("a job").keys().collect();
    assert_eq!(
        keys,
        ["employer", "from", "produces", "until", "wage", "workplace"],
        "a job without title or manager has exactly the fields it had"
    );
}

#[test]
fn a_title_is_one_to_forty_bytes_without_a_control_character() {
    let with = |title: &str| {
        job(&format!(
            "{{ employer: cafe-company, workplace: cafe, from: \"08:00\", until: \"12:00\", \
             wage: 1, title: \"{title}\" }}"
        ))
    };
    assert!(with("barista").is_ok());
    assert!(with(&"a".repeat(40)).is_ok());
    let refused = with(&"a".repeat(41)).expect_err("41 bytes");
    assert!(refused.to_string().contains("job title"), "{refused}");
    assert!(with("").is_err(), "empty");
    assert!(with("bar\\tista").is_err(), "a control character");
}
