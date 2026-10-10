//! A System Pack from another repository, through the real binary (`docs/DECISIONS.md` `ARC-66`;
//! step-16 §16.5 EC-6, EC-7, EC-8).
//!
//! The pack is never named here (EC-5): it is **located** as the one System Pack `packs list` prints
//! `third-party`, cross-checked against the installed set's own `package().bundled()`, and its system id
//! and package id are read from that line. The checkpoint world is Market Town copied into scratch with
//! that system appended to `systems:`, a `requires:` entry for the located pack, a `fishing:` section on
//! the park and a `fish` kind (PD-28). No world under `worlds/` changes.
//!
//! ```text
//! EC-6  packs resolve names the pack third-party with its licence; every other system bundled
//! EC-7  a 30-day seed-7 run: 0 faults; an accepted fish request located, then the catch it caused into
//!       holdings; replay and inspect pass; a second run has the same fingerprint
//! EC-8  refused by name, exit 1: the pack's range missed, the pack not required, the world's licence,
//!       the world's framework range
//! ```

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use mineworld_contracts::{Causation, EventEnvelope};
use mineworld_persistence::{Durability, PersistenceBackend, SqliteBackend, format};
use mineworld_test_support::Scratch;
use mineworld_worldpack::catalog::AVAILABLE;
use serde_json::Value;

const MARKET_TOWN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/market-town");

/// `mineworld <arguments>` with `MINEWORLD_PACKS` removed: (success, stdout, stderr).
fn mineworld(arguments: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_mineworld"))
        .args(arguments)
        .env_remove("MINEWORLD_PACKS")
        .output()
        .expect("the mineworld binary runs");
    (
        output.status.success(),
        String::from_utf8(output.stdout).expect("UTF-8"),
        String::from_utf8(output.stderr).expect("UTF-8"),
    )
}

/// The third-party System Pack of this build: (package id, version, licence, system id).
struct ThirdParty {
    package: String,
    version: String,
    license: String,
    system: String,
}

/// Located from `packs list`, and cross-checked against the installed set: exactly one System Pack is
/// third-party, and it is the capability whose `package().bundled()` is false.
fn located() -> ThirdParty {
    let (ok, listing, err) = mineworld(&["packs", "list"]);
    assert!(ok, "packs list: {err}");
    let found: Vec<ThirdParty> = listing
        .lines()
        .filter(|line| line.starts_with("system-pack "))
        .filter_map(|line| {
            let words: Vec<&str> = line.split_whitespace().collect();
            match words.as_slice() {
                [
                    _,
                    package,
                    version,
                    license,
                    ..,
                    "third-party",
                    "system",
                    system,
                ] => Some(ThirdParty {
                    package: (*package).to_owned(),
                    version: (*version).to_owned(),
                    license: (*license).to_owned(),
                    system: (*system).to_owned(),
                }),
                _ => None,
            }
        })
        .collect();
    let installed: Vec<(String, String)> = AVAILABLE
        .iter()
        .filter(|c| !c.package().bundled())
        .map(|c| (c.package().name().to_owned(), c.id().to_string()))
        .collect();
    assert_eq!(
        found
            .iter()
            .map(|p| (p.package.clone(), p.system.clone()))
            .collect::<Vec<_>>(),
        installed,
        "packs list and the installed set agree on the third-party packs"
    );
    let mut found = found;
    assert_eq!(found.len(), 1, "one third-party System Pack is located");
    found.remove(0)
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a directory");
    for entry in std::fs::read_dir(from).expect("lists") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("a copy");
        }
    }
}

/// The text of `file` with LF line endings: a Windows checkout's copy of a world may be CRLF.
fn read_lf(file: &Path) -> String {
    std::fs::read_to_string(file)
        .expect("reads")
        .replace("\r\n", "\n")
}

/// Replaces exactly one occurrence of `from` in `file`.
fn edit(file: &Path, from: &str, to: &str) {
    let text = read_lf(file);
    assert_eq!(
        text.matches(from).count(),
        1,
        "{}: {from:?}",
        file.display()
    );
    std::fs::write(file, text.replacen(from, to, 1)).expect("writes");
}

/// The checkpoint world (PD-28) in its own scratch, as `market-town/`, with `requires` lines as given
/// (`None`: no `requires:` at all) and the pack's system enabled or not.
fn checkpoint(name: &str, pack: &ThirdParty, range: Option<&str>, enabled: bool) -> Scratch {
    let world =
        mineworld_test_support::scratch!(format!("third-party-{name}")).within("market-town");
    copy_dir(Path::new(MARKET_TOWN), &world);
    let manifest = world.join("world.yaml");
    // The system joins `systems:` beside a market pack (inside the list whatever follows it there); the
    // requirement is a top-level key, appended at the end of the file.
    if enabled {
        edit(
            &manifest,
            "  - consumption\n",
            &format!("  - consumption\n  - {}\n", pack.system),
        );
    }
    edit(&manifest, "  - umbrella\n", "  - umbrella\n  - fish\n");
    if let Some(range) = range {
        let text = read_lf(&manifest);
        std::fs::write(
            &manifest,
            text + &format!("\nrequires:\n  {}: \"{range}\"\n", pack.package),
        )
        .expect("writes");
    }
    std::fs::write(
        world.join("items/fish.yaml"),
        "# A fish, caught at the park's pond.\nitem: { category: food }\n",
    )
    .expect("writes");
    if enabled {
        let park = world.join("places/park.yaml");
        let text = read_lf(&park);
        std::fs::write(park, text + "\nfishing: { catch: fish, minutes: 60 }\n").expect("writes");
    }
    world
}

fn path(world: &Path) -> &str {
    world.to_str().expect("a UTF-8 path")
}

/// EC-6: the composition names the pack third-party, at its version and licence; every other system
/// bundled.
#[test]
fn the_checkpoint_world_resolves_with_the_pack_third_party_and_its_licence_judged() {
    let pack = located();
    let world = checkpoint("resolve", &pack, Some("^0.1"), true);
    let (ok, out, err) = mineworld(&["packs", "resolve", path(&world)]);
    assert!(ok, "{err}");
    println!("{out}");
    assert!(
        out.contains(&format!(
            "requires   {} \"^0.1\" → system-pack {} (this build, third-party)",
            pack.package, pack.version
        )),
        "{out}"
    );
    assert!(
        out.contains(&format!(
            "system     {} → {} {} (third-party)",
            pack.system, pack.package, pack.version
        )),
        "{out}"
    );
    assert!(
        out.contains("system     presence → mineworld-presence 0.1.0 (bundled)"),
        "{out}"
    );
    let systems: Vec<&str> = out.lines().filter(|l| l.starts_with("  system ")).collect();
    let market = mineworld_worldpack::WorldPack::read(MARKET_TOWN)
        .expect("Market Town reads")
        .systems()
        .len();
    assert_eq!(
        systems.len(),
        market + 1,
        "Market Town's {market} and the pack's: {out}"
    );
    assert_eq!(
        systems
            .iter()
            .filter(|l| l.ends_with("(third-party)"))
            .count(),
        1
    );
    assert_eq!(
        pack.license, "MIT",
        "judged by the default policy, which admits MIT"
    );
}

/// Every fact of a save, decoded.
fn facts_of(save: &Path) -> Vec<EventEnvelope> {
    let backend = SqliteBackend::open(save, Durability::ProcessCrash).expect("the save opens");
    let all = usize::try_from(i64::MAX).expect("64-bit");
    backend
        .last_facts(all)
        .expect("facts")
        .into_iter()
        .map(|fact| format::decode(&fact.bytes, "fact").expect("a stored fact decodes"))
        .collect()
}

fn payload(fact: &EventEnvelope) -> Value {
    serde_json::from_slice(fact.payload().payload()).expect("a JSON payload")
}

/// EC-7: the pack lives in a composed world, run headless, saved, replayed and inspected.
#[test]
fn the_checkpoint_world_runs_thirty_days_and_somebody_fishes_and_catches() {
    let pack = located();
    let world = checkpoint("run", &pack, Some("^0.1"), true);
    let saves = mineworld_test_support::scratch!(empty "third-party-saves");
    let (first, second) = (saves.join("first"), saves.join("second"));
    let run = |save: &Path| {
        let (ok, out, err) = mineworld(&[
            "run",
            path(&world),
            "--headless",
            "--seed",
            "7",
            "--days",
            "30",
            "--save",
            path(save),
        ]);
        assert!(ok, "run failed: {err}\n{out}");
        out
    };
    let printed = run(&first);
    assert!(printed.lines().any(|l| l == "faults     0"), "{printed}");

    // Located: the fish requests that were accepted — each states the pack's fishing-started, caused by
    // the request — and then the catches: inventory's items-produced at a started catch's due instant,
    // for its angler, caused by a process.
    let facts = facts_of(&first);
    let owned_by_pack = |fact: &EventEnvelope, slug: &str| fact.event_type().as_str() == slug;
    let started: Vec<(u64, Value)> = facts
        .iter()
        .filter(|fact| owned_by_pack(fact, "fishing-started"))
        .filter_map(|fact| match fact.caused_by() {
            Causation::Action(id) => Some((id.raw(), payload(fact))),
            _ => None,
        })
        .collect();
    // (angler's entity, due instant) of every accepted request's catch.
    let started_catches: BTreeSet<(String, String)> = started
        .iter()
        .map(|(_, p)| (p["angler"]["entity"].to_string(), p["due"].to_string()))
        .collect();
    // The processes whose `fishing-ended` says the catch came in, with their angler.
    let landed: BTreeSet<(String, String)> = facts
        .iter()
        .filter(|fact| owned_by_pack(fact, "fishing-ended"))
        .filter(|fact| payload(fact)["caught"] == Value::Bool(true))
        .filter_map(|fact| match fact.caused_by() {
            Causation::Process(process) => Some((
                format!("{process:?}"),
                payload(fact)["angler"]["entity"].to_string(),
            )),
            _ => None,
        })
        .collect();
    let caught: Vec<&EventEnvelope> = facts
        .iter()
        .filter(|fact| fact.event_type().as_str() == "items-produced")
        .filter(|fact| {
            let Causation::Process(process) = fact.caused_by() else {
                return false;
            };
            let holder = payload(fact)["holder"].to_string();
            let at = serde_json::to_value(fact.at()).expect("an instant");
            landed.contains(&(format!("{process:?}"), holder.clone()))
                && started_catches.contains(&(holder, at.to_string()))
        })
        .collect();
    println!(
        "{} accepted fish requests, {} catches into holdings",
        started.len(),
        caught.len()
    );
    assert!(!started.is_empty(), "no accepted fish");
    assert!(
        !caught.is_empty(),
        "no catch whose cause reaches an accepted fish"
    );

    let (ok, _, err) = mineworld(&["replay", path(&world), "--save", path(&first)]);
    assert!(ok, "replay: {err}");
    let (ok, _, err) = mineworld(&["inspect", path(&first)]);
    assert!(ok, "inspect: {err}");

    let again = run(&second);
    let fingerprint = |printed: &str| -> Vec<String> {
        printed
            .lines()
            .filter(|l| !l.starts_with("[mineworld] run ") && !l.starts_with("wall "))
            .map(str::to_owned)
            .collect()
    };
    assert_eq!(
        fingerprint(&printed),
        fingerprint(&again),
        "same seed, same run"
    );
    assert_eq!(facts_of(&second).len(), facts.len());
}

/// One refusal case: its name, the `requires` range (none: not required), an edit of `world.yaml`
/// (from, to; empty: none), and the words the refusal must name.
type Misuse<'a> = (&'a str, Option<&'a str>, &'a str, &'a str, Vec<String>);

/// EC-8: each refusal names what was wrong, exit 1.
#[test]
fn a_world_that_misuses_the_pack_is_refused_by_name() {
    let pack = located();
    let cases: [Misuse<'_>; 4] = [
        (
            "range",
            Some("^0.2"),
            "",
            "",
            vec![
                pack.package.clone(),
                "^0.2".to_owned(),
                pack.version.clone(),
            ],
        ),
        (
            "unrequired",
            None,
            "",
            "",
            vec![
                pack.system.clone(),
                pack.package.clone(),
                "requires".to_owned(),
            ],
        ),
        (
            "licence",
            Some("^0.1"),
            "  license: MIT\n",
            "  license: GPL-3.0-only\n",
            vec![
                "GPL-3.0-only".to_owned(),
                "MIT".to_owned(),
                "Apache-2.0".to_owned(),
            ],
        ),
        (
            "framework",
            Some("^0.1"),
            "mineworld: \"^0.1\"\n",
            "mineworld: \"^9\"\n",
            vec!["^9".to_owned(), "0.1.0".to_owned()],
        ),
    ];
    for (name, range, from, to, named) in cases {
        let world = checkpoint(name, &pack, range, true);
        if !from.is_empty() {
            edit(&world.join("world.yaml"), from, to);
        }
        for command in [&["validate"][..], &["packs", "resolve"][..]] {
            let arguments = [command, &[path(&world)][..]].concat();
            let output = Command::new(env!("CARGO_BIN_EXE_mineworld"))
                .args(&arguments)
                .env_remove("MINEWORLD_PACKS")
                .output()
                .expect("runs");
            let err = String::from_utf8_lossy(&output.stderr);
            println!("{name} {command:?}: {}", err.trim());
            assert_eq!(output.status.code(), Some(1), "{name} {command:?}: {err}");
            for word in &named {
                assert!(
                    err.contains(word.as_str()),
                    "{name} {command:?}: no {word:?} in {err}"
                );
            }
        }
    }
}
