//! PB-14 (a) — the prototype's long run through the real pipeline: 3 000 requests of the prototype's
//! sequence, its kicks left out, fifteen people in the café, every move through `World::dispatch`
//! (step-11 §9.1, §9.5, §17.4).
//!
//! ```text
//! N-1  after every request no two people are closer than 595 mm; the closest pair is located
//! N-2  every nudge is at most 310 mm
//! N-3  at most two generations and four people per request
//! N-4  nobody outside the floor shrunk by 295 mm or within 295 mm of the counter
//! digest  every fact and the final state are byte-identical in a second process
//! cost    printed, not judged here: the mean time per move that reached Rapier, and the share of
//!         moves the integer fast path answered (`cargo test --release … -- --nocapture` for the
//!         number PB-14 records)
//! ```
//!
//! The sequence is re-derived from the prototype's `mix` and `draw` (seed 7); its unit directions,
//! literals there, are thousandths here, each component truncated as the prototype's `as i32` did.

mod support;

use std::time::{Duration, Instant};

use mineworld_bodies::{Route, explain};
use mineworld_contracts::{EntityType, PersonId};
use support::{Plan, Xy, Yard, cafe, distance2, xy};

const SEED: u64 = 7;
const REQUESTS: u64 = 3_000;
const PEOPLE: usize = 15;
const KEYS: [&str; PEOPLE] = [
    "p00", "p01", "p02", "p03", "p04", "p05", "p06", "p07", "p08", "p09", "p10", "p11", "p12",
    "p13", "p14",
];
/// The prototype's eight directions, in thousandths.
const DIRS: [Xy; 8] = [
    (1_000, 0),
    (707, 707),
    (0, 1_000),
    (-707, 707),
    (-1_000, 0),
    (-707, -707),
    (0, -1_000),
    (707, -707),
];

fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn draw(a: u64, b: u64, c: u64) -> u64 {
    mix(mix(mix(SEED ^ a) ^ b) ^ c)
}

/// Request `r`: a move of one person by an offset, or [`None`] where the prototype kicked a prop.
fn request(r: u64) -> Option<(usize, Xy)> {
    if draw(3, r, 0) % 100 >= 85 {
        return None;
    }
    let person = usize::try_from(draw(4, r, 0) % 15).expect("small");
    let (ux, uy) = DIRS[usize::try_from(draw(5, r, 0) % 8).expect("small")];
    let length = 500 + i32::try_from(draw(6, r, 0) % 1_501).expect("small");
    Some((person, (ux * length / 1_000, uy * length / 1_000)))
}

/// The prototype's starting grid: five across, three deep.
fn start(i: usize) -> Xy {
    let (column, row) = (
        i32::try_from(i % 5).expect("small"),
        i32::try_from(i / 5).expect("small"),
    );
    (800 + column * 1_500, 1_000 + row * 1_600)
}

fn inside_the_cafe((x, y): Xy) -> bool {
    let in_floor = (295..=8_320 - 295).contains(&x) && (295..=10_320 - 295).contains(&y);
    let dx = (3_860 - x).max(x - 8_320).max(0);
    let dy = (6_570 - y).max(y - 7_170).max(0);
    in_floor && i64::from(dx) * i64::from(dx) + i64::from(dy) * i64::from(dy) >= 295 * 295
}

/// The run's bytes (every fact, then the final positions) and what it measured.
struct Run {
    bytes: String,
    swept: u32,
    clear: u32,
    swept_time: Duration,
    closest: (i64, &'static str, &'static str, u64),
}

fn run() -> Run {
    let people: Vec<(&'static str, Xy)> = KEYS
        .iter()
        .enumerate()
        .map(|(i, key)| (*key, start(i)))
        .collect();
    let mut yard = Yard::new(&Plan::room(cafe(), &people));
    let mut facts = Vec::new();
    let (mut swept, mut clear, mut swept_time) = (0, 0, Duration::ZERO);
    let mut closest = (i64::MAX, "", "", 0);
    for r in 0..REQUESTS {
        let Some((person, (dx, dy))) = request(r) else {
            continue;
        };
        let walker = KEYS[person];
        let (x, y) = yard.point(walker).expect("placed");
        let to = yard.at("room", (x + dx, y + dy));
        let id = PersonId::new(yard.people[walker], EntityType::Person).expect("a person");
        let outcome = explain(&yard.world.read(), id, to).expect("a shaped place");
        let before = yard.standing("room");
        let began = Instant::now();
        let moved = yard.walk(walker, to);
        let took = began.elapsed();
        assert!(moved.accepted(), "request {r}: {:?}", moved.result);
        if outcome.route == Route::Clear {
            clear += 1;
        } else {
            swept += 1;
            swept_time += took;
        }
        let displaced = moved.displaced();
        assert!(
            displaced.len() <= 4 && outcome.generations <= 2,
            "N-3, request {r}: {outcome:?}"
        );
        for (other, at) in &displaced {
            let key = yard.key_of(*other);
            let was = before
                .iter()
                .find(|(k, _)| *k == key)
                .expect("stood there")
                .1;
            let length2 = distance2(was, xy(*at));
            assert!(
                length2 <= 310 * 310,
                "N-2, request {r}: {key} nudged {} mm",
                length2.isqrt()
            );
        }
        if let Some((a, b, d)) = yard.closest("room") {
            assert!(
                d >= 595 * 595,
                "N-1, request {r}: {a} and {b} {} mm apart",
                d.isqrt()
            );
            if d < closest.0 {
                closest = (d, a, b, r);
            }
        }
        for (key, at) in yard.standing("room") {
            assert!(inside_the_cafe(at), "N-4, request {r}: {key} at {at:?}");
        }
        facts.extend(moved.events);
    }
    let bytes = serde_json::to_string(&(facts, yard.standing("room"))).expect("encodes");
    Run {
        bytes,
        swept,
        clear,
        swept_time,
        closest,
    }
}

const SECOND_PROCESS: &str = "BODIES_LONG_RUN_SECOND_PROCESS";

#[test]
fn the_prototypes_long_run_keeps_every_bound_and_reproduces_in_a_second_process() {
    if std::env::var_os(SECOND_PROCESS).is_some() {
        println!("LONG-RUN {}", run().bytes);
        return;
    }
    let ours = run();
    let moves = ours.swept + ours.clear;
    let (d, a, b, r) = ours.closest;
    println!(
        "long run: {moves} moves; closest pair {a} and {b}, {} mm, after request {r}",
        d.isqrt()
    );
    println!(
        "cost: {} moves reached Rapier, mean {:.1} µs per such move; fast path answered {} ({:.1} %)",
        ours.swept,
        ours.swept_time.as_secs_f64() * 1e6 / f64::from(ours.swept.max(1)),
        ours.clear,
        f64::from(ours.clear) * 100.0 / f64::from(moves.max(1)),
    );
    let output = std::process::Command::new(std::env::current_exe().expect("this test binary"))
        .args([
            "--exact",
            "the_prototypes_long_run_keeps_every_bound_and_reproduces_in_a_second_process",
            "--nocapture",
        ])
        .env(SECOND_PROCESS, "1")
        .output()
        .expect("the second process runs");
    assert!(output.status.success(), "the second process succeeded");
    let printed = String::from_utf8(output.stdout).expect("utf-8");
    let theirs = printed
        .lines()
        .find_map(|line| {
            line.find("LONG-RUN ")
                .map(|at| &line[at + "LONG-RUN ".len()..])
        })
        .expect("the second process printed its run");
    println!(
        "digest: {} bytes here, {} in the second process",
        ours.bytes.len(),
        theirs.len()
    );
    assert_eq!(ours.bytes, theirs, "byte-identical in two processes");
}
