//! Contract tests for the entity record.
//!
//! Every expected value is a hand-written literal, including the exact JSON an entity is
//! stored as: that shape is what the persistence layer and the event log will read back.

use mineworld_contracts::{
    ContractError, Entity, EntityId, EntityKey, EntityType, IdentifierKind, LifecycleState,
    Metadata, Tag, Tags,
};

fn alice() -> Entity {
    Entity::new(
        EntityId::from_raw(1),
        EntityKey::new("alice").unwrap(),
        EntityType::Person,
    )
}

fn tag(value: &str) -> Tag {
    Tag::new(value).unwrap()
}

/// The full transition table, asserted in both directions. `Destroyed` is terminal and no
/// state transitions to itself: a no-op would otherwise report success for a change that did
/// not happen.
#[test]
fn the_lifecycle_permits_exactly_the_documented_transitions() {
    use LifecycleState::{Active, Destroyed, Dormant};

    let legal = [
        (Active, Dormant),
        (Active, Destroyed),
        (Dormant, Active),
        (Dormant, Destroyed),
    ];
    let illegal = [
        (Active, Active),
        (Dormant, Dormant),
        (Destroyed, Destroyed),
        (Destroyed, Active),
        (Destroyed, Dormant),
    ];

    for (from, to) in legal {
        assert!(
            from.can_transition_to(to),
            "{from} -> {to} should be permitted"
        );
    }
    for (from, to) in illegal {
        assert!(
            !from.can_transition_to(to),
            "{from} -> {to} should be refused"
        );
    }
}

/// An entity's lifecycle moves only through the checked transition, and a refused transition
/// leaves the entity untouched rather than half-changed.
#[test]
fn an_entity_moves_through_its_lifecycle_only_when_the_move_is_legal() {
    let mut entity = alice();
    assert_eq!(entity.lifecycle(), LifecycleState::Active);

    entity.transition_to(LifecycleState::Dormant).unwrap();
    assert_eq!(entity.lifecycle(), LifecycleState::Dormant);

    entity.transition_to(LifecycleState::Active).unwrap();
    entity.transition_to(LifecycleState::Destroyed).unwrap();
    assert_eq!(entity.lifecycle(), LifecycleState::Destroyed);

    for refused in [
        LifecycleState::Active,
        LifecycleState::Dormant,
        LifecycleState::Destroyed,
    ] {
        assert_eq!(
            entity.transition_to(refused),
            Err(ContractError::IllegalLifecycleTransition {
                entity: EntityId::from_raw(1),
                from: LifecycleState::Destroyed,
                to: refused,
            })
        );
        assert_eq!(
            entity.lifecycle(),
            LifecycleState::Destroyed,
            "a refused transition must not change the entity"
        );
    }
}

/// Tags are a set: supplying one twice is not an error, and the order they were authored in
/// does not survive into the value, so two authors who wrote the same tags in a different
/// order produce byte-identical state.
#[test]
fn tags_are_a_set_with_one_fixed_order() {
    let authored_one_way = Tags::new([tag("cafe"), tag("furniture"), tag("cafe")]);
    let authored_another_way = Tags::new([tag("furniture"), tag("cafe")]);

    assert_eq!(authored_one_way.len(), 2);
    assert_eq!(authored_one_way, authored_another_way);
    assert!(authored_one_way.contains(&tag("cafe")));
    assert!(!authored_one_way.contains(&tag("park")));

    let labels: Vec<&str> = authored_one_way.iter().map(Tag::as_str).collect();
    assert_eq!(labels, ["cafe", "furniture"]);

    assert_eq!(
        serde_json::to_string(&authored_one_way).unwrap(),
        r#"["cafe","furniture"]"#
    );
    assert_eq!(
        serde_json::to_vec(&authored_one_way).unwrap(),
        serde_json::to_vec(&authored_another_way).unwrap(),
        "equal tag sets must serialize to identical bytes"
    );

    // A duplicate in stored or authored data is absorbed, not rejected.
    assert_eq!(
        serde_json::from_str::<Tags>(r#"["furniture","cafe","cafe"]"#).unwrap(),
        authored_one_way
    );
}

/// A tag is an authored name and obeys the same character rule as the other authored names,
/// reported as a tag rather than as some other kind of identifier.
#[test]
fn a_tag_is_a_validated_authored_name() {
    assert!(Tag::new("night-shift").is_ok());
    assert_eq!(
        Tag::new("Night Shift"),
        Err(ContractError::IdentifierIllegalCharacter {
            kind: IdentifierKind::Tag,
            character: 'N',
            position: 0,
        })
    );
    assert_eq!(
        Tag::new(""),
        Err(ContractError::IdentifierEmpty {
            kind: IdentifierKind::Tag
        })
    );
}

/// The stored shape of an entity, asserted exactly. An entity authored without provenance is
/// representable, and its metadata is absent from the record rather than present and null.
#[test]
fn an_entity_is_stored_as_its_documented_shape() {
    let plain = alice();
    assert_eq!(
        serde_json::to_string(&plain).unwrap(),
        r#"{"id":"1","key":"alice","entity_type":"person","tags":[],"lifecycle":"active"}"#
    );

    let authored = alice()
        .with_tags(Tags::new([tag("resident"), tag("barista")]))
        .with_metadata(Metadata {
            source_pack: "lakewood".to_owned(),
            source_path: "people/alice.yaml".to_owned(),
            authoring_note: None,
        });
    assert_eq!(
        serde_json::to_string(&authored).unwrap(),
        concat!(
            r#"{"id":"1","key":"alice","entity_type":"person","tags":["barista","resident"],"#,
            r#""lifecycle":"active","#,
            r#""metadata":{"source_pack":"lakewood","source_path":"people/alice.yaml","#,
            r#""authoring_note":null}}"#
        )
    );

    assert_eq!(
        serde_json::from_str::<Entity>(
            r#"{"id":"1","key":"alice","entity_type":"person","tags":[],"lifecycle":"active"}"#
        )
        .unwrap(),
        plain
    );
    assert_eq!(
        serde_json::from_str::<Entity>(&serde_json::to_string(&authored).unwrap()).unwrap(),
        authored
    );
}

/// Every field of a restored entity is readable and equal to what was stored, and a lifecycle
/// value the taxonomy does not contain is an error rather than a silent default — a restored
/// world must not turn an unknown state into `Active` and start simulating a destroyed entity.
#[test]
fn a_restored_entity_keeps_every_field_and_rejects_an_unknown_lifecycle() {
    let mut stored = alice().with_tags(Tags::new([tag("resident")]));
    stored.transition_to(LifecycleState::Dormant).unwrap();

    let restored: Entity = serde_json::from_str(&serde_json::to_string(&stored).unwrap()).unwrap();
    assert_eq!(restored.id(), EntityId::from_raw(1));
    assert_eq!(restored.key().as_str(), "alice");
    assert_eq!(restored.entity_type(), EntityType::Person);
    assert_eq!(restored.tags(), &Tags::new([tag("resident")]));
    assert_eq!(restored.lifecycle(), LifecycleState::Dormant);
    assert_eq!(restored.metadata(), None);

    assert!(
        serde_json::from_str::<Entity>(
            r#"{"id":"1","key":"alice","entity_type":"person","tags":[],"lifecycle":"archived"}"#
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<Entity>(
            r#"{"id":"1","key":"Alice","entity_type":"person","tags":[],"lifecycle":"active"}"#
        )
        .is_err(),
        "an invalid authored key must not survive a round trip through storage"
    );
}
