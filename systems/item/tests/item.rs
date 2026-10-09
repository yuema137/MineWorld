//! Item kinds over a hand-built world: a kind seeded through the section contract exactly as the World
//! Pack loader seeds it, reduced by its owner alone, refused when it names no Item, and validated by
//! its own type as it is decoded.

use std::collections::BTreeMap;

use mineworld_authoring::{AuthoredSection, Seeding};
use mineworld_contracts::{
    Causation, EntityId, EntityKey, EntityType, EventEnvelope, ItemId, Rejection, WorldTime,
};
use mineworld_item::{AuthoredItem, ItemKind, ItemKindDeclared, ItemSystem, is_declared};
use mineworld_kernel::{Emission, KernelError, SystemIdentity, World};
use mineworld_presence::PerceptionProvider;

const GENESIS: WorldTime = WorldTime::from_seconds(0);

fn key(name: &str) -> EntityKey {
    EntityKey::new(name).expect("a key")
}

fn authored(yaml: &str) -> AuthoredItem {
    serde_saphyr::from_str(yaml).expect("a valid section decodes")
}

/// A café, a person, and three Item entities: coffee and tea, which carry an `item:` section, and a
/// lantern, which does not.
fn town() -> (World, BTreeMap<EntityKey, EntityId>) {
    let mut world = World::new();
    world
        .install(ItemSystem)
        .expect("item installs with no dependency");
    let mut keys = BTreeMap::new();
    for (name, entity_type) in [
        ("cafe", EntityType::Place),
        ("alice", EntityType::Person),
        ("coffee", EntityType::Item),
        ("lantern", EntityType::Item),
        ("tea", EntityType::Item),
    ] {
        let id = world
            .create_entity(key(name), entity_type)
            .expect("created");
        keys.insert(key(name), id);
    }
    (world, keys)
}

fn seed(
    world: &World,
    keys: &BTreeMap<EntityKey, EntityId>,
    subject: &str,
    yaml: &str,
) -> Result<Vec<Emission>, Rejection> {
    let read = world.read();
    let seeding = Seeding::new(&read, keys);
    ItemSystem::seed(&seeding, keys[&key(subject)], &authored(yaml))
}

fn item(keys: &BTreeMap<EntityKey, EntityId>, name: &str) -> ItemId {
    ItemId::new(keys[&key(name)], EntityType::Item).expect("an item")
}

#[test]
fn a_seeded_kind_is_one_public_fact_its_owner_reduces_into_item_kind() {
    let (mut world, keys) = town();
    let mut facts = seed(
        &world,
        &keys,
        "coffee",
        "{ category: drink, name: Flat White }",
    )
    .expect("seeded");
    facts.extend(seed(&world, &keys, "tea", "category: drink\nname: Tea").expect("seeded"));
    let genesis: Vec<EventEnvelope> = world.genesis(GENESIS, facts).expect("begins");

    assert_eq!(genesis.len(), 2, "one item-kind-declared per section");
    for fact in &genesis {
        assert_eq!(fact.event_type().as_str(), "item-kind-declared");
        assert_eq!(*fact.caused_by(), Causation::WorldGenesis);
        assert_eq!(
            *fact.visibility(),
            mineworld_contracts::Visibility::Public,
            "what sort of thing a kind is, is public"
        );
    }
    let read = world.read();
    assert_eq!(
        read.component::<ItemKind>(keys[&key("coffee")])
            .map(|kind| (kind.category().as_str(), kind.name().as_str())),
        Some(("drink", "Flat White"))
    );
    assert_eq!(
        genesis[0].payload().schema_version().get(),
        2,
        "item-kind-declared is schema 2: it carries the name"
    );
    assert!(is_declared(&read, item(&keys, "coffee")));
    assert!(is_declared(&read, item(&keys, "tea")));
    assert!(
        !is_declared(&read, item(&keys, "lantern")),
        "an Item entity whose file has no `item:` section is inert: no pack trades it"
    );
}

#[test]
fn a_category_is_validated_by_its_own_type_as_it_is_decoded() {
    let decode =
        |text: &str| serde_saphyr::from_str::<<ItemSystem as AuthoredSection>::Authored>(text);
    assert_eq!(
        decode("{ category: hot-drink-2, name: Cocoa }")
            .expect("valid")
            .category()
            .as_str(),
        "hot-drink-2"
    );
    assert!(
        decode(&format!("{{ category: {}, name: X }}", "x".repeat(32))).is_ok(),
        "32 bytes is the bound, inclusive"
    );
    let long = format!("{{ category: {}, name: X }}", "x".repeat(33));
    for refused in [
        "{ category: \"\", name: X }",
        long.as_str(),
        "{ category: Drink, name: X }",
        "{ category: hot drink, name: X }",
        "{ category: -drink, name: X }",
        "{ category: drink-, name: X }",
        "{ category: café, name: X }",
    ] {
        let error = decode(refused).expect_err("refused by Category");
        assert!(
            error.to_string().contains("a category must be"),
            "refused with Category's own message for {refused:?}: {error}"
        );
    }
    let unknown = decode("{ category: drink, name: X, price: 3 }").expect_err("unknown key");
    assert!(
        unknown.to_string().contains("price"),
        "an unknown key is refused by name: {unknown}"
    );
}

/// R-PK-2 (step-11 TD-13): a kind's name is required, and refused by `ItemName` as it is decoded,
/// at its line — empty, 65 bytes, a control character, surrounding space. 64 bytes, any script, and
/// two kinds sharing one name are fine: a name is display, never identity.
#[test]
fn a_name_is_required_and_validated_by_its_own_type_as_it_is_decoded() {
    let decode =
        |text: &str| serde_saphyr::from_str::<<ItemSystem as AuthoredSection>::Authored>(text);
    assert_eq!(
        decode("category: food\nname: Pain au chocolat")
            .expect("valid")
            .name()
            .as_str(),
        "Pain au chocolat"
    );
    assert!(
        decode(&format!("{{ category: food, name: {} }}", "x".repeat(64))).is_ok(),
        "64 bytes is the bound, inclusive"
    );
    assert!(
        decode("{ category: food, name: Café crème }").is_ok(),
        "any script"
    );

    let missing = decode("{ category: food }").expect_err("a name is required");
    assert!(
        missing.to_string().contains("name"),
        "a missing name is refused by name: {missing}"
    );
    let long = format!("category: food\nname: {}", "x".repeat(65));
    for refused in [
        "category: food\nname: \"\"",
        long.as_str(),
        "category: food\nname: \" Bread\"",
        "category: food\nname: \"Bread \"",
        "category: food\nname: \"Bre\\tad\"",
    ] {
        let error = decode(refused).expect_err("refused by ItemName");
        let text = error.to_string();
        assert!(
            text.contains("an item name must be"),
            "refused with ItemName's own message for {refused:?}: {text}"
        );
        assert!(
            text.contains("line 2"),
            "refused at the name's line for {refused:?}: {text}"
        );
    }
}

/// R-PK-2 (step-11 SD-D10, TD-13): whoever perceives a place is disclosed the world's catalogue on it
/// — every declared kind, `{ item, category, name }`, in `ItemId` order, as authored; the lantern,
/// which declares no kind, is not in it. A person or an item discloses nothing, and a world without
/// kinds discloses no catalogue at all.
#[test]
fn a_place_discloses_the_catalogue_of_kinds_in_item_order() {
    let (mut world, keys) = town();
    // Tea first: the catalogue is in ItemId order, never in the order the kinds were declared.
    let mut facts = seed(&world, &keys, "tea", "{ category: drink, name: Tea }").expect("seeded");
    facts.extend(
        seed(
            &world,
            &keys,
            "coffee",
            "{ category: drink, name: Flat White }",
        )
        .expect("seeded"),
    );
    world.genesis(GENESIS, facts).expect("begins");

    let read = world.read();
    let alice = keys[&key("alice")];
    let cafe = keys[&key("cafe")];
    let records = ItemSystem.discloses(&read, alice, cafe);
    assert_eq!(records.len(), 1, "one catalogue on the place");
    let record = &records[0];
    assert_eq!(record.entity(), cafe, "the record is about the place");
    assert_eq!(record.component_type().as_str(), "item-catalogue");
    let entry = |name: &str, category: &str, entity: &str| {
        serde_json::json!({
            "item": serde_json::to_value(item(&keys, entity)).expect("encodes"),
            "category": category,
            "name": name,
        })
    };
    assert_eq!(
        *record.payload(),
        serde_json::json!({
            "kinds": [entry("Flat White", "drink", "coffee"), entry("Tea", "drink", "tea")]
        }),
        "every declared kind, in ItemId order (coffee before tea), as authored"
    );

    for subject in ["alice", "coffee", "lantern"] {
        assert!(
            ItemSystem
                .discloses(&read, alice, keys[&key(subject)])
                .is_empty(),
            "{subject} discloses nothing: items are never perceived as entities"
        );
    }

    let (bare, keys) = town();
    assert!(
        ItemSystem
            .discloses(&bare.read(), keys[&key("alice")], keys[&key("cafe")])
            .is_empty(),
        "a world with no kinds discloses no catalogue"
    );
}

#[test]
fn a_kind_seeded_for_a_non_item_is_refused_by_its_owner() {
    let (world, keys) = town();
    for subject in ["cafe", "alice"] {
        assert_eq!(
            seed(&world, &keys, subject, "{ category: drink, name: Tea }")
                .expect_err("not an item"),
            Rejection::PreconditionFailed,
            "{subject}"
        );
    }
}

/// The owner decides at reduction too (`ARC-26`): a declaration built by hand about a place — its
/// payload labelled as an item, which the contract's typed id cannot see through — is refused, and
/// nothing is written.
#[test]
fn a_declaration_about_a_place_stated_past_the_seed_is_refused_at_reduction() {
    let (mut world, keys) = town();
    // A real item reference, re-pointed at the café: the label still says "item".
    let mut reference = serde_json::to_value(item(&keys, "coffee")).expect("encodes");
    reference["entity"] = serde_json::to_value(keys[&key("cafe")]).expect("encodes");
    let forged =
        serde_json::json!({ "item": reference, "category": "drink", "name": "Flat White" });
    let fact = Emission::new::<ItemKindDeclared>(
        serde_json::to_vec(&forged).expect("encodes"),
        mineworld_contracts::Visibility::Public,
    );
    match world.genesis(GENESIS, vec![fact]) {
        Err(KernelError::FactRefusedByOwner {
            system,
            event_type,
            reason,
        }) => {
            assert_eq!(system, ItemSystem::ID);
            assert_eq!(event_type.as_str(), "item-kind-declared");
            assert_eq!(reason, Rejection::PreconditionFailed);
        }
        other => panic!("item must refuse as the owner, but genesis returned {other:?}"),
    }
    assert!(
        world
            .read()
            .component::<ItemKind>(keys[&key("cafe")])
            .is_none(),
        "a refused declaration writes nothing"
    );
}
