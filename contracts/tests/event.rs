//! Contract tests for the fact half of the pipeline.
//!
//! The event types are named the way `CORE_CONCEPTS.md` §11 names its examples — and the crate
//! still has no idea what they mean, which is `INV-12`. Encoding is `serde_json` because a test
//! needs one; the contract layer chooses none, so these tests stand in for the persistence layer
//! and the protocol that eventually will. Every expected value is written by hand from the
//! documented shape of the types, never from the output of the code under test.

use std::collections::BTreeSet;

use mineworld_contracts::{
    ActionId, Causation, ContractError, EntityId, EntityType, Event, EventEnvelope, EventId,
    EventRecord, EventSchemaVersion, EventTypeId, PlaceId, ProcessId, Provenance, SystemId,
    Visibility, WorldTime,
};
use serde::{Deserialize, Serialize};

/// A fact one stub system emits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ItemTransferred {
    item: EntityId,
}

impl Event for ItemTransferred {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("item-transferred");
    const OWNER: SystemId = SystemId::from_static("inventory-stub");
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// A fact a *different* stub system emits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ConversationStarted {
    opener: EntityId,
}

impl Event for ConversationStarted {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("conversation-started");
    const OWNER: SystemId = SystemId::from_static("conversation-stub");
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

fn place() -> PlaceId {
    PlaceId::new(EntityId::from_raw(7), EntityType::Place).unwrap()
}

fn transfer_payload() -> EventRecord<String> {
    EventRecord::new::<ItemTransferred>(r#"{"item":18517}"#.to_owned())
}

fn envelope() -> EventEnvelope<String> {
    EventEnvelope::new(
        EventId::from_raw(9001),
        WorldTime::from_seconds(64_800),
        transfer_payload(),
        Causation::Action(ActionId::from_raw(3)),
        Visibility::Place(place()),
        Provenance::new(SystemId::from_static("inventory-stub")),
    )
}

/// Erasing a fact and reading it back loses nothing, and a record is handed only to the event type
/// it was written for. Replaying a log by decoding whatever payload happens to fit would rebuild a
/// state that never existed, which is the one failure the event log cannot be allowed to have.
#[test]
fn a_fact_survives_erasure_and_is_readable_only_as_its_own_event_type() {
    let written = ItemTransferred {
        item: EntityId::from_raw(18_517),
    };
    let record = EventRecord::new::<ItemTransferred>(
        serde_json::to_vec(&written).expect("an event payload must be encodable"),
    );

    assert_eq!(
        record.event_type(),
        &EventTypeId::new("item-transferred").unwrap()
    );
    let payload = record
        .payload_for::<ItemTransferred>()
        .expect("a record written from ItemTransferred is readable as ItemTransferred");
    assert_eq!(
        serde_json::from_slice::<ItemTransferred>(payload).unwrap(),
        written
    );

    assert_eq!(
        record.payload_for::<ConversationStarted>().unwrap_err(),
        ContractError::EventTypeMismatch {
            expected: EventTypeId::new("conversation-started").unwrap(),
            actual: EventTypeId::new("item-transferred").unwrap(),
        }
    );
}

/// Every way a fact can come about must survive the log, or replay would lose the causal chain
/// that `INV-15` exists to guarantee. `WorldGenesis` is the case that matters most: it is what
/// makes a world's initial state explained rather than uncaused, and it must not degenerate into
/// an absent cause on the way to disk.
#[test]
fn every_way_a_fact_can_be_caused_survives_the_log() {
    let causes = [
        Causation::Action(ActionId::from_raw(3)),
        Causation::Process(ProcessId::from_raw(12)),
        Causation::Event(EventId::from_raw(9000)),
        Causation::SystemTick {
            system: SystemId::from_static("inventory-stub"),
        },
        Causation::WorldGenesis,
    ];

    for cause in &causes {
        let text = serde_json::to_string(cause).unwrap();
        assert_eq!(
            &serde_json::from_str::<Causation>(&text).unwrap(),
            cause,
            "{text} must round-trip"
        );
    }

    assert_eq!(
        serde_json::to_string(&Causation::WorldGenesis).unwrap(),
        r#""world_genesis""#
    );
    assert_eq!(
        serde_json::to_string(&Causation::SystemTick {
            system: SystemId::from_static("inventory-stub"),
        })
        .unwrap(),
        r#"{"system_tick":{"system":"inventory-stub"}}"#
    );
}

/// An audience must survive the log, and an explicit audience must be byte-identical however it
/// was assembled. A world whose event log differs between two runs because a set was filled in a
/// different order cannot be reproduced from a seed, which is what `AC-12` requires of it.
#[test]
fn a_declared_audience_survives_the_log_and_does_not_depend_on_insertion_order() {
    let audiences = [
        Visibility::Public,
        Visibility::Place(place()),
        Visibility::Participants,
        Visibility::Entities(BTreeSet::from([
            EntityId::from_raw(41),
            EntityId::from_raw(42),
        ])),
        Visibility::SystemInternal,
    ];

    for audience in &audiences {
        let text = serde_json::to_string(audience).unwrap();
        assert_eq!(
            &serde_json::from_str::<Visibility>(&text).unwrap(),
            audience,
            "{text} must round-trip"
        );
    }

    let ascending = Visibility::Entities(BTreeSet::from([
        EntityId::from_raw(41),
        EntityId::from_raw(42),
        EntityId::from_raw(43),
    ]));
    let descending = Visibility::Entities(BTreeSet::from([
        EntityId::from_raw(43),
        EntityId::from_raw(42),
        EntityId::from_raw(41),
    ]));
    assert_eq!(
        serde_json::to_string(&ascending).unwrap(),
        r#"{"entities":[41,42,43]}"#
    );
    assert_eq!(
        serde_json::to_string(&descending).unwrap(),
        serde_json::to_string(&ascending).unwrap()
    );
    assert_eq!(
        serde_json::to_string(&Visibility::SystemInternal).unwrap(),
        r#""system_internal""#
    );
}

/// `INV-15` and the declared-audience rule must hold for a log read from disk, not only for one
/// built in memory: a record with no cause or no audience is refused where it is read. A missing
/// audience is the more insidious of the two, because a perception system given no audience would
/// have to choose a default, and the only defaults available are omniscience and silence.
#[test]
fn a_fact_with_no_cause_or_no_audience_cannot_be_read_back() {
    let complete = r#"{"id":9001,"at":64800,"event_type":"item-transferred","subjects":[],"participants":[],"place":null,"caused_by":"world_genesis","payload":{"event_type":"item-transferred","schema_version":1,"payload":"{}"},"visibility":"public","provenance":{"emitted_by":"inventory-stub","controller_decision":null}}"#;
    assert!(serde_json::from_str::<EventEnvelope<String>>(complete).is_ok());

    let causeless = complete.replace(r#""caused_by":"world_genesis","#, "");
    let error = serde_json::from_str::<EventEnvelope<String>>(&causeless)
        .expect_err("an event with no cause must not be readable");
    assert!(
        error.to_string().contains("caused_by"),
        "the error must name the missing field, but said: {error}"
    );

    let audienceless = complete.replace(r#""visibility":"public","#, "");
    let error = serde_json::from_str::<EventEnvelope<String>>(&audienceless)
        .expect_err("an event with no declared audience must not be readable");
    assert!(
        error.to_string().contains("visibility"),
        "the error must name the missing field, but said: {error}"
    );
}

/// An envelope labels the fact and its payload record carries it. If those two could disagree, a
/// log reader would select a reducer by the label and hand it another system's bytes.
#[test]
fn an_envelope_cannot_disagree_with_its_payload() {
    assert_eq!(envelope().event_type(), envelope().payload().event_type());

    let disagreeing = r#"{"id":9001,"at":0,"event_type":"conversation-started","subjects":[],"participants":[],"place":null,"caused_by":"world_genesis","payload":{"event_type":"item-transferred","schema_version":1,"payload":"{}"},"visibility":"public","provenance":{"emitted_by":"inventory-stub","controller_decision":null}}"#;
    let error = serde_json::from_str::<EventEnvelope<String>>(disagreeing)
        .expect_err("an envelope whose label and payload disagree is refused");
    assert_eq!(
        error.to_string(),
        ContractError::EventEnvelopePayloadMismatch {
            event_type: EventTypeId::new("conversation-started").unwrap(),
            payload_event_type: EventTypeId::new("item-transferred").unwrap(),
        }
        .to_string()
    );
}

/// The stored shape of an event, asserted exactly: this is the event log's own format, so changing
/// it is a migration of every world's history rather than a refactor. The subjects keep the order
/// the emitting system stated them in, because for a two-ended fact the order is part of the fact.
#[test]
fn an_event_is_stored_as_its_documented_shape() {
    let recorded = envelope()
        .about(vec![EntityId::from_raw(41), EntityId::from_raw(42)])
        .with_participants(vec![EntityId::from_raw(43)])
        .at_place(place());
    let audited = EventEnvelope::new(
        EventId::from_raw(9002),
        WorldTime::EPOCH,
        transfer_payload(),
        Causation::Action(ActionId::from_raw(3)),
        Visibility::Participants,
        Provenance::new(SystemId::from_static("inventory-stub"))
            .from_controller_decision(ActionId::from_raw(3)),
    );

    let text = r#"{"id":9001,"at":64800,"event_type":"item-transferred","subjects":[41,42],"participants":[43],"place":{"entity":7,"entity_type":"place"},"caused_by":{"action":3},"payload":{"event_type":"item-transferred","schema_version":1,"payload":"{\"item\":18517}"},"visibility":{"place":{"entity":7,"entity_type":"place"}},"provenance":{"emitted_by":"inventory-stub","controller_decision":null}}"#;
    assert_eq!(serde_json::to_string(&recorded).unwrap(), text);
    assert_eq!(
        serde_json::from_str::<EventEnvelope<String>>(text).unwrap(),
        recorded
    );
    assert_eq!(
        recorded.subjects(),
        [EntityId::from_raw(41), EntityId::from_raw(42)]
    );

    // A controller's decision is recorded beside the emitting system, which is what makes an LM
    // controller's effect on a world auditable after the fact.
    assert_eq!(
        serde_json::to_string(audited.provenance()).unwrap(),
        r#"{"emitted_by":"inventory-stub","controller_decision":3}"#
    );
    assert_eq!(
        audited.provenance().controller_decision(),
        Some(ActionId::from_raw(3))
    );
}

/// The same fact, as a later version of the emitting system writes it.
///
/// Same event type, different schema — which is exactly the situation a permanent log meets
/// when a System Pack is upgraded and an old world is replayed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ItemTransferredV2 {
    item: EntityId,
    quantity: u32,
}

impl Event for ItemTransferredV2 {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("item-transferred");
    const OWNER: SystemId = SystemId::from_static("inventory-stub");
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(2);
}

#[test]
fn a_record_carries_the_schema_its_payload_was_written_against() {
    let record = EventRecord::<String>::new::<ItemTransferred>("{}".to_owned());
    // Readable without decoding the payload: an old record has to be recognizable as old
    // before anything tries to interpret it.
    assert_eq!(record.schema_version(), EventSchemaVersion::new(1));
}

#[test]
fn an_event_payload_from_a_newer_schema_is_refused_rather_than_decoded() {
    let newer = EventRecord::<String>::new::<ItemTransferredV2>("{}".to_owned());

    let refused = newer.payload_for::<ItemTransferred>();

    match refused {
        Err(ContractError::EventSchemaTooNew {
            event_type,
            record,
            supported,
        }) => {
            assert_eq!(event_type, ItemTransferred::EVENT_TYPE);
            assert_eq!(record, EventSchemaVersion::new(2));
            assert_eq!(supported, EventSchemaVersion::new(1));
        }
        other => panic!("a newer payload must be refused by version, got {other:?}"),
    }
}

#[test]
fn an_event_payload_from_an_older_schema_is_reported_as_history_to_migrate() {
    let older = EventRecord::<String>::new::<ItemTransferred>("{}".to_owned());

    let refused = older.payload_for::<ItemTransferredV2>();

    match refused {
        Err(ContractError::EventSchemaOutdated {
            event_type,
            record,
            supported,
        }) => {
            assert_eq!(event_type, ItemTransferredV2::EVENT_TYPE);
            assert_eq!(record, EventSchemaVersion::new(1));
            assert_eq!(supported, EventSchemaVersion::new(2));
        }
        other => panic!("an older payload must be reported as outdated, got {other:?}"),
    }
}

#[test]
fn the_version_check_is_not_satisfied_by_a_matching_event_type_alone() {
    // The failure this guards: both records claim event type "item-transferred", so a check on
    // the type alone would hand v2 bytes to v1 and rebuild a history that never happened.
    let v1 = EventRecord::<String>::new::<ItemTransferred>("{}".to_owned());
    let v2 = EventRecord::<String>::new::<ItemTransferredV2>("{}".to_owned());

    assert_eq!(v1.event_type(), v2.event_type());
    assert!(v1.payload_for::<ItemTransferred>().is_ok());
    assert!(v2.payload_for::<ItemTransferred>().is_err());
}
