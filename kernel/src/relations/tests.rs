//! Behaviour tests for the relation graph.
//!
//! Two properties carry the weight here. An edge is a fact about *both* of its endpoints, so it is
//! found from either end whichever way it was written; and an edge type belongs to the system that
//! declared it, so nobody else writes its edges. The rest is what a graph has to get right to be
//! replayable: one edge per triple, a canonical form for an undirected edge, and a fixed order.
//!
//! These tests moved inside the crate when PR 03b sealed `WriteAccess::new()` (`BD-1`). A write
//! token now exists only inside a world, so no external test crate can obtain one — which is the
//! guarantee, not an obstacle. What a pack author sees from outside is tested from outside, in
//! `kernel/tests/`; what only the kernel can reach is tested here.

use crate::{
    KernelError, RelationStore, RelationStoreSnapshot, SystemIdentity, WriteAccess, WriteToken,
};
use mineworld_contracts::{
    ContractError, Entity, EntityId, EntityKey, EntityType, EntityTypeSet, LifecycleState,
    RelationDirection, RelationEnd, RelationTypeDeclaration, RelationTypeId, SelfEdges, SystemId,
};

/// The first stub system: it declares the edge types in these tests.
struct FirstStub;
impl SystemIdentity for FirstStub {
    const ID: SystemId = SystemId::from_static("first-stub");
}

/// The second stub system, which owns nothing here and must therefore be refused everywhere.
struct SecondStub;
impl SystemIdentity for SecondStub {
    const ID: SystemId = SystemId::from_static("second-stub");
}

fn tokens() -> (WriteAccess, WriteToken<FirstStub>, WriteToken<SecondStub>) {
    let mut access = WriteAccess::new();
    let first = access.grant(&FirstStub).expect("a first grant succeeds");
    let second = access.grant(&SecondStub).expect("a first grant succeeds");
    (access, first, second)
}

fn entity(raw: u64, key: &str, entity_type: EntityType) -> Entity {
    Entity::new(
        EntityId::from_raw(raw),
        EntityKey::new(key).expect("a legal authoring name"),
        entity_type,
    )
}

fn relation_type(name: &str) -> RelationTypeId {
    RelationTypeId::new(name).expect("a legal declared name")
}

fn types(entity_types: [EntityType; 1]) -> EntityTypeSet {
    EntityTypeSet::new(entity_types).expect("a non-empty set")
}

/// A directed edge from a person to an organization: the two ends mean different things.
fn directed() -> RelationTypeDeclaration {
    RelationTypeDeclaration::directed(
        relation_type("attached-to"),
        SystemId::from_static("first-stub"),
        types([EntityType::Person]),
        types([EntityType::Organization]),
    )
}

/// An undirected edge between two places: the same fact read either way.
fn undirected() -> RelationTypeDeclaration {
    RelationTypeDeclaration::undirected(
        relation_type("linked-to"),
        SystemId::from_static("first-stub"),
        types([EntityType::Place]),
    )
}

/// A store in which the first stub system has declared both edge types.
fn declared_store() -> (RelationStore, WriteToken<FirstStub>, WriteToken<SecondStub>) {
    let (_access, first, second) = tokens();
    let mut store = RelationStore::new();
    store
        .declare(&first, directed())
        .expect("the owner declares its own edge type");
    store
        .declare(&first, undirected())
        .expect("the owner declares its own edge type");
    (store, first, second)
}

/// An edge is a fact about both endpoints, so it is found from either one — including a directed
/// edge found from the end it points at, which is the case an adjacency list kept on one side would
/// get wrong.
#[test]
fn an_edge_is_found_from_either_endpoint() {
    let (mut store, first, _second) = declared_store();
    let alice = entity(1, "alice", EntityType::Person);
    let guild = entity(2, "guild", EntityType::Organization);
    let bystander = entity(3, "bystander", EntityType::Person);

    let edge = store
        .insert(&first, &relation_type("attached-to"), &alice, &guild)
        .expect("the owner may write its own edge type");

    assert_eq!(
        store.touching(alice.id()).collect::<Vec<_>>(),
        vec![&edge],
        "found from the end it leads from"
    );
    assert_eq!(
        store.touching(guild.id()).collect::<Vec<_>>(),
        vec![&edge],
        "and from the end it leads to"
    );
    assert_eq!(store.touching(bystander.id()).count(), 0);
    assert!(store.contains(&edge));
    assert_eq!(edge.from(), alice.id());
    assert_eq!(edge.to(), guild.id());
}

/// An undirected edge is one edge however it is written. The store forms it against the
/// declaration, so the canonical order is not something a caller can get wrong or work around.
#[test]
fn an_undirected_edge_written_both_ways_is_one_edge() {
    let (mut store, first, _second) = declared_store();
    let north = entity(7, "north", EntityType::Place);
    let south = entity(3, "south", EntityType::Place);

    let forwards = store
        .insert(&first, &relation_type("linked-to"), &north, &south)
        .expect("the owner may write");
    let backwards = store
        .insert(&first, &relation_type("linked-to"), &south, &north)
        .expect("writing the same edge again is not an error");

    assert_eq!(forwards, backwards);
    assert_eq!(store.len(), 1);
    assert_eq!(
        forwards.from(),
        south.id(),
        "the lower identity is stored as from"
    );
    assert_eq!(forwards.to(), north.id());
    assert_eq!(store.touching(north.id()).count(), 1);
    assert_eq!(store.touching(south.id()).count(), 1);
}

/// A directed edge is not the same fact read backwards, so the two orderings are two edges.
#[test]
fn a_directed_edge_is_not_the_same_edge_reversed() {
    let (_access, first, _second) = tokens();
    let mut store = RelationStore::new();
    store
        .declare(
            &first,
            RelationTypeDeclaration::directed(
                relation_type("attached-to"),
                SystemId::from_static("first-stub"),
                types([EntityType::Person]),
                types([EntityType::Person]),
            ),
        )
        .expect("the owner declares its own edge type");

    let alice = entity(1, "alice", EntityType::Person);
    let bob = entity(2, "bob", EntityType::Person);
    store
        .insert(&first, &relation_type("attached-to"), &alice, &bob)
        .expect("the owner may write");
    store
        .insert(&first, &relation_type("attached-to"), &bob, &alice)
        .expect("the owner may write");

    assert_eq!(store.len(), 2);
    assert_eq!(store.touching(alice.id()).count(), 2);
}

/// The edge type's declaration is the authority, and the store applies it rather than trusting the
/// caller: an endpoint of the wrong type and a self-edge where the declaration forbids one are both
/// refused, with the contract layer's own errors, and nothing is stored.
#[test]
fn the_declaration_decides_which_edges_can_exist() {
    let (mut store, first, _second) = declared_store();
    let alice = entity(1, "alice", EntityType::Person);
    let guild = entity(2, "guild", EntityType::Organization);
    let north = entity(3, "north", EntityType::Place);

    let wrong_end = store
        .insert(&first, &relation_type("attached-to"), &alice, &north)
        .expect_err("a place is not permitted at that end");
    assert!(matches!(
        wrong_end,
        KernelError::Contract(ContractError::RelationEndpointNotPermitted {
            end: RelationEnd::To,
            actual: EntityType::Place,
            ..
        })
    ));

    let backwards = store
        .insert(&first, &relation_type("attached-to"), &guild, &alice)
        .expect_err("the ends of a directed type are not interchangeable");
    assert!(matches!(
        backwards,
        KernelError::Contract(ContractError::RelationEndpointNotPermitted { .. })
    ));

    let self_edge = store
        .insert(&first, &relation_type("linked-to"), &north, &north)
        .expect_err("this declaration forbids a self-edge");
    assert!(matches!(
        self_edge,
        KernelError::Contract(ContractError::SelfEdgeNotPermitted { .. })
    ));

    assert!(store.is_empty());
}

/// A declaration may permit a self-edge, and then it is one edge rather than a refusal — the rule
/// lives in the declaration, never in the store.
#[test]
fn a_declaration_may_permit_a_self_edge() {
    let (_access, first, _second) = tokens();
    let mut store = RelationStore::new();
    store
        .declare(&first, undirected().permitting_self_edges())
        .expect("the owner declares its own edge type");

    let north = entity(1, "north", EntityType::Place);
    let edge = store
        .insert(&first, &relation_type("linked-to"), &north, &north)
        .expect("this declaration permits it");

    assert_eq!(edge.from(), edge.to());
    assert_eq!(store.len(), 1);
}

/// `INV-7` for edges: the declaring system is the single writer. Every write path refuses the other
/// system by name and changes nothing — and the refusal is about the *declared owner*, not about
/// who happens to hold a token.
#[test]
fn only_the_declaring_system_writes_a_relation_type() {
    let (mut store, first, second) = declared_store();
    let alice = entity(1, "alice", EntityType::Person);
    let guild = entity(2, "guild", EntityType::Organization);
    let edge = store
        .insert(&first, &relation_type("attached-to"), &alice, &guild)
        .expect("the owner may write");

    let refused_write = store
        .insert(&second, &relation_type("attached-to"), &alice, &guild)
        .expect_err("another system cannot write this edge type");
    assert!(matches!(
        refused_write,
        KernelError::RelationTypeNotOwned {
            ref owner,
            ref writing_system,
            ..
        } if owner.as_str() == "first-stub" && writing_system.as_str() == "second-stub"
    ));

    let refused_removal = store
        .remove(&second, &edge)
        .expect_err("another system cannot remove this edge type's edges");
    assert!(matches!(
        refused_removal,
        KernelError::RelationTypeNotOwned { .. }
    ));

    let refused_declaration = store
        .declare(&second, directed())
        .expect_err("a system cannot install another system's edge type");
    assert!(matches!(
        refused_declaration,
        KernelError::RelationTypeNotOwned { .. }
    ));

    assert!(store.contains(&edge), "every refusal left the graph alone");
    assert!(
        store
            .remove(&first, &edge)
            .expect("the owner may remove its own edge")
    );
    assert!(!store.contains(&edge));
}

/// An edge type nobody declared has no edges, and writing one is refused by name rather than
/// declaring the type as a side effect.
#[test]
fn an_undeclared_relation_type_cannot_be_written() {
    let (_access, first, _second) = tokens();
    let mut store = RelationStore::new();
    let alice = entity(1, "alice", EntityType::Person);
    let guild = entity(2, "guild", EntityType::Organization);

    let refused = store
        .insert(&first, &relation_type("attached-to"), &alice, &guild)
        .expect_err("an undeclared edge type cannot be written");

    assert!(matches!(
        refused,
        KernelError::RelationTypeNotDeclared { ref relation_type }
            if relation_type.as_str() == "attached-to"
    ));
    assert!(store.is_empty());
    assert_eq!(store.declarations().count(), 0);
}

/// An edge type has exactly one declaration, for the same reason a component type does: edges
/// already stored were checked against the first one.
#[test]
fn a_relation_type_has_exactly_one_declaration() {
    let (_access, first, second) = tokens();
    let mut store = RelationStore::new();
    store
        .declare(&first, directed())
        .expect("the owner declares its edge type");

    store
        .declare(&first, directed())
        .expect("the identical declaration again is not a conflict");
    assert_eq!(store.declarations().count(), 1);

    let claimed = store
        .declare(
            &second,
            RelationTypeDeclaration::directed(
                relation_type("attached-to"),
                SystemId::from_static("second-stub"),
                types([EntityType::Person]),
                types([EntityType::Organization]),
            ),
        )
        .expect_err("two systems cannot own one edge type");
    assert!(matches!(
        claimed,
        KernelError::RelationTypeClaimedByAnotherSystem {
            ref declared_by,
            ref claimed_by,
            ..
        } if declared_by.as_str() == "first-stub" && claimed_by.as_str() == "second-stub"
    ));

    let changed_rules = store
        .declare(&first, directed().permitting_self_edges())
        .expect_err("one name cannot carry two sets of rules");
    assert!(matches!(
        changed_rules,
        KernelError::RelationTypeAlreadyDeclared { ref relation_type }
            if relation_type.as_str() == "attached-to"
    ));

    assert_eq!(
        store
            .declaration(&relation_type("attached-to"))
            .expect("the first declaration still stands")
            .self_edges(),
        SelfEdges::Forbidden
    );
}

/// Destroying an entity clears every edge touching it, across every type and both directions, and
/// hands the removed edges back so that their declaring systems can be told. Edges between other
/// entities are untouched.
#[test]
fn destroying_an_entity_clears_every_edge_touching_it() {
    let (mut store, first, _second) = declared_store();
    let mut alice = entity(1, "alice", EntityType::Person);
    let guild = entity(2, "guild", EntityType::Organization);
    let north = entity(3, "north", EntityType::Place);
    let south = entity(4, "south", EntityType::Place);
    let bob = entity(5, "bob", EntityType::Person);

    store
        .insert(&first, &relation_type("attached-to"), &alice, &guild)
        .expect("the owner may write");
    store
        .insert(&first, &relation_type("attached-to"), &bob, &guild)
        .expect("the owner may write");
    store
        .insert(&first, &relation_type("linked-to"), &north, &south)
        .expect("the owner may write");

    let still_live = store
        .remove_edges_of_destroyed_entity(&alice)
        .expect_err("a live entity's edges are not the kernel's to remove");
    assert!(matches!(
        still_live,
        KernelError::EntityNotDestroyed {
            lifecycle: LifecycleState::Active,
            ..
        }
    ));
    assert_eq!(store.len(), 3);

    alice
        .transition_to(LifecycleState::Destroyed)
        .expect("an active entity may be destroyed");
    let removed = store
        .remove_edges_of_destroyed_entity(&alice)
        .expect("a destroyed entity's edges go");

    assert_eq!(removed.len(), 1);
    assert_eq!(removed[0].from(), alice.id());
    assert_eq!(store.touching(alice.id()).count(), 0);
    assert_eq!(store.len(), 2, "other entities' edges are untouched");
    assert_eq!(store.touching(guild.id()).count(), 1);
}

/// Iteration order is a property of the edges, not of the run: edges written in one order and the
/// reverse order produce the same sequence, grouped by edge type and then by endpoint identity.
#[test]
fn iteration_order_does_not_depend_on_write_order() {
    let north = entity(9, "north", EntityType::Place);
    let south = entity(2, "south", EntityType::Place);
    let east = entity(5, "east", EntityType::Place);

    let mut forwards = {
        let (_access, first, _second) = tokens();
        let mut store = RelationStore::new();
        store.declare(&first, undirected()).expect("declared");
        for (from, to) in [(&north, &south), (&south, &east), (&north, &east)] {
            store
                .insert(&first, &relation_type("linked-to"), from, to)
                .expect("the owner may write");
        }
        store
    };
    let backwards = {
        let (_access, first, _second) = tokens();
        let mut store = RelationStore::new();
        store.declare(&first, undirected()).expect("declared");
        for (from, to) in [(&east, &north), (&east, &south), (&south, &north)] {
            store
                .insert(&first, &relation_type("linked-to"), from, to)
                .expect("the owner may write");
        }
        store
    };

    let sequence: Vec<(u64, u64)> = forwards
        .iter()
        .map(|edge| (edge.from().raw(), edge.to().raw()))
        .collect();
    assert_eq!(sequence, vec![(2, 5), (2, 9), (5, 9)]);
    assert_eq!(
        sequence,
        backwards
            .iter()
            .map(|edge| (edge.from().raw(), edge.to().raw()))
            .collect::<Vec<_>>()
    );
    assert_eq!(forwards, backwards);

    // Removing and re-adding an edge cannot move it either.
    let (_access, first, _second) = tokens();
    let edge = forwards.iter().next().expect("the graph has edges").clone();
    assert!(
        forwards
            .remove(&first, &edge)
            .expect("the owner may remove")
    );
    forwards
        .insert(&first, &relation_type("linked-to"), &south, &east)
        .expect("the owner may write");
    assert_eq!(
        forwards
            .iter()
            .map(|edge| (edge.from().raw(), edge.to().raw()))
            .collect::<Vec<_>>(),
        sequence
    );
}

/// The graph round-trips through serde, declarations included, and comes back as the same graph.
#[test]
fn the_graph_round_trips_through_serde() {
    let (mut store, first, _second) = declared_store();
    let alice = entity(1, "alice", EntityType::Person);
    let guild = entity(2, "guild", EntityType::Organization);
    let north = entity(3, "north", EntityType::Place);
    let south = entity(4, "south", EntityType::Place);
    store
        .insert(&first, &relation_type("attached-to"), &alice, &guild)
        .expect("the owner may write");
    store
        .insert(&first, &relation_type("linked-to"), &south, &north)
        .expect("the owner may write");

    let written = serde_json::to_string(&store).expect("a graph serializes");
    let restored: RelationStore = serde_json::from_str(&written).expect("a graph deserializes");

    assert_eq!(restored, store);
    assert_eq!(restored.len(), 2);
    assert_eq!(
        restored
            .declaration(&relation_type("linked-to"))
            .expect("the declaration came back")
            .direction(),
        RelationDirection::Undirected
    );
    assert_eq!(
        RelationStoreSnapshot::from(&restored),
        RelationStoreSnapshot::from(&store)
    );
}

/// The load-time refusals. A `Relation` deserializes field by field, so a file can hold an edge no
/// declaration would have produced: an undirected edge with its ends the wrong way round would be
/// one fact stored as two edges, an edge of an undeclared type has no rules at all, and a
/// declaration filed under the wrong name makes lookups disagree with the values they return.
#[test]
fn a_persisted_graph_that_lost_an_invariant_is_refused_at_load() {
    let non_canonical = r#"{
        "declarations": {
            "linked-to": {
                "relation_type": "linked-to", "owner": "first-stub", "direction": "undirected",
                "from_types": ["place"], "to_types": ["place"], "self_edges": "forbidden"
            }
        },
        "edges": [{"relation_type": "linked-to", "from": 9, "to": 2}]
    }"#;
    assert!(matches!(
        refusal(non_canonical),
        KernelError::PersistedRelationNotCanonical { from, to, .. }
            if from == EntityId::from_raw(9) && to == EntityId::from_raw(2)
    ));

    let undeclared = r#"{
        "declarations": {},
        "edges": [{"relation_type": "linked-to", "from": 2, "to": 9}]
    }"#;
    assert!(matches!(
        refusal(undeclared),
        KernelError::RelationTypeNotDeclared { ref relation_type }
            if relation_type.as_str() == "linked-to"
    ));

    let misfiled = r#"{
        "declarations": {
            "misfiled": {
                "relation_type": "linked-to", "owner": "first-stub", "direction": "undirected",
                "from_types": ["place"], "to_types": ["place"], "self_edges": "forbidden"
            }
        },
        "edges": []
    }"#;
    assert!(matches!(
        refusal(misfiled),
        KernelError::PersistedRelationTypeMismatch { ref at, ref found }
            if at.as_str() == "misfiled" && found.as_str() == "linked-to"
    ));

    // And the same refusals apply through `serde` itself, so an inconsistent world fails where it
    // is read rather than somewhere later.
    assert!(serde_json::from_str::<RelationStore>(non_canonical).is_err());
}

fn refusal(written: &str) -> KernelError {
    let snapshot: RelationStoreSnapshot =
        serde_json::from_str(written).expect("the test snapshots are well formed JSON");
    RelationStore::try_from(snapshot).expect_err("an inconsistent graph must not load")
}
