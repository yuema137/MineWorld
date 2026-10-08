//! ZR-3 — the shadow comparison over long_run's 3 000 requests (step-11 §20.4 and its amendment).
//!
//! Every request production answers by integers where the Rapier path would not — Walled (SD-Z4), or
//! Clear where the old box corridor was not (SD-Z3) — is also answered by the Rapier path on the same
//! state (`shadow`), and the two ends are compared. PASS iff at most 1 % of them differ by more than
//! 50 mm on an axis; each one that does is printed with its state. V1 … V4 hold for every answer: after
//! each request (production's, which the run then follows) nobody is closer than 595 mm to anybody,
//! nor outside the floor shrunk by 295 mm or within 295 mm of the counter.
//!
//! Ignored (a measurement, run once per re-capture): `cargo test -p mineworld-bodies --test shadow --
//! --ignored --nocapture`. The request sequence is long_run's, copied (its `mix`, `draw`, `request`
//! and starting grid), so that the two files read the same run.

mod support;

use mineworld_bodies::{Route, shadow};
use mineworld_contracts::{EntityType, PersonId};
use support::{Plan, Xy, Yard, cafe, distance2};

const SEED: u64 = 7;
const REQUESTS: u64 = 3_000;
const KEYS: [&str; 15] = [
    "p00", "p01", "p02", "p03", "p04", "p05", "p06", "p07", "p08", "p09", "p10", "p11", "p12",
    "p13", "p14",
];
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

fn request(r: u64) -> Option<(usize, Xy)> {
    if draw(3, r, 0) % 100 >= 85 {
        return None;
    }
    let person = usize::try_from(draw(4, r, 0) % 15).expect("small");
    let (ux, uy) = DIRS[usize::try_from(draw(5, r, 0) % 8).expect("small")];
    let length = 500 + i32::try_from(draw(6, r, 0) % 1_501).expect("small");
    Some((person, (ux * length / 1_000, uy * length / 1_000)))
}

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

#[test]
#[ignore = "ZR-3, a measurement: run once per re-capture with --ignored --nocapture"]
fn integer_answers_stay_within_50_mm_of_rapiers() {
    let people: Vec<(&'static str, Xy)> = KEYS
        .iter()
        .enumerate()
        .map(|(i, key)| (*key, start(i)))
        .collect();
    let mut yard = Yard::new(&Plan::room(cafe(), &people));
    let (mut walled, mut cleared, mut beyond) = (0_u32, 0_u32, 0_u32);
    let (mut worst, mut total) = (0_i32, 0_i64);
    for r in 0..REQUESTS {
        let Some((person, (dx, dy))) = request(r) else {
            continue;
        };
        let walker = KEYS[person];
        let (x, y) = yard.point(walker).expect("placed");
        let to = yard.at("room", (x + dx, y + dy));
        let id = PersonId::new(yard.people[walker], EntityType::Person).expect("a person");
        if let Some(compared) = shadow(&yard.world.read(), id, to) {
            match compared.route {
                Route::Walled => walled += 1,
                _ => cleared += 1,
            }
            let axis = (compared.integer.0 - compared.rapier.0)
                .abs()
                .max((compared.integer.1 - compared.rapier.1).abs());
            worst = worst.max(axis);
            total += i64::from(axis);
            if axis > 50 {
                beyond += 1;
                println!(
                    "ZR-3 > 50 mm: request {r}, {walker} at ({x}, {y}) to ({}, {}): {compared:?}; standing {:?}",
                    x + dx,
                    y + dy,
                    yard.standing("room")
                );
            }
        }
        assert!(yard.walk(walker, to).accepted(), "request {r}");
        let standing = yard.standing("room");
        for (i, (a, at)) in standing.iter().enumerate() {
            assert!(inside_the_cafe(*at), "V2–V3, request {r}: {a} at {at:?}");
            for (b, other) in &standing[i + 1..] {
                assert!(
                    distance2(*at, *other) >= 595 * 595,
                    "V1, request {r}: {a} and {b}"
                );
            }
        }
    }
    let compared = walled + cleared;
    println!(
        "ZR-3 long_run: {compared} compared ({walled} Walled, {cleared} newly Clear); per-axis \
         difference max {worst} mm, mean {:.1} mm; {beyond} above 50 mm",
        if compared == 0 {
            0.0
        } else {
            f64::from(i32::try_from(total).expect("small")) / f64::from(compared)
        }
    );
    assert!(
        u64::from(beyond) * 100 <= u64::from(compared),
        "ZR-3: {beyond} of {compared} differ by more than 50 mm"
    );
}
