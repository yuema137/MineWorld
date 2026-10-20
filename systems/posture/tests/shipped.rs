//! Sitting in the shipped towns (S34-c AC-c-4, AC-c-5, AC-c-6): a person walks to a café seat, sits, is
//! seen seated by somebody else, and walks away standing up; a seated person is still seated after the
//! town is saved and resumed from SQLite; and a seeded 300-request run keeps every seated person on their
//! point.
//!
//! The scenarios name the seat each world's design names (`window-stools.1` in Social Café, `table-3.1` in
//! Market Town). The sitter is whoever stands nearest it at genesis, read from the world, so no person or
//! coordinate is written here.

mod support;

use std::path::{Path, PathBuf};

use mineworld_bodies::Shove;
use mineworld_contracts::{
    ActionId, ActionIntent, ActionResult, EntityId, EntityType, Location, PersonId, PlaceId,
    WorldTime,
};
use mineworld_kernel::World;
use mineworld_movement::{Destination, Move, WalkStep, WalkTo, is_walking};
use mineworld_persistence::{Creation, Durability, PersistentWorld, SqliteBackend};
use mineworld_posture::{Posture, SitPoints};
use mineworld_presence::Presence;
use mineworld_worldpack::WorldPack;
use support::{GENESIS, Town, accepted, at, key, record, sit, stand, t, types};

fn world(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../worlds")
        .join(name)
}

fn point_of(world: &World, place: PlaceId, seat: &str) -> (i32, i32) {
    let read = world.read();
    let point = read
        .component::<SitPoints>(place.entity_id())
        .and_then(|points| points.get(&key(seat)))
        .unwrap_or_else(|| panic!("{seat} is a sit point of the place"));
    (point.at().x().value(), point.at().y().value())
}

/// The people standing in `place`, nearest `to` first.
fn nearest(world: &World, place: PlaceId, to: (i32, i32)) -> Vec<EntityId> {
    let read = world.read();
    let mut found: Vec<(i64, EntityId)> = read
        .components::<Presence>()
        .filter(|(_, presence)| presence.location().place() == place)
        .filter_map(|(entity, presence)| {
            let local = presence.location().local()?;
            let dx = i64::from(local.x().value() - to.0);
            let dy = i64::from(local.y().value() - to.1);
            Some((dx * dx + dy * dy, entity))
        })
        .collect();
    found.sort();
    found.into_iter().map(|(_, entity)| entity).collect()
}

fn person(entity: EntityId) -> PersonId {
    PersonId::new(entity, EntityType::Person).expect("a person")
}

/// Dispatches as a server would: advance to `when`, then the request.
fn dispatch(
    world: &mut World,
    next: &mut u64,
    when: WorldTime,
    actor: EntityId,
    target: Option<EntityId>,
    action: mineworld_contracts::ActionRecord,
) -> (ActionResult, Vec<String>) {
    let _ = world.advance_to(when).expect("the clock advances");
    *next += 1;
    let mut intent = ActionIntent::new(ActionId::from_raw(*next), actor, action, when);
    if let Some(target) = target {
        intent = intent.with_target(target);
    }
    let dispatched = world.dispatch(&intent, when).expect("dispatch answers");
    (dispatched.result().clone(), types(dispatched.events()))
}

/// Walks `walker` to `to` in `place`, step by step; the number of steps it took.
fn walk(
    world: &mut World,
    next: &mut u64,
    clock: &mut i64,
    walker: EntityId,
    place: PlaceId,
    to: (i32, i32),
) -> Vec<String> {
    *clock += 1;
    let (result, mut facts) = dispatch(
        world,
        next,
        t(*clock),
        walker,
        None,
        record(&WalkTo::new(Destination::Place(at(place, to)))),
    );
    assert!(accepted(&result), "the walk begins: {result:?}");
    for _ in 0..80 {
        if !is_walking(&world.read(), person(walker)) {
            return facts;
        }
        *clock += 1;
        let (_, more) = dispatch(world, next, t(*clock), walker, None, record(&WalkStep {}));
        facts.extend(more);
    }
    panic!("the walk to {to:?} ends within 80 steps");
}

fn seated_on(world: &World, entity: EntityId) -> Option<Posture> {
    world.read().component::<Posture>(entity).cloned()
}

fn location(world: &World, entity: EntityId) -> Location {
    world
        .read()
        .component::<Presence>(entity)
        .map(Presence::location)
        .expect("placed")
}

#[test]
fn in_each_town_a_person_walks_to_a_cafe_seat_sits_is_seen_seated_and_walks_away() {
    // AC-c-4.
    for (name, seat) in [
        ("social-cafe", "window-stools.1"),
        ("market-town", "table-3.1"),
    ] {
        let mut town = Town::open(&world(name));
        let cafe = town.place("cafe");
        let target = point_of(&town.world, cafe, seat);
        let people = nearest(&town.world, cafe, target);
        let (sitter, bystander) = (people[0], people[1]);
        let start = location(&town.world, sitter);
        let (mut next, mut clock) = (1_000, 10);
        walk(&mut town.world, &mut next, &mut clock, sitter, cafe, target);
        clock += 1;
        let (result, facts) = dispatch(
            &mut town.world,
            &mut next,
            t(clock),
            sitter,
            None,
            sit(seat),
        );
        assert!(accepted(&result), "{name}: sits on {seat}: {result:?}");
        assert!(facts.contains(&"sat-down".to_owned()), "{facts:?}");
        let seen = mineworld_presence::observe(
            &town.world,
            bystander,
            t(clock),
            &town.providers.iter().map(AsRef::as_ref).collect::<Vec<_>>(),
        );
        assert!(
            seen.entity(sitter).is_some_and(|entity| entity
                .components()
                .iter()
                .any(|row| row.component_type().as_str() == "posture-state")),
            "{name}: the bystander sees the sitter seated"
        );
        let back = start.local().expect("a position");
        let facts = walk(
            &mut town.world,
            &mut next,
            &mut clock,
            sitter,
            cafe,
            (back.x().value(), back.y().value()),
        );
        assert!(facts.contains(&"stood-up".to_owned()), "{name}: {facts:?}");
        assert_eq!(seated_on(&town.world, sitter), None);
    }
}

#[test]
fn a_person_seated_in_social_cafe_is_seated_after_a_restart() {
    // AC-c-5.
    let path = world("social-cafe");
    let assembled = WorldPack::read(&path)
        .and_then(|pack| pack.assemble())
        .expect("assembles");
    let cafe = *assembled
        .ids
        .get(&mineworld_contracts::EntityKey::new("cafe").expect("a key"))
        .expect("a café");
    let cafe = PlaceId::new(cafe, EntityType::Place).expect("a place");
    let save = mineworld_test_support::scratch!(empty "shipped-resume");
    let backend = SqliteBackend::create(&save, Durability::ProcessCrash).expect("creates");
    let (mut persisted, _) = PersistentWorld::create(
        Box::new(backend),
        assembled.world,
        Creation {
            instance: 0x5eed_0000_0000_0000_0000_0000_0034_0c05,
            pack: "social-cafe".to_owned(),
            at: GENESIS,
            facts: assembled.facts,
        },
    )
    .expect("a save is created");
    let target = point_of(persisted.world(), cafe, "window-stools.1");
    let sitter = nearest(persisted.world(), cafe, target)[0];
    let mut send = |id: u64, when: i64, action| {
        let intent = ActionIntent::new(ActionId::from_raw(id), sitter, action, t(when));
        persisted
            .dispatch(&intent, t(when))
            .expect("dispatches")
            .result()
            .clone()
    };
    let result = send(
        1,
        10,
        record(&WalkTo::new(Destination::Place(at(cafe, target)))),
    );
    assert!(accepted(&result), "{result:?}");
    for step in 0..80 {
        send(
            2 + step,
            11 + i64::try_from(step).expect("small"),
            record(&WalkStep {}),
        );
    }
    let result = send(200, 200, sit("window-stools.1"));
    assert!(accepted(&result), "{result:?}");
    let seated = seated_on(persisted.world(), sitter);
    assert!(seated.is_some());
    drop(persisted);

    let compose = || {
        WorldPack::read(&path)
            .and_then(|pack| pack.compose())
            .expect("composes")
            .world
    };
    let open = SqliteBackend::open(&save, Durability::ProcessCrash).expect("opens");
    let (resumed, _) = PersistentWorld::resume(Box::new(open), compose()).expect("resumes");
    assert_eq!(
        seated_on(resumed.world(), sitter),
        seated,
        "same point, same since"
    );
}

/// A seeded generator: xorshift64*.
struct Seeded(u64);

impl Seeded {
    fn below(&mut self, n: usize) -> usize {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        usize::try_from(self.0.wrapping_mul(0x2545_F491_4F6C_DD1D) % n as u64).expect("small")
    }
}

#[test]
fn a_seeded_run_of_each_town_keeps_every_seated_person_on_their_point() {
    // AC-c-6: I-S34-1, -2, -7 after every one of 300 requests in the café.
    const SEED: u64 = 0x5334_C006;
    for name in ["social-cafe", "market-town"] {
        let mut town = Town::open(&world(name));
        let cafe = town.place("cafe");
        let seats: Vec<(String, (i32, i32))> = town
            .world
            .read()
            .component::<SitPoints>(cafe.entity_id())
            .expect("the café has sit points")
            .iter()
            .map(|point| {
                let at = point.at();
                (
                    point.key().as_str().to_owned(),
                    (at.x().value(), at.y().value()),
                )
            })
            .collect();
        let people = nearest(&town.world, cafe, (0, 0));
        let mut random = Seeded(SEED);
        let (mut next, mut sat) = (0, 0);
        for request in 0..300_i64 {
            let when = t(10 + request);
            let actor = people[random.below(people.len())];
            let pick = random.below(10);
            let here = location(&town.world, actor).local().expect("a position");
            let (x, y) = (here.x().value(), here.y().value());
            let (seat, (sx, sy)) = seats
                .iter()
                .map(|(seat, (sx, sy))| {
                    let (dx, dy) = (i64::from(sx - x), i64::from(sy - y));
                    (dx * dx + dy * dy, seat, (*sx, *sy))
                })
                .min()
                .map(|(_, seat, at)| (seat.clone(), at))
                .expect("seats");
            let action = match pick {
                0..=2 => sit(&seat),
                3 => stand(),
                4 => record(&Shove {}),
                5..=7 => {
                    // Toward the nearest seat, at most 1 900 mm.
                    let (dx, dy) = (i64::from(sx - x), i64::from(sy - y));
                    let length = (dx * dx + dy * dy).isqrt().max(1);
                    let scale =
                        |d: i64| i32::try_from(d * length.min(1_900) / length).expect("small");
                    record(&Move::new(at(cafe, (x + scale(dx), y + scale(dy)))))
                }
                _ => {
                    let offset = |v: i32, r: usize| v + i32::try_from(r).expect("small") - 1_000;
                    record(&Move::new(at(
                        cafe,
                        (
                            offset(x, random.below(2_001)),
                            offset(y, random.below(2_001)),
                        ),
                    )))
                }
            };
            let target = (pick == 4).then(|| people[random.below(people.len())]);
            let (result, facts) = dispatch(&mut town.world, &mut next, when, actor, target, action);
            if facts.contains(&"sat-down".to_owned()) {
                sat += 1;
            }
            let _ = result;
            let read = town.world.read();
            for (entity, row) in read.components::<Posture>() {
                let here = read
                    .component::<Presence>(entity)
                    .map(Presence::location)
                    .expect("a seated person is somewhere");
                assert_eq!(
                    here.place(),
                    row.place(),
                    "{name} seed {SEED:#x} request {request}"
                );
                let point = read
                    .component::<SitPoints>(row.place().entity_id())
                    .and_then(|points| points.get(row.point()))
                    .expect("the row's point exists");
                assert_eq!(
                    here.local(),
                    Some(point.at()),
                    "{name} seed {SEED:#x} request {request}: still on {}",
                    row.point()
                );
            }
        }
        println!("{name}: {sat} sat down in 300 requests");
        assert!(sat > 0, "{name}: somebody sat down");
    }
}
