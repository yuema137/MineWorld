//! Contract tests for the identity vocabulary.
//!
//! These run against the crate's public API only, which is the surface every other layer will
//! use. Every expected value here is written by hand: none is produced by calling the code
//! under test, so a change in behavior cannot silently change what the test expects.

use mineworld_contracts::{
    ActionId, ActionTypeId, Component, ComponentRecord, ComponentSchemaVersion, ComponentTypeId,
    ContractError, EntityId, EntityKey, EntityType, Event, EventId, EventRecord,
    EventSchemaVersion, EventTypeId, IdentifierKind, ItemId, OrganizationId, PersonId, PlaceId,
    ProcessId, ProcessTypeId, RejectionCode, RelationTypeId, SystemId,
};
use serde::{Deserialize, Serialize};

mod support;

use support::{Written, read_binary, write_binary};

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
    assert!(ProcessTypeId::new("having-dinner").is_ok());
    assert_eq!(
        ProcessTypeId::new("Dinner"),
        Err(ContractError::IdentifierIllegalCharacter {
            kind: IdentifierKind::ProcessTypeId,
            character: 'D',
            position: 0,
        })
    );
    assert!(
        serde_json::from_str::<ProcessTypeId>("\"having dinner\"").is_err(),
        "a persisted process type is read through the same rule"
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

/// The opaque identities are written as decimal strings in a human-readable format. The wire shape
/// is a contract: the event log, the persistence layer and every client read it back, so changing
/// it later would be a migration rather than a refactor.
#[test]
fn opaque_identities_are_written_as_decimal_strings_in_json() {
    assert_eq!(
        serde_json::to_string(&EntityId::from_raw(7)).unwrap(),
        "\"7\""
    );
    assert_eq!(
        serde_json::to_string(&EventId::from_raw(8)).unwrap(),
        "\"8\""
    );
    assert_eq!(
        serde_json::to_string(&ActionId::from_raw(9)).unwrap(),
        "\"9\""
    );
    assert_eq!(
        serde_json::to_string(&ProcessId::from_raw(10)).unwrap(),
        "\"10\""
    );

    assert_eq!(
        serde_json::from_str::<EntityId>("\"7\"").unwrap(),
        EntityId::from_raw(7)
    );
    assert_eq!(
        serde_json::from_str::<EventId>("\"8\"").unwrap(),
        EventId::from_raw(8)
    );
    assert_eq!(
        serde_json::from_str::<ActionId>("\"9\"").unwrap(),
        ActionId::from_raw(9)
    );
    assert_eq!(
        serde_json::from_str::<ProcessId>("\"10\"").unwrap(),
        ProcessId::from_raw(10)
    );
}

/// The five identities the renderer spike chose to make the damage unmissable, round-tripped
/// through JSON exactly.
///
/// Every expected string below is the decimal expansion written by hand. The second half of the
/// test is the reason the first half matters: as IEEE-754 doubles these five *distinct* values
/// collapse onto three, which is what `spike/FINDINGS.md` F1 measured from inside Godot —
/// `9007199254740995` (Alice) and `9007199254740997` (a mug) both arriving as `9007199254740996`,
/// so a client that bound a body to a parsed number would raycast the mug and talk to Alice.
///
/// Above 2^53 = `9007199254740992` the representable doubles are two apart, so an odd integer is
/// exactly halfway between two of them and ties to the even mantissa. That is arithmetic, not a
/// measurement, which is why the expected values here are derived rather than observed.
#[test]
fn the_2_53_boundary_survives_json_for_every_opaque_identity() {
    let raws: [u64; 5] = [
        9_007_199_254_740_993,
        9_007_199_254_740_995,
        9_007_199_254_740_997,
        9_007_199_254_740_999,
        9_007_199_254_741_001,
    ];
    let expected = [
        "\"9007199254740993\"",
        "\"9007199254740995\"",
        "\"9007199254740997\"",
        "\"9007199254740999\"",
        "\"9007199254741001\"",
    ];

    for (raw, text) in raws.into_iter().zip(expected) {
        assert_eq!(
            serde_json::to_string(&EntityId::from_raw(raw)).unwrap(),
            text
        );
        assert_eq!(
            serde_json::to_string(&EventId::from_raw(raw)).unwrap(),
            text
        );
        assert_eq!(
            serde_json::to_string(&ActionId::from_raw(raw)).unwrap(),
            text
        );
        assert_eq!(
            serde_json::to_string(&ProcessId::from_raw(raw)).unwrap(),
            text
        );

        assert_eq!(
            serde_json::from_str::<EntityId>(text).unwrap(),
            EntityId::from_raw(raw),
            "the exact value must come back, not the nearest double"
        );
        assert_eq!(
            serde_json::from_str::<EventId>(text).unwrap(),
            EventId::from_raw(raw)
        );
        assert_eq!(
            serde_json::from_str::<ActionId>(text).unwrap(),
            ActionId::from_raw(raw)
        );
        assert_eq!(
            serde_json::from_str::<ProcessId>(text).unwrap(),
            ProcessId::from_raw(raw)
        );
    }

    // What a parser with one number type would have done to those same five values, so the
    // corruption this encoding prevents is stated rather than assumed. Five distinct ids, three
    // distinct doubles, and two collisions.
    let as_doubles: Vec<u64> = raws.iter().map(|raw| *raw as f64 as u64).collect();
    assert_eq!(
        as_doubles,
        vec![
            9_007_199_254_740_992,
            9_007_199_254_740_996,
            9_007_199_254_740_996,
            9_007_199_254_741_000,
            9_007_199_254_741_000,
        ]
    );
}

/// An `EntityId` inside a **component payload** survives JSON — the failure class
/// `spike/FINDINGS.md` F2 identified, and the reason this rule lives on the type rather than at a
/// protocol boundary.
///
/// `ComponentRecord` is a documented payload-erasure boundary: the contract layer never interprets
/// `P`, so neither can a protocol layer, so a wire encoder cannot find the ids in here. Both
/// payload shapes below are the ones F2 names as the normal case for a System Pack — a component
/// that refers to another entity.
#[test]
fn an_id_inside_a_component_payload_survives_json() {
    let alice = EntityId::from_raw(9_007_199_254_740_995);
    let employer = EntityId::from_raw(9_007_199_254_740_993);

    let employment = Employment { employer };
    let record: ComponentRecord<String> = ComponentRecord::new::<Employment>(
        alice,
        serde_json::to_string(&employment).expect("a payload serializes"),
    );

    let text = serde_json::to_string(&record).expect("a record serializes");
    assert_eq!(
        text,
        r#"{"entity":"9007199254740995","component_type":"employment","schema_version":1,"payload":"{\"employer\":\"9007199254740993\"}"}"#,
        "the id in the payload must be a string, exactly as the id in the envelope is"
    );

    let read: ComponentRecord<String> = serde_json::from_str(&text).expect("the record reads back");
    let payload: Employment = serde_json::from_str(
        read.payload_for::<Employment>()
            .expect("the record is labelled for this component"),
    )
    .expect("the payload reads back");
    assert_eq!(payload.employer, employer);
    assert_eq!(read.entity(), alice);

    // The other shape F2 names, to show the rule is about the type and not about one field name.
    let conversation = Conversation { talking_to: alice };
    let encoded = serde_json::to_string(&conversation).expect("a payload serializes");
    assert_eq!(encoded, r#"{"talking_to":"9007199254740995"}"#);
    assert_eq!(
        serde_json::from_str::<Conversation>(&encoded)
            .expect("the payload reads back")
            .talking_to,
        alice
    );
}

/// The same for an **event payload**, which is the other erasure boundary and the permanent one:
/// an event log is append-only, so an id corrupted on the way in is corrupted for the life of the
/// world.
#[test]
fn an_id_inside_an_event_payload_survives_json() {
    let mug = EntityId::from_raw(9_007_199_254_740_997);
    let record: EventRecord<String> = EventRecord::new::<ItemPickedUp>(
        serde_json::to_string(&ItemPickedUp { item: mug }).expect("a payload serializes"),
    );

    let text = serde_json::to_string(&record).expect("a record serializes");
    assert_eq!(
        text,
        r#"{"event_type":"item-picked-up","schema_version":1,"payload":"{\"item\":\"9007199254740997\"}"}"#
    );

    let read: EventRecord<String> = serde_json::from_str(&text).expect("the record reads back");
    let payload: ItemPickedUp = serde_json::from_str(
        read.payload_for::<ItemPickedUp>()
            .expect("the record is labelled for this event"),
    )
    .expect("the payload reads back");
    assert_eq!(payload.item, mug);
}

/// A format that is **not** human-readable still receives the `u64`, which is the half of the rule
/// that keeps `DD-15`'s original objection answered: persistence and replay determinism are
/// untouched, because a binary encoding sees exactly what it saw before.
///
/// The instrument is `tests/support`: a serializer and a deserializer that report
/// `is_human_readable() == false`. The deserializer also refuses `deserialize_any`, because a real
/// binary format is not self-describing — so an implementation that forgot to branch and reached
/// for `deserialize_any` fails here exactly as it would against `bincode` or `postcard`.
#[test]
fn a_binary_format_encodes_an_opaque_identity_as_a_number() {
    let raw = 9_007_199_254_740_993;

    assert_eq!(
        write_binary(&EntityId::from_raw(raw)).expect("an entity id writes"),
        Written::Unsigned(raw)
    );
    assert_eq!(
        write_binary(&EventId::from_raw(raw)).expect("an event id writes"),
        Written::Unsigned(raw)
    );
    assert_eq!(
        write_binary(&ActionId::from_raw(raw)).expect("an action id writes"),
        Written::Unsigned(raw)
    );
    assert_eq!(
        write_binary(&ProcessId::from_raw(raw)).expect("a process id writes"),
        Written::Unsigned(raw)
    );

    assert_eq!(
        read_binary::<EntityId>(raw).expect("an entity id reads back"),
        EntityId::from_raw(raw)
    );
    assert_eq!(
        read_binary::<EventId>(raw).expect("an event id reads back"),
        EventId::from_raw(raw)
    );
    assert_eq!(
        read_binary::<ActionId>(raw).expect("an action id reads back"),
        ActionId::from_raw(raw)
    );
    assert_eq!(
        read_binary::<ProcessId>(raw).expect("a process id reads back"),
        ProcessId::from_raw(raw)
    );
}

/// JSON accepts both forms on the way in, so an authored file, a saved world or a hand-written
/// test datum that holds a number still loads. Only one form is ever *written*.
#[test]
fn json_accepts_both_the_string_and_the_number_form() {
    let raw = 9_007_199_254_740_993;
    let expected = EntityId::from_raw(raw);

    assert_eq!(
        serde_json::from_str::<EntityId>("\"9007199254740993\"").unwrap(),
        expected
    );
    assert_eq!(
        serde_json::from_str::<EntityId>("9007199254740993").unwrap(),
        expected,
        "the number form must still load: existing fixtures hold numbers"
    );
    assert_eq!(
        serde_json::from_str::<EventId>("8").unwrap(),
        EventId::from_raw(8)
    );
    assert_eq!(
        serde_json::from_str::<ActionId>("\"8\"").unwrap(),
        ActionId::from_raw(8)
    );

    // A negative number is not an identity, whichever way it is written.
    assert!(serde_json::from_str::<EntityId>("-1").is_err());
    assert!(serde_json::from_str::<EntityId>("\"-1\"").is_err());
    assert!(serde_json::from_str::<EntityId>("\"\"").is_err());
    assert!(serde_json::from_str::<EntityId>("\"seven\"").is_err());
}

/// A float is refused rather than truncated.
///
/// This is the whole point restated from the other side: `9007199254740993.0` is what a client with
/// one number type produces, and it has *already* lost the value. Accepting it would resolve the id
/// to `9007199254740992` — a different entity, silently. `spike/FINDINGS.md` F9 records the same
/// discipline working for `Millimetres`, where the contract's integer types refuse a float loudly.
#[test]
fn a_float_is_refused_rather_than_truncated() {
    let refused = serde_json::from_str::<EntityId>("9007199254740993.0")
        .expect_err("a float must not deserialize into an identity");
    assert!(
        refused.to_string().contains("floating point"),
        "the refusal should name the float, but was: {refused}"
    );

    for text in ["1.5", "-1.0", "1e3", "\"1.5\"", "\"1e3\""] {
        assert!(
            serde_json::from_str::<EntityId>(text).is_err(),
            "{text} must not deserialize into an identity"
        );
        assert!(serde_json::from_str::<EventId>(text).is_err());
        assert!(serde_json::from_str::<ActionId>(text).is_err());
        assert!(serde_json::from_str::<ProcessId>(text).is_err());
    }
}

/// An employment component: `spike/FINDINGS.md` F2's first example of a payload that carries an
/// `EntityId`, and a stand-in for one only — the contract layer knows nothing about employment,
/// and this declaration lives in a test because that is the only place in this crate an action,
/// component or event name may appear.
#[derive(Debug, Serialize, Deserialize)]
struct Employment {
    employer: EntityId,
}

impl Component for Employment {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("employment");
    const OWNER: SystemId = SystemId::from_static("employment-stub");
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(1);
}

/// F2's second example, to show the rule is not keyed on a field name.
#[derive(Debug, Serialize, Deserialize)]
struct Conversation {
    talking_to: EntityId,
}

impl Component for Conversation {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("conversation");
    const OWNER: SystemId = SystemId::from_static("conversation-stub");
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(1);
}

/// The event-payload half of the same failure class.
#[derive(Debug, Serialize, Deserialize)]
struct ItemPickedUp {
    item: EntityId,
}

impl Event for ItemPickedUp {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("item-picked-up");
    const OWNER: SystemId = SystemId::from_static("inventory-stub");
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
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
    let encoded = r#"{"entity":"3","entity_type":"person"}"#;

    assert_eq!(serde_json::to_string(&person).unwrap(), encoded);
    assert_eq!(serde_json::from_str::<PersonId>(encoded).unwrap(), person);

    let relabelled = serde_json::from_str::<PersonId>(r#"{"entity":"3","entity_type":"place"}"#)
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
