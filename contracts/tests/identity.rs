//! Contract tests for the identity vocabulary.
//!
//! These run against the crate's public API only, which is the surface every other layer will
//! use. Every expected value here is written by hand: none is produced by calling the code
//! under test, so a change in behavior cannot silently change what the test expects.

use mineworld_contracts::{
    ActionId, ActionTypeId, ComponentTypeId, ContractError, EntityId, EntityKey, EntityType,
    EventId, IdentifierKind, ItemId, OrganizationId, PersonId, PlaceId, ProcessId, RejectionCode,
    RelationTypeId, SystemId,
};

/// The identifiers the specification's own examples use must be accepted, and the character
/// rule's boundaries must be where the documentation says they are.
#[test]
fn entity_key_accepts_authored_names_and_the_length_boundary() {
    let legal = [
        "alice".to_owned(),
        "bob".to_owned(),
        "modern-life".to_owned(),
        "pacific_northwest".to_owned(),
        "cafe2".to_owned(),
        "a".to_owned(),
        "a".repeat(64),
    ];

    for value in legal {
        let key = EntityKey::new(value.clone())
            .unwrap_or_else(|error| panic!("{value:?} should be a legal entity key: {error}"));
        assert_eq!(key.as_str(), value);
    }
}

/// Each malformed form is rejected with the variant that names what is wrong, and never
/// normalized into something acceptable.
#[test]
fn entity_key_rejects_malformed_names_with_a_named_error() {
    let kind = IdentifierKind::EntityKey;

    assert_eq!(
        EntityKey::new(""),
        Err(ContractError::IdentifierEmpty { kind })
    );
    assert_eq!(
        EntityKey::new("a".repeat(65)),
        Err(ContractError::IdentifierTooLong {
            kind,
            length: 65,
            max: 64,
        })
    );
    assert_eq!(
        EntityKey::new("Alice"),
        Err(ContractError::IdentifierIllegalCharacter {
            kind,
            character: 'A',
            position: 0,
        })
    );
    assert_eq!(
        EntityKey::new("alice smith"),
        Err(ContractError::IdentifierIllegalCharacter {
            kind,
            character: ' ',
            position: 5,
        })
    );
    assert_eq!(
        EntityKey::new("people/alice"),
        Err(ContractError::IdentifierIllegalCharacter {
            kind,
            character: '/',
            position: 6,
        })
    );
    assert_eq!(
        EntityKey::new("álice"),
        Err(ContractError::IdentifierIllegalCharacter {
            kind,
            character: 'á',
            position: 0,
        })
    );
    assert_eq!(
        EntityKey::new("-alice"),
        Err(ContractError::IdentifierSeparatorAtEdge {
            kind,
            character: '-',
        })
    );
    assert_eq!(
        EntityKey::new("alice_"),
        Err(ContractError::IdentifierSeparatorAtEdge {
            kind,
            character: '_',
        })
    );
}

/// The declaration names share one character rule, and each must report *its own*
/// [`IdentifierKind`] — the failure this test exists for is a copy-paste between the
/// hand-written identifier types, which would otherwise surface as a misleading error in a
/// World Pack loader.
#[test]
fn declaration_names_share_the_rule_and_report_their_own_kind() {
    assert!(SystemId::new("inventory").is_ok());
    assert!(SystemId::new("group-activity").is_ok());
    assert!(ComponentTypeId::new("inventory_component").is_ok());
    assert!(RelationTypeId::new("connected_to").is_ok());
    assert!(ActionTypeId::new("give_item").is_ok());
    assert!(RejectionCode::new("closed-for-the-night").is_ok());

    assert_eq!(
        SystemId::new("Economy"),
        Err(ContractError::IdentifierIllegalCharacter {
            kind: IdentifierKind::SystemId,
            character: 'E',
            position: 0,
        })
    );
    assert_eq!(
        ComponentTypeId::new(""),
        Err(ContractError::IdentifierEmpty {
            kind: IdentifierKind::ComponentTypeId,
        })
    );
    assert_eq!(
        ComponentTypeId::new("a".repeat(65)),
        Err(ContractError::IdentifierTooLong {
            kind: IdentifierKind::ComponentTypeId,
            length: 65,
            max: 64,
        })
    );
    assert_eq!(
        RelationTypeId::new("connected_to_"),
        Err(ContractError::IdentifierSeparatorAtEdge {
            kind: IdentifierKind::RelationTypeId,
            character: '_',
        })
    );
    assert_eq!(
        RelationTypeId::new("employee of"),
        Err(ContractError::IdentifierIllegalCharacter {
            kind: IdentifierKind::RelationTypeId,
            character: ' ',
            position: 8,
        })
    );
    assert_eq!(
        ActionTypeId::new("Talk"),
        Err(ContractError::IdentifierIllegalCharacter {
            kind: IdentifierKind::ActionTypeId,
            character: 'T',
            position: 0,
        })
    );
    assert_eq!(
        RejectionCode::new("_closed"),
        Err(ContractError::IdentifierSeparatorAtEdge {
            kind: IdentifierKind::RejectionCode,
            character: '_',
        })
    );
}

/// A typed reference may be built only for the entity type it names, and a mismatch reports
/// the entity together with both types. The exhaustive table is deliberate: it is what catches
/// a wrong constant in any one of the four hand-written reference types.
#[test]
fn a_typed_reference_is_built_only_for_its_own_entity_type() {
    let entity = EntityId::from_raw(11);

    type CheckedReference = fn(EntityId, EntityType) -> Result<EntityId, ContractError>;
    let references: [(EntityType, CheckedReference); 4] = [
        (EntityType::Person, |entity, actual| {
            PersonId::new(entity, actual).map(PersonId::entity_id)
        }),
        (EntityType::Place, |entity, actual| {
            PlaceId::new(entity, actual).map(PlaceId::entity_id)
        }),
        (EntityType::Item, |entity, actual| {
            ItemId::new(entity, actual).map(ItemId::entity_id)
        }),
        (EntityType::Organization, |entity, actual| {
            OrganizationId::new(entity, actual).map(OrganizationId::entity_id)
        }),
    ];
    let all_types = [
        EntityType::Person,
        EntityType::Place,
        EntityType::Item,
        EntityType::Organization,
    ];

    for (expected, build) in references {
        assert_eq!(
            build(entity, expected),
            Ok(entity),
            "a reference to {expected} should accept an entity of that type"
        );

        for actual in all_types {
            if actual == expected {
                continue;
            }
            assert_eq!(
                build(entity, actual),
                Err(ContractError::EntityTypeMismatch {
                    entity,
                    expected,
                    actual,
                })
            );
        }
    }
}

/// The opaque identities are written as bare integers. The wire shape is a contract: the event
/// log and the persistence layer read it back, so wrapping them in an object later would be a
/// migration, not a refactor.
#[test]
fn opaque_identities_are_written_as_bare_integers() {
    assert_eq!(serde_json::to_string(&EntityId::from_raw(7)).unwrap(), "7");
    assert_eq!(serde_json::to_string(&EventId::from_raw(8)).unwrap(), "8");
    assert_eq!(serde_json::to_string(&ActionId::from_raw(9)).unwrap(), "9");
    assert_eq!(
        serde_json::to_string(&ProcessId::from_raw(10)).unwrap(),
        "10"
    );

    assert_eq!(
        serde_json::from_str::<EntityId>("7").unwrap(),
        EntityId::from_raw(7)
    );
    assert_eq!(
        serde_json::from_str::<EventId>("8").unwrap(),
        EventId::from_raw(8)
    );
    assert_eq!(
        serde_json::from_str::<ActionId>("9").unwrap(),
        ActionId::from_raw(9)
    );
    assert_eq!(
        serde_json::from_str::<ProcessId>("10").unwrap(),
        ProcessId::from_raw(10)
    );
}

/// Authored names are written as plain strings, and reading one back runs the same validation
/// as constructing it, so a malformed name in a saved world or an authored file fails where it
/// is read.
#[test]
fn authored_names_round_trip_as_validated_strings() {
    let key = EntityKey::new("alice").unwrap();
    assert_eq!(serde_json::to_string(&key).unwrap(), "\"alice\"");
    assert_eq!(serde_json::from_str::<EntityKey>("\"alice\"").unwrap(), key);

    let system = SystemId::new("inventory").unwrap();
    assert_eq!(serde_json::to_string(&system).unwrap(), "\"inventory\"");
    assert_eq!(
        serde_json::from_str::<SystemId>("\"inventory\"").unwrap(),
        system
    );

    let component = ComponentTypeId::new("inventory_component").unwrap();
    assert_eq!(
        serde_json::from_str::<ComponentTypeId>("\"inventory_component\"").unwrap(),
        component
    );

    let relation = RelationTypeId::new("connected_to").unwrap();
    assert_eq!(
        serde_json::from_str::<RelationTypeId>("\"connected_to\"").unwrap(),
        relation
    );

    let rejected = serde_json::from_str::<EntityKey>("\"Alice\"").unwrap_err();
    assert!(
        rejected.to_string().contains("lowercase ASCII"),
        "deserialization should apply the identifier rule, but reported: {rejected}"
    );
}

/// The entity taxonomy is closed, and an unrecognized value is an error rather than a default.
/// Without this, a typo in an authored file or an entity kind from a future version would
/// silently become whichever variant happened to be first.
#[test]
fn entity_type_has_stable_names_and_no_silent_fallback() {
    assert_eq!(
        serde_json::to_string(&EntityType::Person).unwrap(),
        "\"person\""
    );
    assert_eq!(
        serde_json::to_string(&EntityType::Organization).unwrap(),
        "\"organization\""
    );
    assert_eq!(
        serde_json::from_str::<EntityType>("\"place\"").unwrap(),
        EntityType::Place
    );
    assert_eq!(
        serde_json::from_str::<EntityType>("\"item\"").unwrap(),
        EntityType::Item
    );

    assert!(serde_json::from_str::<EntityType>("\"vehicle\"").is_err());
    assert!(serde_json::from_str::<EntityType>("\"Person\"").is_err());
}

/// A typed reference carries the type it was checked against through serialization, so that
/// reading one back is the same checked construction as building one. Without the tag, any
/// integer in a saved world could be read back as a person.
#[test]
fn a_typed_reference_is_still_checked_when_it_is_read_back() {
    let person = PersonId::new(EntityId::from_raw(3), EntityType::Person).unwrap();
    let encoded = r#"{"entity":3,"entity_type":"person"}"#;

    assert_eq!(serde_json::to_string(&person).unwrap(), encoded);
    assert_eq!(serde_json::from_str::<PersonId>(encoded).unwrap(), person);

    let relabelled = serde_json::from_str::<PersonId>(r#"{"entity":3,"entity_type":"place"}"#)
        .expect_err("a place must not deserialize into a person reference");
    assert!(
        relabelled
            .to_string()
            .contains("cannot be referenced as person"),
        "the mismatch should be reported as an entity-type mismatch, but was: {relabelled}"
    );
}

/// Serializing the same value twice produces the same bytes. For these scalar identities that
/// is nearly free; the assertion exists because the contract layer's determinism requirement
/// applies to every type in it, and this is where the expectation is recorded from the start.
#[test]
fn repeated_serialization_is_byte_identical() {
    let person = PersonId::new(EntityId::from_raw(4), EntityType::Person).unwrap();
    let key = EntityKey::new("alice").unwrap();

    assert_eq!(
        serde_json::to_vec(&person).unwrap(),
        serde_json::to_vec(&person).unwrap()
    );
    assert_eq!(
        serde_json::to_vec(&key).unwrap(),
        serde_json::to_vec(&key).unwrap()
    );
}
