//! The installed set is consistent: the packs this crate depends on are exactly the packs its list
//! names, and no two listed packs share an id (`DECISIONS.md` `ARC-33`).
//!
//! A pack in the manifest but missing from the list would be linked and never installable; a pack in
//! the list but not in the manifest does not compile, so that half is the compiler's. Two packs with
//! one id would make `Capability::resolve` pick whichever is listed first, silently.

use std::collections::BTreeSet;

use mineworld_contracts::SystemId;
use mineworld_installed_systems::{AVAILABLE, Capability};

/// The crate names this crate's `[dependencies]` table lists, as Rust paths (`mineworld_presence`),
/// read line by line from its own manifest. A line that is neither blank, a comment, nor
/// `name = { … }` fails the test rather than being skipped: a guard that could not read the list
/// must not pass.
fn manifest_dependencies() -> BTreeSet<String> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
    let manifest = std::fs::read_to_string(path).expect("the crate's own manifest");
    let mut in_dependencies = false;
    let mut names = BTreeSet::new();
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            in_dependencies = line == "[dependencies]";
            continue;
        }
        if !in_dependencies || line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, value) = line
            .split_once('=')
            .unwrap_or_else(|| panic!("an unreadable [dependencies] line: {line}"));
        assert!(
            value.trim().starts_with('{'),
            "an unreadable [dependencies] line: {line}"
        );
        names.insert(name.trim().replace('-', "_"));
    }
    assert!(!names.is_empty(), "no [dependencies] were read from {path}");
    names
}

/// The crate each listed pack's system type comes from.
fn listed_crates() -> BTreeSet<String> {
    AVAILABLE
        .into_iter()
        .map(|capability| {
            let path = capability.type_name();
            path.split("::")
                .next()
                .unwrap_or_else(|| panic!("a type path with no crate: {path}"))
                .to_owned()
        })
        .collect()
}

/// Every id that more than one capability in `ids` answers to.
fn listed_twice(ids: Vec<SystemId>) -> BTreeSet<SystemId> {
    let mut seen = BTreeSet::new();
    ids.into_iter()
        .filter(|id| !seen.insert(id.clone()))
        .collect()
}

#[test]
fn every_pack_depended_on_is_listed_and_every_listed_pack_is_depended_on() {
    let mut depended_on = manifest_dependencies();
    assert!(
        depended_on.remove("mineworld_sdk"),
        "the installed set is written with the SDK's macro"
    );
    let listed = listed_crates();

    let unlisted: Vec<&String> = depended_on.difference(&listed).collect();
    assert!(
        unlisted.is_empty(),
        "linked into the build but missing from the installed! list, so never installable: \
         {unlisted:?}"
    );
    let undeclared: Vec<&String> = listed.difference(&depended_on).collect();
    assert!(
        undeclared.is_empty(),
        "listed but not a dependency of this crate: {undeclared:?}"
    );
}

#[test]
fn no_two_installed_packs_share_an_id() {
    let ids: Vec<SystemId> = AVAILABLE.into_iter().map(Capability::id).collect();
    let twice = listed_twice(ids);
    assert!(twice.is_empty(), "two installed packs answer to {twice:?}");
}

/// The negative control: the id check sees a duplicate when there is one. Two stub packs, listed by
/// the same macro the real set uses, under one id.
mod twins {
    use mineworld_contracts::SystemId;
    use mineworld_kernel::{System, SystemDeclaration, SystemIdentity, SystemVersion};
    use mineworld_presence::PerceptionProvider;
    use mineworld_sdk::SystemPack;

    macro_rules! stub {
        ($name:ident) => {
            #[derive(Default)]
            pub struct $name;

            impl SystemIdentity for $name {
                const ID: SystemId = SystemId::from_static("twin");
            }

            impl System for $name {
                const VERSION: SystemVersion = SystemVersion::new(1);

                fn declaration(&self) -> SystemDeclaration {
                    SystemDeclaration::of::<Self>()
                }
            }

            impl PerceptionProvider for $name {}

            impl SystemPack for $name {}
        };
    }

    stub!(Castor);
    stub!(Pollux);

    mineworld_sdk::installed! {
        perception: mineworld_presence::PerceptionProvider;
        Castor => Castor,
        Pollux => Pollux,
    }
}

#[test]
fn the_id_check_sees_two_packs_that_share_an_id() {
    let ids: Vec<SystemId> = twins::AVAILABLE
        .into_iter()
        .map(twins::Capability::id)
        .collect();
    assert_eq!(
        listed_twice(ids),
        BTreeSet::from([SystemId::from_static("twin")])
    );
}
