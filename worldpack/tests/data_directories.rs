//! The directory rule of `ARC-121` item 1 through the real loader (S30-b AC-b-3): a World Pack's
//! directories are its format's, `decisions:`'s, or a data directory an installed System Pack owns —
//! none in this build — and a directory whose name begins with `.` is ignored. A required Entity Pack is
//! held to the same rule. Every reference world still reads (`social_cafe.rs` and the other reference
//! tests hold what they read).

use std::path::PathBuf;

use mineworld_worldpack::{PackError, PackRoots, WorldPack};

/// A scratch directory holding a one-place world `the-world` and a pack root `root/`, removed when the
/// test ends (DEP-29).
struct Scratch {
    base: mineworld_test_support::Scratch,
}

impl Scratch {
    fn new(name: &str, top: &str) -> Self {
        let scratch = Self {
            base: mineworld_test_support::scratch!(format!("data-directories-{name}")),
        };
        scratch.write(
            "the-world/world.yaml",
            &format!(
                "world:\n  id: the-world\n  name: Small\nsystems: [presence]\nplaces: [cafe]\n\
                 population: [alice]\n{top}"
            ),
        );
        scratch.write("the-world/places/cafe.yaml", "tags: [cafe]\n");
        scratch.write(
            "the-world/people/alice.yaml",
            "tags: [barista]\nlocation:\n  place: cafe\n",
        );
        std::fs::create_dir_all(scratch.path("root")).expect("root");
        scratch
    }

    fn path(&self, relative: &str) -> PathBuf {
        relative
            .split('/')
            .fold(self.base.to_path_buf(), |path, part| path.join(part))
    }

    fn write(&self, relative: &str, text: &str) {
        let path = self.path(relative);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("parent directory");
        std::fs::write(path, text).expect("file writes");
    }

    fn read(&self) -> Result<WorldPack, PackError> {
        let roots = PackRoots::new(vec![self.path("root")], None).expect("the root exists");
        WorldPack::read_with(self.path("the-world"), &roots)
    }
}

#[test]
fn an_unknown_directory_is_refused_listing_the_ones_this_build_knows() {
    let scratch = Scratch::new("unknown", "");
    scratch.read().expect("the world reads");
    scratch.write("the-world/extras/notes.yaml", "a: b\n");

    let refusal = scratch
        .read()
        .expect_err("extras/ is no directory of a World Pack");
    let PackError::UnknownDirectory { path, known } = &refusal else {
        panic!("got: {refusal}");
    };
    assert_eq!(*path, scratch.path("the-world/extras"));
    assert_eq!(
        known,
        "attributes, configure, data, items, locks, organizations, people, places, populations, vocabularies",
        "the format's directories and the installed attributes pack's two"
    );
    let message = refusal.to_string();
    assert!(
        message.contains("extras") && message.contains("begins with '.' is ignored"),
        "{message}"
    );
}

#[test]
fn a_dot_directory_is_ignored_and_the_decisions_directory_admitted() {
    let scratch = Scratch::new("admitted", "decisions: cognition/decisions.yaml\n");
    scratch.write("the-world/.git/config", "[core]\n");
    scratch.write("the-world/.notes/ideas.yaml", "anything: at all\n");
    scratch.write(
        "the-world/cognition/decisions.yaml",
        "not: read by the loader\n",
    );
    let pack = scratch.read().expect("both are admitted");
    let (file, _) = pack.decisions().expect("named");
    assert_eq!(file.to_string(), "cognition/decisions.yaml");
}

/// AC-b-7's refusal half: an Entity Pack of data only passes the package check, and is then held to the
/// directory rule where the installed packs are known — in this build nobody owns `schemas/`.
#[test]
fn a_required_entity_pack_is_held_to_the_same_rule() {
    let scratch = Scratch::new("entity", "requires:\n  schemas-only: \"^0.1\"\n");
    scratch.write(
        "root/schemas-only/pack.yaml",
        "id: schemas-only\ntype: entity-pack\nversion: 0.1.0\nmineworld: \"^0.1\"\nlicense: MIT\n\
         authors: [Someone]\n",
    );
    scratch.write("root/schemas-only/schemas/a.yaml", "fields: {}\n");

    let refusal = scratch.read().expect_err("schemas/ has no owner here");
    let PackError::UnknownDirectory { path, known } = &refusal else {
        panic!("got: {refusal}");
    };
    assert_eq!(*path, scratch.path("root/schemas-only/schemas"));
    assert_eq!(known, "attributes, items, locks, populations, vocabularies");
}
