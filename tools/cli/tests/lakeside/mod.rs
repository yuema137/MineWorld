//! Lakeside, Milestone E's world, as `milestone_e.rs` reads it: the commands that name its pack roots,
//! the packs located from `packs list`, what its own files say (seats, water, kinds), and scratch copies
//! of it (step-16 §18; `docs/DECISIONS.md` `ARC-77`).
//!
//! Nothing here names a pack's identity: the third-party pack is located from `packs list` and the
//! installed set, the Entity Pack from `entities/`, and the seats, water and kinds from the world's own
//! files and its loaded identities (`ARC-23`; step-16 I-E5).

pub mod history;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use mineworld_contracts::{EntityId, EntityKey, WorldTime};
use mineworld_packages::PackRoots;
use mineworld_worldpack::WorldPack;
use mineworld_worldpack::catalog::AVAILABLE;
use serde_json::Value;

use crate::headless::Scratch;

const REPOSITORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
pub const BINARY: &str = env!("CARGO_BIN_EXE_mineworld");
pub const SEED: u64 = 7;

/// The fishing pack's action, and the section a place carries to have water to fish (`ARC-66`): slugs
/// read as strings, never through the pack's crate (EC-5).
pub const FISH: &str = "fish";
pub const SECTION: &str = "fishing:";

/// The facts only the fishing pack's catch produces in Lakeside: the pack's own three, and
/// inventory's `items-produced`, whose only producer here is the catch (step-16 PD-49).
pub const FISHING_ONLY: [&str; 4] = [
    "fishing-spot",
    "fishing-started",
    "fishing-ended",
    "items-produced",
];

pub fn repository(relative: &str) -> PathBuf {
    relative
        .split('/')
        .fold(PathBuf::from(REPOSITORY), |path, part| path.join(part))
}

pub fn lakeside() -> PathBuf {
    repository("worlds/lakeside")
}

pub fn text(path: &Path) -> String {
    path.to_str().expect("a UTF-8 path").to_owned()
}

/// The pack roots every Lakeside command names (`ARC-77`).
pub fn root_dirs() -> Vec<PathBuf> {
    vec![
        repository("entities"),
        repository("presentation/mineworld-default"),
    ]
}

pub fn roots() -> Vec<String> {
    root_dirs()
        .iter()
        .flat_map(|root| ["--packs".to_owned(), text(root)])
        .collect()
}

/// `mineworld <arguments>` with `MINEWORLD_PACKS` removed: (success, stdout, stderr).
pub fn mineworld(arguments: &[&str]) -> (bool, String, String) {
    let output = Command::new(BINARY)
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

/// `mineworld <command> <world> <arguments…> <roots…>`.
pub fn on_world(command: &[&str], world: &Path, arguments: &[&str]) -> (bool, String, String) {
    let world = text(world);
    let roots = roots();
    let mut all: Vec<&str> = command.to_vec();
    all.push(&world);
    all.extend(arguments);
    all.extend(roots.iter().map(String::as_str));
    mineworld(&all)
}

/// A headless run of `world`, which must succeed: its stdout.
pub fn run(world: &Path, seed: u64, days: u64, save: Option<&Path>) -> String {
    let (seed, days) = (seed.to_string(), days.to_string());
    let mut arguments = vec!["--headless", "--seed", &seed, "--days", &days];
    let save = save.map(text);
    if let Some(save) = &save {
        arguments.extend(["--save", save]);
    }
    let (ok, out, err) = on_world(&["run"], world, &arguments);
    assert!(ok, "run failed: {err}\n{out}");
    out
}

// ── Locating ───────────────────────────────────────────────────────────────────────────────────────

/// The third-party System Pack of this build: (package id, version, system id).
pub struct ThirdParty {
    pub package: String,
    pub version: String,
    pub system: String,
}

/// The `packs list` lines of one pack type, each split into words.
fn listed(listing: &str, kind: &str) -> Vec<Vec<String>> {
    listing
        .lines()
        .filter(|line| line.starts_with(&format!("{kind} ")))
        .map(|line| line.split_whitespace().map(str::to_owned).collect())
        .collect()
}

/// Located from `packs list`, cross-checked against the installed set: exactly one System Pack is
/// third-party, and it is the capability whose `package().bundled()` is false (`third_party.rs`).
pub fn located(listing: &str) -> ThirdParty {
    let found: Vec<ThirdParty> = listed(listing, "system-pack")
        .into_iter()
        .filter_map(|words| match words.as_slice() {
            [_, package, version, .., origin, word, system]
                if origin == "third-party" && word == "system" =>
            {
                Some(ThirdParty {
                    package: package.clone(),
                    version: version.clone(),
                    system: system.clone(),
                })
            }
            _ => None,
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

/// `packs list` over the world root and the data roots: its stdout.
pub fn listing() -> String {
    let worlds = text(&repository("worlds"));
    let roots = roots();
    let mut arguments = vec!["packs", "list", "--packs", &worlds];
    arguments.extend(roots.iter().map(String::as_str));
    let (ok, out, err) = mineworld(&arguments);
    assert!(ok, "packs list: {err}");
    out
}

/// The repository's own Entity Pack: the one entity pack `packs list` finds under `entities/`. Its id.
pub fn entity_pack(listing: &str) -> String {
    let entities = text(&repository("entities"));
    let found: Vec<String> = listed(listing, "entity-pack")
        .into_iter()
        .filter(|words| words.last().is_some_and(|path| path.starts_with(&entities)))
        .map(|words| words[1].clone())
        .collect();
    assert_eq!(found.len(), 1, "one Entity Pack under entities/: {listing}");
    found[0].clone()
}

/// `world.yaml`'s `requires:` entries, (id, range), read literally.
pub fn requirements(world: &Path) -> Vec<(String, String)> {
    read_lf(&world.join("world.yaml"))
        .lines()
        .skip_while(|line| *line != "requires:")
        .skip(1)
        .take_while(|line| line.starts_with("  ") || line.trim().is_empty())
        .filter_map(|line| line.trim().split_once(": "))
        .map(|(id, range)| (id.to_owned(), range.trim_matches('"').to_owned()))
        .collect()
}

/// Lakeside read with its roots, as the loader reads it.
pub fn read_world(world: &Path) -> WorldPack {
    let roots = PackRoots::new(root_dirs(), None).expect("the roots exist");
    WorldPack::read_with(world, &roots).expect("Lakeside reads")
}

// ── What the world's files say ─────────────────────────────────────────────────────────────────────

/// What the tests read from the world's own files and its loaded identities.
pub struct Lake {
    /// Seat key → its entity, as the raw id a payload carries.
    pub seats: BTreeMap<String, String>,
    /// The places with a `fishing:` section, by key.
    pub water: BTreeSet<String>,
    /// The kinds the water yields (`catch:`), as raw ids.
    pub catch: BTreeSet<String>,
    /// Every item kind's raw id → (key, the pack it came from: its `source_pack`).
    pub kinds: BTreeMap<String, (String, String)>,
    /// The seat that begins at the water, the one a client takes in M-8.
    pub on_the_water: String,
}

/// The literal values after `label` on the lines of `file` (`fixture/`'s reader: never a system's code).
fn values_after(file: &Path, label: &str) -> Vec<String> {
    read_lf(file)
        .lines()
        .filter_map(|line| line.split(label).nth(1))
        .map(|rest| {
            rest.split(|c: char| c == ',' || c == '}' || c.is_whitespace())
                .next()
                .unwrap_or_default()
                .to_owned()
        })
        .collect()
}

/// An entity reference in a payload, as the raw id it names: a bare id, or `{ "entity": … }`.
pub fn entity(value: &Value) -> String {
    match value {
        Value::Object(fields) => entity(&fields["entity"]),
        Value::String(raw) => raw.clone(),
        other => other.to_string(),
    }
}

fn raw(id: EntityId) -> String {
    entity(&serde_json::to_value(id).expect("an id serializes"))
}

impl Lake {
    pub fn read(world: &Path) -> Self {
        let pack = read_world(world);
        let loaded = pack.load(WorldTime::EPOCH).expect("Lakeside loads");
        let id = |key: &str| {
            loaded
                .id(&EntityKey::new(key).expect("a key"))
                .unwrap_or_else(|| panic!("{key} resolves"))
        };
        let place_file = |place: &str| world.join("places").join(format!("{place}.yaml"));
        let water: BTreeSet<String> = pack
            .places()
            .keys()
            .map(|key| key.as_str().to_owned())
            .filter(|key| {
                read_lf(&place_file(key))
                    .lines()
                    .any(|line| line.starts_with(SECTION))
            })
            .collect();
        assert!(
            !water.is_empty(),
            "located first: Lakeside has water to fish"
        );
        let catch: BTreeSet<String> = water
            .iter()
            .flat_map(|place| values_after(&place_file(place), "catch: "))
            .map(|key| raw(id(&key)))
            .collect();
        let read = loaded.world().read();
        let kinds = pack
            .items()
            .keys()
            .map(|key| {
                let at = id(key.as_str());
                let source = read
                    .entity(at)
                    .and_then(|entity| entity.metadata())
                    .expect("an authored kind")
                    .source_pack
                    .clone();
                (raw(at), (key.as_str().to_owned(), source))
            })
            .collect();
        let seats = pack
            .seats()
            .iter()
            .map(|seat| (seat.as_str().to_owned(), raw(id(seat.as_str()))))
            .collect();
        let on_the_water = pack
            .people()
            .iter()
            .find(|(key, person)| {
                pack.seats().contains(*key)
                    && person
                        .location
                        .as_ref()
                        .is_some_and(|at| water.contains(at.place.as_str()))
            })
            .map(|(key, _)| key.as_str().to_owned())
            .expect("a seat begins at the water");
        Self {
            seats,
            water,
            catch,
            kinds,
            on_the_water,
        }
    }

    pub fn seat_of(&self, entity: &str) -> Option<&str> {
        self.seats
            .iter()
            .find(|(_, id)| id.as_str() == entity)
            .map(|(seat, _)| seat.as_str())
    }

    /// Structural: every seat's routine puts it at the water for a part of each day — read from its
    /// file, so "every seat that can reach water" is every seat for a reason the test can see.
    pub fn every_routine_reaches_water(&self, world: &Path) {
        for seat in self.seats.keys() {
            let places = values_after(
                &world.join("people").join(format!("{seat}.yaml")),
                "place: ",
            );
            assert!(
                places.iter().any(|place| self.water.contains(place)),
                "{seat}'s routine never reaches {:?}: {places:?}",
                self.water
            );
        }
    }
}

// ── Scratch copies ─────────────────────────────────────────────────────────────────────────────────

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
pub fn read_lf(file: &Path) -> String {
    std::fs::read_to_string(file)
        .expect("reads")
        .replace("\r\n", "\n")
}

/// Replaces exactly one occurrence of `from` in `file`.
pub fn edit(file: &Path, from: &str, to: &str) {
    let text = read_lf(file);
    assert_eq!(
        text.matches(from).count(),
        1,
        "{}: {from:?}",
        file.display()
    );
    std::fs::write(file, text.replacen(from, to, 1)).expect("writes");
}

/// Lakeside copied into its own scratch, as `lakeside/` (a World Pack's directory is its id), removed
/// when the test ends (DEP-29).
pub fn scratch_lakeside(name: &str) -> Scratch {
    let world = mineworld_test_support::scratch!(format!("milestone-e-{name}")).within("lakeside");
    copy_dir(&lakeside(), &world);
    world
}

/// A scratch copy of Lakeside with the pack's system left out of `systems:` and every `fishing:`
/// section removed — its `requires:` entry kept, so the pack is installed and required but not
/// enabled (step-16 PD-49).
pub fn disabled_copy(name: &str, lake: &Lake) -> Scratch {
    let pack = located(&listing());
    let world = scratch_lakeside(name);
    edit(
        &world.join("world.yaml"),
        &format!("  - {}\n", pack.system),
        "",
    );
    for place in &lake.water {
        let file = world.join("places").join(format!("{place}.yaml"));
        let kept: String = read_lf(&file)
            .lines()
            .filter(|line| !line.starts_with(SECTION))
            .map(|line| format!("{line}\n"))
            .collect();
        std::fs::write(file, kept).expect("writes");
    }
    world
}
