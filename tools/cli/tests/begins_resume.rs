//! ARC-371's resume guarantee: `begins:` applies only to worlds created after it is authored. A save created
//! from a pack that began at midnight, resumed under the same pack now saying `begins: "07:00"`, keeps its
//! genesis instant and behaves exactly as if it had never stopped — through `run` and through `server`.
//!
//! ```text
//! run     a midnight save, 1 day, resumed under Social Café (07:00) to day 2  ==  2 uninterrupted days of
//!         the midnight pack: the same facts, journal, snapshots and sealed blocks; genesis stays 0
//! server  the same midnight save hosted under Social Café (07:00): the clock goes on from the head
//!         (day 1, 00:00), not from 07:00 of day 0, and `replay` under the 07:00 pack verifies it
//! ```

mod headless;
mod support;

use std::path::{Path, PathBuf};

use headless::{Tables, fresh, mineworld};
use support::{SaveDir, Server, run_command};

const PACK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/social-cafe");
const DAY: i64 = 86_400;

/// Social Café as it was before `begins:`: a copy whose `world.yaml` drops the key (its id is its directory
/// name), in a scratch removed when dropped.
fn midnight_pack(name: &str) -> (mineworld_test_support::Scratch, PathBuf) {
    let scratch = mineworld_test_support::scratch!(empty name);
    let root = scratch.path().join("social-cafe");
    copy_tree(Path::new(PACK), &root);
    let manifest = root.join("world.yaml");
    let text = std::fs::read_to_string(&manifest).expect("world.yaml");
    let without: String = text
        .lines()
        .filter(|line| !line.starts_with("begins:"))
        .map(|line| format!("{line}\n"))
        .collect();
    assert_ne!(text, without, "Social Café states begins:");
    std::fs::write(&manifest, without).expect("writable");
    (scratch, root)
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a directory");
    for entry in std::fs::read_dir(from).expect("readable") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("copied");
        }
    }
}

/// `run <pack> --headless --seed 7 --days <days> --save <save>`, which must succeed; its stdout.
fn run(pack: &Path, days: u64, save: &Path) -> String {
    let days = days.to_string();
    let output = mineworld(&[
        "run",
        pack.to_str().expect("UTF-8"),
        "--headless",
        "--seed",
        "7",
        "--days",
        &days,
        "--save",
        save.to_str().expect("UTF-8"),
    ]);
    assert!(output.status.success(), "{}", headless::stderr(&output));
    headless::stdout(&output)
}

#[test]
fn a_midnight_save_resumed_under_a_seven_oclock_pack_runs_on_as_if_it_never_stopped() {
    let (_scratch, midnight) = midnight_pack("begins-midnight-1");
    let straight = fresh("begins-straight");
    run(&midnight, 2, &straight);

    let resumed = fresh("begins-resumed");
    run(&midnight, 1, &resumed);
    let printed = run(Path::new(PACK), 2, &resumed);
    assert!(printed.contains("resumed"), "{printed}");

    let tables = Tables::read(&resumed);
    let genesis = social_genesis(&tables);
    assert_eq!(genesis, 0, "the save keeps its own genesis instant");
    tables.assert_same_history(
        &Tables::read(&straight),
        "resumed under the 07:00 pack vs never stopped",
    );
}

/// The genesis instant of a save: its first fact's.
fn social_genesis(tables: &Tables) -> i64 {
    let (_, bytes) = tables.facts.first().expect("a genesis fact");
    let fact: mineworld_contracts::EventEnvelope =
        mineworld_persistence::format::decode(bytes, "fact").expect("a fact");
    fact.at().seconds()
}

#[tokio::test]
async fn a_midnight_save_hosted_under_a_seven_oclock_pack_goes_on_from_its_head() {
    let (_scratch, midnight) = midnight_pack("begins-midnight-2");
    let save = SaveDir::new("begins-hosted");
    run(&midnight, 1, Path::new(save.path()));
    let head = Tables::read(Path::new(save.path())).head_instant();
    assert_eq!(
        head, DAY,
        "a one-day run of a midnight world ends at day 1, 00:00"
    );

    let mut server = Server::start(&["server", PACK, "--save", save.path()]).await;
    let status = server.status().await;
    let at = status["at"].as_i64().expect("a world time");
    assert!(
        (head..head + 3_600).contains(&at),
        "the hosted clock goes on from the save's head, not from the pack's 07:00: {status}"
    );
    let _ = server.kill();

    let (replayed, printed) = run_command(&["replay", PACK, "--save", save.path()]);
    assert!(
        replayed.success(),
        "the save verifies under the 07:00 pack: {printed}"
    );
}
