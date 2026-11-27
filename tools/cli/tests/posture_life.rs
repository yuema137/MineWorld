//! S34-d1 — **NPCs are seen seated, and nobody bobs** (`pr-s34d-npcs-agents-sit.md` SD-d-1, SD-d-6, §6).
//!
//! ```text
//! worlds     every World Pack under worlds/ whose systems: names posture (found, never listed here),
//!            plus three variants of the first one, generated at test time (variants/mod.rs):
//!            renamed-seats (keys, groups and order rewritten), one-seat, no-posture
//! runs       mineworld run <world> --headless --seed s --save S, for seeds 7 and 11:
//!              default decisions, pace 900          (the world's own decision list)
//!              every seat motivated, pace 900       (--seat-model SEAT=motivated, seats from world.yaml)
//!              default decisions, pace 60           (a hosted-like pace)
//! read       S's facts, decoded with their owners' types (posture/mod.rs)
//! AC-d-1     I-S34-1, I-S34-2 after every request of every run
//! AC-d-2     I-S34-7: no seated person gives way or is shoved
//! measures   hour samples 09:00-18:00 showing somebody seated; spells, their median, bobbing spells
//! ```
//!
//! Two profiles (Q-S34d-6): `posture_life_short` (3 days at pace 900, 1 day at pace 60) runs by default;
//! `posture_life_long` (30 days, 5 at pace 60) is `#[ignore]`d and runs at the milestone or by hand.

mod headless;
mod posture;
mod social;
mod variants;

use std::path::{Path, PathBuf};

use headless::{Tables, fresh, mineworld, stderr, stdout};
use posture::{Ledger, read};
use serde::Deserialize;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const SEEDS: [u64; 2] = [7, 11];
/// The paced run's default pace, in world seconds (`run::PACE`), and a hosted-like one.
const PACE: i64 = 900;
const HOSTED_PACE: i64 = 60;

/// The part of a world.yaml read here: which systems it installs and which seats it offers.
#[derive(Deserialize)]
struct Head {
    systems: Vec<String>,
    #[serde(default)]
    seats: Vec<String>,
}

fn head(world: &Path) -> Head {
    let text = std::fs::read_to_string(world.join("world.yaml")).expect("a world.yaml reads");
    serde_saphyr::from_str(&text).unwrap_or_else(|error| panic!("{}: {error}", world.display()))
}

/// Every World Pack under `worlds/` that installs `posture`, in name order (I-d-1: found, not named).
fn posture_worlds() -> Vec<PathBuf> {
    let mut worlds: Vec<PathBuf> = std::fs::read_dir(format!("{ROOT}/worlds"))
        .expect("worlds/ reads")
        .map(|entry| entry.expect("an entry").path())
        .filter(|path| path.join("world.yaml").is_file())
        .filter(|path| head(path).systems.iter().any(|system| system == "posture"))
        .collect();
    worlds.sort();
    assert!(!worlds.is_empty(), "at least one world installs posture");
    worlds
}

/// How a run is decided.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Model {
    Default,
    Motivated,
}

/// One run: what was measured from its save.
struct Measured {
    label: String,
    pace: i64,
    ledger: Ledger,
    summary: Vec<String>,
}

fn run(label: &str, world: &Path, seed: u64, days: u64, pace: i64, model: Model) -> Measured {
    let save = fresh(&format!("posture-life-{label}-{seed}-{pace}"));
    let pack = world.to_str().expect("a printable path");
    let (entities, presentation) = (
        format!("{ROOT}/entities"),
        format!("{ROOT}/presentation/mineworld-default"),
    );
    let (seed_text, days_text, pace_text) = (seed.to_string(), days.to_string(), pace.to_string());
    let path = save.to_str().expect("a printable path");
    let mut arguments = vec![
        "run",
        pack,
        "--headless",
        "--seed",
        &seed_text,
        "--days",
        &days_text,
        "--pace",
        &pace_text,
        "--save",
        path,
        "--packs",
        &entities,
        "--packs",
        &presentation,
    ];
    let flags: Vec<String> = match model {
        Model::Default => Vec::new(),
        Model::Motivated => head(world)
            .seats
            .iter()
            .map(|seat| format!("{seat}=motivated"))
            .collect(),
    };
    for flag in &flags {
        arguments.extend(["--seat-model", flag.as_str()]);
    }
    let output = mineworld(&arguments);
    assert!(
        output.status.success(),
        "run {label} seed {seed} failed: {}\n{}",
        stderr(&output),
        stdout(&output)
    );
    let tables = Tables::read(&save);
    let reads: Vec<_> = social::facts_of(&tables).iter().map(read).collect();
    let until = reads.last().map_or(0, |read| read.at);
    let summary = headless::deterministic(&stdout(&output))
        .into_iter()
        .map(str::to_owned)
        .collect();
    let model_name = match model {
        Model::Default => "default",
        Model::Motivated => "motivated",
    };
    Measured {
        label: format!("{label} seed {seed} {model_name} pace {pace}"),
        pace,
        ledger: Ledger::of(reads, until),
        summary,
    }
}

impl Measured {
    /// One line of measures, the form §13 records.
    fn report(&self) -> String {
        let ledger = &self.ledger;
        let samples = ledger.samples.len();
        let showing = ledger.showing_seating();
        let requested = ledger.spells.iter().filter(|spell| spell.requested).count();
        format!(
            "{}: samples {samples}, showing seating {showing} ({} %); spells {} (ended by stand {requested}), \
             median {} s = {}.{:02} consults; bobbing {}",
            self.label,
            showing * 100 / samples.max(1),
            ledger.spells.len(),
            ledger.median_spell(),
            ledger.median_spell() / self.pace,
            ledger.median_spell() % self.pace * 100 / self.pace,
            ledger.bobbing(self.pace),
        )
    }

    /// AC-d-1, AC-d-2: no invariant breached after any request.
    fn holds_invariants(&self) {
        assert!(
            self.ledger.breaches.is_empty(),
            "{}: {} breach(es), first {:?}",
            self.label,
            self.ledger.breaches.len(),
            self.ledger.breaches.iter().take(5).collect::<Vec<_>>()
        );
    }
}

/// Every run of one profile: (world label, world path) × seeds × (default 900, motivated 900, default 60).
fn measure(days: u64, hosted_days: u64) -> Vec<Measured> {
    let worlds = posture_worlds();
    let base = worlds[0].to_str().expect("a printable path").to_owned();
    let renamed = variants::renamed_seats(&base);
    let one_seat = variants::one_seat(&base);
    let mut subjects: Vec<(String, PathBuf)> = worlds
        .iter()
        .map(|world| {
            let name = world
                .file_name()
                .expect("named")
                .to_string_lossy()
                .into_owned();
            (name, world.clone())
        })
        .collect();
    subjects.push(("renamed-seats".to_owned(), renamed.path.clone()));
    subjects.push(("one-seat".to_owned(), one_seat.path.clone()));
    let mut measured = Vec::new();
    for (label, world) in &subjects {
        for seed in SEEDS {
            for (pace, model, days) in [
                (PACE, Model::Default, days),
                (PACE, Model::Motivated, days),
                (HOSTED_PACE, Model::Default, hosted_days),
            ] {
                let run = run(label, world, seed, days, pace, model);
                eprintln!("{}", run.report());
                measured.push(run);
            }
        }
    }
    measured
}

fn holds(measured: &[Measured]) {
    for run in measured {
        run.holds_invariants();
    }
}

#[test]
fn posture_life_short() {
    holds(&measure(3, 1));
}

#[test]
#[ignore = "the 30-day set (Q-S34d-6): cargo test -p mineworld-cli --test posture_life -- --ignored"]
fn posture_life_long() {
    holds(&measure(30, 5));
}

/// Where the no-posture variants' summaries recorded on this PR's base are kept (AC-d-6).
const NO_POSTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/posture_life");
const NO_POSTURE_DAYS: u64 = 3;

/// AC-d-6 / I-S34-6: every posture world, with posture and its sit points removed, decides exactly as it
/// did on this PR's base: its run summary equals the one recorded there (`MINEWORLD_WRITE_NO_POSTURE=1`
/// writes it, on the base only; S34-d1 §13).
#[test]
fn a_world_without_posture_decides_as_before() {
    let write = std::env::var("MINEWORLD_WRITE_NO_POSTURE").as_deref() == Ok("1");
    for world in posture_worlds() {
        let name = world
            .file_name()
            .expect("named")
            .to_string_lossy()
            .into_owned();
        let variant = variants::no_posture(world.to_str().expect("a printable path"));
        let summary = run(
            "no-posture",
            &variant.path,
            SEEDS[0],
            NO_POSTURE_DAYS,
            PACE,
            Model::Default,
        )
        .summary
        .join("\n");
        let fixture = Path::new(NO_POSTURE).join(format!(
            "{name}-seed-{}-{NO_POSTURE_DAYS}-days.txt",
            SEEDS[0]
        ));
        if write {
            std::fs::create_dir_all(NO_POSTURE).expect("writable");
            std::fs::write(&fixture, format!("{summary}\n")).expect("writable");
            continue;
        }
        let recorded = std::fs::read_to_string(&fixture)
            .unwrap_or_else(|error| panic!("{}: {error}", fixture.display()));
        assert_eq!(
            summary,
            recorded.trim_end(),
            "AC-d-6: no-posture {name} moved against its base record"
        );
    }
}
