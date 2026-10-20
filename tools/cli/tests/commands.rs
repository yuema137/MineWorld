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

/// The line `validate` must print for `pack`'s genesis: the count the loader itself states.
fn genesis_line(pack: &str) -> String {
    let loaded = mineworld_worldpack::WorldPack::read(pack)
        .expect("reads")
        .load(mineworld_contracts::WorldTime::EPOCH)
        .expect("loads");
    assert!(!loaded.genesis().is_empty(), "the pack states facts");
    format!("{} genesis fact(s)", loaded.genesis().len())
}

#[test]
fn validate_reports_the_world_the_pack_describes_and_succeeds() {
    let output = run(&["validate", PACK]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let report = stdout(&output);
    let genesis = genesis_line(PACK);
    for expected in [
        "Social Café",
        "presence, movement, conversation, group-activity, relationships, naming, schedule, bodies, \
         persona, appearance, calendar, posture\n",
        "alice",
        "visitor",
        "wanderer",
        // As many genesis facts as the loader states for this pack: the count is the pack's, read by
        // the library the command reports through, so a content change never edits this test.
        genesis.as_str(),
    ] {
        assert!(
            report.contains(expected),
            "the report must state {expected}: {report}",
        );
    }
    // The ids, in the order the pack allocates them — places in key order, then people — which is what an
    // author checks before writing a client that refers to them. Read from the report's own key lists,
    // so a pack that gains a place (S29's station) moves every later id and the check still holds.
    let keys = |label: &str| -> Vec<String> {
        report
            .lines()
            .find_map(|line| line.trim_start().strip_prefix(label))
            .map(|rest| rest.split(", ").map(|key| key.trim().to_owned()).collect())
            .unwrap_or_default()
    };
    let (places, people) = (keys("places     "), keys("people     "));
    assert!(places.contains(&"cafe".to_owned()) && people.contains(&"visitor".to_owned()));
    for (index, key) in places.iter().chain(&people).enumerate() {
        assert!(
            report.contains(&format!("{}  {key}\n", index + 1)),
            "the report states which key became which identity ({key} is {}): {report}",
            index + 1
        );
    }
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
    let root = mineworld_test_support::scratch!("cli-malformed");
    std::fs::create_dir_all(root.join("people")).expect("a writable temporary directory");
    std::fs::create_dir_all(root.join("places")).expect("a writable temporary directory");
    std::fs::write(
        root.join("world.yaml"),
        "world:\n  id: cli-malformed\n  name: T\nsettlement: { kind: none }\nsystems: [presence]\nplaces: [cafe]\npopulation: [alice]\n",
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
    // The file is named as the host writes paths (`people\alice.yaml` on Windows).
    let file = std::path::Path::new("people").join("alice.yaml");
    assert!(
        complaint.contains(&file.display().to_string()) && complaint.contains("locatoin"),
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
    // `inspect` and `create` exist since step-08 C6 and C7; the claim is unchanged, about a command
    // that still does not (MODULE_SPEC §8's `install`).
    let output = run(&["install", "employment"]);

    assert!(!output.status.success());
    let complaint = stderr(&output);
    assert!(
        complaint.contains("does not exist yet") && complaint.contains("mineworld server"),
        "a command S7 will add says so, and points at the ones that work: {complaint}",
    );
}
