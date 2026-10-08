//! Every arrival resolver the installed set lists belongs to an installed pack, once (`DECISIONS.md`
//! `ARC-39` item 6; step-11 SD-R8).
//!
//! The compiler already refuses a listed type that is not an `ArrivalResolver`. What it cannot see is
//! whether that type is also a pack this build installs: a resolver whose pack is not installed would
//! be registered by every host and asked in every world, while no world could ever enable the pack
//! whose state it reads. And one id listed twice would be refused only when a host first registers.
//! This guard holds both on the real set, and a stub set that lists a resolver it does not install is
//! its negative control.

use std::collections::BTreeSet;

use mineworld_contracts::SystemId;

/// What is wrong with an installed set's resolvers: ids that are not installed packs, and ids listed
/// more than once.
fn faults(resolvers: Vec<SystemId>, installed: Vec<SystemId>) -> Vec<String> {
    let installed: BTreeSet<SystemId> = installed.into_iter().collect();
    let mut seen = BTreeSet::new();
    let mut faults = Vec::new();
    for id in resolvers {
        if !installed.contains(&id) {
            faults.push(format!(
                "'{id}' is listed on resolution: but is not an installed pack"
            ));
        }
        if !seen.insert(id.clone()) {
            faults.push(format!("'{id}' is listed on resolution: twice"));
        }
    }
    faults
}

#[test]
fn every_listed_resolver_is_an_installed_pack_listed_once() {
    use mineworld_installed_systems::{AVAILABLE, Capability};

    let resolvers = Capability::resolvers()
        .iter()
        .map(|resolver| resolver.resolver_of())
        .collect();
    let installed = AVAILABLE.into_iter().map(Capability::id).collect();
    let faults = faults(resolvers, installed);
    assert!(faults.is_empty(), "{faults:#?}");
}

/// The negative control: a set that installs one stub pack and lists, as a resolver, a type that is
/// not among its packs.
mod stray {
    use mineworld_contracts::SystemId;
    use mineworld_kernel::{System, SystemDeclaration, SystemIdentity, SystemVersion, WorldRead};
    use mineworld_presence::{ArrivalResolver, Arriving, PerceptionProvider, Resolution};
    use mineworld_sdk::SystemPack;

    #[derive(Default)]
    pub struct Lone;

    impl SystemIdentity for Lone {
        const ID: SystemId = SystemId::from_static("lone");
    }

    impl System for Lone {
        const VERSION: SystemVersion = SystemVersion::new(1);

        fn declaration(&self) -> SystemDeclaration {
            SystemDeclaration::of::<Self>()
        }
    }

    impl PerceptionProvider for Lone {}

    impl SystemPack for Lone {}

    /// A resolver whose pack this set does not install.
    #[derive(Default)]
    pub struct Stray;

    impl ArrivalResolver for Stray {
        fn resolver_of(&self) -> SystemId {
            SystemId::from_static("stray")
        }

        fn resolve(&self, _: &WorldRead<'_>, _: &Arriving, so_far: Resolution) -> Resolution {
            so_far
        }
    }

    mineworld_sdk::installed! {
        perception: mineworld_presence::PerceptionProvider;
        resolution: mineworld_presence::ArrivalResolver => [Stray, Stray,];
        Lone => Lone,
    }
}

#[test]
fn the_guard_sees_a_resolver_that_is_not_installed_and_one_listed_twice() {
    let resolvers = stray::Capability::resolvers()
        .iter()
        .map(|resolver| resolver.resolver_of())
        .collect();
    let installed = stray::AVAILABLE
        .into_iter()
        .map(stray::Capability::id)
        .collect();
    assert_eq!(
        faults(resolvers, installed),
        [
            "'stray' is listed on resolution: but is not an installed pack",
            "'stray' is listed on resolution: but is not an installed pack",
            "'stray' is listed on resolution: twice",
        ]
    );
}
