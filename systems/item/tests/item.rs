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
    let mut facts = seed(&world, &keys, "coffee", "{ category: drink }").expect("seeded");
    facts.extend(seed(&world, &keys, "tea", "category: drink").expect("seeded"));
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
            .map(|kind| kind.category().as_str()),
        Some("drink")
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
        decode("{ category: hot-drink-2 }")
            .expect("valid")
            .category()
            .as_str(),
        "hot-drink-2"
    );
    assert!(
        decode(&format!("category: {}", "x".repeat(32))).is_ok(),
        "32 bytes is the bound, inclusive"
    );
    let long = format!("category: {}", "x".repeat(33));
    for refused in [
        "category: \"\"",
        long.as_str(),
        "category: Drink",
        "category: hot drink",
        "category: -drink",
        "category: drink-",
        "category: café",
    ] {
        let error = decode(refused).expect_err("refused by Category");
        assert!(
            error.to_string().contains("a category must be"),
            "refused with Category's own message for {refused:?}: {error}"
        );
    }
    let unknown = decode("{ category: drink, price: 3 }").expect_err("unknown key");
    assert!(
        unknown.to_string().contains("price"),
        "an unknown key is refused by name: {unknown}"
    );
}

#[test]
fn a_kind_seeded_for_a_non_item_is_refused_by_its_owner() {
    let (world, keys) = town();
    for subject in ["cafe", "alice"] {
        assert_eq!(
            seed(&world, &keys, subject, "category: drink").expect_err("not an item"),
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
    let forged = serde_json::json!({ "item": reference, "category": "drink" });
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
