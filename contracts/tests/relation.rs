//! Contract tests for the relation model.
//!
//! The relation types here are named as the specification's own examples name them —
//! `employee_of`, `connected_to` — because those are the shapes the machinery has to support.
//! The machinery is all that is tested: no relationship value, strength or meaning exists in
//! this crate to test.

use mineworld_contracts::{
    ContractError, Entity, EntityId, EntityKey, EntityType, EntityTypeSet, Relation,
    RelationDirection, RelationEnd, RelationTypeDeclaration, RelationTypeId, SystemId,
};

fn entity(raw: u64, key: &str, entity_type: EntityType) -> Entity {
    Entity::new(
        EntityId::from_raw(raw),
        EntityKey::new(key).unwrap(),
        entity_type,
    )
}

fn types(entity_types: impl IntoIterator<Item = EntityType>) -> EntityTypeSet {
    EntityTypeSet::new(entity_types).unwrap()
}

fn relation_type(name: &str) -> RelationTypeId {
    RelationTypeId::new(name).unwrap()
}

fn owner(name: &str) -> SystemId {
    SystemId::new(name).unwrap()
}

fn employee_of() -> RelationTypeDeclaration {
    RelationTypeDeclaration::directed(
        relation_type("employee_of"),
        owner("first-stub"),
        types([EntityType::Person]),
        types([EntityType::Organization]),
    )
}

fn connected_to() -> RelationTypeDeclaration {
    RelationTypeDeclaration::undirected(
        relation_type("connected_to"),
        owner("second-stub"),
        types([EntityType::Place]),
    )
}

/// Every pair `docs/CORE_CONCEPTS.md` §9 lists must be expressible with this one machinery, and
/// nothing about the machinery may privilege `Person ↔ Person`.
#[test]
fn every_entity_type_pair_the_specification_lists_is_representable() {
    let alice = entity(1, "alice", EntityType::Person);
    let bob = entity(2, "bob", EntityType::Person);
    let cafe = entity(3, "cafe", EntityType::Place);
    let park = entity(4, "park", EntityType::Place);
    let guild = entity(5, "guild", EntityType::Organization);
    let cup = entity(6, "cup", EntityType::Item);

    let pairs = [
        (
            "friend_of",
            EntityType::Person,
            EntityType::Person,
            &alice,
            &bob,
        ),
        (
            "employee_of",
            EntityType::Person,
            EntityType::Organization,
            &alice,
            &guild,
        ),
        (
            "operates_at",
            EntityType::Organization,
            EntityType::Place,
            &guild,
            &cafe,
        ),
        ("owns", EntityType::Person, EntityType::Item, &alice, &cup),
        (
            "connected_to",
            EntityType::Place,
            EntityType::Place,
            &cafe,
            &park,
        ),
    ];

    for (name, from_type, to_type, from, to) in pairs {
        let declaration = RelationTypeDeclaration::directed(
            relation_type(name),
            owner("first-stub"),
            types([from_type]),
            types([to_type]),
        );
        let relation = Relation::between(&declaration, from, to)
            .unwrap_or_else(|error| panic!("{name} should be representable: {error}"));
        assert_eq!(relation.relation_type(), &relation_type(name));
        assert_eq!(relation.from(), from.id());
        assert_eq!(relation.to(), to.id());
    }
}

/// An endpoint of a type the declaration does not permit is refused, and the error says which end
/// was wrong, which entity was offered, what type it has and what the declaration permits — all
/// four, because a world author fixing an authored file needs all four.
#[test]
fn an_endpoint_of_the_wrong_type_is_refused_by_the_end_that_refused_it() {
    let alice = entity(1, "alice", EntityType::Person);
    let cafe = entity(3, "cafe", EntityType::Place);
    let guild = entity(5, "guild", EntityType::Organization);

    assert_eq!(
        Relation::between(&employee_of(), &alice, &cafe).unwrap_err(),
        ContractError::RelationEndpointNotPermitted {
            relation_type: relation_type("employee_of"),
            end: RelationEnd::To,
            entity: EntityId::from_raw(3),
            actual: EntityType::Place,
            permitted: types([EntityType::Organization]),
        }
    );

    assert_eq!(
        Relation::between(&employee_of(), &cafe, &guild).unwrap_err(),
        ContractError::RelationEndpointNotPermitted {
            relation_type: relation_type("employee_of"),
            end: RelationEnd::From,
            entity: EntityId::from_raw(3),
            actual: EntityType::Place,
            permitted: types([EntityType::Person]),
        }
    );

    // The right pair in the wrong order is still the wrong pair: a directed type is not
    // symmetric.
    assert!(Relation::between(&employee_of(), &guild, &alice).is_err());
    assert!(Relation::between(&employee_of(), &alice, &guild).is_ok());
}

/// An undirected edge is one edge however it is written: both construction orders produce the
/// same value and the same bytes. Without this, `(cafe, park)` and `(park, cafe)` would be two
/// rows for one fact and nothing would keep their attached state consistent.
#[test]
fn an_undirected_edge_is_canonical_in_both_construction_orders() {
    let cafe = entity(3, "cafe", EntityType::Place);
    let park = entity(4, "park", EntityType::Place);

    let one_way = Relation::between(&connected_to(), &cafe, &park).unwrap();
    let other_way = Relation::between(&connected_to(), &park, &cafe).unwrap();

    assert_eq!(one_way, other_way);
    assert_eq!(one_way.from(), EntityId::from_raw(3));
    assert_eq!(one_way.to(), EntityId::from_raw(4));
    assert_eq!(
        serde_json::to_vec(&one_way).unwrap(),
        serde_json::to_vec(&other_way).unwrap()
    );
    assert_eq!(
        serde_json::to_string(&one_way).unwrap(),
        r#"{"relation_type":"connected_to","from":3,"to":4}"#
    );
}

/// A directed edge keeps its direction: `(a, b)` and `(b, a)` are two different edges, which is
/// the whole point of declaring a type directed.
#[test]
fn a_directed_edge_distinguishes_its_two_orders() {
    let alice = entity(1, "alice", EntityType::Person);
    let bob = entity(2, "bob", EntityType::Person);
    let declaration = RelationTypeDeclaration::directed(
        relation_type("parent_of"),
        owner("first-stub"),
        types([EntityType::Person]),
        types([EntityType::Person]),
    );

    let one_way = Relation::between(&declaration, &alice, &bob).unwrap();
    let other_way = Relation::between(&declaration, &bob, &alice).unwrap();

    assert_ne!(one_way, other_way);
    assert_eq!(one_way.from(), EntityId::from_raw(1));
    assert_eq!(other_way.from(), EntityId::from_raw(2));
}

/// Whether an entity may relate to itself is the declaration's decision, and it is never left to
/// the call site.
#[test]
fn a_self_edge_follows_the_declaration_and_nothing_else() {
    let cafe = entity(3, "cafe", EntityType::Place);

    let forbidding = connected_to();
    assert_eq!(
        Relation::between(&forbidding, &cafe, &cafe).unwrap_err(),
        ContractError::SelfEdgeNotPermitted {
            relation_type: relation_type("connected_to"),
            entity: EntityId::from_raw(3),
        }
    );

    let permitting = connected_to().permitting_self_edges();
    let loop_edge = Relation::between(&permitting, &cafe, &cafe).unwrap();
    assert_eq!(loop_edge.from(), loop_edge.to());
}

/// A declaration is a value a registry stores, so its stored shape is a contract; and a relation
/// type that permits nothing at an end is refused where it is written rather than becoming a
/// declaration no world can ever satisfy.
#[test]
fn a_declaration_is_stored_as_its_documented_shape_and_cannot_permit_nothing() {
    let declaration = connected_to();
    assert_eq!(declaration.direction(), RelationDirection::Undirected);
    assert_eq!(declaration.owner(), &owner("second-stub"));
    assert_eq!(
        serde_json::to_string(&declaration).unwrap(),
        concat!(
            r#"{"relation_type":"connected_to","owner":"second-stub","direction":"undirected","#,
            r#""from_types":["place"],"to_types":["place"],"self_edges":"forbidden"}"#
        )
    );
    assert_eq!(
        serde_json::from_str::<RelationTypeDeclaration>(
            &serde_json::to_string(&declaration).unwrap()
        )
        .unwrap(),
        declaration
    );

    assert_eq!(
        EntityTypeSet::new([]).unwrap_err(),
        ContractError::EmptyEntityTypeSet
    );
    assert!(
        serde_json::from_str::<EntityTypeSet>("[]").is_err(),
        "an empty endpoint set must not survive a round trip through storage either"
    );
}
