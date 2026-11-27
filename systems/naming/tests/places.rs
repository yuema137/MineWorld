//! Place names (`ARC-363`, pr-pdn C1): a place's `name:` section in both shapes, the `place-named`
//! fact, and the two routes a place's name reaches an observer — the same two its tags take.
//!
//! Three places over the real presence and movement packs: A (`hall`) opens onto B (`yard`), and C
//! (`attic`) opens onto nothing. An observer stands in each. The names are this test's own, not a
//! shipped world's.

use std::collections::BTreeMap;

use mineworld_authoring::{AuthoredSection, Seeding};
use mineworld_contracts::{
    Component, EntityId, EntityKey, EntityType, EventEnvelope, LocalPosition, Location,
    Millimetres, Observation, PersonId, PlaceId, WorldTime,
};
use mineworld_kernel::World;
use mineworld_movement::{MovementSystem, passage};
use mineworld_naming::{DisplayName, DoorNames, NameSpec, NamingSystem, names_in};
use mineworld_presence::{PerceptionProvider, PresenceSystem, arrival};
use serde_json::Value;

const GENESIS: WorldTime = WorldTime::from_seconds(0);

struct Yard {
    world: World,
    keys: BTreeMap<EntityKey, EntityId>,
    genesis: Vec<EventEnvelope>,
}

fn key(name: &str) -> EntityKey {
    EntityKey::new(name).expect("a key")
}

fn decode(text: &str) -> Result<NameSpec, serde_saphyr::Error> {
    serde_saphyr::from_str::<<NamingSystem as AuthoredSection>::Authored>(text)
}

/// Hall (named, with a form) ⇄ yard (named by the short form); attic (named) alone. One person in each.
fn yard(names: &[(&str, &str)]) -> Yard {
    let mut world = World::new();
    world.install(PresenceSystem).expect("presence");
    world.install(MovementSystem).expect("movement");
    world.install(NamingSystem).expect("naming");
    let mut keys = BTreeMap::new();
    for place in ["hall", "yard", "attic"] {
        let id = world
            .create_entity(key(place), EntityType::Place)
            .expect("a place");
        keys.insert(key(place), id);
    }
    for person in ["ana", "ben", "cy"] {
        let id = world
            .create_entity(key(person), EntityType::Person)
            .expect("a person");
        keys.insert(key(person), id);
    }
    let place = |name: &str| PlaceId::new(keys[&key(name)], EntityType::Place).expect("a place");
    let mut facts = vec![passage(place("hall"), None, place("yard"), None)];
    for (person, at) in [("ana", "hall"), ("ben", "yard"), ("cy", "attic")] {
        let person = PersonId::new(keys[&key(person)], EntityType::Person).expect("a person");
        let at = Location::in_place(place(at)).with_local(LocalPosition::on_ground(
            Millimetres::new(1_000),
            Millimetres::new(1_000),
        ));
        facts.push(arrival(&world.read(), person, at).expect("admitted"));
    }
    for (subject, text) in names {
        let authored = decode(text).expect("a valid section");
        let read = world.read();
        let seeding = Seeding::new(&read, &keys);
        facts.extend(NamingSystem::seed(&seeding, keys[&key(subject)], &authored).expect("seeded"));
    }
    let genesis = world.genesis(GENESIS, facts).expect("begins");
    Yard {
        world,
        keys,
        genesis,
    }
}

const NAMES: &[(&str, &str)] = &[
    (
        "hall",
        "default: Test Hall\nforms:\n  zh_Hans: 测试大厅\n  fr: Salle d'essai",
    ),
    ("yard", "Test Yard"),
    ("attic", "Test Attic"),
];

impl Yard {
    fn id(&self, name: &str) -> EntityId {
        self.keys[&key(name)]
    }

    fn observation(&self, observer: &str) -> Observation<Value> {
        let providers: [&dyn PerceptionProvider; 3] =
            [&PresenceSystem, &MovementSystem, &NamingSystem];
        mineworld_presence::observe(&self.world, self.id(observer), GENESIS, &providers)
    }
}

/// The payload of `component_type` disclosed about `subject`, if any.
fn disclosed(
    observation: &Observation<Value>,
    subject: EntityId,
    component_type: &str,
) -> Option<Value> {
    observation
        .entity(subject)?
        .components()
        .iter()
        .find(|record| record.component_type().as_str() == component_type)
        .map(|record| record.payload().clone())
}

/// Every entity an observation names by `display-name` or under `door-names`.
fn named_anywhere(observation: &Observation<Value>) -> Vec<EntityId> {
    let mut found = Vec::new();
    for entity in observation.entities() {
        for record in entity.components() {
            if *record.component_type() == DisplayName::COMPONENT_TYPE {
                found.push(entity.id());
            }
            if *record.component_type() == DoorNames::COMPONENT_TYPE {
                let doors: DoorNames =
                    serde_json::from_value(record.payload().clone()).expect("door names");
                found.extend(doors.leads_to.iter().map(|door| door.to.entity_id()));
            }
        }
    }
    found
}

#[test]
fn a_place_name_in_both_shapes_is_seeded_reduced_and_projected() {
    let yard = yard(NAMES);
    let facts: Vec<&EventEnvelope> = yard
        .genesis
        .iter()
        .filter(|fact| fact.event_type().as_str() == "place-named")
        .collect();
    assert_eq!(facts.len(), 3, "one place-named per named place");
    let read = yard.world.read();
    let hall = read
        .component::<DisplayName>(yard.id("hall"))
        .expect("the hall is named");
    assert_eq!(hall.name().as_str(), "Test Hall");
    assert_eq!(
        serde_json::to_value(hall).expect("JSON"),
        serde_json::json!({"name": "Test Hall", "forms": {"fr": "Salle d'essai", "zh_Hans": "测试大厅"}})
    );
    let short = read
        .component::<DisplayName>(yard.id("yard"))
        .expect("the yard is named");
    assert_eq!(
        serde_json::to_string(short).expect("JSON"),
        "{\"name\":\"Test Yard\"}",
        "no forms, no `forms` key: a reader of `name` sees what it always saw"
    );
    let names = names_in(&yard.genesis);
    assert_eq!(names[&yard.id("attic")].as_str(), "Test Attic");
}

#[test]
fn a_place_name_reaches_exactly_the_observers_its_tags_reach() {
    let yard = yard(NAMES);
    let (hall, place_b, attic) = (yard.id("hall"), yard.id("yard"), yard.id("attic"));

    // Ana stands in the hall: its own name, and the yard's over the doorway — not the yard's own record.
    let ana = yard.observation("ana");
    assert!(disclosed(&ana, hall, "display-name").is_some());
    let doors = disclosed(&ana, hall, "door-names").expect("the hall's doorway is named");
    assert_eq!(
        doors,
        serde_json::json!({"leads_to": [{
            "to": {"entity": place_b.to_string(), "entity_type": "place"},
            "name": "Test Yard",
        }]})
    );
    assert!(
        ana.entity(place_b).is_none(),
        "the yard itself is not perceived"
    );

    // Ben in the yard sees the hall over the doorway, with its forms.
    let ben = yard.observation("ben");
    let doors: DoorNames =
        serde_json::from_value(disclosed(&ben, place_b, "door-names").expect("named"))
            .expect("shape");
    assert_eq!(doors.leads_to.len(), 1);
    assert_eq!(doors.leads_to[0].to.entity_id(), hall);
    assert_eq!(doors.leads_to[0].forms.len(), 2);
    assert!(disclosed(&ben, hall, "display-name").is_none());

    // Cy in the attic, with no doorway to the hall: told neither name.
    let cy = yard.observation("cy");
    assert_eq!(
        named_anywhere(&cy),
        vec![attic],
        "only the attic Cy stands in"
    );
    assert!(
        disclosed(&cy, attic, "door-names").is_none(),
        "a place with no doorway discloses no door names — absence, not an empty record"
    );

    // Every name an observation carries is of the observer's place or of a doorway's destination.
    for (who, here, beyond) in [("ana", hall, Some(place_b)), ("ben", place_b, Some(hall))] {
        let named = named_anywhere(&yard.observation(who));
        assert!(named.contains(&here));
        assert!(
            named.iter().all(|id| *id == here || Some(*id) == beyond),
            "{who}: {named:?}"
        );
    }
}

#[test]
fn an_unnamed_destination_is_left_out_of_the_door_names() {
    let yard = yard(&[("hall", "Test Hall")]);
    let ana = yard.observation("ana");
    assert!(
        disclosed(&ana, yard.id("hall"), "door-names").is_none(),
        "the yard is unnamed: no entry, never an empty name"
    );
    let ben = yard.observation("ben");
    assert!(disclosed(&ben, yard.id("yard"), "door-names").is_some());
    assert!(disclosed(&ben, yard.id("yard"), "display-name").is_none());
}

#[test]
fn a_malformed_name_section_is_refused_with_its_rule_and_position() {
    let cases: &[(&str, &str)] = &[
        ("\"\"", "a name must be"),
        ("default: \"\"", "a name must be"),
        ("default: Hall\nforms:\n  zh-Hans: 大厅", "a locale must be"),
        ("default: Hall\nforms:\n  ZH: 大厅", "a locale must be"),
        (
            "default: Hall\nforms:\n  fr: Hall",
            "repeats the default name",
        ),
        (
            "forms:\n  fr: Salle\ndefault: Salle",
            "repeats the default name",
        ),
        ("default: Hall\ncolour: red", "unknown field"),
        ("forms:\n  fr: Salle", "missing field"),
    ];
    for (text, rule) in cases {
        let error = decode(text).expect_err("refused").to_string();
        assert!(
            error.contains(rule),
            "{text:?}: expected {rule:?} in {error}"
        );
        assert!(
            error.contains("line"),
            "{text:?}: the refusal names its position: {error}"
        );
    }
    let mut seventeen = String::from("default: Hall\nforms:\n");
    for code in [
        "aa", "ab", "ac", "ad", "ae", "af", "ag", "ah", "ai", "aj", "ak", "al", "am", "an", "ao",
        "ap", "aq",
    ] {
        seventeen.push_str(&format!("  {code}: Hall {code}\n"));
    }
    let error = decode(&seventeen).expect_err("17 forms").to_string();
    assert!(error.contains("at most 16"), "{error}");
    for locale in ["en", "zh_Hans", "pt_BR", "sr_Latn_RS", "fil"] {
        let text = format!("default: Hall\nforms:\n  {locale}: Other");
        assert!(decode(&text).is_ok(), "{locale} is a catalog stem");
    }
}
