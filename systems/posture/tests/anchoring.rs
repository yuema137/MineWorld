//! A seated person is held where they sat (S34-b; AC-S34-5, I-S34-7; AC-b-1 … AC-b-4, AC-b-6, AC-b-7):
//! a walker aimed through them stops short, an arrival onto them lands beside them, `shove` is refused in
//! dispatch and in the offer, and a sit point where a person does not fit is refused at load in either
//! `systems:` order.
//!
//! Two shaped layouts and a generated one, so nothing here passes because of one layout: the café with a
//! `body:` floor, the pier, and a seeded square of sit points and walkers.
//!
//! What each test would pass without proving, and how it is excluded (`ARC-23`): a walker who simply never
//! reached the sitter (each stride is aimed through the sitter's centre, and the walker is checked to have
//! moved); a shove refused for another reason (the same shove succeeds once the target stands); a load check
//! that runs in only one order (each case is loaded in both); a seeded run whose walkers never come near
//! a sitter (the run counts the strides that ended stopped short, and requires some).

mod support;

use mineworld_bodies::Shove;
use mineworld_contracts::{ActionResult, Location, Rejection};
use mineworld_movement::Move;
use mineworld_worldpack::{AssembledWorld, WorldPack};
use support::{GENESIS, Layout, Town, accepted, at, record, sit, stand, types};

/// The café of `support`, with a 8 000 × 3 000 floor: the same sit points and people.
const CAFE_SHAPED: &str = "\
tags: [cafe]
body:
  floor:
    min: { x: 0, y: 0 }
    max: { x: 8000, y: 3000 }
passages:
  - to: street
    here: { x: 2000, y: 2500 }
    there: { x: 500, y: 500 }
sit-points:
  - { key: bench.1, group: bench, at: { x: 2000, y: 1000 }, facing: 180000 }
  - { key: bench.2, group: bench, at: { x: 2600, y: 1000 }, facing: 180000 }
  - { key: stool.1, at: { x: 6000, y: 1000 }, facing: 0 }
";

/// The café's people of `support`, and ivy in the street on its doorway point.
const CAFE_PEOPLE: &[(&str, &str, i32, i32)] = &[
    ("alice", "cafe", 2_000, 1_500),
    ("bob", "cafe", 4_000, 1_000),
    ("carol", "cafe", 2_600, 1_600),
    ("dave", "cafe", 2_000, 400),
    // 500 mm from frank: in a shaped place people stand at least 495 mm apart.
    ("eve", "cafe", 6_000, 1_200),
    ("frank", "cafe", 6_000, 1_700),
    ("ivy", "street", 500, 500),
];

fn cafe_shaped() -> Layout {
    Layout {
        systems: vec!["presence", "movement", "bodies", "posture"],
        places: vec![("cafe", CAFE_SHAPED), ("street", "tags: [street]\n")],
        people: CAFE_PEOPLE,
        posture: None,
    }
}

fn distance2(a: &Location, b: &Location) -> i64 {
    let (a, b) = (
        a.local().expect("a position"),
        b.local().expect("a position"),
    );
    let dx = i64::from(a.x().value()) - i64::from(b.x().value());
    let dy = i64::from(a.y().value()) - i64::from(b.y().value());
    dx * dx + dy * dy
}

/// People stand at least CLEARANCE = 2 · 250 − 5 mm apart.
const CLEAR: i64 = 495;

/// `walker`, aimed at `target` through `sitter`, seated on `point`: stopped short of them, the sitter
/// unchanged and still seated, nobody stood up.
fn stopped_short_of(town: &mut Town, when: i64, walker: &str, target: (i32, i32), sitter: &str) {
    let place = town.location(walker).place();
    let (seated_at, row) = (town.location(sitter), town.posture(sitter));
    assert!(row.is_some(), "{sitter} is seated");
    let start = town.location(walker);
    let (result, facts) = town.submit(when, walker, record(&Move::new(at(place, target))));
    assert!(accepted(&result), "{result:?}");
    let ended = town.location(walker);
    println!("{walker}: {start:?} → {ended:?}; facts {:?}", types(&facts));
    assert_ne!(ended, start, "{walker} moved");
    assert_ne!(
        ended,
        at(place, target),
        "{walker} did not reach the far side"
    );
    assert!(
        distance2(&ended, &seated_at) >= CLEAR * CLEAR,
        "{walker} keeps clear of {sitter}: {} mm",
        distance2(&ended, &seated_at).isqrt()
    );
    assert_eq!(town.location(sitter), seated_at, "{sitter} was not moved");
    assert_eq!(town.posture(sitter), row, "{sitter} is still seated");
    assert!(
        !types(&facts).contains(&"stood-up".to_owned()),
        "{:?}",
        types(&facts)
    );
}

#[test]
fn a_walker_aimed_through_a_seated_person_stops_short_in_the_cafe_and_on_the_pier() {
    // AC-b-1 (and S34-a's AC-a-9, rewritten by SD-b-8).
    let mut town = Town::begin(&cafe_shaped(), "anchored-walker-cafe");
    let (result, _) = town.submit(10, "alice", sit("bench.1"));
    assert!(accepted(&result), "{result:?}");
    // dave stands 600 mm south of bench.1 and walks 1 200 mm north through it.
    stopped_short_of(&mut town, 20, "dave", (2_000, 1_600), "alice");

    let mut town = Town::begin(&Layout::pier(), "anchored-walker-pier");
    let (result, _) = town.submit(10, "gus", sit("pier-bench.1"));
    assert!(accepted(&result), "{result:?}");
    // hal stands 900 mm north of the bench and walks 1 700 mm south through it.
    stopped_short_of(&mut town, 20, "hal", (3_000, 2_200), "gus");
}

#[test]
fn an_arrival_onto_a_seated_person_lands_at_the_nearest_free_point() {
    // AC-b-2: ivy crosses from the street onto bench.1, where alice sits; only nudging alice would
    // free it, so ivy is placed beside her instead.
    let mut town = Town::begin(&cafe_shaped(), "anchored-entry");
    let cafe = town.place("cafe");
    let (result, _) = town.submit(10, "alice", sit("bench.1"));
    assert!(accepted(&result), "{result:?}");
    let seated_at = town.location("alice");
    let (result, facts) = town.submit(20, "ivy", record(&Move::new(at(cafe, (2_000, 1_000)))));
    assert!(accepted(&result), "{result:?}");
    let ended = town.location("ivy");
    println!("ivy → {ended:?}; facts {:?}", types(&facts));
    assert_eq!(ended.place(), cafe, "ivy entered the café");
    assert!(
        distance2(&ended, &seated_at) >= CLEAR * CLEAR,
        "{} mm",
        distance2(&ended, &seated_at).isqrt()
    );
    assert_eq!(town.location("alice"), seated_at);
    assert!(town.posture("alice").is_some());
}

/// The observer's shove offer to `target`.
fn shove_offer(town: &Town, observer: &str, target: &str, when: i64) -> Option<Rejection> {
    let target = town.id(target);
    town.observation(observer, when)
        .affordances()
        .iter()
        .find(|offer| offer.action_type().as_str() == "shove" && offer.target() == Some(target))
        .unwrap_or_else(|| panic!("{observer} is offered a shove"))
        .unavailable_reason()
        .cloned()
}

#[test]
fn a_seated_person_cannot_be_shoved_and_can_once_they_stand() {
    // AC-b-3, on the pier and in the café.
    for (layout, name, sitter, point, shover) in [
        (
            Layout::pier(),
            "anchored-shove-pier",
            "gus",
            "pier-bench.1",
            "hal",
        ),
        (
            cafe_shaped(),
            "anchored-shove-cafe",
            "alice",
            "bench.1",
            "dave",
        ),
    ] {
        let mut town = Town::begin(&layout, name);
        let (result, _) = town.submit(10, sitter, sit(point));
        assert!(accepted(&result), "{result:?}");
        let seated_at = town.location(sitter);
        assert_eq!(
            shove_offer(&town, shover, sitter, 20),
            Some(Rejection::TargetUnavailable)
        );
        let (result, facts) = town.submit_to(20, shover, Some(sitter), record(&Shove {}));
        assert_eq!(
            result,
            ActionResult::Rejected(Rejection::TargetUnavailable),
            "{name}"
        );
        assert!(facts.is_empty());
        assert_eq!(town.location(sitter), seated_at);

        let (result, _) = town.submit(30, sitter, stand());
        assert!(accepted(&result), "{result:?}");
        assert_eq!(shove_offer(&town, shover, sitter, 40), None);
        let (result, facts) = town.submit_to(40, shover, Some(sitter), record(&Shove {}));
        assert!(accepted(&result), "{name}: {result:?}");
        assert!(types(&facts).contains(&"person-shoved".to_owned()));
    }
}

/// A pier with a 1 000 × 1 000 solid at (4 000, 4 000)–(5 000, 5 000), and one sit point at `point`.
fn pier_with_solid(point: (i32, i32), bodies_first: bool) -> Layout {
    let text = format!(
        "tags: [pier]\nbody:\n  floor:\n    min: {{ x: 0, y: 0 }}\n    max: {{ x: 10000, y: 10000 }}\n  \
         solids:\n    - {{ min: {{ x: 4000, y: 4000 }}, max: {{ x: 5000, y: 5000 }}, height: 1000 }}\n\
         sit-points:\n  - {{ key: kiosk.1, at: {{ x: {}, y: {} }}, facing: 0 }}\n",
        point.0, point.1
    );
    let systems = if bodies_first {
        vec!["presence", "movement", "bodies", "posture"]
    } else {
        vec!["presence", "movement", "posture", "bodies"]
    };
    Layout {
        systems,
        places: vec![("pier", Box::leak(text.into_boxed_str()))],
        people: &[("gus", "pier", 1_000, 1_000)],
        posture: None,
    }
}

/// Genesis of `layout`: `Ok` when it begins, the error's text when it is refused.
fn genesis(layout: &Layout, name: &str) -> Result<(), String> {
    let scratch = layout.write(name);
    let AssembledWorld {
        mut world, facts, ..
    } = WorldPack::read(scratch.path())
        .and_then(|pack| pack.assemble())
        .map_err(|error| error.to_string())?;
    world
        .genesis(GENESIS, facts)
        .map(|_| ())
        .map_err(|error| format!("{error:?}"))
}

#[test]
fn a_sit_point_where_a_person_does_not_fit_is_refused_at_load_in_either_order() {
    // AC-b-4. 100 mm west of the solid; 40 mm inside the floor; exactly 260 mm clear.
    for bodies_first in [true, false] {
        let order = if bodies_first {
            "bodies-first"
        } else {
            "posture-first"
        };
        let near = genesis(
            &pier_with_solid((3_900, 4_500), bodies_first),
            &format!("fixed-near-{order}"),
        )
        .expect_err("a sit point 100 mm from a solid is refused");
        println!("{order}: {near}");
        for word in [
            "bodies-fixed-point-in-solid",
            "pier",
            "kiosk.1",
            "(3900, 4500)",
            "solid 0",
        ] {
            assert!(near.contains(word), "{order} names {word}: {near}");
        }
        let outside = genesis(
            &pier_with_solid((40, 4_500), bodies_first),
            &format!("fixed-outside-{order}"),
        )
        .expect_err("a sit point 40 mm inside the floor's edge is refused");
        for word in [
            "bodies-fixed-point-in-solid",
            "kiosk.1",
            "outside the floor",
        ] {
            assert!(outside.contains(word), "{order} names {word}: {outside}");
        }
        genesis(
            &pier_with_solid((3_740, 4_500), bodies_first),
            &format!("fixed-clear-{order}"),
        )
        .unwrap_or_else(|error| panic!("{order}: a point exactly 260 mm clear loads: {error}"));
    }
}

#[test]
fn posture_without_bodies_sits_and_stands_and_skips_the_load_check() {
    // AC-b-7: the café of `support` enables posture but not bodies.
    let mut town = Town::begin(&Layout::cafe(), "anchored-no-bodies");
    let (result, _) = town.submit(10, "alice", sit("bench.1"));
    assert!(accepted(&result), "{result:?}");
    let (result, _) = town.submit(20, "alice", stand());
    assert!(accepted(&result), "{result:?}");
    assert_eq!(town.posture("alice"), None);
}

/// A seeded generator: xorshift64*, so the run is reproducible from its seed alone.
struct Seeded(u64);

impl Seeded {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

/// AC-b-6's generated variant: a 9 000 × 9 000 square, N sit points and N walkers on cells of a
/// 1 500 mm grid chosen by `seed`, and N / 2 sitters authored onto the first sit points.
/// The generated square: its layout, its sitters, its walkers and its sit points (`seat.<index>`).
struct Generated(Layout, Vec<String>, Vec<String>, Vec<(i32, i32)>);

fn generated(seed: u64, n: usize) -> Generated {
    let mut random = Seeded(seed);
    let mut cells: Vec<(i32, i32)> = (0..5)
        .flat_map(|i| (0..5).map(move |j| (1_500 + 1_500 * i, 1_500 + 1_500 * j)))
        .collect();
    for i in (1..cells.len()).rev() {
        let j = usize::try_from(random.below(i as u64 + 1)).expect("small");
        cells.swap(i, j);
    }
    let points = &cells[..n];
    let mut text = String::from(
        "tags: [square]\nbody:\n  floor:\n    min: { x: 0, y: 0 }\n    max: { x: 9000, y: 9000 }\n\
         sit-points:\n",
    );
    for (index, (x, y)) in points.iter().enumerate() {
        text.push_str(&format!(
            "  - {{ key: seat.{index}, at: {{ x: {x}, y: {y} }}, facing: 0 }}\n"
        ));
    }
    let mut people = Vec::new();
    let sitters: Vec<String> = (0..n / 2).map(|index| format!("sitter{index}")).collect();
    for (index, name) in sitters.iter().enumerate() {
        let (x, y) = points[index];
        people.push((leak(name), "square", x, y));
    }
    let walkers: Vec<String> = (0..n).map(|index| format!("walker{index}")).collect();
    for (index, name) in walkers.iter().enumerate() {
        let (x, y) = cells[n + index];
        people.push((leak(name), "square", x, y));
    }
    let layout = Layout {
        systems: vec!["presence", "movement", "bodies", "posture"],
        places: vec![("square", leak(&text))],
        people: Box::leak(people.into_boxed_slice()),
        posture: None,
    };
    Generated(layout, sitters, walkers, points.to_vec())
}

fn leak(text: &str) -> &'static str {
    Box::leak(text.to_owned().into_boxed_str())
}

#[test]
fn in_a_generated_square_no_walk_ever_moves_a_seated_person() {
    // AC-b-6 / I-S34-7: 2 000 requests of random strides, and an occasional shove of a sitter.
    const SEED: u64 = 0x5334_B0D1_E5A7_0001;
    const N: usize = 8;
    const REQUESTS: i64 = 2_000;
    let Generated(layout, sitters, walkers, points) = generated(SEED, N);
    let mut town = Town::begin(&layout, "anchored-generated");
    let square = town.place("square");
    for (index, sitter) in sitters.iter().enumerate() {
        let (result, _) = town.submit(1, sitter, sit(&format!("seat.{index}")));
        assert!(
            accepted(&result),
            "seed {SEED:#x}: {sitter} sits: {result:?}"
        );
    }
    let mut random = Seeded(SEED ^ 0xA5A5);
    let mut short = 0;
    for request in 0..REQUESTS {
        let when = 10 + request;
        let walker = &walkers[usize::try_from(random.below(N as u64)).expect("small")];
        if random.below(20) == 0 {
            let sitter =
                &sitters[usize::try_from(random.below(sitters.len() as u64)).expect("small")];
            town.submit_to(when, walker, Some(sitter), record(&Shove {}));
        } else {
            let here = town.location(walker).local().expect("a position");
            let step =
                |v: i32, r: u64| (v + i32::try_from(r).expect("small") - 1_400).clamp(300, 8_700);
            let target = (
                step(here.x().value(), random.below(2_801)),
                step(here.y().value(), random.below(2_801)),
            );
            let (_, facts) = town.submit(when, walker, record(&Move::new(at(square, target))));
            if types(&facts).contains(&"stopped-short".to_owned()) {
                short += 1;
            }
        }
        for (index, sitter) in sitters.iter().enumerate() {
            assert!(
                town.posture(sitter).is_some(),
                "seed {SEED:#x}, request {request}: {sitter} is still seated"
            );
            let local = town.location(sitter).local().expect("a position");
            assert_eq!(
                (local.x().value(), local.y().value()),
                points[index],
                "seed {SEED:#x}, request {request}: {sitter} is still on seat.{index}"
            );
        }
    }
    println!("seed {SEED:#x}: {short} strides of {REQUESTS} ended stopped short");
    assert!(short > 0, "seed {SEED:#x}: the walkers met somebody");
}
