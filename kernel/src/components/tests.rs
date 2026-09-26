//! Behaviour tests for component storage and for the ownership gate on writes.
//!
//! The systems and components here are named for their role in the tests rather than for anything
//! a world would call state: the kernel has no domain concepts to test with, which is the point
//! (`INV-12`). Two systems, because one system can never demonstrate that ownership is being
//! enforced rather than assumed.
//!
//! The cross-system write is absent from this file on purpose. It cannot be written down in a
//! program that compiles, so it lives in `tests/compile_fail/` instead.
//!
//! These tests moved inside the crate when PR 03b sealed `WriteAccess::new()` (`BD-1`). A write
//! token now exists only inside a world, so no external test crate can obtain one — which is the
//! guarantee, not an obstacle. What a pack author sees from outside is tested from outside, in
//! `kernel/tests/`; what only the kernel can reach is tested here.

use crate::{ComponentStore, KernelError, OwnedBy, SystemIdentity, WriteAccess, WriteToken};
use mineworld_contracts::{
    Component, ComponentRecord, ComponentSchemaVersion, ComponentTypeId, EntityId, SystemId,
};
use serde::{Deserialize, Serialize};

/// The first stub system.
struct FirstStub;
impl SystemIdentity for FirstStub {
    const ID: SystemId = SystemId::from_static("first-stub");
}

/// The second stub system, so that every test can ask "and what about the other one".
struct SecondStub;
impl SystemIdentity for SecondStub {
    const ID: SystemId = SystemId::from_static("second-stub");
}

/// State the first stub system owns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Measured {
    amount: u32,
}

crate::owned_component! {
    component = Measured,
    owner = FirstStub,
    component_type = "measured",
    schema_version = 1,
}

/// State the second stub system owns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Flagged {
    raised: bool,
}

crate::owned_component! {
    component = Flagged,
    owner = SecondStub,
    component_type = "flagged",
    schema_version = 1,
}

/// A component type the first stub system owns but never declares, so that "the owning system is
/// not installed in this world" has something to be tested with.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Unused {
    note: u8,
}

crate::owned_component! {
    component = Unused,
    owner = FirstStub,
    component_type = "unused",
    schema_version = 1,
}

/// The same component type name as [`Measured`], claimed by the other system: the declaration a
/// store has to refuse, written the only way it can be written — by hand.
#[derive(Debug, Serialize, Deserialize)]
struct MeasuredByTheWrongSystem {
    amount: u32,
}

impl Component for MeasuredByTheWrongSystem {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("measured");
    const OWNER: SystemId = SystemId::from_static("second-stub");
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(1);
}

impl OwnedBy<SecondStub> for MeasuredByTheWrongSystem {}

/// The same component type name as [`Measured`], owned by the same system, one schema version
/// later: one name on two Rust types, which would give each an empty table.
#[derive(Debug, Serialize, Deserialize)]
struct MeasuredV2 {
    amount: u32,
    note: Option<String>,
}

impl Component for MeasuredV2 {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("measured");
    const OWNER: SystemId = SystemId::from_static("first-stub");
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(2);
}

impl OwnedBy<FirstStub> for MeasuredV2 {}

/// A component whose own declaration names the first stub system while its ownership relationship
/// claims the second: the one dishonest declaration the type system cannot catch, because
/// `OwnedBy` is a public trait and this crate defines the component type.
#[derive(Debug, Serialize, Deserialize)]
struct MisownedState {
    amount: u32,
}

impl Component for MisownedState {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("misowned");
    const OWNER: SystemId = SystemId::from_static("first-stub");
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(1);
}

impl OwnedBy<SecondStub> for MisownedState {}

fn entity(raw: u64) -> EntityId {
    EntityId::from_raw(raw)
}

/// A world in which both stub systems have been granted write access.
fn two_systems() -> (WriteAccess, WriteToken<FirstStub>, WriteToken<SecondStub>) {
    let mut access = WriteAccess::new();
    let first = access
        .grant(&FirstStub)
        .expect("a system is granted access once");
    let second = access
        .grant(&SecondStub)
        .expect("a system is granted access once");
    (access, first, second)
}

/// A store in which both stub systems have declared their component type.
fn store_with_both_types() -> (
    ComponentStore,
    WriteToken<FirstStub>,
    WriteToken<SecondStub>,
) {
    let (_access, first, second) = two_systems();
    let mut store = ComponentStore::new();
    store
        .declare::<Measured, _>(&first)
        .expect("the owner declares its own component type");
    store
        .declare::<Flagged, _>(&second)
        .expect("the owner declares its own component type");
    (store, first, second)
}

/// The whole write path for an owner: create, change in place, read back, remove. And the read
/// path for everyone else, which needs no token at all — the other system's state is visible
/// without being writable (`KD-4`).
#[test]
fn an_owning_system_writes_its_component_and_every_system_can_read_it() {
    let (mut store, first, second) = store_with_both_types();

    assert_eq!(
        store
            .insert(&first, entity(1), Measured { amount: 3 })
            .expect("the owner may write"),
        None
    );
    store
        .get_mut::<Measured, _>(&first, entity(1))
        .expect("the owner may write")
        .expect("the component was written")
        .amount = 4;

    // No token is involved in either read, and the second system's read is the interesting one:
    // it observes state it can never write.
    assert_eq!(
        store.get::<Measured>(entity(1)),
        Some(&Measured { amount: 4 })
    );
    assert!(store.contains::<Measured>(entity(1)));

    store
        .insert(&second, entity(1), Flagged { raised: true })
        .expect("the other owner may write its own");
    assert_eq!(
        store.get::<Flagged>(entity(1)),
        Some(&Flagged { raised: true })
    );

    assert_eq!(
        store
            .remove::<Measured, _>(&first, entity(1))
            .expect("the owner may write"),
        Some(Measured { amount: 4 })
    );
    assert_eq!(store.get::<Measured>(entity(1)), None);
    assert_eq!(
        store.get::<Flagged>(entity(1)),
        Some(&Flagged { raised: true }),
        "removing one type must not touch another"
    );
}

/// A write to a type no system in this world declared is refused by name, and nothing about the
/// store changes — including the rest of the state the same system does own.
#[test]
fn a_write_to_an_undeclared_component_type_is_refused_and_changes_nothing() {
    let (mut store, first, _second) = store_with_both_types();
    store
        .insert(&first, entity(1), Measured { amount: 7 })
        .expect("the owner may write");

    let refused = store
        .insert(&first, entity(1), Unused { note: 1 })
        .expect_err("an undeclared component type cannot be written");

    assert!(matches!(
        refused,
        KernelError::ComponentTypeNotDeclared { ref component_type }
            if component_type.as_str() == "unused"
    ));
    assert_eq!(store.count::<Unused>(), 0);
    assert_eq!(
        store.get::<Measured>(entity(1)),
        Some(&Measured { amount: 7 })
    );
    assert_eq!(
        store.declarations().count(),
        2,
        "a refused write must not declare the type it failed to write"
    );
}

/// Declaration is where `INV-7`'s uniqueness is decided, and the three outcomes are distinct: the
/// same declaration again is fine, another system claiming the name is the invariant violation,
/// and another Rust type under the same name is the silent-data-loss case.
#[test]
fn a_component_type_has_exactly_one_declaration() {
    let (_access, first, second) = two_systems();
    let mut store = ComponentStore::new();
    store
        .declare::<Measured, _>(&first)
        .expect("the owner declares its type");

    store
        .declare::<Measured, _>(&first)
        .expect("re-declaring the same type is not a conflict");
    assert_eq!(store.declarations().count(), 1);

    let claimed = store
        .declare::<MeasuredByTheWrongSystem, _>(&second)
        .expect_err("two systems cannot own one component type");
    assert!(matches!(
        claimed,
        KernelError::ComponentTypeClaimedByAnotherSystem {
            ref declared_by,
            ref claimed_by,
            ..
        } if declared_by.as_str() == "first-stub" && claimed_by.as_str() == "second-stub"
    ));

    let renamed = store
        .declare::<MeasuredV2, _>(&first)
        .expect_err("one component type name cannot cover two Rust types");
    assert!(matches!(
        renamed,
        KernelError::ComponentTypeAlreadyDeclared { ref component_type }
            if component_type.as_str() == "measured"
    ));

    // The refusals left the original table in place and writable.
    store
        .insert(&first, entity(1), Measured { amount: 1 })
        .expect("the first declaration still owns the type");
    assert_eq!(store.count::<Measured>(), 1);
}

/// The one dishonest declaration the compiler cannot catch: a pack implements `OwnedBy<Other>` for
/// its own component type while the component's declaration names a different owner. Refused at
/// declaration, so it never reaches a write.
#[test]
fn an_ownership_claim_that_contradicts_the_declaration_is_refused() {
    let (_access, _first, second) = two_systems();
    let mut store = ComponentStore::new();

    let refused = store
        .declare::<MisownedState, _>(&second)
        .expect_err("a token cannot declare a type that names another owner");

    assert!(matches!(
        refused,
        KernelError::ComponentOwnerDisagreesWithDeclaration {
            ref declared_owner,
            ref writing_system,
            ..
        } if declared_owner.as_str() == "first-stub" && writing_system.as_str() == "second-stub"
    ));
    assert_eq!(store.declarations().count(), 0);
}

/// The property that makes a system optional: a world without it has no table for its components,
/// and a reader sees absence rather than an error. A system that read its way to a panic whenever
/// another pack was not installed would make packs mandatory in practice.
#[test]
fn a_component_type_this_world_never_declared_reads_as_absent() {
    let (store, _first, _second) = store_with_both_types();

    assert!(!store.is_declared(&ComponentTypeId::from_static("unused")));
    assert_eq!(store.get::<Unused>(entity(1)), None);
    assert_eq!(store.count::<Unused>(), 0);
    assert_eq!(store.iter::<Unused>().count(), 0);
}

/// Iteration follows identity, never the order the rows were written in (`KD-5`). The rows here
/// are written in descending order, so a store that iterated in write order would produce the
/// reverse.
#[test]
fn iteration_is_in_identity_order_rather_than_write_order() {
    let (mut store, first, _second) = store_with_both_types();
    for raw in [9, 4, 7, 1] {
        store
            .insert(&first, entity(raw), Measured { amount: raw as u32 })
            .expect("the owner may write");
    }

    let order: Vec<u64> = store
        .iter::<Measured>()
        .map(|(entity, _)| entity.raw())
        .collect();
    assert_eq!(order, vec![1, 4, 7, 9]);
}

/// Every component in the store reaches persistence and comes back unchanged.
///
/// The encoding is the test's choice, not the kernel's: the store holds typed values, and
/// `ComponentRecord` — the contract layer's one documented payload-erasure boundary — is what
/// carries them out of it. The round trip goes through JSON text so that nothing survives by
/// staying in memory.
#[test]
fn the_whole_store_round_trips_through_serde() {
    let (mut store, first, second) = store_with_both_types();
    store
        .insert(&first, entity(2), Measured { amount: 20 })
        .expect("the owner may write");
    store
        .insert(&first, entity(1), Measured { amount: 10 })
        .expect("the owner may write");
    store
        .insert(&second, entity(2), Flagged { raised: true })
        .expect("the owner may write");

    let written = serde_json::to_string(&records(&store)).expect("records serialize");
    let read: Vec<ComponentRecord<serde_json::Value>> =
        serde_json::from_str(&written).expect("records deserialize");

    let (mut restored, first, second) = store_with_both_types();
    for record in &read {
        match record.component_type().as_str() {
            "measured" => {
                let payload = record
                    .payload_for::<Measured>()
                    .expect("the record was written for this type and version");
                let component: Measured =
                    serde_json::from_value(payload.clone()).expect("the payload decodes");
                restored
                    .insert(&first, record.entity(), component)
                    .expect("the owner may write");
            }
            "flagged" => {
                let payload = record
                    .payload_for::<Flagged>()
                    .expect("the record was written for this type and version");
                let component: Flagged =
                    serde_json::from_value(payload.clone()).expect("the payload decodes");
                restored
                    .insert(&second, record.entity(), component)
                    .expect("the owner may write");
            }
            other => panic!("unexpected component type in the snapshot: {other}"),
        }
    }

    assert_eq!(
        store.iter::<Measured>().collect::<Vec<_>>(),
        restored.iter::<Measured>().collect::<Vec<_>>()
    );
    assert_eq!(
        store.iter::<Flagged>().collect::<Vec<_>>(),
        restored.iter::<Flagged>().collect::<Vec<_>>()
    );
    assert_eq!(records(&store), records(&restored));
}

/// A record is labelled with the type and schema version it was written from, so a record handed
/// to the wrong type is refused rather than decoded into something plausible.
#[test]
fn a_record_cannot_be_read_back_as_the_other_systems_component() {
    let (mut store, first, _second) = store_with_both_types();
    store
        .insert(&first, entity(1), Measured { amount: 5 })
        .expect("the owner may write");

    let record = records(&store)
        .into_iter()
        .next()
        .expect("the store holds one row");

    assert!(record.payload_for::<Flagged>().is_err());
}

/// A system's write token exists once in a world. The second request is refused, so two holders
/// cannot both believe they are the writer of that system's state.
#[test]
fn a_system_is_granted_its_write_token_exactly_once() {
    let mut access = WriteAccess::new();
    let first = access.grant(&FirstStub).expect("the first grant succeeds");
    assert_eq!(first.system(), SystemId::from_static("first-stub"));

    let refused = access
        .grant(&FirstStub)
        .expect_err("a second token for one system is refused");
    assert!(matches!(
        refused,
        KernelError::WriteAccessAlreadyGranted { ref system } if system.as_str() == "first-stub"
    ));

    access
        .grant(&SecondStub)
        .expect("another system is unaffected");
    assert_eq!(
        access.granted().map(SystemId::as_str).collect::<Vec<_>>(),
        vec!["first-stub", "second-stub"]
    );
}

/// Every row in the store, as the labelled records persistence would carry.
fn records(store: &ComponentStore) -> Vec<ComponentRecord<serde_json::Value>> {
    let measured = store.iter::<Measured>().map(|(entity, component)| {
        ComponentRecord::new::<Measured>(
            entity,
            serde_json::to_value(component).expect("a component serializes"),
        )
    });
    let flagged = store.iter::<Flagged>().map(|(entity, component)| {
        ComponentRecord::new::<Flagged>(
            entity,
            serde_json::to_value(component).expect("a component serializes"),
        )
    });
    measured.chain(flagged).collect()
}
