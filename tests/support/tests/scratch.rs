//! The scratch guard's contract (`ENGINEERING_STANDARDS.md` §22, "Test scratch"): where a scratch
//! lives, that it is gone when its guard drops — pass or fail — unless the keep policy says otherwise,
//! and that a live name cannot be taken twice.
//!
//! Each test that needs to observe the container itself nests a private root inside a scratch of its
//! own, so the tests of this binary, which run in parallel, never observe one another's scratches.

use std::path::{Path, PathBuf};

use mineworld_test_support::{Keep, Scratch, Shape, scratch};

fn container_in(root: &Path) -> PathBuf {
    root.join(format!("mineworld-scratch-{}", std::process::id()))
}

fn root_of(scratch: &Scratch) -> &str {
    scratch.to_str().expect("a UTF-8 path")
}

#[test]
fn a_scratch_is_named_as_asked_under_the_target_tmpdir_and_gone_when_dropped() {
    let save = scratch!("helper-absent");
    let path = save.path().to_path_buf();
    assert_eq!(
        path,
        container_in(Path::new(env!("CARGO_TARGET_TMPDIR"))).join("helper-absent"),
        "the leaf is exactly the name, in this process's container under CARGO_TARGET_TMPDIR"
    );
    assert!(!path.exists(), "an absent scratch does not exist yet");
    std::fs::create_dir_all(path.join("inner")).expect("the test may create it");
    std::fs::write(path.join("inner/file"), "x").expect("and write into it");
    drop(save);
    assert!(!path.exists(), "dropping the guard removes the tree");

    let pack = scratch!(empty format!("helper-{}", "empty"));
    let path = pack.path().to_path_buf();
    assert!(path.is_dir(), "an empty scratch is a directory");
    assert_eq!(
        std::fs::read_dir(&path).expect("readable").count(),
        0,
        "and it is empty"
    );
    drop(pack);
    assert!(!path.exists());
}

#[test]
fn a_pack_copy_is_named_as_its_pack_and_its_whole_scratch_goes_with_it() {
    let copy = scratch!("helper-within").within("market-town");
    let owned = container_in(Path::new(env!("CARGO_TARGET_TMPDIR"))).join("helper-within");
    assert_eq!(copy.path(), owned.join("market-town"));
    std::fs::create_dir_all(copy.join("places")).expect("the copy is written");
    drop(copy);
    assert!(
        !owned.exists(),
        "the scratch is removed, not only the copy in it"
    );
}

#[test]
fn a_failing_test_loses_its_scratch_unless_failures_are_kept() {
    let outer = scratch!(empty "helper-keep-on-failure");
    let root = root_of(&outer);
    for (keep, kept) in [
        (Keep::Never, false),
        (Keep::Failed, true),
        (Keep::All, true),
    ] {
        let path = container_in(Path::new(root)).join("failing");
        let failed = std::panic::catch_unwind(|| {
            let scratch = Scratch::with_keep(root, "failing", Shape::Empty, keep);
            std::fs::write(scratch.join("evidence"), "x").expect("written");
            panic!("the test fails while holding its scratch");
        });
        assert!(failed.is_err());
        assert_eq!(
            path.join("evidence").exists(),
            kept,
            "{keep:?}: a failing test's scratch is kept only when the policy keeps failures"
        );
        let _ = std::fs::remove_dir_all(container_in(Path::new(root)));
    }
}

#[test]
fn a_passing_test_loses_its_scratch_unless_everything_is_kept() {
    let outer = scratch!(empty "helper-keep-on-success");
    let root = root_of(&outer);
    for (keep, kept) in [
        (Keep::Never, false),
        (Keep::Failed, false),
        (Keep::All, true),
    ] {
        let path = container_in(Path::new(root)).join("passing");
        let scratch = Scratch::with_keep(root, "passing", Shape::Empty, keep);
        std::fs::write(scratch.join("evidence"), "x").expect("written");
        drop(scratch);
        assert_eq!(path.exists(), kept, "{keep:?}");
        assert_eq!(
            container_in(Path::new(root)).exists(),
            kept,
            "{keep:?}: the container goes with its last scratch unless something is kept in it"
        );
        let _ = std::fs::remove_dir_all(container_in(Path::new(root)));
    }
}

#[test]
fn a_live_name_cannot_be_taken_twice_and_is_free_again_once_dropped() {
    let first = scratch!("helper-duplicate");
    let second = std::panic::catch_unwind(|| scratch!("helper-duplicate"));
    let message = second.expect_err("the second guard is refused");
    let message = message
        .downcast_ref::<String>()
        .expect("a formatted message");
    assert!(
        message.contains("\"helper-duplicate\" is already in use"),
        "the refusal names the scratch: {message}"
    );
    drop(first);
    let again = scratch!("helper-duplicate");
    assert!(!again.exists());
}

#[test]
fn a_scratch_name_is_one_plain_component() {
    for name in ["", "a/b", "..", "/abs"] {
        assert!(
            std::panic::catch_unwind(|| scratch!(name)).is_err(),
            "{name:?} is refused"
        );
    }
}

#[test]
fn the_keep_variable_accepts_failed_and_all_and_refuses_anything_else() {
    assert_eq!(Keep::parse(None), Ok(Keep::Never));
    assert_eq!(Keep::parse(Some("")), Ok(Keep::Never));
    assert_eq!(Keep::parse(Some("failed")), Ok(Keep::Failed));
    assert_eq!(Keep::parse(Some("all")), Ok(Keep::All));
    let refusal = Keep::parse(Some("1")).expect_err("a typo is refused");
    assert!(refusal.contains("MINEWORLD_KEEP_SCRATCH"), "{refusal}");
}
