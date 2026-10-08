//! A world's package fields — `world.version`, `world.license`, `mineworld:` — are optional to the
//! loader and checked whenever they are present (`DECISIONS.md` `ARC-53`; step-16 §14.4 EA-5, PD-7).
//! None of them is world state: a world reads the same with or without them.

use mineworld_worldpack::{PackError, WorldPack};

/// A one-place, one-person pack in a scratch named after its id (a pack's id is its directory's
/// name), removed when the test ends (DEP-29).
struct Scratch(mineworld_test_support::Scratch);

impl Scratch {
    fn world(id: &str, identity: &str, top: &str) -> Self {
        let root = mineworld_test_support::scratch!(id);
        for directory in ["people", "places"] {
            std::fs::create_dir_all(root.join(directory)).expect("scratch directory");
        }
        let manifest = format!(
            "world:\n  id: {id}\n  name: Fields\n{identity}{top}systems: [presence]\n\
             places: [cafe]\npopulation: [alice]\n"
        );
        std::fs::write(root.join("world.yaml"), manifest).expect("world.yaml");
        std::fs::write(root.join("places/cafe.yaml"), "tags: [cafe]\n").expect("place");
        std::fs::write(
            root.join("people/alice.yaml"),
            "tags: [barista]\nlocation:\n  place: cafe\n",
        )
        .expect("person");
        Self(root)
    }

    fn read(&self) -> Result<WorldPack, PackError> {
        WorldPack::read(&self.0)
    }
}

#[test]
fn the_fields_are_read_typed_and_held_apart_from_the_world() {
    let scratch = Scratch::world(
        "fields-stated",
        "  version: 0.1.0\n  license: MIT OR Apache-2.0\n",
        "mineworld: \"^0.1\"\n",
    );
    let pack = scratch.read().expect("reads");
    let fields = pack.package_fields();
    assert_eq!(
        fields.version.as_ref().map(ToString::to_string).as_deref(),
        Some("0.1.0")
    );
    assert_eq!(
        fields.license.as_ref().map(|l| l.as_str()),
        Some("MIT OR Apache-2.0")
    );
    assert_eq!(
        fields
            .mineworld
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("^0.1")
    );

    let bare = Scratch::world("fields-absent", "", "");
    let pack = bare.read().expect("optional to the loader");
    assert!(pack.package_fields().version.is_none());
}

/// A malformed value is refused at its line, naming the value; a range that excludes the framework is
/// refused naming the range and the framework's version.
#[test]
fn a_stated_field_that_is_wrong_is_refused_by_name() {
    for (id, identity, top, needles) in [
        (
            "fields-bad-version",
            "  version: 1.0\n",
            "",
            &["world.yaml", "line 4"][..],
        ),
        (
            "fields-bad-licence",
            "  license: NOPE\n",
            "",
            &["world.yaml", "'NOPE' is not an SPDX licence expression"][..],
        ),
        (
            "fields-bad-range",
            "",
            "mineworld: \"not a range\"\n",
            &["world.yaml", "'not a range' is not a semver range"][..],
        ),
    ] {
        let refusal = Scratch::world(id, identity, top)
            .read()
            .expect_err(id)
            .to_string();
        for needle in needles {
            assert!(
                refusal.contains(needle),
                "{id}: {needle:?} not in {refusal}"
            );
        }
    }

    let refusal = Scratch::world("fields-framework", "", "mineworld: \"^9\"\n")
        .read()
        .expect_err("^9 excludes 0.1");
    assert!(
        matches!(refusal, PackError::FrameworkNotSupported { .. }),
        "{refusal:?}"
    );
    let refusal = refusal.to_string();
    assert!(
        refusal.contains("world.yaml")
            && refusal.contains("^9")
            && refusal.contains(mineworld_packages::FRAMEWORK_VERSION),
        "{refusal}"
    );
}
