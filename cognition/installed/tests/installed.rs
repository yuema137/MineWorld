//! The installed models are consistent: the model crates this crate depends on are exactly the crates its
//! list names, and every id is spelled by the rule, listed once, and not the reserved `none`
//! (`DECISIONS.md` `ARC-33` note of 2026-10-13).
//!
//! A crate in the manifest but missing from the list would be linked and never selectable; a type in the
//! list but not in the manifest does not compile, so that half is the compiler's. Two models with one id
//! would make `Catalog::find` pick whichever is listed first, silently.

use std::collections::BTreeSet;

use mineworld_decision::ModelId;
use mineworld_installed_models::{AVAILABLE, Catalog, Model};

/// The crate names this crate's `[dependencies]` table lists, as Rust paths, read line by line from its
/// own manifest. A line that is neither blank, a comment, nor `name = { … }` fails the test.
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

/// The crate each listed in-process model's type comes from.
fn listed_crates() -> BTreeSet<String> {
    AVAILABLE
        .iter()
        .filter_map(|model| model.type_name())
        .map(|path| {
            path.split("::")
                .next()
                .unwrap_or_else(|| panic!("a type path with no crate: {path}"))
                .to_owned()
        })
        .collect()
}

/// Every id more than one listed model answers to.
fn listed_twice(ids: &[&str]) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    ids.iter()
        .filter(|id| !seen.insert(**id))
        .map(|id| (*id).to_owned())
        .collect()
}

#[test]
fn every_model_crate_depended_on_is_listed_and_every_listed_model_is_depended_on() {
    let mut depended_on = manifest_dependencies();
    assert!(
        depended_on.remove("mineworld_decision"),
        "the installed models are written with mineworld-decision's macro"
    );
    let listed = listed_crates();
    let unlisted: Vec<&String> = depended_on.difference(&listed).collect();
    assert!(
        unlisted.is_empty(),
        "linked into the build but missing from the models! list, so never selectable: {unlisted:?}"
    );
    let undeclared: Vec<&String> = listed.difference(&depended_on).collect();
    assert!(
        undeclared.is_empty(),
        "listed but not a dependency of this crate: {undeclared:?}"
    );
}

#[test]
fn every_id_is_spelled_by_the_rule_listed_once_and_not_none() {
    let ids: Vec<&str> = AVAILABLE.iter().map(|model| model.id()).collect();
    for id in &ids {
        ModelId::new(*id).unwrap_or_else(|error| panic!("model id '{id}': {error}"));
        assert_ne!(
            *id, "none",
            "'none' ends a fallback chain and cannot be a model"
        );
    }
    assert!(
        listed_twice(&ids).is_empty(),
        "two models answer to {:?}",
        listed_twice(&ids)
    );
    assert_eq!(
        ids.first(),
        Some(&"rule"),
        "rule is installed, first, as the default"
    );
    assert!(
        AVAILABLE
            .iter()
            .filter(|model| !model.is_external())
            .all(|model| model.package().is_some() && model.defaults().is_some()),
        "every in-process model has a package and its defaults"
    );
}

#[test]
fn the_lm_controller_is_the_one_installed_external_model() {
    let external: Vec<&str> = AVAILABLE
        .iter()
        .filter(|model| model.is_external())
        .map(|model| model.id())
        .collect();
    assert_eq!(
        external,
        ["lm"],
        "the LM controller (S10 P6b, DL-a SD-DLa-12) is the only external model listed"
    );
}

/// The negative control: the checks see a duplicate and a reserved id when there is one. Two stub
/// models listed by the same macro the real set uses.
mod twins {
    use mineworld_contracts::ActionRequest;
    use mineworld_decision::{Consult, DecisionModel};

    macro_rules! stub {
        ($name:ident, $id:literal) => {
            #[derive(Default)]
            pub struct $name;

            impl DecisionModel for $name {
                const ID: &'static str = $id;
                const PACKAGE: mineworld_decision::Package = mineworld_rule_controller::PACKAGE;
                type Parameters = ();

                fn decide(&self, (): &(), _: &Consult<'_>) -> Option<ActionRequest> {
                    None
                }

                fn step(&self, (): &(), _: &Consult<'_>) -> Option<ActionRequest> {
                    None
                }
            }
        };
    }

    stub!(Castor, "twin");
    stub!(Pollux, "twin");

    mineworld_decision::models! {
        Castor => Castor,
        Pollux => Pollux,
        external "none",
    }
}

#[test]
fn the_checks_see_a_duplicate_and_the_reserved_id() {
    let ids: Vec<&str> = twins::AVAILABLE.iter().map(|model| model.id()).collect();
    assert_eq!(listed_twice(&ids), BTreeSet::from(["twin".to_owned()]));
    assert!(ids.contains(&"none"), "the reserved id is seen when listed");
    assert_eq!(Model::find("rule"), Some(Model::Rule));
}
