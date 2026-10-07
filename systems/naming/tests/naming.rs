//! Naming over a hand-built world with the real presence pack: a name seeded through the section
//! contract, reduced by its owner, and disclosed to whoever perceives the person — and to nobody else.

use std::collections::BTreeMap;

use mineworld_authoring::{AuthoredSection, Seeding};
use mineworld_contracts::{
    Causation, Component, EntityId, EntityKey, EntityType, EventEnvelope, LocalPosition, Location,
    Millimetres, Observation, PersonId, PlaceId, Rejection, WorldTime,
};
use mineworld_kernel::{Emission, World};
use mineworld_naming::{DisplayName, Name, Named, NamingSystem, names_in};
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

/// A café with Alice and Bob in it, a street with Carol on it; every name seeded through the section
/// contract exactly as the World Pack loader seeds it, plus whatever `extra` facts a test adds.
fn town(
    names: &[(&str, &str)],
    extra: impl Fn(&BTreeMap<EntityKey, EntityId>) -> Vec<Emission>,
) -> Town {
    let mut world = World::new();
    world.install(PresenceSystem).expect("presence");
    world
        .install(NamingSystem)
        .expect("naming installs with no dependency");
    let mut keys = BTreeMap::new();
    for place in ["cafe", "street"] {
        let id = world
            .create_entity(key(place), EntityType::Place)
            .expect("a place");
        keys.insert(key(place), id);
    }
    for person in ["alice", "bob", "carol"] {
        let id = world
            .create_entity(key(person), EntityType::Person)
            .expect("a person");
        keys.insert(key(person), id);
    }
    let mut facts = Vec::new();
    for (person, place, x) in [
        ("alice", "cafe", 1_000),
        ("bob", "cafe", 2_000),
        ("carol", "street", 1_000),
    ] {
        let person = PersonId::new(keys[&key(person)], EntityType::Person).expect("a person");
        let place = PlaceId::new(keys[&key(place)], EntityType::Place).expect("a place");
        let at = Location::in_place(place).with_local(LocalPosition::on_ground(
            Millimetres::new(x),
            Millimetres::new(1_000),
        ));
        facts.push(arrival(&world.read(), person, at).expect("admitted"));
    }
    for (person, name) in names {
        let authored: Name = serde_saphyr::from_str(name).expect("a valid name decodes");
        let read = world.read();
        let seeding = Seeding::new(&read, &keys);
        facts.extend(NamingSystem::seed(&seeding, keys[&key(person)], &authored).expect("seeded"));
    }
    facts.extend(extra(&keys));
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
        let providers: [&dyn PerceptionProvider; 2] = [&PresenceSystem, &NamingSystem];
        mineworld_presence::observe(&self.world, self.id(observer), GENESIS, &providers)
    }
}

/// The `display-name` an observation discloses about `subject`, decoded with this pack's type.
fn disclosed_name(observation: &Observation<Value>, subject: EntityId) -> Option<String> {
    let record = observation
        .entity(subject)?
        .components()
        .iter()
        .find(|record| *record.component_type() == DisplayName::COMPONENT_TYPE)?;
    let payload = record
        .payload_for::<DisplayName>()
        .expect("labelled as a display name");
    assert!(
        payload.is_object(),
        "a JSON object, which is what a Godot reader takes: {payload}"
    );
    let name: DisplayName = serde_json::from_value(payload.clone()).expect("decodes");
    Some(name.name().as_str().to_owned())
}

#[test]
fn a_seeded_name_is_reduced_by_its_owner_into_the_persons_display_name() {
    let town = town(
        &[("alice", "Alice Moreau"), ("bob", "Bob Achterberg")],
        |_| Vec::new(),
    );
    let named: Vec<&EventEnvelope> = town
        .genesis
        .iter()
        .filter(|fact| fact.event_type().as_str() == "named")
        .collect();
    assert_eq!(named.len(), 2, "one `named` per seeded section");
    assert!(
        named
            .iter()
            .all(|fact| *fact.caused_by() == Causation::WorldGenesis),
        "a seeded name is a genesis fact"
    );
    let read = town.world.read();
    for (person, name) in [("alice", "Alice Moreau"), ("bob", "Bob Achterberg")] {
        assert_eq!(
            read.component::<DisplayName>(town.id(person))
                .map(|held| held.name().as_str()),
            Some(name),
            "{person}'s DisplayName is the authored name"
        );
    }
    assert!(
        read.component::<DisplayName>(town.id("carol")).is_none(),
        "a person with no `name:` section has no name — not an invented one"
    );
}

#[test]
fn a_name_is_disclosed_to_whoever_perceives_the_person_and_to_nobody_else() {
    let town = town(
        &[
            ("alice", "Alice Moreau"),
            ("bob", "Bob Achterberg"),
            ("carol", "Carol Mensah"),
        ],
        |_| Vec::new(),
    );
    let alice = town.observation("alice");
    assert_eq!(
        disclosed_name(&alice, town.id("bob")).as_deref(),
        Some("Bob Achterberg"),
        "Alice perceives Bob, so she is told his name"
    );
    assert_eq!(
        disclosed_name(&alice, town.id("alice")).as_deref(),
        Some("Alice Moreau"),
        "and her own"
    );
    assert!(
        alice.entity(town.id("carol")).is_none(),
        "Carol is on the street: not perceived from the café (presence's rule, observed)"
    );
    let carol = town.observation("carol");
    assert_eq!(
        disclosed_name(&carol, town.id("alice")),
        None,
        "and Carol, on the street, is told nothing about Alice"
    );
    assert_eq!(
        disclosed_name(&carol, town.id("carol")).as_deref(),
        Some("Carol Mensah")
    );
}

#[test]
fn a_name_is_validated_by_its_own_type_as_it_is_decoded() {
    let decode =
        |text: &str| serde_saphyr::from_str::<<NamingSystem as AuthoredSection>::Authored>(text);
    assert_eq!(
        decode("Alice Moreau").expect("valid").as_str(),
        "Alice Moreau"
    );
    assert_eq!(
        decode("Zoë Ångström").expect("any script").as_str(),
        "Zoë Ångström"
    );
    let long = "x".repeat(65);
    for refused in [
        "\"\"",
        long.as_str(),
        "\"\\tAlice\"",
        "\"Al\\nice\"",
        "\" Alice\"",
    ] {
        let error = decode(refused).expect_err("refused by Name");
        assert!(
            error.to_string().contains("a name must be"),
            "refused with Name's own message for {refused:?}: {error}"
        );
    }
    assert!(
        decode(&"x".repeat(64)).is_ok(),
        "64 bytes is the bound, inclusive"
    );
}

#[test]
fn a_name_seeded_for_a_place_is_refused_by_its_owner() {
    let town = town(&[], |_| Vec::new());
    let read = town.world.read();
    let seeding = Seeding::new(&read, &town.keys);
    let name = Name::new("The Copper Kettle").expect("a name");
    assert_eq!(
        NamingSystem::seed(&seeding, town.id("cafe"), &name).expect_err("not a person"),
        Rejection::PreconditionFailed
    );
}

#[test]
fn names_in_projects_the_latest_name_of_each_person_from_the_facts() {
    let town = town(
        &[("alice", "Alice Moreau"), ("bob", "Bob Achterberg")],
        |keys| {
            let alice = PersonId::new(keys[&key("alice")], EntityType::Person).expect("a person");
            vec![Named::new(alice, Name::new("Alice Moreau-Lind").expect("a name")).emission()]
        },
    );
    let names = names_in(&town.genesis);
    assert_eq!(names.len(), 2);
    assert_eq!(
        names[&town.id("alice")].as_str(),
        "Alice Moreau-Lind",
        "the later fact wins"
    );
    assert_eq!(names[&town.id("bob")].as_str(), "Bob Achterberg");
    assert_eq!(
        town.world
            .read()
            .component::<DisplayName>(town.id("alice"))
            .map(|held| held.name().as_str()),
        Some("Alice Moreau-Lind"),
        "and the projection agrees with the reduced state"
    );
}
