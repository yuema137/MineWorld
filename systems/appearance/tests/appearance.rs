//! Appearance over a hand-built world with the real presence pack: a look seeded through the section
//! contract, reduced by its owner, and disclosed to whoever perceives the person — and to nobody else.

use std::collections::BTreeMap;

use mineworld_appearance::{Appearance, AppearanceSystem, Look, LookKey};
use mineworld_authoring::{AuthoredSection, Seeding};
use mineworld_contracts::{
    Causation, Component, EntityId, EntityKey, EntityType, EventEnvelope, LocalPosition, Location,
    Millimetres, Observation, PersonId, PlaceId, Rejection, Visibility, WorldTime,
};
use mineworld_kernel::{Emission, World};
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

/// A café with Alice and Bob in it, a street with Carol on it; every look seeded through the section
/// contract exactly as the World Pack loader seeds it, plus whatever `extra` facts a test adds.
fn town(
    looks: &[(&str, &str)],
    extra: impl Fn(&BTreeMap<EntityKey, EntityId>) -> Vec<Emission>,
) -> Town {
    let mut world = World::new();
    world.install(PresenceSystem).expect("presence");
    world
        .install(AppearanceSystem)
        .expect("appearance installs with no dependency");
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
    for (person, text) in looks {
        let authored: Appearance =
            serde_saphyr::from_str(text).expect("a valid appearance decodes");
        let read = world.read();
        let seeding = Seeding::new(&read, &keys);
        facts.extend(
            AppearanceSystem::seed(&seeding, keys[&key(person)], &authored).expect("seeded"),
        );
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
        let providers: [&dyn PerceptionProvider; 2] = [&PresenceSystem, &AppearanceSystem];
        mineworld_presence::observe(&self.world, self.id(observer), GENESIS, &providers)
    }
}

/// The `look` an observation discloses about `subject`, decoded with this pack's type.
fn disclosed_look(observation: &Observation<Value>, subject: EntityId) -> Option<String> {
    let record = observation
        .entity(subject)?
        .components()
        .iter()
        .find(|record| *record.component_type() == Look::COMPONENT_TYPE)?;
    let payload = record.payload_for::<Look>().expect("labelled as a look");
    assert!(
        payload.is_object(),
        "a JSON object, which is what a Godot reader takes: {payload}"
    );
    let look: Look = serde_json::from_value(payload.clone()).expect("decodes");
    Some(look.look().as_str().to_owned())
}

#[test]
fn a_seeded_look_is_reduced_by_its_owner_into_the_persons_look() {
    let town = town(
        &[
            ("alice", "{ look: alice-moreau }"),
            ("bob", "{ look: bob-achterberg }"),
        ],
        |_| Vec::new(),
    );
    let appeared: Vec<&EventEnvelope> = town
        .genesis
        .iter()
        .filter(|fact| fact.event_type().as_str() == "appeared")
        .collect();
    assert_eq!(appeared.len(), 2, "one `appeared` per seeded section");
    assert!(
        appeared
            .iter()
            .all(|fact| *fact.caused_by() == Causation::WorldGenesis),
        "a seeded look is a genesis fact"
    );
    assert!(
        appeared
            .iter()
            .all(|fact| matches!(fact.visibility(), Visibility::Public)),
        "an appearance is public"
    );
    let subjects: Vec<EntityId> = appeared.iter().map(|fact| fact.subjects()[0]).collect();
    assert_eq!(subjects, vec![town.id("alice"), town.id("bob")]);
    let read = town.world.read();
    for (person, look) in [("alice", "alice-moreau"), ("bob", "bob-achterberg")] {
        assert_eq!(
            read.component::<Look>(town.id(person))
                .map(|held| held.look().as_str()),
            Some(look),
            "{person}'s Look is the authored key"
        );
    }
    assert!(
        read.component::<Look>(town.id("carol")).is_none(),
        "a person with no `appearance:` section has no look — not an invented one"
    );
}

#[test]
fn a_look_is_disclosed_to_whoever_perceives_the_person_and_to_nobody_else() {
    let town = town(
        &[
            ("alice", "{ look: alice-moreau }"),
            ("bob", "{ look: bob-achterberg }"),
            ("carol", "{ look: carol-mensah }"),
        ],
        |_| Vec::new(),
    );
    let alice = town.observation("alice");
    assert_eq!(
        disclosed_look(&alice, town.id("bob")).as_deref(),
        Some("bob-achterberg"),
        "Alice perceives Bob, so she is told his look"
    );
    assert_eq!(
        disclosed_look(&alice, town.id("alice")).as_deref(),
        Some("alice-moreau"),
        "and her own"
    );
    assert!(
        alice.entity(town.id("carol")).is_none(),
        "Carol is on the street: not perceived from the café (presence's rule, observed)"
    );
    let carol = town.observation("carol");
    assert_eq!(
        disclosed_look(&carol, town.id("alice")),
        None,
        "and Carol, on the street, is told nothing about Alice"
    );
    assert_eq!(
        disclosed_look(&carol, town.id("carol")).as_deref(),
        Some("carol-mensah")
    );
}

#[test]
fn a_look_is_validated_by_its_own_type_as_it_is_decoded() {
    let decode = |text: &str| serde_saphyr::from_str::<Appearance>(text);
    assert_eq!(
        decode("{ look: alice-moreau }")
            .expect("valid")
            .look()
            .as_str(),
        "alice-moreau"
    );
    for accepted in ["a", "-", "0", "x1-y2"] {
        assert!(
            decode(&format!("{{ look: \"{accepted}\" }}")).is_ok(),
            "{accepted:?} is a look: the pattern is exactly [a-z0-9-]{{1,48}}"
        );
    }
    assert!(
        decode(&format!("{{ look: \"{}\" }}", "a".repeat(48))).is_ok(),
        "48 bytes is the bound, inclusive"
    );
    let too_long = format!("{{ look: {} }}", "a".repeat(49));
    for refused in [
        "{ look: \"\" }",
        too_long.as_str(),
        "{ look: Alice }",
        "{ look: \"alice moreau\" }",
        "{ look: alice_moreau }",
        "{ look: \"élise\" }",
    ] {
        let error = decode(refused).expect_err("refused by LookKey");
        assert!(
            error.to_string().contains("a look must be"),
            "refused with LookKey's own message for {refused:?}: {error}"
        );
    }
    let error = decode("{ look: a, colour: red }").expect_err("an unknown key");
    assert!(
        error.to_string().contains("colour"),
        "an unknown key is named: {error}"
    );
    let error = decode("{}").expect_err("no look");
    assert!(error.to_string().contains("look"), "{error}");
    assert!(
        decode("alice-moreau").is_err(),
        "a bare string is not an appearance section"
    );
}

#[test]
fn a_look_seeded_for_a_place_is_refused_by_its_owner() {
    let town = town(&[], |_| Vec::new());
    let read = town.world.read();
    let seeding = Seeding::new(&read, &town.keys);
    let look = LookKey::new("the-copper-kettle").expect("a key");
    assert_eq!(
        AppearanceSystem::seed(&seeding, town.id("cafe"), &Appearance::new(look))
            .expect_err("not a person"),
        Rejection::PreconditionFailed
    );
}
