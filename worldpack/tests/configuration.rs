//! IA-3: `configure:` is refused by name, through `WorldPack::read` on scratch worlds
//! (`DECISIONS.md` `ARC-61`).
//!
//! These are the refusals this build's packs can reach: none of them is configurable, so a key that
//! resolves to an enabled pack is refused as not configurable, and every later check (a missing file,
//! a decode, a required system, a named entity) is held by worldpack's in-crate tests with a probe
//! configuration, and by the canary (IA-10). A world without `configure:` reads exactly as before —
//! every shipped world's own tests hold that, unedited.
//!
//! Each scratch world lives under cargo's temporary directory for this target and is removed when its
//! test ends.

use std::path::PathBuf;

use mineworld_contracts::SystemId;
use mineworld_worldpack::{PackError, WorldPack};

/// A scratch World Pack: one place, one person, presence and movement enabled. Its directory is
/// `mineworld-test-support`'s scratch, named as the pack and removed when the test ends (`DEP-29`).
struct Scratch {
    root: PathBuf,
    _guard: mineworld_test_support::Scratch,
}

impl Scratch {
    fn new(id: &str, configure: &str) -> Self {
        let guard = mineworld_test_support::scratch!(empty id);
        let root = guard.path().to_path_buf();
        for directory in ["people", "places", "configure"] {
            std::fs::create_dir_all(root.join(directory)).expect("a writable temporary directory");
        }
        let scratch = Self {
            root,
            _guard: guard,
        };
        scratch.write(
            "world.yaml",
            &format!(
                "world:\n  id: {id}\n  name: A Scratch World\nsystems:\n  - presence\n  - movement\n\
                 places:\n  - square\npopulation:\n  - ada\n{configure}"
            ),
        );
        scratch.write("places/square.yaml", "tags: [square]\n");
        scratch.write(
            "people/ada.yaml",
            "tags: [walker]\nlocation:\n  place: square\n",
        );
        scratch
    }

    fn write(&self, relative: &str, contents: &str) {
        std::fs::write(self.root.join(relative), contents).expect("a writable scratch file");
    }

    fn read(&self) -> Result<WorldPack, PackError> {
        WorldPack::read(&self.root)
    }

    fn manifest(&self) -> PathBuf {
        self.root.join("world.yaml")
    }
}

/// A key that names no system of this build is refused, listing the systems that exist.
#[test]
fn a_key_that_is_no_system_of_this_build_is_refused_listing_the_systems() {
    let scratch = Scratch::new("configure-unknown", "configure:\n  - weather\n");
    match scratch.read() {
        Err(PackError::ConfigurationOfUnknownSystem {
            key,
            available,
            path,
        }) => {
            assert_eq!(key, "weather");
            assert!(available.contains("'presence'") && available.contains("'movement'"));
            assert_eq!(path, scratch.manifest());
        }
        other => panic!("expected ConfigurationOfUnknownSystem, got {other:?}"),
    }
}

/// Both reserved keys are refused, saying what each is reserved for.
#[test]
fn the_reserved_keys_are_refused_naming_what_they_are_reserved_for() {
    for (key, reserved) in [
        ("classes", "entity classes"),
        ("packages", "licence policy"),
    ] {
        let scratch = Scratch::new(
            &format!("configure-reserved-{key}"),
            &format!("configure:\n  - {key}\n"),
        );
        let refusal = scratch.read().expect_err("a reserved key is refused");
        let message = refusal.to_string();
        match refusal {
            PackError::ConfigurationReserved {
                key: refused,
                reserved_for,
                ..
            } => {
                assert_eq!(refused, key);
                assert!(reserved_for.contains(reserved), "{reserved_for}");
                assert!(
                    message.contains("not configurable in this build"),
                    "{message}"
                );
            }
            other => panic!("expected ConfigurationReserved, got {other:?}"),
        }
    }
}

/// A system the build has but the world does not enable is refused, naming it.
#[test]
fn a_system_the_world_does_not_enable_is_refused() {
    let scratch = Scratch::new("configure-not-enabled", "configure:\n  - schedule\n");
    match scratch.read() {
        Err(PackError::ConfigurationOwnerNotEnabled { system, path }) => {
            assert_eq!(system, SystemId::from_static("schedule"));
            assert_eq!(path, scratch.manifest());
        }
        other => panic!("expected ConfigurationOwnerNotEnabled, got {other:?}"),
    }
}

/// An enabled system that takes no configuration is refused, naming it.
#[test]
fn an_enabled_system_that_takes_no_configuration_is_refused() {
    let scratch = Scratch::new("configure-not-configurable", "configure:\n  - movement\n");
    scratch.write("configure/movement.yaml", "{}\n");
    match scratch.read() {
        Err(PackError::NotConfigurable { system, .. }) => {
            assert_eq!(system, SystemId::from_static("movement"));
        }
        other => panic!("expected NotConfigurable, got {other:?}"),
    }
}

/// A key listed twice is refused, naming it.
#[test]
fn a_key_listed_twice_is_refused() {
    let scratch = Scratch::new(
        "configure-twice",
        "configure:\n  - movement\n  - movement\n",
    );
    match scratch.read() {
        Err(PackError::ConfigurationListedTwice { key, .. }) => assert_eq!(key, "movement"),
        other => panic!("expected ConfigurationListedTwice, got {other:?}"),
    }
}

/// A file in `configure/` that `configure:` does not list is refused, naming the file: the
/// configuration an author believes they wrote and the world would never see. Only `.yaml` files are
/// configuration; a README beside them is not.
#[test]
fn a_configuration_file_that_is_not_listed_is_refused_naming_the_file() {
    let scratch = Scratch::new("configure-undeclared", "");
    scratch.write("configure/README.md", "notes\n");
    scratch.read().expect("a README is not configuration");

    scratch.write("configure/movement.yaml", "{}\n");
    match scratch.read() {
        Err(PackError::ConfigurationFileNotDeclared { key, path }) => {
            assert_eq!(key, "movement");
            assert_eq!(path, scratch.root.join("configure/movement.yaml"));
        }
        other => panic!("expected ConfigurationFileNotDeclared, got {other:?}"),
    }
}

/// `configure:` is a list of system ids: a key that is not one is refused at its line, by the
/// contract's own rule.
#[test]
fn a_key_that_is_not_a_system_id_is_refused_at_its_line() {
    let scratch = Scratch::new("configure-malformed", "configure:\n  - Not A Key\n");
    match scratch.read() {
        Err(PackError::Malformed { detail, .. }) => {
            assert!(
                detail.contains("line 12 column 5"),
                "the refusal names the line and column: {detail}"
            );
        }
        other => panic!("expected Malformed, got {other:?}"),
    }
}

/// No installed pack has a reserved key as its id, so a reserved key can never shadow a pack.
#[test]
fn no_installed_pack_has_a_reserved_key_as_its_id() {
    for capability in mineworld_worldpack::catalog::AVAILABLE {
        assert!(
            !mineworld_worldpack::configure::RESERVED
                .iter()
                .any(|(key, _)| *key == capability.id().as_str()),
            "'{capability}' has a reserved key as its id"
        );
    }
}
