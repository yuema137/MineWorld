//! Composing a world out of systems, from outside the kernel: what the registry admits, what it
//! refuses by name, and what enabling and disabling change.
//!
//! An external crate, so everything reachable here is everything a **World Pack** can reach when it
//! assembles a world. The claims are the ones §4.4's C3 lists, and every one of them is decided from
//! declarations alone, before any world runs:
//!
//! ```text
//! a declared dependency that is absent is refused, naming it
//! a declared dependency that is disabled is refused, naming it
//! two systems claiming one component type are refused             — INV-7 at world level
//! two systems providing one action type are refused               — INV-10 must be decidable
//! registration order is the order systems were installed          — AC-12
//! disabling removes a system's actions from the route map         — AC-2
//! ```
//!
//! The systems are called `Places`, `Weather` and `Rival` because they have to be called something.
//! None of them knows anything: one owns a count, one owns a flag, and the kernel never learns what
//! either means (`INV-12`).

use mineworld_contracts::{Action, ActionTypeId, Component, SystemId};
use mineworld_kernel::{
    Declarations, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion, World,
    owned_component,
};
use serde::{Deserialize, Serialize};

/// The system every other system in this file is defined against.
struct Places;

impl SystemIdentity for Places {
    const ID: SystemId = SystemId::from_static("places");
}

#[derive(Debug, Serialize, Deserialize)]
struct Occupancy {
    present: u32,
}

owned_component! {
    component = Occupancy,
    owner = Places,
    component_type = "occupancy",
    schema_version = 1,
}

/// The request `Places` answers.
#[derive(Serialize, Deserialize)]
struct Enter {
    party: u32,
}

impl Action for Enter {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("enter");
    const OWNER: SystemId = Places::ID;
}

impl System for Places {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<Occupancy>()
            .providing::<Enter>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Occupancy>()
    }
}

/// A system that depends on `Places` and owns state of its own.
struct Weather;

impl SystemIdentity for Weather {
    const ID: SystemId = SystemId::from_static("weather");
}

#[derive(Debug, Serialize, Deserialize)]
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
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([Places::ID])
            .owning::<Overcast>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Overcast>()
    }
}

/// A second pack that happened to call its own state `"occupancy"` as well. Honest about owning it,
/// and that is the conflict: one piece of state, two writers.
struct Rival;

impl SystemIdentity for Rival {
    const ID: SystemId = SystemId::from_static("rival");
}

#[derive(Debug, Serialize, Deserialize)]
struct RivalOccupancy {
    heads: u32,
}

owned_component! {
    component = RivalOccupancy,
    owner = Rival,
    component_type = "occupancy",
    schema_version = 1,
}

impl System for Rival {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>().owning::<RivalOccupancy>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<RivalOccupancy>()
    }
}

/// A system whose name sorts *before* the two above and which is installed *after* them, so that
/// registration order and name order cannot be mistaken for each other.
struct Almanac;

impl SystemIdentity for Almanac {
    const ID: SystemId = SystemId::from_static("almanac");
}

impl System for Almanac {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
    }
}

/// A second pack that answers for `"enter"` as well.
struct Doorman;

impl SystemIdentity for Doorman {
    const ID: SystemId = SystemId::from_static("doorman");
}

impl System for Doorman {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>().providing::<Enter>()
    }
}

/// A system whose declaration lists a component type belonging to somebody else.
struct Poacher;

impl SystemIdentity for Poacher {
    const ID: SystemId = SystemId::from_static("poacher");
}

impl System for Poacher {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>().owning::<Occupancy>()
    }
}

/// A system that needs one this world will never have.
struct Orphan;

impl SystemIdentity for Orphan {
    const ID: SystemId = SystemId::from_static("orphan");
}

impl System for Orphan {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>().depending_on([SystemId::from_static("absent")])
    }
}

/// A declared dependency that is not installed is refused, and the refusal names it. The name is the
/// whole of the diagnosis: a world pack listed one system and not the one it needs.
#[test]
fn a_missing_dependency_is_refused_by_name() {
    let mut world = World::new();

    let refusal = world
        .install(Orphan)
        .expect_err("a system whose dependency is absent cannot be installed");

    assert_eq!(
        refusal,
        KernelError::SystemDependencyMissing {
            system: Orphan::ID,
            dependency: SystemId::from_static("absent"),
        }
    );
    // Refused from the declaration alone, so nothing was granted and nothing was declared.
    assert!(world.systems().is_empty());
    assert_eq!(world.writers().count(), 0);
}

/// A declared dependency that is installed but disabled is refused too, and differently: the world
/// has the system, and it is not acting.
#[test]
fn a_disabled_dependency_is_refused_as_disabled() {
    let mut world = World::new();
    world.install(Places).expect("a system installs");
    world.disable(&Places::ID).expect("a system is disabled");

    let refusal = world
        .install(Weather)
        .expect_err("a system whose dependency is disabled cannot be installed");

    assert_eq!(
        refusal,
        KernelError::SystemDependencyDisabled {
            system: Weather::ID,
            dependency: Places::ID,
        }
    );
    assert_eq!(world.systems().len(), 1);
}

/// Two systems claiming one component type are refused: two writers for one piece of state is
/// exactly what `INV-7` forbids, and it is decidable from the two declarations.
#[test]
fn two_systems_cannot_claim_one_component_type() {
    let mut world = World::new();
    world.install(Places).expect("a system installs");

    let refusal = world
        .install(Rival)
        .expect_err("a second claim on one component type is refused");

    assert_eq!(
        refusal,
        KernelError::ComponentTypeClaimedByAnotherSystem {
            component_type: Occupancy::COMPONENT_TYPE,
            declared_by: Places::ID,
            claimed_by: Rival::ID,
        }
    );
    // The first system's table is untouched, and the second holds no token.
    assert!(world.components().is_declared(&Occupancy::COMPONENT_TYPE));
    assert_eq!(world.writers().count(), 1);
}

/// A declaration that lists a component type owned by another system is refused before the conflict
/// check is even reached. Ownership is a fact about the component type, so listing somebody else's
/// state is not a claim the registry weighs — it is a claim it rejects.
#[test]
fn a_declaration_cannot_list_another_systems_component_type() {
    let mut world = World::new();
    world.install(Places).expect("a system installs");

    let refusal = world
        .install(Poacher)
        .expect_err("claiming another system's component type is refused");

    assert_eq!(
        refusal,
        KernelError::ComponentOwnerDisagreesWithDeclaration {
            component_type: Occupancy::COMPONENT_TYPE,
            declared_owner: Places::ID,
            writing_system: Poacher::ID,
        }
    );
    assert!(!world.systems().is_installed(&Poacher::ID));
}

/// Two systems providing one action type are refused. Routing has to be a map from action type to
/// exactly one system, or `INV-10`'s answer would depend on which of two systems was asked.
#[test]
fn two_systems_cannot_provide_one_action_type() {
    let mut world = World::new();
    world.install(Places).expect("a system installs");

    let refusal = world
        .install(Doorman)
        .expect_err("a second provider of one action type is refused");

    assert_eq!(
        refusal,
        KernelError::ActionTypeProvidedByAnotherSystem {
            action_type: Enter::ACTION_TYPE,
            provided_by: Places::ID,
            claimed_by: Doorman::ID,
        }
    );
    assert_eq!(
        world.systems().provider(&Enter::ACTION_TYPE),
        Some(&Places::ID)
    );
}

/// Registration order is the order systems were installed, and it is not the order their names sort
/// in. Reduction runs in this order and it is observable in the event log, so it has to be stated
/// rather than inherited from a container (`AC-12`, `BD-4`).
///
/// `almanac` is installed last and sorts first, which is what makes this test able to tell the two
/// orders apart: a registry that reported the iteration order of its name-keyed map would put
/// `almanac` at the front of every one of these three answers.
#[test]
fn registration_order_is_the_order_systems_were_installed() {
    let mut world = World::new();
    world.install(Places).expect("a system installs");
    world.install(Weather).expect("a second system installs");
    world.install(Almanac).expect("a third system installs");

    assert_eq!(
        world.systems().order(),
        [Places::ID, Weather::ID, Almanac::ID]
    );
    assert_eq!(
        world.systems().enabled().collect::<Vec<_>>(),
        [&Places::ID, &Weather::ID, &Almanac::ID]
    );
    let declared: Vec<&SystemId> = world
        .systems()
        .declarations()
        .map(SystemDeclaration::system)
        .collect();
    assert_eq!(declared, [&Places::ID, &Weather::ID, &Almanac::ID]);
}

/// Disabling a system removes its actions from the route map, and enabling it puts them back. The
/// system stays installed and the state it owns stays in the world, because that state is still its
/// state.
#[test]
fn disabling_a_system_removes_its_actions_from_the_route_map() {
    let mut world = World::new();
    world.install(Places).expect("a system installs");
    assert_eq!(
        world.systems().routes().collect::<Vec<_>>(),
        [(&Enter::ACTION_TYPE, &Places::ID)]
    );

    world.disable(&Places::ID).expect("a system is disabled");

    assert_eq!(world.systems().provider(&Enter::ACTION_TYPE), None);
    assert_eq!(world.systems().routes().count(), 0);
    assert_eq!(world.systems().enabled().count(), 0);
    // Installed, not removed: its table is still there and it still holds its write token.
    assert!(world.systems().is_installed(&Places::ID));
    assert!(!world.systems().is_enabled(&Places::ID));
    assert!(world.components().is_declared(&Occupancy::COMPONENT_TYPE));
    assert_eq!(world.writers().count(), 1);

    world.enable(&Places::ID).expect("a system is enabled");

    assert_eq!(
        world.systems().provider(&Enter::ACTION_TYPE),
        Some(&Places::ID)
    );
    assert!(world.systems().is_enabled(&Places::ID));
}

/// A system cannot be disabled out from under a system that depends on it. A world in which an
/// enabled system's declared dependency is disabled is one the registry would have refused to
/// assemble; reaching it by disabling would be the same mistake arrived at backwards.
#[test]
fn a_system_cannot_be_disabled_while_an_enabled_system_depends_on_it() {
    let mut world = World::new();
    world.install(Places).expect("a system installs");
    world.install(Weather).expect("a second system installs");

    let refusal = world
        .disable(&Places::ID)
        .expect_err("a depended-on system cannot be disabled");

    assert_eq!(
        refusal,
        KernelError::SystemRequiredByAnotherSystem {
            system: Places::ID,
            required_by: Weather::ID,
        }
    );
    assert!(world.systems().is_enabled(&Places::ID));

    // Disabling the dependent first is enough: nothing then depends on it.
    world.disable(&Weather::ID).expect("a system is disabled");
    world.disable(&Places::ID).expect("a system is disabled");
    assert_eq!(world.systems().enabled().count(), 0);

    // And enabling the dependent again is refused while its dependency is off, by the same rule
    // read the other way round.
    let refusal = world
        .enable(&Weather::ID)
        .expect_err("a system whose dependency is disabled cannot be enabled");
    assert_eq!(
        refusal,
        KernelError::SystemDependencyDisabled {
            system: Weather::ID,
            dependency: Places::ID,
        }
    );
}

/// Enabling or disabling a name this world never installed is a mistake in the configuration, not a
/// system that happens to be off.
#[test]
fn a_system_this_world_never_installed_cannot_be_enabled_or_disabled() {
    let mut world = World::new();
    let absent = SystemId::from_static("absent");

    assert_eq!(
        world.enable(&absent).expect_err("an unknown system"),
        KernelError::SystemNotInstalled {
            system: absent.clone()
        }
    );
    assert_eq!(
        world.disable(&absent).expect_err("an unknown system"),
        KernelError::SystemNotInstalled { system: absent }
    );
}
