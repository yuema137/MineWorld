//! Shared by the `worlds/bodies-yard` tests: running the real binary on the pack or a copy of it, and
//! the scan that replays a save's facts and checks, after every request, what the `bodies` pack exists
//! to keep (step-11 PB-9, PB-10; `DECISIONS.md` `ARC-39` note).
//!
//! The scan reads the save, never the code under test: positions are replayed from presence's own
//! facts, decoded with its published types, and the rooms' geometry is the world file's literals,
//! copied here once — [`the_world_file_still_says_the_geometry`] holds the copy to the file.

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mineworld_contracts::{
    ActionId, Causation, EntityId, EntityType, EventEnvelope, EventId, PlaceId, WorldTime,
};
use mineworld_persistence::format;
use mineworld_presence::{Arrived, PersonEnteredPlace, StoppedShort};

use crate::headless::{Tables, mineworld, stderr, stdout};

/// The pack.
pub const YARD: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/bodies-yard");

/// A rectangle as literals: `(min.x, min.y, max.x, max.y)`.
pub type Rect = (i32, i32, i32, i32);

/// Where each person stands, as the scan has replayed it: their place and, if they have one, their
/// ground position.
type Standing = BTreeMap<EntityId, (PlaceId, Option<(i32, i32)>)>;

/// The people with a position in each place, by key.
type ByPlace = BTreeMap<String, Vec<(String, (i32, i32))>>;

/// The rooms, as `worlds/bodies-yard/places/*.yaml` author them: each place's floor and solids.
pub const ROOMS: [(&str, Rect, Rect); 2] = [
    (
        "court",
        (0, 0, 10_000, 10_000),
        (4_700, 4_700, 5_300, 5_300),
    ),
    ("hall", (0, 0, 12_000, 9_000), (5_000, 4_000, 7_000, 5_000)),
];

/// The two places' files still say exactly the literals above.
pub fn the_world_file_still_says_the_geometry(pack: &Path) {
    for (place, (x0, y0, x1, y1), (a, b, c, d)) in ROOMS {
        let text = std::fs::read_to_string(pack.join("places").join(format!("{place}.yaml")))
            .expect("a place file reads");
        for line in [
            format!("min: {{ x: {x0}, y: {y0} }}"),
            format!("max: {{ x: {x1}, y: {y1} }}"),
            format!("- {{ min: {{ x: {a}, y: {b} }}, max: {{ x: {c}, y: {d} }}"),
        ] {
            assert!(text.contains(&line), "{place}.yaml still says {line:?}");
        }
    }
}

/// `run` of `pack` with these settings, which must succeed; its stdout.
pub fn run(pack: &Path, seed: u64, days: u64, save: Option<&Path>) -> String {
    let (seed, days) = (seed.to_string(), days.to_string());
    let pack = pack.to_str().expect("a printable path").to_owned();
    let mut arguments = vec!["run", &pack, "--headless", "--seed", &seed, "--days", &days];
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

/// Copies a directory tree.
pub fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a directory");
    for entry in std::fs::read_dir(from).expect("the pack lists") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("a copy");
        }
    }
}

/// A copy of the pack under `directory`, named as the pack (a pack's id is its directory).
pub fn copy_of(directory: &Path) -> PathBuf {
    let _ = std::fs::remove_dir_all(directory);
    let copy = directory.join("bodies-yard");
    copy_dir(Path::new(YARD), &copy);
    copy
}

/// `text` without its top-level `key:` and the indented lines that continue it; whether it had one.
pub fn strip_section(text: &str, key: &str) -> (String, bool) {
    let mut kept = Vec::new();
    let (mut inside, mut found) = (false, false);
    for line in text.lines() {
        if line.starts_with(&format!("{key}:")) {
            inside = true;
            found = true;
            continue;
        }
        if inside && (line.starts_with(' ') || line.starts_with('-')) {
            continue;
        }
        inside = false;
        kept.push(line);
    }
    (kept.join("\n") + "\n", found)
}

/// The counterfactual (step-11 PB-10, `AC-2` at world level): a copy of the pack whose `systems:`
/// list drops `bodies` and whose place files drop their `body:` sections — the market-composition
/// pattern. Asserted to differ from the pack by exactly those.
pub fn without_bodies(directory: &Path) -> PathBuf {
    let copy = copy_of(directory);
    let manifest = copy.join("world.yaml");
    let text = std::fs::read_to_string(&manifest).expect("world.yaml reads");
    assert_eq!(
        text.matches("  - bodies\n").count(),
        1,
        "the yard enables bodies"
    );
    std::fs::write(&manifest, text.replace("  - bodies\n", "")).expect("world.yaml writes");
    for (place, _, _) in ROOMS {
        let file = copy.join("places").join(format!("{place}.yaml"));
        let (kept, found) = strip_section(&std::fs::read_to_string(&file).expect("reads"), "body");
        assert!(found, "{place}.yaml carried a body: section");
        assert!(
            !kept
                .lines()
                .any(|line| line.starts_with("body:") || line.contains("min: {")),
            "{place}.yaml's section is gone"
        );
        std::fs::write(&file, kept).expect("writes");
    }
    copy
}

/// Which request a fact belongs to: the action that caused it, or the action behind the fact that
/// caused it; [`None`] for genesis.
fn request_of(fact: &EventEnvelope, requests: &BTreeMap<EventId, ActionId>) -> Option<ActionId> {
    match fact.caused_by() {
        Causation::Action(id) => Some(*id),
        Causation::Event(cause) => requests.get(cause).copied(),
        _ => None,
    }
}

/// One 10-day bucket's activity: moves per seat, stopped-shorts, displaced arrivals, crossings by
/// direction (from, to).
#[derive(Debug, Default)]
pub struct Bucket {
    pub moves: BTreeMap<String, u64>,
    pub stopped: u64,
    pub displaced: u64,
    pub crossings: BTreeMap<(String, String), u64>,
}

/// What the scan saw.
#[derive(Debug, Default)]
pub struct Report {
    /// The closest pair in one place after any request: distance², the two keys, the place, the
    /// request and its instant.
    pub closest: Option<(i64, String, String, String, Option<ActionId>, WorldTime)>,
    /// Every broken rule, located.
    pub violations: Vec<String>,
    pub buckets: Vec<Bucket>,
    pub requests: u64,
}

impl Report {
    pub fn closest_mm(&self) -> i64 {
        self.closest
            .as_ref()
            .map_or(i64::MAX, |closest| closest.0.isqrt())
    }

    pub fn located(&self) -> String {
        match &self.closest {
            None => "nobody shared a place".to_owned(),
            Some((d, a, b, place, request, at)) => format!(
                "{a} and {b}, {} mm apart in {place}, after request {request:?} at {}s",
                d.isqrt(),
                at.seconds()
            ),
        }
    }
}

/// Replays `tables`' facts in `EventId` order and checks, after each request's facts:
///
/// ```text
/// no pair in one place closer than 595 mm
/// no centre outside the floor shrunk by 295 mm, or within 295 mm of a solid (ROOMS)
/// every displaced arrival — an `arrived` after the first in one request — stays in its place and
///   moves at most 310 mm, at most four per request, caused by the request, stated by movement
/// every stopped-short caused by the request and stated by movement
/// ```
///
/// `keys` names the pack's entities (its loaded ids, which a save's facts carry).
pub fn scan(tables: &Tables, keys: &BTreeMap<EntityId, String>) -> Report {
    let place_of = |entity: EntityId| keys.get(&entity).cloned().unwrap_or_default();
    let mut report = Report::default();
    let mut at: Standing = BTreeMap::new();
    let mut requests: BTreeMap<EventId, ActionId> = BTreeMap::new();
    let mut current: Option<Option<ActionId>> = None;
    let (mut arrivals_in_request, mut last_instant) = (0_u32, WorldTime::EPOCH);
    let facts: Vec<EventEnvelope> = tables
        .facts
        .iter()
        .map(|(_, bytes)| format::decode::<EventEnvelope>(bytes, "fact").expect("a fact"))
        .collect();
    for fact in &facts {
        let request = request_of(fact, &requests);
        if let Some(id) = request {
            requests.insert(fact.id(), id);
        }
        if current.is_some_and(|previous| previous != request) {
            check(&at, &place_of, current.flatten(), last_instant, &mut report);
            arrivals_in_request = 0;
        }
        if current != Some(request) && request.is_some() {
            report.requests += 1;
        }
        current = Some(request);
        last_instant = fact.at();
        let day = usize::try_from(fact.at().seconds() / 86_400).expect("a day");
        let bucket = day / 10;
        while report.buckets.len() <= bucket {
            report.buckets.push(Bucket::default());
        }
        let record = fact.payload();
        let stated = |what: &str, report: &mut Report| {
            if !matches!(fact.caused_by(), Causation::Action(_))
                || fact.provenance().emitted_by().as_str() != "movement"
            {
                report.violations.push(format!(
                    "{what} {:?} is not caused by a request and stated by movement",
                    fact.id()
                ));
            }
        };
        if let Ok(payload) = record.payload_for::<Arrived>() {
            let arrived: Arrived = serde_json::from_slice(payload).expect("presence's encoding");
            let person = arrived.person().entity_id();
            let location = arrived.location();
            let point = location
                .local()
                .map(|local| (local.x().value(), local.y().value()));
            if request.is_some() {
                arrivals_in_request += 1;
                if arrivals_in_request == 1 {
                    *report.buckets[bucket]
                        .moves
                        .entry(place_of(person))
                        .or_default() += 1;
                } else {
                    report.buckets[bucket].displaced += 1;
                    stated("a displaced arrival", &mut report);
                    let before = at.get(&person).copied();
                    match before {
                        Some((place, Some(was))) if place == location.place() => {
                            let moved = distance2(was, point.expect("a position"));
                            if moved > 310 * 310 {
                                report.violations.push(format!(
                                    "{} nudged {} mm at {:?}",
                                    place_of(person),
                                    moved.isqrt(),
                                    fact.id()
                                ));
                            }
                        }
                        _ => report.violations.push(format!(
                            "{} displaced out of their place at {:?}",
                            place_of(person),
                            fact.id()
                        )),
                    }
                    if arrivals_in_request > 5 {
                        report
                            .violations
                            .push(format!("more than four displaced by request {request:?}"));
                    }
                }
            }
            at.insert(person, (location.place(), point));
        } else if record.payload_for::<StoppedShort>().is_ok() {
            report.buckets[bucket].stopped += 1;
            stated("a stopped-short", &mut report);
        } else if let Ok(payload) = record.payload_for::<PersonEnteredPlace>() {
            let entered: PersonEnteredPlace =
                serde_json::from_slice(payload).expect("presence's encoding");
            *report.buckets[bucket]
                .crossings
                .entry((
                    place_of(entered.from().entity_id()),
                    place_of(entered.place().entity_id()),
                ))
                .or_default() += 1;
        }
    }
    check(&at, &place_of, current.flatten(), last_instant, &mut report);
    report
}

fn distance2(a: (i32, i32), b: (i32, i32)) -> i64 {
    let (dx, dy) = (i64::from(a.0 - b.0), i64::from(a.1 - b.1));
    dx * dx + dy * dy
}

/// The state after one request, against the rules.
fn check(
    at: &Standing,
    place_of: &dyn Fn(EntityId) -> String,
    request: Option<ActionId>,
    instant: WorldTime,
    report: &mut Report,
) {
    let mut by_place: ByPlace = BTreeMap::new();
    for (person, (place, point)) in at {
        if let Some(point) = point {
            by_place
                .entry(place_of(place.entity_id()))
                .or_default()
                .push((place_of(*person), *point));
        }
    }
    for (place, people) in &by_place {
        for (i, (a, pa)) in people.iter().enumerate() {
            for (b, pb) in &people[i + 1..] {
                let d = distance2(*pa, *pb);
                if report.closest.as_ref().is_none_or(|closest| d < closest.0) {
                    report.closest =
                        Some((d, a.clone(), b.clone(), place.clone(), request, instant));
                }
                if d < 595 * 595 {
                    report.violations.push(format!(
                        "{a} and {b} {} mm apart in {place} after request {request:?}",
                        d.isqrt()
                    ));
                }
            }
        }
        if let Some((_, (x0, y0, x1, y1), (a, b, c, d))) =
            ROOMS.iter().find(|(name, _, _)| name == place)
        {
            for (person, (x, y)) in people {
                let inside = (x0 + 295..=x1 - 295).contains(x) && (y0 + 295..=y1 - 295).contains(y);
                let dx = (a - x).max(x - c).max(0);
                let dy = (b - y).max(y - d).max(0);
                let clear =
                    i64::from(dx) * i64::from(dx) + i64::from(dy) * i64::from(dy) >= 295 * 295;
                if !inside || !clear {
                    report.violations.push(format!(
                        "{person} at ({x}, {y}) in {place} is outside its floor or in its solid, \
                         after request {request:?}"
                    ));
                }
            }
        }
    }
}

/// Every entity of `pack`, by the key it was authored under, as a save of it names them.
pub fn keys_of(pack: &Path) -> BTreeMap<EntityId, String> {
    let loaded = mineworld_worldpack::WorldPack::read(pack)
        .expect("the pack reads")
        .load(WorldTime::EPOCH)
        .expect("it loads");
    let keys: BTreeMap<EntityId, String> = loaded
        .ids()
        .iter()
        .map(|(key, id)| (*id, key.as_str().to_owned()))
        .collect();
    assert!(
        keys.keys().any(|id| loaded
            .world()
            .read()
            .entity(*id)
            .is_some_and(|entity| entity.entity_type() == EntityType::Place)),
        "the pack's places are named"
    );
    keys
}

/// The scan's verdict on the activity precondition (`ARC-23`: activity first): in every 10-day
/// bucket every seat moved, someone stopped short, someone was nudged, and somebody crossed each way.
/// Prints the counts.
pub fn assert_active(report: &Report, seats: &[&str]) {
    assert!(!report.buckets.is_empty(), "the scan saw days");
    for (index, bucket) in report.buckets.iter().enumerate() {
        println!(
            "days {}-{}: moves {:?}; stopped-short {}; displaced {}; crossings {:?}",
            index * 10 + 1,
            index * 10 + 10,
            bucket.moves,
            bucket.stopped,
            bucket.displaced,
            bucket.crossings
        );
        for seat in seats {
            assert!(
                bucket.moves.get(*seat).copied().unwrap_or(0) > 0,
                "{seat} never moved in bucket {index}"
            );
        }
        assert!(bucket.stopped > 0, "nobody stopped short in bucket {index}");
        assert!(bucket.displaced > 0, "nobody was nudged in bucket {index}");
        for direction in [("hall", "court"), ("court", "hall")] {
            let key = (direction.0.to_owned(), direction.1.to_owned());
            assert!(
                bucket.crossings.get(&key).copied().unwrap_or(0) > 0,
                "nobody crossed {direction:?} in bucket {index}"
            );
        }
    }
}
