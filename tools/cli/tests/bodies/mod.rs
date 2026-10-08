//! Shared by the `worlds/bodies-yard` tests: running the real binary on the pack or a copy of it, and
//! the scan that replays a save's facts and checks, after every request, what the `bodies` pack exists
//! to keep (step-11 PB-9, PB-10, PO-9, PO-10; `DECISIONS.md` `ARC-39` and its notes).
//!
//! The scan reads the save, never the code under test: positions are replayed from presence's own
//! facts, decoded with its published types; bodies' facts are decoded with this file's own mirrors of
//! their four payloads (this crate does not depend on the pack, step-11 §18.11 DO-12), so the scan is
//! also an independent reading of bodies' encoding; the rooms' geometry is the world files' literals,
//! copied here once — [`the_world_file_still_says_the_geometry`] holds the copy to the files.

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use mineworld_test_support::Scratch;

use mineworld_contracts::{
    ActionId, Causation, EntityId, EntityType, EventEnvelope, EventId, LocalPosition, PlaceId,
    WorldTime,
};
use mineworld_conversation::Spoke;
use mineworld_persistence::format;
use mineworld_presence::{Arrived, PersonEnteredPlace, StoppedShort};
use serde_json::Value;

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

/// The rooms, as `worlds/bodies-yard/places/*.yaml` author them: each place's floor and solid, and
/// the solid's height.
pub const ROOMS: [(&str, Rect, Rect); 2] = [
    (
        "court",
        (0, 0, 10_000, 10_000),
        (4_700, 4_700, 5_300, 5_300),
    ),
    ("hall", (0, 0, 12_000, 9_000), (5_000, 4_000, 7_000, 5_000)),
];

/// Each room's solid's height, as authored.
pub const HEIGHTS: [(&str, i32); 2] = [("court", 3_000), ("hall", 750)];

/// The two places' files still say exactly the literals above.
pub fn the_world_file_still_says_the_geometry(pack: &Path) {
    for (place, (x0, y0, x1, y1), (a, b, c, d)) in ROOMS {
        let text = std::fs::read_to_string(pack.join("places").join(format!("{place}.yaml")))
            .expect("a place file reads");
        let height = HEIGHTS
            .iter()
            .find(|(name, _)| *name == place)
            .expect("a height")
            .1;
        for line in [
            format!("min: {{ x: {x0}, y: {y0} }}"),
            format!("max: {{ x: {x1}, y: {y1} }}"),
            format!(
                "- {{ min: {{ x: {a}, y: {b} }}, max: {{ x: {c}, y: {d} }}, height: {height} }}"
            ),
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

/// A copy of the pack in the scratch `directory`, named as the pack (a pack's id is its directory).
pub fn copy_of(directory: Scratch) -> Scratch {
    let copy = directory.within("bodies-yard");
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

/// The item files of the pack: the loose objects.
pub fn item_files(pack: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(pack.join("items"))
        .expect("items/ lists")
        .map(|entry| entry.expect("an entry").path())
        .collect();
    files.sort();
    files
}

/// The counterfactual (step-11 PB-10, PO-10; `AC-2` at world level): a copy of the pack whose
/// `systems:` list drops `bodies` and whose place and item files drop their `body:` sections — the
/// market-composition pattern. Asserted to differ from the pack by exactly those. The items stay, as
/// inert entities.
pub fn without_bodies(directory: Scratch) -> Scratch {
    let copy = copy_of(directory);
    let manifest = copy.join("world.yaml");
    let text = std::fs::read_to_string(&manifest).expect("world.yaml reads");
    assert_eq!(
        text.matches("  - bodies\n").count(),
        1,
        "the yard enables bodies"
    );
    std::fs::write(&manifest, text.replace("  - bodies\n", "")).expect("world.yaml writes");
    let places = ROOMS
        .iter()
        .map(|(place, _, _)| copy.join("places").join(format!("{place}.yaml")));
    let files: Vec<PathBuf> = places.chain(item_files(&copy)).collect();
    assert_eq!(files.len(), 2 + 16, "two places and sixteen objects");
    for file in files {
        let (kept, found) = strip_section(&std::fs::read_to_string(&file).expect("reads"), "body");
        assert!(found, "{file:?} carried a body: section");
        assert!(
            !kept.lines().any(|line| line.starts_with("body:")
                || line.contains("min: {")
                || line.contains("shape:")),
            "{file:?}'s section is gone"
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

// -------------------------------------------------------------------------------------------------
// bodies' payloads, mirrored (DO-12)
// -------------------------------------------------------------------------------------------------

/// `fact`'s JSON payload, when its type is `kind`.
fn bodies_fact(fact: &EventEnvelope, kind: &str) -> Option<Value> {
    let record = fact.payload();
    (record.event_type().as_str() == kind).then(|| {
        serde_json::from_slice(record.payload())
            .unwrap_or_else(|error| panic!("bodies' {kind} decodes: {error}"))
    })
}

/// A typed reference as bodies encodes one, `{ "entity": "<id>", "entity_type": … }`: its entity.
fn entity(value: &Value) -> EntityId {
    serde_json::from_value(value["entity"].clone())
        .unwrap_or_else(|error| panic!("an entity reference: {value}: {error}"))
}

/// A position as bodies encodes one, `{ "x", "y", "z" }`.
fn position(value: &Value) -> (i32, i32, i32) {
    xyz(serde_json::from_value::<LocalPosition>(value.clone())
        .unwrap_or_else(|error| panic!("a position: {value}: {error}")))
}

/// A shape as bodies encodes one: `{ "box": { x, y, z } }` or `{ "ball": r }` → (box?, half-extents).
fn shape(value: &Value) -> (bool, (i32, i32, i32)) {
    let int = |v: &Value| i32::try_from(v.as_i64().expect("an integer")).expect("fits");
    if let Some(half) = value.get("box") {
        (true, (int(&half["x"]), int(&half["y"]), int(&half["z"])))
    } else {
        let r = int(&value["ball"]);
        (false, (r, r, r))
    }
}

/// An object as the scan knows it: box or ball, half-extents (x, y, z), its place and its centre.
#[derive(Debug, Clone)]
struct Object {
    boxy: bool,
    half: (i32, i32, i32),
    place: Option<EntityId>,
    at: (i32, i32, i32),
    /// Where genesis placed it.
    origin: (i32, i32, i32),
    /// How it last moved: pushed, kicked or thrown.
    last: Option<String>,
}

/// An object as it lies at the end of a run.
#[derive(Debug, Clone)]
pub struct Rested {
    pub key: String,
    pub place: String,
    pub at: (i32, i32, i32),
    /// Whether it ever left where genesis placed it.
    pub moved: bool,
    /// How it last moved, if it did.
    pub last: Option<String>,
    boxy: bool,
    half: (i32, i32, i32),
}

impl Object {
    /// The squared distance from `p` to the footprint (zero inside it).
    fn footprint_distance2(&self, p: (i32, i32)) -> i64 {
        let (x, y, _) = self.at;
        let (hx, hy, _) = self.half;
        if self.boxy {
            to_rect2(p, (x - hx, y - hy, x + hx, y + hy))
        } else {
            let d = distance2(p, (x, y)).isqrt() - i64::from(hx);
            d.max(0).pow(2)
        }
    }
}

fn to_rect2(p: (i32, i32), (x0, y0, x1, y1): Rect) -> i64 {
    let dx = i64::from((x0 - p.0).max(p.0 - x1).max(0));
    let dy = i64::from((y0 - p.1).max(p.1 - y1).max(0));
    dx * dx + dy * dy
}

// -------------------------------------------------------------------------------------------------
// the scan
// -------------------------------------------------------------------------------------------------

/// One 10-day bucket's activity.
#[derive(Debug, Default)]
pub struct Bucket {
    /// Moves per seat: the first arrival of a request that is not a shove.
    pub moves: BTreeMap<String, u64>,
    /// Talks per seat: `spoke` facts by speaker.
    pub talks: BTreeMap<String, u64>,
    pub stopped: u64,
    /// Stopped-shorts whose `by` is a loose object.
    pub stopped_by_object: u64,
    pub displaced: u64,
    pub crossings: BTreeMap<(String, String), u64>,
    pub kicks: u64,
    pub throws: u64,
    pub shoves: u64,
    pub pushes: u64,
    /// Kicks, throws and shoves per seat (AO-3).
    pub physical: BTreeMap<String, u64>,
}

/// What the scan saw.
#[derive(Debug, Default)]
pub struct Report {
    /// The closest pair in one place after any request: distance², the two keys, the place, the
    /// request and its instant.
    pub closest: Option<(i64, String, String, String, Option<ActionId>, WorldTime)>,
    /// The closest person to an object's footprint after any request, in the same form.
    pub closest_object: Option<(i64, String, String, String, Option<ActionId>, WorldTime)>,
    /// Every broken rule, located.
    pub violations: Vec<String>,
    pub buckets: Vec<Bucket>,
    pub requests: u64,
    /// Where each object lies at the end, and whether it ever moved.
    pub objects: Vec<Rested>,
    /// The nearest any person's centre came to each object's centre, squared, after any request.
    pub nearest: BTreeMap<String, i64>,
}

impl Report {
    pub fn closest_mm(&self) -> i64 {
        self.closest
            .as_ref()
            .map_or(i64::MAX, |closest| closest.0.isqrt())
    }

    pub fn located(&self) -> String {
        locate(self.closest.as_ref(), "apart")
    }

    pub fn located_object(&self) -> String {
        locate(self.closest_object.as_ref(), "from the footprint of")
    }
}

type Closest = (i64, String, String, String, Option<ActionId>, WorldTime);

fn locate(closest: Option<&Closest>, relation: &str) -> String {
    match closest {
        None => "nothing shared a place".to_owned(),
        Some((d, a, b, place, request, at)) => format!(
            "{a} {} mm {relation} {b} in {place}, after request {request:?} at {}s",
            d.isqrt(),
            at.seconds()
        ),
    }
}

/// The scan's state between facts.
#[derive(Default)]
struct Replay {
    at: Standing,
    objects: BTreeMap<EntityId, Object>,
    requests: BTreeMap<EventId, ActionId>,
    /// The `arrived` facts of the current request.
    arrivals: BTreeSet<EventId>,
    shoving: bool,
}

/// Replays `tables`' facts in `EventId` order and checks, after each request's facts:
///
/// ```text
/// people   no pair in one place closer than 595 mm; no centre outside the floor shrunk by 295 mm or
///          within 295 mm of a solid (ROOMS); every displaced arrival — an `arrived` after the first in
///          one request — stays in its place, moves at most 310 mm, at most four per request, stated
///          by movement (a move) or by bodies (a shove); a shoved person's own arrival moves at most
///          500 mm
/// objects  each within its floor, at rest on the floor or a solid's top (± 5 mm), out of every
///          other solid, ≥ 295 mm from every person's centre measured to its footprint, overlapping no
///          other object by more than 5 mm
/// moves    every `object-moved` starts where the scan last saw the object; a pushed one is caused by
///          an `arrived` of the same request, a kicked or thrown one by the request itself
/// ```
///
/// `keys` names the pack's entities (its loaded ids, which a save's facts carry).
pub fn scan(tables: &Tables, keys: &BTreeMap<EntityId, String>) -> Report {
    let place_of = |entity: EntityId| keys.get(&entity).cloned().unwrap_or_default();
    let mut report = Report::default();
    let mut replay = Replay::default();
    let mut current: Option<Option<ActionId>> = None;
    let (mut arrivals_in_request, mut last_instant) = (0_u32, WorldTime::EPOCH);
    let facts: Vec<EventEnvelope> = tables
        .facts
        .iter()
        .map(|(_, bytes)| format::decode::<EventEnvelope>(bytes, "fact").expect("a fact"))
        .collect();
    for fact in &facts {
        let request = request_of(fact, &replay.requests);
        if let Some(id) = request {
            replay.requests.insert(fact.id(), id);
        }
        if current.is_some_and(|previous| previous != request) {
            check(
                &replay,
                &place_of,
                current.flatten(),
                last_instant,
                &mut report,
            );
            arrivals_in_request = 0;
            replay.arrivals.clear();
            replay.shoving = false;
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
            let by = fact.provenance().emitted_by().as_str();
            if !matches!(fact.caused_by(), Causation::Action(_))
                || (by != "movement" && by != "bodies")
            {
                report.violations.push(format!(
                    "{what} {:?} is not caused by a request and stated by movement or bodies",
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
                replay.arrivals.insert(fact.id());
                arrivals_in_request += 1;
                let before = replay.at.get(&person).copied();
                if arrivals_in_request == 1 && replay.shoving {
                    stated("a shoved arrival", &mut report);
                    moved_at_most(
                        &mut report,
                        before,
                        &location,
                        point,
                        500,
                        &place_of(person),
                        fact,
                    );
                } else if arrivals_in_request == 1 {
                    *report.buckets[bucket]
                        .moves
                        .entry(place_of(person))
                        .or_default() += 1;
                } else {
                    report.buckets[bucket].displaced += 1;
                    stated("a displaced arrival", &mut report);
                    moved_at_most(
                        &mut report,
                        before,
                        &location,
                        point,
                        310,
                        &place_of(person),
                        fact,
                    );
                    if arrivals_in_request > 5 {
                        report
                            .violations
                            .push(format!("more than four displaced by request {request:?}"));
                    }
                }
            }
            replay.at.insert(person, (location.place(), point));
        } else if let Ok(payload) = record.payload_for::<StoppedShort>() {
            let stopped: StoppedShort =
                serde_json::from_slice(payload).expect("presence's encoding");
            report.buckets[bucket].stopped += 1;
            if stopped
                .by()
                .is_some_and(|by| replay.objects.contains_key(&by))
            {
                report.buckets[bucket].stopped_by_object += 1;
            }
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
        } else if let Ok(payload) = record.payload_for::<Spoke>() {
            let spoke: Spoke = serde_json::from_slice(payload).expect("conversation's encoding");
            *report.buckets[bucket]
                .talks
                .entry(place_of(spoke.speaker().entity_id()))
                .or_default() += 1;
        } else {
            objects_fact(fact, &mut replay, &mut report, bucket, &place_of);
        }
    }
    check(
        &replay,
        &place_of,
        current.flatten(),
        last_instant,
        &mut report,
    );
    report.objects = replay
        .objects
        .iter()
        .map(|(id, object)| Rested {
            key: place_of(*id),
            place: object.place.map(place_of).unwrap_or_default(),
            at: object.at,
            moved: object.at != object.origin,
            last: object.last.clone(),
            boxy: object.boxy,
            half: object.half,
        })
        .collect();
    report
}

/// A person's arrival moved them at most `limit` mm, within their place.
fn moved_at_most(
    report: &mut Report,
    before: Option<(PlaceId, Option<(i32, i32)>)>,
    location: &mineworld_contracts::Location,
    point: Option<(i32, i32)>,
    limit: i64,
    who: &str,
    fact: &EventEnvelope,
) {
    match before {
        Some((place, Some(was))) if place == location.place() => {
            let moved = distance2(was, point.expect("a position"));
            if moved > limit * limit {
                report.violations.push(format!(
                    "{who} moved {} mm (at most {limit}) at {:?}",
                    moved.isqrt(),
                    fact.id()
                ));
            }
        }
        _ => report
            .violations
            .push(format!("{who} moved out of their place at {:?}", fact.id())),
    }
}

/// bodies' own facts: shapes and placements at genesis; moves, checked against where the scan last saw
/// the object and against their cause; shoves, counted.
fn objects_fact(
    fact: &EventEnvelope,
    replay: &mut Replay,
    report: &mut Report,
    bucket: usize,
    place_of: &dyn Fn(EntityId) -> String,
) {
    if let Some(formed) = bodies_fact(fact, "body-formed") {
        let (boxy, half) = shape(&formed["shape"]);
        replay.objects.insert(
            entity(&formed["object"]),
            Object {
                boxy,
                half,
                place: None,
                at: (0, 0, 0),
                origin: (0, 0, 0),
                last: None,
            },
        );
    } else if let Some(placed) = bodies_fact(fact, "object-placed") {
        let object = replay
            .objects
            .get_mut(&entity(&placed["object"]))
            .expect("formed before placed");
        object.place = Some(entity(&placed["place"]));
        object.at = position(&placed["at"]);
        object.origin = object.at;
    } else if let Some(moved) = bodies_fact(fact, "object-moved") {
        object_moved(fact, &moved, replay, report, bucket, place_of);
    } else if let Some(shoved) = bodies_fact(fact, "person-shoved") {
        replay.shoving = true;
        let b = &mut report.buckets[bucket];
        b.shoves += 1;
        *b.physical
            .entry(place_of(entity(&shoved["by"])))
            .or_default() += 1;
    }
}

/// One `object-moved`: from where the scan last saw the object, caused as its `how` requires.
fn object_moved(
    fact: &EventEnvelope,
    moved: &Value,
    replay: &mut Replay,
    report: &mut Report,
    bucket: usize,
    place_of: &dyn Fn(EntityId) -> String,
) {
    let id = entity(&moved["object"]);
    let key = place_of(id);
    let how = moved["how"].as_str().expect("a how").to_owned();
    let (from, to) = (position(&moved["from"]), position(&moved["to"]));
    let Some(object) = replay.objects.get_mut(&id) else {
        report
            .violations
            .push(format!("{key} moved but never placed"));
        return;
    };
    if object.at != from || object.place != Some(entity(&moved["place"])) {
        report.violations.push(format!(
            "{key} moved from {from:?}, but lay at {:?} ({:?})",
            object.at,
            fact.id()
        ));
    }
    let cause_ok = match (how.as_str(), fact.caused_by()) {
        ("pushed", Causation::Event(cause)) => replay.arrivals.contains(cause),
        ("kicked" | "thrown", Causation::Action(_)) => true,
        _ => false,
    };
    if !cause_ok {
        report.violations.push(format!(
            "{key} {how} with cause {:?} ({:?})",
            fact.caused_by(),
            fact.id()
        ));
    }
    let path = moved["path"].as_array().expect("a path");
    if how != "pushed" && path.first().map(position) != Some(from) {
        report.violations.push(format!(
            "{key}'s path does not start where it lay ({:?})",
            fact.id()
        ));
    }
    object.at = to;
    object.last = Some(format!("{how} from {from:?}"));
    let b = &mut report.buckets[bucket];
    match how.as_str() {
        "pushed" => b.pushes += 1,
        "kicked" => b.kicks += 1,
        "thrown" => b.throws += 1,
        other => report.violations.push(format!("{key} moved how? {other}")),
    }
    if how != "pushed" {
        *b.physical
            .entry(place_of(entity(&moved["by"])))
            .or_default() += 1;
    }
}

fn xyz(p: LocalPosition) -> (i32, i32, i32) {
    (p.x().value(), p.y().value(), p.z().value())
}

fn distance2(a: (i32, i32), b: (i32, i32)) -> i64 {
    let (dx, dy) = (i64::from(a.0 - b.0), i64::from(a.1 - b.1));
    dx * dx + dy * dy
}

/// The state after one request, against the rules.
fn check(
    replay: &Replay,
    place_of: &dyn Fn(EntityId) -> String,
    request: Option<ActionId>,
    instant: WorldTime,
    report: &mut Report,
) {
    let mut by_place: ByPlace = BTreeMap::new();
    for (person, (place, point)) in &replay.at {
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
    check_objects(replay, &by_place, place_of, request, instant, report);
}

/// Every object against SD-O2's invariant, and the closest person–object approach.
fn check_objects(
    replay: &Replay,
    by_place: &ByPlace,
    place_of: &dyn Fn(EntityId) -> String,
    request: Option<ActionId>,
    instant: WorldTime,
    report: &mut Report,
) {
    let placed: Vec<(EntityId, &Object)> = replay
        .objects
        .iter()
        .filter(|(_, object)| object.place.is_some())
        .map(|(id, object)| (*id, object))
        .collect();
    for (i, (id, object)) in placed.iter().enumerate() {
        let key = place_of(*id);
        let place = place_of(object.place.expect("placed"));
        let Some((_, (x0, y0, x1, y1), solid)) = ROOMS.iter().find(|(name, _, _)| *name == place)
        else {
            report
                .violations
                .push(format!("{key} lies in {place}, which has no floor"));
            continue;
        };
        let height = HEIGHTS
            .iter()
            .find(|(name, _)| *name == place)
            .expect("a height")
            .1;
        let (x, y, z) = object.at;
        let (hx, hy, hz) = object.half;
        let footprint = (x - hx, y - hy, x + hx, y + hy);
        if x - hx < *x0 || x + hx > *x1 || y - hy < *y0 || y + hy > *y1 {
            report
                .violations
                .push(format!("{key} at ({x}, {y}) leaves {place}'s floor"));
        }
        let (a, b, c, d) = *solid;
        let on_floor = (z - hz).abs() <= 5;
        let on_solid = (z - (height + hz)).abs() <= 5
            && x - hx >= a
            && x + hx <= c
            && y - hy >= b
            && y + hy <= d;
        if !on_floor && !on_solid {
            report.violations.push(format!(
                "{key} at ({x}, {y}, {z}) rests on nothing in {place}"
            ));
        }
        let meets = if object.boxy {
            footprint.0 < c && footprint.2 > a && footprint.1 < d && footprint.3 > b
        } else {
            to_rect2((x, y), *solid) < i64::from(hx).pow(2)
        };
        if meets && !on_solid {
            report
                .violations
                .push(format!("{key} at ({x}, {y}) is in {place}'s solid"));
        }
        for (person, at) in by_place.get(&place).into_iter().flatten() {
            let reach2 = distance2(*at, (x, y));
            let nearest = report.nearest.entry(key.clone()).or_insert(i64::MAX);
            *nearest = (*nearest).min(reach2);
            let d2 = object.footprint_distance2(*at);
            if report
                .closest_object
                .as_ref()
                .is_none_or(|closest| d2 < closest.0)
            {
                report.closest_object = Some((
                    d2,
                    person.clone(),
                    key.clone(),
                    place.clone(),
                    request,
                    instant,
                ));
            }
            if d2 < 295 * 295 {
                report.violations.push(format!(
                    "{person} at {at:?} is {} mm from {key}'s footprint in {place} after request \
                     {request:?}",
                    d2.isqrt()
                ));
            }
        }
        for (other_id, other) in &placed[i + 1..] {
            if other.place != object.place {
                continue;
            }
            if overlap(object, other) {
                report.violations.push(format!(
                    "{key} and {} overlap in {place} after request {request:?}",
                    place_of(*other_id)
                ));
            }
        }
    }
}

/// Whether two objects' footprints overlap by more than 5 mm.
fn overlap(a: &Object, b: &Object) -> bool {
    let ((ax, ay, _), (bx, by, _)) = (a.at, b.at);
    let ((ahx, ahy, _), (bhx, bhy, _)) = (a.half, b.half);
    match (a.boxy, b.boxy) {
        (true, true) => ahx + bhx - (ax - bx).abs() > 5 && ahy + bhy - (ay - by).abs() > 5,
        (false, false) => distance2((ax, ay), (bx, by)) < i64::from(ahx + bhx - 5).pow(2),
        (false, true) => {
            to_rect2((ax, ay), (bx - bhx, by - bhy, bx + bhx, by + bhy)) < i64::from(ahx - 5).pow(2)
        }
        (true, false) => overlap(b, a),
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

/// The scan's verdict on 12b's activity precondition (`ARC-23`: activity first): in every 10-day
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

/// AO-1 … AO-3 (step-11 §18.4 PO-9, SD-O18), fixed before measuring, per 10-day bucket: every seat
/// moves and talks; at least one accepted kick, throw and shove, one push and one stopped-short by an
/// object; no seat's kicks + throws + shoves above 144. Prints the counts first, then judges.
pub fn print_objects_activity(report: &Report) {
    println!("objects at the end:");
    for object in &report.objects {
        println!(
            "  {} in {} at {:?}{}",
            object.key,
            object.place,
            object.at,
            if object.moved { "" } else { " (never moved)" }
        );
    }
    let nearest: BTreeMap<&String, i64> = report
        .nearest
        .iter()
        .map(|(key, d2)| (key, d2.isqrt()))
        .collect();
    println!("nearest approach to each object's centre, mm: {nearest:?}");
    for (index, bucket) in report.buckets.iter().enumerate() {
        println!(
            "AO days {}-{}: kicks {}, throws {}, shoves {}, pushes {}, stopped by an object {}; \
             physical per seat {:?}; talks {:?}",
            index * 10 + 1,
            index * 10 + 10,
            bucket.kicks,
            bucket.throws,
            bucket.shoves,
            bucket.pushes,
            bucket.stopped_by_object,
            bucket.physical,
            bucket.talks
        );
    }
}

/// The five actions AO-2′ counts, in one bucket.
fn actions(bucket: &Bucket) -> [(&'static str, u64); 5] {
    [
        ("kick", bucket.kicks),
        ("throw", bucket.throws),
        ("shove", bucket.shoves),
        ("push", bucket.pushes),
        ("stopped-short by an object", bucket.stopped_by_object),
    ]
}

/// AO-1 and AO-3 (unchanged, step-11 §18.4 PO-9), every bucket: every seat moves and talks; no seat's
/// kicks + throws + shoves above 144.
pub fn assert_ao1_ao3(report: &Report, seats: &[&str]) {
    for (index, bucket) in report.buckets.iter().enumerate() {
        for seat in seats {
            assert!(
                bucket.moves.get(*seat).copied().unwrap_or(0) > 0
                    && bucket.talks.get(*seat).copied().unwrap_or(0) > 0,
                "AO-1: {seat} moved and talked in bucket {index}"
            );
            let physical = bucket.physical.get(*seat).copied().unwrap_or(0);
            assert!(
                physical <= 144,
                "AO-3: {seat} did {physical} kicks, throws and shoves in bucket {index}"
            );
        }
    }
}

/// AO-2′ (c), day-1 sanity (the AO-2 ruling, step-11 §18.11): every action occurs in days 1–10.
pub fn assert_day_one(report: &Report) {
    let first = report.buckets.first().expect("a first bucket");
    for (what, count) in actions(first) {
        assert!(count > 0, "AO-2′ (c): no {what} in days 1-10");
    }
}

/// Whether a person's 300 mm disc at `p` overlaps the footprint of `object`.
fn disc_overlaps(p: (i32, i32), object: &Rested) -> bool {
    let (x, y, _) = object.at;
    let (hx, hy, _) = object.half;
    if object.boxy {
        to_rect2(p, (x - hx, y - hy, x + hx, y + hy)) < 300 * 300
    } else {
        distance2(p, (x, y)) < i64::from(300 + hx).pow(2)
    }
}

/// AO-2′ (b′), reachability (the final AO-2 ruling, step-11 §18.11): at day 30, every object has at
/// least one **free standing point** within kick reach — a point of its room's 50 mm lattice (anchored a
/// radius in from the floor's south-west corner) where a person's 300 mm disc lies inside the floor and
/// overlaps no solid and no other object, within 800 mm of the object's ground point. Prints, for each
/// object, how many such points it has.
pub fn assert_reachable(report: &Report) {
    let mut failures = Vec::new();
    for object in &report.objects {
        let (_, (x0, y0, x1, y1), solid) = ROOMS
            .iter()
            .find(|(name, _, _)| *name == object.place)
            .expect("a room");
        let (ox, oy, _) = object.at;
        let others: Vec<&Rested> = report
            .objects
            .iter()
            .filter(|other| other.place == object.place && other.key != object.key)
            .collect();
        let mut free = 0_u32;
        let mut y = y0 + 300;
        while y <= y1 - 300 {
            let mut x = x0 + 300;
            while x <= x1 - 300 {
                let p = (x, y);
                if distance2(p, (ox, oy)) <= 800 * 800
                    && to_rect2(p, *solid) >= 300 * 300
                    && !others.iter().any(|other| disc_overlaps(p, other))
                {
                    free += 1;
                }
                x += 50;
            }
            y += 50;
        }
        println!(
            "AO-2′ (b′): {} at {:?}{}: {free} free standing points within 800 mm",
            object.key,
            object.at,
            if object.moved { "" } else { " (never moved)" }
        );
        if free == 0 {
            failures.push(format!(
                "{} at {:?} in {} has no free standing point within 800 mm",
                object.key, object.at, object.place
            ));
        }
    }
    assert!(failures.is_empty(), "AO-2′ (b′): {failures:#?}");
}

/// AO-2′ (a), aggregate activity (the AO-2 ruling): each action occurs at least once in every bucket,
/// summed over the seeds' reports. Prints the sums.
pub fn assert_aggregate(reports: &[Report]) {
    let buckets = reports
        .iter()
        .map(|report| report.buckets.len())
        .max()
        .unwrap_or(0);
    for index in 0..buckets {
        let sums: Vec<(&str, u64)> = (0..5)
            .map(|which| {
                let name = actions(&Bucket::default())[which].0;
                let sum = reports
                    .iter()
                    .filter_map(|report| report.buckets.get(index))
                    .map(|bucket| actions(bucket)[which].1)
                    .sum();
                (name, sum)
            })
            .collect();
        println!(
            "AO-2′ (a) days {}-{}, summed over the seeds: {sums:?}",
            index * 10 + 1,
            index * 10 + 10
        );
        for (what, sum) in sums {
            assert!(
                sum > 0,
                "AO-2′ (a): no {what} in bucket {index} over the seeds"
            );
        }
    }
}
