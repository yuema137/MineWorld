//! `world.yaml`'s `decisions:` key (`MODULE_SPEC.md` §4.1; `pr-dl-a-decision-list.md` SD-DLa-3): the
//! loader checks the file exists inside the pack and never reads it; a world without the key reads as
//! before.

use std::path::PathBuf;

use mineworld_worldpack::{PackError, WorldPack};

/// A scratch World Pack with one place and one person, and `decisions` appended to its `world.yaml`.
struct Scratch {
    root: PathBuf,
    _guard: mineworld_test_support::Scratch,
}

impl Scratch {
    fn new(id: &str, decisions: &str) -> Self {
        let guard = mineworld_test_support::scratch!(empty id);
        let root = guard.path().to_path_buf();
        for directory in ["people", "places"] {
            std::fs::create_dir_all(root.join(directory)).expect("a writable temporary directory");
        }
        let scratch = Self {
            root,
            _guard: guard,
        };
        scratch.write(
            "world.yaml",
            &format!(
                "world:\n  id: {id}\n  name: A Scratch World\nsystems:\n  - presence\nplaces:\n  - \
                 square\npopulation:\n  - ada\nseats:\n  - ada\n{decisions}"
            ),
        );
        scratch.write("places/square.yaml", "tags: [square]\n");
        scratch.write("people/ada.yaml", "location:\n  place: square\n");
        scratch
    }

    fn write(&self, relative: &str, contents: &str) {
        std::fs::write(self.root.join(relative), contents).expect("a writable scratch file");
    }

    fn read(&self) -> Result<WorldPack, PackError> {
        WorldPack::read(&self.root)
    }
}

#[test]
fn a_world_without_the_key_has_no_decision_list() {
    let scratch = Scratch::new("decisions-absent", "");
    let pack = scratch.read().expect("reads");
    assert!(pack.decisions().is_none());
}

/// The file is the hosts' to parse: the loader accepts content it could not decode.
#[test]
fn the_key_names_a_file_of_the_pack_whose_content_the_loader_never_reads() {
    let scratch = Scratch::new("decisions-present", "decisions: lists/decisions.yaml\n");
    std::fs::create_dir_all(scratch.root.join("lists")).expect("writable");
    scratch.write(
        "lists/decisions.yaml",
        "this: [is not, read: by the loader\n",
    );
    let pack = scratch.read().expect("reads");
    let (file, path) = pack.decisions().expect("named");
    assert_eq!(file.to_string(), "lists/decisions.yaml");
    assert_eq!(path, scratch.root.join("lists").join("decisions.yaml"));
}

#[test]
fn a_missing_file_is_refused_naming_it() {
    let scratch = Scratch::new("decisions-missing", "decisions: decisions.yaml\n");
    let refusal = scratch.read().expect_err("refused");
    assert!(
        matches!(&refusal, PackError::DecisionsFileMissing { file, .. } if file == "decisions.yaml"),
        "{refusal}"
    );
    assert!(
        refusal
            .to_string()
            .contains("decisions: names decisions.yaml, which does not exist")
    );
}

#[test]
fn a_path_that_could_leave_the_pack_is_refused_as_it_decodes() {
    for (name, written, said) in [
        ("decisions-up", "../decisions.yaml", "'..' component"),
        ("decisions-absolute", "/etc/decisions.yaml", "is absolute"),
        (
            "decisions-backslash",
            "lists\\\\decisions.yaml",
            "uses '\\'",
        ),
        ("decisions-drive", "c:decisions.yaml", "names a drive"),
    ] {
        let scratch = Scratch::new(name, &format!("decisions: \"{written}\"\n"));
        let refusal = scratch.read().expect_err("refused").to_string();
        assert!(refusal.contains(said), "{name}: {refusal}");
    }
}

/// A link inside the pack that leads out of it is refused by its real path.
#[cfg(unix)]
#[test]
fn a_link_out_of_the_pack_is_refused() {
    let outside = mineworld_test_support::scratch!(empty "decisions-elsewhere");
    std::fs::write(outside.path().join("decisions.yaml"), "default: rule\n").expect("writable");
    let scratch = Scratch::new("decisions-linked", "decisions: decisions.yaml\n");
    std::os::unix::fs::symlink(
        outside.path().join("decisions.yaml"),
        scratch.root.join("decisions.yaml"),
    )
    .expect("a symlink");
    let refusal = scratch.read().expect_err("refused");
    assert!(
        matches!(refusal, PackError::DecisionsOutsidePack { .. }),
        "{refusal}"
    );
}
