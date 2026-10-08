//! A world's requirements, resolved by the pure function every host uses (`DECISIONS.md` `ARC-54`
//! point 4; step-16 §15.4 EB-2, EB-4). Every rule, its refusal by name, and the order the rules run in.

use std::collections::BTreeMap;
use std::path::PathBuf;

use mineworld_packages::{
    CodePack, Compatibility, FoundPack, Identity, Installed, LicencePolicy, License, PackId,
    PackType, PackageError, Source, Version, WorldRequirements, resolve,
};

fn identity(id: &str, kind: PackType, version: &str, license: &str) -> Identity {
    Identity {
        id: PackId::new(id).expect("an id"),
        kind,
        version: Version::new(version).expect("semver"),
        license: License::new(license).expect("spdx"),
        authors: vec!["Someone".to_owned()],
        repository: None,
        mineworld: None,
    }
}

fn code(id: &str, system: &str, bundled: bool, license: &str) -> CodePack {
    CodePack {
        identity: identity(id, PackType::SystemPack, "0.1.0", license),
        bundled,
        system: system.to_owned(),
    }
}

fn found(id: &str, kind: PackType, version: &str, license: &str) -> FoundPack {
    FoundPack {
        identity: identity(id, kind, version, license),
        dir: PathBuf::from(format!("/roots/{id}")),
    }
}

fn requires(entries: &[(&str, &str)]) -> BTreeMap<PackId, Compatibility> {
    entries
        .iter()
        .map(|(id, range)| {
            (
                PackId::new(id).expect("an id"),
                Compatibility::new(range).expect("a range"),
            )
        })
        .collect()
}

/// The build: two bundled packs and one third-party pack, as E-c will install one.
fn build() -> Vec<CodePack> {
    vec![
        code("mineworld-presence", "presence", true, "MIT"),
        code("mineworld-movement", "movement", true, "MIT"),
        code("acme-fishing", "fishing", false, "MIT"),
    ]
}

/// The pack roots: a presentation pack, a world, an entity pack.
fn roots() -> Vec<FoundPack> {
    vec![
        found("style-a", PackType::PresentationPack, "0.1.0", "CC0-1.0"),
        found("other-world", PackType::WorldPack, "0.1.0", "MIT"),
        found("goods", PackType::EntityPack, "0.1.0", "MIT"),
    ]
}

fn run(
    entries: &[(&str, &str)],
    enabled: &[&str],
    license: Option<&str>,
    build: &[CodePack],
    found: &[FoundPack],
) -> Result<mineworld_packages::Composition, PackageError> {
    let requires = requires(entries);
    let license = license.map(|text| License::new(text).expect("spdx"));
    resolve(
        &WorldRequirements {
            id: "the-world",
            license: license.as_ref(),
            mineworld: None,
            requires: &requires,
        },
        enabled,
        &Installed {
            build,
            found,
            searched: "this build or /roots",
        },
        &LicencePolicy::default(),
    )
}

#[test]
fn a_world_whose_requirements_are_met_resolves_to_its_composition() {
    let composition = run(
        &[("style-a", "^0.1"), ("acme-fishing", "^0.1")],
        &["presence", "fishing"],
        Some("MIT"),
        &build(),
        &roots(),
    )
    .expect("resolves");
    let required: Vec<(&str, &Source)> = composition
        .required
        .iter()
        .map(|r| (r.identity.id.as_str(), &r.source))
        .collect();
    assert_eq!(
        required,
        [
            ("acme-fishing", &Source::Build { bundled: false }),
            (
                "style-a",
                &Source::Directory(PathBuf::from("/roots/style-a"))
            ),
        ],
        "in id order, each from where it was found"
    );
    let systems: Vec<(&str, bool)> = composition
        .systems
        .iter()
        .map(|s| (s.system.as_str(), s.bundled))
        .collect();
    assert_eq!(
        systems,
        [("presence", true), ("fishing", false)],
        "in the world's order"
    );

    let bare = run(&[], &["presence", "movement"], None, &build(), &[]).expect("nothing required");
    assert!(bare.required.is_empty());
}

/// One refusal case: its name, `requires:`, the enabled systems, the world's licence, the packs found,
/// and what the refusal must say.
type Case = (
    &'static str,
    Vec<(&'static str, &'static str)>,
    Vec<&'static str>,
    Option<&'static str>,
    Vec<FoundPack>,
    &'static [&'static str],
);

/// Every refusal, by name: the input that breaks one rule, and what the refusal must say.
#[test]
fn every_failed_rule_is_refused_by_name() {
    let cases: [Case; 10] = [
        (
            "absent",
            vec![("nowhere", "^0.1")],
            vec!["presence"],
            None,
            roots(),
            &["requires: nowhere", "this build or /roots"],
        ),
        (
            "out of range",
            vec![("style-a", "^0.2")],
            vec!["presence"],
            None,
            roots(),
            &["style-a \"^0.2\"", "found is 0.1.0"],
        ),
        (
            "a world",
            vec![("other-world", "^0.1")],
            vec!["presence"],
            None,
            roots(),
            &["other-world is a world-pack"],
        ),
        (
            "an entity pack, before E-d",
            vec![("goods", "^0.1")],
            vec!["presence"],
            None,
            roots(),
            &["goods is a entity-pack", "E-d"],
        ),
        (
            "bundled",
            vec![("mineworld-presence", "^0.1")],
            vec!["presence"],
            None,
            roots(),
            &["mineworld-presence is bundled", "remove it from requires"],
        ),
        (
            "third-party not required",
            vec![],
            vec!["presence", "fishing"],
            None,
            roots(),
            &["enables fishing", "acme-fishing is third-party"],
        ),
        (
            "the world's licence",
            vec![],
            vec!["presence"],
            Some("GPL-3.0-only"),
            roots(),
            &[
                "the-world's licence \"GPL-3.0-only\"",
                "GPL-3.0-only",
                "MIT",
            ],
        ),
        (
            "a required pack's licence",
            vec![("copyleft", "^0.1")],
            vec!["presence"],
            None,
            vec![found(
                "copyleft",
                PackType::PresentationPack,
                "0.1.0",
                "CC-BY-SA-4.0",
            )],
            &["copyleft's licence \"CC-BY-SA-4.0\""],
        ),
        (
            "two packs with one id",
            vec![],
            vec!["presence"],
            None,
            vec![
                found("style-a", PackType::PresentationPack, "0.1.0", "MIT"),
                FoundPack {
                    dir: PathBuf::from("/other/style-a"),
                    ..found("style-a", PackType::PresentationPack, "0.1.0", "MIT")
                },
            ],
            &[
                "two packs have the id style-a",
                "/roots/style-a",
                "/other/style-a",
            ],
        ),
        (
            "a found pack with a build pack's id",
            vec![],
            vec!["presence"],
            None,
            vec![found(
                "mineworld-presence",
                PackType::PresentationPack,
                "0.1.0",
                "MIT",
            )],
            &[
                "two packs have the id mineworld-presence",
                "this build (system presence)",
            ],
        ),
    ];
    for (case, entries, enabled, license, found, needles) in cases {
        let refusal = run(&entries, &enabled, license, &build(), &found)
            .expect_err(case)
            .to_string();
        for needle in needles {
            assert!(
                refusal.contains(needle),
                "{case}: {needle:?} not in {refusal}"
            );
        }
    }
}

/// The rules run in ARC-54's order, so the first refusal is always the same one: a world breaking
/// every rule at once is refused for the duplicate id, then (without it) for the first requirement in
/// id order, then for the enabled third-party system, then for the licence.
#[test]
fn the_first_failure_in_rule_order_is_the_one_refused() {
    let duplicate = vec![
        found("style-a", PackType::PresentationPack, "0.1.0", "MIT"),
        FoundPack {
            dir: PathBuf::from("/other/style-a"),
            ..found("style-a", PackType::PresentationPack, "0.1.0", "MIT")
        },
    ];
    let everything = [("aaa-absent", "^0.1"), ("style-a", "^9")];
    let enabled = ["presence", "fishing"];
    let order = [
        (duplicate, "two packs have the id"),
        (roots(), "requires: aaa-absent"),
    ];
    for (found, first) in order {
        let refusal = run(
            &everything,
            &enabled,
            Some("GPL-3.0-only"),
            &build(),
            &found,
        )
        .expect_err(first)
        .to_string();
        assert!(refusal.contains(first), "{first}: {refusal}");
    }
    let refusal = run(&[], &enabled, Some("GPL-3.0-only"), &build(), &roots())
        .expect_err("third-party first")
        .to_string();
    assert!(refusal.contains("enables fishing"), "{refusal}");
}
