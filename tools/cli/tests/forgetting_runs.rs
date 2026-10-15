//! FA-2 and FA-6 — forgetting through the real binary: saves, resume and replay
//! (`pr-fg-a-relationship-forgetting.md` §6; `docs/DECISIONS.md` `ARC-86`).
//!
//! Scratch copies of `worlds/social-cafe`, each run 30 days at seed 7 with `--save`; one unconfigured
//! twin is shared by both claims, so the file holds one `#[test]`.
//!
//! ```text
//! FA-2  relationships and conversation configured with their defaults written out
//!       ({ extends: default, … none, none, 150 }, { forget_after_days: 0 }): the facts equal the
//!       twin's but for exactly two genesis configuration facts, every later id offset by 2, and the
//!       request outcomes are equal
//! FA-6  half-life over 2 days, linear regard over 30, known_at_most 5:
//!   (a) every acquaintances entry count in the final snapshot ≤ 5        M-FA6 (eviction skipped)
//!   (b) at least one acquaintance-forgotten (else the run proves nothing: INCONCLUSIVE)
//!   (c) `mineworld replay` re-executes the save from genesis
//!   (d) --days 15 then --days 30 on one save: facts byte-equal to the uninterrupted save
//!   (e) the fact types are the twin's plus acquaintance-forgotten and the configured fact — no decay
//!       fact
//!   (f) relationship-changed counts differ from the twin's (decay observable; else INCONCLUSIVE)
//! ```

mod headless;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use headless::{PACK, Scratch, Tables, fresh, lines, mineworld, stderr, stdout};
use mineworld_contracts::EventEnvelope;
use mineworld_persistence::{WorldRevision, format};
use mineworld_relationships::Acquaintances;
use serde_json::Value;

const EXPLICIT_DEFAULT: [(&str, &str); 2] = [
    (
        "relationships",
        "extends: default\nparameters:\n  - { familiarity_decay: none, regard_decay: none, \
         known_at_most: 150 }\n",
    ),
    (
        "conversation",
        "extends: default\nparameters:\n  - { forget_after_days: 0 }\n",
    ),
];

const FORGETFUL: [(&str, &str); 1] = [(
    "relationships",
    "parameters:\n  - { familiarity_decay: half-life, familiarity_decay_days: 2, regard_decay: linear, \
     regard_decay_days: 30, known_at_most: 5 }\n",
)];

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

/// A scratch social-cafe configuring `sections` (none: the twin), and where its save goes.
struct Copy {
    scratch: Scratch,
}

impl Copy {
    fn new(name: &str, sections: &[(&str, &str)]) -> Self {
        let scratch = fresh(name);
        let world = scratch.join("social-cafe");
        copy_dir(Path::new(PACK), &world);
        if !sections.is_empty() {
            let manifest = world.join("world.yaml");
            let original = std::fs::read_to_string(&manifest).expect("reads");
            let listed: String = sections
                .iter()
                .map(|(key, _)| format!("  - {key}\n"))
                .collect();
            std::fs::write(&manifest, format!("{original}\nconfigure:\n{listed}")).expect("writes");
            std::fs::create_dir_all(world.join("configure")).expect("writable");
            for (key, text) in sections {
                std::fs::write(world.join(format!("configure/{key}.yaml")), text).expect("writes");
            }
        }
        Self { scratch }
    }

    fn world(&self) -> PathBuf {
        self.scratch.join("social-cafe")
    }

    fn save(&self, name: &str) -> PathBuf {
        self.scratch.join(name)
    }

    /// `mineworld run` for `days` into the save `name`; its stdout.
    fn run(&self, days: u64, name: &str) -> String {
        let (world, save) = (self.world(), self.save(name));
        let days = days.to_string();
        let output = mineworld(&[
            "run",
            world.to_str().expect("a path"),
            "--headless",
            "--seed",
            "7",
            "--days",
            &days,
            "--save",
            save.to_str().expect("a path"),
        ]);
        assert!(output.status.success(), "run: {}", stderr(&output));
        stdout(&output)
    }
}

fn facts(save: &Path) -> Vec<EventEnvelope> {
    Tables::read(save)
        .facts
        .iter()
        .map(|(_, bytes)| format::decode(bytes, "fact").expect("a fact"))
        .collect()
}

fn fact_json(save: &Path) -> Vec<Value> {
    Tables::read(save)
        .facts
        .iter()
        .map(|(_, bytes)| serde_json::from_slice(bytes).expect("a fact is JSON"))
        .collect()
}

fn counts(facts: &[EventEnvelope]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for fact in facts {
        *counts
            .entry(fact.event_type().as_str().to_owned())
            .or_default() += 1;
    }
    counts
}

/// An id field as the save writes it (a decimal string), moved down by `by` when at or past `from`.
fn shift(value: &mut Value, from: u64, by: u64) {
    let id: u64 = value
        .as_str()
        .and_then(|text| text.parse().ok())
        .expect("an id");
    if id >= from {
        *value = Value::String((id - by).to_string());
    }
}

/// FA-2: the configured copy's facts are the twin's, plus exactly the two configuration facts.
fn explicit_defaults_change_nothing_but_two_genesis_facts(twin: &Path, twin_printed: &str) {
    let copy = Copy::new("fa2-explicit", &EXPLICIT_DEFAULT);
    let printed = copy.run(30, "save");
    let save = copy.save("save");
    let (theirs, mut ours) = (fact_json(twin), fact_json(&save));
    assert_eq!(ours.len(), theirs.len() + 2);
    let configured: Vec<usize> = ours
        .iter()
        .enumerate()
        .filter(|(_, fact)| {
            fact["event_type"]
                .as_str()
                .is_some_and(|kind| kind.ends_with("-interactions-configured"))
        })
        .map(|(index, _)| index)
        .collect();
    assert_eq!(configured.len(), 2, "exactly two configuration facts");
    let first = ours[configured[0]]["id"]
        .as_str()
        .and_then(|id| id.parse::<u64>().ok())
        .expect("an id");
    let genesis_at = ours[0]["at"].clone();
    for index in configured.iter().rev() {
        let removed = ours.remove(*index);
        assert_eq!(removed["at"], genesis_at, "a genesis fact");
    }
    for fact in &mut ours {
        shift(&mut fact["id"], first, 2);
        if let Some(cause) = fact["caused_by"].get_mut("event") {
            shift(cause, first, 2);
        }
    }
    assert!(ours == theirs, "every other fact equal, ids offset by 2");
    assert_eq!(
        lines(&printed, "requests "),
        lines(twin_printed, "requests ")
    );
    assert!(!lines(&printed, "requests ").is_empty());
}

/// FA-6 (a): the largest acquaintance list in the save's last snapshot.
fn widest_list_in_the_last_snapshot(save: &Path) -> usize {
    let tables = Tables::read(save);
    let (revision, stored) = tables.snapshots.last().expect("a snapshot");
    let snapshot = format::decode_snapshot(stored, WorldRevision::from_raw(*revision))
        .expect("the snapshot decodes");
    snapshot
        .components
        .iter()
        .filter(|record| record.component_type().as_str() == "acquaintances")
        .map(|record| {
            serde_json::from_slice::<Acquaintances>(record.payload())
                .expect("decodes")
                .len()
        })
        .max()
        .expect("somebody knows somebody")
}

#[test]
fn forgetting_through_the_binary_saves_resumes_and_replays() {
    let twin = Copy::new("fa-twin", &[]);
    let twin_printed = twin.run(30, "save");
    let twin_facts = facts(&twin.save("save"));

    explicit_defaults_change_nothing_but_two_genesis_facts(&twin.save("save"), &twin_printed);

    let forgetful = Copy::new("fa6-forgetful", &FORGETFUL);
    forgetful.run(30, "save");
    let save = forgetful.save("save");
    let ours = facts(&save);
    let (our_counts, their_counts) = (counts(&ours), counts(&twin_facts));

    // (a) and (b)
    assert!(widest_list_in_the_last_snapshot(&save) <= 5);
    assert!(
        our_counts
            .get("acquaintance-forgotten")
            .copied()
            .unwrap_or(0)
            >= 1,
        "INCONCLUSIVE: no acquaintance was forgotten, so the cap was never exercised"
    );
    // (c)
    let world = forgetful.world();
    let replay = mineworld(&[
        "replay",
        world.to_str().expect("a path"),
        "--save",
        save.to_str().expect("a path"),
    ]);
    assert!(replay.status.success(), "replay: {}", stderr(&replay));
    // (d)
    forgetful.run(15, "continued");
    forgetful.run(30, "continued");
    assert!(
        Tables::read(&forgetful.save("continued")).facts == Tables::read(&save).facts,
        "a resumed run states the same facts, byte for byte"
    );
    // (e)
    let ours_types: BTreeSet<&str> = our_counts.keys().map(String::as_str).collect();
    let mut expected: BTreeSet<&str> = their_counts.keys().map(String::as_str).collect();
    expected.extend([
        "acquaintance-forgotten",
        "relationships-interactions-configured",
    ]);
    assert_eq!(ours_types, expected, "no fact for decay itself");
    // (f)
    assert_ne!(
        our_counts.get("relationship-changed"),
        their_counts.get("relationship-changed"),
        "INCONCLUSIVE: decay changed no level crossing"
    );
    println!(
        "FA-6: acquaintance-forgotten {}, relationship-changed {} (twin {}), became-acquainted {} \
         (twin {})",
        our_counts["acquaintance-forgotten"],
        our_counts.get("relationship-changed").copied().unwrap_or(0),
        their_counts
            .get("relationship-changed")
            .copied()
            .unwrap_or(0),
        our_counts.get("became-acquainted").copied().unwrap_or(0),
        their_counts.get("became-acquainted").copied().unwrap_or(0),
    );
}
