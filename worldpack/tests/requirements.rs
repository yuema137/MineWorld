//! A world's `requires:` is resolved by the loader in this build and the pack roots it is given, and
//! nowhere else (`DECISIONS.md` `ARC-54`; step-16 §15.4 EB-1, EB-2, EB-4). The rules themselves are
//! `mineworld_packages::resolve`'s and are tested there; this file owns that the loader gathers the
//! right inputs and refuses with the world's file named.

use mineworld_worldpack::{PackError, PackRoots, WorldPack};

/// A scratch directory holding a world `the-world` and a pack root `root/`, removed when the test ends
/// (DEP-29).
struct Scratch(mineworld_test_support::Scratch);

const STYLE: &str = "type: presentation-pack\nversion: 0.1.0\nmineworld: \"^0.1\"\nlicense: MIT\n\
                     authors: [Someone]\n";

impl Scratch {
    fn new(name: &str, identity: &str, top: &str) -> Self {
        let base = mineworld_test_support::scratch!(format!("requirements-{name}"));
        let world = base.join("the-world");
        for directory in ["people", "places"] {
            std::fs::create_dir_all(world.join(directory)).expect("scratch directory");
        }
        std::fs::create_dir_all(base.join("root")).expect("root");
        let manifest = format!(
            "world:\n  id: the-world\n  name: Requirements\n{identity}{top}systems: [presence]\n\
             places: [cafe]\npopulation: [alice]\n"
        );
        std::fs::write(world.join("world.yaml"), manifest).expect("world.yaml");
        std::fs::write(world.join("places/cafe.yaml"), "tags: [cafe]\n").expect("place");
        std::fs::write(
            world.join("people/alice.yaml"),
            "tags: [barista]\nlocation:\n  place: cafe\n",
        )
        .expect("person");
        Self(base)
    }

    fn style(self, id: &str, version: &str) -> Self {
        let dir = self.0.join("root").join(id);
        std::fs::create_dir_all(&dir).expect("pack directory");
        let text = format!("id: {id}\n{}", STYLE.replace("0.1.0", version));
        std::fs::write(dir.join("pack.yaml"), text).expect("pack.yaml");
        self
    }

    fn roots(&self) -> PackRoots {
        PackRoots::new(vec![self.0.join("root")], None).expect("the root exists")
    }

    fn read_with(&self, roots: &PackRoots) -> Result<WorldPack, Box<PackError>> {
        WorldPack::read_with(self.0.join("the-world"), roots).map_err(Box::new)
    }
}

#[test]
fn a_requirement_met_in_a_root_is_in_the_composition() {
    let scratch =
        Scratch::new("met", "", "requires:\n  style-a: \"^0.1\"\n").style("style-a", "0.1.3");
    let pack = scratch.read_with(&scratch.roots()).expect("resolves");
    let composition = pack.composition();
    assert_eq!(composition.required.len(), 1);
    assert_eq!(composition.required[0].identity.id.as_str(), "style-a");
    assert_eq!(
        composition.required[0].identity.version.to_string(),
        "0.1.3"
    );
    let systems: Vec<(&str, bool)> = composition
        .systems
        .iter()
        .map(|s| (s.system.as_str(), s.bundled))
        .collect();
    assert_eq!(
        systems,
        [("presence", true)],
        "the installed presence is bundled"
    );
}

/// Each refusal names the world's file and what was wrong.
#[test]
fn an_unmet_requirement_is_refused_naming_the_world_and_the_pack() {
    for (name, identity, top, version, needles) in [
        (
            "absent",
            "",
            "requires:\n  nowhere: \"^0.1\"\n",
            "0.1.0",
            &["the-world/world.yaml", "requires: nowhere", "root"][..],
        ),
        (
            "out-of-range",
            "",
            "requires:\n  style-a: \"^0.2\"\n",
            "0.1.0",
            &["style-a \"^0.2\"", "found is 0.1.0"][..],
        ),
        (
            "bundled",
            "",
            "requires:\n  mineworld-presence: \"^0.1\"\n",
            "0.1.0",
            &["mineworld-presence is bundled"][..],
        ),
        (
            "licence",
            "  license: GPL-3.0-only\n",
            "",
            "0.1.0",
            &["the-world's licence \"GPL-3.0-only\"", "not allowed"][..],
        ),
    ] {
        let scratch = Scratch::new(name, identity, top).style("style-a", version);
        let refusal = scratch.read_with(&scratch.roots()).expect_err(name);
        assert!(
            matches!(*refusal, PackError::Requirements { .. }),
            "{name}: {refusal:?}"
        );
        let refusal = refusal.to_string();
        for needle in needles {
            assert!(
                refusal.contains(needle),
                "{name}: {needle:?} not in {refusal}"
            );
        }
    }
}

/// `read` resolves with no root: a world that requires something says so, and says no pack directory
/// was given — the pack sitting beside it on disk is never found by accident.
#[test]
fn read_without_roots_finds_nothing_outside_the_build() {
    let scratch =
        Scratch::new("no-roots", "", "requires:\n  style-a: \"^0.1\"\n").style("style-a", "0.1.0");
    let refusal = WorldPack::read(scratch.0.join("the-world"))
        .expect_err("no roots")
        .to_string();
    assert!(
        refusal.contains("requires: style-a") && refusal.contains("no pack directory was given"),
        "{refusal}"
    );
}

/// A world found in a root is identified from its world.yaml and never resolved as a world; requiring
/// one is refused as the wrong type.
#[test]
fn a_world_in_a_root_cannot_be_required() {
    let scratch = Scratch::new("world-in-root", "", "requires:\n  other-world: \"^0.1\"\n");
    let other = scratch.0.join("root/other-world");
    std::fs::create_dir_all(&other).expect("directory");
    std::fs::write(
        other.join("world.yaml"),
        "world:\n  id: other-world\n  name: Other\n  version: 0.1.0\n  license: MIT\n\
         mineworld: \"^0.1\"\nrequires:\n  the-world: \"^0.1\"\n",
    )
    .expect("world.yaml");
    let refusal = scratch
        .read_with(&scratch.roots())
        .expect_err("wrong type")
        .to_string();
    assert!(refusal.contains("other-world is a world-pack"), "{refusal}");
}
