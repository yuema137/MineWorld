//! World assembly, from outside the kernel: what installing a system does, and what it refuses.
//!
//! This file is where a **pack author's** view of the kernel is tested. It is an external crate, so
//! everything it can reach is everything a System Pack can reach — and the first thing to notice is
//! what it cannot reach: there is no `WriteAccess` here, no `WriteToken`, and no way to obtain
//! either, because installing a system into a [`World`] is the only act that grants write
//! capability (`BD-1`). The compile-fail suite pins that as a compiler refusal; this file shows what
//! remains possible once it is closed.
//!
//! The systems are named `Places` and `Weather` because they have to be called something. Neither
//! knows anything: one owns a count, the other owns a flag, and the kernel never learns what either
//! means (`INV-12`).

use mineworld_contracts::{Component, EntityKey, EntityType, LifecycleState, SystemId, Tag, Tags};
use mineworld_kernel::{
    Declarations, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion, World,
    owned_component,
};
use serde::{Deserialize, Serialize};

/// A system that owns one component type.
struct Places;

impl SystemIdentity for Places {
    const ID: SystemId = SystemId::from_static("places");
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Occupancy {
    present: u32,
}

owned_component! {
    component = Occupancy,
    owner = Places,
    component_type = "occupancy",
    schema_version = 1,
}

impl System for Places {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>().owning::<Occupancy>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Occupancy>()
    }
}

/// A second system, so that every question can be asked about "the other one".
struct Weather;

impl SystemIdentity for Weather {
    const ID: SystemId = SystemId::from_static("weather");
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Overcast {
    raining: bool,
}

owned_component! {
    component = Overcast,
    owner = Weather,
    component_type = "overcast",
    schema_version = 1,
}

impl System for Weather {
    const VERSION: SystemVersion = SystemVersion::new(4);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>().owning::<Overcast>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Overcast>()
    }
}

/// A system that claims a component type in its declaration and never declares the table.
struct Forgetful;

impl SystemIdentity for Forgetful {
    const ID: SystemId = SystemId::from_static("forgetful");
}

#[derive(Debug, Serialize, Deserialize)]
struct Unbuilt {
    value: u32,
}

owned_component! {
    component = Unbuilt,
    owner = Forgetful,
    component_type = "unbuilt",
    schema_version = 1,
}

impl System for Forgetful {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>().owning::<Unbuilt>()
    }
}

/// A system whose `declaration()` hands back another system's declaration.
struct Impostor;

impl SystemIdentity for Impostor {
    const ID: SystemId = SystemId::from_static("impostor");
}

impl System for Impostor {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Places>().owning::<Occupancy>()
    }
}

fn key(value: &str) -> EntityKey {
    EntityKey::new(value).expect("a legal authoring name")
}

/// A world is what its systems declare it to be: their tables exist, their declarations are held in
/// the order they were installed, and each of them — and only each of them — holds a write token.
#[test]
fn a_world_is_assembled_from_the_systems_installed_into_it() {
    let mut world = World::new();
    world.install(Weather).expect("a system installs");
    world.install(Places).expect("a second system installs");

    // Each system's own table exists, declared by the system that owns it and by nothing else.
    assert!(world.components().is_declared(&Occupancy::COMPONENT_TYPE));
    assert!(world.components().is_declared(&Overcast::COMPONENT_TYPE));

    // Registration order, not name order: "weather" was installed first and sorts second.
    assert_eq!(world.systems().order(), [Weather::ID, Places::ID]);
    let declared: Vec<(&SystemId, SystemVersion)> = world
        .systems()
        .declarations()
        .map(|declaration| (declaration.system(), declaration.version()))
        .collect();
    assert_eq!(
        declared,
        [
            (&Weather::ID, SystemVersion::new(4)),
            (&Places::ID, SystemVersion::new(1)),
        ]
    );

    // Write capability: exactly the installed systems hold a token, because installation is the
    // only thing that grants one and nothing outside the kernel can issue another.
    let writers: Vec<&SystemId> = world.writers().collect();
    assert_eq!(writers, [&Places::ID, &Weather::ID]);
    assert_eq!(world.systems().len(), 2);
}

/// A system is installed once. Refused rather than replaced: the installed system owns state, and a
/// second system under one name would either inherit state it never wrote or orphan it.
#[test]
fn a_system_is_installed_once() {
    let mut world = World::new();
    world.install(Places).expect("a system installs");

    let refusal = world
        .install(Places)
        .expect_err("a second installation is refused");

    assert_eq!(
        refusal,
        KernelError::SystemAlreadyInstalled { system: Places::ID }
    );
    assert_eq!(world.systems().len(), 1);
    assert_eq!(world.writers().count(), 1);
}

/// `declaration()` can hand back a declaration belonging to another system, so installation compares
/// the two names. Without that check, a system could claim another's component types and the
/// registry's conflict detection would be reasoning about the wrong owner.
#[test]
fn a_system_cannot_install_under_another_systems_name() {
    let mut world = World::new();

    let refusal = world
        .install(Impostor)
        .expect_err("a borrowed declaration is refused");

    assert_eq!(
        refusal,
        KernelError::SystemDeclarationNamesAnotherSystem {
            system: Impostor::ID,
            declared: Places::ID,
        }
    );
    // And nothing happened: no system, no token, no table.
    assert!(world.systems().is_empty());
    assert_eq!(world.writers().count(), 0);
    assert!(!world.components().is_declared(&Occupancy::COMPONENT_TYPE));
}

/// A declaration that claims a component type the system never declares a table for is refused. The
/// two halves would otherwise disagree: the registry would defend state that does not exist, and a
/// write to it would be refused as undeclared.
#[test]
fn a_system_that_claims_a_component_type_must_declare_its_table() {
    let mut world = World::new();

    let refusal = world
        .install(Forgetful)
        .expect_err("a claim with no table is refused");

    assert_eq!(
        refusal,
        KernelError::SystemDidNotDeclareOwnedComponent {
            system: Forgetful::ID,
            component_type: Unbuilt::COMPONENT_TYPE,
        }
    );
    assert!(world.systems().is_empty());
}

/// Entities are the world's, not a system's: creating one is ungated, because an entity is the thing
/// systems attach state to rather than state itself. What the world guarantees is identity —
/// monotonic, never reused, and spent for good once the entity is destroyed.
#[test]
fn the_world_allocates_identity_and_never_reuses_it() {
    let mut world = World::new();
    world.install(Places).expect("a system installs");

    let alice = world
        .create_authored_entity(
            key("alice"),
            EntityType::Person,
            Tags::new([Tag::new("resident").expect("a legal tag")]),
            None,
        )
        .expect("an authored entity is created");
    let cafe = world
        .create_entity(key("cafe"), EntityType::Place)
        .expect("an entity is created");
    assert_eq!(alice.raw(), 1);
    assert_eq!(cafe.raw(), 2);

    let cleared = world.destroy_entity(cafe).expect("an entity is destroyed");
    assert!(cleared.is_empty(), "no edges existed to clear");

    // The record stays, its identity stays spent, and the next entity gets a fresh one.
    assert_eq!(
        world
            .entities()
            .require(cafe)
            .expect("the record remains")
            .lifecycle(),
        LifecycleState::Destroyed
    );
    let later = world
        .create_entity(key("market"), EntityType::Place)
        .expect("an entity is created");
    assert_eq!(later.raw(), 3);

    // Destruction is terminal, so a second destruction is refused by the lifecycle itself.
    world
        .destroy_entity(cafe)
        .expect_err("a destroyed entity cannot be destroyed again");
    assert_eq!(world.entities().len(), 3);
}
