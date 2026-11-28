//! A new save exists only for a world that assembled and began (`DECISIONS.md` `ARC-370`; PR FE-a C2,
//! and C3's genesis cases).
//!
//! Before FE-a, `run --save` and `server --save` created `world.sqlite` *before* the pack assembled, so a
//! world refused while seeding left an empty save behind, and the next launch took the resume branch and
//! failed reading a genesis that was never written. These run the real binary on scratch copies of a
//! world (the reference worlds are not edited):
//!
//! ```text
//! assemble refused (a person form on an item file)   run --save, server --save: non-zero, the save
//!                                                    directory that was absent stays absent, no listener
//! a valid world                                      run --save creates; the same command resumes
//! ```

mod headless;
#[path = "fixture/needs.rs"]
mod needs_fixture;

use std::path::Path;

use headless::{BINARY, PACK, Scratch, fresh, mineworld, stderr, stdout};

const MARKET_TOWN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/market-town");

/// A copy of Market Town whose `bread` item carries the person form of `needs:` — refused when its
/// owner seeds it (`needs-section-kind`), which is during assembly, after the pack was read.
fn refused_at_assembly(name: &str) -> Scratch {
    let scratch = needs_fixture::copied(name, Path::new(MARKET_TOWN));
    let bread = scratch.join("items/bread.yaml");
    let text = std::fs::read_to_string(&bread)
        .expect("reads")
        .replace("\r\n", "\n");
    let replaced: Vec<String> = text
        .lines()
        .map(|line| {
            if line.starts_with("needs:") {
                "needs: { hunger: 5 }".to_owned()
            } else {
                line.to_owned()
            }
        })
        .collect();
    assert_ne!(
        replaced.join("\n") + "\n",
        text,
        "bread carries a needs line"
    );
    std::fs::write(&bread, replaced.join("\n") + "\n").expect("writes");
    scratch
}

/// Every file under `directory`, or `None` when it does not exist.
fn listing(directory: &Path) -> Option<Vec<String>> {
    let entries = std::fs::read_dir(directory).ok()?;
    let mut names: Vec<String> = entries
        .map(|entry| {
            entry
                .expect("an entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    Some(names)
}

/// `run --save` and `server --save` on a world refused while it assembles: both stop with the refusal,
/// and the save directory that did not exist does not exist afterwards — no `world.sqlite`, WAL or SHM —
/// and the server never listened.
#[test]
fn a_world_refused_at_assembly_leaves_no_save_and_no_listener() {
    let pack = refused_at_assembly("save-creation-refused");
    let pack_path = pack.to_str().expect("a printable path");
    let saves = fresh("save-creation-refused-saves");
    std::fs::create_dir_all(&*saves).expect("writable");

    let save = saves.join("run");
    let save_path = save.to_str().expect("a printable path");
    let output = mineworld(&[
        "run",
        pack_path,
        "--headless",
        "--seed",
        "1",
        "--days",
        "1",
        "--save",
        save_path,
    ]);
    let said = format!("{}{}", stderr(&output), stdout(&output));
    assert!(!output.status.success(), "run must stop: {said}");
    assert!(
        said.contains("needs-section-kind"),
        "the refusal is named: {said}"
    );
    assert!(!said.contains("created"), "nothing was created: {said}");
    assert_eq!(listing(&save), None, "the absent save stays absent");

    let save = saves.join("server");
    let save_path = save.to_str().expect("a printable path");
    let output = std::process::Command::new(BINARY)
        .args([
            "server",
            pack_path,
            "--listen",
            "127.0.0.1:0",
            "--save",
            save_path,
        ])
        .stdin(std::process::Stdio::null())
        .output()
        .expect("the mineworld binary runs");
    let said = format!("{}{}", stderr(&output), stdout(&output));
    assert!(!output.status.success(), "server must stop: {said}");
    assert!(
        said.contains("needs-section-kind"),
        "the refusal is named: {said}"
    );
    assert!(!said.contains("listening on"), "no listener: {said}");
    assert!(!said.contains("created"), "nothing was created: {said}");
    assert_eq!(listing(&save), None, "the absent save stays absent");
}

/// A valid world: the first `run --save` creates the save, the same command again resumes it — the
/// creation path still writes a save the resume path reads.
#[test]
fn a_valid_world_is_created_then_resumed() {
    let saves = fresh("save-creation-valid");
    let save = saves.join("world");
    let save_path = save.to_str().expect("a printable path");
    let arguments = [
        "run",
        PACK,
        "--headless",
        "--seed",
        "3",
        "--days",
        "1",
        "--save",
        save_path,
    ];
    let first = mineworld(&arguments);
    assert!(first.status.success(), "{}", stderr(&first));
    assert!(stdout(&first).contains("created"), "{}", stdout(&first));
    assert!(
        listing(&save).is_some_and(|files| files.iter().any(|file| file == "world.sqlite")),
        "the save exists"
    );
    let again = mineworld(&arguments);
    assert!(again.status.success(), "{}", stderr(&again));
    assert!(stdout(&again).contains("resumed"), "{}", stdout(&again));
}

/// C3's genesis case (review R3): `server --save` on a food-energy world whose person starts above
/// their daily use. The world assembles, its genesis is refused by `needs` (`InvalidGenesisContent`),
/// and the server stops naming owner, code, entity and field — with no save and no listener.
#[test]
fn a_world_refused_at_genesis_leaves_no_save_and_no_listener() {
    let pack = needs_fixture::food_energy(
        "save-creation-genesis",
        Path::new(MARKET_TOWN),
        needs_fixture::FoodEnergyCopy {
            daily_kcal: 2_000,
            initial_kcal: 2_001,
            whole_currency: false,
        },
    );
    let saves = fresh("save-creation-genesis-saves");
    let save = saves.join("server");
    let output = std::process::Command::new(BINARY)
        .args([
            "server",
            pack.to_str().expect("a printable path"),
            "--listen",
            "127.0.0.1:0",
            "--save",
            save.to_str().expect("a printable path"),
        ])
        .stdin(std::process::Stdio::null())
        .output()
        .expect("the mineworld binary runs");
    let said = format!("{}{}", stderr(&output), stdout(&output));
    assert!(!output.status.success(), "server must stop: {said}");
    for part in [
        "'needs'",
        "needs-initial-energy-out-of-range",
        "food_energy.initial_kcal",
        "alice",
    ] {
        assert!(said.contains(part), "{part:?} missing from: {said}");
    }
    assert!(!said.contains("listening on"), "no listener: {said}");
    assert_eq!(listing(&save), None, "the absent save stays absent");
}
