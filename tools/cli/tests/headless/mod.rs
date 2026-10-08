//! Shared by the `mineworld run` acceptance tests: running the real binary, reading what it printed,
//! and reading a save's three tables back as the bytes they are.

#![allow(dead_code)]

use std::path::Path;
use std::process::{Command, Output};

pub use mineworld_test_support::Scratch;

use mineworld_persistence::{
    Durability, Manifest, PersistenceBackend, SqliteBackend, WorldRevision, format,
};

pub const PACK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/social-cafe");
pub const BINARY: &str = env!("CARGO_BIN_EXE_mineworld");

/// A path called `name` that does not exist yet, removed when the guard is dropped (DEP-29).
pub fn fresh(name: &str) -> Scratch {
    mineworld_test_support::scratch!(name)
}

pub fn mineworld(arguments: &[&str]) -> Output {
    Command::new(BINARY)
        .args(arguments)
        .output()
        .expect("the mineworld binary runs")
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// `run` with these settings, which must succeed; its stdout.
pub fn run(seed: u64, days: u64, save: Option<&Path>) -> String {
    let (seed, days) = (seed.to_string(), days.to_string());
    let mut arguments = vec!["run", PACK, "--headless", "--seed", &seed, "--days", &days];
    let save = save.map(|save| save.to_str().expect("a printable path").to_owned());
    if let Some(save) = &save {
        arguments.extend(["--save", save]);
    }
    let output = mineworld(&arguments);
    assert!(
        output.status.success(),
        "run failed: {}\n{}",
        stderr(&output),
        stdout(&output)
    );
    stdout(&output)
}

/// The lines that are a function of the pack, the seed and the age: everything but the header (it
/// names where the world is kept) and the `wall` line (real time).
pub fn deterministic(printed: &str) -> Vec<&str> {
    printed
        .lines()
        .filter(|line| !line.starts_with("[mineworld] run ") && !line.starts_with("wall "))
        .collect()
}

/// The summary lines beginning with `prefix`.
pub fn lines<'a>(printed: &'a str, prefix: &str) -> Vec<&'a str> {
    printed
        .lines()
        .filter(|line| line.starts_with(prefix))
        .collect()
}

/// A number printed after `label` on the line starting with `prefix`.
pub fn count_after(line: &str, label: &str) -> u64 {
    let rest = line
        .split(label)
        .nth(1)
        .unwrap_or_else(|| panic!("no '{label}' in: {line}"));
    rest.split_whitespace()
        .next()
        .and_then(|number| number.parse().ok())
        .unwrap_or_else(|| panic!("no number after '{label}' in: {line}"))
}

/// The seats the pack offers, in the order `mineworld run` drives them — read from the pack, so the
/// activity precondition covers every seat a run drives rather than a list a test once copied.
pub fn seats() -> Vec<String> {
    mineworld_worldpack::WorldPack::read(PACK)
        .expect("the repository's own World Pack reads")
        .seats()
        .iter()
        .map(|seat| seat.as_str().to_owned())
        .collect()
}

/// How many times somebody entered each of the pack's places in a save's history, by place key —
/// every place the pack declares is listed, entered or not, so a place nobody reached shows as 0
/// rather than being absent (`ARC-23`: the instrument must be able to see a zero).
///
/// Read from presence's own `person-entered-place` facts, decoded with presence's published type.
pub fn entries_per_place(tables: &Tables) -> std::collections::BTreeMap<String, u64> {
    use mineworld_contracts::{Event, EventEnvelope, WorldTime};
    use mineworld_presence::PersonEnteredPlace;

    let loaded = mineworld_worldpack::WorldPack::read(PACK)
        .expect("the repository's own World Pack reads")
        .load(WorldTime::EPOCH)
        .expect("it loads");
    let places: Vec<(String, mineworld_contracts::EntityId)> = loaded
        .ids()
        .iter()
        .filter(|(_, id)| {
            loaded.world().read().entity(**id).is_some_and(|entity| {
                entity.entity_type() == mineworld_contracts::EntityType::Place
            })
        })
        .map(|(key, id)| (key.as_str().to_owned(), *id))
        .collect();
    let mut entered: std::collections::BTreeMap<String, u64> =
        places.iter().map(|(key, _)| (key.clone(), 0)).collect();
    for (_, bytes) in &tables.facts {
        let fact: EventEnvelope = format::decode(bytes, "fact").expect("a fact");
        if *fact.event_type() != PersonEnteredPlace::EVENT_TYPE {
            continue;
        }
        let payload = fact
            .payload()
            .payload_for::<PersonEnteredPlace>()
            .expect("presence's own fact");
        let entry: PersonEnteredPlace = serde_json::from_slice(payload).expect("it decodes");
        let place = entry.place().entity_id();
        let key = places
            .iter()
            .find(|(_, id)| *id == place)
            .map(|(key, _)| key.clone())
            .expect("an entry into a place the pack declares");
        *entered.entry(key).or_default() += 1;
    }
    entered
}

/// The step-08 I-9 precondition, checked off the printed activity table: every seat accepted a
/// `move` and a `talk` in every 30-day bucket. Returns the number of buckets, so a caller can see it
/// read what it meant to (`ARC-23`).
pub fn every_seat_active_in_every_bucket(printed: &str, seats: &[&str]) -> usize {
    let buckets = lines(printed, "activity ");
    assert!(!buckets.is_empty(), "no activity table: {printed}");
    for bucket in &buckets {
        for seat in seats {
            let moved = count_after(bucket, &format!("  {seat} move "));
            let talked = count_after(bucket, &format!("  {seat} move {moved} talk "));
            assert!(moved > 0 && talked > 0, "{seat} idle in: {bucket}");
        }
    }
    buckets.len()
}

/// A save's three append-only tables and its manifest, as stored.
pub struct Tables {
    pub facts: Vec<(u64, Vec<u8>)>,
    pub journal: Vec<(u64, Vec<u8>)>,
    pub snapshots: Vec<(u64, Vec<u8>)>,
    pub manifest: Manifest,
}

impl Tables {
    pub fn read(save: &Path) -> Self {
        let backend = SqliteBackend::open(save, Durability::ProcessCrash).expect("the save opens");
        let all = usize::try_from(i64::MAX).expect("64-bit");
        let facts = backend
            .last_facts(all)
            .expect("facts")
            .into_iter()
            .map(|fact| (fact.id, fact.bytes))
            .collect();
        let journal = backend
            .journal_after(WorldRevision::from_raw(0))
            .expect("journal")
            .into_iter()
            .map(|(revision, bytes)| (revision.raw(), bytes))
            .collect();
        let snapshots = backend
            .snapshot_revisions()
            .expect("snapshot revisions")
            .into_iter()
            .map(|revision| {
                (
                    revision.raw(),
                    backend
                        .snapshot_at(revision)
                        .expect("a snapshot")
                        .expect("present"),
                )
            })
            .collect();
        let manifest = format::decode(&backend.manifest().expect("manifest").body, "manifest")
            .expect("the manifest decodes");
        Self {
            facts,
            journal,
            snapshots,
            manifest,
        }
    }

    /// The instant of the save's head — its last journaled input — in world seconds: where a hosted
    /// world's clock resumes from (`ARC-25`).
    pub fn head_instant(&self) -> i64 {
        let (_, bytes) = self
            .journal
            .last()
            .expect("a save has at least its genesis");
        let entry: mineworld_persistence::JournalEntry =
            format::decode(bytes, "journal entry").expect("the head decodes");
        entry.input.at().seconds()
    }

    /// Byte-for-byte equality of facts, journal and snapshots — the claim `AC-12` and `AC-6` make —
    /// with the first difference named rather than a bare `false`.
    pub fn assert_same_history(&self, other: &Self, what: &str) {
        for (name, ours, theirs) in [
            ("facts", &self.facts, &other.facts),
            ("journal", &self.journal, &other.journal),
            ("snapshots", &self.snapshots, &other.snapshots),
        ] {
            assert_eq!(ours.len(), theirs.len(), "{what}: {name} row counts differ");
            if let Some((index, _)) = ours
                .iter()
                .zip(theirs.iter())
                .enumerate()
                .find(|(_, (a, b))| a != b)
            {
                panic!(
                    "{what}: {name} differ first at row {index} (key {} vs {})",
                    ours[index].0, theirs[index].0
                );
            }
        }
        assert_eq!(self.manifest.pack, other.manifest.pack, "{what}: pack");
        assert_eq!(
            self.manifest.composition, other.manifest.composition,
            "{what}: composition"
        );
    }
}
