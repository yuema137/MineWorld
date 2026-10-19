//! A test copy of a world with needs and sleep switched **off** (`pr-s35-nd-content.md` §6, AC-S35-2 (b)).
//!
//! N-c's copy that switched the packs on is gone: Market Town and Commute Town carry them as content
//! since N-d. What stays is its removal form — the step's §7 "both removed" row:
//!
//! ```text
//! systems, configure   the `sleep` and `needs` entries dropped
//! configure/           sleep.yaml and needs.yaml deleted
//! people/, items/      every `needs:` line dropped (each is a one-line section in the worlds that
//!                      carry one)
//! ```
//!
//! Everything else — supply, prices, wallets — is kept, so the copy is the base world plus N-d's sizing.
//! Nothing here names a world: it reads the copied files, so any World Pack can be copied.

#![allow(dead_code)]

use std::path::Path;

use crate::headless::{Scratch, fresh};

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("writable");
    for entry in std::fs::read_dir(from).expect("readable") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copied");
        }
    }
}

fn read_lf(path: &Path) -> String {
    std::fs::read_to_string(path)
        .expect("reads")
        .replace("\r\n", "\n")
}

/// `path` without the lines `drop` selects; returns how many it dropped.
fn without_lines(path: &Path, drop: impl Fn(&str) -> bool) -> usize {
    let text = read_lf(path);
    let kept: Vec<&str> = text.lines().filter(|line| !drop(line)).collect();
    let dropped = text.lines().count() - kept.len();
    std::fs::write(path, kept.join("\n") + "\n").expect("writes");
    dropped
}

/// A copy of the world at `world` named `name`, with `sleep` and `needs` removed with their files and
/// sections. Fails if the world did not enable them: a removal of nothing proves nothing.
pub fn without_needs(name: &str, world: &Path) -> Scratch {
    let key = world
        .file_name()
        .and_then(|name| name.to_str())
        .expect("a world directory");
    let scratch = fresh(name).within(key);
    copy_dir(world, &scratch);
    let dropped = without_lines(&scratch.join("world.yaml"), |line| {
        line == "  - sleep" || line == "  - needs"
    });
    assert!(dropped >= 2, "{key} enables sleep and needs");
    for file in ["configure/sleep.yaml", "configure/needs.yaml"] {
        let path = scratch.join(file);
        if path.exists() {
            std::fs::remove_file(path).expect("removable");
        }
    }
    let mut sections = 0;
    for directory in ["people", "items"] {
        for entry in std::fs::read_dir(scratch.join(directory)).expect("lists") {
            let path = entry.expect("an entry").path();
            sections += without_lines(&path, |line| line.starts_with("needs:"));
        }
    }
    assert!(sections > 0, "{key} carries needs: sections");
    scratch
}
