//! **Milestone E — a world assembled from independently installable packs**, through the real binary
//! (step-16 §8.2, §18.5 EE-1 … EE-9; `docs/DECISIONS.md` `ARC-77`).
//!
//! Lakeside (`worlds/lakeside`) is composed of bundled System Packs, one third-party System Pack (the
//! fishing pack, installed by `ARC-33`'s two lines and pinned by `ARC-66`), the repository's own Entity
//! Pack and the two default Presentation Packs. Every command here names the data packs' roots
//! (`--packs entities --packs presentation/mineworld-default`) and removes `MINEWORLD_PACKS` from every
//! child it spawns itself, so a developer's own setting cannot leak in (`ARC-54`).
//!
//! ```text
//! M-1  every pack listed with its identity; the third-party pack located; the binary at 0.1.0
//! M-2  the composition: the framework range, each requirement, one system line per enabled system
//! M-3  six misuses refused by name, exit 1, through validate and packs resolve
//! M-4  a new Entity Pack in a fresh root is used without a rebuild
//! M-5  300 headless days: every seat moves, talks, fishes and eats in every 30-day bucket; the catch
//!      is eaten in every bucket; a kind of the Entity Pack is eaten
//! M-6  the same seed is the same world, byte for byte, also across SIGKILL and resume; seed 8 differs
//! M-7  inspect and replay pass; inspect names the located system
//! M-8  the pack reaches a player through the unchanged server
//! M-9  disabling the pack takes away only its own facts, and fish becomes unavailable
//! ```
//!
//! **Pass rule.** Nothing is counted before it is located, and nothing is compared before every
//! compared run has been shown to live (`ARC-23`). No pack identity is copied into this file (I-E5): the
//! third-party pack is the one `packs list` prints `third-party`, cross-checked against the installed
//! set; the Entity Pack is the one found in `entities/`; seats, water and kinds come from the world's
//! own files (`lakeside/`). M-1's revision and M-10's structure are `package_sources.rs`'s (EC-2, EC-4),
//! run by the same suite (step-16 PD-52). M-8 needs sockets: without them it is INCONCLUSIVE, never a
//! pass.

mod headless;
mod lakeside;
mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use headless::{Scratch, Tables, deterministic, lines};
use lakeside::history::{
    alive, fact_types, facts_of, fishes_and_eats_in_every_bucket, killed_at, of,
};
use lakeside::{
    BINARY, FISH, FISHING_ONLY, Lake, SEED, disabled_copy, edit, entity_pack, lakeside, listing,
    located, mineworld, on_world, read_world, repository, requirements, root_dirs, run,
    scratch_lakeside, text,
};
use mineworld_contracts::{ActionResult, EntityId, EventEnvelope};
use mineworld_worldpack::catalog::AVAILABLE;
use serde_json::{Value, json};
use support::{Client, Server};

// ── M-1 … M-4: the composition, and its misuse refused ─────────────────────────────────────────────

/// M-1 (EE-1): every pack listed with its identity — the third-party System Pack located first, then
/// the world's every requirement, the world itself and every bundled System Pack at the framework's
/// version, each with an SPDX licence; the binary reports the same version.
#[test]
fn every_pack_of_the_composition_is_listed_with_its_identity() {
    let listing = listing();
    let pack = located(&listing);
    println!("{listing}");
    let framework = env!("CARGO_PKG_VERSION");

    let everything: Vec<Vec<String>> = listing
        .lines()
        .filter(|line| line.contains("-pack "))
        .map(|line| line.split_whitespace().map(str::to_owned).collect())
        .collect();
    let line_of = |id: &str| {
        everything
            .iter()
            .find(|words| words[1] == id)
            .unwrap_or_else(|| panic!("{id} is listed: {listing}"))
    };
    for words in &everything {
        mineworld_packages::License::new(&words[3])
            .unwrap_or_else(|error| panic!("{}'s licence: {error}", words[1]));
    }

    let world = lakeside();
    assert_eq!(line_of(read_world(&world).id())[0], "world-pack");
    let entity_pack = entity_pack(&listing);
    let required = requirements(&world);
    let kinds: BTreeMap<&str, Vec<&str>> =
        required.iter().fold(BTreeMap::new(), |mut kinds, (id, _)| {
            kinds
                .entry(line_of(id)[0].as_str())
                .or_insert_with(Vec::new)
                .push(id.as_str());
            kinds
        });
    assert_eq!(
        kinds.get("system-pack"),
        Some(&vec![pack.package.as_str()]),
        "Lakeside requires the located third-party pack: {required:?}"
    );
    assert_eq!(
        kinds.get("entity-pack"),
        Some(&vec![entity_pack.as_str()]),
        "and the repository's own Entity Pack"
    );
    assert_eq!(
        kinds.get("presentation-pack").map(Vec::len),
        Some(2),
        "and two Presentation Packs: {kinds:?}"
    );

    let bundled: Vec<&Vec<String>> = everything
        .iter()
        .filter(|words| words[0] == "system-pack" && words.contains(&"bundled".to_owned()))
        .collect();
    assert_eq!(
        bundled.len(),
        AVAILABLE.iter().filter(|c| c.package().bundled()).count(),
        "every bundled System Pack is listed"
    );
    for words in bundled {
        assert_eq!(
            words[2], framework,
            "{} at the framework's version",
            words[1]
        );
    }
    let (ok, version, err) = mineworld(&["--version"]);
    assert!(ok, "{err}");
    assert_eq!(version.trim(), format!("mineworld {framework}"));
    assert_eq!(framework, "0.1.0", "the framework is 0.1.0 (step-16 PD-40)");
}

/// M-2 (EE-2): `packs resolve` prints the composition — the framework range, which pack satisfies
/// each requirement, and one line per enabled system.
#[test]
fn the_composition_is_resolved_requirement_by_requirement() {
    let listing = listing();
    let pack = located(&listing);
    let entity_pack = entity_pack(&listing);
    let world = lakeside();
    let (ok, out, err) = on_world(&["packs", "resolve"], &world, &[]);
    assert!(ok, "{err}");
    println!("{out}");
    assert!(
        out.lines().any(|line| line.starts_with("  framework  ")
            && line.contains(env!("CARGO_PKG_VERSION"))
            && line.contains("mineworld: \"^0.1\"")),
        "{out}"
    );
    let range = |id: &str| {
        requirements(&world)
            .into_iter()
            .find(|(required, _)| required == id)
            .map(|(_, range)| range)
            .unwrap_or_else(|| panic!("{id} is required"))
    };
    assert!(
        out.contains(&format!(
            "requires   {} \"{}\" → system-pack {} (this build, third-party)",
            pack.package,
            range(&pack.package),
            pack.version
        )),
        "{out}"
    );
    let entity_line = out
        .lines()
        .find(|line| line.contains(&format!("requires   {entity_pack} ")))
        .unwrap_or_else(|| panic!("{entity_pack} resolved: {out}"));
    let directory = Path::new("entities").join(&entity_pack);
    assert!(
        entity_line.contains("→ entity-pack 0.1.0")
            && entity_line.contains(&directory.display().to_string()),
        "{entity_line}"
    );
    assert_eq!(
        out.matches("→ presentation-pack ").count(),
        2,
        "both Presentation Packs resolved: {out}"
    );
    let systems = out
        .lines()
        .filter(|line| line.starts_with("  system "))
        .count();
    assert_eq!(
        systems,
        read_world(&world).systems().len(),
        "one system line per enabled system: {out}"
    );
    assert!(
        out.contains(&format!(
            "system     {} → {} {} (third-party)",
            pack.system, pack.package, pack.version
        )),
        "{out}"
    );
}

/// One misuse: its name, the edits of the scratch copy's `world.yaml` (from, to), whether the entities
/// root is named, and the words the refusal must name.
struct Misuse {
    name: &'static str,
    edits: Vec<(String, String)>,
    entities_root: bool,
    named: Vec<String>,
}

/// `<command> <world> <roots…>`, which must exit 1: its stderr.
fn refused(command: &[&str], world: &Path, entities_root: bool) -> String {
    let path = text(world);
    let roots: Vec<String> = root_dirs()
        .iter()
        .filter(|root| entities_root || !root.ends_with("entities"))
        .flat_map(|root| ["--packs".to_owned(), text(root)])
        .collect();
    let mut arguments: Vec<&str> = command.to_vec();
    arguments.push(&path);
    arguments.extend(roots.iter().map(String::as_str));
    let output = Command::new(BINARY)
        .args(&arguments)
        .env_remove("MINEWORLD_PACKS")
        .output()
        .expect("runs");
    let err = String::from_utf8_lossy(&output.stderr).into_owned();
    assert_eq!(output.status.code(), Some(1), "{command:?}: {err}");
    err
}

/// M-3 (EE-3): six misuses of the composition, each refused by name with exit 1, through `validate`
/// and `packs resolve`.
#[test]
fn each_misuse_of_the_composition_is_refused_by_name() {
    let listing = listing();
    let pack = located(&listing);
    let entity_pack = entity_pack(&listing);
    let required = |id: &str| format!("  {id}: \"^0.1\"\n");
    // A key the Entity Pack declares, read from its own directory.
    let mut files: Vec<PathBuf> =
        std::fs::read_dir(repository("entities").join(&entity_pack).join("items"))
            .expect("the pack's items list")
            .map(|entry| entry.expect("an entry").path())
            .collect();
    files.sort();
    let taken = files[0]
        .file_stem()
        .expect("a stem")
        .to_string_lossy()
        .into_owned();
    let cases = [
        Misuse {
            name: "range",
            edits: vec![(
                required(&pack.package),
                format!("  {}: \"^0.2\"\n", pack.package),
            )],
            entities_root: true,
            named: vec![pack.package.clone(), "^0.2".into(), pack.version.clone()],
        },
        Misuse {
            name: "no-entities-root",
            edits: vec![],
            entities_root: false,
            // The roots that were searched: the one named.
            named: vec![
                entity_pack.clone(),
                text(&repository("presentation/mineworld-default")),
            ],
        },
        Misuse {
            name: "one-namespace",
            edits: vec![("  - fish\n".into(), format!("  - fish\n  - {taken}\n"))],
            entities_root: true,
            named: vec![
                format!("'{taken}'"),
                "world.yaml's items".into(),
                entity_pack.clone(),
                // The pack's file; its directory separator is the platform's.
                format!("{taken}.yaml"),
            ],
        },
        Misuse {
            name: "licence",
            edits: vec![(
                "  license: MIT\n".into(),
                "  license: GPL-3.0-only\n".into(),
            )],
            entities_root: true,
            named: vec!["GPL-3.0-only".into(), "MIT".into(), "Apache-2.0".into()],
        },
        Misuse {
            name: "framework",
            edits: vec![("mineworld: \"^0.1\"\n".into(), "mineworld: \"^9\"\n".into())],
            entities_root: true,
            named: vec!["^9".into(), "0.1.0".into()],
        },
        Misuse {
            name: "unrequired",
            edits: vec![(required(&pack.package), String::new())],
            entities_root: true,
            named: vec![pack.system.clone(), pack.package.clone(), "requires".into()],
        },
    ];
    for case in cases {
        let world = scratch_lakeside(case.name);
        for (from, to) in &case.edits {
            edit(&world.join("world.yaml"), from, to);
        }
        if case.name == "one-namespace" {
            std::fs::copy(&files[0], world.join("items").join(format!("{taken}.yaml")))
                .expect("the pack's file copied into the world");
        }
        for command in [&["validate"][..], &["packs", "resolve"][..]] {
            let err = refused(command, &world, case.entities_root);
            println!("{} {command:?}: {}", case.name, err.trim());
            for word in &case.named {
                assert!(
                    err.contains(word.as_str()),
                    "{} {command:?}: no {word:?} in {err}",
                    case.name
                );
            }
        }
    }
}

/// The binary's length and modification time.
fn stamp() -> (u64, std::time::SystemTime) {
    let binary = std::fs::metadata(BINARY).expect("the binary");
    (
        binary.len(),
        binary.modified().expect("a modification time"),
    )
}

/// M-4 (EE-4, I-E9): a new Entity Pack, written into a fresh root after the binary was built, is
/// listed, required by a copy of Lakeside, resolved and run with — and the binary's length and
/// modification time are unchanged. Nothing here spawns `cargo`. `CARGO_BIN_EXE_mineworld` is the
/// `.exe` on Windows.
#[test]
fn a_new_entity_pack_joins_lakeside_without_a_rebuild() {
    let before = stamp();
    let root = mineworld_test_support::scratch!(empty "milestone-e-fresh-root");
    let pack = root.join("lake-provisions");
    std::fs::create_dir_all(pack.join("items")).expect("the pack's directory");
    std::fs::write(
        pack.join("pack.yaml"),
        "id: lake-provisions\ntype: entity-pack\nversion: 0.1.0\nmineworld: \"^0.1\"\n\
         license: MIT\nauthors: [Someone]\n",
    )
    .expect("pack.yaml");
    std::fs::write(
        pack.join("items").join("smoked-fish.yaml"),
        "# Fish smoked to keep.\nitem: { category: food }\n",
    )
    .expect("a kind");
    let root = text(&root);

    let (ok, out, err) = mineworld(&["packs", "list", "--packs", &root]);
    assert!(ok, "{err}");
    assert!(
        out.lines()
            .any(|line| line.starts_with("entity-pack") && line.contains(" lake-provisions ")),
        "{out}"
    );
    let world = scratch_lakeside("requiring-fresh-root");
    edit(
        &world.join("world.yaml"),
        "requires:\n",
        "requires:\n  lake-provisions: \"^0.1\"\n",
    );
    let (ok, out, err) = on_world(&["validate"], &world, &["--packs", &root]);
    assert!(ok, "{err}");
    assert!(
        out.lines()
            .any(|line| line.starts_with("  items ") && line.contains("smoked-fish")),
        "the new pack's kind is in the world: {out}"
    );
    let (ok, out, err) = on_world(&["packs", "resolve"], &world, &["--packs", &root]);
    assert!(
        ok && out.contains("lake-provisions \"^0.1\" → entity-pack 0.1.0"),
        "{err}{out}"
    );
    let one_day = ["--headless", "--seed", "7", "--days", "1", "--packs", &root];
    let (ok, out, err) = on_world(&["run"], &world, &one_day);
    assert!(ok, "{err}");
    assert!(out.lines().any(|line| line == "faults     0"), "{out}");
    assert_eq!(stamp(), before, "the binary was not rebuilt");
}

// ── M-5 … M-7: three hundred days, determinism, SIGKILL ────────────────────────────────────────────

/// M-5 … M-7 (EE-5 … EE-7): Lakeside lives 300 days; the same seed is the same world, across SIGKILL
/// and resume; inspect and replay pass. Five runs in parallel, as `market_town.rs`.
#[test]
fn lakeside_lives_three_hundred_days_then_the_same_seed_is_the_same_world() {
    let world = lakeside();
    let lake = Lake::read(&world);
    lake.every_routine_reaches_water(&world);
    let entity_pack = entity_pack(&listing());
    let fresh = headless::fresh;
    let long = fresh("lakeside-300");
    let (control, twin, killed, other): (Scratch, Scratch, Scratch, Scratch) = (
        fresh("lakeside-30-control"),
        fresh("lakeside-30-twin"),
        fresh("lakeside-30-killed"),
        fresh("lakeside-30-seed-8"),
    );
    let started = std::time::Instant::now();
    let (long_printed, control_printed, twin_printed, (head_at_kill, resumed), other_printed) =
        std::thread::scope(|scope| {
            let long = scope.spawn(|| run(&world, SEED, 300, Some(&long)));
            let control = scope.spawn(|| run(&world, SEED, 30, Some(&control)));
            let twin = scope.spawn(|| run(&world, SEED, 30, Some(&twin)));
            let killed = scope.spawn(|| {
                killed_at(&killed, 15);
                let head = Tables::read(&killed).journal.len();
                (head, run(&world, SEED, 30, Some(&killed)))
            });
            let other = scope.spawn(|| run(&world, 8, 30, Some(&other)));
            (
                long.join().expect("the 300-day run"),
                control.join().expect("the control"),
                twin.join().expect("the twin"),
                killed.join().expect("the killed run"),
                other.join().expect("seed 8"),
            )
        });
    eprintln!(
        "a 300-day and four 30-day Lakeside runs in parallel: {:.1} s wall; the 300-day save {} bytes",
        started.elapsed().as_secs_f64(),
        std::fs::metadata(long.join("world.sqlite")).map_or(0, |m| m.len())
    );

    // ── M-5: activity first. Nothing is compared before every compared run lives. ───────────────
    let buckets = alive(&lake, &long_printed, "300 days, seed 7");
    assert_eq!(buckets, 10, "ten 30-day buckets");
    assert_eq!(lines(&long_printed, "day ").len(), 300, "a line per day");
    let long_facts = facts_of(&long);
    let history = lines(&long_printed, "history ");
    assert!(
        history.len() == 1
            && history[0].starts_with(&format!("history    {} facts", long_facts.len())),
        "the save holds every fact the run counted: {history:?}"
    );
    assert_eq!(
        fishes_and_eats_in_every_bucket(&lake, &long_facts, &entity_pack),
        buckets,
        "the facts and the activity table read the same buckets"
    );
    drop(long_facts);
    for (printed, save, what) in [
        (&control_printed, &control, "30 days, the control"),
        (&twin_printed, &twin, "30 days, the twin"),
        (&resumed, &killed, "30 days, killed and re-run"),
        (&other_printed, &other, "30 days, seed 8"),
    ] {
        alive(&lake, printed, what);
        fishes_and_eats_in_every_bucket(&lake, &facts_of(save), &entity_pack);
    }

    // ── M-6: only now, byte for byte. ────────────────────────────────────────────────────────────
    let control_tables = Tables::read(&control);
    control_tables.assert_same_history(&Tables::read(&twin), "two 30-day runs of seed 7");
    assert_eq!(
        deterministic(&control_printed),
        deterministic(&twin_printed),
        "and they print the same"
    );
    let header = resumed.lines().next().expect("a header");
    assert!(
        header.contains("resumed") && header.contains(&format!("at revision {head_at_kill}")),
        "the re-run resumed the save on disk at the head the kill left: {header}"
    );
    assert!(
        head_at_kill < control_tables.journal.len(),
        "killed short of the end ({head_at_kill} of {})",
        control_tables.journal.len()
    );
    Tables::read(&killed).assert_same_history(&control_tables, "killed after day 15 and re-run");
    let (control_facts, other_facts) = (facts_of(&control), facts_of(&other));
    let differs_at = control_facts
        .iter()
        .zip(&other_facts)
        .position(|(seven, eight)| seven != eight)
        .expect("seed 8's world differs from seed 7's");
    eprintln!(
        "seed 7 and seed 8 first differ at fact #{}: {} vs {}",
        control_facts[differs_at].id().raw(),
        control_facts[differs_at].event_type().as_str(),
        other_facts[differs_at].event_type().as_str()
    );

    // ── M-7: every cause resolves, the located system is in the save, and replay agrees. ────────
    let pack = located(&listing());
    let (ok, report, err) = mineworld(&["inspect", &text(&long), "--last", "0"]);
    assert!(ok, "inspect: {err}");
    let systems = lines(&report, "systems ");
    assert!(
        systems.len() == 1 && systems[0].split([' ', ',']).any(|word| word == pack.system),
        "inspect names {}: {report}",
        pack.system
    );
    let (ok, _, err) = on_world(&["replay"], &world, &["--save", &text(&killed)]);
    assert!(ok, "replay of the resumed save: {err}");
    eprintln!(
        "Lakeside's baseline: {}; {} deterministic lines",
        history[0],
        deterministic(&long_printed).len()
    );
}

// ── M-8, M-9: the pack through the server, and the pack disabled ──────────────────────────────────

/// M-9 (a) (EE-9): with the pack disabled, Lakeside validates, resolves and lives 30 days, and its fact
/// types are the enabled world's minus exactly the four only the catch produces — nothing else appears
/// or vanishes. Its own enabled control, so that no test depends on another's state.
#[test]
fn disabling_the_pack_takes_away_only_its_own_facts() {
    let world = lakeside();
    let lake = Lake::read(&world);
    let disabled = disabled_copy("disabled-run", &lake);
    for command in [&["validate"][..], &["packs", "resolve"][..]] {
        let (ok, _, err) = on_world(command, &disabled, &[]);
        assert!(ok, "{command:?} of the disabled copy: {err}");
    }
    let (enabled, without) = std::thread::scope(|scope| {
        let enabled = scope.spawn(|| run(&world, SEED, 30, None));
        let without = scope.spawn(|| run(&disabled, SEED, 30, None));
        (
            enabled.join().expect("enabled"),
            without.join().expect("disabled"),
        )
    });
    alive(&lake, &enabled, "30 days, the pack enabled");
    alive(&lake, &without, "30 days, the pack disabled");
    let (enabled, without) = (fact_types(&enabled), fact_types(&without));
    for only in FISHING_ONLY {
        assert!(
            enabled.contains(only),
            "located first: the enabled run has {only}"
        );
    }
    let expected: BTreeSet<String> = enabled
        .iter()
        .filter(|kind| !FISHING_ONLY.contains(&kind.as_str()))
        .cloned()
        .collect();
    println!("enabled: {enabled:?}\ndisabled: {without:?}");
    assert_eq!(
        without, expected,
        "the disabled world's fact types are the enabled world's minus the pack's"
    );
}

/// A complete affordance as the request it names, unchanged (`server/PROTOCOL.md` §6).
fn unchanged(actor: EntityId, offer: &mineworld_contracts::Affordance<Value>) -> Value {
    json!({
        "actor": actor,
        "action_type": offer.action_type().as_str(),
        "target": offer.target(),
        "payload": { "action_type": offer.action_type().as_str(), "payload": offer.payload() },
        "actor_location": null,
    })
}

/// `mineworld server <world> --save <save> <roots…>`, through the test harness.
async fn host(world: &Path, save: &Path) -> Server {
    let (world, save) = (text(world), text(save));
    let roots = lakeside::roots();
    let mut arguments = vec!["server", &world, "--save", &save];
    arguments.extend(roots.iter().map(String::as_str));
    Server::start(&arguments).await
}

/// M-8 (EE-8) and M-9 (b): the seat that begins at the water is offered `fish` through the unchanged
/// server, and its request is accepted with the pack's `fishing-started`; in the disabled copy the same
/// seat is offered no `fish`, and the same request is answered `Unavailable` — the kernel's answer to
/// an action no enabled system handles (`INV-10`).
#[tokio::test(flavor = "multi_thread")]
async fn the_pack_reaches_a_player_through_the_server_and_not_once_disabled() {
    assert!(
        std::env::var_os(mineworld_packages::PACKS_VARIABLE).is_none(),
        "the server harness passes the environment on: unset {} to run this test (step-16 PD-46)",
        mineworld_packages::PACKS_VARIABLE
    );
    let pack = located(&listing());
    let world = lakeside();
    let lake = Lake::read(&world);
    let seat = lake.on_the_water.clone();
    let saves = mineworld_test_support::scratch!(empty "milestone-e-server");
    let (enabled_save, disabled_save) = (saves.join("enabled"), saves.join("disabled"));
    let disabled = disabled_copy("disabled-server", &lake);

    // ── M-8: the pack reaches a player. ──────────────────────────────────────────────────────────
    let server = host(&world, &enabled_save).await;
    let mut client = Client::connect(server.address).await;
    let (observer, _) = client.join(&seat).await;
    let seen = client.observation().await;
    let fish = seen
        .affordances()
        .iter()
        .find(|offer| {
            offer.action_type().as_str() == FISH
                && offer.is_available()
                && offer.payload().is_some()
        })
        .cloned()
        .unwrap_or_else(|| panic!("{seat}, at the water, is offered {FISH}"));
    let request = unchanged(observer, &fish);
    let (_, events) = client.submit_accepted(request.clone()).await;
    drop(client);
    drop(server);
    let started: Vec<EventEnvelope> = facts_of(&enabled_save)
        .into_iter()
        .filter(|fact| events.contains(&fact.id()))
        .filter(|fact| of(fact, "fishing-started"))
        .collect();
    assert_eq!(
        started.len(),
        1,
        "the accepted request caused the pack's fishing-started"
    );
    eprintln!("{seat} fished through the server: {request}");

    // ── M-9 (b): disabled, the same seat is offered no fish, and the request is unavailable. ─────
    let server = host(&disabled, &disabled_save).await;
    let mut client = Client::connect(server.address).await;
    let (again, _) = client.join(&seat).await;
    assert_eq!(again, observer, "the same seat, the same person");
    let seen = client.observation().await;
    let offered: Vec<&str> = seen
        .affordances()
        .iter()
        .map(|offer| offer.action_type().as_str())
        .collect();
    assert!(
        !offered.is_empty(),
        "located first: {seat} is offered something"
    );
    assert!(
        !offered.contains(&FISH),
        "nobody is offered {FISH} once {} is disabled: {offered:?}",
        pack.system
    );
    let (_, result) = client.submit(request).await;
    assert_eq!(result, ActionResult::Unavailable, "{result:?}");
}
