//! The towns get bodies (step-11 §19, PR 12d), through the real binary and the real server.
//!
//! ```text
//! TD-2   both towns validate with `bodies`; copies whose doorway, person or object does not fit are
//!        refused by name, with the numbers
//! TD-3   the geometry's one named exception (table north's south face, the TD-D4 rulings) is exactly
//!        the inset the rulings name, and both towns' rooms are the same rooms
//! wall   from carol's home in market-town, a stride through the hall's wall is stopped by the server;
//!        leaving by the disclosed doorway still works (the operator's walk-out-through-the-wall bug)
//! TD-13  an observer in market-town's café is disclosed `item-catalogue`: the twenty kinds, in ItemId
//!        order, with their names; an observer in social-cafe (no `item`) is disclosed none
//! TD-6/7 300 days of each town, seed 7: the physical actions happen and stay rare (counts printed per
//!        30-day bucket), then the scan finds nobody overlapping, in a wall or a solid
//! TD-8   social-cafe without `bodies` runs, nobody is resolved, and the same scan sees the overlaps
//! ```
//!
//! The scan and the rooms are `bodies/mod.rs`'s: read from the save and from the place files' text,
//! never from bodies' code (DO-12).

mod bodies;
mod headless;
mod market;
mod support;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bodies::{
    GAP, R, Report, Room, Rooms, assert_reachable, copy_dir, keys_of, rooms_of, run, scan_in,
    strip_section,
};
use headless::{Tables, fresh, mineworld, stderr, stdout};
use market::Town;
use mineworld_contracts::ActionResult;
use mineworld_server::WireObservation;
use mineworld_test_support::Scratch;
use serde_json::Value;
use support::{Client, Server, stride, walk};

/// A world of this repository, by name, as a path built with `Path::join` (no separator assumed).
fn town(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("worlds")
        .join(name)
}

/// A copy of `name` in a fresh scratch directory, named as the town (a pack's id is its directory).
fn copy_town(name: &str, scratch: &str) -> Scratch {
    let copy = fresh(scratch).within(name);
    copy_dir(&town(name), &copy);
    copy
}

/// Replaces the one occurrence of `from` in the copy's `relative` file by `to`.
fn edit(copy: &Path, relative: &[&str], from: &str, to: &str) {
    let file = relative
        .iter()
        .fold(copy.to_path_buf(), |path, part| path.join(part));
    let text = std::fs::read_to_string(&file).expect("the file reads");
    assert_eq!(
        text.matches(from).count(),
        1,
        "{file:?} holds {from:?} once"
    );
    std::fs::write(&file, text.replace(from, to)).expect("the file writes");
}

/// `mineworld validate` of `pack`: its exit and everything it printed.
fn validate(pack: &Path) -> (bool, String) {
    let output = mineworld(&["validate", pack.to_str().expect("a printable path")]);
    (
        output.status.success(),
        format!("{}{}", stdout(&output), stderr(&output)),
    )
}

/// A copy of social-cafe with one edit is refused, naming every word.
fn refused(case: &str, relative: &[&str], from: &str, to: &str, words: &[&str]) {
    let copy = copy_town("social-cafe", &format!("town-bodies-refused-{case}"));
    edit(&copy, relative, from, to);
    let (success, printed) = validate(&copy);
    println!("{case}: {printed}");
    assert!(!success, "{case}: the copy is refused: {printed}");
    for word in words {
        assert!(printed.contains(word), "{case}: names {word}: {printed}");
    }
}

#[test]
fn both_towns_validate_with_bodies_and_misfits_are_refused_by_name() {
    for name in ["social-cafe", "market-town"] {
        let (success, printed) = validate(&town(name));
        assert!(success, "{name} validates: {printed}");
        assert!(
            printed.contains(", schedule, bodies"),
            "{name}: bodies is installed after schedule: {printed}"
        );
        assert!(
            printed.contains("cafe-ball, cafe-box") && printed.contains("street-ball, street-box"),
            "{name}: the four objects: {printed}"
        );
    }
    let (_, printed) = validate(&town("social-cafe"));
    // 53 before bodies, + six `place-shaped`, four `body-formed`, four `object-placed`.
    assert!(printed.contains("67 genesis fact(s)"), "{printed}");
    assert_eq!(rooms_of(&town("social-cafe")).len(), 6, "six shaped places");

    let doorway = "    here:\n      x: 1610\n      y: 400";
    refused(
        "here",
        &["places", "cafe.yaml"],
        doorway,
        "    here:\n      x: 1610\n      y: 200",
        &["bodies-doorway", "cafe", "street", "(1610, 200)", "200 mm"],
    );
    refused(
        "there",
        &["places", "cafe.yaml"],
        "    there:\n      x: 0\n      y: 2800",
        "    there:\n      x: 0\n      y: 3100",
        &["bodies-doorway", "street", "(0, 3100) in street", "100 mm"],
    );
    refused(
        "in-the-counter",
        &["places", "cafe.yaml"],
        doorway,
        "    here:\n      x: 5000\n      y: 6700",
        &["bodies-doorway", "cafe", "0 mm from a solid"],
    );
    // bob 170 mm from the counter's face (y 6 570), inside a radius (250).
    refused(
        "bob-in-the-counter",
        &["people", "bob.yaml"],
        "    x: 4500\n    y: 6100",
        "    x: 4500\n    y: 6400",
        &["bodies-in-solid", "bob", "cafe", "170 mm"],
    );
    refused(
        "visitor-on-the-wanderer",
        &["people", "visitor.yaml"],
        "    x: 1610\n    y: 600",
        "    x: 7110\n    y: 3900",
        &["bodies-overlap", "visitor", "wanderer", "cafe"],
    );
    refused(
        "ball-on-bob",
        &["items", "cafe-ball.yaml"],
        "x: 2600, y: 5600",
        "x: 4500, y: 6100",
        &["bodies-object-on-person", "cafe-ball", "bob"],
    );
}

/// The one named exception to "a solid is its slice collider" (step-11 §19.13, the TD-D4 rulings):
/// the café's table north, its south face only, inset so the wanderer at (7 110, 3 900) keeps a radius
/// and the gap. The slice's collider is E-TD1's (6 673, 4 123)–(7 547, 4 997), top 716.
#[test]
fn the_geometry_departs_from_the_slice_only_where_the_rulings_name() {
    const SLICE_TABLE_NORTH: ((i32, i32, i32, i32), i32) = ((6_673, 4_123, 7_547, 4_997), 716);
    const WANDERER_Y: i32 = 3_900;
    let social = rooms_of(&town("social-cafe"));
    let market = rooms_of(&town("market-town"));
    assert_eq!(
        social, market,
        "both towns' rooms are the same rooms (check 3)"
    );
    let cafe: &Room = &social["cafe"];
    let authored = cafe
        .solids
        .iter()
        .find(|((x0, _, x1, y1), _)| (*x0, *x1, *y1) == (6_673, 7_547, 4_997))
        .expect("table north is authored");
    let ((x0, y0, x1, y1), height) = SLICE_TABLE_NORTH;
    let face = WANDERER_Y + R + GAP;
    assert_eq!(
        *authored,
        ((x0, face, x1, y1), height),
        "only the south face moves, to the wanderer's y + R + GAP"
    );
    assert_eq!(
        face - y0,
        37,
        "the inset the ruling names: 3 900 + 250 + 10 − 4 123"
    );
    assert!(
        social["street"].solids.len() == 62,
        "the street's 62 solids (the elevated bank tree omitted): {}",
        social["street"].solids.len()
    );
}

/// The ids and the street's doorway for carol's walk, read from the pack.
fn doorway(pack: &Path, place: &str) -> ((i32, i32), (i32, i32)) {
    let read = mineworld_worldpack::WorldPack::read(pack).expect("reads");
    let key = mineworld_contracts::EntityKey::new(place).expect("a key");
    let passage = read.places()[&key]
        .passages
        .iter()
        .find(|passage| passage.to.as_str() == "street")
        .expect("onto the street");
    let point = |position: Option<mineworld_worldpack::AuthoredPosition>| {
        let position = position.expect("positioned");
        (position.x.value(), position.y.value())
    };
    (point(passage.here), point(passage.there))
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

/// The operator's bug, fixed on the server: from carol's home in market-town, a stride through the
/// hall's south wall is accepted only as far as the wall (a radius and the gap short of it) and she
/// is still at home; then she walks to the disclosed doorway and out onto the street.
#[tokio::test]
async fn from_carols_home_a_stride_through_the_wall_stops_at_it_and_the_door_still_works() {
    let pack = town("market-town");
    let ids = Town::read(&pack);
    let (home, street) = (ids.id("apartments"), ids.id("street"));
    let server = Server::start(&["server", pack.to_str().expect("a path")]).await;
    let mut client = Client::connect(server.address).await;
    let (carol, _) = client.join("carol").await;
    let (place, at) = standing(&client.observation().await);
    assert_eq!((place, at), (home.raw(), (3_000, 2_500)), "carol at home");

    // Toward the south wall (y 0) and 1 m through it, in two strides of at most 2 m.
    let (_, first) = client.submit(stride(carol, home, 3_000, 700)).await;
    assert!(matches!(first, ActionResult::Accepted { .. }), "{first:?}");
    let (_, through) = client.submit(stride(carol, home, 3_000, -1_000)).await;
    let seen = client
        .observation_where("carol's stride answered", |o| standing(o).1 != (3_000, 700))
        .await;
    let (place, (x, y)) = standing(&seen);
    println!("through the wall: {through:?}; carol at {place} ({x}, {y})");
    assert_eq!(
        place,
        home.raw(),
        "still at home: nobody walks out through a wall"
    );
    assert!(
        y >= R && x == 3_000,
        "stopped inside the hall, a radius from the wall or more: ({x}, {y})"
    );
    match through {
        ActionResult::Accepted { .. } => assert!(y < 700, "she moved toward the wall: {y}"),
        other => println!("the server refused the stride instead: {other:?}"),
    }

    // Out by the disclosed doorway: to its point inside, then across to the street's side.
    let (inside, outside) = doorway(&pack, "apartments");
    client
        .walk_accepted(walk(carol, home, (x, y), inside))
        .await;
    client
        .submit_accepted(stride(carol, street, outside.0, outside.1))
        .await;
    let seen = client
        .observation_where("carol on the street", |o| standing(o).0 == street.raw())
        .await;
    println!("out of the door: {:?}", standing(&seen));
    assert_eq!(
        standing(&seen),
        (street.raw(), outside),
        "on the street, at its doorway"
    );
}

/// The `item-catalogue` record on the observer's place, if any.
fn catalogue(observation: &WireObservation) -> Option<Value> {
    observation
        .entities()
        .iter()
        .flat_map(|entity| entity.components())
        .find(|record| record.component_type().as_str() == "item-catalogue")
        .map(|record| record.payload().clone())
}

/// TD-13: market-town's café is disclosed the twenty kinds, in ItemId order, as authored; social-cafe,
/// which does not install `item`, discloses none.
#[tokio::test]
async fn an_observer_in_the_cafe_is_disclosed_the_catalogue_of_kinds() {
    const NAMES: [(&str, &str, &str); 20] = [
        ("apple", "food", "Apple"),
        ("book", "goods", "Book"),
        ("bread", "food", "Bread"),
        ("cake", "food", "Cake"),
        ("candle", "goods", "Candle"),
        ("coffee", "drink", "Coffee"),
        ("croissant", "food", "Croissant"),
        ("flowers", "goods", "Flowers"),
        ("juice", "drink", "Juice"),
        ("milk", "drink", "Milk"),
        ("mug", "goods", "Mug"),
        ("newspaper", "goods", "Newspaper"),
        ("notebook", "goods", "Notebook"),
        ("pen", "goods", "Pen"),
        ("sandwich", "food", "Sandwich"),
        ("scarf", "goods", "Scarf"),
        ("soap", "goods", "Soap"),
        ("soup", "food", "Soup"),
        ("tea", "drink", "Tea"),
        ("umbrella", "goods", "Umbrella"),
    ];
    let pack = town("market-town");
    let ids = Town::read(&pack);
    let server = Server::start(&["server", pack.to_str().expect("a path")]).await;
    let mut client = Client::connect(server.address).await;
    client.join("visitor").await;
    let disclosed = catalogue(&client.observation().await).expect("a catalogue on the café");
    let kinds = disclosed["kinds"].as_array().expect("kinds");
    let seen: Vec<(String, String, String)> = kinds
        .iter()
        .map(|kind| {
            let entity: mineworld_contracts::EntityId =
                serde_json::from_value(kind["item"]["entity"].clone()).expect("an id");
            (
                ids.key(entity),
                kind["category"].as_str().expect("a category").to_owned(),
                kind["name"].as_str().expect("a name").to_owned(),
            )
        })
        .collect();
    let expected: Vec<(String, String, String)> = NAMES
        .iter()
        .map(|(key, category, name)| {
            (
                (*key).to_owned(),
                (*category).to_owned(),
                (*name).to_owned(),
            )
        })
        .collect();
    assert_eq!(
        seen, expected,
        "the twenty kinds, in ItemId (key) order, as authored"
    );

    let social = town("social-cafe");
    let server = Server::start(&["server", social.to_str().expect("a path")]).await;
    let mut client = Client::connect(server.address).await;
    client.join("visitor").await;
    assert_eq!(
        catalogue(&client.observation().await),
        None,
        "a world without item discloses no catalogue"
    );
}

/// Every seat of a town.
fn seats_of(pack: &Path) -> Vec<String> {
    Town::read(pack).seats
}

/// TD-6, per 30-day bucket and over the run; prints the counts first.
fn assert_bodies_happen_and_stay_rare(name: &str, report: &Report, seats: &[String]) {
    let thirty: Vec<Vec<&bodies::Bucket>> = report
        .buckets
        .chunks(3)
        .map(|chunk| chunk.iter().collect())
        .collect();
    let (mut kicks, mut throws, mut shoves, mut pushes) = (0, 0, 0, 0);
    for (index, buckets) in thirty.iter().enumerate() {
        let sum = |f: &dyn Fn(&bodies::Bucket) -> u64| buckets.iter().map(|b| f(b)).sum::<u64>();
        let (stopped, displaced) = (sum(&|b| b.stopped), sum(&|b| b.displaced));
        let (k, t, s, p) = (
            sum(&|b| b.kicks),
            sum(&|b| b.throws),
            sum(&|b| b.shoves),
            sum(&|b| b.pushes),
        );
        let mut physical: BTreeMap<&str, u64> = BTreeMap::new();
        for bucket in buckets {
            for (seat, count) in &bucket.physical {
                *physical.entry(seat.as_str()).or_default() += count;
            }
        }
        println!(
            "{name} days {}-{}: stopped-short {stopped}, displaced {displaced}, kicks {k}, throws \
             {t}, shoves {s}, pushes {p}; physical per seat {physical:?}",
            index * 30 + 1,
            index * 30 + 30
        );
        assert!(stopped > 0, "{name}: a stopped-short in bucket {index}");
        assert!(
            displaced > 0,
            "{name}: a displaced arrival in bucket {index}"
        );
        for seat in seats {
            let count = physical.get(seat.as_str()).copied().unwrap_or(0);
            assert!(
                count <= 288,
                "{name}: {seat} did {count} kicks, throws and shoves in bucket {index}"
            );
        }
        (kicks, throws, shoves, pushes) = (kicks + k, throws + t, shoves + s, pushes + p);
    }
    println!(
        "{name} over 300 days: kicks {kicks}, throws {throws}, shoves {shoves}, pushes {pushes}"
    );
    assert!(
        kicks > 0 && throws > 0 && shoves > 0 && pushes > 0,
        "{name}: each happens"
    );
}

/// TD-6 and TD-7 for one town: 300 days, seed 7, saved; activity first, then the scan.
fn a_town_lives_with_bodies(name: &str) {
    let pack = town(name);
    let save = fresh(&format!("town-bodies-{name}-300"));
    let printed = run(&pack, 7, 300, Some(&save));
    assert!(printed.contains("faults     0"), "{name}: faults 0");
    let rooms: Rooms = rooms_of(&pack);
    let report = scan_in(&Tables::read(&save), &keys_of(&pack), rooms);
    assert_bodies_happen_and_stay_rare(name, &report, &seats_of(&pack));
    assert_reachable(&report);
    println!(
        "{name}: {} requests; closest pair {}; closest person–object {}",
        report.requests,
        report.located(),
        report.located_object()
    );
    assert!(
        report.violations.is_empty(),
        "{name}: {} violations, the first: {:#?}",
        report.violations.len(),
        &report.violations[..report.violations.len().min(5)]
    );
}

#[test]
fn social_cafe_lives_with_bodies_for_300_days_and_nobody_overlaps() {
    a_town_lives_with_bodies("social-cafe");
}

#[test]
fn market_town_lives_with_bodies_for_300_days_and_nobody_overlaps() {
    a_town_lives_with_bodies("market-town");
}

/// TD-8 (I-10): social-cafe without `bodies` and without every `body:` runs 30 days, nobody is
/// resolved, and the scan — against the real town's rooms — sees what bodies prevents.
#[test]
fn without_bodies_the_town_runs_and_the_scan_sees_overlaps() {
    let copy = copy_town("social-cafe", "town-bodies-without");
    edit(&copy, &["world.yaml"], "  - bodies\n", "");
    for directory in ["places", "items"] {
        for entry in std::fs::read_dir(copy.join(directory)).expect("lists") {
            let file = entry.expect("an entry").path();
            let (kept, found) =
                strip_section(&std::fs::read_to_string(&file).expect("reads"), "body");
            assert!(found, "{file:?} carried a body:");
            std::fs::write(&file, kept).expect("writes");
        }
    }
    let save = fresh("town-bodies-without-save");
    let printed = run(&copy, 7, 30, Some(&save));
    assert!(printed.contains("faults     0"), "faults 0: {printed}");
    for word in [
        "kick",
        "throw",
        "shove",
        "object-moved",
        "person-shoved",
        "stopped-short",
    ] {
        assert!(!printed.contains(word), "no {word} without bodies");
    }
    let report = scan_in(
        &Tables::read(&save),
        &keys_of(&copy),
        rooms_of(&town("social-cafe")),
    );
    for (index, bucket) in report.buckets.iter().enumerate() {
        assert_eq!((bucket.stopped, bucket.displaced), (0, 0), "bucket {index}");
        for seat in seats_of(&copy) {
            assert!(
                bucket.moves.get(&seat).copied().unwrap_or(0) > 0,
                "{seat} moved in bucket {index}"
            );
        }
    }
    println!(
        "without bodies: {} violations, the first {:?}; closest {}",
        report.violations.len(),
        report.violations.first(),
        report.located()
    );
    assert!(!report.violations.is_empty(), "the scan sees a violation");
}
