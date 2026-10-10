//! Every type an extension line of the installed set lists is an installed pack, once per line
//! (`DECISIONS.md` `ARC-62` item 5, `ARC-39` item 6; step-11 SD-R8).
//!
//! The compiler already refuses a listed type that does not implement its line's trait. What it cannot
//! see is whether that type is also a pack this build installs: an implementation whose pack is not
//! installed would be registered by every host and asked in every world, while no world could ever
//! enable the pack whose state it reads. And one type listed twice would be refused only when a host
//! first registers. This guard holds both on the real set — whose lines today are presence's arrival
//! resolvers and movement's wayfinders — and a stub set that lists, twice, a type it does not install
//! is its negative control.
//!
//! The file keeps its name from when the line was spelled `resolution:` (the seam scan lists it).

use std::collections::BTreeSet;

/// What is wrong with an installed set's extension lines: types that are not installed packs, and
/// types listed more than once on one line.
fn faults(
    lines: Vec<(&'static str, Vec<&'static str>)>,
    installed: Vec<&'static str>,
) -> Vec<String> {
    let installed: BTreeSet<&str> = installed.into_iter().collect();
    let mut faults = Vec::new();
    for (_, types) in lines {
        let mut seen = BTreeSet::new();
        for listed in types {
            if !installed.contains(listed) {
                faults.push(format!(
                    "'{listed}' is listed on an extension line but is not an installed pack"
                ));
            }
            if !seen.insert(listed) {
                faults.push(format!("'{listed}' is listed twice on one extension line"));
            }
        }
    }
    faults
}

#[test]
fn every_type_on_an_extension_line_is_an_installed_pack_listed_once() {
    use mineworld_installed_systems::{AVAILABLE, Capability};

    let lines = Capability::extension_types();
    assert_eq!(
        lines.len(),
        2,
        "the real set has presence's line and movement's"
    );
    let installed = AVAILABLE.into_iter().map(Capability::type_name).collect();
    let faults = faults(lines, installed);
    assert!(faults.is_empty(), "{faults:#?}");
}

/// The negative control: a set that installs one stub pack and lists, on an extension line, a type that
/// is not among its packs — twice.
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

    impl SystemPack for Lone {
        const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    }

    /// An implementation whose pack this set does not install.
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
        extension mineworld_presence::ArrivalResolver => mineworld_presence::register_resolvers: [Stray, Stray,];
        Lone => Lone,
    }
}

#[test]
fn the_guard_sees_a_type_that_is_not_installed_and_one_listed_twice() {
    let lines = stray::Capability::extension_types();
    let installed = stray::AVAILABLE
        .into_iter()
        .map(stray::Capability::type_name)
        .collect();
    let stray = std::any::type_name::<stray::Stray>();
    assert_eq!(
        faults(lines, installed),
        [
            format!("'{stray}' is listed on an extension line but is not an installed pack"),
            format!("'{stray}' is listed on an extension line but is not an installed pack"),
            format!("'{stray}' is listed twice on one extension line"),
        ]
    );
}
