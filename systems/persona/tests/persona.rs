//! Persona over a hand-built world with the real presence pack: a character seeded through the section
//! contract, reduced by its owner, and disclosed to its holder — and to nobody else.

use std::collections::BTreeMap;

use mineworld_authoring::{AuthoredSection, Seeding};
use mineworld_contracts::{
    Component, EntityId, EntityKey, EntityType, EventEnvelope, LocalPosition, Location,
    Millimetres, Observation, PersonId, PlaceId, WorldTime,
};
use mineworld_kernel::World;
use mineworld_persona::{Persona, PersonaComponent, PersonaSystem};
use mineworld_presence::{PerceptionProvider, PresenceSystem, arrival};
use serde_json::Value;

const GENESIS: WorldTime = WorldTime::from_seconds(0);

struct Town {
    world: World,
    keys: BTreeMap<EntityKey, EntityId>,
    genesis: Vec<EventEnvelope>,
}

fn key(name: &str) -> EntityKey {
    EntityKey::new(name).expect("a key")
}

/// A café with Alice and Bob in it, every character seeded through the section contract exactly as
/// the World Pack loader seeds it.
fn town(personas: &[(&str, &str)]) -> Town {
    let mut world = World::new();
    world.install(PresenceSystem).expect("presence");
    world
        .install(PersonaSystem)
        .expect("persona installs with no dependency");
    let mut keys = BTreeMap::new();
    let cafe = world
        .create_entity(key("cafe"), EntityType::Place)
        .expect("a place");
    keys.insert(key("cafe"), cafe);
    for person in ["alice", "bob"] {
        let id = world
            .create_entity(key(person), EntityType::Person)
            .expect("a person");
        keys.insert(key(person), id);
    }
    let mut facts = Vec::new();
    for (x, person) in [(1_000, "alice"), (2_000, "bob")] {
        let person = PersonId::new(keys[&key(person)], EntityType::Person).expect("a person");
        let place = PlaceId::new(cafe, EntityType::Place).expect("a place");
        let at = Location::in_place(place).with_local(LocalPosition::on_ground(
            Millimetres::new(x),
            Millimetres::new(1_000),
        ));
        facts.push(arrival(&world.read(), person, at).expect("admitted"));
    }
    for (person, text) in personas {
        let authored: Persona = serde_saphyr::from_str(text).expect("a valid persona decodes");
        let read = world.read();
        let seeding = Seeding::new(&read, &keys);
        facts.extend(PersonaSystem::seed(&seeding, keys[&key(person)], &authored).expect("seeded"));
    }
    let genesis = world.genesis(GENESIS, facts).expect("begins");
    Town {
        world,
        keys,
        genesis,
    }
}

impl Town {
    fn id(&self, name: &str) -> EntityId {
        self.keys[&key(name)]
    }

    fn observation(&self, observer: &str) -> Observation<Value> {
        let providers: [&dyn PerceptionProvider; 2] = [&PresenceSystem, &PersonaSystem];
        mineworld_presence::observe(&self.world, self.id(observer), GENESIS, &providers)
    }
}

/// The `persona` an observation discloses about `subject`, if any.
fn disclosed_persona(observation: &Observation<Value>, subject: EntityId) -> Option<Value> {
    let record = observation
        .entity(subject)?
        .components()
        .iter()
        .find(|record| *record.component_type() == PersonaComponent::COMPONENT_TYPE)?;
    Some(
        record
            .payload_for::<PersonaComponent>()
            .expect("labelled as a persona")
            .clone(),
    )
}

const ALICE: &str = "summary: Runs the café counter. Warm with regulars.\n\
                      traits: [warm, dry]\n\
                      speech: Short sentences, a little dry.\n";

#[test]
fn a_persona_is_reduced_by_its_owner_into_the_persons_component() {
    let town = town(&[("alice", ALICE)]);
    let read = town.world.read();
    let held = read
        .component::<PersonaComponent>(town.id("alice"))
        .expect("Alice has a persona");
    assert_eq!(
        held.persona().summary(),
        "Runs the café counter. Warm with regulars."
    );
    assert_eq!(held.persona().traits(), ["warm", "dry"]);
    assert_eq!(
        held.persona().speech(),
        Some("Short sentences, a little dry.")
    );
    assert!(
        read.component::<PersonaComponent>(town.id("bob")).is_none(),
        "a person with no `persona:` section has no character, and none is invented"
    );
    let authored = town
        .genesis
        .iter()
        .filter(|fact| fact.event_type().as_str() == "persona-authored")
        .count();
    assert_eq!(
        authored, 1,
        "one `persona-authored` fact per seeded section"
    );
}

#[test]
fn a_persona_is_disclosed_to_its_holder_and_to_nobody_else() {
    let town = town(&[("alice", ALICE)]);
    let alice = town.observation("alice");
    let own =
        disclosed_persona(&alice, town.id("alice")).expect("Alice perceives her own character");
    assert_eq!(
        own["summary"], "Runs the café counter. Warm with regulars.",
        "the holder sees the summary as authored"
    );
    let bob = town.observation("bob");
    assert!(
        bob.entity(town.id("alice")).is_some(),
        "Bob perceives Alice, so the observation carries her entity"
    );
    assert_eq!(
        disclosed_persona(&bob, town.id("alice")),
        None,
        "and Bob, who perceives her, is told nothing of her character"
    );
}

#[test]
fn a_persona_is_refused_when_it_breaks_a_bound() {
    let decode =
        |text: &str| serde_saphyr::from_str::<<PersonaSystem as AuthoredSection>::Authored>(text);
    let nine_traits = format!(
        "summary: x\ntraits: [{}]\n",
        (0..9)
            .map(|n| format!("t{n}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let long_summary = format!("summary: {}\n", "x".repeat(281));
    for (refused, reason, position) in [
        ("summary: Warm\nmood: calm\n", "mood", "line 2 column 1"),
        (long_summary.as_str(), "281", "line 1 column 10"),
        (nine_traits.as_str(), "9", "line 2 column 9"),
        ("summary: \"\"\n", "must not be empty", "line 1 column 10"),
        ("traits: [warm]\n", "summary", "line 1 column 1"),
    ] {
        let error = decode(refused).expect_err("refused");
        let text = error.to_string();
        assert!(
            text.contains(reason),
            "{refused:?} is refused naming {reason:?}: {text}"
        );
        assert!(
            text.contains(position),
            "{refused:?} is refused at {position}: {text}"
        );
    }
    assert!(
        decode(&format!("summary: {}\n", "x".repeat(280))).is_ok(),
        "280 bytes is the bound, inclusive"
    );
}
