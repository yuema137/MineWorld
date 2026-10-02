//! What the command does when it is not hosting: reporting a pack, and refusing a bad one.
//!
//! These run the real binary and read its exit status and output, because that is the whole of the
//! contract a command line has with a person: **it says what is wrong, and it fails.** A panic
//! satisfies neither — it reports a backtrace and an author has to know what a `PackError` is.

use std::process::{Command, Output};

const PACK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/social-cafe");

fn run(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mineworld"))
        .args(arguments)
        .output()
        .expect("the mineworld binary runs")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn validate_reports_the_world_the_pack_describes_and_succeeds() {
    let output = run(&["validate", PACK]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let report = stdout(&output);
    for expected in [
        "Social Café",
        "presence, movement, conversation",
        "alice",
        "visitor",
        "wanderer",
        // One passage (the café's front door) and four placements.
        "5 genesis fact(s)",
    ] {
        assert!(
            report.contains(expected),
            "the report must state {expected}: {report}",
        );
    }
    // The ids, in the order the pack allocates them: this is what an author checks before writing a
    // client that refers to them.
    assert!(
        report.contains("1  cafe")
            && report.contains("2  street")
            && report.contains("5  visitor")
            && report.contains("6  wanderer"),
        "the report states which key became which identity: {report}",
    );
}

#[test]
fn a_pack_that_is_not_there_is_refused_by_name_rather_than_panicking() {
    let output = run(&["server", "worlds/there-is-no-such-world"]);

    assert!(!output.status.success(), "a missing pack must fail");
    let complaint = stderr(&output);
    assert!(
        complaint.contains("worlds/there-is-no-such-world"),
        "the complaint names the path: {complaint}",
    );
    assert!(
        !complaint.contains("panicked"),
        "and it is a refusal, not a panic: {complaint}",
    );
}

#[test]
fn a_malformed_pack_is_refused_by_the_command_with_its_position_in_the_file() {
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-malformed");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("people")).expect("a writable temporary directory");
    std::fs::create_dir_all(root.join("places")).expect("a writable temporary directory");
    std::fs::write(
        root.join("world.yaml"),
        "world:\n  id: cli-malformed\n  name: T\nsystems: [presence]\nplaces: [cafe]\npopulation: [alice]\n",
    )
    .expect("the fixture is written");
    std::fs::write(root.join("places/cafe.yaml"), "tags: [cafe]\n").expect("written");
    std::fs::write(
        root.join("people/alice.yaml"),
        "tags: [barista]\nlocatoin:\n  place: cafe\n",
    )
    .expect("written");

    let output = run(&["validate", root.to_str().expect("a printable path")]);

    assert!(!output.status.success(), "a malformed pack must fail");
    let complaint = stderr(&output);
    assert!(
        complaint.contains("people/alice.yaml") && complaint.contains("locatoin"),
        "the complaint names the file and the field: {complaint}",
    );
    assert!(
        complaint.contains("line 2"),
        "and where in the file it is: {complaint}",
    );
    assert!(!complaint.contains("panicked"), "not a panic: {complaint}");
}

#[test]
fn a_command_that_does_not_exist_says_so_and_says_what_does() {
    let output = run(&["inspect", PACK]);

    assert!(!output.status.success());
    let complaint = stderr(&output);
    assert!(
        complaint.contains("does not exist yet") && complaint.contains("mineworld server"),
        "a command S7 will add says so, and points at the ones that work: {complaint}",
    );
}
