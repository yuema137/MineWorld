//! Contract tests for the component model.
//!
//! The components here are named for their role in these tests, not for anything a world would
//! call state: the contract layer has no domain concepts to test with, which is the point.
//! Encoding is `serde_json` because a test needs *some* encoding; the contract layer does not
//! choose one, so the tests stand in for the persistence layer that eventually will.

use mineworld_contracts::{
    Component, ComponentDeclaration, ComponentRecord, ComponentSchemaVersion, ComponentTypeId,
    ContractError, EntityId, SystemId,
};
use serde::{Deserialize, Serialize};

/// State owned by the first stub system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Measured {
    amount: u32,
    label: String,
}

impl Component for Measured {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("measured");
    const OWNER: SystemId = SystemId::from_static("first-stub");
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(1);
}

/// State owned by the second stub system — a different owner, so that ownership is visibly a
/// property of the component type rather than of the crate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Flagged {
    raised: bool,
}

impl Component for Flagged {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("flagged");
    const OWNER: SystemId = SystemId::from_static("second-stub");
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(1);
}

/// The same component type as [`Measured`], one schema version later: what the first stub system
/// looks like after it ships a schema change, and what lets these tests hold a record from a
/// version other than the one being read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MeasuredV2 {
    amount: u32,
    label: String,
    note: Option<String>,
}

impl Component for MeasuredV2 {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("measured");
    const OWNER: SystemId = SystemId::from_static("first-stub");
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(2);
}

/// The same component type as [`Measured`], claimed by a different system: the declaration a
/// registry has to refuse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MeasuredByTheWrongSystem {
    amount: u32,
}

impl Component for MeasuredByTheWrongSystem {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("measured");
    const OWNER: SystemId = SystemId::from_static("second-stub");
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(1);
}

fn encode<C: Component>(component: &C) -> Vec<u8> {
    serde_json::to_vec(component).expect("a component must be encodable")
}

fn entity() -> EntityId {
    EntityId::from_raw(42)
}

/// A declaration says what the component type is, who owns it, and which schema version that
/// owner writes — the three facts the kernel's registry needs, taken from the type rather than
/// from whoever is registering it.
#[test]
fn a_declaration_reports_the_component_types_own_facts() {
    let measured = ComponentDeclaration::of::<Measured>();
    assert_eq!(
        measured.component_type(),
        &ComponentTypeId::new("measured").unwrap()
    );
    assert_eq!(measured.owner(), &SystemId::new("first-stub").unwrap());
    assert_eq!(measured.schema_version(), ComponentSchemaVersion::new(1));

    let flagged = ComponentDeclaration::of::<Flagged>();
    assert_eq!(
        flagged.component_type(),
        &ComponentTypeId::new("flagged").unwrap()
    );
    assert_eq!(flagged.owner(), &SystemId::new("second-stub").unwrap());

    // The declaration is a value, so a registry can store and compare it.
    assert_eq!(
        serde_json::to_string(&measured).unwrap(),
        r#"{"component_type":"measured","owner":"first-stub","schema_version":1}"#
    );
}

/// The conflict the single-writer rule forbids is detectable from declarations alone, before any
/// world runs: two systems claiming one component type. Two declarations of the same type by the
/// same system are not a conflict — that is one system registering twice, or one system's own
/// schema moving on.
#[test]
fn two_owners_of_one_component_type_are_a_conflict() {
    let owner = ComponentDeclaration::of::<Measured>();
    let other_owner = ComponentDeclaration::of::<MeasuredByTheWrongSystem>();
    let same_owner_next_version = ComponentDeclaration::of::<MeasuredV2>();
    let unrelated = ComponentDeclaration::of::<Flagged>();

    assert!(owner.conflicts_with(&other_owner));
    assert!(other_owner.conflicts_with(&owner));

    assert!(!owner.conflicts_with(&same_owner_next_version));
    assert!(!owner.conflicts_with(&unrelated));
    assert!(!owner.conflicts_with(&owner));
}

/// Erasing a component and reading it back loses nothing, and the record carries the label the
/// component type gave it rather than one the caller chose.
#[test]
fn a_component_survives_erasure_and_comes_back_whole() {
    let written = Measured {
        amount: 3,
        label: "left".to_owned(),
    };
    let record = ComponentRecord::new::<Measured>(entity(), encode(&written));

    assert_eq!(record.entity(), entity());
    assert_eq!(
        record.component_type(),
        &ComponentTypeId::new("measured").unwrap()
    );
    assert_eq!(record.schema_version(), ComponentSchemaVersion::new(1));

    let payload = record
        .payload_for::<Measured>()
        .expect("a record written from Measured is readable as Measured");
    let read: Measured = serde_json::from_slice(payload).unwrap();
    assert_eq!(read, written);
}

/// A record is only handed to the component type it was written for. Without this, a routing
/// mistake would decode one system's bytes as another system's state — and with `serde` defaults
/// it could even succeed, producing a plausible value that was never written.
#[test]
fn a_record_is_not_readable_as_another_component_type() {
    let record = ComponentRecord::new::<Flagged>(entity(), encode(&Flagged { raised: true }));

    assert_eq!(
        record.payload_for::<Measured>().unwrap_err(),
        ContractError::ComponentTypeMismatch {
            expected: ComponentTypeId::new("measured").unwrap(),
            actual: ComponentTypeId::new("flagged").unwrap(),
        }
    );
}

/// The two directions of a schema difference are different problems and get different errors: a
/// newer record cannot be read by this code at all, while an older one is ordinary history for a
/// migration to bring forward. Neither is silently decoded.
#[test]
fn a_schema_difference_is_reported_in_the_direction_it_points() {
    let newer = ComponentRecord::new::<MeasuredV2>(
        entity(),
        encode(&MeasuredV2 {
            amount: 3,
            label: "left".to_owned(),
            note: Some("added in v2".to_owned()),
        }),
    );
    assert_eq!(
        newer.payload_for::<Measured>().unwrap_err(),
        ContractError::ComponentSchemaTooNew {
            component_type: ComponentTypeId::new("measured").unwrap(),
            record: ComponentSchemaVersion::new(2),
            supported: ComponentSchemaVersion::new(1),
        }
    );

    let older = ComponentRecord::new::<Measured>(
        entity(),
        encode(&Measured {
            amount: 3,
            label: "left".to_owned(),
        }),
    );
    assert_eq!(
        older.payload_for::<MeasuredV2>().unwrap_err(),
        ContractError::ComponentSchemaOutdated {
            component_type: ComponentTypeId::new("measured").unwrap(),
            record: ComponentSchemaVersion::new(1),
            supported: ComponentSchemaVersion::new(2),
        }
    );

    // An outdated record is still readable as data — a migration needs its payload and its
    // version, which is why the version is on the record and not only in the error.
    assert_eq!(older.schema_version(), ComponentSchemaVersion::new(1));
    assert_eq!(
        serde_json::from_slice::<Measured>(older.payload()).unwrap(),
        Measured {
            amount: 3,
            label: "left".to_owned()
        }
    );
}

/// The stored shape of a record, asserted exactly: this is what a persistence layer writes to a
/// row and reads back, so changing it is a migration rather than a refactor.
#[test]
fn a_record_is_stored_as_its_documented_shape() {
    let record = ComponentRecord::new::<Flagged>(EntityId::from_raw(7), "{}".to_owned());

    assert_eq!(
        serde_json::to_string(&record).unwrap(),
        r#"{"entity":7,"component_type":"flagged","schema_version":1,"payload":"{}"}"#
    );
    assert_eq!(
        serde_json::from_str::<ComponentRecord<String>>(
            r#"{"entity":7,"component_type":"flagged","schema_version":1,"payload":"{}"}"#
        )
        .unwrap(),
        record
    );
}
