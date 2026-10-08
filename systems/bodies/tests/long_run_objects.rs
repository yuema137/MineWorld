//! PO-13 a — the prototype's long run with its four props, through the real pipeline (step-11 §9.5,
//! §18.4): 3 000 requests of the prototype's sequence, fifteen people in the café, every request
//! through `World::dispatch`, the prototype's kicks as `kick` requests.
//!
//! ```text
//! props    two boxes half 200 at (2 000, 5 500) and (6 000, 5 000); a ball r 110 at (4 000, 5 800);
//!          a crate half 300 at (1 000, 8 500) — the prototype's PROPS
//! kicks    request r is a kick of prop draw(7, r, 0) % 4 (the prototype's own draw) by the person
//!          nearest it; one beyond reach first steps toward it, to 750 mm from its centre, in one
//!          `move` of at most 2 000 mm (§18.11 DO-15)
//! after    every request: N-1 … N-4 for people, and every object keeps SD-O2's invariant
//! digest   every fact and the final state are byte-identical in a second process
//! cost     printed, judged in release (`cargo test --release … -- --nocapture`, PO-13 a): the mean
//!          time per move that reached Rapier ≤ 100 µs, per accepted kick ≤ 2 000 µs
//! ```

mod support;

use std::time::{Duration, Instant};

use mineworld_bodies::{Kick, Route, explain};
use mineworld_contracts::{ActionRecord, EntityType, PersonId};
use support::{Plan, Thing, Xy, Yard, assert_holds, ball, cafe, cube, distance2, encode};

const SEED: u64 = 7;
const REQUESTS: u64 = 3_000;
const PEOPLE: usize = 15;
const KEYS: [&str; PEOPLE] = [
    "p00", "p01", "p02", "p03", "p04", "p05", "p06", "p07", "p08", "p09", "p10", "p11", "p12",
    "p13", "p14",
];
const PROPS: [&str; 4] = ["prop0", "prop1", "prop2", "prop3"];
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

enum Request {
    Move(usize, Xy),
    Kick(usize),
}

/// Request `r`, as the prototype's `request_q` draws it.
fn request(r: u64) -> Request {
    if draw(3, r, 0) % 100 < 85 {
        let person = usize::try_from(draw(4, r, 0) % 15).expect("small");
        let (ux, uy) = DIRS[usize::try_from(draw(5, r, 0) % 8).expect("small")];
        let length = 500 + i32::try_from(draw(6, r, 0) % 1_501).expect("small");
        Request::Move(person, (ux * length / 1_000, uy * length / 1_000))
    } else {
        Request::Kick(usize::try_from(draw(7, r, 0) % 4).expect("small"))
    }
}

fn start(i: usize) -> Xy {
    let (column, row) = (
        i32::try_from(i % 5).expect("small"),
        i32::try_from(i / 5).expect("small"),
    );
    (800 + column * 1_500, 1_000 + row * 1_600)
}

fn props() -> Vec<Thing> {
    vec![
        ("prop0", cube(200), "room", (2_000, 5_500)),
        ("prop1", cube(200), "room", (6_000, 5_000)),
        ("prop2", ball(110), "room", (4_000, 5_800)),
        ("prop3", cube(300), "room", (1_000, 8_500)),
    ]
}

struct Run {
    bytes: String,
    swept: u32,
    swept_time: Duration,
    kicks: u32,
    kicks_refused: u32,
    kick_time: Duration,
    closest_object: (i64, &'static str, &'static str, u64),
}

/// Times one walk and says whether it reached Rapier.
fn walk(yard: &mut Yard, who: &'static str, to: Xy, run: &mut Run, r: u64) {
    let target = yard.at("room", to);
    let id = PersonId::new(yard.people[who], EntityType::Person).expect("a person");
    let outcome = explain(&yard.world.read(), id, target).expect("a shaped place");
    let began = Instant::now();
    let moved = yard.walk(who, target);
    let took = began.elapsed();
    assert!(
        moved.accepted(),
        "request {r}: {who} to {to:?} from {:?}: {:?}",
        yard.point(who),
        moved.result
    );
    if outcome.route != Route::Clear {
        run.swept += 1;
        run.swept_time += took;
    }
}

fn run() -> Run {
    let people: Vec<(&'static str, Xy)> = KEYS
        .iter()
        .enumerate()
        .map(|(i, key)| (*key, start(i)))
        .collect();
    let mut yard = Yard::new(&Plan {
        objects: props(),
        ..Plan::room(cafe(), &people)
    });
    let mut run = Run {
        bytes: String::new(),
        swept: 0,
        swept_time: Duration::ZERO,
        kicks: 0,
        kicks_refused: 0,
        kick_time: Duration::ZERO,
        closest_object: (i64::MAX, "", "", 0),
    };
    let mut facts = Vec::new();
    for r in 0..REQUESTS {
        let before = facts.len();
        match request(r) {
            Request::Move(person, (dx, dy)) => {
                let who = KEYS[person];
                let (x, y) = yard.point(who).expect("placed");
                walk(&mut yard, who, (x + dx, y + dy), &mut run, r);
            }
            Request::Kick(prop) => {
                let (px, py, _) = yard.object(PROPS[prop]);
                let who = *KEYS
                    .iter()
                    .min_by_key(|key| {
                        (distance2(yard.point(key).expect("placed"), (px, py)), **key)
                    })
                    .expect("people");
                let (x, y) = yard.point(who).expect("placed");
                let d2 = distance2((x, y), (px, py));
                if d2 > 800 * 800 {
                    // One step toward the prop, to 750 mm from its centre, at most 2 000 mm.
                    let d = d2.isqrt();
                    // 1 999, not 2 000: |d| is rounded down, so a 2 000 mm step could come out a
                    // fraction longer than movement's MAX_STRIDE.
                    let along = (d - 750).min(1_999);
                    let step = |v: i32| i32::try_from(i64::from(v) * along / d).expect("fits");
                    walk(
                        &mut yard,
                        who,
                        (x + step(px - x), y + step(py - y)),
                        &mut run,
                        r,
                    );
                }
                let record = ActionRecord::new::<Kick>(encode(&Kick::new(yard.items[PROPS[prop]])));
                let began = Instant::now();
                let kicked = yard.submit(who, record);
                let took = began.elapsed();
                if kicked.accepted() {
                    run.kicks += 1;
                    run.kick_time += took;
                } else {
                    run.kicks_refused += 1;
                }
                facts.extend(kicked.events);
            }
        }
        let _ = before;
        assert_holds(
            &yard,
            "room",
            (0, 0, 8_320, 10_320),
            &[((3_860, 6_570, 8_320, 7_170), 1_100)],
        );
        for (key, at) in yard.standing("room") {
            for prop in PROPS {
                let (px, py, _) = yard.object(prop);
                let d = distance2(at, (px, py));
                if d < run.closest_object.0 {
                    run.closest_object = (d, key, prop, r);
                }
            }
        }
    }
    run.bytes = serde_json::to_string(&(facts, yard.standing("room"), yard.objects("room")))
        .expect("encodes");
    run
}

const SECOND_PROCESS: &str = "BODIES_LONG_RUN_OBJECTS_SECOND_PROCESS";

#[test]
fn the_prototypes_long_run_with_props_keeps_every_bound_and_reproduces_in_a_second_process() {
    if std::env::var_os(SECOND_PROCESS).is_some() {
        println!("LONG-RUN-OBJECTS {}", run().bytes);
        return;
    }
    let ours = run();
    let (d, person, prop, r) = ours.closest_object;
    println!(
        "long run with props: closest person–object centres {person} and {prop}, {} mm, after \
         request {r}",
        d.isqrt()
    );
    println!(
        "cost: {} moves reached Rapier, mean {:.1} µs each; {} kicks accepted, mean {:.1} µs each; \
         {} kicks refused",
        ours.swept,
        ours.swept_time.as_secs_f64() * 1e6 / f64::from(ours.swept.max(1)),
        ours.kicks,
        ours.kick_time.as_secs_f64() * 1e6 / f64::from(ours.kicks.max(1)),
        ours.kicks_refused,
    );
    assert!(ours.kicks > 0, "kicks were made");
    let output = std::process::Command::new(std::env::current_exe().expect("this test binary"))
        .args([
            "--exact",
            "the_prototypes_long_run_with_props_keeps_every_bound_and_reproduces_in_a_second_process",
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
            line.find("LONG-RUN-OBJECTS ")
                .map(|at| &line[at + "LONG-RUN-OBJECTS ".len()..])
        })
        .expect("the second process printed its run");
    println!(
        "digest: {} bytes here, {} there",
        ours.bytes.len(),
        theirs.len()
    );
    assert_eq!(ours.bytes, theirs, "byte-identical in two processes");
}
