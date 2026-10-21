//! Every town's station is a place people go, its frames are frames a client can draw, and a broken town
//! is refused alike by `run` and by `validate` (S29-a §9 C6; `MODULE_SPEC.md` §4.3; `ARC-105`).
//!
//! ```text
//! static (every run)
//!   nothing_is_sold_at_the_ticket_window               no shop is opened at a ticket office (AC-a-9)
//!   every_platform_anchor_lies_on_one_edge            a stop's origin on exactly one floor edge
//!   the_station_places_lay_out_without_overlap        the doorway rule, every town with bodies (AC-a-14)
//!   a_broken_town_is_refused_by_run_as_by_validate    X-2, X-3 and the scattered tag (AC-a-3 … AC-a-5)
//! runs (ignored by default; 30 days, seed 7, release build; the milestone runs them with --ignored)
//!   people_meet_at_the_station                        AC-a-7, AC-a-8, AC-a-10
//!   the_station_resumes_identically                    a run stopped at day 15 and continued is the same
//! ```
//!
//! The oracles are the authored files, the loaded world's genesis facts and the saves' stored facts, each
//! decoded by its owner's published type; never a value the loader reports about itself.

mod bodies;
mod headless;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use headless::{Tables, fresh, mineworld, stderr};
use mineworld_bodies::PlaceShaped;
use mineworld_contracts::{EntityId, EntityKey, EventEnvelope, Tag, WorldTime};
use mineworld_conversation::Spoke;
use mineworld_persistence::format;
use mineworld_presence::{PersonEnteredPlace, StoppedShort};
use mineworld_test_support::Scratch;
use mineworld_worldpack::{PackRoots, SettlementKind, WorldPack};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const DAY: i64 = 86_400;
/// The inset of every doorway point from its wall (12d; `CORE_CONCEPTS.md` §6.1).
const INSET: i32 = 400;

fn world(name: &str) -> PathBuf {
    Path::new(ROOT).join("worlds").join(name)
}

/// The pack roots the reference worlds are validated with (Lakeside requires them).
fn roots() -> PackRoots {
    PackRoots::new(
        vec![
            Path::new(ROOT).join("entities"),
            Path::new(ROOT).join("presentation/mineworld-default"),
        ],
        None,
    )
    .expect("the repository's pack roots")
}

fn root_args() -> Vec<String> {
    vec![
        "--packs".into(),
        format!("{ROOT}/entities"),
        "--packs".into(),
        format!("{ROOT}/presentation/mineworld-default"),
    ]
}

/// Every reference world that declares itself a town, by directory name.
fn towns() -> Vec<String> {
    let mut found: Vec<String> = std::fs::read_dir(Path::new(ROOT).join("worlds"))
        .expect("worlds/ lists")
        .map(|entry| entry.expect("an entry").path())
        .filter(|path| path.join("world.yaml").is_file())
        .filter(|path| {
            WorldPack::read_with(path, &roots())
                .expect("every reference world reads")
                .settlement()
                .is_some_and(|s| matches!(s.kind(), SettlementKind::Town { .. }))
        })
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    found.sort();
    found
}

/// A rectangle, millimetres: (min x, min y, max x, max y).
type Rect = (i32, i32, i32, i32);

/// One side of a doorway: the place it is in, its point there, the place it leads to, its point there.
type Door = (String, (i32, i32), String, (i32, i32));

/// Each shaped place's floor, from the genesis `place-shaped` facts bodies states (its own decoding).
fn floors(pack: &WorldPack) -> BTreeMap<String, Rect> {
    let loaded = pack.load(WorldTime::EPOCH).expect("the town loads");
    let keys: BTreeMap<EntityId, String> = loaded
        .ids()
        .iter()
        .map(|(key, id)| (*id, key.as_str().to_owned()))
        .collect();
    let mut floors = BTreeMap::new();
    for fact in loaded.genesis() {
        let record = fact.payload();
        let Ok(payload) = record.payload_for::<PlaceShaped>() else {
            continue;
        };
        let shaped: PlaceShaped = serde_json::from_slice(payload).expect("bodies' encoding");
        let floor = shaped.shape().floor();
        floors.insert(
            keys[&shaped.place().entity_id()].clone(),
            (
                floor.min().x().value(),
                floor.min().y().value(),
                floor.max().x().value(),
                floor.max().y().value(),
            ),
        );
    }
    floors
}

/// The outward normal of the one floor edge `point` stands `INSET` inside, or why there is not one.
fn facing(place: &str, floor: Rect, point: (i32, i32)) -> Result<(i32, i32), String> {
    let (x0, y0, x1, y1) = floor;
    let (x, y) = point;
    let inside = x0 < x && x < x1 && y0 < y && y < y1;
    let edges = [
        (x - x0, (-1, 0)),
        (x1 - x, (1, 0)),
        (y - y0, (0, -1)),
        (y1 - y, (0, 1)),
    ];
    let near: Vec<(i32, i32)> = edges
        .iter()
        .filter(|(distance, _)| *distance == INSET)
        .map(|(_, normal)| *normal)
        .collect();
    match (inside, near.as_slice()) {
        (true, [normal]) => Ok(*normal),
        _ => Err(format!(
            "{place}: the doorway point {point:?} is not {INSET} mm inside exactly one edge of its floor {floor:?}"
        )),
    }
}

/// The station group and the places it opens onto, laid out in the hall's frame by the doorway rule: a
/// neighbour's origin is `there − here`, moved 800 mm across the shared wall. Returns each place's
/// floor in the hall's frame, or the first door that faces the wrong way or the first overlap.
fn layout(pack: &WorldPack) -> Result<BTreeMap<String, Rect>, String> {
    let Some(SettlementKind::Town { station, .. }) = pack.settlement().map(|s| s.kind()) else {
        return Err("not a town".into());
    };
    let floors = floors(pack);
    let station_tag = Tag::new("station").unwrap();
    let in_group = |key: &EntityKey| pack.places()[key].tags.contains(&station_tag);
    // Every passage once, as (from, here, to, there), both ways.
    let mut doors: Vec<Door> = Vec::new();
    for (key, place) in pack.places() {
        for passage in &place.passages {
            let (Some(here), Some(there)) = (&passage.here, &passage.there) else {
                continue;
            };
            let (here, there) = (
                (here.x.value(), here.y.value()),
                (there.x.value(), there.y.value()),
            );
            if in_group(key) || in_group(&passage.to) {
                doors.push((key.to_string(), here, passage.to.to_string(), there));
                doors.push((passage.to.to_string(), there, key.to_string(), here));
            }
        }
    }
    let mut origin: BTreeMap<String, (i32, i32)> = BTreeMap::from([(station.to_string(), (0, 0))]);
    let mut queue = vec![station.to_string()];
    while let Some(a) = queue.pop() {
        for (from, here, to, there) in doors.iter().filter(|door| door.0 == a) {
            let floor_a = *floors.get(from).ok_or(format!("{from} has no floor"))?;
            let floor_b = *floors.get(to).ok_or(format!("{to} has no floor"))?;
            let n_a = facing(from, floor_a, *here)?;
            let n_b = facing(to, floor_b, *there)?;
            if n_a != (-n_b.0, -n_b.1) {
                return Err(format!(
                    "the door between {from} and {to}: {from}'s point faces {n_a:?}, {to}'s faces {n_b:?}; \
                     they must stand on opposite sides of one wall"
                ));
            }
            let o_a = origin[from];
            let o_b = (
                o_a.0 + here.0 - there.0 - 2 * INSET * n_b.0,
                o_a.1 + here.1 - there.1 - 2 * INSET * n_b.1,
            );
            if !origin.contains_key(to) {
                origin.insert(to.clone(), o_b);
                if in_group(&EntityKey::new(to.as_str()).unwrap()) {
                    queue.push(to.clone());
                }
            }
        }
    }
    let laid: BTreeMap<String, Rect> = origin
        .iter()
        .map(|(key, (ox, oy))| {
            let (x0, y0, x1, y1) = floors[key];
            (key.clone(), (x0 + ox, y0 + oy, x1 + ox, y1 + oy))
        })
        .collect();
    let keys: Vec<&String> = laid.keys().collect();
    for (i, a) in keys.iter().enumerate() {
        for b in &keys[i + 1..] {
            let (ra, rb) = (laid[*a], laid[*b]);
            if ra.0 < rb.2 && rb.0 < ra.2 && ra.1 < rb.3 && rb.1 < ra.3 {
                return Err(format!(
                    "{a} {ra:?} and {b} {rb:?} overlap in the hall's frame"
                ));
            }
        }
    }
    Ok(laid)
}

/// The towns whose places carry bodies' `body:` sections.
fn shaped_towns() -> Vec<String> {
    towns()
        .into_iter()
        .filter(|town| {
            let pack = WorldPack::read_with(world(town), &roots()).expect("reads");
            !floors(&pack).is_empty()
        })
        .collect()
}

/// A copy of the reference world `town` in scratch, under its own name (a pack's id is its directory's).
fn copy_of(town: &str, name: &str) -> (Scratch, PathBuf) {
    let scratch = fresh(name);
    let copy = scratch.join(town);
    bodies::copy_dir(&world(town), &copy);
    (scratch, copy)
}

fn replace_in(file: &Path, from: &str, to: &str) {
    let text = std::fs::read_to_string(file).expect("the file reads");
    assert_eq!(
        text.matches(from).count(),
        1,
        "{}: `{from}` once",
        file.display()
    );
    std::fs::write(file, text.replace(from, to)).expect("writes");
}

#[test]
fn the_station_places_lay_out_without_overlap() {
    let towns = shaped_towns();
    assert!(
        towns.len() >= 2,
        "at least two towns with bodies: {towns:?}"
    );
    for town in &towns {
        let pack = WorldPack::read_with(world(town), &roots()).expect("reads");
        let laid = layout(&pack).unwrap_or_else(|why| panic!("{town}: {why}"));
        for (key, rect) in &laid {
            eprintln!("{town}: {key} {rect:?}");
        }
        assert!(
            laid.len() >= 4,
            "{town}: the group and its street are laid out"
        );
    }

    // A generated variant: the hall joins the street 2 m further north. The check reads the frames.
    let (_moved, moved) = copy_of("market-town", "station-layout-moved");
    replace_in(
        &moved.join("places/station.yaml"),
        "there: { x: -62050, y: -4900 }",
        "there: { x: -62050, y: -2900 }",
    );
    layout(&WorldPack::read(&moved).expect("the variant reads")).expect("the variant lays out");

    // Revision 3's frames, planted: each is refused, naming the place.
    let (_bay, bay) = copy_of("market-town", "station-layout-bay");
    replace_in(
        &bay.join("places/station-tickets.yaml"),
        "here: { x: 3000, y: 4200 }",
        "here: { x: 3000, y: 400 }",
    );
    // bodies refuses this door at genesis already (`bodies-doorway`); the layout check holds it for a world
    // whose places carry no bodies as well. Either refusal names the place.
    let pack = WorldPack::read(&bay).expect("reads");
    let refused = match pack.load(WorldTime::EPOCH) {
        Err(error) => error.to_string(),
        Ok(_) => layout(&pack).expect_err("a misfacing door"),
    };
    assert!(refused.contains("station-tickets"), "{refused}");

    let (_platform, platform) = copy_of("market-town", "station-layout-platform");
    let file = platform.join("places/station-platform.yaml");
    replace_in(
        &file,
        "floor: { min: { x: 0, y: -30000 }, max: { x: 11000, y: 30000 } }",
        "floor: { min: { x: -30000, y: 0 }, max: { x: 30000, y: 11000 } }",
    );
    // bodies refuses the shape as it is read (a furnishing off the floor) or at genesis (a door on its
    // edge); the layout check holds the frame when the shape alone would pass. Each refusal names the place.
    let refused = match WorldPack::read(&platform) {
        Err(error) => format!("{error:?}"),
        Ok(pack) => match pack.load(WorldTime::EPOCH) {
            Err(error) => error.to_string(),
            Ok(_) => layout(&pack).expect_err("a platform across the hall"),
        },
    };
    assert!(refused.contains("station-platform"), "{refused}");
}

#[test]
fn every_platform_anchor_lies_on_one_edge() {
    let stop = Tag::new("stop").unwrap();
    let mut checked = 0;
    for town in shaped_towns() {
        let pack = WorldPack::read_with(world(&town), &roots()).expect("reads");
        let floors = floors(&pack);
        for (key, place) in pack.places() {
            if !place.tags.contains(&stop) {
                continue;
            }
            let (x0, y0, x1, y1) = floors[key.as_str()];
            let on = [x0 == 0, x1 == 0, y0 == 0, y1 == 0]
                .iter()
                .filter(|edge| **edge)
                .count();
            let within = x0 <= 0 && 0 <= x1 && y0 <= 0 && 0 <= y1;
            assert!(
                within && on == 1,
                "{town}: {key}'s origin lies on {on} edges of {:?}",
                (x0, y0, x1, y1)
            );
            checked += 1;
        }
    }
    assert!(checked >= 2, "every town with bodies has a stop");
}

#[test]
fn nothing_is_sold_at_the_ticket_window() {
    let office = Tag::new("ticket-office").unwrap();
    let mut shops_seen = 0;
    for town in towns() {
        let pack = WorldPack::read_with(world(&town), &roots()).expect("reads");
        let loaded = pack.load(WorldTime::EPOCH).expect("loads");
        let offices: Vec<serde_json::Value> = pack
            .places()
            .iter()
            .filter(|(_, place)| place.tags.contains(&office))
            .map(|(key, _)| serde_json::to_value(loaded.ids()[key]).expect("an id"))
            .collect();
        assert!(!offices.is_empty(), "{town} has a ticket office");
        for fact in loaded.genesis() {
            if fact.event_type().as_str() != "shop-opened" {
                continue;
            }
            shops_seen += 1;
            let shop: serde_json::Value =
                serde_json::from_slice(fact.payload().payload()).expect("economy's encoding");
            assert!(
                !offices.contains(&shop["place"]),
                "{town}: a shop is opened at a ticket office: {shop}"
            );
        }
    }
    // Not vacuous: Market Town's other shops are found the same way.
    assert!(shops_seen > 0, "the oracle sees the towns' shops");
}

#[test]
fn a_broken_town_is_refused_by_run_as_by_validate() {
    let cases = [
        (
            "station-broken-no-stop",
            "places/station-platform.yaml",
            "  - stop\n",
            "",
            "TOWN_STATION_NO_STOP",
            "station",
        ),
        (
            "station-broken-unreachable",
            "places/station.yaml",
            "passages:\n  # The arch on the east façade, 0.4 m inside; in the street, 0.4 m in front of the façade (design).\n  - to: street\n    here: { x: 16400, y: 25900 }\n    there: { x: -62050, y: -4900 }\n",
            "",
            "TOWN_STATION_UNREACHABLE",
            "",
        ),
        (
            "station-broken-scattered",
            "places/park.yaml",
            "tags:\n",
            "tags:\n  - station\n",
            "TOWN_STATION_SCATTERED",
            "park",
        ),
    ];
    for (name, file, from, to, code, key) in cases {
        let (_copy, copy) = copy_of("market-town", name);
        replace_in(&copy.join(file), from, to);
        let path = copy.to_str().expect("a printable path");
        for command in [
            vec!["validate", path],
            vec!["run", path, "--headless", "--seed", "1", "--days", "1"],
        ] {
            let output = mineworld(&command);
            let said = stderr(&output);
            assert!(
                !output.status.success(),
                "{name}: {} must refuse",
                command[0]
            );
            assert!(
                said.contains(code),
                "{name}: {} says {code}: {said}",
                command[0]
            );
            assert!(said.contains(key), "{name}: names {key}: {said}");
        }
    }
}

/// Per day: the distinct people who entered a station place, and the lines spoken by someone standing in
/// one; and over the run, geometry-caused stops on the way to the station against arrivals there.
struct Visits {
    entered: BTreeMap<i64, BTreeSet<String>>,
    spoken: BTreeMap<i64, u64>,
    geometry_stops: u64,
    arrivals: u64,
}

fn visits(tables: &Tables, keys: &BTreeMap<EntityId, String>, group: &BTreeSet<String>) -> Visits {
    let mut at: BTreeMap<String, String> = BTreeMap::new();
    let mut out = Visits {
        entered: BTreeMap::new(),
        spoken: BTreeMap::new(),
        geometry_stops: 0,
        arrivals: 0,
    };
    let name = |id: EntityId| keys.get(&id).cloned().unwrap_or_default();
    for (_, bytes) in &tables.facts {
        let fact: EventEnvelope = format::decode(bytes, "fact").expect("a fact");
        let day = fact.at().seconds().div_euclid(DAY);
        let record = fact.payload();
        if let Ok(payload) = record.payload_for::<PersonEnteredPlace>() {
            let entered: PersonEnteredPlace = serde_json::from_slice(payload).expect("presence");
            let (person, place) = (
                name(entered.person().entity_id()),
                name(entered.place().entity_id()),
            );
            if group.contains(&place) {
                out.entered.entry(day).or_default().insert(person.clone());
                out.arrivals += 1;
            }
            at.insert(person, place);
        } else if let Ok(payload) = record.payload_for::<Spoke>() {
            let spoke: Spoke = serde_json::from_slice(payload).expect("conversation");
            let speaker = name(spoke.speaker().entity_id());
            if at.get(&speaker).is_some_and(|place| group.contains(place)) {
                *out.spoken.entry(day).or_default() += 1;
            }
        } else if let Ok(payload) = record.payload_for::<StoppedShort>() {
            let stopped: StoppedShort = serde_json::from_slice(payload).expect("presence");
            if stopped.by().is_none() && group.contains(&name(stopped.wanted().place().entity_id()))
            {
                out.geometry_stops += 1;
            }
        }
    }
    out
}

fn station_group(pack: &WorldPack) -> BTreeSet<String> {
    let station = Tag::new("station").unwrap();
    pack.places()
        .iter()
        .filter(|(_, place)| place.tags.contains(&station))
        .map(|(key, _)| key.to_string())
        .collect()
}

fn run_town(town: &str, days: u64, save: &Path) {
    let (days, path) = (
        days.to_string(),
        save.to_str().expect("printable").to_owned(),
    );
    let pack = world(town);
    let pack = pack.to_str().expect("printable");
    let mut arguments: Vec<&str> = vec![
        "run",
        pack,
        "--headless",
        "--seed",
        "7",
        "--days",
        &days,
        "--save",
        &path,
    ];
    let roots = root_args();
    arguments.extend(roots.iter().map(String::as_str));
    let output = mineworld(&arguments);
    assert!(output.status.success(), "{town}: {}", stderr(&output));
}

#[test]
#[ignore = "30-day runs of three towns: the milestone runs them (`--ignored`, release build)"]
fn people_meet_at_the_station() {
    for town in towns() {
        let save = fresh(&format!("station-visit-{town}"));
        run_town(&town, 30, &save);
        let pack = WorldPack::read_with(world(&town), &roots()).expect("reads");
        let keys: BTreeMap<EntityId, String> = pack
            .load(WorldTime::EPOCH)
            .expect("loads")
            .ids()
            .iter()
            .map(|(key, id)| (*id, key.as_str().to_owned()))
            .collect();
        let group = station_group(&pack);
        let seen = visits(&Tables::read(&save), &keys, &group);
        for day in 0..30 {
            let people = seen.entered.get(&day).map_or(0, BTreeSet::len);
            let lines = seen.spoken.get(&day).copied().unwrap_or(0);
            eprintln!(
                "{town} day {day}: {people} people entered the station, {lines} lines spoken there"
            );
        }
        eprintln!(
            "{town}: geometry-caused stops on the way to the station {} of {} arrivals",
            seen.geometry_stops, seen.arrivals
        );
        for day in 0..30 {
            assert!(
                seen.entered.get(&day).map_or(0, BTreeSet::len) >= 3,
                "{town} day {day}: at least three people come to the station"
            );
            assert!(
                seen.spoken.get(&day).copied().unwrap_or(0) >= 1,
                "{town} day {day}: someone speaks at the station"
            );
        }
        assert!(
            seen.geometry_stops * 10 <= seen.arrivals,
            "{town}: geometry-caused stops ≤ 10 % of arrivals"
        );
    }
}

#[test]
#[ignore = "two 30-day runs: the milestone runs it (`--ignored`, release build)"]
fn the_station_resumes_identically() {
    let control = fresh("station-resume-control");
    run_town("market-town", 30, &control);
    let continued = fresh("station-resume-continued");
    run_town("market-town", 15, &continued);
    run_town("market-town", 30, &continued);
    Tables::read(&continued).assert_same_history(&Tables::read(&control), "stopped at day 15");
}
