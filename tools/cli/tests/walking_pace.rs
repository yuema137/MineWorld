//! NW-9 — **embodied pace is wall-clock, whatever the time scale** (step-11 §21.8 NW-9, SD-N14;
//! `DECISIONS.md` `ARC-42` note of 2026-10-09, `ARC-75` item 7; step-19 §4, QTW-13).
//!
//! ```text
//! mineworld server <street-and-shop> --town --time-scale s      s = 1, 6, 12, 24, one after another
//! the walker   driven in-server by the paced rule controller; its day says "the shop", whose door is
//!              28 m along the street from where it stands: it asks walk-to, then a stride a wall second
//! the watcher  a seat this test joins and never moves; it receives the street's observations at
//!              10 Hz and records the walker's position and the wall instant of every frame
//! ```
//!
//! Speed = straight-line displacement between consecutive changed positions ÷ the wall time between
//! the frames that showed them; the median over the walk. PASS iff every scale's median is
//! 1.34 m/s ± 15 % (one 1 340 mm stride per wall second, with tolerance for tick and frame jitter),
//! the medians at 6×, 12× and 24× lie within 10 % of each other, and the world time between two
//! strides is the scale (± 15 %) — one stride per `s` world seconds, which shows that the scale was
//! really applied (`ARC-23`). Mutation M-N7 (the hosted step stated as one *world* second) makes the
//! 24× walker several times faster than the 6× one, and this test fails naming the scales.
//!
//! The world is written by the test: a street and a shop, with presence, movement and schedule —
//! the smallest world in which a paced person walks 20 m or more toward their day (`N-D23`).

mod support;

use std::path::Path;
use std::time::{Duration, Instant};

use mineworld_contracts::EntityId;
use mineworld_server::WireObservation;
use support::{Client, Server};

/// How long one scale may take, in wall time: the walk is 21 strides, about 21 wall seconds.
const BOUND: Duration = Duration::from_secs(180);
/// The walk's stride, millimetres per wall second: movement's `WALK_STRIDE` at one step a wall second.
const EXPECTED_MM_PER_S: f64 = 1_340.0;

/// Writes the street-and-shop world into `directory` and returns the pack's path.
fn world(directory: &Path) -> std::path::PathBuf {
    let pack = directory.join("pace-street");
    for sub in ["places", "people"] {
        std::fs::create_dir_all(pack.join(sub)).expect("a directory");
    }
    std::fs::write(
        pack.join("world.yaml"),
        "world:\n  id: pace-street\n  name: Pace Street\n  version: 0.1.0\n  license: MIT\n\
         mineworld: \"^0.1\"\nsystems:\n  - presence\n  - movement\n  - schedule\nplaces:\n  \
         - shop\n  - street\npopulation:\n  - walker\n  - watcher\nseats:\n  - walker\n  \
         - watcher\n",
    )
    .expect("world.yaml");
    std::fs::write(pack.join("places/street.yaml"), "tags:\n  - street\n").expect("street");
    std::fs::write(
        pack.join("places/shop.yaml"),
        "tags:\n  - shop\npassages:\n  - to: street\n    here: { x: 1500, y: 200 }\n    \
         there: { x: 12000, y: 3000 }\n",
    )
    .expect("shop");
    // The walker's whole day is the shop; it starts 28 m east of the shop's street door.
    std::fs::write(
        pack.join("people/walker.yaml"),
        "routine:\n  - { from: \"00:00\", place: shop, label: work }\n  \
         - { from: \"12:00\", place: shop, label: stock }\ntags:\n  - walker\n\
         location:\n  place: street\n  position:\n    x: 40000\n    y: 3000\n",
    )
    .expect("walker");
    // The watcher stands on the street, out of the way, for the whole day.
    std::fs::write(
        pack.join("people/watcher.yaml"),
        "routine:\n  - { from: \"00:00\", place: street, label: watch }\n  \
         - { from: \"12:00\", place: street, label: wait }\ntags:\n  - watcher\n\
         location:\n  place: street\n  position:\n    x: 26000\n    y: 9000\n",
    )
    .expect("watcher");
    pack
}

/// One frame as the watcher saw it: when (wall), the world instant, and the walker's position.
struct Seen {
    wall: Instant,
    world: i64,
    at: (i64, i64),
}

fn walker_at(observation: &WireObservation, walker: EntityId) -> Option<(i64, i64)> {
    let local = observation.entity(walker)?.location()?.local()?;
    Some((i64::from(local.x().value()), i64::from(local.y().value())))
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

/// One scale: the walker's median speed in mm per wall second, and the median world seconds between
/// two of its strides.
async fn walk_at(pack: &Path, scale: u32) -> (f64, f64, usize) {
    let scale_text = scale.to_string();
    let server = Server::start(&[
        "server",
        pack.to_str().expect("a printable path"),
        "--town",
        "--time-scale",
        &scale_text,
    ])
    .await;
    let mut watcher = Client::connect(server.address).await;
    watcher.join("watcher").await;
    let first = watcher.observation().await;
    let walker = support::tagged(&first, "walker").expect("the walker is on the street");
    let started = Instant::now();
    let mut seen: Vec<Seen> = Vec::new();
    loop {
        assert!(
            started.elapsed() < BOUND,
            "scale {scale}: the walk did not end within {BOUND:?} ({} positions seen)",
            seen.len()
        );
        let observation = watcher.observation().await;
        let Some(at) = walker_at(&observation, walker) else {
            // Gone from the street: the walker has entered the shop.
            break;
        };
        if seen.last().is_none_or(|last| last.at != at) {
            seen.push(Seen {
                wall: Instant::now(),
                world: observation.at().seconds(),
                at,
            });
        }
    }
    drop(server);
    let (mut speeds, mut world_gaps) = (Vec::new(), Vec::new());
    for pair in seen.windows(2) {
        let (dx, dy) = (pair[1].at.0 - pair[0].at.0, pair[1].at.1 - pair[0].at.1);
        #[allow(clippy::cast_precision_loss)]
        let travelled = ((dx * dx + dy * dy) as f64).sqrt();
        let wall = pair[1].wall.duration_since(pair[0].wall).as_secs_f64();
        speeds.push(travelled / wall);
        #[allow(clippy::cast_precision_loss)]
        world_gaps.push((pair[1].world - pair[0].world) as f64);
    }
    assert!(
        speeds.len() >= 10,
        "scale {scale}: a walk of 20 m or more, not {} strides",
        speeds.len()
    );
    (median(speeds), median(world_gaps), seen.len())
}

#[tokio::test]
async fn a_hosted_walker_covers_the_same_ground_per_wall_second_at_every_time_scale() {
    let directory = mineworld_test_support::scratch!("walking-pace");
    std::fs::create_dir_all(&*directory).expect("the scratch directory");
    let pack = world(&directory);
    let mut medians = Vec::new();
    for scale in [1_u32, 6, 12, 24] {
        let (speed, world_gap, positions) = walk_at(&pack, scale).await;
        println!(
            "NW-9 scale {scale}: median {speed:.0} mm per wall second, {world_gap:.1} world s \
             between strides, {positions} positions"
        );
        medians.push((scale, speed, world_gap));
    }
    for (scale, speed, world_gap) in &medians {
        assert!(
            (speed - EXPECTED_MM_PER_S).abs() <= EXPECTED_MM_PER_S * 0.15,
            "scale {scale}: median {speed:.0} mm/s, not 1 340 ± 15 % — every scale: {medians:?}"
        );
        let expected_gap = f64::from(*scale);
        assert!(
            (world_gap - expected_gap).abs() <= expected_gap * 0.15,
            "scale {scale}: {world_gap} world seconds between strides, not {expected_gap} ± 15 % — \
             the scale was not applied: {medians:?}"
        );
    }
    let scaled: Vec<f64> = medians
        .iter()
        .filter(|(scale, ..)| *scale > 1)
        .map(|(_, speed, _)| *speed)
        .collect();
    let (slowest, fastest) = (
        scaled.iter().copied().fold(f64::INFINITY, f64::min),
        scaled.iter().copied().fold(0.0, f64::max),
    );
    assert!(
        fastest <= slowest * 1.10,
        "the 6×, 12× and 24× medians are not within 10 % of each other: {medians:?}"
    );
}
