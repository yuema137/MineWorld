//! Pack roots come from the command line, then the environment, and from nowhere else
//! (`DECISIONS.md` `ARC-54` point 3; step-16 §15.4 EB-3).

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use mineworld_packages::PackRoots;

/// Three empty directories under Cargo's per-test scratch root, removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("packages-roots");
        let _ = std::fs::remove_dir_all(&path);
        for name in ["a", "b", "c"] {
            std::fs::create_dir_all(path.join(name)).expect("scratch directory");
        }
        Self(path)
    }

    fn dir(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn roots_are_the_command_lines_then_the_environments_in_order() {
    let scratch = Scratch::new();
    let environment =
        std::env::join_paths([scratch.dir("c"), PathBuf::new(), scratch.dir("b")]).expect("joins");
    let roots =
        PackRoots::new(vec![scratch.dir("a")], Some(environment.as_os_str())).expect("all exist");
    let dirs: Vec<&Path> = roots.dirs().collect();
    assert_eq!(dirs, [scratch.dir("a"), scratch.dir("c"), scratch.dir("b")]);
    assert!(roots.searched().contains("this build or"));

    let none = PackRoots::new(Vec::new(), None).expect("none");
    assert_eq!(none, PackRoots::none());
    assert!(none.searched().contains("no pack directory was given"));
}

#[test]
fn a_root_that_is_not_a_directory_is_refused_naming_its_source() {
    let scratch = Scratch::new();
    let missing = scratch.dir("missing");
    let refusal = PackRoots::new(vec![missing.clone()], None)
        .expect_err("missing")
        .to_string();
    assert!(
        refusal.contains("missing") && refusal.contains("--packs"),
        "{refusal}"
    );

    let environment = OsString::from(missing.as_os_str());
    let refusal = PackRoots::new(Vec::new(), Some(environment.as_os_str()))
        .expect_err("missing")
        .to_string();
    assert!(refusal.contains("MINEWORLD_PACKS"), "{refusal}");
}
