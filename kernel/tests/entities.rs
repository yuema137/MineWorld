//! Behaviour tests for entity identity.
//!
//! What is being pinned here is determinism, not bookkeeping: identities are allocated in one
//! fixed order from a fixed starting point, they are never handed out twice, and a registry
//! written down and read back is the same registry — including the fact that it will not
//! re-allocate an identity it has already spent.

use mineworld_contracts::{
    ContractError, EntityId, EntityKey, EntityType, LifecycleState, Metadata, Tag, Tags,
};
use mineworld_kernel::{EntityRegistry, EntityRegistrySnapshot, KernelError};

fn key(value: &str) -> EntityKey {
    EntityKey::new(value).expect("the test keys are legal authoring names")
}

/// The authored form of an entity, as a World Pack would supply it.
fn authored_metadata() -> Metadata {
    Metadata {
        source_pack: "test-pack".to_owned(),
        source_path: "people/alice.yaml".to_owned(),
        authoring_note: None,
    }
}

/// Creates three entities in one order and returns the identities in the order they were
/// allocated.
fn create_three(registry: &mut EntityRegistry) -> Vec<EntityId> {
    vec![
        registry
            .create(key("alice"), EntityType::Person)
            .expect("a fresh key is accepted"),
        registry
            .create(key("cafe"), EntityType::Place)
            .expect("a fresh key is accepted"),
        registry
            .create(key("mug"), EntityType::Item)
            .expect("a fresh key is accepted"),
    ]
}

/// Allocation is a counter, and the counter is the whole of it: the first identity is 1, each
/// one after it is the previous plus one, and the values do not depend on the entity types, the
/// keys, or anything else about the entities.
#[test]
fn identities_are_allocated_in_one_monotonic_sequence_from_one() {
    let mut registry = EntityRegistry::new();
    let allocated = create_three(&mut registry);

    assert_eq!(
        allocated,
        vec![
            EntityId::from_raw(1),
            EntityId::from_raw(2),
            EntityId::from_raw(3)
        ]
    );
}

/// The determinism `AC-12` needs, stated as the property a replay depends on: the same
/// creations in the same order produce the same identities, in two registries that share
/// nothing.
#[test]
fn two_registries_given_the_same_creations_allocate_the_same_identities() {
    let mut first = EntityRegistry::new();
    let mut second = EntityRegistry::new();

    let from_first = create_three(&mut first);
    let from_second = create_three(&mut second);

    assert_eq!(from_first, from_second);
    assert_eq!(
        first.iter().map(|entity| entity.id()).collect::<Vec<_>>(),
        second.iter().map(|entity| entity.id()).collect::<Vec<_>>()
    );
}

/// Destroying an entity spends its identity permanently. The record stays — the event log
/// refers to it — and the next entity gets a new number rather than the dead one's.
#[test]
fn a_destroyed_entity_keeps_its_identity_and_the_next_entity_does_not_inherit_it() {
    let mut registry = EntityRegistry::new();
    let alice = registry
        .create(key("alice"), EntityType::Person)
        .expect("a fresh key is accepted");

    registry
        .transition(alice, LifecycleState::Destroyed)
        .expect("an active entity may be destroyed");

    let successor = registry
        .create(key("bob"), EntityType::Person)
        .expect("a fresh key is accepted");

    assert_ne!(successor, alice);
    assert_eq!(successor, EntityId::from_raw(alice.raw() + 1));
    assert_eq!(
        registry
            .get(alice)
            .expect("a destroyed entity still has a record")
            .lifecycle(),
        LifecycleState::Destroyed
    );
    assert_eq!(registry.len(), 2);
}

/// `Destroyed` is terminal, and the registry does not invent a way around the contract layer's
/// state machine.
#[test]
fn a_destroyed_entity_cannot_be_brought_back() {
    let mut registry = EntityRegistry::new();
    let alice = registry
        .create(key("alice"), EntityType::Person)
        .expect("a fresh key is accepted");
    registry
        .transition(alice, LifecycleState::Destroyed)
        .expect("an active entity may be destroyed");

    let refused = registry
        .transition(alice, LifecycleState::Active)
        .expect_err("resurrection is not a legal transition");

    assert!(matches!(
        refused,
        KernelError::Contract(ContractError::IllegalLifecycleTransition {
            from: LifecycleState::Destroyed,
            to: LifecycleState::Active,
            ..
        })
    ));
}

/// An authoring key names one entity in a world. A second claim on it is refused and names the
/// entity that already holds it, and the refusal leaves the registry untouched — nothing is
/// allocated for the rejected creation.
#[test]
fn an_authoring_key_belongs_to_exactly_one_entity() {
    let mut registry = EntityRegistry::new();
    let alice = registry
        .create(key("alice"), EntityType::Person)
        .expect("a fresh key is accepted");

    let refused = registry
        .create(key("alice"), EntityType::Place)
        .expect_err("a key cannot be claimed twice");

    assert!(matches!(
        refused,
        KernelError::EntityKeyAlreadyUsed { existing, .. } if existing == alice
    ));
    assert_eq!(registry.len(), 1);
    assert_eq!(
        registry
            .create(key("bob"), EntityType::Person)
            .expect("the refusal consumed no identity"),
        EntityId::from_raw(alice.raw() + 1)
    );
}

/// Resolution is how authored content reaches the runtime, so an unresolvable key is a named
/// error carrying the key, not an absent value the caller has to interpret.
#[test]
fn resolving_a_key_no_entity_holds_names_the_key() {
    let mut registry = EntityRegistry::new();
    let alice = registry
        .create(key("alice"), EntityType::Person)
        .expect("a fresh key is accepted");

    assert_eq!(
        registry.resolve(&key("alice")).expect("alice was created"),
        alice
    );

    let refused = registry
        .resolve(&key("nobody"))
        .expect_err("an unknown key does not resolve");
    assert!(matches!(
        refused,
        KernelError::UnknownEntityKey { ref key } if key.as_str() == "nobody"
    ));
}

/// Iteration follows identity, not the order entities happened to be created in. The test
/// creates entities whose keys sort in the opposite direction to their identities, so a registry
/// that iterated by key would pass the ordering check by accident.
#[test]
fn iteration_follows_identity_rather_than_insertion_or_key_order() {
    let mut registry = EntityRegistry::new();
    for name in ["zoe", "yara", "xena"] {
        registry
            .create(key(name), EntityType::Person)
            .expect("a fresh key is accepted");
    }

    let order: Vec<&str> = registry
        .iter()
        .map(|entity| entity.key().as_str())
        .collect();
    assert_eq!(order, vec!["zoe", "yara", "xena"]);

    let identities: Vec<u64> = registry.iter().map(|entity| entity.id().raw()).collect();
    assert_eq!(identities, vec![1, 2, 3]);
}

/// A registry round-trips through serde with every entity, the key index, and — the part that
/// matters — the allocation counter, so that a reloaded world continues the sequence instead of
/// restarting it.
#[test]
fn a_registry_read_back_continues_the_same_identity_sequence() {
    let mut registry = EntityRegistry::new();
    registry
        .create_authored(
            key("alice"),
            EntityType::Person,
            Tags::new([Tag::new("resident").expect("a legal tag")]),
            Some(authored_metadata()),
        )
        .expect("a fresh key is accepted");
    let last = registry
        .create(key("cafe"), EntityType::Place)
        .expect("a fresh key is accepted");

    let written = serde_json::to_string(&registry).expect("a registry serializes");
    let mut restored: EntityRegistry =
        serde_json::from_str(&written).expect("a registry deserializes");

    assert_eq!(restored, registry);
    assert_eq!(
        restored
            .resolve(&key("alice"))
            .expect("the index is rebuilt"),
        EntityId::from_raw(1)
    );
    assert_eq!(
        restored
            .create(key("bob"), EntityType::Person)
            .expect("a fresh key is accepted"),
        EntityId::from_raw(last.raw() + 1)
    );
}

/// The load-time refusals. Each of these snapshots is internally inconsistent in a way that
/// would break an invariant later and quietly: an identity that would be handed out twice, a
/// record filed under someone else's identity, one authoring key on two entities, and a counter
/// below the first identity a world allocates.
#[test]
fn a_persisted_registry_that_lost_an_invariant_is_refused_at_load() {
    let would_reuse = snapshot(
        r#"{
        "next_id": 2,
        "entities": {
            "1": {"id": 1, "key": "alice", "entity_type": "person", "tags": [], "lifecycle": "active"},
            "2": {"id": 2, "key": "cafe", "entity_type": "place", "tags": [], "lifecycle": "active"}
        }
    }"#,
    );
    assert!(matches!(
        refusal(would_reuse),
        KernelError::PersistedIdWouldBeReused { next: 2, allocated }
            if allocated == EntityId::from_raw(2)
    ));

    let misfiled = snapshot(
        r#"{
        "next_id": 9,
        "entities": {
            "1": {"id": 7, "key": "alice", "entity_type": "person", "tags": [], "lifecycle": "active"}
        }
    }"#,
    );
    assert!(matches!(
        refusal(misfiled),
        KernelError::PersistedEntityIdMismatch { at, found }
            if at == EntityId::from_raw(1) && found == EntityId::from_raw(7)
    ));

    let repeated_key = snapshot(
        r#"{
        "next_id": 9,
        "entities": {
            "1": {"id": 1, "key": "alice", "entity_type": "person", "tags": [], "lifecycle": "active"},
            "2": {"id": 2, "key": "alice", "entity_type": "person", "tags": [], "lifecycle": "active"}
        }
    }"#,
    );
    assert!(matches!(
        refusal(repeated_key),
        KernelError::PersistedEntityKeyRepeated { ref key, .. } if key.as_str() == "alice"
    ));

    assert!(matches!(
        refusal(snapshot(r#"{"next_id": 0, "entities": {}}"#)),
        KernelError::PersistedIdCounterTooLow { next: 0, first: 1 }
    ));
}

/// Deserializing a registry runs the same refusals, so an inconsistent world fails where it is
/// read rather than somewhere later. `serde` reports a conversion failure as a message, which is
/// why the variants themselves are checked through [`EntityRegistrySnapshot`] above.
#[test]
fn deserializing_an_inconsistent_registry_fails_rather_than_repairing_it() {
    let would_reuse = r#"{
        "next_id": 1,
        "entities": {
            "1": {"id": 1, "key": "alice", "entity_type": "person", "tags": [], "lifecycle": "active"}
        }
    }"#;

    let message = serde_json::from_str::<EntityRegistry>(would_reuse)
        .expect_err("an inconsistent registry must not load")
        .to_string();
    assert!(
        message.contains("already holds"),
        "the refusal should explain the reuse, but said: {message}"
    );
}

/// A registry converts to the shape persistence writes, and back, unchanged.
#[test]
fn a_registry_and_its_persisted_shape_are_the_same_world() {
    let mut registry = EntityRegistry::new();
    create_three(&mut registry);

    let written = EntityRegistrySnapshot::from(&registry);
    let restored = EntityRegistry::try_from(written.clone()).expect("a consistent snapshot loads");

    assert_eq!(restored, registry);
    assert_eq!(EntityRegistrySnapshot::from(&restored), written);
}

fn snapshot(written: &str) -> EntityRegistrySnapshot {
    serde_json::from_str(written).expect("the test snapshots are well formed JSON")
}

fn refusal(snapshot: EntityRegistrySnapshot) -> KernelError {
    EntityRegistry::try_from(snapshot).expect_err("an inconsistent registry must not load")
}
