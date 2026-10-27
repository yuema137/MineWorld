//! Lakeside gets walls (step-33 S33-b, `pr-s33b-lakeside-bodies.md` §7), through the real binary and the
//! real server.
//!
//! ```text
//! (1) walls   a stride east off the pier's deck and one south past the shore's water line each stop
//!             short with `by: None`, inside the floor
//! (2) save    30 days, seed 7, saved: no recorded position outside its floor shrunk by R or in a solid
//! (3) NW-3    over the same run: walls and furniture stop at most 10 % of strides; every place is
//!             entered at least 90 % as often as in the copy without bodies
//! (4) plan    every place's floor, carried into the lane's frame by its doorway (§5.1), is disjoint
//!             from every other's (I-S33-9: the walls can be drawn)
//! (5) only    removing `bodies` from `systems:` gives exactly the world after only moving the doorway
//!             points (Q-S33b-1 as ruled): the copy without the system line runs the same summary as
//!             the copy without the line and without any `body:` section
//! ```
//!
//! The oracles are the world's own files and the save (`bodies/mod.rs`'s scan and rooms), never a value
//! the System reports about itself.

mod bodies;
mod headless;
#[allow(dead_code)]
mod lakeside;
mod support;

use std::collections::BTreeMap;
use std::path::Path;

use bodies::{R, Report, Rooms, rooms_of, scan_in, strip_section};
use headless::{Tables, fresh};
use lakeside::history::{facts_of, of};
use lakeside::{edit, lakeside, read_world, run, scratch_lakeside, text};
use mineworld_contracts::{ActionResult, EntityId, WorldTime};
use mineworld_presence::StoppedShort;
use mineworld_server::WireObservation;
use serde_json::Value;
use support::{Client, Server, stride};

const SEED: u64 = 7;
const DAYS: u64 = 30;
/// The doorway rule (12d SD-D3): each point stands this far inside its floor's edge.
const INSET: i32 = 400;

/// Every entity of `world`, by key, read with Lakeside's pack roots.
fn ids(world: &Path) -> BTreeMap<String, EntityId> {
    let loaded = read_world(world).load(WorldTime::EPOCH).expect("it loads");
    loaded
        .ids()
        .iter()
        .map(|(key, id)| (key.as_str().to_owned(), *id))
        .collect()
}

fn keys(world: &Path) -> BTreeMap<EntityId, String> {
    ids(world).into_iter().map(|(k, id)| (id, k)).collect()
}

/// Where the observer stands, as the server last said.
fn standing(observation: &WireObservation) -> (u64, (i32, i32)) {
    let location = observation.self_location().expect("located");
    let local = location.local().expect("a position");
    (
        location.place().entity_id().raw(),
        (local.x().value(), local.y().value()),
    )
}

/// `mineworld server <world> --save <save> <roots…>`.
async fn host(world: &Path, save: &Path) -> Server {
    let (world, save) = (text(world), text(save));
    let roots = lakeside::roots();
    let mut arguments = vec!["server", &world, "--save", &save];
    arguments.extend(roots.iter().map(String::as_str));
    Server::start(&arguments).await
}

/// One stride past an edge: accepted as far as it goes; the observer is still in `place`, inside its
/// floor by a radius at least. Returns where they stopped.
async fn stride_past(
    client: &mut Client,
    who: EntityId,
    place: EntityId,
    from: (i32, i32),
    to: (i32, i32),
) -> (i32, i32) {
    let (_, answer) = client.submit(stride(who, place, to.0, to.1)).await;
    assert!(
        matches!(answer, ActionResult::Accepted { .. }),
        "accepted as far as it goes: {answer:?}"
    );
    let seen = client
        .observation_where("the stride past the edge answered", |o| {
            standing(o).1 != from
        })
        .await;
    let (at, point) = standing(&seen);
    assert_eq!(
        at,
        place.raw(),
        "still in the place: nobody leaves by an edge"
    );
    point
}

/// (1) AC-b-3: Nils, on the pier, strides east off the deck; then walks by the server's route to the
/// shore and strides south into the lake. Both stop inside the floor, and the save holds two
/// `stopped-short` facts with no `by`: a floor's edge is a wall (F-S33b-4).
#[tokio::test(flavor = "multi_thread")]
async fn the_lake_and_the_deck_edges_are_walls() {
    let world = lakeside();
    let ids = ids(&world);
    let (pier, shore) = (ids["pier"], ids["shore"]);
    let save = fresh("lakeside-bodies-walls");
    let server = host(&world, &save).await;
    let mut client = Client::connect(server.address).await;
    let (nils, _) = client.join("nils").await;
    assert_eq!(
        standing(&client.observation().await),
        (pier.raw(), (2_000, 10_000)),
        "nils on the pier"
    );
    // East: 1.4 m to (3 400, 10 000), then 1.6 m on, past the deck's east edge (x 4 000).
    client
        .submit_accepted(stride(nils, pier, 3_400, 10_000))
        .await;
    client
        .observation_where("nils at (3 400, 10 000)", |o| {
            standing(o).1 == (3_400, 10_000)
        })
        .await;
    let (x, y) = stride_past(&mut client, nils, pier, (3_400, 10_000), (5_000, 10_000)).await;
    println!("off the deck: nils stopped at ({x}, {y})");
    assert!(
        y == 10_000 && x > 3_400 && x <= 4_000 - R,
        "stopped on the deck, a radius from its edge: ({x}, {y})"
    );

    // To the shore by the server's route, then south past the water line (y 0).
    client.walk_there(nils, shore, Some((2_000, 1_000))).await;
    client
        .observation_where("nils on the shore", |o| {
            standing(o) == (shore.raw(), (2_000, 1_000))
        })
        .await;
    let (x, y) = stride_past(&mut client, nils, shore, (2_000, 1_000), (2_000, -1_000)).await;
    println!("into the lake: nils stopped at ({x}, {y})");
    assert!(
        x == 2_000 && (R..1_000).contains(&y),
        "stopped on the shingle, a radius from the water: ({x}, {y})"
    );
    drop(client);
    drop(server);

    let walls: Vec<StoppedShort> = facts_of(&save)
        .iter()
        .filter(|fact| of(fact, "stopped-short"))
        .map(|fact| serde_json::from_slice(fact.payload().payload()).expect("presence's encoding"))
        .filter(|stopped: &StoppedShort| stopped.by().is_none())
        .collect();
    assert!(
        walls.len() >= 2,
        "both strides stopped short by geometry (by: None): {}",
        walls.len()
    );
}

/// 30 days of Lakeside, seed 7, saved, scanned against its own files' rooms.
fn thirty_days(world: &Path, save_name: &str, rooms: Rooms) -> Report {
    let save = fresh(save_name);
    let printed = run(world, SEED, DAYS, Some(&save));
    assert!(printed.contains("faults     0"), "faults 0: {printed}");
    scan_in(&Tables::read(&save), &keys(world), rooms)
}

/// Entries into each place over a run, by key.
fn entries(report: &Report) -> BTreeMap<String, u64> {
    let mut into: BTreeMap<String, u64> = BTreeMap::new();
    for bucket in &report.buckets {
        for ((_, to), count) in &bucket.crossings {
            *into.entry(to.clone()).or_default() += count;
        }
    }
    into
}

/// A copy of Lakeside without `bodies` in `systems:` and without every `body:` (the loader refuses a
/// section no enabled system owns).
fn without_bodies(name: &str) -> headless::Scratch {
    let copy = scratch_lakeside(name);
    edit(&copy.join("world.yaml"), "  - bodies\n", "");
    for entry in std::fs::read_dir(copy.join("places")).expect("lists") {
        let file = entry.expect("an entry").path();
        let (kept, found) = strip_section(&lakeside::read_lf(&file), "body");
        assert!(found, "{file:?} carried a body:");
        std::fs::write(&file, kept).expect("writes");
    }
    copy
}

/// (2) AC-b-2 and (3) AC-b-5 over one run.
#[test]
fn nobody_stands_in_the_lake_in_thirty_days_and_bodies_stay_rare() {
    let world = lakeside();
    let rooms = rooms_of(&world);
    assert_eq!(rooms.len(), 10, "every place is shaped");
    let report = thirty_days(&world, "lakeside-bodies-30", rooms.clone());
    println!(
        "{} requests; closest pair {}",
        report.requests,
        report.located()
    );
    assert!(
        report.violations.is_empty(),
        "{} violations, the first: {:#?}",
        report.violations.len(),
        &report.violations[..report.violations.len().min(5)]
    );

    // NW-3 (the M-3 ruling): strides stopped by walls and furniture at most 10 %.
    let strides: u64 = report
        .buckets
        .iter()
        .flat_map(|bucket| bucket.moves.values())
        .sum();
    let geometry: u64 = report.buckets.iter().map(|b| b.stopped_by_geometry).sum();
    let stopped: u64 = report.buckets.iter().map(|b| b.stopped).sum();
    println!(
        "strides {strides}; stopped by geometry {geometry} ({:.1} %), by a person {}",
        100.0 * geometry as f64 / strides.max(1) as f64,
        stopped - geometry
    );
    assert!(strides > 0, "located first: people walk");
    assert!(
        geometry * 10 <= strides,
        "{geometry} of {strides} strides stopped by walls and furniture, over 10 %"
    );

    // Every place entered at least 90 % as often as without bodies (judged against the same rooms).
    let without = without_bodies("lakeside-bodies-nw3-without");
    let base = thirty_days(&without, "lakeside-bodies-30-without", rooms);
    let (with, base) = (entries(&report), entries(&base));
    for (place, count) in &base {
        let have = with.get(place).copied().unwrap_or(0);
        println!(
            "entered {place}: {have} with bodies, {count} without ({:.1} %)",
            100.0 * have as f64 / (*count).max(1) as f64
        );
        assert!(
            have * 10 >= count * 9,
            "{place}: entered {have} times with bodies, {count} without: under 90 %"
        );
    }
}

/// One place file, parsed as YAML data.
fn place_doc(world: &Path, key: &str) -> Value {
    let file = world.join("places").join(format!("{key}.yaml"));
    let text = std::fs::read_to_string(&file).expect("reads");
    serde_saphyr::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", file.display()))
}

type Rect = (i32, i32, i32, i32);

fn point(value: &Value) -> (i32, i32) {
    let n = |k: &str| i32::try_from(value[k].as_i64().expect("a number")).expect("an i32");
    (n("x"), n("y"))
}

fn floor(doc: &Value) -> Rect {
    let floor = &doc["body"]["floor"];
    let (min, max) = (point(&floor["min"]), point(&floor["max"]));
    (min.0, min.1, max.0, max.1)
}

/// The outward normal of `rect`'s edge that `p` stands `INSET` inside.
fn outward(rect: Rect, p: (i32, i32), place: &str) -> (i32, i32) {
    let (x0, y0, x1, y1) = rect;
    let edges = [
        (p.0 - x0, (-1, 0)),
        (x1 - p.0, (1, 0)),
        (p.1 - y0, (0, -1)),
        (y1 - p.1, (0, 1)),
    ];
    let near: Vec<(i32, i32)> = edges
        .iter()
        .filter(|(d, _)| *d == INSET)
        .map(|(_, n)| *n)
        .collect();
    assert_eq!(
        near.len(),
        1,
        "{place}: the doorway point {p:?} stands {INSET} mm inside exactly one edge of {rect:?}"
    );
    near[0]
}

/// (4) AC-b-6: every floor carried into the lane's frame (§5.1: `p − here + there − 2·INSET·n`, n the
/// place's outward normal at its door) is disjoint from every other.
#[test]
fn the_village_floors_do_not_overlap() {
    let world = lakeside();
    let places: Vec<String> = read_world(&world)
        .places()
        .keys()
        .map(|key| key.as_str().to_owned())
        .collect();
    let docs: BTreeMap<String, Value> = places
        .iter()
        .map(|key| (key.clone(), place_doc(&world, key)))
        .collect();
    let mut offset: BTreeMap<String, (i32, i32)> = BTreeMap::from([("lane".to_owned(), (0, 0))]);
    for _ in 0..places.len() {
        for (key, doc) in &docs {
            if offset.contains_key(key) {
                continue;
            }
            for passage in doc["passages"].as_array().into_iter().flatten() {
                let to = passage["to"].as_str().expect("a key");
                let Some(&(ox, oy)) = offset.get(to) else {
                    continue;
                };
                let (here, there) = (point(&passage["here"]), point(&passage["there"]));
                outward(floor(&docs[to]), there, to);
                let (nx, ny) = outward(floor(doc), here, key);
                offset.insert(
                    key.clone(),
                    (
                        ox + there.0 - here.0 - 2 * INSET * nx,
                        oy + there.1 - here.1 - 2 * INSET * ny,
                    ),
                );
                break;
            }
        }
    }
    assert_eq!(
        offset.len(),
        places.len(),
        "every place reached from the lane"
    );
    let footprints: Vec<(String, Rect)> = offset
        .iter()
        .map(|(key, (ox, oy))| {
            let (x0, y0, x1, y1) = floor(&docs[key]);
            (key.clone(), (x0 + ox, y0 + oy, x1 + ox, y1 + oy))
        })
        .collect();
    let mut overlaps = Vec::new();
    for (i, (a, ra)) in footprints.iter().enumerate() {
        println!("{a}: lane frame {ra:?}");
        for (b, rb) in &footprints[i + 1..] {
            if ra.0 < rb.2 && rb.0 < ra.2 && ra.1 < rb.3 && rb.1 < ra.3 {
                overlaps.push(format!("{a} {ra:?} overlaps {b} {rb:?}"));
            }
        }
    }
    assert!(overlaps.is_empty(), "{overlaps:#?}");
}

/// (5) AC-b-7 (Q-S33b-1 as ruled): removing `bodies` from `systems:` — and with it every `body:`,
/// which the loader refuses once nothing owns it — gives exactly the world after only moving the
/// doorway points. The expectation is the 30-day, seed-7 summary (every line but `wall`, as
/// `ci_parity.py` keeps it) of `worlds/lakeside` at S33-b's C1 commit (`30b7c521`: the doorway move,
/// before any body), recorded then: evidence from before the code under test. A merged change outside
/// bodies and the doorways that moves the summary re-records it, with a dated `# reason` line naming
/// the commit (the fixture's `#` lines are notes, not output).
#[test]
fn without_bodies_only_the_doorways_moved() {
    let expected: String = lakeside::read_lf(&lakeside::repository(
        "tools/cli/tests/fixtures/lakeside/doorways-only-30d-seed7.txt",
    ))
    .lines()
    .filter(|line| !line.starts_with('#'))
    .map(|line| format!("{line}\n"))
    .collect();
    let without = without_bodies("lakeside-bodies-only");
    let printed: String = run(&without, SEED, DAYS, None)
        .lines()
        .filter(|line| !line.starts_with("wall "))
        .map(|line| format!("{line}\n"))
        .collect();
    assert!(
        printed.contains("faults     0"),
        "located first: the copy lived: {printed}"
    );
    assert!(
        !printed.contains("stopped-short"),
        "nobody is resolved without the system"
    );
    assert_eq!(
        printed, expected,
        "without bodies, Lakeside is the doorway move alone"
    );
}
